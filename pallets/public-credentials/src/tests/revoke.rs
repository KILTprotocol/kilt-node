// KILT Blockchain – <https://kilt.io>
// Copyright (C) 2025, KILT Foundation

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

// If you feel like getting in touch with us, you can do so at <hello@kilt.io>

use frame_support::{assert_noop, assert_ok, traits::Get};

use ctype::mock::get_ctype_hash;
use kilt_support::mock::mock_origin::DoubleOrigin;

use crate::{mock::*, Config, CredentialIdOf, Credentials, Error};

#[test]
fn revoke_successful() {
	let attester = sr25519_did_from_seed(&ALICE_SEED);
	let subject_id: <Test as Config>::SubjectId = SUBJECT_ID_00;
	let ctype_hash_1 = get_ctype_hash::<Test>(true);
	let new_credential =
		generate_base_credential_entry::<Test>(ACCOUNT_00, 0, attester.clone(), Some(ctype_hash_1), None);
	let credential_id: CredentialIdOf<Test> = CredentialIdOf::<Test>::default();
	let deposit: Balance = <Test as Config>::Deposit::get();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, deposit + MIN_BALANCE)])
		.with_public_credentials(vec![(subject_id, credential_id, new_credential)])
		.with_ctypes(vec![(ctype_hash_1, attester.clone())])
		.build_and_execute_with_sanity_tests(|| {
			assert_ok!(PublicCredentials::revoke(
				DoubleOrigin(ACCOUNT_00, attester.clone()).into(),
				credential_id,
				None,
			));

			let stored_public_credential_details = Credentials::<Test>::get(subject_id, credential_id)
				.expect("Public credential details should be present on chain.");

			// Test this pallet logic
			assert!(stored_public_credential_details.revoked);

			// Revoking the same credential does nothing
			assert_ok!(PublicCredentials::revoke(
				DoubleOrigin(ACCOUNT_00, attester.clone()).into(),
				credential_id,
				None,
			));

			let stored_public_credential_details_2 = Credentials::<Test>::get(subject_id, credential_id)
				.expect("Public credential details should be present on chain.");

			assert_eq!(stored_public_credential_details, stored_public_credential_details_2);
		});
}

#[test]
fn revoke_same_attester_wrong_ac() {
	let attester = sr25519_did_from_seed(&ALICE_SEED);
	let wrong_submitter = sr25519_did_from_seed(&BOB_SEED);
	let subject_id: <Test as Config>::SubjectId = SUBJECT_ID_00;
	let ctype_hash_1 = get_ctype_hash::<Test>(true);
	let mut new_credential =
		generate_base_credential_entry::<Test>(ACCOUNT_00, 0, attester.clone(), Some(ctype_hash_1), None);
	new_credential.authorization_id = Some(attester.clone());
	let credential_id: CredentialIdOf<Test> = CredentialIdOf::<Test>::default();
	let deposit: Balance = <Test as Config>::Deposit::get();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, deposit + MIN_BALANCE)])
		.with_public_credentials(vec![(subject_id, credential_id, new_credential)])
		.with_ctypes(vec![(ctype_hash_1, attester.clone())])
		.build_and_execute_with_sanity_tests(|| {
			assert_ok!(PublicCredentials::revoke(
				DoubleOrigin(ACCOUNT_00, attester.clone()).into(),
				credential_id,
				Some(MockAccessControl(wrong_submitter))
			));

			let stored_public_credential_details = Credentials::<Test>::get(subject_id, credential_id)
				.expect("Public credential details should be present on chain.");

			// Test this pallet logic
			assert!(stored_public_credential_details.revoked);
		});
}

#[test]
fn revoke_unauthorized() {
	let attester = sr25519_did_from_seed(&ALICE_SEED);
	let ctype_hash_1 = get_ctype_hash::<Test>(true);
	let wrong_submitter = sr25519_did_from_seed(&BOB_SEED);
	let subject_id: <Test as Config>::SubjectId = SUBJECT_ID_00;
	let mut new_credential =
		generate_base_credential_entry::<Test>(ACCOUNT_00, 0, attester.clone(), Some(ctype_hash_1), None);
	new_credential.authorization_id = Some(attester.clone());
	let credential_id: CredentialIdOf<Test> = CredentialIdOf::<Test>::default();
	let deposit: Balance = <Test as Config>::Deposit::get();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, deposit + MIN_BALANCE)])
		.with_public_credentials(vec![(subject_id, credential_id, new_credential)])
		.with_ctypes(vec![(ctype_hash_1, attester.clone())])
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				PublicCredentials::revoke(
					DoubleOrigin(ACCOUNT_00, wrong_submitter).into(),
					credential_id,
					Some(MockAccessControl(attester))
				),
				Error::<Test>::NotAuthorized
			);
		});
}

#[test]
fn revoke_ac_not_found() {
	let attester = sr25519_did_from_seed(&ALICE_SEED);
	let wrong_submitter = sr25519_did_from_seed(&BOB_SEED);
	let ctype_hash_1 = get_ctype_hash::<Test>(true);
	let subject_id: <Test as Config>::SubjectId = SUBJECT_ID_00;
	let mut new_credential =
		generate_base_credential_entry::<Test>(ACCOUNT_00, 0, attester.clone(), Some(ctype_hash_1), None);
	new_credential.authorization_id = Some(attester.clone());
	let credential_id: CredentialIdOf<Test> = CredentialIdOf::<Test>::default();
	let deposit: Balance = <Test as Config>::Deposit::get();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, deposit + MIN_BALANCE)])
		.with_public_credentials(vec![(subject_id, credential_id, new_credential)])
		.with_ctypes(vec![(ctype_hash_1, attester)])
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				PublicCredentials::revoke(
					DoubleOrigin(ACCOUNT_00, wrong_submitter.clone()).into(),
					credential_id,
					Some(MockAccessControl(wrong_submitter))
				),
				Error::<Test>::NotAuthorized
			);
		});
}

#[test]
fn revoke_credential_not_found() {
	let attester = sr25519_did_from_seed(&ALICE_SEED);
	let credential_id: CredentialIdOf<Test> = CredentialIdOf::<Test>::default();
	let deposit: Balance = <Test as Config>::Deposit::get();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, deposit)])
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				PublicCredentials::revoke(DoubleOrigin(ACCOUNT_00, attester.clone()).into(), credential_id, None,),
				Error::<Test>::NotFound
			);
		});
}

// unrevoke

#[test]
fn unrevoke_successful() {
	let attester = sr25519_did_from_seed(&ALICE_SEED);
	let subject_id: <Test as Config>::SubjectId = SUBJECT_ID_00;
	let ctype_hash_1 = get_ctype_hash::<Test>(true);
	let mut new_credential =
		generate_base_credential_entry::<Test>(ACCOUNT_00, 0, attester.clone(), Some(ctype_hash_1), None);
	new_credential.revoked = true;
	let credential_id: CredentialIdOf<Test> = CredentialIdOf::<Test>::default();
	let deposit: Balance = <Test as Config>::Deposit::get();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, deposit + MIN_BALANCE)])
		.with_public_credentials(vec![(subject_id, credential_id, new_credential)])
		.with_ctypes(vec![(ctype_hash_1, attester.clone())])
		.build_and_execute_with_sanity_tests(|| {
			assert_ok!(PublicCredentials::unrevoke(
				DoubleOrigin(ACCOUNT_00, attester.clone()).into(),
				credential_id,
				None,
			));

			let stored_public_credential_details = Credentials::<Test>::get(subject_id, credential_id)
				.expect("Public credential details should be present on chain.");

			// Test this pallet logic
			assert!(!stored_public_credential_details.revoked);

			// Unrevoking the same credential does nothing
			assert_ok!(PublicCredentials::unrevoke(
				DoubleOrigin(ACCOUNT_00, attester.clone()).into(),
				credential_id,
				None,
			));

			let stored_public_credential_details_2 = Credentials::<Test>::get(subject_id, credential_id)
				.expect("Public credential details should be present on chain.");

			assert_eq!(stored_public_credential_details, stored_public_credential_details_2);
		});
}

#[test]
fn unrevoke_same_attester_wrong_ac() {
	let attester = sr25519_did_from_seed(&ALICE_SEED);
	let wrong_submitter = sr25519_did_from_seed(&BOB_SEED);
	let ctype_hash_1 = get_ctype_hash::<Test>(true);
	let subject_id: <Test as Config>::SubjectId = SUBJECT_ID_00;
	let mut new_credential =
		generate_base_credential_entry::<Test>(ACCOUNT_00, 0, attester.clone(), Some(ctype_hash_1), None);
	new_credential.revoked = true;
	new_credential.authorization_id = Some(attester.clone());
	let credential_id: CredentialIdOf<Test> = CredentialIdOf::<Test>::default();
	let deposit: Balance = <Test as Config>::Deposit::get();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, deposit + MIN_BALANCE)])
		.with_public_credentials(vec![(subject_id, credential_id, new_credential)])
		.with_ctypes(vec![(ctype_hash_1, attester.clone())])
		.build_and_execute_with_sanity_tests(|| {
			assert_ok!(PublicCredentials::unrevoke(
				DoubleOrigin(ACCOUNT_00, attester.clone()).into(),
				credential_id,
				Some(MockAccessControl(wrong_submitter))
			));

			let stored_public_credential_details = Credentials::<Test>::get(subject_id, credential_id)
				.expect("Public credential details should be present on chain.");

			// Test this pallet logic
			assert!(!stored_public_credential_details.revoked);
		});
}

#[test]
fn unrevoke_unauthorized() {
	let attester = sr25519_did_from_seed(&ALICE_SEED);
	let wrong_submitter = sr25519_did_from_seed(&BOB_SEED);
	let ctype_hash_1 = get_ctype_hash::<Test>(true);
	let subject_id: <Test as Config>::SubjectId = SUBJECT_ID_00;
	let mut new_credential =
		generate_base_credential_entry::<Test>(ACCOUNT_00, 0, attester.clone(), Some(ctype_hash_1), None);
	new_credential.revoked = true;
	new_credential.authorization_id = Some(attester.clone());
	let credential_id: CredentialIdOf<Test> = CredentialIdOf::<Test>::default();
	let deposit: Balance = <Test as Config>::Deposit::get();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, deposit + MIN_BALANCE)])
		.with_ctypes(vec![(ctype_hash_1, attester.clone())])
		.with_public_credentials(vec![(subject_id, credential_id, new_credential)])
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				PublicCredentials::unrevoke(
					DoubleOrigin(ACCOUNT_00, wrong_submitter).into(),
					credential_id,
					Some(MockAccessControl(attester))
				),
				Error::<Test>::NotAuthorized
			);
		});
}

#[test]
fn unrevoke_ac_not_found() {
	let attester = sr25519_did_from_seed(&ALICE_SEED);
	let wrong_submitter = sr25519_did_from_seed(&BOB_SEED);
	let ctype_hash_1 = get_ctype_hash::<Test>(true);
	let subject_id: <Test as Config>::SubjectId = SUBJECT_ID_00;
	let mut new_credential =
		generate_base_credential_entry::<Test>(ACCOUNT_00, 0, attester.clone(), Some(ctype_hash_1), None);
	new_credential.revoked = true;
	new_credential.authorization_id = Some(attester.clone());
	let credential_id: CredentialIdOf<Test> = CredentialIdOf::<Test>::default();
	let deposit: Balance = <Test as Config>::Deposit::get();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, deposit + MIN_BALANCE)])
		.with_public_credentials(vec![(subject_id, credential_id, new_credential)])
		.with_ctypes(vec![(ctype_hash_1, attester)])
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				PublicCredentials::unrevoke(
					DoubleOrigin(ACCOUNT_00, wrong_submitter.clone()).into(),
					credential_id,
					Some(MockAccessControl(wrong_submitter))
				),
				Error::<Test>::NotAuthorized
			);
		});
}

#[test]
fn unrevoke_credential_not_found() {
	let attester = sr25519_did_from_seed(&ALICE_SEED);
	let credential_id: CredentialIdOf<Test> = CredentialIdOf::<Test>::default();
	let deposit: Balance = <Test as Config>::Deposit::get();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, deposit)])
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				PublicCredentials::unrevoke(DoubleOrigin(ACCOUNT_00, attester.clone()).into(), credential_id, None,),
				Error::<Test>::NotFound
			);
		});
}
