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
use sp_runtime::traits::Zero;

use crate::{self as attestation, mock::*, AttesterOf, Config, HoldReason};

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
