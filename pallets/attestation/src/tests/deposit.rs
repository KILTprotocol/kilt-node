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

use frame_support::{assert_noop, assert_ok, traits::fungible::InspectHold};
use kilt_support::{mock::mock_origin::DoubleOrigin, Deposit};
use sp_runtime::{traits::Zero, TokenError};

use crate::{self as attestation, mock::*, AttesterOf, Config, Error, Event, HoldReason};

#[test]
fn test_reclaim_deposit_not_found() {
	let attester: AttesterOf<Test> = sr25519_did_from_public_key(&BOB_SEED);
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

#[test]
fn test_change_deposit_owner() {
	let attester: AttesterOf<Test> = sr25519_did_from_public_key(&ALICE_SEED);
	let other_authorized: AttesterOf<Test> = sr25519_did_from_public_key(&BOB_SEED);
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
			assert!(System::events().iter().any(|e| e.event
				== Event::<Test>::DepositOwnerChanged {
					id: claim_hash,
					from: ACCOUNT_00,
					to: ACCOUNT_01
				}
				.into()));
		});
}

#[test]
fn test_change_deposit_owner_insufficient_balance() {
	let attester: AttesterOf<Test> = sr25519_did_from_public_key(&ALICE_SEED);
	let other_authorized: AttesterOf<Test> = sr25519_did_from_public_key(&BOB_SEED);
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
				TokenError::FundsUnavailable
			);
		});
}

#[test]
fn test_change_deposit_owner_unauthorized() {
	let attester: AttesterOf<Test> = sr25519_did_from_public_key(&BOB_SEED);
	let evil_actor: AttesterOf<Test> = sr25519_did_from_public_key(&ALICE_SEED);
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
	let attester: AttesterOf<Test> = sr25519_did_from_public_key(&BOB_SEED);
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
	let attester: AttesterOf<Test> = sr25519_did_from_public_key(&BOB_SEED);
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
	let attester: AttesterOf<Test> = sr25519_did_from_public_key(&BOB_SEED);
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

#[test]
fn test_reclaim_deposit_authorization() {
	let attester: AttesterOf<Test> = sr25519_did_from_public_key(&ALICE_SEED);
	let other_authorized: AttesterOf<Test> = sr25519_did_from_public_key(&BOB_SEED);
	let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_01);
	let mut attestation = generate_base_attestation::<Test>(attester.clone(), ACCOUNT_00);
	attestation.authorization_id = Some(other_authorized.clone());
	let ctype_hash = attestation.ctype_hash;

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get() * 100)])
		.with_ctypes(vec![(attestation.ctype_hash, attester.clone())])
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
			assert_eq!(
				events(),
				vec![
					Event::AttestationRevoked {
						attester: attester.clone(),
						claim_hash,
						ctype_hash,
						authorized_by: attestation::authorized_by::AuthorizedBy::DepositOwner(ACCOUNT_00)
					},
					Event::AttestationRemoved {
						attester: attester.clone(),
						claim_hash,
						ctype_hash,
						authorized_by: attestation::authorized_by::AuthorizedBy::DepositOwner(ACCOUNT_00)
					}
				]
			);
		});
}

#[test]
fn test_reclaim_deposit() {
	let attester: AttesterOf<Test> = sr25519_did_from_public_key(&BOB_SEED);
	let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_01);
	let attestation = generate_base_attestation::<Test>(attester.clone(), ACCOUNT_00);
	let ctype_hash = attestation.ctype_hash;

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get() * 100)])
		.with_ctypes(vec![(attestation.ctype_hash, attester.clone())])
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

			assert_eq!(
				events(),
				vec![
					Event::AttestationRevoked {
						attester: attester.clone(),
						claim_hash,
						ctype_hash,
						authorized_by: attestation::authorized_by::AuthorizedBy::DepositOwner(ACCOUNT_00)
					},
					Event::AttestationRemoved {
						attester: attester.clone(),
						claim_hash,
						ctype_hash,
						authorized_by: attestation::authorized_by::AuthorizedBy::DepositOwner(ACCOUNT_00)
					}
				]
			);
		});
}

#[test]
fn test_reclaim_deposit_revoked() {
	let attester: AttesterOf<Test> = sr25519_did_from_public_key(&BOB_SEED);
	let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_01);
	let mut attestation = generate_base_attestation::<Test>(attester.clone(), ACCOUNT_00);
	attestation.revoked = true;
	let ctype_hash = attestation.ctype_hash;

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get() * 100)])
		.with_ctypes(vec![(attestation.ctype_hash, attester.clone())])
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

			assert_eq!(
				events(),
				vec![Event::AttestationRemoved {
					attester: attester.clone(),
					claim_hash,
					ctype_hash,
					authorized_by: attestation::authorized_by::AuthorizedBy::DepositOwner(ACCOUNT_00)
				}]
			);
		});
}

#[test]
fn test_reclaim_deposit_unauthorized() {
	let attester: AttesterOf<Test> = sr25519_did_from_public_key(&BOB_SEED);
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
