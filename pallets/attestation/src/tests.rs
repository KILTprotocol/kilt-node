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

use ctype::mock::get_ctype_hash;
use frame_support::{assert_noop, assert_ok, traits::fungible::InspectHold};
use sp_runtime::{traits::Zero, DispatchError, TokenError};

use kilt_support::{mock::mock_origin::DoubleOrigin, Deposit};

use crate::{
	self as attestation,
	mock::{runtime::Balances, *},
	AttestationAccessControl, AttesterOf, Config, Error, HoldReason,
};

// #############################################################################
// add

#[test]
fn test_attest_without_authorization() {
	let attester: AttesterOf<Test> = sr25519_did_from_seed(&ALICE_SEED);
	let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_01);
	let ctype_hash = get_ctype_hash::<Test>(true);
	let authorization_info = None;

	ExtBuilder::default()
		.with_ctypes(vec![(ctype_hash, attester.clone())])
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get() * 100)])
		.build_and_execute_with_sanity_tests(|| {
			assert_ok!(Attestation::add(
				DoubleOrigin(ACCOUNT_00, attester.clone()).into(),
				claim_hash,
				ctype_hash,
				authorization_info.clone()
			));
			let stored_attestation =
				Attestation::attestations(claim_hash).expect("Attestation should be present on chain.");

			assert_eq!(stored_attestation.ctype_hash, ctype_hash);
			assert_eq!(stored_attestation.attester, attester);
			assert_eq!(
				stored_attestation.authorization_id,
				authorization_info.map(|ac| ac.authorization_id())
			);
			assert!(!stored_attestation.revoked);
		});
}

#[test]
fn test_attest_authorized() {
	let attester: AttesterOf<Test> = sr25519_did_from_seed(&ALICE_SEED);
	let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_01);
	let ctype = get_ctype_hash::<Test>(true);
	let authorization_info = Some(MockAccessControl(attester.clone()));

	ExtBuilder::default()
		.with_ctypes(vec![(ctype, attester.clone())])
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get() * 100)])
		.build_and_execute_with_sanity_tests(|| {
			assert_ok!(Attestation::add(
				DoubleOrigin(ACCOUNT_00, attester.clone()).into(),
				claim_hash,
				ctype,
				authorization_info.clone()
			));
			let stored_attestation =
				Attestation::attestations(claim_hash).expect("Attestation should be present on chain.");
			assert!(Attestation::external_attestations(attester.clone(), claim_hash));

			assert_eq!(stored_attestation.ctype_hash, ctype);
			assert_eq!(stored_attestation.attester, attester);
			assert_eq!(
				stored_attestation.authorization_id,
				authorization_info.map(|ac| ac.authorization_id())
			);
			assert!(!stored_attestation.revoked);
		});
}

#[test]
fn test_attest_unauthorized() {
	let attester: AttesterOf<Test> = sr25519_did_from_seed(&ALICE_SEED);
	let bob: AttesterOf<Test> = sr25519_did_from_seed(&BOB_SEED);
	let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_01);
	let ctype = get_ctype_hash::<Test>(true);
	let authorization_info = Some(MockAccessControl(bob));

	ExtBuilder::default()
		.with_ctypes(vec![(ctype, attester.clone())])
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get() * 100)])
		.build_and_execute_with_sanity_tests(|| {
			assert_eq!(
				Attestation::add(
					DoubleOrigin(ACCOUNT_00, attester.clone()).into(),
					claim_hash,
					ctype,
					authorization_info
				),
				Err(DispatchError::Other("Unauthorized"))
			);
		});
}

#[test]
fn test_attest_ctype_not_found() {
	let attester: AttesterOf<Test> = sr25519_did_from_seed(&ALICE_SEED);
	let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_01);
	let ctype_hash = get_ctype_hash::<Test>(true);

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get() * 100)])
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				Attestation::add(
					DoubleOrigin(ACCOUNT_00, attester.clone()).into(),
					claim_hash,
					ctype_hash,
					None
				),
				ctype::Error::<Test>::NotFound
			);
		});
}

#[test]
fn test_attest_already_exists() {
	let attester: AttesterOf<Test> = sr25519_did_from_seed(&ALICE_SEED);
	let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_01);
	let attestation = generate_base_attestation::<Test>(attester.clone(), ACCOUNT_00);

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get() * 100)])
		.with_ctypes(vec![(attestation.ctype_hash, attester.clone())])
		.with_attestations(vec![(claim_hash, attestation.clone())])
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				Attestation::add(
					DoubleOrigin(ACCOUNT_00, attester.clone()).into(),
					claim_hash,
					attestation.ctype_hash,
					None
				),
				attestation::Error::<Test>::AlreadyAttested
			);
		});
}

// #############################################################################
// revoke

#[test]
fn test_revoke_remove() {
	let revoker: AttesterOf<Test> = sr25519_did_from_seed(&ALICE_SEED);
	let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_01);
	let attestation = generate_base_attestation::<Test>(revoker.clone(), ACCOUNT_00);

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get() * 100)])
		.with_ctypes(vec![(attestation.ctype_hash, revoker.clone())])
		.with_attestations(vec![(claim_hash, attestation)])
		.build_and_execute_with_sanity_tests(|| {
			assert_ok!(Attestation::revoke(
				DoubleOrigin(ACCOUNT_00, revoker.clone()).into(),
				claim_hash,
				None
			));
			let stored_attestation =
				Attestation::attestations(claim_hash).expect("Attestation should be present on chain.");

			assert!(stored_attestation.revoked);
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				<Test as Config>::Deposit::get()
			);

			assert_ok!(Attestation::remove(
				DoubleOrigin(ACCOUNT_00, revoker.clone()).into(),
				claim_hash,
				None
			));
			assert!(Attestation::attestations(claim_hash).is_none());
			assert!(Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00).is_zero());
		});
}

#[test]
fn test_authorized_revoke() {
	let attester: AttesterOf<Test> = sr25519_did_from_seed(&ALICE_SEED);
	let revoker: AttesterOf<Test> = sr25519_did_from_seed(&BOB_SEED);
	let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_01);
	let authorization_info = Some(MockAccessControl(revoker.clone()));
	let mut attestation = generate_base_attestation::<Test>(attester.clone(), ACCOUNT_00);
	attestation.authorization_id = Some(revoker.clone());

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get() * 100)])
		.with_ctypes(vec![(attestation.ctype_hash, attester)])
		.with_attestations(vec![(claim_hash, attestation)])
		.build_and_execute_with_sanity_tests(|| {
			assert_ok!(Attestation::revoke(
				DoubleOrigin(ACCOUNT_00, revoker.clone()).into(),
				claim_hash,
				authorization_info
			));
			let stored_attestation =
				Attestation::attestations(claim_hash).expect("Attestation should be present on chain.");
			assert!(Attestation::external_attestations(revoker.clone(), claim_hash));

			assert!(stored_attestation.revoked);
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				<Test as Config>::Deposit::get()
			);
		});
}

#[test]
fn test_unauthorized_revoke() {
	let attester: AttesterOf<Test> = sr25519_did_from_seed(&ALICE_SEED);
	let revoker: AttesterOf<Test> = sr25519_did_from_seed(&BOB_SEED);
	let evil: AttesterOf<Test> = sr25519_did_from_seed(&CHARLIE_SEED);

	let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_01);
	let authorization_info = Some(MockAccessControl(revoker.clone()));
	let mut attestation = generate_base_attestation::<Test>(attester.clone(), ACCOUNT_00);
	attestation.authorization_id = Some(revoker);

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get() * 100)])
		.with_ctypes(vec![(attestation.ctype_hash, attester)])
		.with_attestations(vec![(claim_hash, attestation)])
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				Attestation::revoke(DoubleOrigin(ACCOUNT_00, evil).into(), claim_hash, authorization_info),
				DispatchError::Other("Unauthorized")
			);
		});
}

#[test]
fn test_revoke_not_found() {
	let revoker: AttesterOf<Test> = sr25519_did_from_seed(&ALICE_SEED);
	let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_01);
	let authorization_info = Some(MockAccessControl(revoker.clone()));
	let attestation = generate_base_attestation::<Test>(revoker.clone(), ACCOUNT_00);

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get() * 100)])
		.with_ctypes(vec![(attestation.ctype_hash, revoker.clone())])
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				Attestation::revoke(
					DoubleOrigin(ACCOUNT_00, revoker.clone()).into(),
					claim_hash,
					authorization_info
				),
				attestation::Error::<Test>::NotFound
			);
		});
}

#[test]
fn test_already_revoked() {
	let revoker: AttesterOf<Test> = sr25519_did_from_seed(&ALICE_SEED);
	let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_01);
	let authorization_info = Some(MockAccessControl(revoker.clone()));

	// Attestation already revoked
	let mut attestation = generate_base_attestation::<Test>(revoker.clone(), ACCOUNT_00);
	attestation.revoked = true;

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get() * 100)])
		.with_ctypes(vec![(attestation.ctype_hash, revoker.clone())])
		.with_attestations(vec![(claim_hash, attestation)])
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				Attestation::revoke(
					DoubleOrigin(ACCOUNT_00, revoker.clone()).into(),
					claim_hash,
					authorization_info
				),
				attestation::Error::<Test>::AlreadyRevoked
			);
		});
}

// #############################################################################
// remove attestation

#[test]
fn test_remove() {
	let attester: AttesterOf<Test> = sr25519_did_from_seed(&ALICE_SEED);
	let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_01);
	let attestation = generate_base_attestation::<Test>(attester.clone(), ACCOUNT_00);
	let authorization_info = None;

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get() * 100)])
		.with_ctypes(vec![(attestation.ctype_hash, attester.clone())])
		.with_attestations(vec![(claim_hash, attestation)])
		.build_and_execute_with_sanity_tests(|| {
			assert_ok!(Attestation::remove(
				DoubleOrigin(ACCOUNT_00, attester.clone()).into(),
				claim_hash,
				authorization_info
			));
			assert!(Attestation::attestations(claim_hash).is_none());
		});
}

#[test]
fn test_remove_authorized() {
	let attester: AttesterOf<Test> = sr25519_did_from_seed(&ALICE_SEED);
	let revoker: AttesterOf<Test> = sr25519_did_from_seed(&BOB_SEED);
	let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_01);
	let mut attestation = generate_base_attestation::<Test>(attester, ACCOUNT_00);
	attestation.authorization_id = Some(revoker.clone());
	let authorization_info = Some(MockAccessControl(revoker.clone()));

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get() * 100)])
		.with_ctypes(vec![(attestation.ctype_hash, revoker.clone())])
		.with_attestations(vec![(claim_hash, attestation)])
		.build_and_execute_with_sanity_tests(|| {
			assert_ok!(Attestation::remove(
				DoubleOrigin(ACCOUNT_00, revoker.clone()).into(),
				claim_hash,
				authorization_info
			));
			assert!(Attestation::attestations(claim_hash).is_none());
			assert!(!Attestation::external_attestations(revoker.clone(), claim_hash));
		});
}

#[test]
fn test_remove_unauthorized() {
	let attester: AttesterOf<Test> = sr25519_did_from_seed(&ALICE_SEED);
	let evil: AttesterOf<Test> = sr25519_did_from_seed(&BOB_SEED);
	let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_01);
	let attestation = generate_base_attestation::<Test>(attester.clone(), ACCOUNT_00);
	let authorization_info = Some(MockAccessControl(evil.clone()));

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get() * 100)])
		.with_ctypes(vec![(attestation.ctype_hash, attester)])
		.with_attestations(vec![(claim_hash, attestation)])
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				Attestation::remove(
					DoubleOrigin(ACCOUNT_00, evil.clone()).into(),
					claim_hash,
					authorization_info
				),
				attestation::Error::<Test>::NotAuthorized
			);
		});
}

#[test]
fn test_remove_not_found() {
	let attester: AttesterOf<Test> = sr25519_did_from_seed(&ALICE_SEED);
	let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_01);
	let attestation = generate_base_attestation::<Test>(attester.clone(), ACCOUNT_00);

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get() * 100)])
		.with_ctypes(vec![(attestation.ctype_hash, attester.clone())])
		.build_and_execute_with_sanity_tests(|| {
			assert!(Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00).is_zero());
			assert_noop!(
				Attestation::remove(DoubleOrigin(ACCOUNT_00, attester.clone()).into(), claim_hash, None),
				attestation::Error::<Test>::NotFound
			);
		});
}

// #############################################################################
// reclaim deposit

#[test]
fn test_reclaim_deposit() {
	let attester: AttesterOf<Test> = sr25519_did_from_seed(&ALICE_SEED);
	let other_authorized: AttesterOf<Test> = sr25519_did_from_seed(&BOB_SEED);
	let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_01);
	let mut attestation = generate_base_attestation::<Test>(attester.clone(), ACCOUNT_00);
	attestation.authorization_id = Some(other_authorized.clone());

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get() * 100)])
		.with_ctypes(vec![(attestation.ctype_hash, attester)])
		.with_attestations(vec![(claim_hash, attestation)])
		.build_and_execute_with_sanity_tests(|| {
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				<Test as Config>::Deposit::get()
			);
			assert_ok!(Attestation::reclaim_deposit(
				RuntimeOrigin::signed(ACCOUNT_00),
				claim_hash
			));
			assert!(!Attestation::external_attestations(
				other_authorized.clone(),
				claim_hash
			));
			assert!(Attestation::attestations(claim_hash).is_none());
			assert!(Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00).is_zero());
		});
}

#[test]
fn test_reclaim_deposit_authorization() {
	let attester: AttesterOf<Test> = sr25519_did_from_seed(&BOB_SEED);
	let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_01);
	let attestation = generate_base_attestation::<Test>(attester.clone(), ACCOUNT_00);

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get() * 100)])
		.with_ctypes(vec![(attestation.ctype_hash, attester)])
		.with_attestations(vec![(claim_hash, attestation)])
		.build_and_execute_with_sanity_tests(|| {
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				<Test as Config>::Deposit::get()
			);
			assert_ok!(Attestation::reclaim_deposit(
				RuntimeOrigin::signed(ACCOUNT_00),
				claim_hash
			));
			assert!(Attestation::attestations(claim_hash).is_none());
			assert!(Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00).is_zero());
		});
}

#[test]
fn test_reclaim_unauthorized() {
	let attester: AttesterOf<Test> = sr25519_did_from_seed(&BOB_SEED);
	let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_01);
	let attestation = generate_base_attestation::<Test>(attester.clone(), ACCOUNT_00);

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get() * 100)])
		.with_ctypes(vec![(attestation.ctype_hash, attester)])
		.with_attestations(vec![(claim_hash, attestation)])
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				Attestation::reclaim_deposit(RuntimeOrigin::signed(ACCOUNT_01), claim_hash),
				attestation::Error::<Test>::NotAuthorized,
			);
		});
}

#[test]
fn test_reclaim_deposit_not_found() {
	let attester: AttesterOf<Test> = sr25519_did_from_seed(&BOB_SEED);
	let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_01);
	let attestation = generate_base_attestation::<Test>(attester.clone(), ACCOUNT_00);

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get() * 100)])
		.with_ctypes(vec![(attestation.ctype_hash, attester)])
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				Attestation::reclaim_deposit(RuntimeOrigin::signed(ACCOUNT_01), claim_hash),
				attestation::Error::<Test>::NotFound,
			);
		});
}

// #############################################################################
// transfer deposit

#[test]
fn test_change_deposit_owner() {
	let attester: AttesterOf<Test> = sr25519_did_from_seed(&ALICE_SEED);
	let other_authorized: AttesterOf<Test> = sr25519_did_from_seed(&BOB_SEED);
	let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_01);
	let mut attestation = generate_base_attestation::<Test>(attester.clone(), ACCOUNT_00);
	attestation.authorization_id = Some(other_authorized);

	ExtBuilder::default()
		.with_balances(vec![
			(ACCOUNT_00, <Test as Config>::Deposit::get() * 100),
			(ACCOUNT_01, <Test as Config>::Deposit::get() * 100),
		])
		.with_ctypes(vec![(attestation.ctype_hash, attester.clone())])
		.with_attestations(vec![(claim_hash, attestation)])
		.build_and_execute_with_sanity_tests(|| {
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				<Test as Config>::Deposit::get()
			);
			assert_ok!(Attestation::change_deposit_owner(
				DoubleOrigin(ACCOUNT_01, attester).into(),
				claim_hash
			));
			assert_eq!(
				Attestation::attestations(claim_hash)
					.expect("Attestation must be retained")
					.deposit,
				Deposit {
					owner: ACCOUNT_01,
					amount: <Test as Config>::Deposit::get()
				}
			);
			assert!(Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00).is_zero());
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_01),
				<Test as Config>::Deposit::get()
			);
		});
}

#[test]
fn test_change_deposit_owner_insufficient_balance() {
	let attester: AttesterOf<Test> = sr25519_did_from_seed(&ALICE_SEED);
	let other_authorized: AttesterOf<Test> = sr25519_did_from_seed(&BOB_SEED);
	let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_01);
	let mut attestation = generate_base_attestation::<Test>(attester.clone(), ACCOUNT_00);
	attestation.authorization_id = Some(other_authorized);

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get() * 100)])
		.with_ctypes(vec![(attestation.ctype_hash, attester.clone())])
		.with_attestations(vec![(claim_hash, attestation)])
		.build_and_execute_with_sanity_tests(|| {
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				<Test as Config>::Deposit::get()
			);
			assert_noop!(
				Attestation::change_deposit_owner(DoubleOrigin(ACCOUNT_01, attester).into(), claim_hash),
				TokenError::CannotCreateHold
			);
		});
}

#[test]
fn test_change_deposit_owner_unauthorized() {
	let attester: AttesterOf<Test> = sr25519_did_from_seed(&BOB_SEED);
	let evil_actor: AttesterOf<Test> = sr25519_did_from_seed(&ALICE_SEED);
	let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_01);
	let attestation = generate_base_attestation::<Test>(attester.clone(), ACCOUNT_00);

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get() * 100)])
		.with_ctypes(vec![(attestation.ctype_hash, attester)])
		.with_attestations(vec![(claim_hash, attestation)])
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				Attestation::change_deposit_owner(DoubleOrigin(ACCOUNT_00, evil_actor).into(), claim_hash),
				attestation::Error::<Test>::NotAuthorized,
			);
		});
}

#[test]
fn test_change_deposit_owner_not_found() {
	let attester: AttesterOf<Test> = sr25519_did_from_seed(&BOB_SEED);
	let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_01);
	let attestation = generate_base_attestation::<Test>(attester.clone(), ACCOUNT_00);

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get() * 100)])
		.with_ctypes(vec![(attestation.ctype_hash, attester.clone())])
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				Attestation::change_deposit_owner(DoubleOrigin(ACCOUNT_00, attester).into(), claim_hash),
				attestation::Error::<Test>::NotFound,
			);
		});
}

/// Update the deposit amount
#[test]
fn test_update_deposit() {
	let attester: AttesterOf<Test> = sr25519_did_from_seed(&BOB_SEED);
	let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_01);
	let attestation = generate_base_attestation_with_deposit::<Test>(
		attester.clone(),
		ACCOUNT_00,
		<Test as Config>::Deposit::get() * 2,
	);

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get() * 100)])
		.with_ctypes(vec![(attestation.ctype_hash, attester)])
		.with_attestations(vec![(claim_hash, attestation)])
		.build_and_execute_with_sanity_tests(|| {
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				<Test as Config>::Deposit::get() * 2
			);
			assert_ok!(Attestation::update_deposit(
				RuntimeOrigin::signed(ACCOUNT_00),
				claim_hash
			));
			assert_eq!(
				Attestation::attestations(claim_hash)
					.expect("Attestation must be retained")
					.deposit,
				Deposit {
					owner: ACCOUNT_00,
					amount: <Test as Config>::Deposit::get()
				}
			);
			// old deposit was 2x Deposit::get(), new deposit should be the the default
			// deposit value.
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				<Test as Config>::Deposit::get()
			);
		});
}

/// Update the deposit amount
#[test]
fn test_update_deposit_unauthorized() {
	let attester: AttesterOf<Test> = sr25519_did_from_seed(&BOB_SEED);
	let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_01);
	let attestation = generate_base_attestation_with_deposit::<Test>(
		attester.clone(),
		ACCOUNT_00,
		<Test as Config>::Deposit::get() * 2,
	);

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get() * 100)])
		.with_ctypes(vec![(attestation.ctype_hash, attester)])
		.with_attestations(vec![(claim_hash, attestation)])
		.build_and_execute_with_sanity_tests(|| {
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				<Test as Config>::Deposit::get() * 2
			);
			assert_noop!(
				Attestation::update_deposit(RuntimeOrigin::signed(ACCOUNT_01), claim_hash),
				Error::<Test>::NotAuthorized
			);
		});
}
