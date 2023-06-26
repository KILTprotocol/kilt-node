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
use kilt_support::{mock::mock_origin::DoubleOrigin, Deposit};

use crate::{
	mock::*, Config, CredentialIdOf, CredentialSubjects, Credentials, Error, HoldReason, InputClaimsContentOf,
};

// add

#[test]
fn add_successful_without_authorization() {
	let attester = sr25519_did_from_seed(&ALICE_SEED);
	let subject_id = SUBJECT_ID_00;
	let ctype_hash_1 = get_ctype_hash::<Test>(true);
	let ctype_hash_2 = get_ctype_hash::<Test>(false);
	let new_credential_1 = generate_base_public_credential_creation_op::<Test>(
		subject_id.into(),
		ctype_hash_1,
		InputClaimsContentOf::<Test>::default(),
	);
	let credential_id_1: CredentialIdOf<Test> = generate_credential_id::<Test>(&new_credential_1, &attester);
	let new_credential_2 = generate_base_public_credential_creation_op::<Test>(
		subject_id.into(),
		ctype_hash_2,
		InputClaimsContentOf::<Test>::default(),
	);
	let credential_id_2: CredentialIdOf<Test> = generate_credential_id::<Test>(&new_credential_2, &attester);
	let deposit: Balance = <Test as Config>::Deposit::get();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, (deposit) * 2 + MIN_BALANCE)])
		.with_ctypes(vec![(ctype_hash_1, attester.clone()), (ctype_hash_2, attester.clone())])
		.build_and_execute_with_sanity_tests(|| {
			// Check for 0 reserved deposit
			assert!(Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00).is_zero());

			assert_ok!(PublicCredentials::add(
				DoubleOrigin(ACCOUNT_00, attester.clone()).into(),
				Box::new(new_credential_1.clone())
			));
			let stored_public_credential_details = Credentials::<Test>::get(subject_id, credential_id_1)
				.expect("Public credential details should be present on chain.");

			// Test this pallet logic
			assert_eq!(stored_public_credential_details.attester, attester);
			assert!(!stored_public_credential_details.revoked);
			assert_eq!(stored_public_credential_details.block_number, 0);
			assert_eq!(stored_public_credential_details.ctype_hash, ctype_hash_1);
			assert_eq!(stored_public_credential_details.authorization_id, None);
			assert_eq!(CredentialSubjects::<Test>::get(credential_id_1), Some(subject_id));

			// Check deposit reservation logic
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				deposit
			);

			// Re-issuing the same credential will fail
			assert_noop!(
				PublicCredentials::add(
					DoubleOrigin(ACCOUNT_00, attester.clone()).into(),
					Box::new(new_credential_1.clone())
				),
				Error::<Test>::AlreadyAttested
			);

			// Check deposit has not changed
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				deposit
			);

			System::set_block_number(1);

			// Issuing a completely new credential will work
			assert_ok!(PublicCredentials::add(
				DoubleOrigin(ACCOUNT_00, attester.clone()).into(),
				Box::new(new_credential_2.clone())
			));

			let stored_public_credential_details = Credentials::<Test>::get(subject_id, credential_id_2)
				.expect("Public credential #2 details should be present on chain.");

			// Test this pallet logic
			assert_eq!(stored_public_credential_details.attester, attester);
			assert!(!stored_public_credential_details.revoked);
			assert_eq!(stored_public_credential_details.block_number, 1);
			assert_eq!(stored_public_credential_details.ctype_hash, ctype_hash_2);
			assert_eq!(stored_public_credential_details.authorization_id, None);
			assert_eq!(CredentialSubjects::<Test>::get(credential_id_2), Some(subject_id));

			// Deposit is 2x now
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				2 * deposit
			);
		});
}

#[test]
fn add_successful_with_authorization() {
	let attester = sr25519_did_from_seed(&ALICE_SEED);
	let subject_id = SUBJECT_ID_00;
	let ctype_hash = get_ctype_hash::<Test>(true);
	let mut new_credential = generate_base_public_credential_creation_op::<Test>(
		subject_id.into(),
		ctype_hash,
		InputClaimsContentOf::<Test>::default(),
	);
	new_credential.authorization = Some(MockAccessControl(attester.clone()));
	let credential_id: CredentialIdOf<Test> = generate_credential_id::<Test>(&new_credential, &attester);
	let deposit: Balance = <Test as Config>::Deposit::get();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, deposit + MIN_BALANCE)])
		.with_ctypes(vec![(ctype_hash, attester.clone())])
		.build_and_execute_with_sanity_tests(|| {
			assert_ok!(PublicCredentials::add(
				DoubleOrigin(ACCOUNT_00, attester.clone()).into(),
				Box::new(new_credential.clone())
			));
			let stored_public_credential_details = Credentials::<Test>::get(subject_id, credential_id)
				.expect("Public credential details should be present on chain.");

			// Test this pallet logic
			assert_eq!(stored_public_credential_details.attester, attester);
			assert!(!stored_public_credential_details.revoked);
			assert_eq!(stored_public_credential_details.block_number, 0);
			assert_eq!(stored_public_credential_details.ctype_hash, ctype_hash);
			assert_eq!(stored_public_credential_details.authorization_id, Some(attester));
			assert_eq!(CredentialSubjects::<Test>::get(credential_id), Some(subject_id));
		});
}

#[test]
fn add_unauthorized() {
	let attester = sr25519_did_from_seed(&ALICE_SEED);
	let wrong_attester = sr25519_did_from_seed(&BOB_SEED);
	let subject_id = SUBJECT_ID_00;
	let ctype_hash = get_ctype_hash::<Test>(true);
	let mut new_credential = generate_base_public_credential_creation_op::<Test>(
		subject_id.into(),
		ctype_hash,
		InputClaimsContentOf::<Test>::default(),
	);
	new_credential.authorization = Some(MockAccessControl(wrong_attester));
	let deposit: Balance = <Test as Config>::Deposit::get();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, deposit)])
		.with_ctypes(vec![(ctype_hash, attester.clone())])
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				PublicCredentials::add(
					DoubleOrigin(ACCOUNT_00, attester.clone()).into(),
					Box::new(new_credential.clone())
				),
				Error::<Test>::NotAuthorized
			);
		});
}

#[test]
fn add_ctype_not_existing() {
	let attester = sr25519_did_from_seed(&ALICE_SEED);
	let subject_id = SUBJECT_ID_00;
	let ctype_hash = get_ctype_hash::<Test>(true);
	let new_credential = generate_base_public_credential_creation_op::<Test>(
		subject_id.into(),
		ctype_hash,
		InputClaimsContentOf::<Test>::default(),
	);
	let deposit: Balance = <Test as Config>::Deposit::get();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, deposit)])
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				PublicCredentials::add(
					DoubleOrigin(ACCOUNT_00, attester.clone()).into(),
					Box::new(new_credential)
				),
				ctype::Error::<Test>::NotFound
			);
		});
}

#[test]
fn add_invalid_subject() {
	let attester = sr25519_did_from_seed(&ALICE_SEED);
	let subject_id = INVALID_SUBJECT_ID;
	let ctype_hash = get_ctype_hash::<Test>(true);
	let new_credential = generate_base_public_credential_creation_op::<Test>(
		subject_id.into(),
		ctype_hash,
		InputClaimsContentOf::<Test>::default(),
	);
	let deposit: Balance = <Test as Config>::Deposit::get();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, deposit)])
		.with_ctypes(vec![(ctype_hash, attester.clone())])
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				PublicCredentials::add(
					DoubleOrigin(ACCOUNT_00, attester.clone()).into(),
					Box::new(new_credential)
				),
				Error::<Test>::InvalidInput
			);
		});
}

#[test]
fn add_not_enough_balance() {
	let attester = sr25519_did_from_seed(&ALICE_SEED);
	let subject_id = SUBJECT_ID_00;
	let ctype_hash = get_ctype_hash::<Test>(true);
	let new_credential = generate_base_public_credential_creation_op::<Test>(
		subject_id.into(),
		ctype_hash,
		InputClaimsContentOf::<Test>::default(),
	);
	let deposit: Balance = <Test as Config>::Deposit::get();

	ExtBuilder::default()
		// One less than the minimum required
		.with_balances(vec![(ACCOUNT_00, deposit - 1)])
		.with_ctypes(vec![(ctype_hash, attester.clone())])
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				PublicCredentials::add(
					DoubleOrigin(ACCOUNT_00, attester.clone()).into(),
					Box::new(new_credential)
				),
				Error::<Test>::UnableToPayFees
			);
		});
}

// revoke

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

// remove

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

// reclaim_deposit

#[test]
fn reclaim_deposit_successful() {
	let attester = sr25519_did_from_seed(&ALICE_SEED);
	let subject_id: <Test as Config>::SubjectId = SUBJECT_ID_00;
	let new_credential = generate_base_credential_entry::<Test>(ACCOUNT_00, 0, attester, None, None);
	let credential_id: CredentialIdOf<Test> = CredentialIdOf::<Test>::default();
	let deposit: Balance = <Test as Config>::Deposit::get();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, deposit + MIN_BALANCE)])
		.with_public_credentials(vec![(subject_id, credential_id, new_credential)])
		.build_and_execute_with_sanity_tests(|| {
			assert_ok!(PublicCredentials::reclaim_deposit(
				RuntimeOrigin::signed(ACCOUNT_00),
				credential_id
			));

			// Test this pallet logic
			assert!(Credentials::<Test>::get(subject_id, credential_id).is_none());
			assert!(CredentialSubjects::<Test>::get(credential_id).is_none());

			// Check deposit release logic
			assert!(Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00).is_zero());

			// Reclaiming the deposit for the same credential again will fail
			assert_noop!(
				PublicCredentials::reclaim_deposit(RuntimeOrigin::signed(ACCOUNT_00), credential_id),
				Error::<Test>::NotFound
			);

			assert_noop!(
				PublicCredentials::reclaim_deposit(RuntimeOrigin::signed(ACCOUNT_00), credential_id),
				Error::<Test>::NotFound
			);
		});
}

#[test]
fn reclaim_deposit_credential_not_found() {
	let credential_id: CredentialIdOf<Test> = CredentialIdOf::<Test>::default();
	let deposit: Balance = <Test as Config>::Deposit::get();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, deposit)])
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				PublicCredentials::reclaim_deposit(RuntimeOrigin::signed(ACCOUNT_00), credential_id),
				Error::<Test>::NotFound
			);
		});
}

#[test]
fn reclaim_deposit_unauthorized() {
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
		.with_ctypes(vec![(ctype_hash_1, attester)])
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				PublicCredentials::reclaim_deposit(RuntimeOrigin::signed(ACCOUNT_01), credential_id),
				Error::<Test>::NotAuthorized
			);
		});
}

// change deposit owner

#[test]
fn test_change_deposit_owner() {
	let attester = sr25519_did_from_seed(&ALICE_SEED);
	let subject_id: <Test as Config>::SubjectId = SUBJECT_ID_00;
	let deposit: Balance = <Test as Config>::Deposit::get();
	let ctype_hash_1 = get_ctype_hash::<Test>(true);
	let new_credential = generate_base_credential_entry::<Test>(
		ACCOUNT_00,
		0,
		attester.clone(),
		Some(ctype_hash_1),
		Some(Deposit {
			owner: ACCOUNT_00,
			amount: deposit,
		}),
	);
	let credential_id: CredentialIdOf<Test> = CredentialIdOf::<Test>::default();

	ExtBuilder::default()
		.with_balances(vec![
			(ACCOUNT_00, deposit + MIN_BALANCE),
			(ACCOUNT_01, deposit + MIN_BALANCE),
		])
		.with_public_credentials(vec![(subject_id, credential_id, new_credential)])
		.with_ctypes(vec![(ctype_hash_1, attester.clone())])
		.build_and_execute_with_sanity_tests(|| {
			assert_ok!(PublicCredentials::change_deposit_owner(
				DoubleOrigin(ACCOUNT_01, attester.clone()).into(),
				credential_id
			));

			// Check
			assert_eq!(
				Credentials::<Test>::get(subject_id, credential_id)
					.expect("credential should exist")
					.deposit
					.owner,
				ACCOUNT_01
			);
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_01),
				<Test as Config>::Deposit::get()
			);
			assert!(Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00).is_zero());
		});
}

#[test]
fn test_change_deposit_owner_not_found() {
	let attester = sr25519_did_from_seed(&ALICE_SEED);
	let deposit: Balance = <Test as Config>::Deposit::get();
	let credential_id: CredentialIdOf<Test> = CredentialIdOf::<Test>::default();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, deposit), (ACCOUNT_01, deposit)])
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				PublicCredentials::change_deposit_owner(
					DoubleOrigin(ACCOUNT_01, attester.clone()).into(),
					credential_id
				),
				Error::<Test>::NotFound
			);
		});
}

#[test]
fn test_change_deposit_owner_unauthorized() {
	let attester = sr25519_did_from_seed(&ALICE_SEED);
	let evil = sr25519_did_from_seed(&BOB_SEED);
	let subject_id: <Test as Config>::SubjectId = SUBJECT_ID_00;
	let ctype_hash_1 = get_ctype_hash::<Test>(true);
	let deposit: Balance = <Test as Config>::Deposit::get();
	let new_credential = generate_base_credential_entry::<Test>(
		ACCOUNT_00,
		0,
		attester.clone(),
		Some(ctype_hash_1),
		Some(Deposit {
			owner: ACCOUNT_00,
			amount: deposit,
		}),
	);
	let credential_id: CredentialIdOf<Test> = CredentialIdOf::<Test>::default();

	ExtBuilder::default()
		.with_balances(vec![
			(ACCOUNT_00, deposit + MIN_BALANCE),
			(ACCOUNT_01, deposit + MIN_BALANCE),
		])
		.with_public_credentials(vec![(subject_id, credential_id, new_credential)])
		.with_ctypes(vec![(ctype_hash_1, attester)])
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				PublicCredentials::change_deposit_owner(DoubleOrigin(ACCOUNT_01, evil.clone()).into(), credential_id),
				Error::<Test>::NotAuthorized
			);
		});
}

// update deposit

#[test]
fn test_update_deposit() {
	let attester = sr25519_did_from_seed(&ALICE_SEED);
	let subject_id: <Test as Config>::SubjectId = SUBJECT_ID_00;
	let deposit_old: Balance = MILLI_UNIT * 10;
	let ctype_hash_1 = get_ctype_hash::<Test>(true);
	let new_credential = generate_base_credential_entry::<Test>(
		ACCOUNT_00,
		0,
		attester.clone(),
		Some(ctype_hash_1),
		Some(Deposit {
			owner: ACCOUNT_00,
			amount: deposit_old,
		}),
	);
	let credential_id: CredentialIdOf<Test> = CredentialIdOf::<Test>::default();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, deposit_old + MIN_BALANCE)])
		.with_public_credentials(vec![(subject_id, credential_id, new_credential)])
		.with_ctypes(vec![(ctype_hash_1, attester)])
		.build_and_execute_with_sanity_tests(|| {
			assert_ok!(PublicCredentials::update_deposit(
				RuntimeOrigin::signed(ACCOUNT_00),
				credential_id
			));

			// Check
			assert_eq!(
				Credentials::<Test>::get(subject_id, credential_id)
					.expect("credential should exist")
					.deposit
					.amount,
				<Test as Config>::Deposit::get()
			);
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				<Test as Config>::Deposit::get()
			);
		});
}

#[test]
fn test_update_deposit_not_found() {
	let credential_id: CredentialIdOf<Test> = CredentialIdOf::<Test>::default();
	let deposit: Balance = <Test as Config>::Deposit::get();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, deposit)])
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				PublicCredentials::update_deposit(RuntimeOrigin::signed(ACCOUNT_01), credential_id),
				Error::<Test>::NotFound
			);
		});
}

#[test]
fn test_update_deposit_unauthorized() {
	let attester = sr25519_did_from_seed(&ALICE_SEED);
	let ctype_hash_1 = get_ctype_hash::<Test>(true);
	let subject_id: <Test as Config>::SubjectId = SUBJECT_ID_00;
	let new_credential =
		generate_base_credential_entry::<Test>(ACCOUNT_00, 0, attester.clone(), Some(ctype_hash_1), None);
	let credential_id: CredentialIdOf<Test> = CredentialIdOf::<Test>::default();
	let deposit: Balance = <Test as Config>::Deposit::get();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, deposit + MIN_BALANCE)])
		.with_ctypes(vec![(ctype_hash_1, attester)])
		.with_public_credentials(vec![(subject_id, credential_id, new_credential)])
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				PublicCredentials::update_deposit(RuntimeOrigin::signed(ACCOUNT_01), credential_id),
				Error::<Test>::NotAuthorized
			);
		});
}
