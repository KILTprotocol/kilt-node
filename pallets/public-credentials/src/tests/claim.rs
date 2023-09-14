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

use crate::{
	mock::*, Config, CredentialIdOf, CredentialSubjects, Credentials, Error, HoldReason, InputClaimsContentOf,
};
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
