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

use crate::{self as attestation, mock::*};
use ctype::mock as ctype_mock;
use delegation::mock as delegation_mock;
use did::mock as did_mock;
use frame_support::{assert_err, assert_noop, assert_ok};
use sp_core::Pair;

use codec::Encode;

// submit_attestation_creation_operation

#[test]
fn check_no_delegation_submit_attestation_creation_operation_successful() {
	let did_auth_key = did_mock::get_ed25519_authentication_key(true);
	let did_att_key = did_mock::get_sr25519_attestation_key(true);
	let mut mock_did_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(did_auth_key.public()));
	mock_did_details.update_attestation_key(did::DidVerificationKey::from(did_att_key.public()), 0u64);

	let caller_did = did_mock::ALICE_DID;
	let claim_hash = get_claim_hash(true);
	let attestation = generate_base_attestation(caller_did.clone());

	let operation = generate_base_attestation_creation_operation(claim_hash, attestation);
	let signature = did_att_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(caller_did.clone(), mock_did_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(operation.ctype_hash, caller_did.clone())]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_ok!(Attestation::submit_attestation_creation_operation(
			Origin::signed(DEFAULT_ACCOUNT),
			operation.clone(),
			did::DidSignature::from(signature)
		));
	});

	let stored_attestation =
		ext.execute_with(|| Attestation::attestations(claim_hash).expect("Attestation should be present on chain."));

	assert_eq!(stored_attestation.ctype_hash, operation.ctype_hash);
	assert_eq!(stored_attestation.attester, operation.attester_did);
	assert_eq!(stored_attestation.delegation_id, operation.delegation_id);
	assert!(!stored_attestation.revoked);

	// Verify that the DID tx counter has increased
	let new_attester_details =
		ext.execute_with(|| Did::get_did(caller_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_attester_details.get_tx_counter_value(),
		mock_did_details.get_tx_counter_value() + 1u64
	);
}

#[test]
fn check_with_delegation_submit_attestation_creation_operation_successful() {
	let did_auth_key = did_mock::get_ed25519_authentication_key(true);
	let did_att_key = did_mock::get_sr25519_attestation_key(true);
	let mut mock_did_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(did_auth_key.public()));
	mock_did_details.update_attestation_key(did::DidVerificationKey::from(did_att_key.public()), 0u64);

	let caller_did = did_mock::ALICE_DID;
	let claim_hash = get_claim_hash(true);
	let (root_id, root_node) = (
		delegation_mock::get_delegation_root_id(true),
		delegation_mock::generate_base_delegation_root(caller_did.clone()),
	);
	let (delegation_id, mut delegation_node) = (
		delegation_mock::get_delegation_id(true),
		delegation_mock::generate_base_delegation_node(root_id, caller_did.clone()),
	);
	delegation_node.permissions = delegation::Permissions::ATTEST;
	let mut attestation = generate_base_attestation(caller_did.clone());
	attestation.delegation_id = Some(delegation_id);

	let operation = generate_base_attestation_creation_operation(claim_hash, attestation);
	let signature = did_att_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(caller_did.clone(), mock_did_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(operation.ctype_hash, caller_did.clone())]);
	let builder = delegation_mock::ExtBuilder::from(builder)
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![(delegation_id, delegation_node)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_ok!(Attestation::submit_attestation_creation_operation(
			Origin::signed(DEFAULT_ACCOUNT),
			operation.clone(),
			did::DidSignature::from(signature)
		));
	});

	let stored_attestation =
		ext.execute_with(|| Attestation::attestations(claim_hash).expect("Attestation should be present on chain."));

	assert_eq!(stored_attestation.ctype_hash, operation.ctype_hash);
	assert_eq!(stored_attestation.attester, operation.attester_did);
	assert_eq!(stored_attestation.delegation_id, operation.delegation_id);
	assert!(!stored_attestation.revoked);

	let delegated_attestations = ext.execute_with(|| {
		Attestation::delegated_attestations(&delegation_id).expect("Attested delegation should be present on chain.")
	});

	assert_eq!(delegated_attestations, vec![claim_hash]);

	// Verify that the DID tx counter has increased
	let new_attester_details =
		ext.execute_with(|| Did::get_did(caller_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_attester_details.get_tx_counter_value(),
		mock_did_details.get_tx_counter_value() + 1u64
	);
}

#[test]
fn check_did_not_present_submit_attestation_creation_operation() {
	let did_auth_key = did_mock::get_ed25519_authentication_key(true);
	let did_att_key = did_mock::get_sr25519_attestation_key(true);
	let mut mock_did_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(did_auth_key.public()));
	mock_did_details.update_attestation_key(did::DidVerificationKey::from(did_att_key.public()), 0u64);

	let caller_did = did_mock::ALICE_DID;
	let alternative_did = did_mock::CHARLIE_DID;
	let claim_hash = get_claim_hash(true);
	let attestation = generate_base_attestation(caller_did);

	let operation = generate_base_attestation_creation_operation(claim_hash, attestation);
	let signature = did_att_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(alternative_did.clone(), mock_did_details)]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(operation.ctype_hash, alternative_did)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Attestation::submit_attestation_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::DidNotPresent
		);
	});
}

#[test]
fn check_did_max_tx_counter_submit_attestation_creation_operation() {
	let did_auth_key = did_mock::get_ed25519_authentication_key(true);
	let did_att_key = did_mock::get_sr25519_attestation_key(true);
	let mut mock_did_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(did_auth_key.public()));
	mock_did_details.update_attestation_key(did::DidVerificationKey::from(did_att_key.public()), 0u64);
	mock_did_details.set_tx_counter(u64::MAX);

	let caller_did = did_mock::ALICE_DID;
	let claim_hash = get_claim_hash(true);
	let attestation = generate_base_attestation(caller_did.clone());

	let operation = generate_base_attestation_creation_operation(claim_hash, attestation);
	let signature = did_att_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(caller_did.clone(), mock_did_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(operation.ctype_hash, caller_did)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Attestation::submit_attestation_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::MaxTxCounterValue
		);
	});

	// Verify that the DID tx counter has not increased
	let new_mock_details =
		ext.execute_with(|| Did::get_did(&operation.attester_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_mock_details.get_tx_counter_value(),
		mock_did_details.get_tx_counter_value()
	);
}

#[test]
fn check_did_too_small_tx_counter_submit_attestation_creation_operation() {
	let did_auth_key = did_mock::get_ed25519_authentication_key(true);
	let did_att_key = did_mock::get_sr25519_attestation_key(true);
	let mut mock_did_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(did_auth_key.public()));
	mock_did_details.update_attestation_key(did::DidVerificationKey::from(did_att_key.public()), 0u64);
	mock_did_details.set_tx_counter(1u64);

	let caller_did = did_mock::ALICE_DID;
	let claim_hash = get_claim_hash(true);
	let attestation = generate_base_attestation(caller_did.clone());

	let mut operation = generate_base_attestation_creation_operation(claim_hash, attestation);
	operation.tx_counter = mock_did_details.get_tx_counter_value() - 1u64;
	let signature = did_att_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(caller_did.clone(), mock_did_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(operation.ctype_hash, caller_did)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Attestation::submit_attestation_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidNonce
		);
	});

	// Verify that the DID tx counter has not increased
	let new_mock_details =
		ext.execute_with(|| Did::get_did(&operation.attester_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_mock_details.get_tx_counter_value(),
		mock_did_details.get_tx_counter_value()
	);
}

#[test]
fn check_did_equal_tx_counter_submit_attestation_creation_operation() {
	let did_auth_key = did_mock::get_ed25519_authentication_key(true);
	let did_att_key = did_mock::get_sr25519_attestation_key(true);
	let mut mock_did_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(did_auth_key.public()));
	mock_did_details.update_attestation_key(did::DidVerificationKey::from(did_att_key.public()), 0u64);

	let caller_did = did_mock::ALICE_DID;
	let claim_hash = get_claim_hash(true);
	let attestation = generate_base_attestation(caller_did.clone());

	let mut operation = generate_base_attestation_creation_operation(claim_hash, attestation);
	operation.tx_counter = mock_did_details.get_tx_counter_value();
	let signature = did_att_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(caller_did.clone(), mock_did_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(operation.ctype_hash, caller_did)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Attestation::submit_attestation_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidNonce
		);
	});

	// Verify that the DID tx counter has not increased
	let new_mock_details =
		ext.execute_with(|| Did::get_did(&operation.attester_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_mock_details.get_tx_counter_value(),
		mock_did_details.get_tx_counter_value()
	);
}

#[test]
fn check_did_too_large_tx_counter_submit_attestation_creation_operation() {
	let did_auth_key = did_mock::get_ed25519_authentication_key(true);
	let did_att_key = did_mock::get_sr25519_attestation_key(true);
	let mut mock_did_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(did_auth_key.public()));
	mock_did_details.update_attestation_key(did::DidVerificationKey::from(did_att_key.public()), 0u64);

	let caller_did = did_mock::ALICE_DID;
	let claim_hash = get_claim_hash(true);
	let attestation = generate_base_attestation(caller_did.clone());

	let mut operation = generate_base_attestation_creation_operation(claim_hash, attestation);
	operation.tx_counter = mock_did_details.get_tx_counter_value() + 2u64;
	let signature = did_att_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(caller_did.clone(), mock_did_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(operation.ctype_hash, caller_did)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Attestation::submit_attestation_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidNonce
		);
	});

	// Verify that the DID tx counter has not increased
	let new_mock_details =
		ext.execute_with(|| Did::get_did(&operation.attester_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_mock_details.get_tx_counter_value(),
		mock_did_details.get_tx_counter_value()
	);
}

#[test]
fn check_did_attestation_key_not_present_submit_attestation_creation_operation() {
	let did_auth_key = did_mock::get_ed25519_authentication_key(true);
	let did_att_key = did_mock::get_sr25519_attestation_key(true);
	let mock_did_details = did_mock::generate_base_did_details(did::DidVerificationKey::from(did_auth_key.public()));
	// Attestation key not set

	let caller_did = did_mock::ALICE_DID;
	let claim_hash = get_claim_hash(true);
	let attestation = generate_base_attestation(caller_did.clone());

	let operation = generate_base_attestation_creation_operation(claim_hash, attestation);
	let signature = did_att_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(caller_did.clone(), mock_did_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(operation.ctype_hash, caller_did)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Attestation::submit_attestation_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::VerificationKeyNotPresent
		);
	});

	// Verify that the DID tx counter has not increased
	let new_mock_details =
		ext.execute_with(|| Did::get_did(&operation.attester_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_mock_details.get_tx_counter_value(),
		mock_did_details.get_tx_counter_value()
	);
}

#[test]
fn check_did_invalid_signature_format_submit_attestation_creation_operation() {
	let did_auth_key = did_mock::get_ed25519_authentication_key(true);
	let did_att_key = did_mock::get_sr25519_attestation_key(true);
	let alternative_att_key = did_mock::get_ed25519_attestation_key(true);
	let mut mock_did_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(did_auth_key.public()));
	mock_did_details.update_attestation_key(did::DidVerificationKey::from(did_att_key.public()), 0u64);

	let caller_did = did_mock::ALICE_DID;
	let claim_hash = get_claim_hash(true);
	let attestation = generate_base_attestation(caller_did.clone());

	let operation = generate_base_attestation_creation_operation(claim_hash, attestation);
	let signature = alternative_att_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(caller_did.clone(), mock_did_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(operation.ctype_hash, caller_did)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Attestation::submit_attestation_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidSignatureFormat
		);
	});

	// Verify that the DID tx counter has not increased
	let new_mock_details =
		ext.execute_with(|| Did::get_did(&operation.attester_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_mock_details.get_tx_counter_value(),
		mock_did_details.get_tx_counter_value()
	);
}

#[test]
fn check_did_invalid_signature_submit_attestation_creation_operation() {
	let did_auth_key = did_mock::get_ed25519_authentication_key(true);
	let did_att_key = did_mock::get_sr25519_attestation_key(true);
	let alternative_att_key = did_mock::get_sr25519_attestation_key(false);
	let mut mock_did_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(did_auth_key.public()));
	mock_did_details.update_attestation_key(did::DidVerificationKey::from(did_att_key.public()), 0u64);

	let caller_did = did_mock::ALICE_DID;
	let claim_hash = get_claim_hash(true);
	let attestation = generate_base_attestation(caller_did.clone());

	let operation = generate_base_attestation_creation_operation(claim_hash, attestation);
	let signature = alternative_att_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(caller_did.clone(), mock_did_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(operation.ctype_hash, caller_did)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Attestation::submit_attestation_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidSignature
		);
	});

	// Verify that the DID tx counter has not increased
	let new_mock_details =
		ext.execute_with(|| Did::get_did(&operation.attester_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_mock_details.get_tx_counter_value(),
		mock_did_details.get_tx_counter_value()
	);
}

#[test]
fn check_ctype_not_present_submit_attestation_creation_operation() {
	let did_auth_key = did_mock::get_ed25519_authentication_key(true);
	let did_att_key = did_mock::get_sr25519_attestation_key(true);
	let mut mock_did_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(did_auth_key.public()));
	mock_did_details.update_attestation_key(did::DidVerificationKey::from(did_att_key.public()), 0u64);

	let caller_did = did_mock::ALICE_DID;
	let alternative_ctype = ctype_mock::get_ctype_hash(false);
	let claim_hash = get_claim_hash(true);
	let attestation = generate_base_attestation(caller_did.clone());

	let operation = generate_base_attestation_creation_operation(claim_hash, attestation);
	let signature = did_att_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(caller_did.clone(), mock_did_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(alternative_ctype, caller_did)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_err!(
			Attestation::submit_attestation_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			ctype::Error::<Test>::CTypeNotFound
		);
	});

	// Verify that the DID tx counter has increased
	let new_mock_details =
		ext.execute_with(|| Did::get_did(&operation.attester_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_mock_details.get_tx_counter_value(),
		mock_did_details.get_tx_counter_value() + 1u64
	);
}

#[test]
fn check_duplicate_attestation_submit_attestation_creation_operation() {
	let did_auth_key = did_mock::get_ed25519_authentication_key(true);
	let did_att_key = did_mock::get_sr25519_attestation_key(true);
	let mut mock_did_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(did_auth_key.public()));
	mock_did_details.update_attestation_key(did::DidVerificationKey::from(did_att_key.public()), 0u64);

	let caller_did = did_mock::ALICE_DID;
	let claim_hash = get_claim_hash(true);
	let attestation = generate_base_attestation(caller_did.clone());

	let operation = generate_base_attestation_creation_operation(claim_hash, attestation.clone());
	let signature = did_att_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(caller_did.clone(), mock_did_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(operation.ctype_hash, caller_did)]);
	let builder = delegation_mock::ExtBuilder::from(builder);
	let builder = ExtBuilder::from(builder).with_attestations(vec![(claim_hash, attestation)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_err!(
			Attestation::submit_attestation_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			attestation::Error::<Test>::AlreadyAttested
		);
	});

	// Verify that the DID tx counter has increased
	let new_mock_details =
		ext.execute_with(|| Did::get_did(&operation.attester_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_mock_details.get_tx_counter_value(),
		mock_did_details.get_tx_counter_value() + 1u64
	);
}

#[test]
fn check_delegation_not_found_submit_attestation_creation_operation() {
	let did_auth_key = did_mock::get_ed25519_authentication_key(true);
	let did_att_key = did_mock::get_sr25519_attestation_key(true);
	let mut mock_did_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(did_auth_key.public()));
	mock_did_details.update_attestation_key(did::DidVerificationKey::from(did_att_key.public()), 0u64);

	let caller_did = did_mock::ALICE_DID;
	let claim_hash = get_claim_hash(true);
	let delegation_id = delegation_mock::get_delegation_id(true);
	let mut attestation = generate_base_attestation(caller_did.clone());
	attestation.delegation_id = Some(delegation_id);

	let operation = generate_base_attestation_creation_operation(claim_hash, attestation);
	let signature = did_att_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(caller_did.clone(), mock_did_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(operation.ctype_hash, caller_did)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_err!(
			Attestation::submit_attestation_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			delegation::Error::<Test>::DelegationNotFound
		);
	});

	// Verify that the DID tx counter has increased
	let new_mock_details =
		ext.execute_with(|| Did::get_did(&operation.attester_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_mock_details.get_tx_counter_value(),
		mock_did_details.get_tx_counter_value() + 1u64
	);
}

#[test]
fn check_delegation_revoked_submit_attestation_creation_operation() {
	let did_auth_key = did_mock::get_ed25519_authentication_key(true);
	let did_att_key = did_mock::get_sr25519_attestation_key(true);
	let mut mock_did_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(did_auth_key.public()));
	mock_did_details.update_attestation_key(did::DidVerificationKey::from(did_att_key.public()), 0u64);

	let caller_did = did_mock::ALICE_DID;
	let claim_hash = get_claim_hash(true);
	let (root_id, root_node) = (
		delegation_mock::get_delegation_root_id(true),
		delegation_mock::generate_base_delegation_root(caller_did.clone()),
	);
	let (delegation_id, mut delegation_node) = (
		delegation_mock::get_delegation_id(true),
		delegation_mock::generate_base_delegation_node(root_id, caller_did.clone()),
	);
	delegation_node.permissions = delegation::Permissions::ATTEST;
	delegation_node.revoked = true;
	let mut attestation = generate_base_attestation(caller_did.clone());
	attestation.delegation_id = Some(delegation_id);

	let operation = generate_base_attestation_creation_operation(claim_hash, attestation);
	let signature = did_att_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(caller_did.clone(), mock_did_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(operation.ctype_hash, caller_did)]);
	let builder = delegation_mock::ExtBuilder::from(builder)
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![(delegation_id, delegation_node)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_err!(
			Attestation::submit_attestation_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			attestation::Error::<Test>::DelegationRevoked
		);
	});

	// Verify that the DID tx counter has increased
	let new_mock_details =
		ext.execute_with(|| Did::get_did(&operation.attester_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_mock_details.get_tx_counter_value(),
		mock_did_details.get_tx_counter_value() + 1u64
	);
}

#[test]
fn check_not_delegation_owner_submit_attestation_creation_operation() {
	let did_auth_key = did_mock::get_ed25519_authentication_key(true);
	let did_att_key = did_mock::get_sr25519_attestation_key(true);
	let mut mock_did_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(did_auth_key.public()));
	mock_did_details.update_attestation_key(did::DidVerificationKey::from(did_att_key.public()), 0u64);

	let caller_did = did_mock::ALICE_DID;
	let alternative_did = did_mock::BOB_DID;
	let claim_hash = get_claim_hash(true);
	let (root_id, root_node) = (
		delegation_mock::get_delegation_root_id(true),
		delegation_mock::generate_base_delegation_root(alternative_did.clone()),
	);
	let (delegation_id, mut delegation_node) = (
		delegation_mock::get_delegation_id(true),
		delegation_mock::generate_base_delegation_node(root_id, alternative_did),
	);
	delegation_node.permissions = delegation::Permissions::ATTEST;
	let mut attestation = generate_base_attestation(caller_did.clone());
	attestation.delegation_id = Some(delegation_id);

	let operation = generate_base_attestation_creation_operation(claim_hash, attestation);
	let signature = did_att_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(caller_did.clone(), mock_did_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(operation.ctype_hash, caller_did)]);
	let builder = delegation_mock::ExtBuilder::from(builder)
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![(delegation_id, delegation_node)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_err!(
			Attestation::submit_attestation_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			attestation::Error::<Test>::NotDelegatedToAttester
		);
	});

	// Verify that the DID tx counter has increased
	let new_mock_details =
		ext.execute_with(|| Did::get_did(&operation.attester_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_mock_details.get_tx_counter_value(),
		mock_did_details.get_tx_counter_value() + 1u64
	);
}

#[test]
fn check_unauthorised_permissions_submit_attestation_creation_operation() {
	let did_auth_key = did_mock::get_ed25519_authentication_key(true);
	let did_att_key = did_mock::get_sr25519_attestation_key(true);
	let mut mock_did_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(did_auth_key.public()));
	mock_did_details.update_attestation_key(did::DidVerificationKey::from(did_att_key.public()), 0u64);

	let caller_did = did_mock::ALICE_DID;
	let claim_hash = get_claim_hash(true);
	let (root_id, root_node) = (
		delegation_mock::get_delegation_root_id(true),
		delegation_mock::generate_base_delegation_root(caller_did.clone()),
	);
	let (delegation_id, mut delegation_node) = (
		delegation_mock::get_delegation_id(true),
		delegation_mock::generate_base_delegation_node(root_id, caller_did.clone()),
	);
	delegation_node.permissions = delegation::Permissions::DELEGATE;
	let mut attestation = generate_base_attestation(caller_did.clone());
	attestation.delegation_id = Some(delegation_id);

	let operation = generate_base_attestation_creation_operation(claim_hash, attestation);
	let signature = did_att_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(caller_did.clone(), mock_did_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(operation.ctype_hash, caller_did)]);
	let builder = delegation_mock::ExtBuilder::from(builder)
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![(delegation_id, delegation_node)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_err!(
			Attestation::submit_attestation_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			attestation::Error::<Test>::DelegationUnauthorizedToAttest
		);
	});

	// Verify that the DID tx counter has increased
	let new_mock_details =
		ext.execute_with(|| Did::get_did(&operation.attester_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_mock_details.get_tx_counter_value(),
		mock_did_details.get_tx_counter_value() + 1u64
	);
}

#[test]
fn check_root_not_present_submit_attestation_creation_operation() {
	let did_auth_key = did_mock::get_ed25519_authentication_key(true);
	let did_att_key = did_mock::get_sr25519_attestation_key(true);
	let mut mock_did_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(did_auth_key.public()));
	mock_did_details.update_attestation_key(did::DidVerificationKey::from(did_att_key.public()), 0u64);

	let caller_did = did_mock::ALICE_DID;
	let claim_hash = get_claim_hash(true);
	let (root_id, root_node) = (
		delegation_mock::get_delegation_root_id(true),
		delegation_mock::generate_base_delegation_root(caller_did.clone()),
	);
	let alternative_root_id = delegation_mock::get_delegation_root_id(false);
	let (delegation_id, mut delegation_node) = (
		delegation_mock::get_delegation_id(true),
		delegation_mock::generate_base_delegation_node(root_id, caller_did.clone()),
	);
	delegation_node.permissions = delegation::Permissions::ATTEST;
	let mut attestation = generate_base_attestation(caller_did.clone());
	attestation.delegation_id = Some(delegation_id);

	let operation = generate_base_attestation_creation_operation(claim_hash, attestation);
	let signature = did_att_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(caller_did.clone(), mock_did_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(operation.ctype_hash, caller_did)]);
	let builder = delegation_mock::ExtBuilder::from(builder)
		.with_root_delegations(vec![(alternative_root_id, root_node)])
		.with_delegations(vec![(delegation_id, delegation_node)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_err!(
			Attestation::submit_attestation_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			delegation::Error::<Test>::RootNotFound
		);
	});

	// Verify that the DID tx counter has increased
	let new_mock_details =
		ext.execute_with(|| Did::get_did(&operation.attester_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_mock_details.get_tx_counter_value(),
		mock_did_details.get_tx_counter_value() + 1u64
	);
}

#[test]
fn check_root_ctype_mismatch_submit_attestation_creation_operation() {
	let did_auth_key = did_mock::get_ed25519_authentication_key(true);
	let did_att_key = did_mock::get_sr25519_attestation_key(true);
	let mut mock_did_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(did_auth_key.public()));
	mock_did_details.update_attestation_key(did::DidVerificationKey::from(did_att_key.public()), 0u64);

	let caller_did = did_mock::ALICE_DID;
	let claim_hash = get_claim_hash(true);
	let alternative_ctype_hash = ctype_mock::get_ctype_hash(false);
	let (root_id, mut root_node) = (
		delegation_mock::get_delegation_root_id(true),
		delegation_mock::generate_base_delegation_root(caller_did.clone()),
	);
	root_node.ctype_hash = alternative_ctype_hash;
	let (delegation_id, mut delegation_node) = (
		delegation_mock::get_delegation_id(true),
		delegation_mock::generate_base_delegation_node(root_id, caller_did.clone()),
	);
	delegation_node.permissions = delegation::Permissions::ATTEST;
	let mut attestation = generate_base_attestation(caller_did.clone());
	attestation.delegation_id = Some(delegation_id);

	let operation = generate_base_attestation_creation_operation(claim_hash, attestation);
	let signature = did_att_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(caller_did.clone(), mock_did_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(operation.ctype_hash, caller_did)]);
	let builder = delegation_mock::ExtBuilder::from(builder)
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![(delegation_id, delegation_node)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_err!(
			Attestation::submit_attestation_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			attestation::Error::<Test>::CTypeMismatch
		);
	});

	// Verify that the DID tx counter has increased
	let new_mock_details =
		ext.execute_with(|| Did::get_did(&operation.attester_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_mock_details.get_tx_counter_value(),
		mock_did_details.get_tx_counter_value() + 1u64
	);
}

// submit_attestation_revocation_operation

#[test]
fn check_direct_attestation_submit_attestation_revocation_operation_successful() {
	let did_auth_key = did_mock::get_ed25519_authentication_key(true);
	let did_att_key = did_mock::get_sr25519_attestation_key(true);
	let mut mock_did_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(did_auth_key.public()));
	mock_did_details.update_attestation_key(did::DidVerificationKey::from(did_att_key.public()), 0u64);

	let caller_did = did_mock::ALICE_DID;
	let claim_hash = get_claim_hash(true);
	let attestation = generate_base_attestation(caller_did.clone());

	let operation = generate_base_attestation_revocation_operation(claim_hash, attestation.clone());
	let signature = did_att_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(caller_did.clone(), mock_did_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(attestation.ctype_hash, caller_did)]);
	let builder = delegation_mock::ExtBuilder::from(builder);
	let builder = ExtBuilder::from(builder).with_attestations(vec![(claim_hash, attestation)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_ok!(Attestation::submit_attestation_revocation_operation(
			Origin::signed(DEFAULT_ACCOUNT),
			operation.clone(),
			did::DidSignature::from(signature)
		));
	});

	let stored_attestation =
		ext.execute_with(|| Attestation::attestations(claim_hash).expect("Attestation should be present on chain."));

	assert!(stored_attestation.revoked);

	// Verify that the DID tx counter has increased
	let new_mock_details =
		ext.execute_with(|| Did::get_did(&operation.revoker_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_mock_details.get_tx_counter_value(),
		mock_did_details.get_tx_counter_value() + 1u64
	);
}

#[test]
fn check_direct_delegation_submit_attestation_revocation_operation_successful() {
	let did_auth_key = did_mock::get_ed25519_authentication_key(true);
	let did_att_key = did_mock::get_sr25519_attestation_key(true);
	let mut mock_did_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(did_auth_key.public()));
	mock_did_details.update_attestation_key(did::DidVerificationKey::from(did_att_key.public()), 0u64);

	let caller_did = did_mock::ALICE_DID;
	let alternative_did = did_mock::BOB_DID;
	let claim_hash = get_claim_hash(true);
	let (root_id, root_node) = (
		delegation_mock::get_delegation_root_id(true),
		delegation_mock::generate_base_delegation_root(caller_did.clone()),
	);
	let (delegation_id, mut delegation_node) = (
		delegation_mock::get_delegation_id(true),
		delegation_mock::generate_base_delegation_node(root_id, caller_did.clone()),
	);
	delegation_node.permissions = delegation::Permissions::ATTEST;
	// Attestation owned by a different user, but delegation owned by the user
	// submitting the operation.
	let mut attestation = generate_base_attestation(alternative_did);
	attestation.delegation_id = Some(delegation_id);

	let mut operation = generate_base_attestation_revocation_operation(claim_hash, attestation.clone());
	operation.revoker_did = caller_did.clone();
	// Set to 0 as we only need to check the delegation node itself and no parent.
	operation.max_parent_checks = 0u32;
	let signature = did_att_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(caller_did.clone(), mock_did_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(attestation.ctype_hash, caller_did)]);
	let builder = delegation_mock::ExtBuilder::from(builder)
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![(delegation_id, delegation_node)]);
	let builder = ExtBuilder::from(builder).with_attestations(vec![(claim_hash, attestation)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_ok!(Attestation::submit_attestation_revocation_operation(
			Origin::signed(DEFAULT_ACCOUNT),
			operation.clone(),
			did::DidSignature::from(signature)
		));
	});

	let stored_attestation =
		ext.execute_with(|| Attestation::attestations(claim_hash).expect("Attestation should be present on chain."));

	assert!(stored_attestation.revoked);

	// Verify that the DID tx counter has increased
	let new_mock_details =
		ext.execute_with(|| Did::get_did(&operation.revoker_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_mock_details.get_tx_counter_value(),
		mock_did_details.get_tx_counter_value() + 1u64
	);
}

#[test]
fn check_parent_delegation_submit_attestation_revocation_operation_successful() {
	let did_auth_key = did_mock::get_ed25519_authentication_key(true);
	let did_att_key = did_mock::get_sr25519_attestation_key(true);
	let mut mock_did_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(did_auth_key.public()));
	mock_did_details.update_attestation_key(did::DidVerificationKey::from(did_att_key.public()), 0u64);

	let caller_did = did_mock::ALICE_DID;
	let alternative_did = did_mock::BOB_DID;
	let claim_hash = get_claim_hash(true);
	let (root_id, root_node) = (
		delegation_mock::get_delegation_root_id(true),
		delegation_mock::generate_base_delegation_root(caller_did.clone()),
	);
	let (parent_id, mut parent_node) = (
		delegation_mock::get_delegation_id(true),
		delegation_mock::generate_base_delegation_node(root_id, caller_did.clone()),
	);
	parent_node.permissions = delegation::Permissions::ATTEST;
	let (delegation_id, delegation_node) = (
		delegation_mock::get_delegation_id(false),
		delegation_mock::generate_base_delegation_node(root_id, alternative_did.clone()),
	);
	// Attestation owned by a different user
	let mut attestation = generate_base_attestation(alternative_did);
	attestation.delegation_id = Some(delegation_id);

	let mut operation = generate_base_attestation_revocation_operation(claim_hash, attestation.clone());
	operation.revoker_did = caller_did.clone();
	// Set to 1 as the delegation referenced in the attestation is the child of the
	// node we want to use
	operation.max_parent_checks = 1u32;
	let signature = did_att_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(caller_did.clone(), mock_did_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(attestation.ctype_hash, caller_did)]);
	let builder = delegation_mock::ExtBuilder::from(builder)
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)]);
	let builder = ExtBuilder::from(builder).with_attestations(vec![(claim_hash, attestation)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_ok!(Attestation::submit_attestation_revocation_operation(
			Origin::signed(DEFAULT_ACCOUNT),
			operation.clone(),
			did::DidSignature::from(signature)
		));
	});

	let stored_attestation =
		ext.execute_with(|| Attestation::attestations(claim_hash).expect("Attestation should be present on chain."));

	assert!(stored_attestation.revoked);

	// Verify that the DID tx counter has increased
	let new_mock_details =
		ext.execute_with(|| Did::get_did(&operation.revoker_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_mock_details.get_tx_counter_value(),
		mock_did_details.get_tx_counter_value() + 1u64
	);
}

#[test]
fn check_parent_delegation_no_attestation_permissions_submit_attestation_revocation_operation_successful() {
	let did_auth_key = did_mock::get_ed25519_authentication_key(true);
	let did_att_key = did_mock::get_sr25519_attestation_key(true);
	let mut mock_did_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(did_auth_key.public()));
	mock_did_details.update_attestation_key(did::DidVerificationKey::from(did_att_key.public()), 0u64);

	let caller_did = did_mock::ALICE_DID;
	let alternative_did = did_mock::BOB_DID;
	let claim_hash = get_claim_hash(true);
	let (root_id, root_node) = (
		delegation_mock::get_delegation_root_id(true),
		delegation_mock::generate_base_delegation_root(caller_did.clone()),
	);
	let (parent_id, mut parent_node) = (
		delegation_mock::get_delegation_id(true),
		delegation_mock::generate_base_delegation_node(root_id, caller_did.clone()),
	);
	// Parent node has only permissions to delegate, and we are testing whether it
	// could also revoke attestations issued by its children
	parent_node.permissions = delegation::Permissions::DELEGATE;
	let (delegation_id, delegation_node) = (
		delegation_mock::get_delegation_id(false),
		delegation_mock::generate_base_delegation_node(root_id, alternative_did.clone()),
	);
	// Attestation owned by a different user
	let mut attestation = generate_base_attestation(alternative_did);
	attestation.delegation_id = Some(delegation_id);

	let mut operation = generate_base_attestation_revocation_operation(claim_hash, attestation.clone());
	operation.revoker_did = caller_did.clone();
	// Set to 1 as the delegation referenced in the attestation is the child of the
	// node we want to use
	operation.max_parent_checks = 1u32;
	let signature = did_att_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(caller_did.clone(), mock_did_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(attestation.ctype_hash, caller_did)]);
	let builder = delegation_mock::ExtBuilder::from(builder)
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)]);
	let builder = ExtBuilder::from(builder).with_attestations(vec![(claim_hash, attestation)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_ok!(Attestation::submit_attestation_revocation_operation(
			Origin::signed(DEFAULT_ACCOUNT),
			operation.clone(),
			did::DidSignature::from(signature)
		));
	});

	let stored_attestation =
		ext.execute_with(|| Attestation::attestations(claim_hash).expect("Attestation should be present on chain."));

	assert!(stored_attestation.revoked);

	// Verify that the DID tx counter has increased
	let new_mock_details =
		ext.execute_with(|| Did::get_did(&operation.revoker_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_mock_details.get_tx_counter_value(),
		mock_did_details.get_tx_counter_value() + 1u64
	);
}

#[test]
fn check_parent_delegation_with_direct_delegation_revoked_submit_attestation_revocation_operation_successful() {
	let did_auth_key = did_mock::get_ed25519_authentication_key(true);
	let did_att_key = did_mock::get_sr25519_attestation_key(true);
	let mut mock_did_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(did_auth_key.public()));
	mock_did_details.update_attestation_key(did::DidVerificationKey::from(did_att_key.public()), 0u64);

	let caller_did = did_mock::ALICE_DID;
	let alternative_did = did_mock::BOB_DID;
	let claim_hash = get_claim_hash(true);
	let (root_id, root_node) = (
		delegation_mock::get_delegation_root_id(true),
		delegation_mock::generate_base_delegation_root(caller_did.clone()),
	);
	let (parent_id, mut parent_node) = (
		delegation_mock::get_delegation_id(true),
		delegation_mock::generate_base_delegation_node(root_id, caller_did.clone()),
	);
	parent_node.permissions = delegation::Permissions::ATTEST;
	let (delegation_id, mut delegation_node) = (
		delegation_mock::get_delegation_id(false),
		delegation_mock::generate_base_delegation_node(root_id, alternative_did.clone()),
	);
	// Set the direct delegation node as revoked, but the one that is needed (its
	// parent) is not
	delegation_node.revoked = true;
	// Attestation owned by a different user
	let mut attestation = generate_base_attestation(alternative_did);
	attestation.delegation_id = Some(delegation_id);

	let mut operation = generate_base_attestation_revocation_operation(claim_hash, attestation.clone());
	operation.revoker_did = caller_did.clone();
	// Set to 1 as the delegation referenced in the attestation is the child of the
	// node we want to use
	operation.max_parent_checks = 1u32;
	let signature = did_att_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(caller_did.clone(), mock_did_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(attestation.ctype_hash, caller_did)]);
	let builder = delegation_mock::ExtBuilder::from(builder)
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)]);
	let builder = ExtBuilder::from(builder).with_attestations(vec![(claim_hash, attestation)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_ok!(Attestation::submit_attestation_revocation_operation(
			Origin::signed(DEFAULT_ACCOUNT),
			operation.clone(),
			did::DidSignature::from(signature)
		));
	});

	let stored_attestation =
		ext.execute_with(|| Attestation::attestations(claim_hash).expect("Attestation should be present on chain."));

	assert!(stored_attestation.revoked);

	// Verify that the DID tx counter has increased
	let new_mock_details =
		ext.execute_with(|| Did::get_did(&operation.revoker_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_mock_details.get_tx_counter_value(),
		mock_did_details.get_tx_counter_value() + 1u64
	);
}

#[test]
fn check_did_not_present_submit_attestation_revocation_operation() {
	let did_auth_key = did_mock::get_ed25519_authentication_key(true);
	let did_att_key = did_mock::get_sr25519_attestation_key(true);
	let mut mock_did_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(did_auth_key.public()));
	mock_did_details.update_attestation_key(did::DidVerificationKey::from(did_att_key.public()), 0u64);

	let caller_did = did_mock::ALICE_DID;
	let alternative_did = did_mock::BOB_DID;
	let claim_hash = get_claim_hash(true);
	let attestation = generate_base_attestation(caller_did.clone());

	let operation = generate_base_attestation_revocation_operation(claim_hash, attestation.clone());
	let signature = did_att_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(alternative_did, mock_did_details)]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(attestation.ctype_hash, caller_did)]);
	let builder = delegation_mock::ExtBuilder::from(builder);
	let builder = ExtBuilder::from(builder).with_attestations(vec![(claim_hash, attestation)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Attestation::submit_attestation_revocation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::DidNotPresent
		);
	});
}

#[test]
fn check_did_max_tx_counter_submit_attestation_revocation_operation() {
	let did_auth_key = did_mock::get_ed25519_authentication_key(true);
	let did_att_key = did_mock::get_sr25519_attestation_key(true);
	let mut mock_did_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(did_auth_key.public()));
	mock_did_details.update_attestation_key(did::DidVerificationKey::from(did_att_key.public()), 0u64);
	mock_did_details.set_tx_counter(u64::MAX);

	let caller_did = did_mock::ALICE_DID;
	let claim_hash = get_claim_hash(true);
	let attestation = generate_base_attestation(caller_did.clone());

	let operation = generate_base_attestation_revocation_operation(claim_hash, attestation.clone());
	let signature = did_att_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(caller_did.clone(), mock_did_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(attestation.ctype_hash, caller_did)]);
	let builder = delegation_mock::ExtBuilder::from(builder);
	let builder = ExtBuilder::from(builder).with_attestations(vec![(claim_hash, attestation)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Attestation::submit_attestation_revocation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::MaxTxCounterValue
		);
	});

	// Verify that the DID tx counter has not increased
	let new_mock_details =
		ext.execute_with(|| Did::get_did(&operation.revoker_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_mock_details.get_tx_counter_value(),
		mock_did_details.get_tx_counter_value()
	);
}

#[test]
fn check_too_small_tx_counter_submit_attestation_revocation_operation() {
	let did_auth_key = did_mock::get_ed25519_authentication_key(true);
	let did_att_key = did_mock::get_sr25519_attestation_key(true);
	let mut mock_did_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(did_auth_key.public()));
	mock_did_details.update_attestation_key(did::DidVerificationKey::from(did_att_key.public()), 0u64);
	mock_did_details.set_tx_counter(1u64);

	let caller_did = did_mock::ALICE_DID;
	let claim_hash = get_claim_hash(true);
	let attestation = generate_base_attestation(caller_did.clone());

	let mut operation = generate_base_attestation_revocation_operation(claim_hash, attestation.clone());
	operation.tx_counter = mock_did_details.get_tx_counter_value() - 1u64;
	let signature = did_att_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(caller_did.clone(), mock_did_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(attestation.ctype_hash, caller_did)]);
	let builder = delegation_mock::ExtBuilder::from(builder);
	let builder = ExtBuilder::from(builder).with_attestations(vec![(claim_hash, attestation)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Attestation::submit_attestation_revocation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidNonce
		);
	});

	// Verify that the DID tx counter has not increased
	let new_mock_details =
		ext.execute_with(|| Did::get_did(&operation.revoker_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_mock_details.get_tx_counter_value(),
		mock_did_details.get_tx_counter_value()
	);
}

#[test]
fn check_equal_tx_counter_submit_attestation_revocation_operation() {
	let did_auth_key = did_mock::get_ed25519_authentication_key(true);
	let did_att_key = did_mock::get_sr25519_attestation_key(true);
	let mut mock_did_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(did_auth_key.public()));
	mock_did_details.update_attestation_key(did::DidVerificationKey::from(did_att_key.public()), 0u64);

	let caller_did = did_mock::ALICE_DID;
	let claim_hash = get_claim_hash(true);
	let attestation = generate_base_attestation(caller_did.clone());

	let mut operation = generate_base_attestation_revocation_operation(claim_hash, attestation.clone());
	operation.tx_counter = mock_did_details.get_tx_counter_value();
	let signature = did_att_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(caller_did.clone(), mock_did_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(attestation.ctype_hash, caller_did)]);
	let builder = delegation_mock::ExtBuilder::from(builder);
	let builder = ExtBuilder::from(builder).with_attestations(vec![(claim_hash, attestation)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Attestation::submit_attestation_revocation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidNonce
		);
	});

	// Verify that the DID tx counter has not increased
	let new_mock_details =
		ext.execute_with(|| Did::get_did(&operation.revoker_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_mock_details.get_tx_counter_value(),
		mock_did_details.get_tx_counter_value()
	);
}

#[test]
fn check_too_large_tx_counter_submit_attestation_revocation_operation() {
	let did_auth_key = did_mock::get_ed25519_authentication_key(true);
	let did_att_key = did_mock::get_sr25519_attestation_key(true);
	let mut mock_did_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(did_auth_key.public()));
	mock_did_details.update_attestation_key(did::DidVerificationKey::from(did_att_key.public()), 0u64);

	let caller_did = did_mock::ALICE_DID;
	let claim_hash = get_claim_hash(true);
	let attestation = generate_base_attestation(caller_did.clone());

	let mut operation = generate_base_attestation_revocation_operation(claim_hash, attestation.clone());
	operation.tx_counter = mock_did_details.get_tx_counter_value() + 2u64;
	let signature = did_att_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(caller_did.clone(), mock_did_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(attestation.ctype_hash, caller_did)]);
	let builder = delegation_mock::ExtBuilder::from(builder);
	let builder = ExtBuilder::from(builder).with_attestations(vec![(claim_hash, attestation)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Attestation::submit_attestation_revocation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidNonce
		);
	});

	// Verify that the DID tx counter has not increased
	let new_mock_details =
		ext.execute_with(|| Did::get_did(&operation.revoker_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_mock_details.get_tx_counter_value(),
		mock_did_details.get_tx_counter_value()
	);
}

#[test]
fn check_attestation_key_not_present_submit_attestation_revocation_operation() {
	let did_auth_key = did_mock::get_ed25519_authentication_key(true);
	let did_att_key = did_mock::get_sr25519_attestation_key(true);
	let mock_did_details = did_mock::generate_base_did_details(did::DidVerificationKey::from(did_auth_key.public()));
	// No attestation key set

	let caller_did = did_mock::ALICE_DID;
	let claim_hash = get_claim_hash(true);
	let attestation = generate_base_attestation(caller_did.clone());

	let operation = generate_base_attestation_revocation_operation(claim_hash, attestation.clone());
	let signature = did_att_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(caller_did.clone(), mock_did_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(attestation.ctype_hash, caller_did)]);
	let builder = delegation_mock::ExtBuilder::from(builder);
	let builder = ExtBuilder::from(builder).with_attestations(vec![(claim_hash, attestation)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Attestation::submit_attestation_revocation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::VerificationKeyNotPresent
		);
	});

	// Verify that the DID tx counter has not increased
	let new_mock_details =
		ext.execute_with(|| Did::get_did(&operation.revoker_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_mock_details.get_tx_counter_value(),
		mock_did_details.get_tx_counter_value()
	);
}

#[test]
fn check_invalid_signature_format_submit_attestation_revocation_operation() {
	let did_auth_key = did_mock::get_ed25519_authentication_key(true);
	let did_att_key = did_mock::get_sr25519_attestation_key(true);
	let alternative_att_key = did_mock::get_ed25519_attestation_key(true);
	let mut mock_did_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(did_auth_key.public()));
	mock_did_details.update_attestation_key(did::DidVerificationKey::from(did_att_key.public()), 0u64);

	let caller_did = did_mock::ALICE_DID;
	let claim_hash = get_claim_hash(true);
	let attestation = generate_base_attestation(caller_did.clone());

	let operation = generate_base_attestation_revocation_operation(claim_hash, attestation.clone());
	let signature = alternative_att_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(caller_did.clone(), mock_did_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(attestation.ctype_hash, caller_did)]);
	let builder = delegation_mock::ExtBuilder::from(builder);
	let builder = ExtBuilder::from(builder).with_attestations(vec![(claim_hash, attestation)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Attestation::submit_attestation_revocation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidSignatureFormat
		);
	});

	// Verify that the DID tx counter has not increased
	let new_mock_details =
		ext.execute_with(|| Did::get_did(&operation.revoker_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_mock_details.get_tx_counter_value(),
		mock_did_details.get_tx_counter_value()
	);
}

#[test]
fn check_invalid_signature_submit_attestation_revocation_operation() {
	let did_auth_key = did_mock::get_ed25519_authentication_key(true);
	let did_att_key = did_mock::get_sr25519_attestation_key(true);
	let alternative_att_key = did_mock::get_sr25519_attestation_key(false);
	let mut mock_did_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(did_auth_key.public()));
	mock_did_details.update_attestation_key(did::DidVerificationKey::from(did_att_key.public()), 0u64);

	let caller_did = did_mock::ALICE_DID;
	let claim_hash = get_claim_hash(true);
	let attestation = generate_base_attestation(caller_did.clone());

	let operation = generate_base_attestation_revocation_operation(claim_hash, attestation.clone());
	let signature = alternative_att_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(caller_did.clone(), mock_did_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(attestation.ctype_hash, caller_did)]);
	let builder = delegation_mock::ExtBuilder::from(builder);
	let builder = ExtBuilder::from(builder).with_attestations(vec![(claim_hash, attestation)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Attestation::submit_attestation_revocation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidSignature
		);
	});

	// Verify that the DID tx counter has not increased
	let new_mock_details =
		ext.execute_with(|| Did::get_did(&operation.revoker_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_mock_details.get_tx_counter_value(),
		mock_did_details.get_tx_counter_value()
	);
}

#[test]
fn check_attestation_not_present_submit_attestation_revocation_operation() {
	let did_auth_key = did_mock::get_ed25519_authentication_key(true);
	let did_att_key = did_mock::get_sr25519_attestation_key(true);
	let mut mock_did_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(did_auth_key.public()));
	mock_did_details.update_attestation_key(did::DidVerificationKey::from(did_att_key.public()), 0u64);

	let caller_did = did_mock::ALICE_DID;
	let claim_hash = get_claim_hash(true);
	let attestation = generate_base_attestation(caller_did.clone());

	let operation = generate_base_attestation_revocation_operation(claim_hash, attestation.clone());
	let signature = did_att_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(caller_did.clone(), mock_did_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(attestation.ctype_hash, caller_did)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_err!(
			Attestation::submit_attestation_revocation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			attestation::Error::<Test>::AttestationNotFound
		);
	});

	// Verify that the DID tx counter has increased
	let new_mock_details =
		ext.execute_with(|| Did::get_did(&operation.revoker_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_mock_details.get_tx_counter_value(),
		mock_did_details.get_tx_counter_value() + 1u64
	);
}

#[test]
fn check_already_revoked_attestation_submit_attestation_revocation_operation() {
	let did_auth_key = did_mock::get_ed25519_authentication_key(true);
	let did_att_key = did_mock::get_sr25519_attestation_key(true);
	let mut mock_did_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(did_auth_key.public()));
	mock_did_details.update_attestation_key(did::DidVerificationKey::from(did_att_key.public()), 0u64);

	let caller_did = did_mock::ALICE_DID;
	let claim_hash = get_claim_hash(true);
	let mut attestation = generate_base_attestation(caller_did.clone());
	attestation.revoked = true;

	let operation = generate_base_attestation_revocation_operation(claim_hash, attestation.clone());
	let signature = did_att_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(caller_did.clone(), mock_did_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(attestation.ctype_hash, caller_did)]);
	let builder = delegation_mock::ExtBuilder::from(builder);
	let builder = ExtBuilder::from(builder).with_attestations(vec![(claim_hash, attestation)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_err!(
			Attestation::submit_attestation_revocation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			attestation::Error::<Test>::AlreadyRevoked
		);
	});

	// Verify that the DID tx counter has increased
	let new_mock_details =
		ext.execute_with(|| Did::get_did(&operation.revoker_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_mock_details.get_tx_counter_value(),
		mock_did_details.get_tx_counter_value() + 1u64
	);
}

#[test]
fn check_unauthorised_attestation_no_delegation_submit_attestation_revocation_operation() {
	let did_auth_key = did_mock::get_ed25519_authentication_key(true);
	let did_att_key = did_mock::get_sr25519_attestation_key(true);
	let mut mock_did_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(did_auth_key.public()));
	mock_did_details.update_attestation_key(did::DidVerificationKey::from(did_att_key.public()), 0u64);

	let caller_did = did_mock::ALICE_DID;
	let alternative_did = did_mock::BOB_DID;
	let claim_hash = get_claim_hash(true);
	let attestation = generate_base_attestation(alternative_did.clone());

	let mut operation = generate_base_attestation_revocation_operation(claim_hash, attestation.clone());
	operation.revoker_did = caller_did.clone();
	let signature = did_att_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(caller_did.clone(), mock_did_details.clone()),
		(alternative_did, mock_did_details.clone()),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(attestation.ctype_hash, caller_did)]);
	let builder = delegation_mock::ExtBuilder::from(builder);
	let builder = ExtBuilder::from(builder).with_attestations(vec![(claim_hash, attestation)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_err!(
			Attestation::submit_attestation_revocation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			attestation::Error::<Test>::UnauthorizedRevocation
		);
	});

	// Verify that the DID tx counter has increased
	let new_mock_details =
		ext.execute_with(|| Did::get_did(&operation.revoker_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_mock_details.get_tx_counter_value(),
		mock_did_details.get_tx_counter_value() + 1u64
	);
}

#[test]
fn check_max_parent_lookups_submit_attestation_revocation_operation() {
	let did_auth_key = did_mock::get_ed25519_authentication_key(true);
	let did_att_key = did_mock::get_sr25519_attestation_key(true);
	let mut mock_did_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(did_auth_key.public()));
	mock_did_details.update_attestation_key(did::DidVerificationKey::from(did_att_key.public()), 0u64);

	let caller_did = did_mock::ALICE_DID;
	let alternative_did = did_mock::BOB_DID;
	let claim_hash = get_claim_hash(true);
	let (root_id, root_node) = (
		delegation_mock::get_delegation_root_id(true),
		delegation_mock::generate_base_delegation_root(alternative_did.clone()),
	);
	let (parent_delegation_id, parent_delegation_node) = (
		delegation_mock::get_delegation_id(true),
		delegation_mock::generate_base_delegation_node(root_id, caller_did.clone()),
	);
	let (delegation_id, mut delegation_node) = (
		delegation_mock::get_delegation_id(true),
		delegation_mock::generate_base_delegation_node(root_id, alternative_did.clone()),
	);
	delegation_node.permissions = delegation::Permissions::ATTEST;
	delegation_node.parent = Some(parent_delegation_id);
	let mut attestation = generate_base_attestation(alternative_did.clone());
	attestation.delegation_id = Some(delegation_id);

	let mut operation = generate_base_attestation_revocation_operation(claim_hash, attestation.clone());
	operation.revoker_did = caller_did.clone();
	operation.max_parent_checks = 0u32;
	let signature = did_att_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(caller_did.clone(), mock_did_details.clone()),
		(alternative_did, mock_did_details.clone()),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, caller_did)]);
	let builder = delegation_mock::ExtBuilder::from(builder)
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![
			(parent_delegation_id, parent_delegation_node),
			(delegation_id, delegation_node),
		])
		.with_children(vec![
			(root_id, vec![parent_delegation_id]),
			(parent_delegation_id, vec![delegation_id]),
		]);
	let builder = ExtBuilder::from(builder).with_attestations(vec![(claim_hash, attestation)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_err!(
			Attestation::submit_attestation_revocation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			delegation::Error::<Test>::MaxSearchDepthReached
		);
	});

	// Verify that the DID tx counter has increased
	let new_mock_details =
		ext.execute_with(|| Did::get_did(&operation.revoker_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_mock_details.get_tx_counter_value(),
		mock_did_details.get_tx_counter_value() + 1u64
	);
}

#[test]
fn check_revoked_delegation_submit_attestation_revocation_operation() {
	let did_auth_key = did_mock::get_ed25519_authentication_key(true);
	let did_att_key = did_mock::get_sr25519_attestation_key(true);
	let mut mock_did_details =
		did_mock::generate_base_did_details(did::DidVerificationKey::from(did_auth_key.public()));
	mock_did_details.update_attestation_key(did::DidVerificationKey::from(did_att_key.public()), 0u64);

	let caller_did = did_mock::ALICE_DID;
	let alternative_did = did_mock::BOB_DID;
	let claim_hash = get_claim_hash(true);
	let (root_id, root_node) = (
		delegation_mock::get_delegation_root_id(true),
		delegation_mock::generate_base_delegation_root(alternative_did.clone()),
	);
	let (delegation_id, mut delegation_node) = (
		delegation_mock::get_delegation_id(true),
		delegation_mock::generate_base_delegation_node(root_id, caller_did.clone()),
	);
	delegation_node.permissions = delegation::Permissions::ATTEST;
	delegation_node.revoked = true;
	let mut attestation = generate_base_attestation(alternative_did);
	attestation.delegation_id = Some(delegation_id);

	let mut operation = generate_base_attestation_revocation_operation(claim_hash, attestation.clone());
	operation.revoker_did = caller_did.clone();
	let signature = did_att_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(caller_did.clone(), mock_did_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(root_node.ctype_hash, caller_did)]);
	let builder = delegation_mock::ExtBuilder::from(builder)
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![(delegation_id, delegation_node)]);
	let builder = ExtBuilder::from(builder).with_attestations(vec![(claim_hash, attestation)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_err!(
			Attestation::submit_attestation_revocation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			attestation::Error::<Test>::UnauthorizedRevocation
		);
	});

	// Verify that the DID tx counter has increased
	let new_mock_details =
		ext.execute_with(|| Did::get_did(&operation.revoker_did).expect("DID submitter should be present on chain."));
	assert_eq!(
		new_mock_details.get_tx_counter_value(),
		mock_did_details.get_tx_counter_value() + 1u64
	);
}
