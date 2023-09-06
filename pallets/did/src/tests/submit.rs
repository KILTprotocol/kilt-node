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

use frame_support::{assert_err, assert_noop, assert_ok};
use parity_scale_codec::Encode;
use sp_core::{ed25519, Pair};
use sp_runtime::traits::Hash;

use crate::{
	self as did,
	did_details::{DidVerificationKey, DidVerificationKeyRelationship},
	mock::*,
	mock_utils::*,
};

#[test]
fn check_call_authentication_key_successful() {
	let auth_key = get_sr25519_authentication_key(&AUTH_SEED_0);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = ACCOUNT_00;

	let mock_did = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(did.clone()));

	let call_operation = generate_test_did_call(
		DidVerificationKeyRelationship::Authentication,
		did.clone(),
		caller.clone(),
	);
	let signature = auth_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_balances(vec![(did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(did, mock_did)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_ok!(Did::submit_did_call(
				RuntimeOrigin::signed(caller),
				Box::new(call_operation.operation),
				did::DidSignature::from(signature)
			));
		});
}

#[test]
fn check_did_not_found_call_error() {
	let auth_key = get_sr25519_authentication_key(&AUTH_SEED_0);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = ACCOUNT_00;

	let call_operation = generate_test_did_call(DidVerificationKeyRelationship::Authentication, did, caller.clone());
	let signature = auth_key.sign(call_operation.encode().as_ref());

	// No DID added
	ExtBuilder::default().build(None).execute_with(|| {
		assert_noop!(
			Did::submit_did_call(
				RuntimeOrigin::signed(caller),
				Box::new(call_operation.operation),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::NotFound
		);
	});
}

#[test]
fn check_too_small_tx_counter_after_wrap_call_error() {
	let auth_key = get_sr25519_authentication_key(&AUTH_SEED_0);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = ACCOUNT_00;
	let mut mock_did =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(did.clone()));
	// After wrapping tx_counter becomes 0 again.
	mock_did.last_tx_counter = 0u64;

	let mut call_operation = generate_test_did_call(
		DidVerificationKeyRelationship::Authentication,
		did.clone(),
		caller.clone(),
	);
	call_operation.operation.tx_counter = u64::MAX;
	let signature = auth_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_balances(vec![(did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(did, mock_did)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::submit_did_call(
					RuntimeOrigin::signed(caller),
					Box::new(call_operation.operation),
					did::DidSignature::from(signature)
				),
				did::Error::<Test>::InvalidNonce
			);
		});
}

#[test]
fn check_too_small_tx_counter_call_error() {
	let auth_key = get_sr25519_authentication_key(&AUTH_SEED_0);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = ACCOUNT_00;
	let mut mock_did =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(did.clone()));
	mock_did.last_tx_counter = 1u64;

	let mut call_operation = generate_test_did_call(
		DidVerificationKeyRelationship::Authentication,
		did.clone(),
		caller.clone(),
	);
	call_operation.operation.tx_counter = mock_did.last_tx_counter - 1;
	let signature = auth_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_balances(vec![(did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(did, mock_did)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::submit_did_call(
					RuntimeOrigin::signed(caller),
					Box::new(call_operation.operation),
					did::DidSignature::from(signature)
				),
				did::Error::<Test>::InvalidNonce
			);
		});
}

#[test]
fn check_equal_tx_counter_call_error() {
	let auth_key = get_sr25519_authentication_key(&AUTH_SEED_0);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = ACCOUNT_00;
	let mock_did = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(did.clone()));

	let mut call_operation = generate_test_did_call(
		DidVerificationKeyRelationship::Authentication,
		did.clone(),
		caller.clone(),
	);
	call_operation.operation.tx_counter = mock_did.last_tx_counter;
	let signature = auth_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_balances(vec![(did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(did, mock_did)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::submit_did_call(
					RuntimeOrigin::signed(caller),
					Box::new(call_operation.operation),
					did::DidSignature::from(signature)
				),
				did::Error::<Test>::InvalidNonce
			);
		});
}

#[test]
fn check_too_large_tx_counter_call_error() {
	let auth_key = get_sr25519_authentication_key(&AUTH_SEED_0);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = ACCOUNT_00;
	let mock_did = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(did.clone()));

	let mut call_operation = generate_test_did_call(
		DidVerificationKeyRelationship::Authentication,
		did.clone(),
		caller.clone(),
	);
	call_operation.operation.tx_counter = mock_did.last_tx_counter + 2u64;
	let signature = auth_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_balances(vec![(did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(did, mock_did)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::submit_did_call(
					RuntimeOrigin::signed(caller),
					Box::new(call_operation.operation),
					did::DidSignature::from(signature)
				),
				did::Error::<Test>::InvalidNonce
			);
		});
}

#[test]
fn check_tx_block_number_too_low_error() {
	let auth_key = get_sr25519_authentication_key(&AUTH_SEED_0);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = ACCOUNT_00;

	let mock_did = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(did.clone()));

	let call_operation = generate_test_did_call(
		DidVerificationKeyRelationship::Authentication,
		did.clone(),
		caller.clone(),
	);
	let signature = auth_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_balances(vec![(did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(did, mock_did)])
		.build_and_execute_with_sanity_tests(None, || {
			// System block number 1 past the max block the operation was allowed for.
			System::set_block_number(call_operation.operation.block_number + MaxBlocksTxValidity::get() + 1);
			assert_noop!(
				Did::submit_did_call(
					RuntimeOrigin::signed(caller.clone()),
					Box::new(call_operation.operation.clone()),
					did::DidSignature::from(signature.clone())
				),
				did::Error::<Test>::TransactionExpired
			);

			// But it would work if the system would be one block earlier.
			System::set_block_number(call_operation.operation.block_number + MaxBlocksTxValidity::get());
			assert_ok!(Did::submit_did_call(
				RuntimeOrigin::signed(caller),
				Box::new(call_operation.operation),
				did::DidSignature::from(signature)
			));
		});
}

#[test]
fn check_tx_block_number_too_high_error() {
	let auth_key = get_sr25519_authentication_key(&AUTH_SEED_0);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = ACCOUNT_00;

	let mock_did = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(did.clone()));

	let mut call_operation = generate_test_did_call(
		DidVerificationKeyRelationship::Authentication,
		did.clone(),
		caller.clone(),
	);

	call_operation.operation.block_number = MaxBlocksTxValidity::get() + 100;
	let signature = auth_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_balances(vec![(did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(did, mock_did)])
		.build_and_execute_with_sanity_tests(None, || {
			// System block number is still too low, meaning that the block number used in
			// the operation was too high.
			System::set_block_number(call_operation.operation.block_number - MaxBlocksTxValidity::get() - 1);
			assert_noop!(
				Did::submit_did_call(
					RuntimeOrigin::signed(caller.clone()),
					Box::new(call_operation.operation.clone()),
					did::DidSignature::from(signature.clone())
				),
				did::Error::<Test>::TransactionExpired
			);
		});
}

#[test]
fn check_verification_key_not_present_call_error() {
	let auth_key = get_sr25519_authentication_key(&AUTH_SEED_0);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = ACCOUNT_00;
	let mock_did = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(did.clone()));

	// The operation requires the delegation key that is currently not stored for
	// the given DID.
	let call_operation = generate_test_did_call(
		DidVerificationKeyRelationship::CapabilityDelegation,
		did.clone(),
		caller.clone(),
	);
	let signature = auth_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_dids(vec![(did.clone(), mock_did)])
		.with_balances(vec![(did, DEFAULT_BALANCE)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::submit_did_call(
					RuntimeOrigin::signed(caller),
					Box::new(call_operation.operation),
					did::DidSignature::from(signature)
				),
				did::Error::<Test>::VerificationKeyNotFound
			);
		});
}

#[test]
fn check_invalid_signature_format_call_error() {
	let auth_key = get_sr25519_authentication_key(&AUTH_SEED_0);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = ACCOUNT_00;
	let alternative_auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let mock_did = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(did.clone()));

	let call_operation = generate_test_did_call(
		DidVerificationKeyRelationship::Authentication,
		did.clone(),
		caller.clone(),
	);
	let signature = alternative_auth_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_balances(vec![(did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(did, mock_did)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::submit_did_call(
					RuntimeOrigin::signed(caller),
					Box::new(call_operation.operation),
					did::DidSignature::from(signature)
				),
				did::Error::<Test>::InvalidSignatureFormat
			);
		});
}

#[test]
fn check_bad_submitter_error() {
	let auth_key = get_sr25519_authentication_key(&AUTH_SEED_0);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = ACCOUNT_00;
	let alternative_auth_key = get_sr25519_authentication_key(&AUTH_SEED_1);
	let mock_did = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(did.clone()));

	let submitter = ACCOUNT_01;

	let call_operation = generate_test_did_call(DidVerificationKeyRelationship::Authentication, did.clone(), submitter);
	let signature = alternative_auth_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_dids(vec![(did.clone(), mock_did)])
		.with_balances(vec![(did, DEFAULT_BALANCE)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::submit_did_call(
					RuntimeOrigin::signed(caller),
					Box::new(call_operation.operation),
					did::DidSignature::from(signature)
				),
				did::Error::<Test>::BadDidOrigin
			);
		});
}

#[test]
fn check_invalid_signature_call_error() {
	let auth_key = get_sr25519_authentication_key(&AUTH_SEED_0);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = ACCOUNT_00;
	let alternative_auth_key = get_sr25519_authentication_key(&AUTH_SEED_1);
	let mock_did = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(did.clone()));

	let call_operation = generate_test_did_call(
		DidVerificationKeyRelationship::Authentication,
		did.clone(),
		caller.clone(),
	);
	let signature = alternative_auth_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_balances(vec![(did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(did, mock_did)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::submit_did_call(
					RuntimeOrigin::signed(caller),
					Box::new(call_operation.operation),
					did::DidSignature::from(signature)
				),
				did::Error::<Test>::InvalidSignature
			);
		});
}

#[test]
fn check_call_attestation_key_successful() {
	let auth_key = get_sr25519_authentication_key(&AUTH_SEED_0);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = ACCOUNT_00;
	let attestation_key = get_ed25519_attestation_key(&ATT_SEED_0);

	let mut mock_did =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(did.clone()));
	assert_ok!(mock_did.update_attestation_key(DidVerificationKey::from(attestation_key.public()), 0));

	let call_operation = generate_test_did_call(
		DidVerificationKeyRelationship::AssertionMethod,
		did.clone(),
		caller.clone(),
	);
	let signature = attestation_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_balances(vec![(did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(did, mock_did)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_ok!(Did::submit_did_call(
				RuntimeOrigin::signed(caller),
				Box::new(call_operation.operation),
				did::DidSignature::from(signature)
			));
		});
}

#[test]
fn check_call_attestation_key_error() {
	let auth_key = get_sr25519_authentication_key(&AUTH_SEED_0);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = ACCOUNT_00;
	let attestation_key = get_ed25519_attestation_key(&ATT_SEED_0);

	let mut mock_did =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(did.clone()));
	assert_ok!(mock_did.update_attestation_key(DidVerificationKey::from(attestation_key.public()), 0));

	let call_operation = generate_test_did_call(
		DidVerificationKeyRelationship::AssertionMethod,
		did.clone(),
		caller.clone(),
	);
	let signature = attestation_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_balances(vec![(did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(did.clone(), mock_did)])
		.with_ctypes(vec![(
			<Test as frame_system::Config>::Hashing::hash(&get_attestation_key_test_input()[..]),
			did,
		)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_err!(
				Did::submit_did_call(
					RuntimeOrigin::signed(caller),
					Box::new(call_operation.operation),
					did::DidSignature::from(signature)
				),
				ctype::Error::<Test>::AlreadyExists
			);
		});
}

#[test]
fn check_call_delegation_key_successful() {
	let auth_key = get_sr25519_authentication_key(&AUTH_SEED_0);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = ACCOUNT_00;
	let delegation_key = get_ed25519_delegation_key(&DEL_SEED_0);

	let mut mock_did =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(caller.clone()));
	assert_ok!(mock_did.update_delegation_key(DidVerificationKey::from(delegation_key.public()), 0));

	let call_operation = generate_test_did_call(
		DidVerificationKeyRelationship::CapabilityDelegation,
		did.clone(),
		caller.clone(),
	);
	let signature = delegation_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_dids(vec![(did, mock_did)])
		.with_balances(vec![(caller.clone(), DEFAULT_BALANCE)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_ok!(Did::submit_did_call(
				RuntimeOrigin::signed(caller),
				Box::new(call_operation.operation),
				did::DidSignature::from(signature)
			));
		});
}

#[test]
fn check_call_delegation_key_error() {
	let auth_key = get_sr25519_authentication_key(&AUTH_SEED_0);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = ACCOUNT_00;
	let delegation_key = get_ed25519_delegation_key(&ATT_SEED_0);

	let mut mock_did =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(did.clone()));
	assert_ok!(mock_did.update_delegation_key(DidVerificationKey::from(delegation_key.public()), 0));

	let call_operation = generate_test_did_call(
		DidVerificationKeyRelationship::CapabilityDelegation,
		did.clone(),
		caller.clone(),
	);
	let signature = delegation_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_balances(vec![(did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(did.clone(), mock_did)])
		.with_ctypes(vec![(
			<Test as frame_system::Config>::Hashing::hash(&get_delegation_key_test_input()[..]),
			did,
		)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_err!(
				Did::submit_did_call(
					RuntimeOrigin::signed(caller),
					Box::new(call_operation.operation),
					did::DidSignature::from(signature)
				),
				ctype::Error::<Test>::AlreadyExists
			);
		});
}

#[test]
fn check_call_authentication_key_error() {
	let auth_key = get_sr25519_authentication_key(&AUTH_SEED_0);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = ACCOUNT_00;

	let mock_did = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(did.clone()));

	let call_operation = generate_test_did_call(
		DidVerificationKeyRelationship::Authentication,
		did.clone(),
		caller.clone(),
	);
	let signature = auth_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_dids(vec![(did.clone(), mock_did)])
		.with_balances(vec![(did.clone(), DEFAULT_BALANCE)])
		.with_ctypes(vec![(
			<Test as frame_system::Config>::Hashing::hash(&get_authentication_key_test_input()[..]),
			did,
		)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_err!(
				Did::submit_did_call(
					RuntimeOrigin::signed(caller),
					Box::new(call_operation.operation),
					did::DidSignature::from(signature)
				),
				ctype::Error::<Test>::AlreadyExists
			);
		});
}

#[test]
fn check_null_key_error() {
	let auth_key = get_sr25519_authentication_key(&AUTH_SEED_0);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = ACCOUNT_00;

	// CapabilityInvocation is not supported at the moment, so it should return no
	// key and hence the operation fail.
	let call_operation = generate_test_did_call(
		DidVerificationKeyRelationship::CapabilityInvocation,
		did,
		caller.clone(),
	);
	let signature = ed25519::Signature::from_raw([0u8; 64]);

	ExtBuilder::default().build(None).execute_with(|| {
		assert_noop!(
			Did::submit_did_call(
				RuntimeOrigin::signed(caller),
				Box::new(call_operation.operation),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::UnsupportedDidAuthorizationCall
		);
	});
}
