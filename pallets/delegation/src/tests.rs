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

	let mut delegator_details =
		did_mock::generate_base_did_details(did::PublicVerificationKey::from(auth_key.public()));
	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(del_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let (root_id, root_node) = (get_delegation_root_id(true), generate_base_delegation_root(delegator_did.clone()));

	let operation = generate_base_delegation_root_creation_operation(root_id, root_node.clone());
	let signature = del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(delegator_did.clone(), delegator_details.clone())]);
	let builder =
		ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, delegator_did.clone())]);

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
	assert_eq!(stored_delegation_root.owner, operation.caller_did);
	assert_eq!(stored_delegation_root.revoked, false);

	// Verify that the DID tx counter has increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.caller_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value() + 1u64
	);
}

#[test]
fn check_did_not_found_submit_delegation_root_creation_operation() {
	let auth_key = did_mock::get_ed25519_authentication_key(true);
	let del_key = did_mock::get_sr25519_delegation_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::PublicVerificationKey::from(auth_key.public()));
	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(del_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let alternative_did = did_mock::BOB_DID;
	let (root_id, root_node) = (get_delegation_root_id(true), generate_base_delegation_root(delegator_did.clone()));

	let operation = generate_base_delegation_root_creation_operation(root_id, root_node.clone());
	let signature = del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(alternative_did.clone(), delegator_details.clone())]);

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

	let mut delegator_details =
		did_mock::generate_base_did_details(did::PublicVerificationKey::from(auth_key.public()));
	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(del_key.public()));
	delegator_details.set_tx_counter(u64::MAX);

	let delegator_did = did_mock::ALICE_DID;
	let (root_id, root_node) = (get_delegation_root_id(true), generate_base_delegation_root(delegator_did.clone()));

	let operation = generate_base_delegation_root_creation_operation(root_id, root_node.clone());
	let signature = del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(delegator_did.clone(), delegator_details.clone())]);
	let builder =
		ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(operation.ctype_hash.clone(), delegator_did.clone())]);

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
		ext.execute_with(|| Did::get_did(&operation.caller_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value()
	);
}

#[test]
fn check_did_too_small_tx_counter_submit_delegation_root_creation_operation() {
	let auth_key = did_mock::get_ed25519_authentication_key(true);
	let del_key = did_mock::get_sr25519_delegation_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::PublicVerificationKey::from(auth_key.public()));
	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(del_key.public()));
	delegator_details.set_tx_counter(1u64);

	let delegator_did = did_mock::ALICE_DID;
	let (root_id, root_node) = (get_delegation_root_id(true), generate_base_delegation_root(delegator_did.clone()));

	let mut operation = generate_base_delegation_root_creation_operation(root_id, root_node.clone());
	operation.tx_counter = delegator_details.get_tx_counter_value() - 1u64;
	let signature = del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(delegator_did.clone(), delegator_details.clone())]);
	let builder =
		ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(operation.ctype_hash.clone(), delegator_did.clone())]);

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
		ext.execute_with(|| Did::get_did(&operation.caller_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value()
	);
}

#[test]
fn check_did_equal_tx_counter_submit_delegation_root_creation_operation() {
	let auth_key = did_mock::get_ed25519_authentication_key(true);
	let del_key = did_mock::get_sr25519_delegation_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::PublicVerificationKey::from(auth_key.public()));
	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(del_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let (root_id, root_node) = (get_delegation_root_id(true), generate_base_delegation_root(delegator_did.clone()));

	let mut operation = generate_base_delegation_root_creation_operation(root_id, root_node.clone());
	operation.tx_counter = delegator_details.get_tx_counter_value();
	let signature = del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(delegator_did.clone(), delegator_details.clone())]);
	let builder =
		ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(operation.ctype_hash.clone(), delegator_did.clone())]);

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
		ext.execute_with(|| Did::get_did(&operation.caller_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value()
	);
}

#[test]
fn check_did_too_large_tx_counter_submit_delegation_root_creation_operation() {
	let auth_key = did_mock::get_ed25519_authentication_key(true);
	let del_key = did_mock::get_sr25519_delegation_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::PublicVerificationKey::from(auth_key.public()));
	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(del_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let (root_id, root_node) = (get_delegation_root_id(true), generate_base_delegation_root(delegator_did.clone()));

	let mut operation = generate_base_delegation_root_creation_operation(root_id, root_node.clone());
	operation.tx_counter = delegator_details.get_tx_counter_value() + 2u64;
	let signature = del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(delegator_did.clone(), delegator_details.clone())]);
	let builder =
		ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(operation.ctype_hash.clone(), delegator_did.clone())]);

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
		ext.execute_with(|| Did::get_did(&operation.caller_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value()
	);
}

#[test]
fn check_did_delegation_key_not_present_submit_delegation_root_creation_operation() {
	let auth_key = did_mock::get_ed25519_authentication_key(true);
	let del_key = did_mock::get_sr25519_delegation_key(true);

	let delegator_details = did_mock::generate_base_did_details(did::PublicVerificationKey::from(auth_key.public()));
	// No delegation key is added to the delegator details

	let delegator_did = did_mock::ALICE_DID;
	let (root_id, root_node) = (get_delegation_root_id(true), generate_base_delegation_root(delegator_did.clone()));

	let operation = generate_base_delegation_root_creation_operation(root_id, root_node.clone());
	let signature = del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(delegator_did.clone(), delegator_details.clone())]);
	let builder =
		ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(operation.ctype_hash.clone(), delegator_did.clone())]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_root_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::VerificationKeysNotPresent
		);
	});

	// Verify that the DID tx counter has not increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.caller_did).expect("DID submitter should be present on chain."));
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

	let mut delegator_details =
		did_mock::generate_base_did_details(did::PublicVerificationKey::from(auth_key.public()));
	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(del_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let (root_id, root_node) = (get_delegation_root_id(true), generate_base_delegation_root(delegator_did.clone()));

	let operation = generate_base_delegation_root_creation_operation(root_id, root_node.clone());
	let signature = invalid_format_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(delegator_did.clone(), delegator_details.clone())]);
	let builder =
		ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(operation.ctype_hash.clone(), delegator_did.clone())]);

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
		ext.execute_with(|| Did::get_did(&operation.caller_did).expect("DID submitter should be present on chain."));
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

	let mut delegator_details =
		did_mock::generate_base_did_details(did::PublicVerificationKey::from(auth_key.public()));
	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(del_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let (root_id, root_node) = (get_delegation_root_id(true), generate_base_delegation_root(delegator_did.clone()));

	let operation = generate_base_delegation_root_creation_operation(root_id, root_node.clone());
	let signature = invalid_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(delegator_did.clone(), delegator_details.clone())]);
	let builder =
		ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(operation.ctype_hash.clone(), delegator_did.clone())]);

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
		ext.execute_with(|| Did::get_did(&operation.caller_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value()
	);
}

#[test]
fn check_duplicate_submit_delegation_root_creation_operation() {
	let auth_key = did_mock::get_ed25519_authentication_key(true);
	let del_key = did_mock::get_sr25519_delegation_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::PublicVerificationKey::from(auth_key.public()));
	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(del_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let (root_id, root_node) = (get_delegation_root_id(true), generate_base_delegation_root(delegator_did.clone()));

	let operation = generate_base_delegation_root_creation_operation(root_id, root_node.clone());
	let signature = del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(delegator_did.clone(), delegator_details.clone())]);
	let builder =
		ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(operation.ctype_hash.clone(), delegator_did.clone())]);
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
		ext.execute_with(|| Did::get_did(&operation.caller_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value() + 1u64
	);
}

#[test]
fn check_ctype_not_found_submit_delegation_root_creation_operation() {
	let auth_key = did_mock::get_ed25519_authentication_key(true);
	let del_key = did_mock::get_sr25519_delegation_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::PublicVerificationKey::from(auth_key.public()));
	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(del_key.public()));

	let delegator_did = did_mock::ALICE_DID;
	let (root_id, root_node) = (get_delegation_root_id(true), generate_base_delegation_root(delegator_did.clone()));

	let operation = generate_base_delegation_root_creation_operation(root_id, root_node.clone());
	let signature = del_key.sign(&operation.encode());

	// No CTYPE created in the builder
	let builder = did_mock::ExtBuilder::default().with_dids(vec![(delegator_did.clone(), delegator_details.clone())]);

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
		ext.execute_with(|| Did::get_did(&operation.caller_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value() + 1u64
	);
}

// submit_delegation_creation_operation()

#[test]
fn check_submit_delegation_no_parent_creation_operation_successful() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

	let mut delegator_details =
		did_mock::generate_base_did_details(did::PublicVerificationKey::from(delegator_auth_key.public()));
	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(delegator_del_key.public()));

	let delegate_details =
		did_mock::generate_base_did_details(did::PublicVerificationKey::from(delegate_auth_key.public()));

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

	let operation = generate_base_delegation_creation_operation(delegator_did.clone(), delegation_id, delegate_signature.clone(), delegation_node.clone());
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(delegator_did.clone(), delegator_details.clone()),
		(delegate_did.clone(), delegate_details.clone()),
	]);
	let builder =
		ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash.clone(), delegator_did.clone())]);
	let builder = ExtBuilder::from(builder).with_root_delegations(vec![(root_id, root_node.clone())]);

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
	assert_eq!(stored_delegation.revoked, false);

	// Verify that the root has the new delegation among its children
	let stored_root_children = ext.execute_with(|| {
		Delegation::children(&operation.root_id).expect("Delegation root children should be present on chain.")
	});

	assert_eq!(stored_root_children, vec![operation.delegation_id]);

	// Verify that the DID tx counter has increased
	let new_delegator_details =
		ext.execute_with(|| Did::get_did(&operation.caller_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value() + 1u64
	);
}

// #[test]
// fn check_submit_delegation_with_parent_creation_operation_successful() {
// 	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
// 	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
// 	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

// 	let mut delegator_details =
// 		did_mock::generate_base_did_details(did::PublicVerificationKey::from(delegator_auth_key.public()));
// 	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(delegator_del_key.public()));

// 	let delegate_details =
// 		did_mock::generate_base_did_details(did::PublicVerificationKey::from(delegate_auth_key.public()));

// 	let delegator_did = did_mock::ALICE_DID;
// 	let delegate_did = did_mock::BOB_DID;
// 	let (root_id, root_node) = (
// 		get_delegation_root_id(true),
// 		generate_base_delegation_root(delegator_did.clone()),
// 	);
// 	let (parent_delegation_id, parent_delegation_node) = (
// 		get_delegation_id(true),
// 		generate_base_delegation_node(delegator_did.clone()),
// 	);
// 	let (delegation_id, mut delegation_node) =(
// 		get_delegation_id(false),
// 		generate_base_delegation_node(delegate_did.clone()),
// 	);
// 	delegation_node.parent = Some(parent_delegation_id);

// 	let delegate_signature = did::DidSignature::from(delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
// 		&delegation_id,
// 		&delegation_node.root_id,
// 		&delegation_node.parent,
// 		&delegation_node.permissions,
// 	))));

// 	let operation = generate_base_delegation_creation_operation(
// 		delegator_did.clone(),
// 		delegate_signature.clone(),
// 		delegation_node.clone(),
// 	);
// 	let signature = delegator_del_key.sign(&operation.encode());

// 	let builder = did_mock::ExtBuilder::default().with_dids(vec![
// 		(delegator_did.clone(), delegator_details.clone()),
// 		(delegate_did.clone(), delegate_details.clone()),
// 	]);
// 	let builder =
// 		ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash.clone(), delegator_did.clone())]);
// 	let builder = ExtBuilder::from(builder)
// 		.with_root_delegations(vec![(root_id, root_node.clone())])
// 		.with_delegations(vec![(parent_delegation_id, parent_delegation_node.clone())]);

// 	let mut ext = builder.build();

// 	ext.execute_with(|| {
// 		assert_ok!(Delegation::submit_delegation_creation_operation(
// 			Origin::signed(DEFAULT_ACCOUNT),
// 			operation.clone(),
// 			did::DidSignature::from(signature)
// 		));
// 	});

// 	let stored_delegation = ext.execute_with(|| {
// 		Delegation::delegations(&operation.delegation_id).expect("Delegation should be present on chain.")
// 	});

// 	assert_eq!(stored_delegation.root_id, operation.root_id);
// 	assert_eq!(stored_delegation.parent, operation.parent_id);
// 	assert_eq!(stored_delegation.owner, operation.delegate_did);
// 	assert_eq!(stored_delegation.permissions, operation.permissions);
// 	assert_eq!(stored_delegation.revoked, false);

// 	// Verify that the parent has the new delegation among its children
// 	let stored_parent_children = ext.execute_with(|| {
// 		Delegation::children(&operation.parent_id.unwrap())
// 			.expect("Delegation parent children should be present on chain.")
// 	});

// 	assert_eq!(stored_parent_children, vec![delegation_id]);

// 	// Verify that the DID tx counter has increased
// 	let new_delegator_details =
// 		ext.execute_with(|| Did::get_did(&operation.caller_did).expect("DID submitter should be present on chain."));
// 	assert_eq!(
// 		new_delegator_details.get_tx_counter_value(),
// 		delegator_details.get_tx_counter_value() + 1u64
// 	);
// }

// #[test]
// fn check_delegator_did_not_found_submit_delegation_creation_operation() {
// 	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
// 	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
// 	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

// 	let mut delegator_details =
// 		did_mock::generate_base_did_details(did::PublicVerificationKey::from(delegator_auth_key.public()));
// 	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(delegator_del_key.public()));

// 	let delegate_details =
// 		did_mock::generate_base_did_details(did::PublicVerificationKey::from(delegate_auth_key.public()));

// 	let delegator_did = did_mock::ALICE_DID;
// 	let alternative_did = did_mock::CHARLIE_DID;
// 	let delegate_did = did_mock::BOB_DID;
// 	let (root_id, root_node) = (
// 		get_delegation_root_id(true),
// 		generate_base_delegation_root(alternative_did.clone()),
// 	);
// 	let (delegation_id, delegation_node) = (
// 		get_delegation_id(true),
// 		generate_base_delegation_node(delegate_did.clone()),
// 	);

// 	let delegate_signature = did::DidSignature::from(delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
// 		&delegation_id,
// 		&delegation_node.root_id,
// 		&delegation_node.parent,
// 		&delegation_node.permissions,
// 	))));

// 	let operation = generate_base_delegation_creation_operation(
// 		delegator_did.clone(),
// 		delegate_did.clone(),
// 		delegate_signature.clone(),
// 	);
// 	let signature = delegator_del_key.sign(&operation.encode());

// 	let builder = did_mock::ExtBuilder::default().with_dids(vec![
// 		(alternative_did.clone(), delegator_details.clone()),
// 		(delegate_did.clone(), delegate_details.clone()),
// 	]);
// 	let builder = ctype_mock::ExtBuilder::from(builder)
// 		.with_ctypes(vec![(root_node.ctype_hash.clone(), alternative_did.clone())]);
// 	let builder = ExtBuilder::from(builder).with_root_delegations(vec![(root_id, root_node.clone())]);

// 	let mut ext = builder.build();

// 	ext.execute_with(|| {
// 		assert_noop!(
// 			Delegation::submit_delegation_creation_operation(
// 				Origin::signed(DEFAULT_ACCOUNT),
// 				operation.clone(),
// 				did::DidSignature::from(signature)
// 			),
// 			did::Error::<Test>::DidNotPresent
// 		);
// 	});
// }

// #[test]
// fn check_delegator_max_tx_counter_value_submit_delegation_creation_operation() {
// 	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
// 	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
// 	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

// 	let mut delegator_details =
// 		did_mock::generate_base_did_details(did::PublicVerificationKey::from(delegator_auth_key.public()));
// 	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(delegator_del_key.public()));
// 	delegator_details.set_tx_counter(u64::MAX);

// 	let delegate_details =
// 		did_mock::generate_base_did_details(did::PublicVerificationKey::from(delegate_auth_key.public()));

// 	let delegator_did = did_mock::ALICE_DID;
// 	let delegate_did = did_mock::BOB_DID;
// 	let (root_id, root_node) = (
// 		get_delegation_root_id(true),
// 		generate_base_delegation_root(delegator_did.clone()),
// 	);
// 	let (delegation_id, delegation_node) = (
// 		get_delegation_id(true),
// 		generate_base_delegation_node(delegate_did.clone()),
// 	);

// 	let delegate_signature = did::DidSignature::from(delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
// 		&delegation_id,
// 		&delegation_node.root_id,
// 		&delegation_node.parent,
// 		&delegation_node.permissions,
// 	))));

// 	let operation = generate_base_delegation_creation_operation(
// 		delegator_did.clone(),
// 		delegate_did.clone(),
// 		delegate_signature.clone(),
// 	);
// 	let signature = delegator_del_key.sign(&operation.encode());

// 	let builder = did_mock::ExtBuilder::default().with_dids(vec![
// 		(delegator_did.clone(), delegator_details.clone()),
// 		(delegate_did.clone(), delegate_details.clone()),
// 	]);
// 	let builder =
// 		ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash.clone(), delegator_did.clone())]);
// 	let builder = ExtBuilder::from(builder).with_root_delegations(vec![(root_id, root_node.clone())]);

// 	let mut ext = builder.build();

// 	ext.execute_with(|| {
// 		assert_noop!(
// 			Delegation::submit_delegation_creation_operation(
// 				Origin::signed(DEFAULT_ACCOUNT),
// 				operation.clone(),
// 				did::DidSignature::from(signature)
// 			),
// 			did::Error::<Test>::MaxTxCounterValue
// 		);
// 	});
// }

// #[test]
// fn check_delegator_too_small_tx_counter_submit_delegation_creation_operation() {
// 	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
// 	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
// 	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

// 	let mut delegator_details =
// 		did_mock::generate_base_did_details(did::PublicVerificationKey::from(delegator_auth_key.public()));
// 	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(delegator_del_key.public()));
// 	delegator_details.set_tx_counter(1u64);

// 	let delegate_details =
// 		did_mock::generate_base_did_details(did::PublicVerificationKey::from(delegate_auth_key.public()));

// 	let delegator_did = did_mock::ALICE_DID;
// 	let delegate_did = did_mock::BOB_DID;
// 	let (root_id, root_node) = (
// 		get_delegation_root_id(true),
// 		generate_base_delegation_root(delegator_did.clone()),
// 	);
// 	let (delegation_id, delegation_node) = (
// 		get_delegation_id(true),
// 		generate_base_delegation_node(delegate_did.clone()),
// 	);

// 	let delegate_signature = did::DidSignature::from(delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
// 		&delegation_id,
// 		&delegation_node.root_id,
// 		&delegation_node.parent,
// 		&delegation_node.permissions,
// 	))));

// 	let mut operation = generate_base_delegation_creation_operation(
// 		delegator_did.clone(),
// 		delegate_did.clone(),
// 		delegate_signature.clone(),
// 	);
// 	operation.tx_counter = delegator_details.get_tx_counter_value() - 1u64;
// 	let signature = delegator_del_key.sign(&operation.encode());

// 	let builder = did_mock::ExtBuilder::default().with_dids(vec![
// 		(delegator_did.clone(), delegator_details.clone()),
// 		(delegate_did.clone(), delegate_details.clone()),
// 	]);
// 	let builder =
// 		ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash.clone(), delegator_did.clone())]);
// 	let builder = ExtBuilder::from(builder).with_root_delegations(vec![(root_id, root_node.clone())]);

// 	let mut ext = builder.build();

// 	ext.execute_with(|| {
// 		assert_noop!(
// 			Delegation::submit_delegation_creation_operation(
// 				Origin::signed(DEFAULT_ACCOUNT),
// 				operation.clone(),
// 				did::DidSignature::from(signature)
// 			),
// 			did::Error::<Test>::InvalidNonce
// 		);
// 	});
// }

// #[test]
// fn check_delegator_equal_tx_counter_submit_delegation_creation_operation() {
// 	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
// 	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
// 	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

// 	let mut delegator_details =
// 		did_mock::generate_base_did_details(did::PublicVerificationKey::from(delegator_auth_key.public()));
// 	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(delegator_del_key.public()));

// 	let delegate_details =
// 		did_mock::generate_base_did_details(did::PublicVerificationKey::from(delegate_auth_key.public()));

// 	let delegator_did = did_mock::ALICE_DID;
// 	let delegate_did = did_mock::BOB_DID;
// 	let (root_id, root_node) = (
// 		get_delegation_root_id(true),
// 		generate_base_delegation_root(delegator_did.clone()),
// 	);
// 	let (delegation_id, delegation_node) = (
// 		get_delegation_id(true),
// 		generate_base_delegation_node(delegate_did.clone()),
// 	);

// 	let delegate_signature = did::DidSignature::from(delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
// 		&delegation_id,
// 		&delegation_node.root_id,
// 		&delegation_node.parent,
// 		&delegation_node.permissions,
// 	))));

// 	let mut operation = generate_base_delegation_creation_operation(
// 		delegator_did.clone(),
// 		delegate_did.clone(),
// 		delegate_signature.clone(),
// 	);
// 	operation.tx_counter = delegator_details.get_tx_counter_value();
// 	let signature = delegator_del_key.sign(&operation.encode());

// 	let builder = did_mock::ExtBuilder::default().with_dids(vec![
// 		(delegator_did.clone(), delegator_details.clone()),
// 		(delegate_did.clone(), delegate_details.clone()),
// 	]);
// 	let builder =
// 		ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash.clone(), delegator_did.clone())]);
// 	let builder = ExtBuilder::from(builder).with_root_delegations(vec![(root_id, root_node.clone())]);

// 	let mut ext = builder.build();

// 	ext.execute_with(|| {
// 		assert_noop!(
// 			Delegation::submit_delegation_creation_operation(
// 				Origin::signed(DEFAULT_ACCOUNT),
// 				operation.clone(),
// 				did::DidSignature::from(signature)
// 			),
// 			did::Error::<Test>::InvalidNonce
// 		);
// 	});
// }

// #[test]
// fn check_delegator_too_large_tx_counter_submit_delegation_creation_operation() {
// 	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
// 	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
// 	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

// 	let mut delegator_details =
// 		did_mock::generate_base_did_details(did::PublicVerificationKey::from(delegator_auth_key.public()));
// 	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(delegator_del_key.public()));

// 	let delegate_details =
// 		did_mock::generate_base_did_details(did::PublicVerificationKey::from(delegate_auth_key.public()));

// 	let delegator_did = did_mock::ALICE_DID;
// 	let delegate_did = did_mock::BOB_DID;
// 	let (root_id, root_node) = (
// 		get_delegation_root_id(true),
// 		generate_base_delegation_root(delegator_did.clone()),
// 	);
// 	let (delegation_id, delegation_node) = (
// 		get_delegation_id(true),
// 		generate_base_delegation_node(delegate_did.clone()),
// 	);

// 	let delegate_signature = did::DidSignature::from(delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
// 		&delegation_id,
// 		&delegation_node.root_id,
// 		&delegation_node.parent,
// 		&delegation_node.permissions,
// 	))));

// 	let mut operation = generate_base_delegation_creation_operation(
// 		delegator_did.clone(),
// 		delegate_did.clone(),
// 		delegate_signature.clone(),
// 	);
// 	operation.tx_counter = delegator_details.get_tx_counter_value() + 2u64;
// 	let signature = delegator_del_key.sign(&operation.encode());

// 	let builder = did_mock::ExtBuilder::default().with_dids(vec![
// 		(delegator_did.clone(), delegator_details.clone()),
// 		(delegate_did.clone(), delegate_details.clone()),
// 	]);
// 	let builder =
// 		ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash.clone(), delegator_did.clone())]);
// 	let builder = ExtBuilder::from(builder).with_root_delegations(vec![(root_id, root_node.clone())]);

// 	let mut ext = builder.build();

// 	ext.execute_with(|| {
// 		assert_noop!(
// 			Delegation::submit_delegation_creation_operation(
// 				Origin::signed(DEFAULT_ACCOUNT),
// 				operation.clone(),
// 				did::DidSignature::from(signature)
// 			),
// 			did::Error::<Test>::InvalidNonce
// 		);
// 	});
// }

// #[test]
// fn check_delegator_delegation_key_not_present_submit_delegation_creation_operation() {
// 	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
// 	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
// 	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

// 	let delegator_details =
// 		did_mock::generate_base_did_details(did::PublicVerificationKey::from(delegator_auth_key.public()));
// 	// No delegation key specified

// 	let delegate_details =
// 		did_mock::generate_base_did_details(did::PublicVerificationKey::from(delegate_auth_key.public()));

// 	let delegator_did = did_mock::ALICE_DID;
// 	let delegate_did = did_mock::BOB_DID;
// 	let (root_id, root_node) = (
// 		get_delegation_root_id(true),
// 		generate_base_delegation_root(delegator_did.clone()),
// 	);
// 	let (delegation_id, delegation_node) = (
// 		get_delegation_id(true),
// 		generate_base_delegation_node(delegate_did.clone()),
// 	);

// 	let delegate_signature = did::DidSignature::from(delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
// 		&delegation_id,
// 		&delegation_node.root_id,
// 		&delegation_node.parent,
// 		&delegation_node.permissions,
// 	))));

// 	let operation = generate_base_delegation_creation_operation(
// 		delegator_did.clone(),
// 		delegate_did.clone(),
// 		delegate_signature.clone(),
// 	);
// 	let signature = delegator_del_key.sign(&operation.encode());

// 	let builder = did_mock::ExtBuilder::default().with_dids(vec![
// 		(delegator_did.clone(), delegator_details.clone()),
// 		(delegate_did.clone(), delegate_details.clone()),
// 	]);
// 	let builder =
// 		ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash.clone(), delegator_did.clone())]);
// 	let builder = ExtBuilder::from(builder).with_root_delegations(vec![(root_id, root_node.clone())]);

// 	let mut ext = builder.build();

// 	ext.execute_with(|| {
// 		assert_noop!(
// 			Delegation::submit_delegation_creation_operation(
// 				Origin::signed(DEFAULT_ACCOUNT),
// 				operation.clone(),
// 				did::DidSignature::from(signature)
// 			),
// 			did::Error::<Test>::VerificationKeysNotPresent
// 		);
// 	});
// }

// #[test]
// fn check_delegator_invalid_signature_format_submit_delegation_creation_operation() {
// 	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
// 	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
// 	let alternative_del_key = did_mock::get_ed25519_delegation_key(true);
// 	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

// 	let mut delegator_details =
// 		did_mock::generate_base_did_details(did::PublicVerificationKey::from(delegator_auth_key.public()));
// 	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(delegator_del_key.public()));

// 	let delegate_details =
// 		did_mock::generate_base_did_details(did::PublicVerificationKey::from(delegate_auth_key.public()));

// 	let delegator_did = did_mock::ALICE_DID;
// 	let delegate_did = did_mock::BOB_DID;
// 	let (root_id, root_node) = (
// 		get_delegation_root_id(true),
// 		generate_base_delegation_root(delegator_did.clone()),
// 	);
// 	let (delegation_id, delegation_node) = (
// 		get_delegation_id(true),
// 		generate_base_delegation_node(delegate_did.clone()),
// 	);

// 	let delegate_signature = did::DidSignature::from(delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
// 		&delegation_id,
// 		&delegation_node.root_id,
// 		&delegation_node.parent,
// 		&delegation_node.permissions,
// 	))));

// 	let operation = generate_base_delegation_creation_operation(
// 		delegator_did.clone(),
// 		delegate_did.clone(),
// 		delegate_signature.clone(),
// 	);
// 	let signature = alternative_del_key.sign(&operation.encode());

// 	let builder = did_mock::ExtBuilder::default().with_dids(vec![
// 		(delegator_did.clone(), delegator_details.clone()),
// 		(delegate_did.clone(), delegate_details.clone()),
// 	]);
// 	let builder =
// 		ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash.clone(), delegator_did.clone())]);
// 	let builder = ExtBuilder::from(builder).with_root_delegations(vec![(root_id, root_node.clone())]);

// 	let mut ext = builder.build();

// 	ext.execute_with(|| {
// 		assert_noop!(
// 			Delegation::submit_delegation_creation_operation(
// 				Origin::signed(DEFAULT_ACCOUNT),
// 				operation.clone(),
// 				did::DidSignature::from(signature)
// 			),
// 			did::Error::<Test>::InvalidSignatureFormat
// 		);
// 	});
// }

// #[test]
// fn check_delegator_invalid_signature_submit_delegation_creation_operation() {
// 	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
// 	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
// 	let alternative_del_key = did_mock::get_sr25519_delegation_key(false);
// 	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

// 	let mut delegator_details =
// 		did_mock::generate_base_did_details(did::PublicVerificationKey::from(delegator_auth_key.public()));
// 	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(delegator_del_key.public()));

// 	let delegate_details =
// 		did_mock::generate_base_did_details(did::PublicVerificationKey::from(delegate_auth_key.public()));

// 	let delegator_did = did_mock::ALICE_DID;
// 	let delegate_did = did_mock::BOB_DID;
// 	let (root_id, root_node) = (
// 		get_delegation_root_id(true),
// 		generate_base_delegation_root(delegator_did.clone()),
// 	);
// 	let (delegation_id, delegation_node) = (
// 		get_delegation_id(true),
// 		generate_base_delegation_node(delegate_did.clone()),
// 	);

// 	let delegate_signature = did::DidSignature::from(delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
// 		&delegation_id,
// 		&delegation_node.root_id,
// 		&delegation_node.parent,
// 		&delegation_node.permissions,
// 	))));

// 	let operation = generate_base_delegation_creation_operation(
// 		delegator_did.clone(),
// 		delegate_did.clone(),
// 		delegate_signature.clone(),
// 	);
// 	let signature = alternative_del_key.sign(&operation.encode());

// 	let builder = did_mock::ExtBuilder::default().with_dids(vec![
// 		(delegator_did.clone(), delegator_details.clone()),
// 		(delegate_did.clone(), delegate_details.clone()),
// 	]);
// 	let builder =
// 		ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash.clone(), delegator_did.clone())]);
// 	let builder = ExtBuilder::from(builder).with_root_delegations(vec![(root_id, root_node.clone())]);

// 	let mut ext = builder.build();

// 	ext.execute_with(|| {
// 		assert_noop!(
// 			Delegation::submit_delegation_creation_operation(
// 				Origin::signed(DEFAULT_ACCOUNT),
// 				operation.clone(),
// 				did::DidSignature::from(signature)
// 			),
// 			did::Error::<Test>::InvalidSignature
// 		);
// 	});
// }

// #[test]
// fn check_delegate_did_not_found_submit_delegation_creation_operation() {
// 	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
// 	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
// 	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

// 	let mut delegator_details =
// 		did_mock::generate_base_did_details(did::PublicVerificationKey::from(delegator_auth_key.public()));
// 	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(delegator_del_key.public()));

// 	let delegator_did = did_mock::ALICE_DID;
// 	let delegate_did = did_mock::BOB_DID;
// 	let (root_id, root_node) = (
// 		get_delegation_root_id(true),
// 		generate_base_delegation_root(delegator_did.clone()),
// 	);
// 	let (delegation_id, delegation_node) = (
// 		get_delegation_id(true),
// 		generate_base_delegation_node(delegate_did.clone()),
// 	);

// 	let delegate_signature = did::DidSignature::from(delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
// 		&delegation_id,
// 		&delegation_node.root_id,
// 		&delegation_node.parent,
// 		&delegation_node.permissions,
// 	))));

// 	let operation = generate_base_delegation_creation_operation(
// 		delegator_did.clone(),
// 		delegate_did.clone(),
// 		delegate_signature.clone(),
// 	);
// 	let signature = delegator_del_key.sign(&operation.encode());

// 	let builder = did_mock::ExtBuilder::default().with_dids(vec![
// 		(delegator_did.clone(), delegator_details.clone()),
// 		// No delegate DID
// 	]);
// 	let builder =
// 		ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash.clone(), delegator_did.clone())]);
// 	let builder = ExtBuilder::from(builder).with_root_delegations(vec![(root_id, root_node.clone())]);

// 	let mut ext = builder.build();

// 	ext.execute_with(|| {
// 		assert_err!(
// 			Delegation::submit_delegation_creation_operation(
// 				Origin::signed(DEFAULT_ACCOUNT),
// 				operation.clone(),
// 				did::DidSignature::from(signature)
// 			),
// 			delegation::Error::<Test>::DelegateNotFound
// 		);
// 	});

// 	// Verify that the DID tx counter has increased
// 	let new_delegator_details =
// 		ext.execute_with(|| Did::get_did(&operation.caller_did).expect("DID submitter should be present on chain."));
// 	assert_eq!(
// 		new_delegator_details.get_tx_counter_value(),
// 		delegator_details.get_tx_counter_value() + 1u64
// 	);
// }

// #[test]
// fn check_invalid_delegate_signature_submit_delegation_creation_operation() {
// 	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
// 	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
// 	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);
// 	let alternative_auth_key = did_mock::get_sr25519_attestation_key(false);

// 	let mut delegator_details =
// 		did_mock::generate_base_did_details(did::PublicVerificationKey::from(delegator_auth_key.public()));
// 	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(delegator_del_key.public()));

// 	let delegate_details =
// 		did_mock::generate_base_did_details(did::PublicVerificationKey::from(delegate_auth_key.public()));

// 	let delegator_did = did_mock::ALICE_DID;
// 	let delegate_did = did_mock::BOB_DID;
// 	let (root_id, root_node) = (
// 		get_delegation_root_id(true),
// 		generate_base_delegation_root(delegator_did.clone()),
// 	);
// 	let (delegation_id, delegation_node) = (
// 		get_delegation_id(true),
// 		generate_base_delegation_node(delegate_did.clone()),
// 	);

// 	let delegate_signature =
// 		did::DidSignature::from(alternative_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
// 			&delegation_id,
// 			&delegation_node.root_id,
// 			&delegation_node.parent,
// 			&delegation_node.permissions,
// 		))));

// 	let operation = generate_base_delegation_creation_operation(
// 		delegator_did.clone(),
// 		delegate_did.clone(),
// 		delegate_signature.clone(),
// 	);
// 	let signature = delegator_del_key.sign(&operation.encode());

// 	let builder = did_mock::ExtBuilder::default().with_dids(vec![
// 		(delegator_did.clone(), delegator_details.clone()),
// 		(delegate_did.clone(), delegate_details.clone()),
// 	]);
// 	let builder =
// 		ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash.clone(), delegator_did.clone())]);
// 	let builder = ExtBuilder::from(builder).with_root_delegations(vec![(root_id, root_node.clone())]);

// 	let mut ext = builder.build();

// 	ext.execute_with(|| {
// 		assert_err!(
// 			Delegation::submit_delegation_creation_operation(
// 				Origin::signed(DEFAULT_ACCOUNT),
// 				operation.clone(),
// 				did::DidSignature::from(signature)
// 			),
// 			delegation::Error::<Test>::InvalidDelegateSignature
// 		);
// 	});

// 	// Verify that the DID tx counter has increased
// 	let new_delegator_details =
// 		ext.execute_with(|| Did::get_did(&operation.caller_did).expect("DID submitter should be present on chain."));
// 	assert_eq!(
// 		new_delegator_details.get_tx_counter_value(),
// 		delegator_details.get_tx_counter_value() + 1u64
// 	);
// }

// #[test]
// fn check_duplicate_delegation_submit_delegation_creation_operation() {
// 	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
// 	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
// 	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

// 	let mut delegator_details =
// 		did_mock::generate_base_did_details(did::PublicVerificationKey::from(delegator_auth_key.public()));
// 	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(delegator_del_key.public()));

// 	let delegate_details =
// 		did_mock::generate_base_did_details(did::PublicVerificationKey::from(delegate_auth_key.public()));

// 	let delegator_did = did_mock::ALICE_DID;
// 	let delegate_did = did_mock::BOB_DID;
// 	let (root_id, root_node) = (
// 		get_delegation_root_id(true),
// 		generate_base_delegation_root(delegator_did.clone()),
// 	);
// 	let (delegation_id, delegation_node) = (
// 		get_delegation_id(true),
// 		generate_base_delegation_node(delegate_did.clone()),
// 	);

// 	let delegate_signature = did::DidSignature::from(delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
// 		&delegation_id,
// 		&delegation_node.root_id,
// 		&delegation_node.parent,
// 		&delegation_node.permissions,
// 	))));

// 	let operation = generate_base_delegation_creation_operation(
// 		delegator_did.clone(),
// 		delegate_did.clone(),
// 		delegate_signature.clone(),
// 	);
// 	let signature = delegator_del_key.sign(&operation.encode());

// 	let builder = did_mock::ExtBuilder::default().with_dids(vec![
// 		(delegator_did.clone(), delegator_details.clone()),
// 		(delegate_did.clone(), delegate_details.clone()),
// 	]);
// 	let builder =
// 		ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash.clone(), delegator_did.clone())]);
// 	let builder = ExtBuilder::from(builder)
// 		.with_root_delegations(vec![(root_id, root_node.clone())])
// 		.with_delegations(vec![(delegation_id, delegation_node.clone())]);

// 	let mut ext = builder.build();

// 	ext.execute_with(|| {
// 		assert_err!(
// 			Delegation::submit_delegation_creation_operation(
// 				Origin::signed(DEFAULT_ACCOUNT),
// 				operation.clone(),
// 				did::DidSignature::from(signature)
// 			),
// 			delegation::Error::<Test>::DelegationAlreadyExists
// 		);
// 	});

// 	// Verify that the DID tx counter has increased
// 	let new_delegator_details =
// 		ext.execute_with(|| Did::get_did(&operation.caller_did).expect("DID submitter should be present on chain."));
// 	assert_eq!(
// 		new_delegator_details.get_tx_counter_value(),
// 		delegator_details.get_tx_counter_value() + 1u64
// 	);
// }

// #[test]
// fn check_root_not_existing_submit_delegation_creation_operation() {
// 	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
// 	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
// 	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

// 	let mut delegator_details =
// 		did_mock::generate_base_did_details(did::PublicVerificationKey::from(delegator_auth_key.public()));
// 	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(delegator_del_key.public()));

// 	let delegate_details =
// 		did_mock::generate_base_did_details(did::PublicVerificationKey::from(delegate_auth_key.public()));

// 	let delegator_did = did_mock::ALICE_DID;
// 	let delegate_did = did_mock::BOB_DID;
// 	let (root_id, root_node) = (
// 		get_delegation_root_id(true),
// 		generate_base_delegation_root(delegator_did.clone()),
// 	);
// 	let alternative_root_id = get_delegation_root_id(false);
// 	let (delegation_id, delegation_node) = (
// 		get_delegation_id(true),
// 		generate_base_delegation_node(delegate_did.clone()),
// 	);

// 	let delegate_signature = did::DidSignature::from(delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
// 		&delegation_id,
// 		&alternative_root_id,
// 		&delegation_node.parent,
// 		&delegation_node.permissions,
// 	))));

// 	let mut operation = generate_base_delegation_creation_operation(
// 		delegator_did.clone(),
// 		delegate_did.clone(),
// 		delegate_signature.clone(),
// 	);
// 	operation.root_id = alternative_root_id;
// 	let signature = delegator_del_key.sign(&operation.encode());

// 	let builder = did_mock::ExtBuilder::default().with_dids(vec![
// 		(delegator_did.clone(), delegator_details.clone()),
// 		(delegate_did.clone(), delegate_details.clone()),
// 	]);
// 	let builder =
// 		ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash.clone(), delegator_did.clone())]);
// 	let builder = ExtBuilder::from(builder)
// 		.with_root_delegations(vec![(root_id, root_node.clone())]);

// 	let mut ext = builder.build();

// 	ext.execute_with(|| {
// 		assert_err!(
// 			Delegation::submit_delegation_creation_operation(
// 				Origin::signed(DEFAULT_ACCOUNT),
// 				operation.clone(),
// 				did::DidSignature::from(signature)
// 			),
// 			delegation::Error::<Test>::RootNotFound
// 		);
// 	});

// 	// Verify that the DID tx counter has increased
// 	let new_delegator_details =
// 		ext.execute_with(|| Did::get_did(&operation.caller_did).expect("DID submitter should be present on chain."));
// 	assert_eq!(
// 		new_delegator_details.get_tx_counter_value(),
// 		delegator_details.get_tx_counter_value() + 1u64
// 	);
// }

// #[test]
// fn check_parent_not_existing_submit_delegation_creation_operation() {
// 	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
// 	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
// 	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

// 	let mut delegator_details =
// 		did_mock::generate_base_did_details(did::PublicVerificationKey::from(delegator_auth_key.public()));
// 	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(delegator_del_key.public()));

// 	let delegate_details =
// 		did_mock::generate_base_did_details(did::PublicVerificationKey::from(delegate_auth_key.public()));

// 	let delegator_did = did_mock::ALICE_DID;
// 	let delegate_did = did_mock::BOB_DID;
// 	let (root_id, root_node) = (
// 		get_delegation_root_id(true),
// 		generate_base_delegation_root(delegator_did.clone()),
// 	);
// 	let alternative_parent_id = get_delegation_id(false);
// 	let (delegation_id, mut delegation_node) = (
// 		get_delegation_id(true),
// 		generate_base_delegation_node(delegate_did.clone()),
// 	);
// 	delegation_node.parent = Some(alternative_parent_id);

// 	let delegate_signature = did::DidSignature::from(delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
// 		&delegation_id,
// 		&delegation_node.root_id,
// 		&delegation_node.parent,
// 		&delegation_node.permissions,
// 	))));

// 	let operation = generate_base_delegation_creation_operation(
// 		delegator_did.clone(),
// 		delegate_did.clone(),
// 		delegate_signature.clone(),
// 	);
// 	let signature = delegator_del_key.sign(&operation.encode());

// 	let builder = did_mock::ExtBuilder::default().with_dids(vec![
// 		(delegator_did.clone(), delegator_details.clone()),
// 		(delegate_did.clone(), delegate_details.clone()),
// 	]);
// 	let builder =
// 		ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash.clone(), delegator_did.clone())]);
// 	let builder = ExtBuilder::from(builder)
// 		.with_root_delegations(vec![(root_id, root_node.clone())]);

// 	let mut ext = builder.build();

// 	ext.execute_with(|| {
// 		assert_err!(
// 			Delegation::submit_delegation_creation_operation(
// 				Origin::signed(DEFAULT_ACCOUNT),
// 				operation.clone(),
// 				did::DidSignature::from(signature)
// 			),
// 			delegation::Error::<Test>::ParentDelegationNotFound
// 		);
// 	});

// 	// Verify that the DID tx counter has increased
// 	let new_delegator_details =
// 		ext.execute_with(|| Did::get_did(&operation.caller_did).expect("DID submitter should be present on chain."));
// 	assert_eq!(
// 		new_delegator_details.get_tx_counter_value(),
// 		delegator_details.get_tx_counter_value() + 1u64
// 	);
// }

// // #[test]
// // fn check_not_owner_of_parent_submit_delegation_creation_operation() {
// // 	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
// // 	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
// // 	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);

// // 	let mut delegator_details =
// // 		did_mock::generate_base_did_details(did::PublicVerificationKey::from(delegator_auth_key.public()));
// // 	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(delegator_del_key.public()));

// // 	let delegate_details =
// // 		did_mock::generate_base_did_details(did::PublicVerificationKey::from(delegate_auth_key.public()));

// // 	let delegator_did = did_mock::ALICE_DID;
// // 	let delegate_did = did_mock::BOB_DID;
// // 	let (root_id, root_node) = (
// // 		get_delegation_root_id(true),
// // 		generate_base_delegation_root(delegator_did.clone()),
// // 	);
// // 	let (parent_delegation_id, parent_delegation_node) = (
// // 		get_delegation_id(true),
// // 		generate_base_delegation_node(delegator_did.clone()),
// // 	);
// // 	let (delegation_id, mut delegation_node) = (
// // 		get_delegation_id(true),
// // 		generate_base_delegation_node(delegate_did.clone()),
// // 	);
// // 	delegation_node.parent = Some(get_delegation_id(false));

// // 	let delegate_signature = did::DidSignature::from(delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
// // 		&delegation_id,
// // 		&delegation_node.root_id,
// // 		&delegation_node.parent,
// // 		&delegation_node.permissions,
// // 	))));

// // 	let mut operation = generate_base_delegation_creation_operation(
// // 		delegator_did.clone(),
// // 		delegate_did.clone(),
// // 		delegate_signature.clone(),
// // 	);
// // 	operation.root_id = alternative_root_id;
// // 	let signature = delegator_del_key.sign(&operation.encode());

// // 	let builder = did_mock::ExtBuilder::default().with_dids(vec![
// // 		(delegator_did.clone(), delegator_details.clone()),
// // 		(delegate_did.clone(), delegate_details.clone()),
// // 	]);
// // 	let builder =
// // 		ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash.clone(), delegator_did.clone())]);
// // 	let builder = ExtBuilder::from(builder)
// // 		.with_root_delegations(vec![(root_id, root_node.clone())]);

// // 	let mut ext = builder.build();

// // 	ext.execute_with(|| {
// // 		assert_err!(
// // 			Delegation::submit_delegation_creation_operation(
// // 				Origin::signed(DEFAULT_ACCOUNT),
// // 				operation.clone(),
// // 				did::DidSignature::from(signature)
// // 			),
// // 			delegation::Error::<Test>::ParentDelegationNotFound
// // 		);
// // 	});

// // 	// Verify that the DID tx counter has increased
// // 	let new_delegator_details =
// // 		ext.execute_with(|| Did::get_did(&operation.caller_did).expect("DID submitter should be present on chain."));
// // 	assert_eq!(
// // 		new_delegator_details.get_tx_counter_value(),
// // 		delegator_details.get_tx_counter_value() + 1u64
// // 	);
// // }

// // #[test]
// // fn check_unauthorised_delegation_submit_delegation_creation_operation() {
// // 	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
// // 	let delegator_enc_key = did_mock::get_x25519_encryption_key(true);
// // 	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
// // 	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);
// // 	let delegate_enc_key = did_mock::get_x25519_encryption_key(false);

// // 	let mut delegator_details = did_mock::generate_base_did_details(
// // 		did::PublicVerificationKey::from(delegator_auth_key.public()),
// // 		did::PublicEncryptionKey::from(delegator_enc_key),
// // 	);
// // 	delegator_details.delegation_key =
// // Some(did::PublicVerificationKey::from(delegator_del_key.public()));

// // 	let delegate_details = did_mock::generate_base_did_details(
// // 		did::PublicVerificationKey::from(delegate_auth_key.public()),
// // 		delegate_enc_key,
// // 	);

// // 	let delegator_did = did_mock::ALICE_DID;
// // 	let delegate_did = did_mock::BOB_DID;
// // 	let ctype_hash = ctype_mock::get_ctype_hash(true);
// // 	let root_id = get_delegation_root_id(true);
// // 	let parent_id = get_delegation_id(true);
// // 	let delegation_id = get_delegation_id(false);
// // 	let permissions = delegation::Permissions::DELEGATE;
// // 	let delegate_signature =
// // delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
// // 		&delegation_id,
// // 		&root_id,
// // 		&Some(parent_id),
// // 		&permissions,
// // 	)));

// // 	let operation = delegation::DelegationCreationOperation {
// // 		caller_did: delegator_did.clone(),
// // 		delegate_did: delegate_did.clone(),
// // 		delegation_id,
// // 		parent_id: Some(parent_id),
// // 		permissions: permissions,
// // 		root_id: root_id,
// // 		delegate_signature: did::DidSignature::from(delegate_signature),
// // 		tx_counter: delegator_details.get_tx_counter_value() + 1u64,
// // 	};
// // 	let signature = delegator_del_key.sign(&operation.encode());

// // 	let builder = did_mock::ExtBuilder::default().with_dids(vec![
// // 		(delegator_did.clone(), delegator_details.clone()),
// // 		(delegate_did.clone(), delegate_details.clone()),
// // 	]);
// // 	let builder =
// // ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash,
// // delegator_did.clone())]); 	let builder = ExtBuilder::from(builder)
// // 		.with_root_delegations(vec![(
// // 			root_id,
// // 			delegation::DelegationRoot {
// // 				owner: delegator_did.clone(),
// // 				ctype_hash: ctype_hash,
// // 				revoked: false,
// // 			},
// // 		)])
// // 		.with_delegations(vec![(
// // 			parent_id,
// // 			delegation::DelegationNode {
// // 				owner: delegator_did.clone(),
// // 				parent: None,
// // 				revoked: false,
// // 				root_id: root_id,
// // 				// Only attestation is possible
// // 				permissions: delegation::Permissions::ATTEST,
// // 			},
// // 		)]);

// // 	let mut ext = builder.build();

// // 	ext.execute_with(|| {
// // 		assert_err!(
// // 			Delegation::submit_delegation_creation_operation(
// // 				Origin::signed(DEFAULT_ACCOUNT),
// // 				operation.clone(),
// // 				did::DidSignature::from(signature)
// // 			),
// // 			delegation::Error::<Test>::UnauthorizedDelegation
// // 		);
// // 	});

// // 	// Verify that the DID tx counter has increased
// // 	let new_delegator_details =
// // 		ext.execute_with(|| Did::get_did(&operation.caller_did).expect("DID submitter
// // should be present on chain.")); 	assert_eq!(
// // 		new_delegator_details.get_tx_counter_value(),
// // 		delegator_details.get_tx_counter_value() + 1u64
// // 	);
// // }

// // #[test]
// // fn check_not_owner_of_root_delegation_submit_delegation_creation_operation()
// // { 	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
// // 	let delegator_enc_key = did_mock::get_x25519_encryption_key(true);
// // 	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
// // 	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);
// // 	let delegate_enc_key = did_mock::get_x25519_encryption_key(false);

// // 	let mut delegator_details = did_mock::generate_base_did_details(
// // 		did::PublicVerificationKey::from(delegator_auth_key.public()),
// // 		did::PublicEncryptionKey::from(delegator_enc_key),
// // 	);
// // 	delegator_details.delegation_key =
// // Some(did::PublicVerificationKey::from(delegator_del_key.public()));

// // 	let delegate_details = did_mock::generate_base_did_details(
// // 		did::PublicVerificationKey::from(delegate_auth_key.public()),
// // 		delegate_enc_key,
// // 	);

// // 	let delegator_did = did_mock::ALICE_DID;
// // 	let delegate_did = did_mock::BOB_DID;
// // 	let alternative_did = did_mock::CHARLIE_DID;
// // 	let ctype_hash = ctype_mock::get_ctype_hash(true);
// // 	let root_id = get_delegation_root_id(true);
// // 	let delegation_id = get_delegation_id(false);
// // 	let permissions = delegation::Permissions::DELEGATE;
// // 	let delegate_signature =
// // delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
// // 		&delegation_id,
// // 		&root_id,
// // 		&None,
// // 		&permissions,
// // 	)));

// // 	let operation = delegation::DelegationCreationOperation {
// // 		caller_did: delegator_did.clone(),
// // 		delegate_did: delegate_did.clone(),
// // 		delegation_id,
// // 		parent_id: None,
// // 		permissions: permissions,
// // 		root_id: root_id,
// // 		delegate_signature: did::DidSignature::from(delegate_signature),
// // 		tx_counter: delegator_details.get_tx_counter_value() + 1u64,
// // 	};
// // 	let signature = delegator_del_key.sign(&operation.encode());

// // 	let builder = did_mock::ExtBuilder::default().with_dids(vec![
// // 		(delegator_did.clone(), delegator_details.clone()),
// // 		(delegate_did.clone(), delegate_details.clone()),
// // 		(alternative_did.clone(), delegate_details.clone()),
// // 	]);
// // 	let builder =
// // ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash,
// // delegator_did.clone())]); 	let builder =
// // ExtBuilder::from(builder).with_root_delegations(vec![( 		root_id,
// // 		delegation::DelegationRoot {
// // 			owner: alternative_did.clone(),
// // 			ctype_hash: ctype_hash,
// // 			revoked: false,
// // 		},
// // 	)]);

// // 	let mut ext = builder.build();

// // 	ext.execute_with(|| {
// // 		assert_err!(
// // 			Delegation::submit_delegation_creation_operation(
// // 				Origin::signed(DEFAULT_ACCOUNT),
// // 				operation.clone(),
// // 				did::DidSignature::from(signature)
// // 			),
// // 			delegation::Error::<Test>::NotOwnerOfRootDelegation
// // 		);
// // 	});

// // 	// Verify that the DID tx counter has increased
// // 	let new_delegator_details =
// // 		ext.execute_with(|| Did::get_did(&operation.caller_did).expect("DID submitter
// // should be present on chain.")); 	assert_eq!(
// // 		new_delegator_details.get_tx_counter_value(),
// // 		delegator_details.get_tx_counter_value() + 1u64
// // 	);
// // }

// // // submit_delegation_root_revocation_operation()

// // #[test]
// // fn check_list_hierarchy_submit_delegation_root_revocation_operation_successful() {
// // 	let auth_key = did_mock::get_ed25519_authentication_key(true);
// // 	let enc_key = did_mock::get_x25519_encryption_key(true);
// // 	let del_key = did_mock::get_sr25519_delegation_key(true);

// // 	let mut delegator_details = did_mock::generate_base_did_details(
// // 		did::PublicVerificationKey::from(auth_key.public()),
// // 		did::PublicEncryptionKey::from(enc_key),
// // 	);
// // 	delegator_details.delegation_key =
// // Some(did::PublicVerificationKey::from(del_key.public()));

// // 	let delegator_did = did_mock::ALICE_DID;
// // 	let ctype_hash = ctype_mock::get_ctype_hash(true);
// // 	let max_children = 2u32;
// // 	let root_id = get_delegation_root_id(true);
// // 	let delegation_1_id = get_delegation_id(true);
// // 	let delegation_1_owner_did = did_mock::BOB_DID;
// // 	let delegation_2_id = get_delegation_id(false);
// // 	let delegation_2_owner_did = did_mock::CHARLIE_DID;

// // 	let operation = delegation::DelegationRootRevocationOperation {
// // 		caller_did: delegator_did.clone(),
// // 		root_id: root_id,
// // 		max_children: max_children,
// // 		tx_counter: delegator_details.get_tx_counter_value() + 1u64,
// // 	};
// // 	let signature = del_key.sign(&operation.encode());

// // 	let builder =
// // did_mock::ExtBuilder::default().with_dids(vec![(delegator_did.clone(),
// // delegator_details.clone())]); 	let builder =
// // ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash,
// // delegator_did.clone())]); 	let builder = ExtBuilder::from(builder)
// // 		.with_root_delegations(vec![(
// // 			root_id,
// // 			delegation::DelegationRoot {
// // 				owner: delegator_did.clone(),
// // 				ctype_hash: ctype_hash,
// // 				revoked: false,
// // 			},
// // 		)])
// // 		// Root -> Delegation 1 -> Delegation 2
// // 		.with_delegations(vec![
// // 			(
// // 				delegation_1_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_1_owner_did.clone(),
// // 					parent: None,
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 			(
// // 				delegation_2_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_2_owner_did.clone(),
// // 					parent: Some(delegation_1_id),
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 		])
// // 		.with_children(vec![
// // 			(root_id, vec![delegation_1_id]),
// // 			(delegation_1_id, vec![delegation_2_id]),
// // 		]);

// // 	let mut ext = builder.build();

// // 	ext.execute_with(|| {
// // 		assert_ok!(Delegation::submit_delegation_root_revocation_operation(
// // 			Origin::signed(DEFAULT_ACCOUNT),
// // 			operation.clone(),
// // 			did::DidSignature::from(signature)
// // 		));
// // 	});

// // 	let stored_delegation_root = ext
// // 		.execute_with(|| Delegation::roots(&operation.root_id).expect("Delegation
// // root should be present on chain.")); 	assert_eq!(stored_delegation_root.
// // revoked, true);

// // 	let stored_delegation_1 = ext
// // 		.execute_with(|| Delegation::delegations(&delegation_1_id).expect("Delegation
// // 1 should be present on chain.")); 	assert_eq!(stored_delegation_1.revoked,
// // true);

// // 	let stored_delegation_2 = ext
// // 		.execute_with(|| Delegation::delegations(&delegation_2_id).expect("Delegation
// // 2 should be present on chain.")); 	assert_eq!(stored_delegation_2.revoked,
// // true);

// // 	// Verify that the DID tx counter has increased
// // 	let new_delegator_details =
// // 		ext.execute_with(|| Did::get_did(&operation.caller_did).expect("DID submitter
// // should be present on chain.")); 	assert_eq!(
// // 		new_delegator_details.get_tx_counter_value(),
// // 		delegator_details.get_tx_counter_value() + 1u64
// // 	);
// // }

// // #[test]
// // fn check_tree_hierarchy_submit_delegation_root_revocation_operation_successful() {
// // 	let auth_key = did_mock::get_ed25519_authentication_key(true);
// // 	let enc_key = did_mock::get_x25519_encryption_key(true);
// // 	let del_key = did_mock::get_sr25519_delegation_key(true);

// // 	let mut delegator_details = did_mock::generate_base_did_details(
// // 		did::PublicVerificationKey::from(auth_key.public()),
// // 		did::PublicEncryptionKey::from(enc_key),
// // 	);
// // 	delegator_details.delegation_key =
// // Some(did::PublicVerificationKey::from(del_key.public()));

// // 	let delegator_did = did_mock::ALICE_DID;
// // 	let ctype_hash = ctype_mock::get_ctype_hash(true);
// // 	let max_children = 2u32;
// // 	let root_id = get_delegation_root_id(true);
// // 	let delegation_1_id = get_delegation_id(true);
// // 	let delegation_1_owner_did = did_mock::BOB_DID;
// // 	let delegation_2_id = get_delegation_id(false);
// // 	let delegation_2_owner_did = did_mock::CHARLIE_DID;

// // 	let operation = delegation::DelegationRootRevocationOperation {
// // 		caller_did: delegator_did.clone(),
// // 		root_id: root_id,
// // 		max_children: max_children,
// // 		tx_counter: delegator_details.get_tx_counter_value() + 1u64,
// // 	};
// // 	let signature = del_key.sign(&operation.encode());

// // 	let builder =
// // did_mock::ExtBuilder::default().with_dids(vec![(delegator_did.clone(),
// // delegator_details.clone())]); 	let builder =
// // ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash,
// // delegator_did.clone())]); 	let builder = ExtBuilder::from(builder)
// // 		.with_root_delegations(vec![(
// // 			root_id,
// // 			delegation::DelegationRoot {
// // 				owner: delegator_did.clone(),
// // 				ctype_hash: ctype_hash,
// // 				revoked: false,
// // 			},
// // 		)])
// // 		// Root -> Delegation 1 && Delegation 2
// // 		.with_delegations(vec![
// // 			(
// // 				delegation_1_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_1_owner_did.clone(),
// // 					parent: None,
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 			(
// // 				delegation_2_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_2_owner_did.clone(),
// // 					parent: None,
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 		])
// // 		.with_children(vec![(root_id, vec![delegation_1_id, delegation_2_id])]);

// // 	let mut ext = builder.build();

// // 	ext.execute_with(|| {
// // 		assert_ok!(Delegation::submit_delegation_root_revocation_operation(
// // 			Origin::signed(DEFAULT_ACCOUNT),
// // 			operation.clone(),
// // 			did::DidSignature::from(signature)
// // 		));
// // 	});

// // 	let stored_delegation_root = ext
// // 		.execute_with(|| Delegation::roots(&operation.root_id).expect("Delegation
// // root should be present on chain.")); 	assert_eq!(stored_delegation_root.
// // revoked, true);

// // 	let stored_delegation_1 = ext
// // 		.execute_with(|| Delegation::delegations(&delegation_1_id).expect("Delegation
// // 1 should be present on chain.")); 	assert_eq!(stored_delegation_1.revoked,
// // true);

// // 	let stored_delegation_2 = ext
// // 		.execute_with(|| Delegation::delegations(&delegation_2_id).expect("Delegation
// // 2 should be present on chain.")); 	assert_eq!(stored_delegation_2.revoked,
// // true);

// // 	// Verify that the DID tx counter has increased
// // 	let new_delegator_details =
// // 		ext.execute_with(|| Did::get_did(&operation.caller_did).expect("DID submitter
// // should be present on chain.")); 	assert_eq!(
// // 		new_delegator_details.get_tx_counter_value(),
// // 		delegator_details.get_tx_counter_value() + 1u64
// // 	);
// // }

// // #[test]
// // fn check_greater_max_revocations_submit_delegation_root_revocation_operation_successful() {
// // 	let auth_key = did_mock::get_ed25519_authentication_key(true);
// // 	let enc_key = did_mock::get_x25519_encryption_key(true);
// // 	let del_key = did_mock::get_sr25519_delegation_key(true);

// // 	let mut delegator_details = did_mock::generate_base_did_details(
// // 		did::PublicVerificationKey::from(auth_key.public()),
// // 		did::PublicEncryptionKey::from(enc_key),
// // 	);
// // 	delegator_details.delegation_key =
// // Some(did::PublicVerificationKey::from(del_key.public()));

// // 	let delegator_did = did_mock::ALICE_DID;
// // 	let ctype_hash = ctype_mock::get_ctype_hash(true);
// // 	// max_children = 1 larger than the # of the root children
// // 	let max_children = 3u32;
// // 	let root_id = get_delegation_root_id(true);
// // 	let delegation_1_id = get_delegation_id(true);
// // 	let delegation_1_owner_did = did_mock::BOB_DID;
// // 	let delegation_2_id = get_delegation_id(false);
// // 	let delegation_2_owner_did = did_mock::CHARLIE_DID;

// // 	let operation = delegation::DelegationRootRevocationOperation {
// // 		caller_did: delegator_did.clone(),
// // 		root_id: root_id,
// // 		max_children: max_children,
// // 		tx_counter: delegator_details.get_tx_counter_value() + 1u64,
// // 	};
// // 	let signature = del_key.sign(&operation.encode());

// // 	let builder =
// // did_mock::ExtBuilder::default().with_dids(vec![(delegator_did.clone(),
// // delegator_details.clone())]); 	let builder =
// // ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash,
// // delegator_did.clone())]); 	let builder = ExtBuilder::from(builder)
// // 		.with_root_delegations(vec![(
// // 			root_id,
// // 			delegation::DelegationRoot {
// // 				owner: delegator_did.clone(),
// // 				ctype_hash: ctype_hash,
// // 				revoked: false,
// // 			},
// // 		)])
// // 		// Root -> Delegation 1 && Delegation 2
// // 		.with_delegations(vec![
// // 			(
// // 				delegation_1_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_1_owner_did.clone(),
// // 					parent: None,
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 			(
// // 				delegation_2_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_2_owner_did.clone(),
// // 					parent: None,
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 		])
// // 		.with_children(vec![(root_id, vec![delegation_1_id, delegation_2_id])]);

// // 	let mut ext = builder.build();

// // 	ext.execute_with(|| {
// // 		assert_ok!(Delegation::submit_delegation_root_revocation_operation(
// // 			Origin::signed(DEFAULT_ACCOUNT),
// // 			operation.clone(),
// // 			did::DidSignature::from(signature)
// // 		));
// // 	});

// // 	let stored_delegation_root = ext
// // 		.execute_with(|| Delegation::roots(&operation.root_id).expect("Delegation
// // root should be present on chain.")); 	assert_eq!(stored_delegation_root.
// // revoked, true);

// // 	let stored_delegation_1 = ext
// // 		.execute_with(|| Delegation::delegations(&delegation_1_id).expect("Delegation
// // 1 should be present on chain.")); 	assert_eq!(stored_delegation_1.revoked,
// // true);

// // 	let stored_delegation_2 = ext
// // 		.execute_with(|| Delegation::delegations(&delegation_2_id).expect("Delegation
// // 2 should be present on chain.")); 	assert_eq!(stored_delegation_2.revoked,
// // true);

// // 	// Verify that the DID tx counter has increased
// // 	let new_delegator_details =
// // 		ext.execute_with(|| Did::get_did(&operation.caller_did).expect("DID submitter
// // should be present on chain.")); 	assert_eq!(
// // 		new_delegator_details.get_tx_counter_value(),
// // 		delegator_details.get_tx_counter_value() + 1u64
// // 	);
// // }

// // #[test]
// // fn check_delegator_did_not_present_submit_hierarchy_delegation_root_revocation_operation_successful() {
// // 	let auth_key = did_mock::get_ed25519_authentication_key(true);
// // 	let enc_key = did_mock::get_x25519_encryption_key(true);
// // 	let del_key = did_mock::get_sr25519_delegation_key(true);

// // 	let mut delegator_details = did_mock::generate_base_did_details(
// // 		did::PublicVerificationKey::from(auth_key.public()),
// // 		did::PublicEncryptionKey::from(enc_key),
// // 	);
// // 	delegator_details.delegation_key =
// // Some(did::PublicVerificationKey::from(del_key.public()));

// // 	let delegator_did = did_mock::ALICE_DID;
// // 	let max_children = 2u32;
// // 	let root_id = get_delegation_root_id(true);

// // 	let operation = delegation::DelegationRootRevocationOperation {
// // 		caller_did: delegator_did.clone(),
// // 		root_id: root_id,
// // 		max_children: max_children,
// // 		tx_counter: delegator_details.get_tx_counter_value() + 1u64,
// // 	};
// // 	let signature = del_key.sign(&operation.encode());

// // 	// No DID added to mock storage
// // 	let builder = ExtBuilder::default();

// // 	let mut ext = builder.build();

// // 	ext.execute_with(|| {
// // 		assert_noop!(
// // 			Delegation::submit_delegation_root_revocation_operation(
// // 				Origin::signed(DEFAULT_ACCOUNT),
// // 				operation.clone(),
// // 				did::DidSignature::from(signature)
// // 			),
// // 			did::Error::<Test>::DidNotPresent
// // 		);
// // 	});
// // }

// // #[test]
// // fn check_max_did_tx_counter_submit_delegation_root_revocation_operation_successful() {
// // 	let auth_key = did_mock::get_ed25519_authentication_key(true);
// // 	let enc_key = did_mock::get_x25519_encryption_key(true);
// // 	let del_key = did_mock::get_sr25519_delegation_key(true);

// // 	let mut delegator_details = did_mock::generate_base_did_details(
// // 		did::PublicVerificationKey::from(auth_key.public()),
// // 		did::PublicEncryptionKey::from(enc_key),
// // 	);
// // 	delegator_details.delegation_key =
// // Some(did::PublicVerificationKey::from(del_key.public())); 	delegator_details.
// // set_tx_counter(u64::MAX);

// // 	let delegator_did = did_mock::ALICE_DID;
// // 	let ctype_hash = ctype_mock::get_ctype_hash(true);
// // 	let max_children = 2u32;
// // 	let root_id = get_delegation_root_id(true);
// // 	let delegation_1_id = get_delegation_id(true);
// // 	let delegation_1_owner_did = did_mock::BOB_DID;
// // 	let delegation_2_id = get_delegation_id(false);
// // 	let delegation_2_owner_did = did_mock::CHARLIE_DID;

// // 	let operation = delegation::DelegationRootRevocationOperation {
// // 		caller_did: delegator_did.clone(),
// // 		root_id: root_id,
// // 		max_children: max_children,
// // 		tx_counter: delegator_details.get_tx_counter_value(),
// // 	};
// // 	let signature = del_key.sign(&operation.encode());

// // 	let builder =
// // did_mock::ExtBuilder::default().with_dids(vec![(delegator_did.clone(),
// // delegator_details.clone())]); 	let builder =
// // ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash,
// // delegator_did.clone())]); 	let builder = ExtBuilder::from(builder)
// // 		.with_root_delegations(vec![(
// // 			root_id,
// // 			delegation::DelegationRoot {
// // 				owner: delegator_did.clone(),
// // 				ctype_hash: ctype_hash,
// // 				revoked: false,
// // 			},
// // 		)])
// // 		// Root -> Delegation 1 && Delegation 2
// // 		.with_delegations(vec![
// // 			(
// // 				delegation_1_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_1_owner_did.clone(),
// // 					parent: None,
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 			(
// // 				delegation_2_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_2_owner_did.clone(),
// // 					parent: None,
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 		])
// // 		.with_children(vec![(root_id, vec![delegation_1_id, delegation_2_id])]);

// // 	let mut ext = builder.build();

// // 	ext.execute_with(|| {
// // 		assert_noop!(
// // 			Delegation::submit_delegation_root_revocation_operation(
// // 				Origin::signed(DEFAULT_ACCOUNT),
// // 				operation.clone(),
// // 				did::DidSignature::from(signature)
// // 			),
// // 			did::Error::<Test>::MaxTxCounterValue
// // 		);
// // 	});

// // 	// Verify that the DID tx counter has not increased
// // 	let new_delegator_details =
// // 		ext.execute_with(|| Did::get_did(&operation.caller_did).expect("DID submitter
// // should be present on chain.")); 	assert_eq!(
// // 		new_delegator_details.get_tx_counter_value(),
// // 		delegator_details.get_tx_counter_value()
// // 	);
// // }

// // #[test]
// // fn check_too_small_did_tx_counter_submit_delegation_root_revocation_operation_successful() {
// // 	let auth_key = did_mock::get_ed25519_authentication_key(true);
// // 	let enc_key = did_mock::get_x25519_encryption_key(true);
// // 	let del_key = did_mock::get_sr25519_delegation_key(true);

// // 	let mut delegator_details = did_mock::generate_base_did_details(
// // 		did::PublicVerificationKey::from(auth_key.public()),
// // 		did::PublicEncryptionKey::from(enc_key),
// // 	);
// // 	delegator_details.delegation_key =
// // Some(did::PublicVerificationKey::from(del_key.public())); 	delegator_details.
// // set_tx_counter(1u64);

// // 	let delegator_did = did_mock::ALICE_DID;
// // 	let ctype_hash = ctype_mock::get_ctype_hash(true);
// // 	let max_children = 2u32;
// // 	let root_id = get_delegation_root_id(true);
// // 	let delegation_1_id = get_delegation_id(true);
// // 	let delegation_1_owner_did = did_mock::BOB_DID;
// // 	let delegation_2_id = get_delegation_id(false);
// // 	let delegation_2_owner_did = did_mock::CHARLIE_DID;

// // 	let operation = delegation::DelegationRootRevocationOperation {
// // 		caller_did: delegator_did.clone(),
// // 		root_id: root_id,
// // 		max_children: max_children,
// // 		tx_counter: 0u64,
// // 	};
// // 	let signature = del_key.sign(&operation.encode());

// // 	let builder =
// // did_mock::ExtBuilder::default().with_dids(vec![(delegator_did.clone(),
// // delegator_details.clone())]); 	let builder =
// // ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash,
// // delegator_did.clone())]); 	let builder = ExtBuilder::from(builder)
// // 		.with_root_delegations(vec![(
// // 			root_id,
// // 			delegation::DelegationRoot {
// // 				owner: delegator_did.clone(),
// // 				ctype_hash: ctype_hash,
// // 				revoked: false,
// // 			},
// // 		)])
// // 		// Root -> Delegation 1 && Delegation 2
// // 		.with_delegations(vec![
// // 			(
// // 				delegation_1_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_1_owner_did.clone(),
// // 					parent: None,
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 			(
// // 				delegation_2_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_2_owner_did.clone(),
// // 					parent: None,
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 		])
// // 		.with_children(vec![(root_id, vec![delegation_1_id, delegation_2_id])]);

// // 	let mut ext = builder.build();

// // 	ext.execute_with(|| {
// // 		assert_noop!(
// // 			Delegation::submit_delegation_root_revocation_operation(
// // 				Origin::signed(DEFAULT_ACCOUNT),
// // 				operation.clone(),
// // 				did::DidSignature::from(signature)
// // 			),
// // 			did::Error::<Test>::InvalidNonce
// // 		);
// // 	});

// // 	// Verify that the DID tx counter has not increased
// // 	let new_delegator_details =
// // 		ext.execute_with(|| Did::get_did(&operation.caller_did).expect("DID submitter
// // should be present on chain.")); 	assert_eq!(
// // 		new_delegator_details.get_tx_counter_value(),
// // 		delegator_details.get_tx_counter_value()
// // 	);
// // }

// // #[test]
// // fn check_equal_did_tx_counter_submit_delegation_root_revocation_operation_successful() {
// // 	let auth_key = did_mock::get_ed25519_authentication_key(true);
// // 	let enc_key = did_mock::get_x25519_encryption_key(true);
// // 	let del_key = did_mock::get_sr25519_delegation_key(true);

// // 	let mut delegator_details = did_mock::generate_base_did_details(
// // 		did::PublicVerificationKey::from(auth_key.public()),
// // 		did::PublicEncryptionKey::from(enc_key),
// // 	);
// // 	delegator_details.delegation_key =
// // Some(did::PublicVerificationKey::from(del_key.public()));

// // 	let delegator_did = did_mock::ALICE_DID;
// // 	let ctype_hash = ctype_mock::get_ctype_hash(true);
// // 	let max_children = 2u32;
// // 	let root_id = get_delegation_root_id(true);
// // 	let delegation_1_id = get_delegation_id(true);
// // 	let delegation_1_owner_did = did_mock::BOB_DID;
// // 	let delegation_2_id = get_delegation_id(false);
// // 	let delegation_2_owner_did = did_mock::CHARLIE_DID;

// // 	let operation = delegation::DelegationRootRevocationOperation {
// // 		caller_did: delegator_did.clone(),
// // 		root_id: root_id,
// // 		max_children: max_children,
// // 		tx_counter: delegator_details.get_tx_counter_value(),
// // 	};
// // 	let signature = del_key.sign(&operation.encode());

// // 	let builder =
// // did_mock::ExtBuilder::default().with_dids(vec![(delegator_did.clone(),
// // delegator_details.clone())]); 	let builder =
// // ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash,
// // delegator_did.clone())]); 	let builder = ExtBuilder::from(builder)
// // 		.with_root_delegations(vec![(
// // 			root_id,
// // 			delegation::DelegationRoot {
// // 				owner: delegator_did.clone(),
// // 				ctype_hash: ctype_hash,
// // 				revoked: false,
// // 			},
// // 		)])
// // 		// Root -> Delegation 1 && Delegation 2
// // 		.with_delegations(vec![
// // 			(
// // 				delegation_1_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_1_owner_did.clone(),
// // 					parent: None,
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 			(
// // 				delegation_2_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_2_owner_did.clone(),
// // 					parent: None,
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 		])
// // 		.with_children(vec![(root_id, vec![delegation_1_id, delegation_2_id])]);

// // 	let mut ext = builder.build();

// // 	ext.execute_with(|| {
// // 		assert_noop!(
// // 			Delegation::submit_delegation_root_revocation_operation(
// // 				Origin::signed(DEFAULT_ACCOUNT),
// // 				operation.clone(),
// // 				did::DidSignature::from(signature)
// // 			),
// // 			did::Error::<Test>::InvalidNonce
// // 		);
// // 	});

// // 	// Verify that the DID tx counter has not increased
// // 	let new_delegator_details =
// // 		ext.execute_with(|| Did::get_did(&operation.caller_did).expect("DID submitter
// // should be present on chain.")); 	assert_eq!(
// // 		new_delegator_details.get_tx_counter_value(),
// // 		delegator_details.get_tx_counter_value()
// // 	);
// // }

// // #[test]
// // fn check_too_large_did_tx_counter_submit_delegation_root_revocation_operation_successful() {
// // 	let auth_key = did_mock::get_ed25519_authentication_key(true);
// // 	let enc_key = did_mock::get_x25519_encryption_key(true);
// // 	let del_key = did_mock::get_sr25519_delegation_key(true);

// // 	let mut delegator_details = did_mock::generate_base_did_details(
// // 		did::PublicVerificationKey::from(auth_key.public()),
// // 		did::PublicEncryptionKey::from(enc_key),
// // 	);
// // 	delegator_details.delegation_key =
// // Some(did::PublicVerificationKey::from(del_key.public()));

// // 	let delegator_did = did_mock::ALICE_DID;
// // 	let ctype_hash = ctype_mock::get_ctype_hash(true);
// // 	let max_children = 2u32;
// // 	let root_id = get_delegation_root_id(true);
// // 	let delegation_1_id = get_delegation_id(true);
// // 	let delegation_1_owner_did = did_mock::BOB_DID;
// // 	let delegation_2_id = get_delegation_id(false);
// // 	let delegation_2_owner_did = did_mock::CHARLIE_DID;

// // 	let operation = delegation::DelegationRootRevocationOperation {
// // 		caller_did: delegator_did.clone(),
// // 		root_id: root_id,
// // 		max_children: max_children,
// // 		tx_counter: delegator_details.get_tx_counter_value() + 2u64,
// // 	};
// // 	let signature = del_key.sign(&operation.encode());

// // 	let builder =
// // did_mock::ExtBuilder::default().with_dids(vec![(delegator_did.clone(),
// // delegator_details.clone())]); 	let builder =
// // ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash,
// // delegator_did.clone())]); 	let builder = ExtBuilder::from(builder)
// // 		.with_root_delegations(vec![(
// // 			root_id,
// // 			delegation::DelegationRoot {
// // 				owner: delegator_did.clone(),
// // 				ctype_hash: ctype_hash,
// // 				revoked: false,
// // 			},
// // 		)])
// // 		// Root -> Delegation 1 && Delegation 2
// // 		.with_delegations(vec![
// // 			(
// // 				delegation_1_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_1_owner_did.clone(),
// // 					parent: None,
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 			(
// // 				delegation_2_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_2_owner_did.clone(),
// // 					parent: None,
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 		])
// // 		.with_children(vec![(root_id, vec![delegation_1_id, delegation_2_id])]);

// // 	let mut ext = builder.build();

// // 	ext.execute_with(|| {
// // 		assert_noop!(
// // 			Delegation::submit_delegation_root_revocation_operation(
// // 				Origin::signed(DEFAULT_ACCOUNT),
// // 				operation.clone(),
// // 				did::DidSignature::from(signature)
// // 			),
// // 			did::Error::<Test>::InvalidNonce
// // 		);
// // 	});

// // 	// Verify that the DID tx counter has not increased
// // 	let new_delegator_details =
// // 		ext.execute_with(|| Did::get_did(&operation.caller_did).expect("DID submitter
// // should be present on chain.")); 	assert_eq!(
// // 		new_delegator_details.get_tx_counter_value(),
// // 		delegator_details.get_tx_counter_value()
// // 	);
// // }

// // #[test]
// // fn check_delegation_key_not_present_submit_delegation_root_revocation_operation_successful() {
// // 	let auth_key = did_mock::get_ed25519_authentication_key(true);
// // 	let enc_key = did_mock::get_x25519_encryption_key(true);
// // 	let del_key = did_mock::get_sr25519_delegation_key(true);

// // 	let delegator_details = did_mock::generate_base_did_details(
// // 		did::PublicVerificationKey::from(auth_key.public()),
// // 		did::PublicEncryptionKey::from(enc_key),
// // 	);
// // 	// Not setting the delegation key for the DID

// // 	let delegator_did = did_mock::ALICE_DID;
// // 	let ctype_hash = ctype_mock::get_ctype_hash(true);
// // 	let max_children = 2u32;
// // 	let root_id = get_delegation_root_id(true);
// // 	let delegation_1_id = get_delegation_id(true);
// // 	let delegation_1_owner_did = did_mock::BOB_DID;
// // 	let delegation_2_id = get_delegation_id(false);
// // 	let delegation_2_owner_did = did_mock::CHARLIE_DID;

// // 	let operation = delegation::DelegationRootRevocationOperation {
// // 		caller_did: delegator_did.clone(),
// // 		root_id: root_id,
// // 		max_children: max_children,
// // 		tx_counter: delegator_details.get_tx_counter_value() + 1u64,
// // 	};
// // 	let signature = del_key.sign(&operation.encode());

// // 	let builder =
// // did_mock::ExtBuilder::default().with_dids(vec![(delegator_did.clone(),
// // delegator_details.clone())]); 	let builder =
// // ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash,
// // delegator_did.clone())]); 	let builder = ExtBuilder::from(builder)
// // 		.with_root_delegations(vec![(
// // 			root_id,
// // 			delegation::DelegationRoot {
// // 				owner: delegator_did.clone(),
// // 				ctype_hash: ctype_hash,
// // 				revoked: false,
// // 			},
// // 		)])
// // 		// Root -> Delegation 1 && Delegation 2
// // 		.with_delegations(vec![
// // 			(
// // 				delegation_1_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_1_owner_did.clone(),
// // 					parent: None,
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 			(
// // 				delegation_2_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_2_owner_did.clone(),
// // 					parent: None,
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 		])
// // 		.with_children(vec![(root_id, vec![delegation_1_id, delegation_2_id])]);

// // 	let mut ext = builder.build();

// // 	ext.execute_with(|| {
// // 		assert_noop!(
// // 			Delegation::submit_delegation_root_revocation_operation(
// // 				Origin::signed(DEFAULT_ACCOUNT),
// // 				operation.clone(),
// // 				did::DidSignature::from(signature)
// // 			),
// // 			did::Error::<Test>::VerificationKeysNotPresent
// // 		);
// // 	});

// // 	// Verify that the DID tx counter has not increased
// // 	let new_delegator_details =
// // 		ext.execute_with(|| Did::get_did(&operation.caller_did).expect("DID submitter
// // should be present on chain.")); 	assert_eq!(
// // 		new_delegator_details.get_tx_counter_value(),
// // 		delegator_details.get_tx_counter_value()
// // 	);
// // }

// // #[test]
// // fn check_invalid_signature_format_submit_delegation_root_revocation_operation_successful() {
// // 	let auth_key = did_mock::get_ed25519_authentication_key(true);
// // 	let enc_key = did_mock::get_x25519_encryption_key(true);
// // 	let del_key = did_mock::get_sr25519_delegation_key(true);
// // 	let alternative_del_key = did_mock::get_ed25519_delegation_key(true);

// // 	let mut delegator_details = did_mock::generate_base_did_details(
// // 		did::PublicVerificationKey::from(auth_key.public()),
// // 		did::PublicEncryptionKey::from(enc_key),
// // 	);
// // 	delegator_details.delegation_key =
// // Some(did::PublicVerificationKey::from(del_key.public()));

// // 	let delegator_did = did_mock::ALICE_DID;
// // 	let ctype_hash = ctype_mock::get_ctype_hash(true);
// // 	let max_children = 2u32;
// // 	let root_id = get_delegation_root_id(true);
// // 	let delegation_1_id = get_delegation_id(true);
// // 	let delegation_1_owner_did = did_mock::BOB_DID;
// // 	let delegation_2_id = get_delegation_id(false);
// // 	let delegation_2_owner_did = did_mock::CHARLIE_DID;

// // 	let operation = delegation::DelegationRootRevocationOperation {
// // 		caller_did: delegator_did.clone(),
// // 		root_id: root_id,
// // 		max_children: max_children,
// // 		tx_counter: delegator_details.get_tx_counter_value() + 1u64,
// // 	};
// // 	let signature = alternative_del_key.sign(&operation.encode());

// // 	let builder =
// // did_mock::ExtBuilder::default().with_dids(vec![(delegator_did.clone(),
// // delegator_details.clone())]); 	let builder =
// // ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash,
// // delegator_did.clone())]); 	let builder = ExtBuilder::from(builder)
// // 		.with_root_delegations(vec![(
// // 			root_id,
// // 			delegation::DelegationRoot {
// // 				owner: delegator_did.clone(),
// // 				ctype_hash: ctype_hash,
// // 				revoked: false,
// // 			},
// // 		)])
// // 		// Root -> Delegation 1 && Delegation 2
// // 		.with_delegations(vec![
// // 			(
// // 				delegation_1_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_1_owner_did.clone(),
// // 					parent: None,
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 			(
// // 				delegation_2_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_2_owner_did.clone(),
// // 					parent: None,
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 		])
// // 		.with_children(vec![(root_id, vec![delegation_1_id, delegation_2_id])]);

// // 	let mut ext = builder.build();

// // 	ext.execute_with(|| {
// // 		assert_noop!(
// // 			Delegation::submit_delegation_root_revocation_operation(
// // 				Origin::signed(DEFAULT_ACCOUNT),
// // 				operation.clone(),
// // 				did::DidSignature::from(signature)
// // 			),
// // 			did::Error::<Test>::InvalidSignatureFormat
// // 		);
// // 	});

// // 	// Verify that the DID tx counter has not increased
// // 	let new_delegator_details =
// // 		ext.execute_with(|| Did::get_did(&operation.caller_did).expect("DID submitter
// // should be present on chain.")); 	assert_eq!(
// // 		new_delegator_details.get_tx_counter_value(),
// // 		delegator_details.get_tx_counter_value()
// // 	);
// // }

// // #[test]
// // fn check_invalid_signature_submit_delegation_root_revocation_operation_successful() {
// // 	let auth_key = did_mock::get_ed25519_authentication_key(true);
// // 	let enc_key = did_mock::get_x25519_encryption_key(true);
// // 	let del_key = did_mock::get_sr25519_delegation_key(true);
// // 	let alternative_del_key = did_mock::get_sr25519_delegation_key(false);

// // 	let mut delegator_details = did_mock::generate_base_did_details(
// // 		did::PublicVerificationKey::from(auth_key.public()),
// // 		did::PublicEncryptionKey::from(enc_key),
// // 	);
// // 	delegator_details.delegation_key =
// // Some(did::PublicVerificationKey::from(del_key.public()));

// // 	let delegator_did = did_mock::ALICE_DID;
// // 	let ctype_hash = ctype_mock::get_ctype_hash(true);
// // 	let max_children = 2u32;
// // 	let root_id = get_delegation_root_id(true);
// // 	let delegation_1_id = get_delegation_id(true);
// // 	let delegation_1_owner_did = did_mock::BOB_DID;
// // 	let delegation_2_id = get_delegation_id(false);
// // 	let delegation_2_owner_did = did_mock::CHARLIE_DID;

// // 	let operation = delegation::DelegationRootRevocationOperation {
// // 		caller_did: delegator_did.clone(),
// // 		root_id: root_id,
// // 		max_children: max_children,
// // 		tx_counter: delegator_details.get_tx_counter_value() + 1u64,
// // 	};
// // 	let signature = alternative_del_key.sign(&operation.encode());

// // 	let builder =
// // did_mock::ExtBuilder::default().with_dids(vec![(delegator_did.clone(),
// // delegator_details.clone())]); 	let builder =
// // ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash,
// // delegator_did.clone())]); 	let builder = ExtBuilder::from(builder)
// // 		.with_root_delegations(vec![(
// // 			root_id,
// // 			delegation::DelegationRoot {
// // 				owner: delegator_did.clone(),
// // 				ctype_hash: ctype_hash,
// // 				revoked: false,
// // 			},
// // 		)])
// // 		// Root -> Delegation 1 && Delegation 2
// // 		.with_delegations(vec![
// // 			(
// // 				delegation_1_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_1_owner_did.clone(),
// // 					parent: None,
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 			(
// // 				delegation_2_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_2_owner_did.clone(),
// // 					parent: None,
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 		])
// // 		.with_children(vec![(root_id, vec![delegation_1_id, delegation_2_id])]);

// // 	let mut ext = builder.build();

// // 	ext.execute_with(|| {
// // 		assert_noop!(
// // 			Delegation::submit_delegation_root_revocation_operation(
// // 				Origin::signed(DEFAULT_ACCOUNT),
// // 				operation.clone(),
// // 				did::DidSignature::from(signature)
// // 			),
// // 			did::Error::<Test>::InvalidSignature
// // 		);
// // 	});

// // 	// Verify that the DID tx counter has not increased
// // 	let new_delegator_details =
// // 		ext.execute_with(|| Did::get_did(&operation.caller_did).expect("DID submitter
// // should be present on chain.")); 	assert_eq!(
// // 		new_delegator_details.get_tx_counter_value(),
// // 		delegator_details.get_tx_counter_value()
// // 	);
// // }

// // #[test]
// // fn check_root_not_found_submit_delegation_root_revocation_operation_successful() {
// // 	let auth_key = did_mock::get_ed25519_authentication_key(true);
// // 	let enc_key = did_mock::get_x25519_encryption_key(true);
// // 	let del_key = did_mock::get_sr25519_delegation_key(true);

// // 	let mut delegator_details = did_mock::generate_base_did_details(
// // 		did::PublicVerificationKey::from(auth_key.public()),
// // 		did::PublicEncryptionKey::from(enc_key),
// // 	);
// // 	delegator_details.delegation_key =
// // Some(did::PublicVerificationKey::from(del_key.public()));

// // 	let delegator_did = did_mock::ALICE_DID;
// // 	let max_children = 2u32;
// // 	let root_id = get_delegation_root_id(true);

// // 	let operation = delegation::DelegationRootRevocationOperation {
// // 		caller_did: delegator_did.clone(),
// // 		root_id: root_id,
// // 		max_children: max_children,
// // 		tx_counter: delegator_details.get_tx_counter_value() + 1u64,
// // 	};
// // 	let signature = del_key.sign(&operation.encode());

// // 	// No root is added to the mock storage
// // 	let builder =
// // did_mock::ExtBuilder::default().with_dids(vec![(delegator_did.clone(),
// // delegator_details.clone())]); 	let builder =
// // ctype_mock::ExtBuilder::from(builder);

// // 	let mut ext = builder.build();

// // 	ext.execute_with(|| {
// // 		assert_err!(
// // 			Delegation::submit_delegation_root_revocation_operation(
// // 				Origin::signed(DEFAULT_ACCOUNT),
// // 				operation.clone(),
// // 				did::DidSignature::from(signature)
// // 			),
// // 			delegation::Error::<Test>::RootNotFound
// // 		);
// // 	});

// // 	// Verify that the DID tx counter has increased
// // 	let new_delegator_details =
// // 		ext.execute_with(|| Did::get_did(&operation.caller_did).expect("DID submitter
// // should be present on chain.")); 	assert_eq!(
// // 		new_delegator_details.get_tx_counter_value(),
// // 		delegator_details.get_tx_counter_value() + 1u64
// // 	);
// // }

// // #[test]
// // fn check_different_root_creator_submit_delegation_root_revocation_operation_successful() {
// // 	let auth_key = did_mock::get_ed25519_authentication_key(true);
// // 	let enc_key = did_mock::get_x25519_encryption_key(true);
// // 	let del_key = did_mock::get_sr25519_delegation_key(true);

// // 	let mut delegator_details = did_mock::generate_base_did_details(
// // 		did::PublicVerificationKey::from(auth_key.public()),
// // 		did::PublicEncryptionKey::from(enc_key),
// // 	);
// // 	delegator_details.delegation_key =
// // Some(did::PublicVerificationKey::from(del_key.public()));

// // 	let delegator_did = did_mock::ALICE_DID;
// // 	let alternative_owner_did = did_mock::BOB_DID;
// // 	let ctype_hash = ctype_mock::get_ctype_hash(true);
// // 	let max_children = 2u32;
// // 	let root_id = get_delegation_root_id(true);

// // 	let operation = delegation::DelegationRootRevocationOperation {
// // 		caller_did: delegator_did.clone(),
// // 		root_id: root_id,
// // 		max_children: max_children,
// // 		tx_counter: delegator_details.get_tx_counter_value() + 1u64,
// // 	};
// // 	let signature = del_key.sign(&operation.encode());

// // 	let builder = did_mock::ExtBuilder::default().with_dids(vec![
// // 		(delegator_did.clone(), delegator_details.clone()),
// // 		(alternative_owner_did.clone(), delegator_details.clone()),
// // 	]);
// // 	let builder =
// // ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash,
// // delegator_did.clone())]); 	let builder =
// // ExtBuilder::from(builder).with_root_delegations(vec![( 		root_id,
// // 		delegation::DelegationRoot {
// // 			owner: alternative_owner_did.clone(),
// // 			ctype_hash: ctype_hash,
// // 			revoked: false,
// // 		},
// // 	)]);

// // 	let mut ext = builder.build();

// // 	ext.execute_with(|| {
// // 		assert_err!(
// // 			Delegation::submit_delegation_root_revocation_operation(
// // 				Origin::signed(DEFAULT_ACCOUNT),
// // 				operation.clone(),
// // 				did::DidSignature::from(signature)
// // 			),
// // 			delegation::Error::<Test>::UnauthorizedRevocation
// // 		);
// // 	});

// // 	// Verify that the DID tx counter has increased
// // 	let new_delegator_details =
// // 		ext.execute_with(|| Did::get_did(&operation.caller_did).expect("DID submitter
// // should be present on chain.")); 	assert_eq!(
// // 		new_delegator_details.get_tx_counter_value(),
// // 		delegator_details.get_tx_counter_value() + 1u64
// // 	);
// // }

// // #[test]
// // fn check_too_small_max_revocations_submit_delegation_root_revocation_operation_successful() {
// // 	let auth_key = did_mock::get_ed25519_authentication_key(true);
// // 	let enc_key = did_mock::get_x25519_encryption_key(true);
// // 	let del_key = did_mock::get_sr25519_delegation_key(true);

// // 	let mut delegator_details = did_mock::generate_base_did_details(
// // 		did::PublicVerificationKey::from(auth_key.public()),
// // 		did::PublicEncryptionKey::from(enc_key),
// // 	);
// // 	delegator_details.delegation_key =
// // Some(did::PublicVerificationKey::from(del_key.public()));

// // 	let delegator_did = did_mock::ALICE_DID;
// // 	let ctype_hash = ctype_mock::get_ctype_hash(true);
// // 	let max_children = 0u32;
// // 	let root_id = get_delegation_root_id(true);
// // 	let delegation_1_id = get_delegation_id(true);
// // 	let delegation_1_owner_did = did_mock::BOB_DID;
// // 	let delegation_2_id = get_delegation_id(false);
// // 	let delegation_3_id = get_delegation_root_id(false);
// // 	let delegation_2_owner_did = did_mock::CHARLIE_DID;

// // 	let operation = delegation::DelegationRootRevocationOperation {
// // 		caller_did: delegator_did.clone(),
// // 		root_id: root_id,
// // 		max_children: max_children,
// // 		tx_counter: delegator_details.get_tx_counter_value() + 1u64,
// // 	};
// // 	let signature = del_key.sign(&operation.encode());

// // 	let builder =
// // did_mock::ExtBuilder::default().with_dids(vec![(delegator_did.clone(),
// // delegator_details.clone())]); 	let builder =
// // ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash,
// // delegator_did.clone())]); 	let builder = ExtBuilder::from(builder)
// // 		.with_root_delegations(vec![(
// // 			root_id,
// // 			delegation::DelegationRoot {
// // 				owner: delegator_did.clone(),
// // 				ctype_hash: ctype_hash,
// // 				revoked: false,
// // 			},
// // 		)])
// // 		// Root -> Delegation 1 -> Delegation 2 -> Delegation 3
// // 		.with_delegations(vec![
// // 			(
// // 				delegation_1_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_1_owner_did.clone(),
// // 					parent: None,
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 			(
// // 				delegation_2_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_2_owner_did.clone(),
// // 					parent: Some(delegation_1_id),
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 			(
// // 				delegation_3_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_2_owner_did.clone(),
// // 					parent: Some(delegation_2_id),
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 		])
// // 		.with_children(vec![
// // 			(root_id, vec![delegation_1_id]),
// // 			(delegation_1_id, vec![delegation_2_id]),
// // 			(delegation_2_id, vec![delegation_3_id]),
// // 		]);

// // 	let mut ext = builder.build();

// // 	ext.execute_with(|| {
// // 		assert_err!(
// // 			Delegation::submit_delegation_root_revocation_operation(
// // 				Origin::signed(DEFAULT_ACCOUNT),
// // 				operation.clone(),
// // 				did::DidSignature::from(signature)
// // 			),
// // 			delegation::Error::<Test>::ExceededRevocationBounds
// // 		);
// // 	});

// // 	// Verify that the DID tx counter has increased
// // 	let new_delegator_details =
// // 		ext.execute_with(|| Did::get_did(&operation.caller_did).expect("DID submitter
// // should be present on chain.")); 	assert_eq!(
// // 		new_delegator_details.get_tx_counter_value(),
// // 		delegator_details.get_tx_counter_value() + 1u64
// // 	);
// // }

// // #[test]
// // fn check_exact_children_max_revocations_submit_delegation_root_revocation_operation_successful() {
// // 	let auth_key = did_mock::get_ed25519_authentication_key(true);
// // 	let enc_key = did_mock::get_x25519_encryption_key(true);
// // 	let del_key = did_mock::get_sr25519_delegation_key(true);

// // 	let mut delegator_details = did_mock::generate_base_did_details(
// // 		did::PublicVerificationKey::from(auth_key.public()),
// // 		did::PublicEncryptionKey::from(enc_key),
// // 	);
// // 	delegator_details.delegation_key =
// // Some(did::PublicVerificationKey::from(del_key.public()));

// // 	let delegator_did = did_mock::ALICE_DID;
// // 	let ctype_hash = ctype_mock::get_ctype_hash(true);
// // 	let max_children = 2u32;
// // 	let root_id = get_delegation_root_id(true);
// // 	let delegation_1_id = get_delegation_id(true);
// // 	let delegation_1_owner_did = did_mock::BOB_DID;
// // 	let delegation_2_id = get_delegation_id(false);
// // 	let delegation_3_id = get_delegation_root_id(false);
// // 	let delegation_2_owner_did = did_mock::CHARLIE_DID;

// // 	let operation = delegation::DelegationRootRevocationOperation {
// // 		caller_did: delegator_did.clone(),
// // 		root_id: root_id,
// // 		max_children: max_children,
// // 		tx_counter: delegator_details.get_tx_counter_value() + 1u64,
// // 	};
// // 	let signature = del_key.sign(&operation.encode());

// // 	let builder =
// // did_mock::ExtBuilder::default().with_dids(vec![(delegator_did.clone(),
// // delegator_details.clone())]); 	let builder =
// // ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash,
// // delegator_did.clone())]); 	let builder = ExtBuilder::from(builder)
// // 		.with_root_delegations(vec![(
// // 			root_id,
// // 			delegation::DelegationRoot {
// // 				owner: delegator_did.clone(),
// // 				ctype_hash: ctype_hash,
// // 				revoked: false,
// // 			},
// // 		)])
// // 		// Root -> Delegation 1 -> Delegation 2 && Delegation 3
// // 		.with_delegations(vec![
// // 			(
// // 				delegation_1_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_1_owner_did.clone(),
// // 					parent: None,
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 			(
// // 				delegation_2_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_2_owner_did.clone(),
// // 					parent: Some(delegation_1_id),
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 			(
// // 				delegation_3_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_2_owner_did.clone(),
// // 					parent: Some(delegation_1_id),
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 		])
// // 		.with_children(vec![
// // 			(root_id, vec![delegation_1_id]),
// // 			(delegation_1_id, vec![delegation_2_id, delegation_3_id]),
// // 		]);

// // 	let mut ext = builder.build();

// // 	ext.execute_with(|| {
// // 		// Storage changes, but extrinsic still returns an error.
// // 		assert_err!(
// // 			Delegation::submit_delegation_root_revocation_operation(
// // 				Origin::signed(DEFAULT_ACCOUNT),
// // 				operation.clone(),
// // 				did::DidSignature::from(signature)
// // 			),
// // 			delegation::Error::<Test>::ExceededRevocationBounds
// // 		);
// // 	});

// // 	// Only one leaf should have been revoked
// // 	let stored_delegation_root = ext
// // 		.execute_with(|| Delegation::roots(&operation.root_id).expect("Delegation
// // root should be present on chain.")); 	assert_eq!(stored_delegation_root.
// // revoked, false);

// // 	let stored_delegation_1 = ext
// // 		.execute_with(|| Delegation::delegations(&delegation_1_id).expect("Delegation
// // 1 should be present on chain.")); 	assert_eq!(stored_delegation_1.revoked,
// // false);

// // 	let stored_delegation_2 = ext
// // 		.execute_with(|| Delegation::delegations(&delegation_2_id).expect("Delegation
// // 2 should be present on chain.")); 	assert_eq!(stored_delegation_2.revoked,
// // true);

// // 	let stored_delegation_3 = ext
// // 		.execute_with(|| Delegation::delegations(&delegation_3_id).expect("Delegation
// // 3 should be present on chain.")); 	assert_eq!(stored_delegation_3.revoked,
// // false);

// // 	// Verify that the DID tx counter has increased
// // 	let new_delegator_details =
// // 		ext.execute_with(|| Did::get_did(&operation.caller_did).expect("DID submitter
// // should be present on chain.")); 	assert_eq!(
// // 		new_delegator_details.get_tx_counter_value(),
// // 		delegator_details.get_tx_counter_value() + 1u64
// // 	);
// // }

// // // submit_delegation_revocation_operation()

// // #[test]
// // fn check_direct_owner_submit_delegation_revocation_operation_successful() {
// // 	let auth_key = did_mock::get_ed25519_authentication_key(true);
// // 	let enc_key = did_mock::get_x25519_encryption_key(true);
// // 	let del_key = did_mock::get_sr25519_delegation_key(true);

// // 	let mut delegator_details = did_mock::generate_base_did_details(
// // 		did::PublicVerificationKey::from(auth_key.public()),
// // 		did::PublicEncryptionKey::from(enc_key),
// // 	);
// // 	delegator_details.delegation_key =
// // Some(did::PublicVerificationKey::from(del_key.public()));

// // 	let delegator_did = did_mock::ALICE_DID;
// // 	let ctype_hash = ctype_mock::get_ctype_hash(true);
// // 	let max_depth = 0u32;
// // 	let max_revocations = 2u32;
// // 	let root_id = get_delegation_root_id(true);
// // 	let delegation_1_id = get_delegation_id(true);
// // 	let delegation_1_owner_did = did_mock::BOB_DID;
// // 	let delegation_2_id = get_delegation_id(false);
// // 	let delegation_2_owner_did = did_mock::CHARLIE_DID;

// // 	let operation = delegation::DelegationRevocationOperation {
// // 		caller_did: delegation_1_owner_did.clone(),
// // 		delegation_id: delegation_1_id,
// // 		max_parent_checks: max_depth,
// // 		max_revocations: max_revocations,
// // 		tx_counter: delegator_details.get_tx_counter_value() + 1u64,
// // 	};
// // 	let signature = del_key.sign(&operation.encode());

// // 	let builder = did_mock::ExtBuilder::default().with_dids(vec![
// // 		(delegator_did.clone(), delegator_details.clone()),
// // 		(delegation_1_owner_did.clone(), delegator_details.clone()),
// // 		(delegation_2_owner_did.clone(), delegator_details.clone()),
// // 	]);
// // 	let builder =
// // ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash,
// // delegator_did.clone())]); 	let builder = ExtBuilder::from(builder)
// // 		.with_root_delegations(vec![(
// // 			root_id,
// // 			delegation::DelegationRoot {
// // 				owner: delegator_did.clone(),
// // 				ctype_hash: ctype_hash,
// // 				revoked: false,
// // 			},
// // 		)])
// // 		// Root -> Delegation 1 -> Delegation 2
// // 		.with_delegations(vec![
// // 			(
// // 				delegation_1_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_1_owner_did.clone(),
// // 					parent: None,
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 			(
// // 				delegation_2_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_2_owner_did.clone(),
// // 					parent: Some(delegation_1_id),
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 		])
// // 		.with_children(vec![
// // 			(root_id, vec![delegation_1_id]),
// // 			(delegation_1_id, vec![delegation_2_id]),
// // 		]);

// // 	let mut ext = builder.build();

// // 	ext.execute_with(|| {
// // 		assert_ok!(Delegation::submit_delegation_revocation_operation(
// // 			Origin::signed(DEFAULT_ACCOUNT),
// // 			operation.clone(),
// // 			did::DidSignature::from(signature)
// // 		));
// // 	});

// // 	let stored_delegation_1 = ext
// // 		.execute_with(|| Delegation::delegations(&delegation_1_id).expect("Delegation
// // 1 should be present on chain.")); 	assert_eq!(stored_delegation_1.revoked,
// // true);

// // 	let stored_delegation_2 = ext
// // 		.execute_with(|| Delegation::delegations(&delegation_2_id).expect("Delegation
// // 2 should be present on chain.")); 	assert_eq!(stored_delegation_2.revoked,
// // true);

// // 	// Verify that the DID tx counter has increased
// // 	let new_delegator_details =
// // 		ext.execute_with(|| Did::get_did(&operation.caller_did).expect("DID submitter
// // should be present on chain.")); 	assert_eq!(
// // 		new_delegator_details.get_tx_counter_value(),
// // 		delegator_details.get_tx_counter_value() + 1u64
// // 	);
// // }

// // #[test]
// // fn check_parent_owner_submit_delegation_revocation_operation_successful() {
// // 	let auth_key = did_mock::get_ed25519_authentication_key(true);
// // 	let enc_key = did_mock::get_x25519_encryption_key(true);
// // 	let del_key = did_mock::get_sr25519_delegation_key(true);

// // 	let mut delegator_details = did_mock::generate_base_did_details(
// // 		did::PublicVerificationKey::from(auth_key.public()),
// // 		did::PublicEncryptionKey::from(enc_key),
// // 	);
// // 	delegator_details.delegation_key =
// // Some(did::PublicVerificationKey::from(del_key.public()));

// // 	let delegator_did = did_mock::ALICE_DID;
// // 	let ctype_hash = ctype_mock::get_ctype_hash(true);
// // 	let max_depth = 1u32;
// // 	let max_revocations = 2u32;
// // 	let root_id = get_delegation_root_id(true);
// // 	let delegation_1_id = get_delegation_id(true);
// // 	let delegation_1_owner_did = did_mock::BOB_DID;
// // 	let delegation_2_id = get_delegation_id(false);
// // 	let delegation_2_owner_did = did_mock::CHARLIE_DID;

// // 	let operation = delegation::DelegationRevocationOperation {
// // 		caller_did: delegation_1_owner_did.clone(),
// // 		// Removing delegation_2, a child of delegation_1
// // 		delegation_id: delegation_2_id,
// // 		max_parent_checks: max_depth,
// // 		max_revocations: max_revocations,
// // 		tx_counter: delegator_details.get_tx_counter_value() + 1u64,
// // 	};
// // 	let signature = del_key.sign(&operation.encode());

// // 	let builder = did_mock::ExtBuilder::default().with_dids(vec![
// // 		(delegator_did.clone(), delegator_details.clone()),
// // 		(delegation_1_owner_did.clone(), delegator_details.clone()),
// // 		(delegation_2_owner_did.clone(), delegator_details.clone()),
// // 	]);
// // 	let builder =
// // ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash,
// // delegator_did.clone())]); 	let builder = ExtBuilder::from(builder)
// // 		.with_root_delegations(vec![(
// // 			root_id,
// // 			delegation::DelegationRoot {
// // 				owner: delegator_did.clone(),
// // 				ctype_hash: ctype_hash,
// // 				revoked: false,
// // 			},
// // 		)])
// // 		// Root -> Delegation 1 -> Delegation 2
// // 		.with_delegations(vec![
// // 			(
// // 				delegation_1_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_1_owner_did.clone(),
// // 					parent: None,
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 			(
// // 				delegation_2_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_2_owner_did.clone(),
// // 					parent: Some(delegation_1_id),
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 		])
// // 		.with_children(vec![
// // 			(root_id, vec![delegation_1_id]),
// // 			(delegation_1_id, vec![delegation_2_id]),
// // 		]);

// // 	let mut ext = builder.build();

// // 	ext.execute_with(|| {
// // 		assert_ok!(Delegation::submit_delegation_revocation_operation(
// // 			Origin::signed(DEFAULT_ACCOUNT),
// // 			operation.clone(),
// // 			did::DidSignature::from(signature)
// // 		));
// // 	});

// // 	let stored_delegation_1 = ext
// // 		.execute_with(|| Delegation::delegations(&delegation_1_id).expect("Delegation
// // 1 should be present on chain.")); 	assert_eq!(stored_delegation_1.revoked,
// // false);

// // 	// Only delegation_1 is revoked
// // 	let stored_delegation_2 = ext
// // 		.execute_with(|| Delegation::delegations(&delegation_2_id).expect("Delegation
// // 2 should be present on chain.")); 	assert_eq!(stored_delegation_2.revoked,
// // true);

// // 	// Verify that the DID tx counter has increased
// // 	let new_delegator_details =
// // 		ext.execute_with(|| Did::get_did(&operation.caller_did).expect("DID submitter
// // should be present on chain.")); 	assert_eq!(
// // 		new_delegator_details.get_tx_counter_value(),
// // 		delegator_details.get_tx_counter_value() + 1u64
// // 	);
// // }

// // #[test]
// // fn check_delegator_did_not_present_submit_delegation_revocation_operation_successful() {
// // 	let auth_key = did_mock::get_ed25519_authentication_key(true);
// // 	let enc_key = did_mock::get_x25519_encryption_key(true);
// // 	let del_key = did_mock::get_sr25519_delegation_key(true);

// // 	let mut delegator_details = did_mock::generate_base_did_details(
// // 		did::PublicVerificationKey::from(auth_key.public()),
// // 		did::PublicEncryptionKey::from(enc_key),
// // 	);
// // 	delegator_details.delegation_key =
// // Some(did::PublicVerificationKey::from(del_key.public()));

// // 	let delegator_did = did_mock::ALICE_DID;
// // 	let ctype_hash = ctype_mock::get_ctype_hash(true);
// // 	let max_depth = 1u32;
// // 	let max_revocations = 2u32;
// // 	let root_id = get_delegation_root_id(true);
// // 	let delegation_1_id = get_delegation_id(true);
// // 	let delegation_1_owner_did = did_mock::BOB_DID;
// // 	let delegation_2_id = get_delegation_id(false);
// // 	let delegation_2_owner_did = did_mock::CHARLIE_DID;

// // 	let operation = delegation::DelegationRevocationOperation {
// // 		caller_did: delegator_did.clone(),
// // 		// Removing delegation_2, a child of delegation_1
// // 		delegation_id: delegation_2_id,
// // 		max_parent_checks: max_depth,
// // 		max_revocations: max_revocations,
// // 		tx_counter: delegator_details.get_tx_counter_value() + 1u64,
// // 	};
// // 	let signature = del_key.sign(&operation.encode());

// // 	let builder = did_mock::ExtBuilder::default().with_dids(vec![
// // 		// No delegator DID in the mock storage
// // 		(delegation_1_owner_did.clone(), delegator_details.clone()),
// // 		(delegation_2_owner_did.clone(), delegator_details.clone()),
// // 	]);
// // 	let builder =
// // ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash,
// // delegator_did.clone())]); 	let builder = ExtBuilder::from(builder)
// // 		.with_root_delegations(vec![(
// // 			root_id,
// // 			delegation::DelegationRoot {
// // 				owner: delegation_1_owner_did.clone(),
// // 				ctype_hash: ctype_hash,
// // 				revoked: false,
// // 			},
// // 		)])
// // 		// Root -> Delegation 1 -> Delegation 2
// // 		.with_delegations(vec![
// // 			(
// // 				delegation_1_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_1_owner_did.clone(),
// // 					parent: None,
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 			(
// // 				delegation_2_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_2_owner_did.clone(),
// // 					parent: Some(delegation_1_id),
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 		])
// // 		.with_children(vec![
// // 			(root_id, vec![delegation_1_id]),
// // 			(delegation_1_id, vec![delegation_2_id]),
// // 		]);

// // 	let mut ext = builder.build();

// // 	ext.execute_with(|| {
// // 		assert_noop!(
// // 			Delegation::submit_delegation_revocation_operation(
// // 				Origin::signed(DEFAULT_ACCOUNT),
// // 				operation.clone(),
// // 				did::DidSignature::from(signature)
// // 			),
// // 			did::Error::<Test>::DidNotPresent
// // 		);
// // 	});
// // }

// // #[test]
// // fn check_did_max_tx_counter_submit_delegation_revocation_operation() {
// // 	let auth_key = did_mock::get_ed25519_authentication_key(true);
// // 	let enc_key = did_mock::get_x25519_encryption_key(true);
// // 	let del_key = did_mock::get_sr25519_delegation_key(true);

// // 	let mut delegator_details = did_mock::generate_base_did_details(
// // 		did::PublicVerificationKey::from(auth_key.public()),
// // 		did::PublicEncryptionKey::from(enc_key),
// // 	);
// // 	delegator_details.delegation_key =
// // Some(did::PublicVerificationKey::from(del_key.public())); 	delegator_details.
// // set_tx_counter(u64::MAX);

// // 	let delegator_did = did_mock::ALICE_DID;
// // 	let ctype_hash = ctype_mock::get_ctype_hash(true);
// // 	let max_depth = 1u32;
// // 	let max_revocations = 2u32;
// // 	let root_id = get_delegation_root_id(true);
// // 	let delegation_1_id = get_delegation_id(true);
// // 	let delegation_1_owner_did = did_mock::BOB_DID;
// // 	let delegation_2_id = get_delegation_id(false);
// // 	let delegation_2_owner_did = did_mock::CHARLIE_DID;

// // 	let operation = delegation::DelegationRevocationOperation {
// // 		caller_did: delegator_did.clone(),
// // 		// Removing delegation_2, a child of delegation_1
// // 		delegation_id: delegation_2_id,
// // 		max_parent_checks: max_depth,
// // 		max_revocations: max_revocations,
// // 		tx_counter: delegator_details.get_tx_counter_value(),
// // 	};
// // 	let signature = del_key.sign(&operation.encode());

// // 	let builder = did_mock::ExtBuilder::default().with_dids(vec![
// // 		(delegator_did.clone(), delegator_details.clone()),
// // 		(delegation_1_owner_did.clone(), delegator_details.clone()),
// // 		(delegation_2_owner_did.clone(), delegator_details.clone()),
// // 	]);
// // 	let builder =
// // ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash,
// // delegator_did.clone())]); 	let builder = ExtBuilder::from(builder)
// // 		.with_root_delegations(vec![(
// // 			root_id,
// // 			delegation::DelegationRoot {
// // 				owner: delegation_1_owner_did.clone(),
// // 				ctype_hash: ctype_hash,
// // 				revoked: false,
// // 			},
// // 		)])
// // 		// Root -> Delegation 1 -> Delegation 2
// // 		.with_delegations(vec![
// // 			(
// // 				delegation_1_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_1_owner_did.clone(),
// // 					parent: None,
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 			(
// // 				delegation_2_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_2_owner_did.clone(),
// // 					parent: Some(delegation_1_id),
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 		])
// // 		.with_children(vec![
// // 			(root_id, vec![delegation_1_id]),
// // 			(delegation_1_id, vec![delegation_2_id]),
// // 		]);

// // 	let mut ext = builder.build();

// // 	ext.execute_with(|| {
// // 		assert_noop!(
// // 			Delegation::submit_delegation_revocation_operation(
// // 				Origin::signed(DEFAULT_ACCOUNT),
// // 				operation.clone(),
// // 				did::DidSignature::from(signature)
// // 			),
// // 			did::Error::<Test>::MaxTxCounterValue
// // 		);
// // 	});
// // }

// // #[test]
// // fn check_delegator_too_small_tx_counter_submit_delegation_revocation_operation() {
// // 	let auth_key = did_mock::get_ed25519_authentication_key(true);
// // 	let enc_key = did_mock::get_x25519_encryption_key(true);
// // 	let del_key = did_mock::get_sr25519_delegation_key(true);

// // 	let mut delegator_details = did_mock::generate_base_did_details(
// // 		did::PublicVerificationKey::from(auth_key.public()),
// // 		did::PublicEncryptionKey::from(enc_key),
// // 	);
// // 	delegator_details.delegation_key =
// // Some(did::PublicVerificationKey::from(del_key.public())); 	delegator_details.
// // set_tx_counter(1u64);

// // 	let delegator_did = did_mock::ALICE_DID;
// // 	let ctype_hash = ctype_mock::get_ctype_hash(true);
// // 	let max_depth = 1u32;
// // 	let max_revocations = 2u32;
// // 	let root_id = get_delegation_root_id(true);
// // 	let delegation_1_id = get_delegation_id(true);
// // 	let delegation_1_owner_did = did_mock::BOB_DID;
// // 	let delegation_2_id = get_delegation_id(false);
// // 	let delegation_2_owner_did = did_mock::CHARLIE_DID;

// // 	let operation = delegation::DelegationRevocationOperation {
// // 		caller_did: delegator_did.clone(),
// // 		// Removing delegation_2, a child of delegation_1
// // 		delegation_id: delegation_2_id,
// // 		max_parent_checks: max_depth,
// // 		max_revocations: max_revocations,
// // 		tx_counter: delegator_details.get_tx_counter_value() - 1u64,
// // 	};
// // 	let signature = del_key.sign(&operation.encode());

// // 	let builder = did_mock::ExtBuilder::default().with_dids(vec![
// // 		(delegator_did.clone(), delegator_details.clone()),
// // 		(delegation_1_owner_did.clone(), delegator_details.clone()),
// // 		(delegation_2_owner_did.clone(), delegator_details.clone()),
// // 	]);
// // 	let builder =
// // ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash,
// // delegator_did.clone())]); 	let builder = ExtBuilder::from(builder)
// // 		.with_root_delegations(vec![(
// // 			root_id,
// // 			delegation::DelegationRoot {
// // 				owner: delegation_1_owner_did.clone(),
// // 				ctype_hash: ctype_hash,
// // 				revoked: false,
// // 			},
// // 		)])
// // 		// Root -> Delegation 1 -> Delegation 2
// // 		.with_delegations(vec![
// // 			(
// // 				delegation_1_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_1_owner_did.clone(),
// // 					parent: None,
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 			(
// // 				delegation_2_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_2_owner_did.clone(),
// // 					parent: Some(delegation_1_id),
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 		])
// // 		.with_children(vec![
// // 			(root_id, vec![delegation_1_id]),
// // 			(delegation_1_id, vec![delegation_2_id]),
// // 		]);

// // 	let mut ext = builder.build();

// // 	ext.execute_with(|| {
// // 		assert_noop!(
// // 			Delegation::submit_delegation_revocation_operation(
// // 				Origin::signed(DEFAULT_ACCOUNT),
// // 				operation.clone(),
// // 				did::DidSignature::from(signature)
// // 			),
// // 			did::Error::<Test>::InvalidNonce
// // 		);
// // 	});
// // }

// // #[test]
// // fn check_delegator_equal_tx_counter_submit_delegation_revocation_operation()
// // { 	let auth_key = did_mock::get_ed25519_authentication_key(true);
// // 	let enc_key = did_mock::get_x25519_encryption_key(true);
// // 	let del_key = did_mock::get_sr25519_delegation_key(true);

// // 	let mut delegator_details = did_mock::generate_base_did_details(
// // 		did::PublicVerificationKey::from(auth_key.public()),
// // 		did::PublicEncryptionKey::from(enc_key),
// // 	);
// // 	delegator_details.delegation_key =
// // Some(did::PublicVerificationKey::from(del_key.public()));

// // 	let delegator_did = did_mock::ALICE_DID;
// // 	let ctype_hash = ctype_mock::get_ctype_hash(true);
// // 	let max_depth = 1u32;
// // 	let max_revocations = 2u32;
// // 	let root_id = get_delegation_root_id(true);
// // 	let delegation_1_id = get_delegation_id(true);
// // 	let delegation_1_owner_did = did_mock::BOB_DID;
// // 	let delegation_2_id = get_delegation_id(false);
// // 	let delegation_2_owner_did = did_mock::CHARLIE_DID;

// // 	let operation = delegation::DelegationRevocationOperation {
// // 		caller_did: delegator_did.clone(),
// // 		// Removing delegation_2, a child of delegation_1
// // 		delegation_id: delegation_2_id,
// // 		max_parent_checks: max_depth,
// // 		max_revocations: max_revocations,
// // 		tx_counter: delegator_details.get_tx_counter_value(),
// // 	};
// // 	let signature = del_key.sign(&operation.encode());

// // 	let builder = did_mock::ExtBuilder::default().with_dids(vec![
// // 		(delegator_did.clone(), delegator_details.clone()),
// // 		(delegation_1_owner_did.clone(), delegator_details.clone()),
// // 		(delegation_2_owner_did.clone(), delegator_details.clone()),
// // 	]);
// // 	let builder =
// // ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash,
// // delegator_did.clone())]); 	let builder = ExtBuilder::from(builder)
// // 		.with_root_delegations(vec![(
// // 			root_id,
// // 			delegation::DelegationRoot {
// // 				owner: delegation_1_owner_did.clone(),
// // 				ctype_hash: ctype_hash,
// // 				revoked: false,
// // 			},
// // 		)])
// // 		// Root -> Delegation 1 -> Delegation 2
// // 		.with_delegations(vec![
// // 			(
// // 				delegation_1_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_1_owner_did.clone(),
// // 					parent: None,
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 			(
// // 				delegation_2_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_2_owner_did.clone(),
// // 					parent: Some(delegation_1_id),
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 		])
// // 		.with_children(vec![
// // 			(root_id, vec![delegation_1_id]),
// // 			(delegation_1_id, vec![delegation_2_id]),
// // 		]);

// // 	let mut ext = builder.build();

// // 	ext.execute_with(|| {
// // 		assert_noop!(
// // 			Delegation::submit_delegation_revocation_operation(
// // 				Origin::signed(DEFAULT_ACCOUNT),
// // 				operation.clone(),
// // 				did::DidSignature::from(signature)
// // 			),
// // 			did::Error::<Test>::InvalidNonce
// // 		);
// // 	});
// // }

// // #[test]
// // fn check_delegator_too_large_tx_counter_submit_delegation_revocation_operation() {
// // 	let auth_key = did_mock::get_ed25519_authentication_key(true);
// // 	let enc_key = did_mock::get_x25519_encryption_key(true);
// // 	let del_key = did_mock::get_sr25519_delegation_key(true);

// // 	let mut delegator_details = did_mock::generate_base_did_details(
// // 		did::PublicVerificationKey::from(auth_key.public()),
// // 		did::PublicEncryptionKey::from(enc_key),
// // 	);
// // 	delegator_details.delegation_key =
// // Some(did::PublicVerificationKey::from(del_key.public()));

// // 	let delegator_did = did_mock::ALICE_DID;
// // 	let ctype_hash = ctype_mock::get_ctype_hash(true);
// // 	let max_depth = 1u32;
// // 	let max_revocations = 2u32;
// // 	let root_id = get_delegation_root_id(true);
// // 	let delegation_1_id = get_delegation_id(true);
// // 	let delegation_1_owner_did = did_mock::BOB_DID;
// // 	let delegation_2_id = get_delegation_id(false);
// // 	let delegation_2_owner_did = did_mock::CHARLIE_DID;

// // 	let operation = delegation::DelegationRevocationOperation {
// // 		caller_did: delegator_did.clone(),
// // 		// Removing delegation_2, a child of delegation_1
// // 		delegation_id: delegation_2_id,
// // 		max_parent_checks: max_depth,
// // 		max_revocations: max_revocations,
// // 		tx_counter: delegator_details.get_tx_counter_value() + 2u64,
// // 	};
// // 	let signature = del_key.sign(&operation.encode());

// // 	let builder = did_mock::ExtBuilder::default().with_dids(vec![
// // 		(delegator_did.clone(), delegator_details.clone()),
// // 		(delegation_1_owner_did.clone(), delegator_details.clone()),
// // 		(delegation_2_owner_did.clone(), delegator_details.clone()),
// // 	]);
// // 	let builder =
// // ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash,
// // delegator_did.clone())]); 	let builder = ExtBuilder::from(builder)
// // 		.with_root_delegations(vec![(
// // 			root_id,
// // 			delegation::DelegationRoot {
// // 				owner: delegation_1_owner_did.clone(),
// // 				ctype_hash: ctype_hash,
// // 				revoked: false,
// // 			},
// // 		)])
// // 		// Root -> Delegation 1 -> Delegation 2
// // 		.with_delegations(vec![
// // 			(
// // 				delegation_1_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_1_owner_did.clone(),
// // 					parent: None,
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 			(
// // 				delegation_2_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_2_owner_did.clone(),
// // 					parent: Some(delegation_1_id),
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 		])
// // 		.with_children(vec![
// // 			(root_id, vec![delegation_1_id]),
// // 			(delegation_1_id, vec![delegation_2_id]),
// // 		]);

// // 	let mut ext = builder.build();

// // 	ext.execute_with(|| {
// // 		assert_noop!(
// // 			Delegation::submit_delegation_revocation_operation(
// // 				Origin::signed(DEFAULT_ACCOUNT),
// // 				operation.clone(),
// // 				did::DidSignature::from(signature)
// // 			),
// // 			did::Error::<Test>::InvalidNonce
// // 		);
// // 	});
// // }

// // #[test]
// // fn check_delegator_delegation_key_not_present_submit_delegation_revocation_operation() {
// // 	let auth_key = did_mock::get_ed25519_authentication_key(true);
// // 	let enc_key = did_mock::get_x25519_encryption_key(true);
// // 	let del_key = did_mock::get_sr25519_delegation_key(true);

// // 	let delegator_details = did_mock::generate_base_did_details(
// // 		did::PublicVerificationKey::from(auth_key.public()),
// // 		did::PublicEncryptionKey::from(enc_key),
// // 	);
// // 	// No del_key added

// // 	let delegator_did = did_mock::ALICE_DID;
// // 	let ctype_hash = ctype_mock::get_ctype_hash(true);
// // 	let max_depth = 1u32;
// // 	let max_revocations = 2u32;
// // 	let root_id = get_delegation_root_id(true);
// // 	let delegation_1_id = get_delegation_id(true);
// // 	let delegation_1_owner_did = did_mock::BOB_DID;
// // 	let delegation_2_id = get_delegation_id(false);
// // 	let delegation_2_owner_did = did_mock::CHARLIE_DID;

// // 	let operation = delegation::DelegationRevocationOperation {
// // 		caller_did: delegator_did.clone(),
// // 		// Removing delegation_2, a child of delegation_1
// // 		delegation_id: delegation_2_id,
// // 		max_parent_checks: max_depth,
// // 		max_revocations: max_revocations,
// // 		tx_counter: delegator_details.get_tx_counter_value() + 1u64,
// // 	};
// // 	let signature = del_key.sign(&operation.encode());

// // 	let builder = did_mock::ExtBuilder::default().with_dids(vec![
// // 		(delegator_did.clone(), delegator_details.clone()),
// // 		(delegation_1_owner_did.clone(), delegator_details.clone()),
// // 		(delegation_2_owner_did.clone(), delegator_details.clone()),
// // 	]);
// // 	let builder =
// // ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash,
// // delegator_did.clone())]); 	let builder = ExtBuilder::from(builder)
// // 		.with_root_delegations(vec![(
// // 			root_id,
// // 			delegation::DelegationRoot {
// // 				owner: delegation_1_owner_did.clone(),
// // 				ctype_hash: ctype_hash,
// // 				revoked: false,
// // 			},
// // 		)])
// // 		// Root -> Delegation 1 -> Delegation 2
// // 		.with_delegations(vec![
// // 			(
// // 				delegation_1_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_1_owner_did.clone(),
// // 					parent: None,
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 			(
// // 				delegation_2_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_2_owner_did.clone(),
// // 					parent: Some(delegation_1_id),
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 		])
// // 		.with_children(vec![
// // 			(root_id, vec![delegation_1_id]),
// // 			(delegation_1_id, vec![delegation_2_id]),
// // 		]);

// // 	let mut ext = builder.build();

// // 	ext.execute_with(|| {
// // 		assert_noop!(
// // 			Delegation::submit_delegation_revocation_operation(
// // 				Origin::signed(DEFAULT_ACCOUNT),
// // 				operation.clone(),
// // 				did::DidSignature::from(signature)
// // 			),
// // 			did::Error::<Test>::VerificationKeysNotPresent
// // 		);
// // 	});
// // }

// // #[test]
// // fn check_invalid_signature_format_submit_delegation_revocation_operation() {
// // 	let auth_key = did_mock::get_ed25519_authentication_key(true);
// // 	let enc_key = did_mock::get_x25519_encryption_key(true);
// // 	let del_key = did_mock::get_sr25519_delegation_key(true);
// // 	let alternative_del_key = did_mock::get_ed25519_delegation_key(true);

// // 	let mut delegator_details = did_mock::generate_base_did_details(
// // 		did::PublicVerificationKey::from(auth_key.public()),
// // 		did::PublicEncryptionKey::from(enc_key),
// // 	);
// // 	delegator_details.delegation_key =
// // Some(did::PublicVerificationKey::from(del_key.public()));

// // 	let delegator_did = did_mock::ALICE_DID;
// // 	let ctype_hash = ctype_mock::get_ctype_hash(true);
// // 	let max_depth = 1u32;
// // 	let max_revocations = 2u32;
// // 	let root_id = get_delegation_root_id(true);
// // 	let delegation_1_id = get_delegation_id(true);
// // 	let delegation_1_owner_did = did_mock::BOB_DID;
// // 	let delegation_2_id = get_delegation_id(false);
// // 	let delegation_2_owner_did = did_mock::CHARLIE_DID;

// // 	let operation = delegation::DelegationRevocationOperation {
// // 		caller_did: delegator_did.clone(),
// // 		// Removing delegation_2, a child of delegation_1
// // 		delegation_id: delegation_2_id,
// // 		max_parent_checks: max_depth,
// // 		max_revocations: max_revocations,
// // 		tx_counter: delegator_details.get_tx_counter_value() + 1u64,
// // 	};
// // 	let signature = alternative_del_key.sign(&operation.encode());

// // 	let builder = did_mock::ExtBuilder::default().with_dids(vec![
// // 		(delegator_did.clone(), delegator_details.clone()),
// // 		(delegation_1_owner_did.clone(), delegator_details.clone()),
// // 		(delegation_2_owner_did.clone(), delegator_details.clone()),
// // 	]);
// // 	let builder =
// // ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash,
// // delegator_did.clone())]); 	let builder = ExtBuilder::from(builder)
// // 		.with_root_delegations(vec![(
// // 			root_id,
// // 			delegation::DelegationRoot {
// // 				owner: delegation_1_owner_did.clone(),
// // 				ctype_hash: ctype_hash,
// // 				revoked: false,
// // 			},
// // 		)])
// // 		// Root -> Delegation 1 -> Delegation 2
// // 		.with_delegations(vec![
// // 			(
// // 				delegation_1_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_1_owner_did.clone(),
// // 					parent: None,
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 			(
// // 				delegation_2_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_2_owner_did.clone(),
// // 					parent: Some(delegation_1_id),
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 		])
// // 		.with_children(vec![
// // 			(root_id, vec![delegation_1_id]),
// // 			(delegation_1_id, vec![delegation_2_id]),
// // 		]);

// // 	let mut ext = builder.build();

// // 	ext.execute_with(|| {
// // 		assert_noop!(
// // 			Delegation::submit_delegation_revocation_operation(
// // 				Origin::signed(DEFAULT_ACCOUNT),
// // 				operation.clone(),
// // 				did::DidSignature::from(signature)
// // 			),
// // 			did::Error::<Test>::InvalidSignatureFormat
// // 		);
// // 	});
// // }

// // #[test]
// // fn check_invalid_signature_submit_delegation_revocation_operation() {
// // 	let auth_key = did_mock::get_ed25519_authentication_key(true);
// // 	let enc_key = did_mock::get_x25519_encryption_key(true);
// // 	let del_key = did_mock::get_sr25519_delegation_key(true);
// // 	let alternative_del_key = did_mock::get_sr25519_delegation_key(false);

// // 	let mut delegator_details = did_mock::generate_base_did_details(
// // 		did::PublicVerificationKey::from(auth_key.public()),
// // 		did::PublicEncryptionKey::from(enc_key),
// // 	);
// // 	delegator_details.delegation_key =
// // Some(did::PublicVerificationKey::from(del_key.public()));

// // 	let delegator_did = did_mock::ALICE_DID;
// // 	let ctype_hash = ctype_mock::get_ctype_hash(true);
// // 	let max_depth = 1u32;
// // 	let max_revocations = 2u32;
// // 	let root_id = get_delegation_root_id(true);
// // 	let delegation_1_id = get_delegation_id(true);
// // 	let delegation_1_owner_did = did_mock::BOB_DID;
// // 	let delegation_2_id = get_delegation_id(false);
// // 	let delegation_2_owner_did = did_mock::CHARLIE_DID;

// // 	let operation = delegation::DelegationRevocationOperation {
// // 		caller_did: delegator_did.clone(),
// // 		// Removing delegation_2, a child of delegation_1
// // 		delegation_id: delegation_2_id,
// // 		max_parent_checks: max_depth,
// // 		max_revocations: max_revocations,
// // 		tx_counter: delegator_details.get_tx_counter_value() + 1u64,
// // 	};
// // 	let signature = alternative_del_key.sign(&operation.encode());

// // 	let builder = did_mock::ExtBuilder::default().with_dids(vec![
// // 		(delegator_did.clone(), delegator_details.clone()),
// // 		(delegation_1_owner_did.clone(), delegator_details.clone()),
// // 		(delegation_2_owner_did.clone(), delegator_details.clone()),
// // 	]);
// // 	let builder =
// // ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash,
// // delegator_did.clone())]); 	let builder = ExtBuilder::from(builder)
// // 		.with_root_delegations(vec![(
// // 			root_id,
// // 			delegation::DelegationRoot {
// // 				owner: delegation_1_owner_did.clone(),
// // 				ctype_hash: ctype_hash,
// // 				revoked: false,
// // 			},
// // 		)])
// // 		// Root -> Delegation 1 -> Delegation 2
// // 		.with_delegations(vec![
// // 			(
// // 				delegation_1_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_1_owner_did.clone(),
// // 					parent: None,
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 			(
// // 				delegation_2_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_2_owner_did.clone(),
// // 					parent: Some(delegation_1_id),
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 		])
// // 		.with_children(vec![
// // 			(root_id, vec![delegation_1_id]),
// // 			(delegation_1_id, vec![delegation_2_id]),
// // 		]);

// // 	let mut ext = builder.build();

// // 	ext.execute_with(|| {
// // 		assert_noop!(
// // 			Delegation::submit_delegation_revocation_operation(
// // 				Origin::signed(DEFAULT_ACCOUNT),
// // 				operation.clone(),
// // 				did::DidSignature::from(signature)
// // 			),
// // 			did::Error::<Test>::InvalidSignature
// // 		);
// // 	});
// // }

// // #[test]
// // fn check_delegation_not_found_submit_delegation_revocation_operation_successful() {
// // 	let auth_key = did_mock::get_ed25519_authentication_key(true);
// // 	let enc_key = did_mock::get_x25519_encryption_key(true);
// // 	let del_key = did_mock::get_sr25519_delegation_key(true);

// // 	let mut delegator_details = did_mock::generate_base_did_details(
// // 		did::PublicVerificationKey::from(auth_key.public()),
// // 		did::PublicEncryptionKey::from(enc_key),
// // 	);
// // 	delegator_details.delegation_key =
// // Some(did::PublicVerificationKey::from(del_key.public()));

// // 	let delegator_did = did_mock::ALICE_DID;
// // 	let ctype_hash = ctype_mock::get_ctype_hash(true);
// // 	let max_depth = 1u32;
// // 	let max_revocations = 2u32;
// // 	let root_id = get_delegation_root_id(true);
// // 	let delegation_id = get_delegation_id(true);
// // 	let delegation_owner_did = did_mock::BOB_DID;
// // 	let alternative_delegation_id = get_delegation_id(false);
// // 	let alternative_delegation_owner_did = did_mock::CHARLIE_DID;

// // 	let operation = delegation::DelegationRevocationOperation {
// // 		caller_did: delegation_owner_did.clone(),
// // 		delegation_id: delegation_id,
// // 		max_parent_checks: max_depth,
// // 		max_revocations: max_revocations,
// // 		tx_counter: delegator_details.get_tx_counter_value() + 1u64,
// // 	};
// // 	let signature = del_key.sign(&operation.encode());

// // 	let builder = did_mock::ExtBuilder::default().with_dids(vec![
// // 		(delegator_did.clone(), delegator_details.clone()),
// // 		(delegation_owner_did.clone(), delegator_details.clone()),
// // 	]);
// // 	let builder =
// // ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash,
// // delegator_did.clone())]); 	let builder = ExtBuilder::from(builder)
// // 		.with_root_delegations(vec![(
// // 			root_id,
// // 			delegation::DelegationRoot {
// // 				owner: delegator_did.clone(),
// // 				ctype_hash: ctype_hash,
// // 				revoked: false,
// // 			},
// // 		)])
// // 		// Root -> Delegation 1
// // 		.with_delegations(vec![(
// // 			alternative_delegation_id,
// // 			delegation::DelegationNode {
// // 				owner: alternative_delegation_owner_did.clone(),
// // 				parent: None,
// // 				root_id: root_id,
// // 				permissions: delegation::Permissions::DELEGATE,
// // 				revoked: false,
// // 			},
// // 		)])
// // 		.with_children(vec![(root_id, vec![delegation_id])]);

// // 	let mut ext = builder.build();

// // 	ext.execute_with(|| {
// // 		assert_err!(
// // 			Delegation::submit_delegation_revocation_operation(
// // 				Origin::signed(DEFAULT_ACCOUNT),
// // 				operation.clone(),
// // 				did::DidSignature::from(signature)
// // 			),
// // 			delegation::Error::<Test>::DelegationNotFound
// // 		);
// // 	});

// // 	// Verify that the DID tx counter has increased
// // 	let new_delegator_details =
// // 		ext.execute_with(|| Did::get_did(&operation.caller_did).expect("DID submitter
// // should be present on chain.")); 	assert_eq!(
// // 		new_delegator_details.get_tx_counter_value(),
// // 		delegator_details.get_tx_counter_value() + 1u64
// // 	);
// // }

// // #[test]
// // fn check_not_delegating_submit_delegation_revocation_operation_successful() {
// // 	let auth_key = did_mock::get_ed25519_authentication_key(true);
// // 	let enc_key = did_mock::get_x25519_encryption_key(true);
// // 	let del_key = did_mock::get_sr25519_delegation_key(true);

// // 	let mut delegator_details = did_mock::generate_base_did_details(
// // 		did::PublicVerificationKey::from(auth_key.public()),
// // 		did::PublicEncryptionKey::from(enc_key),
// // 	);
// // 	delegator_details.delegation_key =
// // Some(did::PublicVerificationKey::from(del_key.public()));

// // 	let delegator_did = did_mock::ALICE_DID;
// // 	let ctype_hash = ctype_mock::get_ctype_hash(true);
// // 	let max_depth = 1u32;
// // 	let max_revocations = 2u32;
// // 	let root_id = get_delegation_root_id(true);
// // 	let delegation_id = get_delegation_id(true);
// // 	let delegation_owner_did = did_mock::BOB_DID;
// // 	let alternative_delegation_owner_did = did_mock::CHARLIE_DID;

// // 	let operation = delegation::DelegationRevocationOperation {
// // 		caller_did: delegation_owner_did.clone(),
// // 		delegation_id: delegation_id,
// // 		max_parent_checks: max_depth,
// // 		max_revocations: max_revocations,
// // 		tx_counter: delegator_details.get_tx_counter_value() + 1u64,
// // 	};
// // 	let signature = del_key.sign(&operation.encode());

// // 	let builder = did_mock::ExtBuilder::default().with_dids(vec![
// // 		(delegator_did.clone(), delegator_details.clone()),
// // 		(delegation_owner_did.clone(), delegator_details.clone()),
// // 		(alternative_delegation_owner_did.clone(), delegator_details.clone()),
// // 	]);
// // 	let builder =
// // ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash,
// // delegator_did.clone())]); 	let builder = ExtBuilder::from(builder)
// // 		.with_root_delegations(vec![(
// // 			root_id,
// // 			delegation::DelegationRoot {
// // 				owner: delegator_did.clone(),
// // 				ctype_hash: ctype_hash,
// // 				revoked: false,
// // 			},
// // 		)])
// // 		// Root -> Delegation 1
// // 		.with_delegations(vec![(
// // 			delegation_id,
// // 			delegation::DelegationNode {
// // 				owner: alternative_delegation_owner_did.clone(),
// // 				parent: None,
// // 				root_id: root_id,
// // 				permissions: delegation::Permissions::DELEGATE,
// // 				revoked: false,
// // 			},
// // 		)])
// // 		.with_children(vec![(root_id, vec![delegation_id])]);

// // 	let mut ext = builder.build();

// // 	ext.execute_with(|| {
// // 		assert_err!(
// // 			Delegation::submit_delegation_revocation_operation(
// // 				Origin::signed(DEFAULT_ACCOUNT),
// // 				operation.clone(),
// // 				did::DidSignature::from(signature)
// // 			),
// // 			delegation::Error::<Test>::UnauthorizedRevocation
// // 		);
// // 	});

// // 	// Verify that the DID tx counter has increased
// // 	let new_delegator_details =
// // 		ext.execute_with(|| Did::get_did(&operation.caller_did).expect("DID submitter
// // should be present on chain.")); 	assert_eq!(
// // 		new_delegator_details.get_tx_counter_value(),
// // 		delegator_details.get_tx_counter_value() + 1u64
// // 	);
// // }

// // #[test]
// // fn check_parent_too_far_submit_delegation_revocation_operation_successful() {
// // 	let auth_key = did_mock::get_ed25519_authentication_key(true);
// // 	let enc_key = did_mock::get_x25519_encryption_key(true);
// // 	let del_key = did_mock::get_sr25519_delegation_key(true);

// // 	let mut delegator_details = did_mock::generate_base_did_details(
// // 		did::PublicVerificationKey::from(auth_key.public()),
// // 		did::PublicEncryptionKey::from(enc_key),
// // 	);
// // 	delegator_details.delegation_key =
// // Some(did::PublicVerificationKey::from(del_key.public()));

// // 	let delegator_did = did_mock::ALICE_DID;
// // 	let ctype_hash = ctype_mock::get_ctype_hash(true);
// // 	let max_depth = 0u32;
// // 	let max_revocations = 2u32;
// // 	let root_id = get_delegation_root_id(true);
// // 	let delegation_1_id = get_delegation_id(true);
// // 	let delegation_1_owner_did = did_mock::BOB_DID;
// // 	let delegation_2_id = get_delegation_id(false);
// // 	let delegation_2_owner_did = did_mock::CHARLIE_DID;

// // 	let operation = delegation::DelegationRevocationOperation {
// // 		caller_did: delegator_did.clone(),
// // 		// Root delegator tries to delete delegation 2, where root -> delegation 1 ->
// // delegation 2 		delegation_id: delegation_2_id,
// // 		max_parent_checks: max_depth,
// // 		max_revocations: max_revocations,
// // 		tx_counter: delegator_details.get_tx_counter_value() + 1u64,
// // 	};
// // 	let signature = del_key.sign(&operation.encode());

// // 	let builder = did_mock::ExtBuilder::default().with_dids(vec![
// // 		(delegator_did.clone(), delegator_details.clone()),
// // 		(delegation_1_owner_did.clone(), delegator_details.clone()),
// // 		(delegation_2_owner_did.clone(), delegator_details.clone()),
// // 	]);
// // 	let builder =
// // ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash,
// // delegator_did.clone())]); 	let builder = ExtBuilder::from(builder)
// // 		.with_root_delegations(vec![(
// // 			root_id,
// // 			delegation::DelegationRoot {
// // 				owner: delegator_did.clone(),
// // 				ctype_hash: ctype_hash,
// // 				revoked: false,
// // 			},
// // 		)])
// // 		// Root -> Delegation 1 -> Delegation 2
// // 		.with_delegations(vec![
// // 			(
// // 				delegation_1_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_1_owner_did.clone(),
// // 					parent: None,
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 			(
// // 				delegation_2_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_2_owner_did.clone(),
// // 					parent: Some(delegation_1_id),
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 		])
// // 		.with_children(vec![
// // 			(root_id, vec![delegation_1_id]),
// // 			(delegation_1_id, vec![delegation_2_id]),
// // 		]);

// // 	let mut ext = builder.build();

// // 	ext.execute_with(|| {
// // 		assert_err!(
// // 			Delegation::submit_delegation_revocation_operation(
// // 				Origin::signed(DEFAULT_ACCOUNT),
// // 				operation.clone(),
// // 				did::DidSignature::from(signature)
// // 			),
// // 			delegation::Error::<Test>::MaxSearchDepthReached
// // 		);
// // 	});

// // 	// Verify that the DID tx counter has increased
// // 	let new_delegator_details =
// // 		ext.execute_with(|| Did::get_did(&operation.caller_did).expect("DID submitter
// // should be present on chain.")); 	assert_eq!(
// // 		new_delegator_details.get_tx_counter_value(),
// // 		delegator_details.get_tx_counter_value() + 1u64
// // 	);
// // }

// // #[test]
// // fn check_too_many_revocations_submit_delegation_revocation_operation_successful() {
// // 	let auth_key = did_mock::get_ed25519_authentication_key(true);
// // 	let enc_key = did_mock::get_x25519_encryption_key(true);
// // 	let del_key = did_mock::get_sr25519_delegation_key(true);

// // 	let mut delegator_details = did_mock::generate_base_did_details(
// // 		did::PublicVerificationKey::from(auth_key.public()),
// // 		did::PublicEncryptionKey::from(enc_key),
// // 	);
// // 	delegator_details.delegation_key =
// // Some(did::PublicVerificationKey::from(del_key.public()));

// // 	let delegator_did = did_mock::ALICE_DID;
// // 	let ctype_hash = ctype_mock::get_ctype_hash(true);
// // 	let max_depth = 2u32;
// // 	let max_revocations = 0u32;
// // 	let root_id = get_delegation_root_id(true);
// // 	let delegation_1_id = get_delegation_id(true);
// // 	let delegation_1_owner_did = did_mock::BOB_DID;
// // 	let delegation_2_id = get_delegation_id(false);
// // 	let delegation_2_owner_did = did_mock::CHARLIE_DID;

// // 	let operation = delegation::DelegationRevocationOperation {
// // 		caller_did: delegation_1_owner_did.clone(),
// // 		// Delegator 1 trying to delete delegation 2
// // 		delegation_id: delegation_2_id,
// // 		max_parent_checks: max_depth,
// // 		max_revocations: max_revocations,
// // 		tx_counter: delegator_details.get_tx_counter_value() + 1u64,
// // 	};
// // 	let signature = del_key.sign(&operation.encode());

// // 	let builder = did_mock::ExtBuilder::default().with_dids(vec![
// // 		(delegator_did.clone(), delegator_details.clone()),
// // 		(delegation_1_owner_did.clone(), delegator_details.clone()),
// // 		(delegation_2_owner_did.clone(), delegator_details.clone()),
// // 	]);
// // 	let builder =
// // ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash,
// // delegator_did.clone())]); 	let builder = ExtBuilder::from(builder)
// // 		.with_root_delegations(vec![(
// // 			root_id,
// // 			delegation::DelegationRoot {
// // 				owner: delegator_did.clone(),
// // 				ctype_hash: ctype_hash,
// // 				revoked: false,
// // 			},
// // 		)])
// // 		// Root -> Delegation 1 -> Delegation 2
// // 		.with_delegations(vec![
// // 			(
// // 				delegation_1_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_1_owner_did.clone(),
// // 					parent: None,
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 			(
// // 				delegation_2_id,
// // 				delegation::DelegationNode {
// // 					owner: delegation_2_owner_did.clone(),
// // 					parent: Some(delegation_1_id),
// // 					root_id: root_id,
// // 					permissions: delegation::Permissions::DELEGATE,
// // 					revoked: false,
// // 				},
// // 			),
// // 		])
// // 		.with_children(vec![
// // 			(root_id, vec![delegation_1_id]),
// // 			(delegation_1_id, vec![delegation_2_id]),
// // 		]);

// // 	let mut ext = builder.build();

// // 	ext.execute_with(|| {
// // 		assert_err!(
// // 			Delegation::submit_delegation_revocation_operation(
// // 				Origin::signed(DEFAULT_ACCOUNT),
// // 				operation.clone(),
// // 				did::DidSignature::from(signature)
// // 			),
// // 			delegation::Error::<Test>::ExceededRevocationBounds
// // 		);
// // 	});

// // 	// Verify that the DID tx counter has increased
// // 	let new_delegator_details =
// // 		ext.execute_with(|| Did::get_did(&operation.caller_did).expect("DID submitter
// // should be present on chain.")); 	assert_eq!(
// // 		new_delegator_details.get_tx_counter_value(),
// // 		delegator_details.get_tx_counter_value() + 1u64
// // 	);
// // }
