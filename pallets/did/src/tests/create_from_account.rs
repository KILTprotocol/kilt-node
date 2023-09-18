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
	traits::fungible::{Inspect, InspectHold},
};
use sp_core::{ecdsa, ed25519, sr25519};

use crate::{
	did_details::{DidPublicKey, DidVerificationKey},
	mock::*,
	AccountIdOf, Config, Error, HoldReason,
};

/// Tests the creation of a DID.
/// This assumes that the `account` can be derived from the `verification_key`
/// and the creation is successful.
fn blueprint_test_successful(account_id: AccountIdOf<Test>, verification_key: DidVerificationKey<AccountIdOf<Test>>) {
	let balance = <Test as Config>::BaseDeposit::get()
		+ <Test as Config>::Fee::get()
		+ <<Test as Config>::Currency as Inspect<AccountIdOf<Test>>>::minimum_balance();

	ExtBuilder::default()
		.with_balances(vec![(account_id.clone(), balance)])
		.build_and_execute_with_sanity_tests(None, || {
			assert!(Did::get_did(&account_id).is_none());

			assert_ok!(Did::create_from_account(
				RuntimeOrigin::signed(account_id.clone()),
				verification_key.clone(),
			));

			let stored_did = Did::get_did(&account_id).expect("DID should be present on chain.");
			assert_eq!(stored_did.key_agreement_keys.len(), 0);
			assert_eq!(stored_did.delegation_key, None);
			assert_eq!(stored_did.attestation_key, None);
			assert_eq!(stored_did.public_keys.len(), 1);
			assert!(stored_did
				.public_keys
				.contains_key(&generate_key_id(&verification_key.clone().into())));
			assert_eq!(stored_did.last_tx_counter, 0u64);
			assert_eq!(
				stored_did
					.public_keys
					.values()
					.next()
					.map(|details| details.key.clone()),
				Some(DidPublicKey::PublicVerificationKey(verification_key))
			);
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &account_id),
				<Test as Config>::BaseDeposit::get()
			);
		});
}

#[test]
fn successful_ed25519() {
	let verification_key = DidVerificationKey::Ed25519(ed25519::Public(*ACCOUNT_00.as_ref()));
	blueprint_test_successful(ACCOUNT_00, verification_key);
}

#[test]
fn successful_sr25519() {
	let verification_key = DidVerificationKey::Sr25519(sr25519::Public(*ACCOUNT_00.as_ref()));
	blueprint_test_successful(ACCOUNT_00, verification_key);
}

#[test]
fn successful_ecdsa() {
	// these values where generated with `subkey generate -n kilt --scheme ecdsa`
	let verification_key = DidVerificationKey::Ecdsa(ecdsa::Public(hex_literal::hex!(
		"02484c08122e16f2cbce7697b5a9393280ca67dd8b91a907c1bc4b93451ebf4093"
	)));
	let account_id: AccountIdOf<Test> =
		hex_literal::hex!("375df6416958de6cb384516d3dead111c3a932c9e658ec1afd776e71bd2303b3").into();
	blueprint_test_successful(account_id, verification_key);
}

#[test]
fn successful_account() {
	let verification_key = DidVerificationKey::Account(ACCOUNT_00);
	blueprint_test_successful(ACCOUNT_00, verification_key);
}

#[test]
fn should_not_overwrite() {
	let verification_key = DidVerificationKey::Sr25519(sr25519::Public(*ACCOUNT_00.as_ref()));
	let account_id = ACCOUNT_00;

	let balance = <Test as Config>::BaseDeposit::get()
		+ <Test as Config>::Fee::get()
		+ <<Test as Config>::Currency as Inspect<AccountIdOf<Test>>>::minimum_balance();

	ExtBuilder::default()
		// we take twice the amount of balance so that we can create two DIDs
		.with_balances(vec![(account_id.clone(), balance * 2)])
		.build_and_execute_with_sanity_tests(None, || {
			assert!(Did::get_did(&account_id).is_none());

			assert_ok!(Did::create_from_account(
				RuntimeOrigin::signed(account_id.clone()),
				verification_key.clone(),
			));

			assert_noop!(
				Did::create_from_account(RuntimeOrigin::signed(account_id.clone()), verification_key.clone(),),
				Error::<Test>::AlreadyExists
			);
		});
}

#[test]
fn should_not_recreate_deleted_did() {
	let verification_key = DidVerificationKey::Sr25519(sr25519::Public(*ACCOUNT_00.as_ref()));
	let account_id = ACCOUNT_00;

	let balance = <Test as Config>::BaseDeposit::get()
		+ <Test as Config>::Fee::get()
		+ <<Test as Config>::Currency as Inspect<AccountIdOf<Test>>>::minimum_balance();

	ExtBuilder::default()
		.with_balances(vec![(account_id.clone(), balance)])
		.build_and_execute_with_sanity_tests(None, || {
			assert!(Did::get_did(&account_id).is_none());

			assert_ok!(Did::create_from_account(
				RuntimeOrigin::signed(account_id.clone()),
				verification_key.clone(),
			));

			let origin = build_test_origin(account_id.clone(), account_id.clone());
			assert_ok!(Did::delete(origin, 0));

			assert_noop!(
				Did::create_from_account(RuntimeOrigin::signed(account_id.clone()), verification_key.clone(),),
				Error::<Test>::AlreadyDeleted
			);
		});
}

#[test]
fn should_not_create_without_funds() {
	let verification_key = DidVerificationKey::Sr25519(sr25519::Public(*ACCOUNT_00.as_ref()));
	let account_id = ACCOUNT_00;

	ExtBuilder::default().build_and_execute_with_sanity_tests(None, || {
		assert!(Did::get_did(&account_id).is_none());

		assert_noop!(
			Did::create_from_account(RuntimeOrigin::signed(account_id.clone()), verification_key.clone(),),
			Error::<Test>::UnableToPayFees
		);
	});
}
