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
use frame_support::{assert_noop, assert_ok};
use kilt_support::mock::mock_origin::DoubleOrigin;
use sp_runtime::DispatchError;

use crate::{self as attestation, mock::*, AttestationAccessControl, AttesterOf, Config};

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
