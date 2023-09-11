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

use crate::{mock::*, Config, CredentialIdOf, CredentialSubjects, Credentials, Error, HoldReason};

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
