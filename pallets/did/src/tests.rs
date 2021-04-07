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

use frame_support::{assert_noop, assert_ok};
use sp_core::Pair;
use sp_std::collections::btree_set::BTreeSet;
use std::iter::FromIterator;

use codec::Encode;

use crate::{self as did, mock::*, PublicVerificationKey, UrlEncoding};

// submit_did_create_operation

#[test]
fn check_successful_simple_ed25519_creation() {
	let auth_key = get_ed25519_authentication_key(true);
	let enc_key = get_x25519_encryption_key(true);
	let did_creation_operation =
		generate_base_did_creation_operation(ALICE_DID, did::PublicVerificationKey::from(auth_key.public()), enc_key);

	let signature = auth_key.sign(did_creation_operation.encode().as_ref());

	let mut ext = ExtBuilder::default().build();

	ext.execute_with(|| {
		assert_ok!(Did::submit_did_create_operation(
			Origin::signed(DEFAULT_ACCOUNT),
			did_creation_operation.clone(),
			did::DidSignature::from(signature),
		));
	});

	let stored_did = ext.execute_with(|| Did::get_did(ALICE_DID).expect("ALICE_DID should be present on chain."));
	assert_eq!(stored_did.auth_key, did_creation_operation.new_auth_key);
	assert_eq!(
		stored_did.key_agreement_key,
		did_creation_operation.new_key_agreement_key
	);
	assert_eq!(stored_did.delegation_key, did_creation_operation.new_delegation_key);
	assert_eq!(stored_did.attestation_key, did_creation_operation.new_attestation_key);
	assert_eq!(
		stored_did.verification_keys,
		<BTreeSet<did::PublicVerificationKey>>::new()
	);
	assert_eq!(stored_did.endpoint_url, did_creation_operation.new_endpoint_url);
	assert_eq!(stored_did.last_tx_counter, 0u64);
}

#[test]
fn check_successful_simple_sr25519_creation() {
	let auth_key = get_sr25519_authentication_key(true);
	let enc_key = get_x25519_encryption_key(true);
	let did_creation_operation =
		generate_base_did_creation_operation(ALICE_DID, did::PublicVerificationKey::from(auth_key.public()), enc_key);

	let signature = auth_key.sign(did_creation_operation.encode().as_ref());

	let mut ext = ExtBuilder::default().build();

	ext.execute_with(|| {
		assert_ok!(Did::submit_did_create_operation(
			Origin::signed(DEFAULT_ACCOUNT),
			did_creation_operation.clone(),
			did::DidSignature::from(signature),
		));
	});

	let stored_did = ext.execute_with(|| Did::get_did(ALICE_DID).expect("ALICE_DID should be present on chain."));
	assert_eq!(stored_did.auth_key, did_creation_operation.new_auth_key);
	assert_eq!(
		stored_did.key_agreement_key,
		did_creation_operation.new_key_agreement_key
	);
	assert_eq!(stored_did.delegation_key, did_creation_operation.new_delegation_key);
	assert_eq!(stored_did.attestation_key, did_creation_operation.new_attestation_key);
	assert_eq!(
		stored_did.verification_keys,
		<BTreeSet<did::PublicVerificationKey>>::new()
	);
	assert_eq!(stored_did.endpoint_url, did_creation_operation.new_endpoint_url);
	assert_eq!(stored_did.last_tx_counter, 0u64);
}

#[test]
fn check_successful_complete_creation() {
	let auth_key = get_sr25519_authentication_key(true);
	let enc_key = get_x25519_encryption_key(true);
	let del_key = get_sr25519_delegation_key(true);
	let att_key = get_ed25519_attestation_key(true);
	let mut did_creation_operation =
		generate_base_did_creation_operation(ALICE_DID, did::PublicVerificationKey::from(auth_key.public()), enc_key);
	did_creation_operation.new_attestation_key = Some(did::PublicVerificationKey::from(att_key.public()));
	did_creation_operation.new_delegation_key = Some(did::PublicVerificationKey::from(del_key.public()));
	did_creation_operation.new_endpoint_url = Some("https://kilt.io".into());

	let signature = auth_key.sign(did_creation_operation.encode().as_ref());

	let mut ext = ExtBuilder::default().build();

	ext.execute_with(|| {
		assert_ok!(Did::submit_did_create_operation(
			Origin::signed(DEFAULT_ACCOUNT),
			did_creation_operation.clone(),
			did::DidSignature::from(signature),
		));
	});

	let stored_did = ext.execute_with(|| Did::get_did(ALICE_DID).expect("ALICE_DID should be present on chain."));
	assert_eq!(stored_did.auth_key, did_creation_operation.new_auth_key);
	assert_eq!(
		stored_did.key_agreement_key,
		did_creation_operation.new_key_agreement_key
	);
	assert_eq!(stored_did.delegation_key, did_creation_operation.new_delegation_key);
	assert_eq!(stored_did.attestation_key, did_creation_operation.new_attestation_key);
	assert_eq!(
		stored_did.verification_keys,
		<BTreeSet<did::PublicVerificationKey>>::new()
	);
	assert_eq!(stored_did.endpoint_url, did_creation_operation.new_endpoint_url);
	assert_eq!(stored_did.last_tx_counter, 0u64);
}

#[test]
fn check_duplicate_did_creation() {
	let auth_key = get_sr25519_authentication_key(true);
	let enc_key = get_x25519_encryption_key(true);
	let mock_did = generate_mock_did_details(did::PublicVerificationKey::from(auth_key.public()), enc_key);
	let did_creation_operation =
		generate_base_did_creation_operation(ALICE_DID, did::PublicVerificationKey::from(auth_key.public()), enc_key);

	let signature = auth_key.sign(did_creation_operation.encode().as_ref());

	let mut ext = ExtBuilder::default().with_dids(vec![(ALICE_DID, mock_did)]).build();

	ext.execute_with(|| {
		assert_noop!(
			Did::submit_did_create_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				did_creation_operation.clone(),
				did::DidSignature::from(signature),
			),
			did::Error::<Test>::DidAlreadyPresent
		);
	});
}

#[test]
fn check_invalid_signature_format_did_creation() {
	let auth_key = get_sr25519_authentication_key(true);
	let enc_key = get_x25519_encryption_key(true);
	// Using an Ed25519 key where an Sr25519 is expected
	let invalid_key = get_ed25519_authentication_key(true);
	// DID creation contains auth_key, but signature is generated using invalid_key
	let did_creation_operation =
		generate_base_did_creation_operation(ALICE_DID, did::PublicVerificationKey::from(auth_key.public()), enc_key);

	let signature = invalid_key.sign(did_creation_operation.encode().as_ref());

	let mut ext = ExtBuilder::default().build();

	ext.execute_with(|| {
		assert_noop!(
			Did::submit_did_create_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				did_creation_operation.clone(),
				did::DidSignature::from(signature),
			),
			did::Error::<Test>::InvalidSignatureFormat
		);
	});
}

#[test]
fn check_invalid_signature_did_creation() {
	let auth_key = get_sr25519_authentication_key(true);
	let enc_key = get_x25519_encryption_key(true);
	// Using an Sr25519 key as expected, but from a different seed (default = false)
	let alternative_key = get_sr25519_authentication_key(false);
	// DID creation contains auth_key, but signature is generated using
	// alternative_key
	let did_creation_operation =
		generate_base_did_creation_operation(ALICE_DID, did::PublicVerificationKey::from(auth_key.public()), enc_key);

	let signature = alternative_key.sign(did_creation_operation.encode().as_ref());

	let mut ext = ExtBuilder::default().build();

	ext.execute_with(|| {
		assert_noop!(
			Did::submit_did_create_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				did_creation_operation.clone(),
				did::DidSignature::from(signature),
			),
			did::Error::<Test>::InvalidSignature
		);
	});
}

// submit_did_update_operation

#[test]
fn check_successful_complete_update() {
	let old_auth_key = get_ed25519_authentication_key(true);
	let new_auth_key = get_ed25519_authentication_key(false);
	let old_enc_key = get_x25519_encryption_key(true);
	let new_enc_key = get_x25519_encryption_key(false);
	let old_att_key = get_ed25519_attestation_key(true);
	let new_att_key = get_ed25519_attestation_key(false);
	let new_del_key = get_sr25519_attestation_key(true);
	let new_url: UrlEncoding = "https://new_kilt.io".into();

	let mut old_did_details =
		generate_mock_did_details(did::PublicVerificationKey::from(old_auth_key.public()), old_enc_key);
	old_did_details.attestation_key = Some(PublicVerificationKey::from(old_att_key.public()));

	// Update all keys, URL endpoint and tx counter. No keys are removed in this
	// test
	let mut did_update_operation = generate_base_did_update_operation(ALICE_DID);
	did_update_operation.new_auth_key = Some(PublicVerificationKey::from(new_auth_key.public()));
	did_update_operation.new_key_agreement_key = Some(new_enc_key);
	did_update_operation.new_attestation_key = Some(PublicVerificationKey::from(new_att_key.public()));
	did_update_operation.new_delegation_key = Some(PublicVerificationKey::from(new_del_key.public()));
	did_update_operation.new_endpoint_url = Some(new_url);
	did_update_operation.tx_counter = old_did_details.last_tx_counter + 1u64;

	// Generate signature using the old authentication key
	let signature = old_auth_key.sign(did_update_operation.encode().as_ref());

	let mut ext = ExtBuilder::default()
		.with_dids(vec![(ALICE_DID, old_did_details.clone())])
		.build();

	ext.execute_with(|| {
		assert_ok!(Did::submit_did_update_operation(
			Origin::signed(DEFAULT_ACCOUNT),
			did_update_operation.clone(),
			did::DidSignature::from(signature),
		));
	});
	let new_did_details = ext.execute_with(|| Did::get_did(ALICE_DID).expect("ALICE_DID should be present on chain."));
	assert_eq!(
		new_did_details.auth_key,
		did_update_operation.new_auth_key.expect("Missing new auth key.")
	);
	assert_eq!(
		new_did_details.key_agreement_key,
		did_update_operation
			.new_key_agreement_key
			.expect("Missing new key agreement key.")
	);
	assert_eq!(new_did_details.delegation_key, did_update_operation.new_delegation_key);
	assert_eq!(
		new_did_details.attestation_key,
		did_update_operation.new_attestation_key
	);
	// Verification keys should contain the previous attestation key.
	assert_eq!(
		new_did_details.verification_keys,
		BTreeSet::from_iter(vec![PublicVerificationKey::from(old_att_key.public())].into_iter())
	);
	assert_eq!(new_did_details.endpoint_url, did_update_operation.new_endpoint_url);
	assert_eq!(new_did_details.last_tx_counter, did_update_operation.tx_counter);
}

#[test]
fn check_successful_verification_keys_deletion() {
	let auth_key = get_ed25519_authentication_key(true);
	let enc_key = get_x25519_encryption_key(true);
	let old_verification_keys_vector = vec![
		PublicVerificationKey::from(get_ed25519_attestation_key(true).public()),
		PublicVerificationKey::from(get_ed25519_attestation_key(false).public()),
		PublicVerificationKey::from(get_sr25519_attestation_key(true).public()),
		PublicVerificationKey::from(get_sr25519_attestation_key(false).public()),
	];
	let old_verification_keys_set = BTreeSet::from_iter(old_verification_keys_vector.into_iter());
	let mut old_did_details = generate_mock_did_details(PublicVerificationKey::from(auth_key.public()), enc_key);
	old_did_details.verification_keys = old_verification_keys_set.clone();

	// Create update operation to remove all verification keys
	let mut did_update_operation = generate_base_did_update_operation(ALICE_DID);
	did_update_operation.verification_keys_to_remove = Some(old_verification_keys_set);

	let signature = auth_key.sign(did_update_operation.encode().as_ref());

	let mut ext = ExtBuilder::default()
		.with_dids(vec![(ALICE_DID, old_did_details.clone())])
		.build();

	ext.execute_with(|| {
		assert_ok!(Did::submit_did_update_operation(
			Origin::signed(DEFAULT_ACCOUNT),
			did_update_operation.clone(),
			did::DidSignature::from(signature),
		));
	});
	let new_did_details = ext.execute_with(|| Did::get_did(ALICE_DID).expect("ALICE_DID should be present on chain."));
	// All fields but verification_keys should remain unchanged
	assert_eq!(new_did_details.auth_key, old_did_details.auth_key);
	assert_eq!(new_did_details.key_agreement_key, old_did_details.key_agreement_key);
	assert_eq!(new_did_details.delegation_key, old_did_details.delegation_key);
	assert_eq!(new_did_details.attestation_key, old_did_details.attestation_key);
	assert_eq!(new_did_details.endpoint_url, old_did_details.endpoint_url);
	assert_eq!(new_did_details.last_tx_counter, did_update_operation.tx_counter);

	// Set of verification keys should be empty now
	assert_eq!(new_did_details.verification_keys, BTreeSet::new());
}

#[test]
fn check_did_not_present_update() {
	let auth_key = get_ed25519_authentication_key(true);
	let enc_key = get_x25519_encryption_key(true);
	let mock_did = generate_mock_did_details(PublicVerificationKey::from(auth_key.public()), enc_key);
	let did_update_operation = generate_base_did_update_operation(BOB_DID);

	let signature = auth_key.sign(did_update_operation.encode().as_ref());

	let mut ext = ExtBuilder::default().with_dids(vec![(ALICE_DID, mock_did)]).build();

	ext.execute_with(|| {
		assert_noop!(
			Did::submit_did_update_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				did_update_operation.clone(),
				did::DidSignature::from(signature),
			),
			did::Error::<Test>::DidNotPresent
		);
	});
}

#[test]
fn check_invalid_signature_format_did_update() {
	let auth_key = get_ed25519_authentication_key(true);
	let enc_key = get_x25519_encryption_key(true);
	// Using an Sr25519 key where an Ed25519 is expected
	let invalid_key = get_sr25519_authentication_key(true);
	let mock_did = generate_mock_did_details(did::PublicVerificationKey::from(auth_key.public()), enc_key);
	// DID update contains auth_key, but signature is generated using invalid_key
	let did_update_operation = generate_base_did_update_operation(ALICE_DID);

	let signature = invalid_key.sign(did_update_operation.encode().as_ref());

	let mut ext = ExtBuilder::default().with_dids(vec![(ALICE_DID, mock_did)]).build();

	ext.execute_with(|| {
		assert_noop!(
			Did::submit_did_update_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				did_update_operation.clone(),
				did::DidSignature::from(signature),
			),
			did::Error::<Test>::InvalidSignatureFormat
		);
	});
}

#[test]
fn check_invalid_signature_did_update() {
	let auth_key = get_sr25519_authentication_key(true);
	let enc_key = get_x25519_encryption_key(true);
	// Using an Sr25519 key as expected, but from a different seed (default = false)
	let alternative_key = get_sr25519_authentication_key(false);
	let mock_did = generate_mock_did_details(PublicVerificationKey::from(auth_key.public()), enc_key);
	let did_update_operation = generate_base_did_update_operation(ALICE_DID);

	let signature = alternative_key.sign(did_update_operation.encode().as_ref());

	let mut ext = ExtBuilder::default().with_dids(vec![(ALICE_DID, mock_did)]).build();

	ext.execute_with(|| {
		assert_noop!(
			Did::submit_did_update_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				did_update_operation.clone(),
				did::DidSignature::from(signature),
			),
			did::Error::<Test>::InvalidSignature
		);
	});
}

#[test]
fn check_invalid_verification_keys_deletion() {
	let auth_key = get_ed25519_authentication_key(true);
	let enc_key = get_x25519_encryption_key(true);
	let key1 = PublicVerificationKey::from(get_ed25519_attestation_key(true).public());
	let key2 = PublicVerificationKey::from(get_ed25519_attestation_key(false).public());
	let key3 = PublicVerificationKey::from(get_sr25519_attestation_key(true).public());
	let key4 = PublicVerificationKey::from(get_sr25519_attestation_key(false).public());
	let old_verification_keys_vector = vec![key1, key2, key3];
	let old_verification_keys_set = BTreeSet::from_iter(old_verification_keys_vector.into_iter());
	let mut old_did_details = generate_mock_did_details(PublicVerificationKey::from(auth_key.public()), enc_key);
	old_did_details.verification_keys = old_verification_keys_set;

	// Remove some verification keys including one not stored on chain (key4)
	let verification_keys_to_remove = vec![key1, key3, key4];
	let mut did_update_operation = generate_base_did_update_operation(ALICE_DID);
	did_update_operation.verification_keys_to_remove =
		Some(BTreeSet::from_iter(verification_keys_to_remove.into_iter()));

	let signature = auth_key.sign(did_update_operation.encode().as_ref());

	let mut ext = ExtBuilder::default()
		.with_dids(vec![(ALICE_DID, old_did_details.clone())])
		.build();

	ext.execute_with(|| {
		assert_noop!(
			Did::submit_did_update_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				did_update_operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::VerificationKeysNotPresent
		);
	});
}

#[test]
fn check_smaller_tx_counter_did_update() {
	let auth_key = get_sr25519_authentication_key(true);
	let enc_key = get_x25519_encryption_key(true);
	let mock_did = generate_mock_did_details(PublicVerificationKey::from(auth_key.public()), enc_key);
	let mut did_update_operation = generate_base_did_update_operation(ALICE_DID);
	did_update_operation.tx_counter = 0;

	let signature = auth_key.sign(did_update_operation.encode().as_ref());

	let mut ext = ExtBuilder::default().with_dids(vec![(ALICE_DID, mock_did)]).build();

	ext.execute_with(|| {
		assert_noop!(
			Did::submit_did_update_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				did_update_operation.clone(),
				did::DidSignature::from(signature),
			),
			did::Error::<Test>::InvalidNonce
		);
	});
}

#[test]
fn check_equal_tx_counter_did_update() {
	let auth_key = get_sr25519_authentication_key(true);
	let enc_key = get_x25519_encryption_key(true);
	let mock_did = generate_mock_did_details(PublicVerificationKey::from(auth_key.public()), enc_key);
	let mut did_update_operation = generate_base_did_update_operation(ALICE_DID);
	did_update_operation.tx_counter = mock_did.last_tx_counter;

	let signature = auth_key.sign(did_update_operation.encode().as_ref());

	let mut ext = ExtBuilder::default().with_dids(vec![(ALICE_DID, mock_did)]).build();

	ext.execute_with(|| {
		assert_noop!(
			Did::submit_did_update_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				did_update_operation.clone(),
				did::DidSignature::from(signature),
			),
			did::Error::<Test>::InvalidNonce
		);
	});
}

// submit_did_delete_operation

#[test]
fn check_successful_deletion() {
	let auth_key = get_ed25519_authentication_key(true);
	let enc_key = get_x25519_encryption_key(true);

	let did_details = generate_mock_did_details(did::PublicVerificationKey::from(auth_key.public()), enc_key);

	// Update all keys, URL endpoint and tx counter. No keys are removed in this
	// test
	let did_delete_operation = generate_base_did_delete_operation(ALICE_DID);

	// Generate signature using the old authentication key
	let signature = auth_key.sign(did_delete_operation.encode().as_ref());

	let mut ext = ExtBuilder::default()
		.with_dids(vec![(ALICE_DID, did_details.clone())])
		.build();

	ext.execute_with(|| {
		assert_ok!(Did::submit_did_delete_operation(
			Origin::signed(DEFAULT_ACCOUNT),
			did_delete_operation.clone(),
			did::DidSignature::from(signature),
		));
	});

	assert_eq!(ext.execute_with(|| Did::get_did(ALICE_DID)), None);

	// Re-adding the same DID identifier, which should not fail.
	let auth_key = get_sr25519_authentication_key(true);
	let enc_key = get_x25519_encryption_key(true);
	let did_creation_operation =
		generate_base_did_creation_operation(ALICE_DID, did::PublicVerificationKey::from(auth_key.public()), enc_key);

	let signature = auth_key.sign(did_creation_operation.encode().as_ref());

	let mut ext = ExtBuilder::default().build();

	ext.execute_with(|| {
		assert_ok!(Did::submit_did_create_operation(
			Origin::signed(DEFAULT_ACCOUNT),
			did_creation_operation.clone(),
			did::DidSignature::from(signature),
		));
	});
}

#[test]
fn check_did_not_present_deletion() {
	let auth_key = get_ed25519_authentication_key(true);

	// Update all keys, URL endpoint and tx counter. No keys are removed in this
	// test
	let did_delete_operation = generate_base_did_delete_operation(ALICE_DID);

	// Generate signature using the old authentication key
	let signature = auth_key.sign(did_delete_operation.encode().as_ref());

	let mut ext = ExtBuilder::default().build();

	ext.execute_with(|| {
		assert_noop!(
			Did::submit_did_delete_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				did_delete_operation.clone(),
				did::DidSignature::from(signature),
			),
			did::Error::<Test>::DidNotPresent
		);
	});
}

#[test]
fn check_invalid_signature_format_did_deletion() {
	let auth_key = get_ed25519_authentication_key(true);
	let enc_key = get_x25519_encryption_key(true);
	// Using an Sr25519 key where an Ed25519 is expected
	let invalid_key = get_sr25519_authentication_key(true);
	let mock_did = generate_mock_did_details(did::PublicVerificationKey::from(auth_key.public()), enc_key);
	// DID update contains auth_key, but signature is generated using invalid_key
	let did_deletion_operation = generate_base_did_delete_operation(ALICE_DID);

	let signature = invalid_key.sign(did_deletion_operation.encode().as_ref());

	let mut ext = ExtBuilder::default().with_dids(vec![(ALICE_DID, mock_did)]).build();

	ext.execute_with(|| {
		assert_noop!(
			Did::submit_did_delete_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				did_deletion_operation.clone(),
				did::DidSignature::from(signature),
			),
			did::Error::<Test>::InvalidSignatureFormat
		);
	});
}

#[test]
fn check_invalid_signature_did_deletion() {
	let auth_key = get_sr25519_authentication_key(true);
	let enc_key = get_x25519_encryption_key(true);
	// Using an Sr25519 key as expected, but from a different seed (default = false)
	let alternative_key = get_sr25519_authentication_key(false);
	let mock_did = generate_mock_did_details(PublicVerificationKey::from(auth_key.public()), enc_key);
	let did_delete_operation = generate_base_did_delete_operation(ALICE_DID);

	let signature = alternative_key.sign(did_delete_operation.encode().as_ref());

	let mut ext = ExtBuilder::default().with_dids(vec![(ALICE_DID, mock_did)]).build();

	ext.execute_with(|| {
		assert_noop!(
			Did::submit_did_delete_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				did_delete_operation.clone(),
				did::DidSignature::from(signature),
			),
			did::Error::<Test>::InvalidSignature
		);
	});
}

#[test]
fn check_smaller_tx_counter_did_deletion() {
	let auth_key = get_sr25519_authentication_key(true);
	let enc_key = get_x25519_encryption_key(true);
	let mut mock_did = generate_mock_did_details(PublicVerificationKey::from(auth_key.public()), enc_key);
	mock_did.last_tx_counter = 1;
	let mut did_delete_operation = generate_base_did_delete_operation(ALICE_DID);
	did_delete_operation.tx_counter = 0;

	let signature = auth_key.sign(did_delete_operation.encode().as_ref());

	let mut ext = ExtBuilder::default().with_dids(vec![(ALICE_DID, mock_did)]).build();

	ext.execute_with(|| {
		assert_noop!(
			Did::submit_did_delete_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				did_delete_operation.clone(),
				did::DidSignature::from(signature),
			),
			did::Error::<Test>::InvalidNonce
		);
	});
}

#[test]
fn check_equal_tx_counter_did_deletion() {
	let auth_key = get_sr25519_authentication_key(true);
	let enc_key = get_x25519_encryption_key(true);
	let mut mock_did = generate_mock_did_details(PublicVerificationKey::from(auth_key.public()), enc_key);
	mock_did.last_tx_counter = 1;
	let mut did_delete_operation = generate_base_did_delete_operation(ALICE_DID);
	did_delete_operation.tx_counter = mock_did.last_tx_counter;

	let signature = auth_key.sign(did_delete_operation.encode().as_ref());

	let mut ext = ExtBuilder::default().with_dids(vec![(ALICE_DID, mock_did)]).build();

	ext.execute_with(|| {
		assert_noop!(
			Did::submit_did_delete_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				did_delete_operation.clone(),
				did::DidSignature::from(signature),
			),
			did::Error::<Test>::InvalidNonce
		);
	});
}

// Internal function: verify_did_operation_signature

#[test]
fn check_authentication_successful_operation_verification() {
	let auth_key = get_sr25519_authentication_key(true);
	let enc_key = get_x25519_encryption_key(true);
	let mock_did = generate_mock_did_details(did::PublicVerificationKey::from(auth_key.public()), enc_key);
	let did_operation = TestDIDOperation {
		did: ALICE_DID,
		verification_key_type: did::DidVerificationKeyType::Authentication,
	};

	let did_operation_signature = auth_key.sign(&did_operation.encode());

	let mut ext = ExtBuilder::default().with_dids(vec![(ALICE_DID, mock_did)]).build();

	ext.execute_with(|| {
		assert_ok!(Did::verify_did_operation_signature::<TestDIDOperation>(
			&did_operation,
			&did::DidSignature::from(did_operation_signature)
		));
	});
}

#[test]
fn check_attestation_successful_operation_verification() {
	let auth_key = get_ed25519_authentication_key(true);
	let enc_key = get_x25519_encryption_key(true);
	let att_key = get_sr25519_attestation_key(true);
	let mut mock_did = generate_mock_did_details(did::PublicVerificationKey::from(auth_key.public()), enc_key);
	mock_did.attestation_key = Some(did::PublicVerificationKey::from(att_key.public()));
	let did_operation = TestDIDOperation {
		did: ALICE_DID,
		verification_key_type: did::DidVerificationKeyType::AssertionMethod,
	};

	let did_operation_signature = att_key.sign(&did_operation.encode());

	let mut ext = ExtBuilder::default().with_dids(vec![(ALICE_DID, mock_did)]).build();

	ext.execute_with(|| {
		assert_ok!(Did::verify_did_operation_signature::<TestDIDOperation>(
			&did_operation,
			&did::DidSignature::from(did_operation_signature)
		));
	});
}

#[test]
fn check_delegation_successful_operation_verification() {
	let auth_key = get_ed25519_authentication_key(true);
	let enc_key = get_x25519_encryption_key(true);
	let del_key = get_ed25519_delegation_key(true);
	let mut mock_did = generate_mock_did_details(did::PublicVerificationKey::from(auth_key.public()), enc_key);
	mock_did.delegation_key = Some(did::PublicVerificationKey::from(del_key.public()));
	let did_operation = TestDIDOperation {
		did: ALICE_DID,
		verification_key_type: did::DidVerificationKeyType::CapabilityDelegation,
	};
	let did_operation_signature = del_key.sign(&did_operation.encode());

	let mut ext = ExtBuilder::default().with_dids(vec![(ALICE_DID, mock_did)]).build();

	ext.execute_with(|| {
		assert_ok!(Did::verify_did_operation_signature::<TestDIDOperation>(
			&did_operation,
			&did::DidSignature::from(did_operation_signature)
		));
	});
}

#[test]
fn check_did_not_present_operation_verification() {
	let auth_key = get_ed25519_authentication_key(true);
	let enc_key = get_x25519_encryption_key(true);
	let mock_did = generate_mock_did_details(did::PublicVerificationKey::from(auth_key.public()), enc_key);
	let did_operation = TestDIDOperation {
		did: BOB_DID,
		verification_key_type: did::DidVerificationKeyType::Authentication,
	};

	let did_operation_signature = auth_key.sign(&did_operation.encode());

	let mut ext = ExtBuilder::default().with_dids(vec![(ALICE_DID, mock_did)]).build();

	ext.execute_with(|| {
		assert_noop!(
			Did::verify_did_operation_signature::<TestDIDOperation>(
				&did_operation,
				&did::DidSignature::from(did_operation_signature)
			),
			did::DidError::StorageError(did::StorageError::DidNotPresent)
		);
	});
}

#[test]
fn check_verification_key_not_present_operation_verification() {
	let auth_key = get_ed25519_authentication_key(true);
	let enc_key = get_x25519_encryption_key(true);
	let mock_did = generate_mock_did_details(did::PublicVerificationKey::from(auth_key.public()), enc_key);
	let verification_key_required = did::DidVerificationKeyType::CapabilityInvocation;
	let did_operation = TestDIDOperation {
		did: ALICE_DID,
		verification_key_type: verification_key_required.clone(),
	};

	let did_operation_signature = auth_key.sign(&did_operation.encode());

	let mut ext = ExtBuilder::default().with_dids(vec![(ALICE_DID, mock_did)]).build();

	ext.execute_with(|| {
		assert_noop!(
			Did::verify_did_operation_signature::<TestDIDOperation>(
				&did_operation,
				&did::DidSignature::from(did_operation_signature)
			),
			did::DidError::StorageError(did::StorageError::DidKeyNotPresent(verification_key_required.clone()))
		);
	});
}

#[test]
fn check_invalid_signature_format_operation_verification() {
	let auth_key = get_sr25519_authentication_key(true);
	let enc_key = get_x25519_encryption_key(true);
	// Expected an Sr25519, given an Ed25519
	let invalid_key = get_ed25519_authentication_key(true);
	let mock_did = generate_mock_did_details(did::PublicVerificationKey::from(auth_key.public()), enc_key);
	let did_operation = TestDIDOperation {
		did: ALICE_DID,
		verification_key_type: did::DidVerificationKeyType::Authentication,
	};

	let did_operation_signature = invalid_key.sign(&did_operation.encode());

	let mut ext = ExtBuilder::default().with_dids(vec![(ALICE_DID, mock_did)]).build();

	ext.execute_with(|| {
		assert_noop!(
			Did::verify_did_operation_signature::<TestDIDOperation>(
				&did_operation,
				&did::DidSignature::from(did_operation_signature)
			),
			did::DidError::SignatureError(did::SignatureError::InvalidSignatureFormat)
		);
	});
}

#[test]
fn check_invalid_signature_operation_verification() {
	let auth_key = get_sr25519_authentication_key(true);
	let enc_key = get_x25519_encryption_key(true);
	// Using same key type but different seed (default = false)
	let alternative_key = get_sr25519_authentication_key(false);
	let mock_did = generate_mock_did_details(did::PublicVerificationKey::from(auth_key.public()), enc_key);
	let did_operation = TestDIDOperation {
		did: ALICE_DID,
		verification_key_type: did::DidVerificationKeyType::Authentication,
	};

	let did_operation_signature = alternative_key.sign(&did_operation.encode());

	let mut ext = ExtBuilder::default().with_dids(vec![(ALICE_DID, mock_did)]).build();

	ext.execute_with(|| {
		assert_noop!(
			Did::verify_did_operation_signature::<TestDIDOperation>(
				&did_operation,
				&did::DidSignature::from(did_operation_signature)
			),
			did::DidError::SignatureError(did::SignatureError::InvalidSignature)
		);
	});
}
