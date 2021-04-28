// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2021 BOTLabs GmbH

// The KILT Blockchain is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The KILT Blockchain is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

// If you feel like getting in touch with us, you can do so at info@botlabs.org

use crate::{self as delegation, mock::*};
use ctype::mock as ctype_mock;
use did::mock as did_mock;
use frame_support::{assert_err, assert_noop, assert_ok};
use sp_core::Pair;

use codec::Encode;

// submit_delegation_root_creation_operation()

#[test]
fn check_submit_delegation_root_creation_operation_successful() {
	let auth_key = did_mock::get_ed25519_authentication_key(true);
	let del_key = did_mock::get_sr25519_delegation_key(true);

	let mut delegator_details = did_mock::generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(del_key.public()), 0u64);

	let delegator_did = did_mock::ALICE_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);

	let operation = generate_base_delegation_root_creation_operation(root_id, root_node.clone());
	let signature = del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(delegator_did.clone(), delegator_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, delegator_did)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_ok!(Delegation::submit_delegation_root_creation_operation(
			Origin::signed(DEFAULT_ACCOUNT),
			operation.clone(),
			did::DidSignature::from(signature)
		));
	});

	let stored_delegation_root = ext
		.execute_with(|| Delegation::roots(&operation.root_id).expect("Delegation root should be present on chain."));

	assert_eq!(stored_delegation_root.ctype_hash, operation.ctype_hash);
	assert_eq!(stored_delegation_root.owner, operation.creator_did);
	assert!(!stored_delegation_root.revoked);

	// Verify that the DID tx counter has increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.creator_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value() + 1u64
	);
}

#[test]
fn check_did_not_found_submit_delegation_root_creation_operation() {
	let auth_key = did_mock::get_ed25519_authentication_key(true);
	let del_key = did_mock::get_sr25519_delegation_key(true);

	let mut delegator_details = did_mock::generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(del_key.public()), 0u64);

	let delegator_did = did_mock::ALICE_DID;
	let alternative_did = did_mock::BOB_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did),
	);

	let operation = generate_base_delegation_root_creation_operation(root_id, root_node);
	let signature = del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(alternative_did, delegator_details)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_root_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::DidNotPresent
		);
	});
}

#[test]
fn check_did_max_tx_counter_submit_delegation_root_creation_operation() {
	let auth_key = did_mock::get_ed25519_authentication_key(true);
	let del_key = did_mock::get_sr25519_delegation_key(true);

	let mut delegator_details = did_mock::generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(del_key.public()), 0u64);
	delegator_details.set_tx_counter(u64::MAX);

	let delegator_did = did_mock::ALICE_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);

	let operation = generate_base_delegation_root_creation_operation(root_id, root_node);
	let signature = del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(delegator_did.clone(), delegator_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(operation.ctype_hash, delegator_did)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_root_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::MaxTxCounterValue
		);
	});

	// Verify that the DID tx counter has not increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.creator_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value()
	);
}

#[test]
fn check_did_too_small_tx_counter_submit_delegation_root_creation_operation() {
	let auth_key = did_mock::get_ed25519_authentication_key(true);
	let del_key = did_mock::get_sr25519_delegation_key(true);

	let mut delegator_details = did_mock::generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(del_key.public()), 0u64);
	delegator_details.set_tx_counter(1u64);

	let delegator_did = did_mock::ALICE_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);

	let mut operation = generate_base_delegation_root_creation_operation(root_id, root_node);
	operation.tx_counter = delegator_details.get_tx_counter_value() - 1u64;
	let signature = del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(delegator_did.clone(), delegator_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(operation.ctype_hash, delegator_did)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_root_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidNonce
		);
	});

	// Verify that the DID tx counter has not increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.creator_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value()
	);
}

#[test]
fn check_did_equal_tx_counter_submit_delegation_root_creation_operation() {
	let auth_key = did_mock::get_ed25519_authentication_key(true);
	let del_key = did_mock::get_sr25519_delegation_key(true);

	let mut delegator_details = did_mock::generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(del_key.public()), 0u64);

	let delegator_did = did_mock::ALICE_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);

	let mut operation = generate_base_delegation_root_creation_operation(root_id, root_node);
	operation.tx_counter = delegator_details.get_tx_counter_value();
	let signature = del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(delegator_did.clone(), delegator_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(operation.ctype_hash, delegator_did)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_root_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidNonce
		);
	});

	// Verify that the DID tx counter has not increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.creator_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value()
	);
}

#[test]
fn check_did_too_large_tx_counter_submit_delegation_root_creation_operation() {
	let auth_key = did_mock::get_ed25519_authentication_key(true);
	let del_key = did_mock::get_sr25519_delegation_key(true);

	let mut delegator_details = did_mock::generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(del_key.public()), 0u64);

	let delegator_did = did_mock::ALICE_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);

	let mut operation = generate_base_delegation_root_creation_operation(root_id, root_node);
	operation.tx_counter = delegator_details.get_tx_counter_value() + 2u64;
	let signature = del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(delegator_did.clone(), delegator_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(operation.ctype_hash, delegator_did)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_root_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidNonce
		);
	});

	// Verify that the DID tx counter has not increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.creator_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value()
	);
}

#[test]
fn check_did_delegation_key_not_present_submit_delegation_root_creation_operation() {
	let auth_key = did_mock::get_ed25519_authentication_key(true);
	let del_key = did_mock::get_sr25519_delegation_key(true);

	let delegator_details = did_mock::generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));
	// No delegation key is added to the delegator details

	let delegator_did = did_mock::ALICE_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);

	let operation = generate_base_delegation_root_creation_operation(root_id, root_node);
	let signature = del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(delegator_did.clone(), delegator_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(operation.ctype_hash, delegator_did)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_root_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::VerificationKeyNotPresent
		);
	});

	// Verify that the DID tx counter has not increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.creator_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value()
	);
}

#[test]
fn check_did_invalid_signature_format_submit_delegation_root_creation_operation() {
	let auth_key = did_mock::get_ed25519_authentication_key(true);
	let del_key = did_mock::get_sr25519_delegation_key(true);
	let invalid_format_del_key = did_mock::get_ed25519_delegation_key(true);

	let mut delegator_details = did_mock::generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(del_key.public()), 0u64);

	let delegator_did = did_mock::ALICE_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);

	let operation = generate_base_delegation_root_creation_operation(root_id, root_node);
	let signature = invalid_format_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(delegator_did.clone(), delegator_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(operation.ctype_hash, delegator_did)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_root_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidSignatureFormat
		);
	});

	// Verify that the DID tx counter has not increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.creator_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value()
	);
}

#[test]
fn check_did_invalid_signature_submit_delegation_root_creation_operation() {
	let auth_key = did_mock::get_ed25519_authentication_key(true);
	let del_key = did_mock::get_sr25519_delegation_key(true);
	let invalid_del_key = did_mock::get_sr25519_delegation_key(false);

	let mut delegator_details = did_mock::generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(del_key.public()), 0u64);

	let delegator_did = did_mock::ALICE_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);

	let operation = generate_base_delegation_root_creation_operation(root_id, root_node);
	let signature = invalid_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(delegator_did.clone(), delegator_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(operation.ctype_hash, delegator_did)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_root_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidSignature
		);
	});

	// Verify that the DID tx counter has not increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.creator_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value()
	);
}

#[test]
fn check_duplicate_submit_delegation_root_creation_operation() {
	let auth_key = did_mock::get_ed25519_authentication_key(true);
	let del_key = did_mock::get_sr25519_delegation_key(true);

	let mut delegator_details = did_mock::generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(del_key.public()), 0u64);

	let delegator_did = did_mock::ALICE_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);

	let operation = generate_base_delegation_root_creation_operation(root_id, root_node.clone());
	let signature = del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(delegator_did.clone(), delegator_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(operation.ctype_hash, delegator_did)]);
	let builder = ExtBuilder::from(builder).with_root_delegations(vec![(root_id, root_node)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_err!(
			Delegation::submit_delegation_root_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			delegation::Error::<Test>::RootAlreadyExists
		);
	});

	// Verify that the DID tx counter has increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.creator_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value() + 1u64
	);
}

#[test]
fn check_ctype_not_found_submit_delegation_root_creation_operation() {
	let auth_key = did_mock::get_ed25519_authentication_key(true);
	let del_key = did_mock::get_sr25519_delegation_key(true);

	let mut delegator_details = did_mock::generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(del_key.public()), 0u64);

	let delegator_did = did_mock::ALICE_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);

	let operation = generate_base_delegation_root_creation_operation(root_id, root_node);
	let signature = del_key.sign(&operation.encode());

	// No CTYPE created in the builder
	let builder = did_mock::ExtBuilder::default().with_dids(vec![(delegator_did, delegator_details.clone())]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_err!(
			Delegation::submit_delegation_root_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			ctype::Error::<Test>::CTypeNotFound
		);
	});

	// Verify that the DID tx counter has increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.creator_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value() + 1u64
	);
}

// submit_delegation_creation_operation()

#[test]
fn check_submit_delegation_no_parent_creation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegator_auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(delegator_del_key.public()), 0u64);

	let delegate_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegate_auth_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let delegate_did = did_mock::BOB_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(root_id, delegate_did.clone()),
	);

	let delegate_signature = did::DidSignature::from(delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
		&delegation_id,
		&delegation_node.root_id,
		&delegation_node.parent,
		&delegation_node.permissions,
	))));

	let operation = generate_base_delegation_creation_operation(
		delegator_did.clone(),
		delegation_id,
		delegate_signature,
		delegation_node,
	);
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(delegator_did.clone(), delegator_details.clone()),
		(delegate_did, delegate_details),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, delegator_did)]);
	let builder = ExtBuilder::from(builder).with_root_delegations(vec![(root_id, root_node)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_ok!(Delegation::submit_delegation_creation_operation(
			Origin::signed(DEFAULT_ACCOUNT),
			operation.clone(),
			did::DidSignature::from(signature)
		));
	});

	let stored_delegation = ext.execute_with(|| {
		Delegation::delegations(&operation.delegation_id).expect("Delegation should be present on chain.")
	});

	assert_eq!(stored_delegation.root_id, operation.root_id);
	assert_eq!(stored_delegation.parent, operation.parent_id);
	assert_eq!(stored_delegation.owner, operation.delegate_did);
	assert_eq!(stored_delegation.permissions, operation.permissions);
	assert!(!stored_delegation.revoked);

	// Verify that the root has the new delegation among its children
	let stored_root_children = ext.execute_with(|| {
		Delegation::children(&operation.root_id).expect("Delegation root children should be present on chain.")
	});

	assert_eq!(stored_root_children, vec![operation.delegation_id]);

	// Verify that the DID tx counter has increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.creator_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value() + 1u64
	);
}

#[test]
fn check_submit_delegation_with_parent_creation_operation_successful() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegator_auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(delegator_del_key.public()), 0u64);

	let delegate_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegate_auth_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let delegate_did = did_mock::BOB_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);
	let (parent_delegation_id, parent_delegation_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(root_id, delegator_did.clone()),
	);
	let (delegation_id, mut delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(root_id, delegate_did.clone()),
	);
	delegation_node.parent = Some(parent_delegation_id);

	let delegate_signature = did::DidSignature::from(delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
		&delegation_id,
		&delegation_node.root_id,
		&delegation_node.parent,
		&delegation_node.permissions,
	))));

	let operation = generate_base_delegation_creation_operation(
		delegator_did.clone(),
		delegation_id,
		delegate_signature,
		delegation_node,
	);
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(delegator_did.clone(), delegator_details.clone()),
		(delegate_did, delegate_details),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, delegator_did)]);
	let builder = ExtBuilder::from(builder)
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![(parent_delegation_id, parent_delegation_node)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_ok!(Delegation::submit_delegation_creation_operation(
			Origin::signed(DEFAULT_ACCOUNT),
			operation.clone(),
			did::DidSignature::from(signature)
		));
	});

	let stored_delegation = ext.execute_with(|| {
		Delegation::delegations(&operation.delegation_id).expect("Delegation should be present on chain.")
	});

	assert_eq!(stored_delegation.root_id, operation.root_id);
	assert_eq!(stored_delegation.parent, operation.parent_id);
	assert_eq!(stored_delegation.owner, operation.delegate_did);
	assert_eq!(stored_delegation.permissions, operation.permissions);
	assert!(!stored_delegation.revoked);

	// Verify that the parent has the new delegation among its children
	let stored_parent_children = ext.execute_with(|| {
		Delegation::children(&operation.parent_id.unwrap())
			.expect("Delegation parent children should be present on chain.")
	});

	assert_eq!(stored_parent_children, vec![delegation_id]);

	// Verify that the DID tx counter has increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.creator_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value() + 1u64
	);
}

#[test]
fn check_delegator_did_not_found_submit_delegation_creation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegator_auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(delegator_del_key.public()), 0u64);

	let delegate_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegate_auth_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let alternative_did = did_mock::CHARLIE_DID;
	let delegate_did = did_mock::BOB_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(alternative_did.clone()),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(root_id, delegate_did.clone()),
	);

	let delegate_signature = did::DidSignature::from(delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
		&delegation_id,
		&delegation_node.root_id,
		&delegation_node.parent,
		&delegation_node.permissions,
	))));

	let operation =
		generate_base_delegation_creation_operation(delegator_did, delegation_id, delegate_signature, delegation_node);
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(alternative_did.clone(), delegator_details),
		(delegate_did, delegate_details),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, alternative_did)]);
	let builder = ExtBuilder::from(builder).with_root_delegations(vec![(root_id, root_node)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::DidNotPresent
		);
	});
}

#[test]
fn check_delegator_max_tx_counter_value_submit_delegation_creation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegator_auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(delegator_del_key.public()), 0u64);
	delegator_details.set_tx_counter(u64::MAX);

	let delegate_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegate_auth_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let delegate_did = did_mock::BOB_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(root_id, delegate_did.clone()),
	);

	let delegate_signature = did::DidSignature::from(delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
		&delegation_id,
		&delegation_node.root_id,
		&delegation_node.parent,
		&delegation_node.permissions,
	))));

	let operation = generate_base_delegation_creation_operation(
		delegator_did.clone(),
		delegation_id,
		delegate_signature,
		delegation_node,
	);
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(delegator_did.clone(), delegator_details.clone()),
		(delegate_did, delegate_details),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, delegator_did)]);
	let builder = ExtBuilder::from(builder).with_root_delegations(vec![(root_id, root_node)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::MaxTxCounterValue
		);
	});

	// Verify that the DID tx counter has not increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.creator_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value()
	);
}

#[test]
fn check_delegator_too_small_tx_counter_submit_delegation_creation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegator_auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(delegator_del_key.public()), 0u64);
	delegator_details.set_tx_counter(1u64);

	let delegate_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegate_auth_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let delegate_did = did_mock::BOB_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(root_id, delegate_did.clone()),
	);

	let delegate_signature = did::DidSignature::from(delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
		&delegation_id,
		&delegation_node.root_id,
		&delegation_node.parent,
		&delegation_node.permissions,
	))));

	let mut operation = generate_base_delegation_creation_operation(
		delegator_did.clone(),
		delegation_id,
		delegate_signature,
		delegation_node,
	);
	operation.tx_counter = delegator_details.get_tx_counter_value() - 1u64;
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(delegator_did.clone(), delegator_details.clone()),
		(delegate_did, delegate_details),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, delegator_did)]);
	let builder = ExtBuilder::from(builder).with_root_delegations(vec![(root_id, root_node)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidNonce
		);
	});

	// Verify that the DID tx counter has not increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.creator_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value()
	);
}

#[test]
fn check_delegator_equal_tx_counter_submit_delegation_creation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegator_auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(delegator_del_key.public()), 0u64);

	let delegate_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegate_auth_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let delegate_did = did_mock::BOB_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(root_id, delegate_did.clone()),
	);

	let delegate_signature = did::DidSignature::from(delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
		&delegation_id,
		&delegation_node.root_id,
		&delegation_node.parent,
		&delegation_node.permissions,
	))));

	let mut operation = generate_base_delegation_creation_operation(
		delegator_did.clone(),
		delegation_id,
		delegate_signature,
		delegation_node,
	);
	operation.tx_counter = delegator_details.get_tx_counter_value();
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(delegator_did.clone(), delegator_details.clone()),
		(delegate_did, delegate_details),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, delegator_did)]);
	let builder = ExtBuilder::from(builder).with_root_delegations(vec![(root_id, root_node)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidNonce
		);
	});

	// Verify that the DID tx counter has not increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.creator_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value()
	);
}

#[test]
fn check_delegator_too_large_tx_counter_submit_delegation_creation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegator_auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(delegator_del_key.public()), 0u64);

	let delegate_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegate_auth_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let delegate_did = did_mock::BOB_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(root_id, delegate_did.clone()),
	);

	let delegate_signature = did::DidSignature::from(delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
		&delegation_id,
		&delegation_node.root_id,
		&delegation_node.parent,
		&delegation_node.permissions,
	))));

	let mut operation = generate_base_delegation_creation_operation(
		delegator_did.clone(),
		delegation_id,
		delegate_signature,
		delegation_node,
	);
	operation.tx_counter = delegator_details.get_tx_counter_value() + 2u64;
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(delegator_did.clone(), delegator_details.clone()),
		(delegate_did, delegate_details),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, delegator_did)]);
	let builder = ExtBuilder::from(builder).with_root_delegations(vec![(root_id, root_node)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidNonce
		);
	});

	// Verify that the DID tx counter has not increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.creator_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value()
	);
}

#[test]
fn check_delegator_delegation_key_not_present_submit_delegation_creation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

	let delegator_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegator_auth_key.public()));
	// No delegation key specified

	let delegate_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegate_auth_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let delegate_did = did_mock::BOB_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(root_id, delegate_did.clone()),
	);

	let delegate_signature = did::DidSignature::from(delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
		&delegation_id,
		&delegation_node.root_id,
		&delegation_node.parent,
		&delegation_node.permissions,
	))));

	let operation = generate_base_delegation_creation_operation(
		delegator_did.clone(),
		delegation_id,
		delegate_signature,
		delegation_node,
	);
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(delegator_did.clone(), delegator_details.clone()),
		(delegate_did, delegate_details),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, delegator_did)]);
	let builder = ExtBuilder::from(builder).with_root_delegations(vec![(root_id, root_node)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::VerificationKeyNotPresent
		);
	});

	// Verify that the DID tx counter has not increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.creator_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value()
	);
}

#[test]
fn check_delegator_invalid_signature_format_submit_delegation_creation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let alternative_del_key = did_mock::get_ed25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegator_auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(delegator_del_key.public()), 0u64);

	let delegate_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegate_auth_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let delegate_did = did_mock::BOB_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(root_id, delegate_did.clone()),
	);

	let delegate_signature = did::DidSignature::from(delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
		&delegation_id,
		&delegation_node.root_id,
		&delegation_node.parent,
		&delegation_node.permissions,
	))));

	let operation = generate_base_delegation_creation_operation(
		delegator_did.clone(),
		delegation_id,
		delegate_signature,
		delegation_node,
	);
	let signature = alternative_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(delegator_did.clone(), delegator_details.clone()),
		(delegate_did, delegate_details),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, delegator_did)]);
	let builder = ExtBuilder::from(builder).with_root_delegations(vec![(root_id, root_node)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidSignatureFormat
		);
	});

	// Verify that the DID tx counter has not increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.creator_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value()
	);
}

#[test]
fn check_delegator_invalid_signature_submit_delegation_creation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let alternative_del_key = did_mock::get_sr25519_delegation_key(false);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegator_auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(delegator_del_key.public()), 0u64);

	let delegate_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegate_auth_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let delegate_did = did_mock::BOB_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(root_id, delegate_did.clone()),
	);

	let delegate_signature = did::DidSignature::from(delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
		&delegation_id,
		&delegation_node.root_id,
		&delegation_node.parent,
		&delegation_node.permissions,
	))));

	let operation = generate_base_delegation_creation_operation(
		delegator_did.clone(),
		delegation_id,
		delegate_signature,
		delegation_node,
	);
	let signature = alternative_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(delegator_did.clone(), delegator_details.clone()),
		(delegate_did, delegate_details),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, delegator_did)]);
	let builder = ExtBuilder::from(builder).with_root_delegations(vec![(root_id, root_node)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidSignature
		);
	});

	// Verify that the DID tx counter has not increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.creator_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value()
	);
}

#[test]
fn check_delegate_did_not_found_submit_delegation_creation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let alternative_del_key = did_mock::get_sr25519_delegation_key(false);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegator_auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(delegator_del_key.public()), 0u64);

	let delegator_did = did_mock::ALICE_DID;
	let delegate_did = did_mock::BOB_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(root_id, delegate_did),
	);

	let delegate_signature = did::DidSignature::from(delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
		&delegation_id,
		&delegation_node.root_id,
		&delegation_node.parent,
		&delegation_node.permissions,
	))));

	let operation = generate_base_delegation_creation_operation(
		delegator_did.clone(),
		delegation_id,
		delegate_signature,
		delegation_node,
	);
	let signature = alternative_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(delegator_did.clone(), delegator_details.clone()),
		// No delegate DID stored
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, delegator_did)]);
	let builder = ExtBuilder::from(builder).with_root_delegations(vec![(root_id, root_node)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidSignature
		);
	});

	// Verify that the DID tx counter has not increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.creator_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value()
	);
}

#[test]
fn check_invalid_delegate_signature_submit_delegation_creation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);
	let alternative_auth_key = did_mock::get_sr25519_attestation_key(false);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegator_auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(delegator_del_key.public()), 0u64);

	let delegate_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegate_auth_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let delegate_did = did_mock::BOB_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(root_id, delegate_did.clone()),
	);

	let delegate_signature =
		did::DidSignature::from(alternative_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
			&delegation_id,
			&delegation_node.root_id,
			&delegation_node.parent,
			&delegation_node.permissions,
		))));

	let operation = generate_base_delegation_creation_operation(
		delegator_did.clone(),
		delegation_id,
		delegate_signature,
		delegation_node,
	);
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(delegator_did.clone(), delegator_details.clone()),
		(delegate_did, delegate_details),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, delegator_did)]);
	let builder = ExtBuilder::from(builder).with_root_delegations(vec![(root_id, root_node)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_err!(
			Delegation::submit_delegation_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			delegation::Error::<Test>::InvalidDelegateSignature
		);
	});

	// Verify that the DID tx counter has increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.creator_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value() + 1u64
	);
}

#[test]
fn check_duplicate_delegation_submit_delegation_creation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegator_auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(delegator_del_key.public()), 0u64);

	let delegate_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegate_auth_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let delegate_did = did_mock::BOB_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(root_id, delegate_did.clone()),
	);

	let delegate_signature = did::DidSignature::from(delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
		&delegation_id,
		&delegation_node.root_id,
		&delegation_node.parent,
		&delegation_node.permissions,
	))));

	let operation = generate_base_delegation_creation_operation(
		delegator_did.clone(),
		delegation_id,
		delegate_signature,
		delegation_node.clone(),
	);
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(delegator_did.clone(), delegator_details.clone()),
		(delegate_did, delegate_details),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, delegator_did)]);
	let builder = ExtBuilder::from(builder)
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![(delegation_id, delegation_node)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_err!(
			Delegation::submit_delegation_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			delegation::Error::<Test>::DelegationAlreadyExists
		);
	});

	// Verify that the DID tx counter has increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.creator_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value() + 1u64
	);
}

#[test]
fn check_root_not_existing_submit_delegation_creation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegator_auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(delegator_del_key.public()), 0u64);

	let delegate_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegate_auth_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let delegate_did = did_mock::BOB_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);
	let alternative_root_id = get_delegation_root_id(false);
	let (delegation_id, delegation_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(root_id, delegate_did.clone()),
	);

	let delegate_signature = did::DidSignature::from(delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
		&delegation_id,
		&alternative_root_id,
		&delegation_node.parent,
		&delegation_node.permissions,
	))));

	let mut operation = generate_base_delegation_creation_operation(
		delegator_did.clone(),
		delegation_id,
		delegate_signature,
		delegation_node,
	);
	operation.root_id = alternative_root_id;
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(delegator_did.clone(), delegator_details.clone()),
		(delegate_did, delegate_details),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, delegator_did)]);
	let builder = ExtBuilder::from(builder).with_root_delegations(vec![(root_id, root_node)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_err!(
			Delegation::submit_delegation_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			delegation::Error::<Test>::RootNotFound
		);
	});

	// Verify that the DID tx counter has increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.creator_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value() + 1u64
	);
}

#[test]
fn check_parent_not_existing_submit_delegation_creation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegator_auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(delegator_del_key.public()), 0u64);

	let delegate_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegate_auth_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let delegate_did = did_mock::BOB_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);
	let alternative_parent_id = get_delegation_id(false);
	let (delegation_id, mut delegation_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(root_id, delegate_did.clone()),
	);
	delegation_node.parent = Some(alternative_parent_id);

	let delegate_signature = did::DidSignature::from(delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
		&delegation_id,
		&delegation_node.root_id,
		&delegation_node.parent,
		&delegation_node.permissions,
	))));

	let operation = generate_base_delegation_creation_operation(
		delegator_did.clone(),
		delegation_id,
		delegate_signature,
		delegation_node,
	);
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(delegator_did.clone(), delegator_details.clone()),
		(delegate_did, delegate_details),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, delegator_did)]);
	let builder = ExtBuilder::from(builder).with_root_delegations(vec![(root_id, root_node)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_err!(
			Delegation::submit_delegation_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			delegation::Error::<Test>::ParentDelegationNotFound
		);
	});

	// Verify that the DID tx counter has increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.creator_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value() + 1u64
	);
}

#[test]
fn check_not_owner_of_parent_submit_delegation_creation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegator_auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(delegator_del_key.public()), 0u64);

	let delegate_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegate_auth_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let alternative_did = did_mock::CHARLIE_DID;
	let delegate_did = did_mock::BOB_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);
	let (parent_delegation_id, parent_delegation_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(root_id, alternative_did.clone()),
	);
	let (delegation_id, mut delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(root_id, delegate_did.clone()),
	);
	delegation_node.parent = Some(parent_delegation_id);

	let delegate_signature = did::DidSignature::from(delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
		&delegation_id,
		&delegation_node.root_id,
		&delegation_node.parent,
		&delegation_node.permissions,
	))));

	let operation = generate_base_delegation_creation_operation(
		delegator_did.clone(),
		delegation_id,
		delegate_signature,
		delegation_node,
	);
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(delegator_did.clone(), delegator_details.clone()),
		(alternative_did, delegator_details.clone()),
		(delegate_did, delegate_details),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, delegator_did)]);
	let builder = ExtBuilder::from(builder)
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![(parent_delegation_id, parent_delegation_node)])
		.with_children(vec![(root_id, vec![parent_delegation_id])]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_err!(
			Delegation::submit_delegation_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			delegation::Error::<Test>::NotOwnerOfParentDelegation
		);
	});

	// Verify that the DID tx counter has increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.creator_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value() + 1u64
	);
}

#[test]
fn check_unauthorised_delegation_submit_delegation_creation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegator_auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(delegator_del_key.public()), 0u64);

	let delegate_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegate_auth_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let delegate_did = did_mock::BOB_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);
	let (parent_delegation_id, mut parent_delegation_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(root_id, delegator_did.clone()),
	);
	parent_delegation_node.permissions = delegation::Permissions::ATTEST;
	let (delegation_id, mut delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(root_id, delegate_did.clone()),
	);
	delegation_node.parent = Some(parent_delegation_id);

	let delegate_signature = did::DidSignature::from(delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
		&delegation_id,
		&delegation_node.root_id,
		&delegation_node.parent,
		&delegation_node.permissions,
	))));

	let operation = generate_base_delegation_creation_operation(
		delegator_did.clone(),
		delegation_id,
		delegate_signature,
		delegation_node,
	);
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(delegator_did.clone(), delegator_details.clone()),
		(delegate_did, delegate_details),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, delegator_did)]);
	let builder = ExtBuilder::from(builder)
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![(parent_delegation_id, parent_delegation_node)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_err!(
			Delegation::submit_delegation_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			delegation::Error::<Test>::UnauthorizedDelegation
		);
	});

	// Verify that the DID tx counter has increased
	let new_delegator_details = ext
		.execute_with(|| Did::get_did(&operation.creator_did).expect("DID submitter // should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value() + 1u64
	);
}

#[test]
fn check_not_owner_of_root_delegation_submit_delegation_creation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegator_auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(delegator_del_key.public()), 0u64);

	let delegate_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegate_auth_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let alternative_did = did_mock::CHARLIE_DID;
	let delegate_did = did_mock::BOB_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(alternative_did.clone()),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(root_id, delegate_did.clone()),
	);

	let delegate_signature = did::DidSignature::from(delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
		&delegation_id,
		&delegation_node.root_id,
		&delegation_node.parent,
		&delegation_node.permissions,
	))));

	let operation = generate_base_delegation_creation_operation(
		delegator_did.clone(),
		delegation_id,
		delegate_signature,
		delegation_node,
	);
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(delegator_did.clone(), delegator_details.clone()),
		(alternative_did, delegator_details.clone()),
		(delegate_did, delegate_details),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, delegator_did)]);
	let builder = ExtBuilder::from(builder).with_root_delegations(vec![(root_id, root_node)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_err!(
			Delegation::submit_delegation_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			delegation::Error::<Test>::NotOwnerOfRootDelegation
		);
	});

	// Verify that the DID tx counter has increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.creator_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value() + 1u64
	);
}

// submit_delegation_root_revocation_operation()

#[test]
fn check_list_hierarchy_submit_delegation_root_revocation_operation_successful() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegator_auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(delegator_del_key.public()), 0u64);

	let delegate_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegate_auth_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let delegate_did = did_mock::BOB_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);
	let (parent_delegation_id, parent_delegation_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(root_id, delegator_did.clone()),
	);
	let (delegation_id, mut delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(root_id, delegate_did.clone()),
	);
	delegation_node.parent = Some(parent_delegation_id);

	let mut operation = generate_base_delegation_root_revocation_operation(root_id, root_node.clone());
	operation.max_children = 2u32;
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(delegator_did.clone(), delegator_details.clone()),
		(delegate_did, delegate_details),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, delegator_did)]);
	let builder = ExtBuilder::from(builder)
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![
			(parent_delegation_id, parent_delegation_node),
			(delegation_id, delegation_node),
		])
		.with_children(vec![
			// Root -> Parent -> Delegation
			(root_id, vec![parent_delegation_id]),
			(parent_delegation_id, vec![delegation_id]),
		]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_ok!(Delegation::submit_delegation_root_revocation_operation(
			Origin::signed(DEFAULT_ACCOUNT),
			operation.clone(),
			did::DidSignature::from(signature)
		));
	});

	let stored_delegation_root = ext
		.execute_with(|| Delegation::roots(&operation.root_id).expect("Delegation root should be present on chain."));
	assert!(stored_delegation_root.revoked);

	let stored_parent_delegation = ext.execute_with(|| {
		Delegation::delegations(&parent_delegation_id).expect("Parent delegation should be present on chain.")
	});
	assert!(stored_parent_delegation.revoked);

	let stored_delegation =
		ext.execute_with(|| Delegation::delegations(&delegation_id).expect("Delegation should be present on chain."));
	assert!(stored_delegation.revoked);

	// Verify that the DID tx counter has increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.revoker_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value() + 1u64
	);
}

#[test]
fn check_tree_hierarchy_submit_delegation_root_revocation_operation_successful() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegator_auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(delegator_del_key.public()), 0u64);

	let delegate_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegate_auth_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let delegate_did = did_mock::BOB_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);
	let (delegation_id_1, delegation_node_1) = (
		get_delegation_id(true),
		generate_base_delegation_node(root_id, delegator_did.clone()),
	);
	let (delegation_id_2, delegation_node_2) = (
		get_delegation_id(false),
		generate_base_delegation_node(root_id, delegate_did.clone()),
	);

	let mut operation = generate_base_delegation_root_revocation_operation(root_id, root_node.clone());
	operation.max_children = 2u32;
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(delegator_did.clone(), delegator_details.clone()),
		(delegate_did, delegate_details),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, delegator_did)]);
	let builder = ExtBuilder::from(builder)
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![
			(delegation_id_1, delegation_node_1),
			(delegation_id_2, delegation_node_2),
		])
		.with_children(vec![
			// Root -> Delegation 1 && Delegation 2
			(root_id, vec![delegation_id_1, delegation_id_2]),
		]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_ok!(Delegation::submit_delegation_root_revocation_operation(
			Origin::signed(DEFAULT_ACCOUNT),
			operation.clone(),
			did::DidSignature::from(signature)
		));
	});

	let stored_delegation_root = ext
		.execute_with(|| Delegation::roots(&operation.root_id).expect("Delegation root should be present on chain."));
	assert!(stored_delegation_root.revoked);

	let stored_delegation_1 = ext
		.execute_with(|| Delegation::delegations(&delegation_id_1).expect("Delegation 1 should be present on chain."));
	assert!(stored_delegation_1.revoked);

	let stored_delegation_2 = ext
		.execute_with(|| Delegation::delegations(&delegation_id_2).expect("Delegation 2 should be present on chain."));
	assert!(stored_delegation_2.revoked);

	// Verify that the DID tx counter has increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.revoker_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value() + 1u64
	);
}

#[test]
fn check_greater_max_revocations_submit_delegation_root_revocation_operation_successful() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegator_auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(delegator_del_key.public()), 0u64);

	let delegate_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegate_auth_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let delegate_did = did_mock::BOB_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);

	let (delegation_id, delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(root_id, delegate_did.clone()),
	);

	let mut operation = generate_base_delegation_root_revocation_operation(root_id, root_node.clone());
	operation.max_children = u32::MAX;
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(delegator_did.clone(), delegator_details.clone()),
		(delegate_did, delegate_details),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, delegator_did)]);
	let builder = ExtBuilder::from(builder)
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![(delegation_id, delegation_node)])
		.with_children(vec![
			// Root -> Delegation
			(root_id, vec![delegation_id]),
		]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_ok!(Delegation::submit_delegation_root_revocation_operation(
			Origin::signed(DEFAULT_ACCOUNT),
			operation.clone(),
			did::DidSignature::from(signature)
		));
	});

	let stored_delegation_root = ext
		.execute_with(|| Delegation::roots(&operation.root_id).expect("Delegation root should be present on chain."));
	assert!(stored_delegation_root.revoked);

	let stored_delegation =
		ext.execute_with(|| Delegation::delegations(&delegation_id).expect("Delegation should be present on chain."));
	assert!(stored_delegation.revoked);

	// Verify that the DID tx counter has increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.revoker_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value() + 1u64
	);
}

#[test]
fn check_delegator_did_not_present_submit_hierarchy_delegation_root_revocation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegator_auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(delegator_del_key.public()), 0u64);

	let delegate_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegate_auth_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let alternative_did = did_mock::CHARLIE_DID;
	let delegate_did = did_mock::BOB_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did),
	);

	let (delegation_id, delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(root_id, delegate_did.clone()),
	);

	let operation = generate_base_delegation_root_revocation_operation(root_id, root_node.clone());
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(alternative_did.clone(), delegator_details),
		(delegate_did, delegate_details),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, alternative_did)]);
	let builder = ExtBuilder::from(builder)
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![(delegation_id, delegation_node)])
		.with_children(vec![
			// Root -> Delegation
			(root_id, vec![delegation_id]),
		]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_root_revocation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::DidNotPresent
		);
	});
}

#[test]
fn check_max_did_tx_counter_submit_delegation_root_revocation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegator_auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(delegator_del_key.public()), 0u64);
	delegator_details.set_tx_counter(u64::MAX);

	let delegate_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegate_auth_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let delegate_did = did_mock::BOB_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);

	let (delegation_id, delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(root_id, delegate_did.clone()),
	);

	let operation = generate_base_delegation_root_revocation_operation(root_id, root_node.clone());
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(delegator_did.clone(), delegator_details.clone()),
		(delegate_did, delegate_details),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, delegator_did)]);
	let builder = ExtBuilder::from(builder)
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![(delegation_id, delegation_node)])
		.with_children(vec![
			// Root -> Delegation
			(root_id, vec![delegation_id]),
		]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_root_revocation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::MaxTxCounterValue
		);
	});

	// Verify that the DID tx counter has not increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.revoker_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value()
	);
}

#[test]
fn check_too_small_did_tx_counter_submit_delegation_root_revocation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegator_auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(delegator_del_key.public()), 0u64);
	delegator_details.set_tx_counter(1u64);

	let delegate_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegate_auth_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let delegate_did = did_mock::BOB_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);

	let (delegation_id, delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(root_id, delegate_did.clone()),
	);

	let mut operation = generate_base_delegation_root_revocation_operation(root_id, root_node.clone());
	operation.tx_counter = delegator_details.get_tx_counter_value() - 1u64;
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(delegator_did.clone(), delegator_details.clone()),
		(delegate_did, delegate_details),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, delegator_did)]);
	let builder = ExtBuilder::from(builder)
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![(delegation_id, delegation_node)])
		.with_children(vec![
			// Root -> Delegation
			(root_id, vec![delegation_id]),
		]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_root_revocation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidNonce
		);
	});

	// Verify that the DID tx counter has not increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.revoker_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value()
	);
}

#[test]
fn check_equal_did_tx_counter_submit_delegation_root_revocation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegator_auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(delegator_del_key.public()), 0u64);

	let delegate_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegate_auth_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let delegate_did = did_mock::BOB_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);

	let (delegation_id, delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(root_id, delegate_did.clone()),
	);

	let mut operation = generate_base_delegation_root_revocation_operation(root_id, root_node.clone());
	operation.tx_counter = delegator_details.get_tx_counter_value();
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(delegator_did.clone(), delegator_details.clone()),
		(delegate_did, delegate_details),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, delegator_did)]);
	let builder = ExtBuilder::from(builder)
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![(delegation_id, delegation_node)])
		.with_children(vec![
			// Root -> Delegation
			(root_id, vec![delegation_id]),
		]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_root_revocation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidNonce
		);
	});

	// Verify that the DID tx counter has not increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.revoker_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value()
	);
}

#[test]
fn check_too_large_did_tx_counter_submit_delegation_root_revocation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegator_auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(delegator_del_key.public()), 0u64);

	let delegate_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegate_auth_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let delegate_did = did_mock::BOB_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);

	let (delegation_id, delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(root_id, delegate_did.clone()),
	);

	let mut operation = generate_base_delegation_root_revocation_operation(root_id, root_node.clone());
	operation.tx_counter = delegator_details.get_tx_counter_value() + 2u64;
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(delegator_did.clone(), delegator_details.clone()),
		(delegate_did, delegate_details),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, delegator_did)]);
	let builder = ExtBuilder::from(builder)
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![(delegation_id, delegation_node)])
		.with_children(vec![
			// Root -> Delegation
			(root_id, vec![delegation_id]),
		]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_root_revocation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidNonce
		);
	});

	// Verify that the DID tx counter has not increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.revoker_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value()
	);
}

#[test]
fn check_delegation_key_not_present_submit_delegation_root_revocation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

	let delegator_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegator_auth_key.public()));
	// No delegation key added

	let delegate_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegate_auth_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let delegate_did = did_mock::BOB_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);

	let (delegation_id, delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(root_id, delegate_did.clone()),
	);

	let operation = generate_base_delegation_root_revocation_operation(root_id, root_node.clone());
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(delegator_did.clone(), delegator_details.clone()),
		(delegate_did, delegate_details),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, delegator_did)]);
	let builder = ExtBuilder::from(builder)
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![(delegation_id, delegation_node)])
		.with_children(vec![
			// Root -> Delegation
			(root_id, vec![delegation_id]),
		]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_root_revocation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::VerificationKeyNotPresent
		);
	});

	// Verify that the DID tx counter has not increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.revoker_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value()
	);
}

#[test]
fn check_invalid_signature_format_submit_delegation_root_revocation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let alternative_del_key = did_mock::get_ed25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegator_auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(delegator_del_key.public()), 0u64);

	let delegate_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegate_auth_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let delegate_did = did_mock::BOB_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);

	let (delegation_id, delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(root_id, delegate_did.clone()),
	);

	let operation = generate_base_delegation_root_revocation_operation(root_id, root_node.clone());
	let signature = alternative_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(delegator_did.clone(), delegator_details.clone()),
		(delegate_did, delegate_details),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, delegator_did)]);
	let builder = ExtBuilder::from(builder)
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![(delegation_id, delegation_node)])
		.with_children(vec![
			// Root -> Delegation
			(root_id, vec![delegation_id]),
		]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_root_revocation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidSignatureFormat
		);
	});

	// Verify that the DID tx counter has not increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.revoker_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value()
	);
}

#[test]
fn check_invalid_signature_submit_delegation_root_revocation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let alternative_del_key = did_mock::get_sr25519_delegation_key(false);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegator_auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(delegator_del_key.public()), 0u64);

	let delegate_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegate_auth_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let delegate_did = did_mock::BOB_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);

	let (delegation_id, delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(root_id, delegate_did.clone()),
	);

	let operation = generate_base_delegation_root_revocation_operation(root_id, root_node.clone());
	let signature = alternative_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(delegator_did.clone(), delegator_details.clone()),
		(delegate_did, delegate_details),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, delegator_did)]);
	let builder = ExtBuilder::from(builder)
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![(delegation_id, delegation_node)])
		.with_children(vec![
			// Root -> Delegation
			(root_id, vec![delegation_id]),
		]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_root_revocation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidSignature
		);
	});

	// Verify that the DID tx counter has not increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.revoker_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value()
	);
}

#[test]
fn check_root_not_found_submit_delegation_root_revocation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegator_auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(delegator_del_key.public()), 0u64);

	let delegate_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegate_auth_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let delegate_did = did_mock::BOB_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);
	let alternative_root_id = get_delegation_root_id(false);

	let (delegation_id, delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(root_id, delegate_did.clone()),
	);

	let operation = generate_base_delegation_root_revocation_operation(root_id, root_node.clone());
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(delegator_did.clone(), delegator_details.clone()),
		(delegate_did, delegate_details),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, delegator_did)]);
	let builder = ExtBuilder::from(builder)
		.with_root_delegations(vec![(alternative_root_id, root_node)])
		.with_delegations(vec![(delegation_id, delegation_node)])
		.with_children(vec![
			// Root -> Delegation
			(alternative_root_id, vec![delegation_id]),
		]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_err!(
			Delegation::submit_delegation_root_revocation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			delegation::Error::<Test>::RootNotFound
		);
	});

	// Verify that the DID tx counter has increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.revoker_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value() + 1u64
	);
}

#[test]
fn check_different_root_creator_submit_delegation_root_revocation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegator_auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(delegator_del_key.public()), 0u64);

	let delegate_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegate_auth_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let alternative_did = did_mock::CHARLIE_DID;
	let delegate_did = did_mock::BOB_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(alternative_did.clone()),
	);

	let (delegation_id, delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(root_id, delegate_did.clone()),
	);

	let mut operation = generate_base_delegation_root_revocation_operation(root_id, root_node.clone());
	operation.revoker_did = delegator_did.clone();
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(delegator_did.clone(), delegator_details.clone()),
		(alternative_did, delegator_details.clone()),
		(delegate_did, delegate_details),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, delegator_did)]);
	let builder = ExtBuilder::from(builder)
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![(delegation_id, delegation_node)])
		.with_children(vec![
			// Root -> Delegation
			(root_id, vec![delegation_id]),
		]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_err!(
			Delegation::submit_delegation_root_revocation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			delegation::Error::<Test>::UnauthorizedRevocation
		);
	});

	// Verify that the DID tx counter has increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.revoker_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value() + 1u64
	);
}

#[test]
fn check_too_small_max_revocations_submit_delegation_root_revocation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegator_auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(delegator_del_key.public()), 0u64);

	let delegate_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegate_auth_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let delegate_did = did_mock::BOB_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);

	let (delegation_id, delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(root_id, delegate_did.clone()),
	);

	let mut operation = generate_base_delegation_root_revocation_operation(root_id, root_node.clone());
	operation.max_children = 0u32;
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(delegator_did.clone(), delegator_details.clone()),
		(delegate_did, delegate_details),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, delegator_did)]);
	let builder = ExtBuilder::from(builder)
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![(delegation_id, delegation_node)])
		.with_children(vec![
			// Root -> Delegation
			(root_id, vec![delegation_id]),
		]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_err!(
			Delegation::submit_delegation_root_revocation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			delegation::Error::<Test>::ExceededRevocationBounds
		);
	});

	// Verify that the DID tx counter has increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.revoker_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value() + 1u64
	);
}

#[test]
fn check_exact_children_max_revocations_submit_delegation_root_revocation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegator_auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(delegator_del_key.public()), 0u64);

	let delegate_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegate_auth_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let delegate_did = did_mock::BOB_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);
	let (delegation_id_1, delegation_node_1) = (
		get_delegation_id(true),
		generate_base_delegation_node(root_id, delegate_did.clone()),
	);
	let (delegation_id_2, mut delegation_node_2) = (
		get_delegation_id(false),
		generate_base_delegation_node(root_id, delegate_did.clone()),
	);
	delegation_node_2.parent = Some(delegation_id_1);
	let (delegation_id_3, mut delegation_node_3) = (
		get_delegation_root_id(false),
		generate_base_delegation_node(root_id, delegate_did.clone()),
	);
	delegation_node_3.parent = Some(delegation_id_1);

	let mut operation = generate_base_delegation_root_revocation_operation(root_id, root_node.clone());
	operation.max_children = 2u32;
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(delegator_did.clone(), delegator_details.clone()),
		(delegate_did, delegate_details),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, delegator_did)]);
	let builder = ExtBuilder::from(builder)
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![
			(delegation_id_1, delegation_node_1),
			(delegation_id_2, delegation_node_2),
			(delegation_id_3, delegation_node_3),
		])
		.with_children(vec![
			// Root -> Delegation 1 -> Delegation 2 && Delegation 3
			(root_id, vec![delegation_id_1]),
			(delegation_id_1, vec![delegation_id_2, delegation_id_3]),
		]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_err!(
			Delegation::submit_delegation_root_revocation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			delegation::Error::<Test>::ExceededRevocationBounds
		);
	});

	let stored_delegation_root = ext
		.execute_with(|| Delegation::roots(&operation.root_id).expect("Delegation root should be present on chain."));
	assert!(!stored_delegation_root.revoked);

	let stored_delegation_1 = ext
		.execute_with(|| Delegation::delegations(&delegation_id_1).expect("Delegation 1 should be present on chain."));
	assert!(!stored_delegation_1.revoked);

	// Only this leaf should have been revoked as it is the first child of
	// delegation_1
	let stored_delegation_2 = ext
		.execute_with(|| Delegation::delegations(&delegation_id_2).expect("Delegation 2 should be present on chain."));
	assert!(stored_delegation_2.revoked);

	let stored_delegation_3 = ext
		.execute_with(|| Delegation::delegations(&delegation_id_3).expect("Delegation 3 should be present on chain."));
	assert!(!stored_delegation_3.revoked);

	// Verify that the DID tx counter has increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.revoker_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value() + 1u64
	);
}

// submit_delegation_revocation_operation()

#[test]
fn check_direct_owner_submit_delegation_revocation_operation_successful() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegator_auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(delegator_del_key.public()), 0u64);

	let delegate_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegate_auth_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let delegate_did = did_mock::BOB_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);
	let (parent_delegation_id, parent_delegation_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(root_id, delegator_did.clone()),
	);
	let (delegation_id, mut delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(root_id, delegate_did.clone()),
	);
	delegation_node.parent = Some(parent_delegation_id);

	let mut operation =
		generate_base_delegation_revocation_operation(parent_delegation_id, parent_delegation_node.clone());
	operation.max_revocations = 2u32;
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(delegator_did.clone(), delegator_details.clone()),
		(delegate_did, delegate_details),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, delegator_did)]);
	let builder = ExtBuilder::from(builder)
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![
			(parent_delegation_id, parent_delegation_node),
			(delegation_id, delegation_node),
		])
		.with_children(vec![(parent_delegation_id, vec![delegation_id])]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_ok!(Delegation::submit_delegation_revocation_operation(
			Origin::signed(DEFAULT_ACCOUNT),
			operation.clone(),
			did::DidSignature::from(signature)
		));
	});

	let stored_parent_delegation = ext.execute_with(|| {
		Delegation::delegations(&parent_delegation_id).expect("Parent delegation should be present on chain.")
	});
	assert!(stored_parent_delegation.revoked);

	let stored_child_delegation = ext.execute_with(|| {
		Delegation::delegations(&delegation_id).expect("Child delegation should be present on chain.")
	});
	assert!(stored_child_delegation.revoked);

	// Verify that the DID tx counter has increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.revoker_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value() + 1u64
	);
}

#[test]
fn check_parent_owner_submit_delegation_revocation_operation_successful() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegator_auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(delegator_del_key.public()), 0u64);

	let delegate_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegate_auth_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let delegate_did = did_mock::BOB_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);
	let (parent_delegation_id, parent_delegation_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(root_id, delegator_did.clone()),
	);
	let (delegation_id, mut delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(root_id, delegate_did.clone()),
	);
	delegation_node.parent = Some(parent_delegation_id);

	let mut operation = generate_base_delegation_revocation_operation(delegation_id, parent_delegation_node.clone());
	operation.max_parent_checks = 1u32;
	operation.max_revocations = 1u32;
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(delegator_did.clone(), delegator_details.clone()),
		(delegate_did, delegate_details),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, delegator_did)]);
	let builder = ExtBuilder::from(builder)
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![
			(parent_delegation_id, parent_delegation_node),
			(delegation_id, delegation_node),
		])
		.with_children(vec![(parent_delegation_id, vec![delegation_id])]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_ok!(Delegation::submit_delegation_revocation_operation(
			Origin::signed(DEFAULT_ACCOUNT),
			operation.clone(),
			did::DidSignature::from(signature)
		));
	});

	let stored_parent_delegation = ext.execute_with(|| {
		Delegation::delegations(&parent_delegation_id).expect("Parent delegation should be present on chain.")
	});
	assert!(!stored_parent_delegation.revoked);

	let stored_child_delegation = ext.execute_with(|| {
		Delegation::delegations(&delegation_id).expect("Child delegation should be present on chain.")
	});
	assert!(stored_child_delegation.revoked);

	// Verify that the DID tx counter has increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.revoker_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value() + 1u64
	);
}

#[test]
fn check_delegator_did_not_present_submit_delegation_revocation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegator_auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(delegator_del_key.public()), 0u64);

	let delegate_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegate_auth_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let alternative_did = did_mock::CHARLIE_DID;
	let delegate_did = did_mock::BOB_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(alternative_did.clone()),
	);
	let (parent_delegation_id, parent_delegation_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(root_id, delegator_did),
	);
	let (delegation_id, mut delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(root_id, delegate_did.clone()),
	);
	delegation_node.parent = Some(parent_delegation_id);

	let mut operation =
		generate_base_delegation_revocation_operation(parent_delegation_id, parent_delegation_node.clone());
	operation.max_revocations = 2u32;
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(alternative_did.clone(), delegator_details),
		(delegate_did, delegate_details),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, alternative_did)]);
	let builder = ExtBuilder::from(builder)
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![
			(parent_delegation_id, parent_delegation_node),
			(delegation_id, delegation_node),
		])
		.with_children(vec![(parent_delegation_id, vec![delegation_id])]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_revocation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::DidNotPresent
		);
	});
}

#[test]
fn check_did_max_tx_counter_submit_delegation_revocation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegator_auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(delegator_del_key.public()), 0u64);
	delegator_details.set_tx_counter(u64::MAX);

	let delegate_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegate_auth_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let delegate_did = did_mock::BOB_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);
	let (parent_delegation_id, parent_delegation_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(root_id, delegator_did.clone()),
	);
	let (delegation_id, mut delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(root_id, delegate_did.clone()),
	);
	delegation_node.parent = Some(parent_delegation_id);

	let mut operation =
		generate_base_delegation_revocation_operation(parent_delegation_id, parent_delegation_node.clone());
	operation.max_revocations = 2u32;
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(delegator_did.clone(), delegator_details.clone()),
		(delegate_did, delegate_details),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, delegator_did)]);
	let builder = ExtBuilder::from(builder)
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![
			(parent_delegation_id, parent_delegation_node),
			(delegation_id, delegation_node),
		])
		.with_children(vec![(parent_delegation_id, vec![delegation_id])]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_revocation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::MaxTxCounterValue
		);
	});

	// Verify that the DID tx counter has not increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.revoker_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value()
	);
}

#[test]
fn check_delegator_too_small_tx_counter_submit_delegation_revocation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegator_auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(delegator_del_key.public()), 0u64);
	delegator_details.set_tx_counter(1u64);

	let delegate_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegate_auth_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let delegate_did = did_mock::BOB_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);
	let (parent_delegation_id, parent_delegation_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(root_id, delegator_did.clone()),
	);
	let (delegation_id, mut delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(root_id, delegate_did.clone()),
	);
	delegation_node.parent = Some(parent_delegation_id);

	let mut operation =
		generate_base_delegation_revocation_operation(parent_delegation_id, parent_delegation_node.clone());
	operation.tx_counter = delegator_details.get_tx_counter_value() - 1u64;
	operation.max_revocations = 2u32;
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(delegator_did.clone(), delegator_details.clone()),
		(delegate_did, delegate_details),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, delegator_did)]);
	let builder = ExtBuilder::from(builder)
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![
			(parent_delegation_id, parent_delegation_node),
			(delegation_id, delegation_node),
		])
		.with_children(vec![(parent_delegation_id, vec![delegation_id])]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_revocation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidNonce
		);
	});

	// Verify that the DID tx counter has not increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.revoker_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value()
	);
}

#[test]
fn check_delegator_equal_tx_counter_submit_delegation_revocation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegator_auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(delegator_del_key.public()), 0u64);

	let delegate_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegate_auth_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let delegate_did = did_mock::BOB_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);
	let (parent_delegation_id, parent_delegation_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(root_id, delegator_did.clone()),
	);
	let (delegation_id, mut delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(root_id, delegate_did.clone()),
	);
	delegation_node.parent = Some(parent_delegation_id);

	let mut operation =
		generate_base_delegation_revocation_operation(parent_delegation_id, parent_delegation_node.clone());
	operation.tx_counter = delegator_details.get_tx_counter_value();
	operation.max_revocations = 2u32;
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(delegator_did.clone(), delegator_details.clone()),
		(delegate_did, delegate_details),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, delegator_did)]);
	let builder = ExtBuilder::from(builder)
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![
			(parent_delegation_id, parent_delegation_node),
			(delegation_id, delegation_node),
		])
		.with_children(vec![(parent_delegation_id, vec![delegation_id])]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_revocation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidNonce
		);
	});

	// Verify that the DID tx counter has not increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.revoker_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value()
	);
}

#[test]
fn check_delegator_too_large_tx_counter_submit_delegation_revocation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegator_auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(delegator_del_key.public()), 0u64);

	let delegate_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegate_auth_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let delegate_did = did_mock::BOB_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);
	let (parent_delegation_id, parent_delegation_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(root_id, delegator_did.clone()),
	);
	let (delegation_id, mut delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(root_id, delegate_did.clone()),
	);
	delegation_node.parent = Some(parent_delegation_id);

	let mut operation =
		generate_base_delegation_revocation_operation(parent_delegation_id, parent_delegation_node.clone());
	operation.tx_counter = delegator_details.get_tx_counter_value() + 2u64;
	operation.max_revocations = 2u32;
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(delegator_did.clone(), delegator_details.clone()),
		(delegate_did, delegate_details),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, delegator_did)]);
	let builder = ExtBuilder::from(builder)
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![
			(parent_delegation_id, parent_delegation_node),
			(delegation_id, delegation_node),
		])
		.with_children(vec![(parent_delegation_id, vec![delegation_id])]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_revocation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidNonce
		);
	});

	// Verify that the DID tx counter has not increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.revoker_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value()
	);
}

#[test]
fn check_delegator_delegation_key_not_present_submit_delegation_revocation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

	let delegator_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegator_auth_key.public()));
	// No delegation key specified

	let delegate_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegate_auth_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let delegate_did = did_mock::BOB_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);
	let (parent_delegation_id, parent_delegation_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(root_id, delegator_did.clone()),
	);
	let (delegation_id, mut delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(root_id, delegate_did.clone()),
	);
	delegation_node.parent = Some(parent_delegation_id);

	let mut operation =
		generate_base_delegation_revocation_operation(parent_delegation_id, parent_delegation_node.clone());
	operation.max_revocations = 2u32;
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(delegator_did.clone(), delegator_details.clone()),
		(delegate_did, delegate_details),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, delegator_did)]);
	let builder = ExtBuilder::from(builder)
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![
			(parent_delegation_id, parent_delegation_node),
			(delegation_id, delegation_node),
		])
		.with_children(vec![(parent_delegation_id, vec![delegation_id])]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_revocation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::VerificationKeyNotPresent
		);
	});

	// Verify that the DID tx counter has not increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.revoker_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value()
	);
}

#[test]
fn check_invalid_signature_format_submit_delegation_revocation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let alternative_del_key = did_mock::get_ed25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegator_auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(delegator_del_key.public()), 0u64);

	let delegate_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegate_auth_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let delegate_did = did_mock::BOB_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);
	let (parent_delegation_id, parent_delegation_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(root_id, delegator_did.clone()),
	);
	let (delegation_id, mut delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(root_id, delegate_did.clone()),
	);
	delegation_node.parent = Some(parent_delegation_id);

	let mut operation =
		generate_base_delegation_revocation_operation(parent_delegation_id, parent_delegation_node.clone());
	operation.max_revocations = 2u32;
	let signature = alternative_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(delegator_did.clone(), delegator_details.clone()),
		(delegate_did, delegate_details),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, delegator_did)]);
	let builder = ExtBuilder::from(builder)
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![
			(parent_delegation_id, parent_delegation_node),
			(delegation_id, delegation_node),
		])
		.with_children(vec![(parent_delegation_id, vec![delegation_id])]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_revocation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidSignatureFormat
		);
	});

	// Verify that the DID tx counter has not increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.revoker_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value()
	);
}

#[test]
fn check_invalid_signature_submit_delegation_revocation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let alternative_del_key = did_mock::get_sr25519_delegation_key(false);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegator_auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(delegator_del_key.public()), 0u64);

	let delegate_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegate_auth_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let delegate_did = did_mock::BOB_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);
	let (parent_delegation_id, parent_delegation_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(root_id, delegator_did.clone()),
	);
	let (delegation_id, mut delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(root_id, delegate_did.clone()),
	);
	delegation_node.parent = Some(parent_delegation_id);

	let mut operation =
		generate_base_delegation_revocation_operation(parent_delegation_id, parent_delegation_node.clone());
	operation.max_revocations = 2u32;
	let signature = alternative_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(delegator_did.clone(), delegator_details.clone()),
		(delegate_did, delegate_details),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, delegator_did)]);
	let builder = ExtBuilder::from(builder)
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![
			(parent_delegation_id, parent_delegation_node),
			(delegation_id, delegation_node),
		])
		.with_children(vec![(parent_delegation_id, vec![delegation_id])]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_revocation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidSignature
		);
	});

	// Verify that the DID tx counter has not increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.revoker_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value()
	);
}

#[test]
fn check_delegation_not_found_submit_delegation_revocation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegator_auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(delegator_del_key.public()), 0u64);

	let delegate_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegate_auth_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let delegate_did = did_mock::BOB_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);
	let (parent_delegation_id, parent_delegation_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(root_id, delegator_did.clone()),
	);
	let (delegation_id, mut delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(root_id, delegate_did.clone()),
	);
	let alternative_delegation_id = get_delegation_root_id(false);
	delegation_node.parent = Some(parent_delegation_id);

	let mut operation =
		generate_base_delegation_revocation_operation(parent_delegation_id, parent_delegation_node.clone());
	operation.max_revocations = 2u32;
	operation.delegation_id = alternative_delegation_id;
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(delegator_did.clone(), delegator_details.clone()),
		(delegate_did, delegate_details),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, delegator_did)]);
	let builder = ExtBuilder::from(builder)
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![
			(parent_delegation_id, parent_delegation_node),
			(delegation_id, delegation_node),
		])
		.with_children(vec![(parent_delegation_id, vec![delegation_id])]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_err!(
			Delegation::submit_delegation_revocation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			delegation::Error::<Test>::DelegationNotFound
		);
	});

	// Verify that the DID tx counter has increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.revoker_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value() + 1u64
	);
}

#[test]
fn check_not_delegating_submit_delegation_revocation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegator_auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(delegator_del_key.public()), 0u64);

	let delegate_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegate_auth_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let alternative_did = did_mock::CHARLIE_DID;
	let delegate_did = did_mock::BOB_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(alternative_did.clone()),
	);
	let (parent_delegation_id, parent_delegation_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(root_id, alternative_did.clone()),
	);
	let (delegation_id, mut delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(root_id, delegate_did.clone()),
	);
	delegation_node.parent = Some(parent_delegation_id);

	let mut operation =
		generate_base_delegation_revocation_operation(parent_delegation_id, parent_delegation_node.clone());
	operation.revoker_did = delegator_did.clone();
	operation.max_parent_checks = u32::MAX;
	operation.max_revocations = 2u32;
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(delegator_did, delegator_details.clone()),
		(alternative_did.clone(), delegator_details.clone()),
		(delegate_did, delegate_details),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, alternative_did)]);
	let builder = ExtBuilder::from(builder)
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![
			(parent_delegation_id, parent_delegation_node),
			(delegation_id, delegation_node),
		])
		.with_children(vec![(parent_delegation_id, vec![delegation_id])]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_err!(
			Delegation::submit_delegation_revocation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			delegation::Error::<Test>::UnauthorizedRevocation
		);
	});

	// Verify that the DID tx counter has increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.revoker_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value() + 1u64
	);
}

#[test]
fn check_parent_too_far_submit_delegation_revocation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegator_auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(delegator_del_key.public()), 0u64);

	let delegate_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegate_auth_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let alternative_did = did_mock::CHARLIE_DID;
	let delegate_did = did_mock::BOB_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(alternative_did.clone()),
	);
	let (parent_delegation_id, parent_delegation_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(root_id, alternative_did.clone()),
	);
	let (delegation_id, mut delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(root_id, delegate_did.clone()),
	);
	delegation_node.parent = Some(parent_delegation_id);

	let mut operation = generate_base_delegation_revocation_operation(delegation_id, delegation_node.clone());
	operation.revoker_did = delegator_did.clone();
	operation.max_parent_checks = 0u32;
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(delegator_did, delegator_details.clone()),
		(alternative_did.clone(), delegator_details.clone()),
		(delegate_did, delegate_details),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, alternative_did)]);
	let builder = ExtBuilder::from(builder)
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![
			(parent_delegation_id, parent_delegation_node),
			(delegation_id, delegation_node),
		])
		.with_children(vec![(parent_delegation_id, vec![delegation_id])]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_err!(
			Delegation::submit_delegation_revocation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			delegation::Error::<Test>::MaxSearchDepthReached
		);
	});

	// Verify that the DID tx counter has increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.revoker_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value() + 1u64
	);
}

#[test]
fn check_too_many_revocations_submit_delegation_revocation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegator_auth_key.public()));
	delegator_details.update_delegation_key(did::DidVerificationKey::from(delegator_del_key.public()), 0u64);

	let delegate_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(delegate_auth_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let delegate_did = did_mock::BOB_DID;
	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(delegator_did.clone()),
	);
	let (parent_delegation_id, parent_delegation_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(root_id, delegator_did.clone()),
	);
	let (delegation_id, mut delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(root_id, delegate_did.clone()),
	);
	delegation_node.parent = Some(parent_delegation_id);

	let mut operation = generate_base_delegation_revocation_operation(delegation_id, delegation_node.clone());
	operation.revoker_did = delegator_did.clone();
	operation.max_parent_checks = 1u32;
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(delegator_did.clone(), delegator_details.clone()),
		(delegate_did, delegate_details),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, delegator_did)]);
	let builder = ExtBuilder::from(builder)
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![
			(parent_delegation_id, parent_delegation_node),
			(delegation_id, delegation_node),
		])
		.with_children(vec![(parent_delegation_id, vec![delegation_id])]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_err!(
			Delegation::submit_delegation_revocation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			delegation::Error::<Test>::ExceededRevocationBounds
		);
	});

	// Verify that the DID tx counter has increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.revoker_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value() + 1u64
	);
}

// Internal function: is_actively_delegating()

#[test]
fn check_is_actively_delegating_direct_not_revoked() {
	let actor_did = did_mock::ALICE_DID;
	let alternative_did_1 = did_mock::BOB_DID;
	let alternative_did_2 = did_mock::CHARLIE_DID;

	let (root_id, root_node) = (get_delegation_root_id(true), generate_base_delegation_root(actor_did));
	let (delegation_id_1, delegation_node_1) = (
		get_delegation_id(true),
		generate_base_delegation_node(root_id, alternative_did_1),
	);
	let (delegation_id_2, mut delegation_node_2) = (
		get_delegation_id(false),
		generate_base_delegation_node(root_id, alternative_did_2.clone()),
	);
	delegation_node_2.parent = Some(delegation_id_1);
	let max_parent_checks = 0u32;

	// Root -> Delegation 1 -> Delegation 2
	let builder = ExtBuilder::default()
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![
			(delegation_id_1, delegation_node_1),
			(delegation_id_2, delegation_node_2),
		])
		.with_children(vec![
			(root_id, vec![delegation_id_1]),
			(delegation_id_1, vec![delegation_id_2]),
		]);

	let mut ext = builder.build();

	let is_actively_delegating =
		ext.execute_with(|| Delegation::is_delegating(&alternative_did_2, &delegation_id_2, max_parent_checks));
	assert_eq!(is_actively_delegating, Ok(true));
}

#[test]
fn check_is_actively_delegating_direct_not_revoked_max_parent_checks_value() {
	let actor_did = did_mock::ALICE_DID;
	let alternative_did_1 = did_mock::BOB_DID;
	let alternative_did_2 = did_mock::CHARLIE_DID;

	let (root_id, root_node) = (get_delegation_root_id(true), generate_base_delegation_root(actor_did));
	let (delegation_id_1, delegation_node_1) = (
		get_delegation_id(true),
		generate_base_delegation_node(root_id, alternative_did_1),
	);
	let (delegation_id_2, mut delegation_node_2) = (
		get_delegation_id(false),
		generate_base_delegation_node(root_id, alternative_did_2.clone()),
	);
	delegation_node_2.parent = Some(delegation_id_1);
	let max_parent_checks = u32::MAX;

	// Root -> Delegation 1 -> Delegation 2
	let builder = ExtBuilder::default()
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![
			(delegation_id_1, delegation_node_1),
			(delegation_id_2, delegation_node_2),
		])
		.with_children(vec![
			(root_id, vec![delegation_id_1]),
			(delegation_id_1, vec![delegation_id_2]),
		]);

	let mut ext = builder.build();

	let is_actively_delegating =
		ext.execute_with(|| Delegation::is_delegating(&alternative_did_2, &delegation_id_2, max_parent_checks));
	assert_eq!(is_actively_delegating, Ok(true));
}

#[test]
fn check_is_actively_delegating_direct_revoked() {
	let actor_did = did_mock::ALICE_DID;
	let alternative_did_1 = did_mock::BOB_DID;
	let alternative_did_2 = did_mock::CHARLIE_DID;

	let (root_id, root_node) = (get_delegation_root_id(true), generate_base_delegation_root(actor_did));
	let (delegation_id_1, delegation_node_1) = (
		get_delegation_id(true),
		generate_base_delegation_node(root_id, alternative_did_1),
	);
	let (delegation_id_2, mut delegation_node_2) = (
		get_delegation_id(false),
		generate_base_delegation_node(root_id, alternative_did_2.clone()),
	);
	delegation_node_2.parent = Some(delegation_id_1);
	delegation_node_2.revoked = true;
	let max_parent_checks = 0u32;

	// Root -> Delegation 1 -> Delegation 2
	let builder = ExtBuilder::default()
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![
			(delegation_id_1, delegation_node_1),
			(delegation_id_2, delegation_node_2),
		])
		.with_children(vec![
			(root_id, vec![delegation_id_1]),
			(delegation_id_1, vec![delegation_id_2]),
		]);

	let mut ext = builder.build();

	let is_actively_delegating =
		ext.execute_with(|| Delegation::is_delegating(&alternative_did_2, &delegation_id_2, max_parent_checks));
	assert_eq!(is_actively_delegating, Ok(false));
}

#[test]
fn check_is_actively_delegating_direct_revoked_max_parent_checks_value() {
	let actor_did = did_mock::ALICE_DID;
	let alternative_did_1 = did_mock::BOB_DID;
	let alternative_did_2 = did_mock::CHARLIE_DID;

	let (root_id, root_node) = (get_delegation_root_id(true), generate_base_delegation_root(actor_did));
	let (delegation_id_1, delegation_node_1) = (
		get_delegation_id(true),
		generate_base_delegation_node(root_id, alternative_did_1),
	);
	let (delegation_id_2, mut delegation_node_2) = (
		get_delegation_id(false),
		generate_base_delegation_node(root_id, alternative_did_2.clone()),
	);
	delegation_node_2.parent = Some(delegation_id_1);
	delegation_node_2.revoked = true;
	let max_parent_checks = u32::MAX;

	// Root -> Delegation 1 -> Delegation 2
	let builder = ExtBuilder::default()
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![
			(delegation_id_1, delegation_node_1),
			(delegation_id_2, delegation_node_2),
		])
		.with_children(vec![
			(root_id, vec![delegation_id_1]),
			(delegation_id_1, vec![delegation_id_2]),
		]);

	let mut ext = builder.build();

	let is_actively_delegating =
		ext.execute_with(|| Delegation::is_delegating(&alternative_did_2, &delegation_id_2, max_parent_checks));
	assert_eq!(is_actively_delegating, Ok(false));
}

#[test]
fn check_is_actively_delegating_max_parent_not_revoked() {
	let actor_did = did_mock::ALICE_DID;
	let alternative_did_1 = did_mock::BOB_DID;
	let alternative_did_2 = did_mock::CHARLIE_DID;

	let (root_id, root_node) = (get_delegation_root_id(true), generate_base_delegation_root(actor_did));
	let (delegation_id_1, delegation_node_1) = (
		get_delegation_id(true),
		generate_base_delegation_node(root_id, alternative_did_1.clone()),
	);
	let (delegation_id_2, mut delegation_node_2) = (
		get_delegation_id(false),
		generate_base_delegation_node(root_id, alternative_did_2),
	);
	delegation_node_2.parent = Some(delegation_id_1);
	let max_parent_checks = 2u32;

	// Root -> Delegation 1 -> Delegation 2
	let builder = ExtBuilder::default()
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![
			(delegation_id_1, delegation_node_1),
			(delegation_id_2, delegation_node_2),
		])
		.with_children(vec![
			(root_id, vec![delegation_id_1]),
			(delegation_id_1, vec![delegation_id_2]),
		]);

	let mut ext = builder.build();

	let is_actively_delegating =
		ext.execute_with(|| Delegation::is_delegating(&alternative_did_1, &delegation_id_2, max_parent_checks));
	assert_eq!(is_actively_delegating, Ok(true));
}

#[test]
fn check_is_actively_delegating_max_parent_revoked() {
	let actor_did = did_mock::ALICE_DID;
	let alternative_did_1 = did_mock::BOB_DID;
	let alternative_did_2 = did_mock::CHARLIE_DID;

	let (root_id, root_node) = (get_delegation_root_id(true), generate_base_delegation_root(actor_did));
	let (delegation_id_1, mut delegation_node_1) = (
		get_delegation_id(true),
		generate_base_delegation_node(root_id, alternative_did_1.clone()),
	);
	delegation_node_1.revoked = true;
	let (delegation_id_2, mut delegation_node_2) = (
		get_delegation_id(false),
		generate_base_delegation_node(root_id, alternative_did_2),
	);
	delegation_node_2.parent = Some(delegation_id_1);
	let max_parent_checks = 2u32;

	// Root -> Delegation 1 -> Delegation 2
	let builder = ExtBuilder::default()
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![
			(delegation_id_1, delegation_node_1),
			(delegation_id_2, delegation_node_2),
		])
		.with_children(vec![
			(root_id, vec![delegation_id_1]),
			(delegation_id_1, vec![delegation_id_2]),
		]);

	let mut ext = builder.build();

	let is_actively_delegating =
		ext.execute_with(|| Delegation::is_delegating(&alternative_did_1, &delegation_id_2, max_parent_checks));
	assert_eq!(is_actively_delegating, Ok(false));
}

#[test]
fn check_is_actively_delegating_root_owner_not_revoked() {
	let actor_did = did_mock::ALICE_DID;
	let alternative_did_1 = did_mock::BOB_DID;
	let alternative_did_2 = did_mock::CHARLIE_DID;

	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(actor_did.clone()),
	);
	let (delegation_id_1, delegation_node_1) = (
		get_delegation_id(true),
		generate_base_delegation_node(root_id, alternative_did_1),
	);
	let (delegation_id_2, mut delegation_node_2) = (
		get_delegation_id(false),
		generate_base_delegation_node(root_id, alternative_did_2),
	);
	delegation_node_2.parent = Some(delegation_id_1);
	let max_parent_checks = 2u32;

	// Root -> Delegation 1 -> Delegation 2
	let builder = ExtBuilder::default()
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![
			(delegation_id_1, delegation_node_1),
			(delegation_id_2, delegation_node_2),
		])
		.with_children(vec![
			(root_id, vec![delegation_id_1]),
			(delegation_id_1, vec![delegation_id_2]),
		]);

	let mut ext = builder.build();

	let is_actively_delegating =
		ext.execute_with(|| Delegation::is_delegating(&actor_did, &delegation_id_2, max_parent_checks));
	assert_eq!(is_actively_delegating, Ok(true));
}

#[test]
fn check_is_actively_delegating_root_owner_revoked() {
	let actor_did = did_mock::ALICE_DID;
	let alternative_did_1 = did_mock::BOB_DID;
	let alternative_did_2 = did_mock::CHARLIE_DID;

	let (root_id, mut root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(actor_did.clone()),
	);
	root_node.revoked = true;
	let (delegation_id_1, delegation_node_1) = (
		get_delegation_id(true),
		generate_base_delegation_node(root_id, alternative_did_1),
	);
	let (delegation_id_2, mut delegation_node_2) = (
		get_delegation_id(false),
		generate_base_delegation_node(root_id, alternative_did_2),
	);
	delegation_node_2.parent = Some(delegation_id_1);
	let max_parent_checks = u32::MAX;

	// Root -> Delegation 1 -> Delegation 2
	let builder = ExtBuilder::default()
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![
			(delegation_id_1, delegation_node_1),
			(delegation_id_2, delegation_node_2),
		])
		.with_children(vec![
			(root_id, vec![delegation_id_1]),
			(delegation_id_1, vec![delegation_id_2]),
		]);

	let mut ext = builder.build();

	let is_actively_delegating =
		ext.execute_with(|| Delegation::is_delegating(&actor_did, &delegation_id_2, max_parent_checks));
	assert_eq!(is_actively_delegating, Ok(false));
}

#[test]
fn check_is_actively_delegating_delegation_not_found() {
	let actor_did = did_mock::ALICE_DID;
	let alternative_did_1 = did_mock::BOB_DID;
	let alternative_did_2 = did_mock::CHARLIE_DID;

	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(actor_did.clone()),
	);
	let (delegation_id_1, delegation_node_1) = (
		get_delegation_id(true),
		generate_base_delegation_node(root_id, alternative_did_1),
	);
	let (delegation_id_2, mut delegation_node_2) = (
		get_delegation_id(false),
		generate_base_delegation_node(root_id, alternative_did_2),
	);
	delegation_node_2.parent = Some(delegation_id_1);
	let max_parent_checks = 2u32;

	// Root -> Delegation 1
	let builder = ExtBuilder::default()
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![(delegation_id_1, delegation_node_1)])
		.with_children(vec![(root_id, vec![delegation_id_1])]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::is_delegating(&actor_did, &delegation_id_2, max_parent_checks),
			delegation::Error::<Test>::DelegationNotFound
		);
	});
}

#[test]
fn check_is_actively_delegating_root_after_max_limit() {
	let actor_did = did_mock::ALICE_DID;
	let alternative_did_1 = did_mock::BOB_DID;
	let alternative_did_2 = did_mock::CHARLIE_DID;

	let (root_id, root_node) = (
		get_delegation_root_id(true),
		generate_base_delegation_root(actor_did.clone()),
	);
	let (delegation_id_1, delegation_node_1) = (
		get_delegation_id(true),
		generate_base_delegation_node(root_id, alternative_did_1),
	);
	let (delegation_id_2, mut delegation_node_2) = (
		get_delegation_id(false),
		generate_base_delegation_node(root_id, alternative_did_2),
	);
	delegation_node_2.parent = Some(delegation_id_1);
	// 1 less than needed
	let max_parent_checks = 1u32;

	// Root -> Delegation 1 -> Delegation 2
	let builder = ExtBuilder::default()
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![
			(delegation_id_1, delegation_node_1),
			(delegation_id_2, delegation_node_2),
		])
		.with_children(vec![
			(root_id, vec![delegation_id_1]),
			(delegation_id_1, vec![delegation_id_2]),
		]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::is_delegating(&actor_did, &delegation_id_2, max_parent_checks),
			delegation::Error::<Test>::MaxSearchDepthReached
		);
	});
}
