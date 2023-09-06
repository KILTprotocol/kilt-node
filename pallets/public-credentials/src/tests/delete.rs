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

use frame_support::{
	assert_noop, assert_ok,
	traits::{fungible::InspectHold, Get},
};
use sp_runtime::traits::Zero;

use ctype::mock::get_ctype_hash;
use kilt_support::mock::mock_origin::DoubleOrigin;

use crate::{mock::*, Config, CredentialIdOf, CredentialSubjects, Credentials, Error, HoldReason};

#[test]
fn remove_successful() {
	let attester = sr25519_did_from_seed(&ALICE_SEED);
	let subject_id: <Test as Config>::SubjectId = SUBJECT_ID_00;
	let new_credential = generate_base_credential_entry::<Test>(ACCOUNT_00, 0, attester.clone(), None, None);
	let credential_id: CredentialIdOf<Test> = CredentialIdOf::<Test>::default();
	let deposit: Balance = <Test as Config>::Deposit::get();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, deposit + MIN_BALANCE)])
		.with_public_credentials(vec![(subject_id, credential_id, new_credential)])
		.build_and_execute_with_sanity_tests(|| {
			assert_ok!(PublicCredentials::remove(
				DoubleOrigin(ACCOUNT_00, attester.clone()).into(),
				credential_id,
				None,
			));

			// Test this pallet logic
			assert!(Credentials::<Test>::get(subject_id, credential_id).is_none());
			assert!(CredentialSubjects::<Test>::get(credential_id).is_none());

			// Check deposit release logic
			assert!(Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00).is_zero());

			// Removing the same credential again will fail
			assert_noop!(
				PublicCredentials::remove(DoubleOrigin(ACCOUNT_00, attester.clone()).into(), credential_id, None,),
				Error::<Test>::NotFound
			);
		});
}

#[test]
fn remove_same_attester_wrong_ac() {
	let attester = sr25519_did_from_seed(&ALICE_SEED);
	let wrong_submitter = sr25519_did_from_seed(&BOB_SEED);
	let subject_id: <Test as Config>::SubjectId = SUBJECT_ID_00;
	let mut new_credential = generate_base_credential_entry::<Test>(ACCOUNT_00, 0, attester.clone(), None, None);
	new_credential.authorization_id = Some(attester.clone());
	let credential_id: CredentialIdOf<Test> = CredentialIdOf::<Test>::default();
	let deposit: Balance = <Test as Config>::Deposit::get();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, deposit + MIN_BALANCE)])
		.with_public_credentials(vec![(subject_id, credential_id, new_credential)])
		.build_and_execute_with_sanity_tests(|| {
			assert_ok!(PublicCredentials::remove(
				DoubleOrigin(ACCOUNT_00, attester.clone()).into(),
				credential_id,
				Some(MockAccessControl(wrong_submitter))
			));

			// Test this pallet logic
			assert!(Credentials::<Test>::get(subject_id, credential_id).is_none());
			assert!(CredentialSubjects::<Test>::get(credential_id).is_none());
		});
}

#[test]
fn remove_unauthorized() {
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
			assert_noop!(
				PublicCredentials::remove(
					DoubleOrigin(ACCOUNT_00, wrong_submitter).into(),
					credential_id,
					Some(MockAccessControl(attester))
				),
				Error::<Test>::NotAuthorized
			);
		});
}

#[test]
fn remove_ac_not_found() {
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
				PublicCredentials::remove(
					DoubleOrigin(ACCOUNT_00, wrong_submitter.clone()).into(),
					credential_id,
					Some(MockAccessControl(wrong_submitter))
				),
				Error::<Test>::NotAuthorized
			);
		});
}

#[test]
fn remove_credential_not_found() {
	let attester = sr25519_did_from_seed(&ALICE_SEED);
	let credential_id: CredentialIdOf<Test> = CredentialIdOf::<Test>::default();
	let deposit: Balance = <Test as Config>::Deposit::get();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, deposit)])
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				PublicCredentials::remove(DoubleOrigin(ACCOUNT_00, attester.clone()).into(), credential_id, None,),
				Error::<Test>::NotFound
			);
		});
}
