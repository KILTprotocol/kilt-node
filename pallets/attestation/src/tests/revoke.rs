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

use frame_support::{assert_noop, assert_ok, traits::fungible::InspectHold};
use kilt_support::mock::mock_origin::DoubleOrigin;
use sp_runtime::{traits::Zero, DispatchError};

use crate::{self as attestation, mock::*, AttesterOf, Config, HoldReason};

#[test]
fn test_revoke_remove() {
	let revoker: AttesterOf<Test> = sr25519_did_from_public_key(&ALICE_SEED);
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
	let attester: AttesterOf<Test> = sr25519_did_from_public_key(&ALICE_SEED);
	let revoker: AttesterOf<Test> = sr25519_did_from_public_key(&BOB_SEED);
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
	let attester: AttesterOf<Test> = sr25519_did_from_public_key(&ALICE_SEED);
	let revoker: AttesterOf<Test> = sr25519_did_from_public_key(&BOB_SEED);
	let evil: AttesterOf<Test> = sr25519_did_from_public_key(&CHARLIE_SEED);

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
	let revoker: AttesterOf<Test> = sr25519_did_from_public_key(&ALICE_SEED);
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
	let revoker: AttesterOf<Test> = sr25519_did_from_public_key(&ALICE_SEED);
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
