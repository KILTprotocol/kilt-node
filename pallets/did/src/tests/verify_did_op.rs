// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2023 BOTLabs GmbH

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
use parity_scale_codec::Encode;
use sp_core::Pair;

use crate::{
	self as did,
	did_details::{DidVerificationKey, DidVerificationKeyRelationship},
	mock::*,
	mock_utils::*,
};

#[test]
fn check_authentication_successful_operation_verification() {
	let auth_key = get_sr25519_authentication_key(&AUTH_SEED_0);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());

	let mock_did = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(did.clone()));

	let call_operation =
		generate_test_did_call(DidVerificationKeyRelationship::Authentication, did.clone(), ACCOUNT_00);
	let signature = auth_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_dids(vec![(did.clone(), mock_did.clone())])
		.with_balances(vec![(did, DEFAULT_BALANCE)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_ok!(Did::verify_did_operation_signature_and_increase_nonce(
				&call_operation,
				&did::DidSignature::from(signature)
			));
			// Verify that the DID tx counter has increased
			let did_details = Did::get_did(&call_operation.operation.did).expect("DID should be present on chain.");
			assert_eq!(did_details.last_tx_counter, mock_did.last_tx_counter + 1u64);
		});
}

#[test]
fn check_attestation_successful_operation_verification() {
	let auth_key = get_sr25519_authentication_key(&AUTH_SEED_0);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let attestation_key = get_ed25519_attestation_key(&ATT_SEED_0);

	let mut mock_did =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(did.clone()));
	assert_ok!(mock_did.update_attestation_key(DidVerificationKey::from(attestation_key.public()), 0));

	let call_operation =
		generate_test_did_call(DidVerificationKeyRelationship::AssertionMethod, did.clone(), ACCOUNT_00);
	let signature = attestation_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_dids(vec![(did.clone(), mock_did.clone())])
		.with_balances(vec![(did, DEFAULT_BALANCE)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_ok!(Did::verify_did_operation_signature_and_increase_nonce(
				&call_operation,
				&did::DidSignature::from(signature)
			));
			// Verify that the DID tx counter has increased
			let did_details = Did::get_did(&call_operation.operation.did).expect("DID should be present on chain.");
			assert_eq!(did_details.last_tx_counter, mock_did.last_tx_counter + 1u64);
		});
}

#[test]
fn check_delegation_successful_operation_verification() {
	let auth_key = get_sr25519_authentication_key(&AUTH_SEED_0);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let delegation_key = get_ecdsa_delegation_key(&DEL_SEED_0);

	let mut mock_did =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(did.clone()));
	assert_ok!(mock_did.update_delegation_key(DidVerificationKey::from(delegation_key.public()), 0));

	let call_operation = generate_test_did_call(
		DidVerificationKeyRelationship::CapabilityDelegation,
		did.clone(),
		ACCOUNT_00,
	);
	let signature = delegation_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_balances(vec![(did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(did, mock_did.clone())])
		.build_and_execute_with_sanity_tests(None, || {
			assert_ok!(Did::verify_did_operation_signature_and_increase_nonce(
				&call_operation,
				&did::DidSignature::from(signature)
			));
			// Verify that the DID tx counter has increased
			let did_details = Did::get_did(&call_operation.operation.did).expect("DID should be present on chain.");
			assert_eq!(did_details.last_tx_counter, mock_did.last_tx_counter + 1u64);
		});
}

#[test]
fn check_did_not_present_operation_verification() {
	let auth_key = get_sr25519_authentication_key(&AUTH_SEED_0);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());

	let call_operation = generate_test_did_call(DidVerificationKeyRelationship::CapabilityDelegation, did, ACCOUNT_00);
	let signature = auth_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default().build(None).execute_with(|| {
		assert_noop!(
			Did::verify_did_operation_signature_and_increase_nonce(
				&call_operation,
				&did::DidSignature::from(signature)
			),
			did::errors::StorageError::NotFound(did::errors::NotFoundKind::Did)
		);
	});
}

#[test]
fn check_tx_counter_wrap_operation_verification() {
	let auth_key = get_sr25519_authentication_key(&AUTH_SEED_0);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());

	let mut mock_did =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(did.clone()));
	mock_did.last_tx_counter = u64::MAX;

	let mut call_operation =
		generate_test_did_call(DidVerificationKeyRelationship::Authentication, did.clone(), ACCOUNT_00);
	// Counter should wrap, so 0 is now expected.
	call_operation.operation.tx_counter = 0u64;
	let signature = auth_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_dids(vec![(did.clone(), mock_did)])
		.with_balances(vec![(did, DEFAULT_BALANCE)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_ok!(Did::verify_did_operation_signature_and_increase_nonce(
				&call_operation,
				&did::DidSignature::from(signature)
			));
			// Verify that the DID tx counter has wrapped around
			let did_details = Did::get_did(&call_operation.operation.did).expect("DID should be present on chain.");
			assert_eq!(did_details.last_tx_counter, 0u64);
		});
}

#[test]
fn check_smaller_counter_operation_verification() {
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let did = get_did_identifier_from_ed25519_key(auth_key.public());

	let mut mock_did =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(did.clone()));
	mock_did.last_tx_counter = 1;

	let mut call_operation = generate_test_did_call(
		DidVerificationKeyRelationship::CapabilityDelegation,
		did.clone(),
		ACCOUNT_00,
	);
	call_operation.operation.tx_counter = 0u64;
	let signature = auth_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_dids(vec![(did.clone(), mock_did)])
		.with_balances(vec![(did, DEFAULT_BALANCE)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::verify_did_operation_signature_and_increase_nonce(
					&call_operation,
					&did::DidSignature::from(signature)
				),
				did::errors::DidError::Signature(did::errors::SignatureError::InvalidNonce)
			);
		});
}

#[test]
fn check_equal_counter_operation_verification() {
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let did = get_did_identifier_from_ed25519_key(auth_key.public());

	let mock_did = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(did.clone()));

	let mut call_operation = generate_test_did_call(
		DidVerificationKeyRelationship::CapabilityDelegation,
		did.clone(),
		ACCOUNT_00,
	);
	call_operation.operation.tx_counter = mock_did.last_tx_counter;
	let signature = auth_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_balances(vec![(did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(did, mock_did)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::verify_did_operation_signature_and_increase_nonce(
					&call_operation,
					&did::DidSignature::from(signature)
				),
				did::errors::DidError::Signature(did::errors::SignatureError::InvalidNonce)
			);
		});
}

#[test]
fn check_too_large_counter_operation_verification() {
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let did = get_did_identifier_from_ed25519_key(auth_key.public());

	let mock_did = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(did.clone()));

	let mut call_operation = generate_test_did_call(
		DidVerificationKeyRelationship::CapabilityDelegation,
		did.clone(),
		ACCOUNT_00,
	);
	call_operation.operation.tx_counter = mock_did.last_tx_counter + 2;
	let signature = auth_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_dids(vec![(did.clone(), mock_did)])
		.with_balances(vec![(did, DEFAULT_BALANCE)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::verify_did_operation_signature_and_increase_nonce(
					&call_operation,
					&did::DidSignature::from(signature)
				),
				did::errors::DidError::Signature(did::errors::SignatureError::InvalidNonce)
			);
		});
}

#[test]
fn check_verification_key_not_present_operation_verification() {
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let did = get_did_identifier_from_ed25519_key(auth_key.public());

	let mock_did = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(did.clone()));

	let call_operation =
		generate_test_did_call(DidVerificationKeyRelationship::AssertionMethod, did.clone(), ACCOUNT_00);
	let signature = auth_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_balances(vec![(did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(did, mock_did)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::verify_did_operation_signature_and_increase_nonce(
					&call_operation,
					&did::DidSignature::from(signature)
				),
				did::errors::DidError::Storage(did::errors::StorageError::NotFound(did::errors::NotFoundKind::Key(
					did::errors::KeyType::AssertionMethod
				)))
			);
		});
}

#[test]
fn check_invalid_signature_format_operation_verification() {
	let auth_key = get_sr25519_authentication_key(&AUTH_SEED_0);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	// Expected an Sr25519, given an Ed25519
	let invalid_key = get_ed25519_authentication_key(&AUTH_SEED_0);

	let mock_did = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(did.clone()));

	let call_operation =
		generate_test_did_call(DidVerificationKeyRelationship::Authentication, did.clone(), ACCOUNT_00);
	let signature = invalid_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_balances(vec![(did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(did, mock_did)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::verify_did_operation_signature_and_increase_nonce(
					&call_operation,
					&did::DidSignature::from(signature)
				),
				did::errors::DidError::Signature(did::errors::SignatureError::InvalidFormat)
			);
		});
}

#[test]
fn check_invalid_signature_operation_verification() {
	let auth_key = get_sr25519_authentication_key(&AUTH_SEED_0);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	// Using same key type but different seed (default = false)
	let alternative_key = get_sr25519_authentication_key(&AUTH_SEED_1);

	let mock_did = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(did.clone()));

	let call_operation =
		generate_test_did_call(DidVerificationKeyRelationship::Authentication, did.clone(), ACCOUNT_00);
	let signature = alternative_key.sign(&call_operation.encode());

	ExtBuilder::default()
		.with_balances(vec![(did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(did, mock_did)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::verify_did_operation_signature_and_increase_nonce(
					&call_operation,
					&did::DidSignature::from(signature)
				),
				did::errors::DidError::Signature(did::errors::SignatureError::InvalidData)
			);
		});
}
