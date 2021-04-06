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

use std::collections::BTreeSet;

use frame_support::{assert_noop, assert_ok};
use sp_core::Pair;

use codec::Encode;

use crate as did;
use crate::mock::*;

#[test]
fn check_successful_simple_ed25519_creation() {
	let auth_key = get_ed25519_authentication_key(true);
	let enc_key = get_x25519_encryption_key();
	let did_creation_operation =
		generate_simple_did_creation_operation(ALICE_DID, did::PublicVerificationKey::from(auth_key.public()), enc_key);
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
	let enc_key = get_x25519_encryption_key();
	let did_creation_operation =
		generate_simple_did_creation_operation(ALICE_DID, did::PublicVerificationKey::from(auth_key.public()), enc_key);
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
	let enc_key = get_x25519_encryption_key();
	let del_key = get_sr25519_delegation_key(true);
	let att_key = get_ed25519_attestation_key(true);
	let did_creation_operation = generate_complete_did_creation_operation(
		ALICE_DID,
		did::PublicVerificationKey::from(auth_key.public()),
		enc_key,
		Some(did::PublicVerificationKey::from(att_key.public())),
		Some(did::PublicVerificationKey::from(del_key.public())),
		Some("https://kilt.io".into()),
	);
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
	let mock_did = generate_mock_did_details();
	let auth_key = get_sr25519_authentication_key(true);
	let enc_key = get_x25519_encryption_key();
	let did_creation_operation =
		generate_simple_did_creation_operation(ALICE_DID, did::PublicVerificationKey::from(auth_key.public()), enc_key);
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
	let enc_key = get_x25519_encryption_key();
	// Using an Ed25519 key where an Sr25519 is expected
	let invalid_key = get_ed25519_authentication_key(true);
	// DID creation contains auth_key, but signature is generated using invalid_key
	let did_creation_operation =
		generate_simple_did_creation_operation(ALICE_DID, did::PublicVerificationKey::from(auth_key.public()), enc_key);
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
	let enc_key = get_x25519_encryption_key();
	// Using an Sr25519 key as expected, but from a different seed (default = false)
	let alternative_key = get_sr25519_authentication_key(false);
	// DID creation contains auth_key, but signature is generated using
	// alternative_key
	let did_creation_operation =
		generate_simple_did_creation_operation(ALICE_DID, did::PublicVerificationKey::from(auth_key.public()), enc_key);
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

#[test]
fn check_authentication_successful_operation_verification() {
	let auth_key = get_sr25519_authentication_key(true);
	let enc_key = get_x25519_encryption_key();
	let mock_did =
		generate_mock_did_details_with_keys(did::PublicVerificationKey::from(auth_key.public()), enc_key, None, None);
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
	let enc_key = get_x25519_encryption_key();
	let att_key = get_sr25519_attestation_key(true);
	let mock_did = generate_mock_did_details_with_keys(
		did::PublicVerificationKey::from(auth_key.public()),
		enc_key,
		Some(did::PublicVerificationKey::from(att_key.public())),
		None,
	);
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
	let enc_key = get_x25519_encryption_key();
	let del_key = get_ed25519_delegation_key(true);
	let mock_did = generate_mock_did_details_with_keys(
		did::PublicVerificationKey::from(auth_key.public()),
		enc_key,
		None,
		Some(did::PublicVerificationKey::from(del_key.public())),
	);
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
	let enc_key = get_x25519_encryption_key();
	let mock_did =
		generate_mock_did_details_with_keys(did::PublicVerificationKey::from(auth_key.public()), enc_key, None, None);
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
	let enc_key = get_x25519_encryption_key();
	let mock_did =
		generate_mock_did_details_with_keys(did::PublicVerificationKey::from(auth_key.public()), enc_key, None, None);
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
	let enc_key = get_x25519_encryption_key();
	// Expected an Sr25519, given an Ed25519
	let invalid_key = get_ed25519_authentication_key(true);
	let mock_did =
		generate_mock_did_details_with_keys(did::PublicVerificationKey::from(auth_key.public()), enc_key, None, None);
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
	let enc_key = get_x25519_encryption_key();
	// Using same key type but different seed (default = false)
	let alternative_key = get_sr25519_authentication_key(false);
	let mock_did =
		generate_mock_did_details_with_keys(did::PublicVerificationKey::from(auth_key.public()), enc_key, None, None);
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
