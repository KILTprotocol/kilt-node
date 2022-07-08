// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2022 BOTLabs GmbH

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

use frame_support::{assert_noop, assert_ok};
use sp_runtime::traits::Zero;

use attestation::{attestations::AttestationDetails, mock::generate_base_attestation, Attestations};
use ctype::mock::get_ctype_hash;
use kilt_support::mock::mock_origin::DoubleOrigin;

use crate::{mock::*, Config, Credentials, CredentialsUnicityIndex, Error, InputClaimsContentOf};

// add

#[test]
fn add_with_no_signature_successful() {
	let attester = sr25519_did_from_seed(&ALICE_SEED);
	let subject_id = SUBJECT_ID_00;
	let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_01);
	let claim_hash_2 = claim_hash_from_seed(CLAIM_HASH_SEED_02);
	let ctype_hash = get_ctype_hash::<Test>(true);
	let new_credential_1 =
		generate_base_public_credential_creation_op::<Test>(subject_id.into(), claim_hash, ctype_hash, InputClaimsContentOf::<Test>::default(), None);
	let new_credential_2 =
		generate_base_public_credential_creation_op::<Test>(subject_id.into(), claim_hash_2, ctype_hash, InputClaimsContentOf::<Test>::default(), None);
	let public_credential_deposit = <Test as Config>::Deposit::get();
	let attestation_deposit = <Test as attestation::Config>::Deposit::get();

	ExtBuilder::default()
		.with_balances(vec![(
			ACCOUNT_00,
			(public_credential_deposit + attestation_deposit) * 2,
		)])
		.with_ctypes(vec![(ctype_hash, attester.clone())])
		.build()
		.execute_with(|| {
			// Check for 0 reserved deposit
			assert!(Balances::reserved_balance(ACCOUNT_00).is_zero());

			assert_ok!(PublicCredentials::add(
				DoubleOrigin(ACCOUNT_00, attester.clone()).into(),
				Box::new(new_credential_1.clone())
			));
			let stored_attestation =
				Attestations::<Test>::get(&claim_hash).expect("Attestation should be present on chain.");
			let stored_public_credential_details =
				Credentials::<Test>::get(&subject_id, &claim_hash)
					.expect("Public credential details should be present on chain.");

			// Test interactions with attestation pallet
			assert_eq!(stored_attestation.ctype_hash, ctype_hash);
			assert_eq!(stored_attestation.attester, attester);

			// Test this pallet logic
			assert_eq!(stored_public_credential_details.block_number, 0);
			assert_eq!(
				CredentialsUnicityIndex::<Test>::get(&claim_hash),
				Some(subject_id)
			);

			// Check deposit reservation logic
			assert_eq!(
				Balances::reserved_balance(ACCOUNT_00),
				public_credential_deposit + attestation_deposit
			);

			// Re-issuing the same credential will fail
			assert_noop!(
				PublicCredentials::add(
					DoubleOrigin(ACCOUNT_00, attester.clone()).into(),
					Box::new(new_credential_1.clone())
				),
				Error::<Test>::CredentialAlreadyIssued
			);

			// Check deposit has not changed
			assert_eq!(
				Balances::reserved_balance(ACCOUNT_00),
				public_credential_deposit + attestation_deposit
			);

			System::set_block_number(1);

			// Issuing a completely new credential will work
			assert_ok!(PublicCredentials::add(
				DoubleOrigin(ACCOUNT_00, attester.clone()).into(),
				Box::new(new_credential_2.clone())
			));

			let stored_attestation =
				Attestations::<Test>::get(&claim_hash_2).expect("Attestation #2 should be present on chain.");
			let stored_public_credential_details =
				Credentials::<Test>::get(&subject_id, &claim_hash_2)
					.expect("Public credential #2 details should be present on chain.");

			// Test interactions with attestation pallet
			assert_eq!(stored_attestation.ctype_hash, ctype_hash);
			assert_eq!(stored_attestation.attester, attester);

			// Test this pallet logic
			assert_eq!(stored_public_credential_details.block_number, 1);
			assert_eq!(
				CredentialsUnicityIndex::<Test>::get(&claim_hash_2),
				Some(subject_id)
			);

			// Deposit is 2x now
			assert_eq!(
				Balances::reserved_balance(ACCOUNT_00),
				2 * (public_credential_deposit + attestation_deposit)
			);

			// Deleting the attestation only from the attestation pallet will still fail
			Attestation::reclaim_deposit(Origin::signed(ACCOUNT_00), claim_hash)
				.expect("Attestation deposit reclaim should not fail");
			assert_noop!(
				PublicCredentials::add(
					DoubleOrigin(ACCOUNT_00, attester.clone()).into(),
					Box::new(new_credential_1.clone())
				),
				Error::<Test>::CredentialAlreadyIssued
			);

			// Deposit should now be equal to 1 attestation + 2 public credentials
			assert_eq!(
				Balances::reserved_balance(ACCOUNT_00),
				2 * public_credential_deposit + attestation_deposit
			);
		});
}

#[test]
fn add_with_claimer_signature_successful() {
	let attester = sr25519_did_from_seed(&ALICE_SEED);
	let subject_id = SUBJECT_ID_00;
	let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_01);
	let ctype_hash = get_ctype_hash::<Test>(true);
	let claimer_id = sr25519_did_from_seed(&BOB_SEED);
	// FIXME: Change the definition of Signature so that we can simply use a tuple
	// (claimer_id, claim_hash) as signature
	let new_credential = generate_base_public_credential_creation_op::<Test>(
		subject_id.into(),
		claim_hash,
		ctype_hash,
		InputClaimsContentOf::<Test>::default(),
		Some(ClaimerSignatureInfoOf::<Test> {
			claimer_id: claimer_id.clone(),
			signature_payload: (claimer_id, hash_to_u8(claim_hash)),
		}),
	);
	let public_credential_deposit = <Test as Config>::Deposit::get();
	let attestation_deposit = <Test as attestation::Config>::Deposit::get();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, public_credential_deposit + attestation_deposit)])
		.with_ctypes(vec![(ctype_hash, attester.clone())])
		.build()
		.execute_with(|| {
			assert_ok!(PublicCredentials::add(
				DoubleOrigin(ACCOUNT_00, attester.clone()).into(),
				Box::new(new_credential.clone())
			));
			let stored_attestation =
				Attestations::<Test>::get(&claim_hash).expect("Attestation should be present on chain.");
			let stored_public_credential_details =
				Credentials::<Test>::get(&subject_id, &claim_hash)
					.expect("Public credential details should be present on chain.");

			// Test interactions with attestation pallet
			assert_eq!(stored_attestation.ctype_hash, ctype_hash);
			assert_eq!(stored_attestation.attester, attester);

			// Test this pallet logic
			assert_eq!(stored_public_credential_details.block_number, System::block_number());
			assert_eq!(
				CredentialsUnicityIndex::<Test>::get(&claim_hash),
				Some(subject_id)
			);
		});
}

#[test]
fn add_not_enough_balance() {
	let attester = sr25519_did_from_seed(&ALICE_SEED);
	let subject_id = SUBJECT_ID_00;
	let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_01);
	let ctype_hash = get_ctype_hash::<Test>(true);
	let new_credential =
		generate_base_public_credential_creation_op::<Test>(subject_id.into(), claim_hash, ctype_hash, InputClaimsContentOf::<Test>::default(), None);
	let public_credential_deposit = <Test as Config>::Deposit::get();
	let attestation_deposit = <Test as attestation::Config>::Deposit::get();

	ExtBuilder::default()
		// One less than the minimum required
		.with_balances(vec![(ACCOUNT_00, public_credential_deposit + attestation_deposit - 1)])
		.with_ctypes(vec![(ctype_hash, attester.clone())])
		.build()
		.execute_with(|| {
			assert_noop!(
				PublicCredentials::add(
					DoubleOrigin(ACCOUNT_00, attester.clone()).into(),
					Box::new(new_credential.clone())
				),
				Error::<Test>::UnableToPayFees
			);
		});
}

#[test]
fn add_invalid_signature() {
	let attester = sr25519_did_from_seed(&ALICE_SEED);
	let subject_id = SUBJECT_ID_00;
	let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_01);
	let ctype_hash = get_ctype_hash::<Test>(true);
	let claimer_id = sr25519_did_from_seed(&CHARLIE_SEED);
	let new_credential = generate_base_public_credential_creation_op::<Test>(
		subject_id.into(),
		claim_hash,
		ctype_hash,
		InputClaimsContentOf::<Test>::default(),
		Some(ClaimerSignatureInfoOf::<Test> {
			claimer_id,
			signature_payload: (sr25519_did_from_seed(&BOB_SEED), hash_to_u8(claim_hash)),
		}),
	);
	let public_credential_deposit = <Test as Config>::Deposit::get();
	let attestation_deposit = <Test as attestation::Config>::Deposit::get();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, public_credential_deposit + attestation_deposit)])
		.with_ctypes(vec![(ctype_hash, attester.clone())])
		.build()
		.execute_with(|| {
			assert_noop!(
				PublicCredentials::add(
					DoubleOrigin(ACCOUNT_00, attester.clone()).into(),
					Box::new(new_credential.clone())
				),
				Error::<Test>::InvalidClaimerSignature
			);
		});
}

#[test]
fn add_ctype_not_found() {
	let attester = sr25519_did_from_seed(&ALICE_SEED);
	let subject_id = SUBJECT_ID_00;
	let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_01);
	let ctype_hash = get_ctype_hash::<Test>(true);
	let claimer_id = sr25519_did_from_seed(&CHARLIE_SEED);
	let new_credential = generate_base_public_credential_creation_op::<Test>(
		subject_id.into(),
		claim_hash,
		ctype_hash,
		InputClaimsContentOf::<Test>::default(),
		Some(ClaimerSignatureInfoOf::<Test> {
			claimer_id: claimer_id.clone(),
			signature_payload: (claimer_id, hash_to_u8(claim_hash)),
		}),
	);
	let public_credential_deposit = <Test as Config>::Deposit::get();
	let attestation_deposit = <Test as attestation::Config>::Deposit::get();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, public_credential_deposit + attestation_deposit)])
		.build()
		.execute_with(|| {
			assert_noop!(
				PublicCredentials::add(
					DoubleOrigin(ACCOUNT_00, attester.clone()).into(),
					Box::new(new_credential.clone())
				),
				ctype::Error::<Test>::CTypeNotFound
			);
		});
}

// remove

#[test]
fn remove_successful() {
	let attester = sr25519_did_from_seed(&ALICE_SEED);
	let subject_id: <Test as Config>::SubjectId = SUBJECT_ID_00;
	let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_01);
	let attestation: AttestationDetails<Test> = generate_base_attestation(attester.clone(), ACCOUNT_00);
	let new_credential = generate_base_credential_entry(ACCOUNT_00, 0);
	let public_credential_deposit = <Test as Config>::Deposit::get();
	let attestation_deposit = <Test as attestation::Config>::Deposit::get();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, public_credential_deposit + attestation_deposit)])
		.with_ctypes(vec![(attestation.ctype_hash, attester.clone())])
		.with_attestations(vec![(claim_hash, attestation.clone())])
		.with_public_credentials(vec![(subject_id, claim_hash, new_credential)])
		.build()
		.execute_with(|| {
			assert_ok!(PublicCredentials::remove(
				DoubleOrigin(ACCOUNT_00, attester.clone()).into(),
				claim_hash,
				None
			));
			// Test interactions with attestation pallet
			assert_eq!(Attestations::<Test>::get(&claim_hash), None);

			// Test this pallet logic
			assert_eq!(Credentials::<Test>::get(&subject_id, &claim_hash), None);
			assert_eq!(CredentialsUnicityIndex::<Test>::get(&claim_hash), None);

			// Check deposit release logic
			assert!(Balances::reserved_balance(ACCOUNT_00).is_zero());

			// Removing the same credential again will fail
			assert_noop!(
				PublicCredentials::remove(DoubleOrigin(ACCOUNT_00, attester.clone()).into(), claim_hash, None),
				Error::<Test>::CredentialNotFound
			);

			// Adding only the attestation without the credential will also fail to remove
			// the credential.
			Attestation::add(
				DoubleOrigin(ACCOUNT_00, attester.clone()).into(),
				claim_hash,
				attestation.ctype_hash,
				None,
			)
			.expect("Adding the same attestation again should not fail");

			assert_noop!(
				PublicCredentials::remove(DoubleOrigin(ACCOUNT_00, attester.clone()).into(), claim_hash, None),
				Error::<Test>::CredentialNotFound
			);

			// Check that only the attestation deposit is reserved
			assert_eq!(Balances::reserved_balance(ACCOUNT_00), attestation_deposit);
		});
}

#[test]
fn remove_unauthorized() {
	let attester = sr25519_did_from_seed(&ALICE_SEED);
	let wrong_attester = sr25519_did_from_seed(&BOB_SEED);
	let subject_id: <Test as Config>::SubjectId = SUBJECT_ID_00;
	let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_01);
	let attestation: AttestationDetails<Test> = generate_base_attestation(attester.clone(), ACCOUNT_00);
	let new_credential = generate_base_credential_entry(ACCOUNT_00, 0);
	let public_credential_deposit = <Test as Config>::Deposit::get();
	let attestation_deposit = <Test as attestation::Config>::Deposit::get();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, public_credential_deposit + attestation_deposit)])
		.with_ctypes(vec![(attestation.ctype_hash, attester)])
		.with_attestations(vec![(claim_hash, attestation)])
		.with_public_credentials(vec![(subject_id, claim_hash, new_credential)])
		.build()
		.execute_with(|| {
			assert_noop!(
				PublicCredentials::remove(DoubleOrigin(ACCOUNT_00, wrong_attester).into(), claim_hash, None),
				attestation::Error::<Test>::Unauthorized
			);
		});
}

// reclaim_deposit

#[test]
fn reclaim_deposit_successful() {
	let attester = sr25519_did_from_seed(&ALICE_SEED);
	let subject_id: <Test as Config>::SubjectId = SUBJECT_ID_00;
	let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_01);
	let attestation: AttestationDetails<Test> = generate_base_attestation(attester.clone(), ACCOUNT_00);
	let new_credential = generate_base_credential_entry(ACCOUNT_00, 0);
	let public_credential_deposit = <Test as Config>::Deposit::get();
	let attestation_deposit = <Test as attestation::Config>::Deposit::get();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, public_credential_deposit + attestation_deposit)])
		.with_ctypes(vec![(attestation.ctype_hash, attester.clone())])
		.with_attestations(vec![(claim_hash, attestation.clone())])
		.with_public_credentials(vec![(subject_id, claim_hash, new_credential)])
		.build()
		.execute_with(|| {
			assert_ok!(PublicCredentials::reclaim_deposit(
				Origin::signed(ACCOUNT_00),
				claim_hash
			));
			// Test interactions with attestation pallet
			assert_eq!(Attestations::<Test>::get(&claim_hash), None);

			// Test this pallet logic
			assert_eq!(Credentials::<Test>::get(&subject_id, &claim_hash), None);
			assert_eq!(CredentialsUnicityIndex::<Test>::get(&claim_hash), None);

			// Check deposit release logic
			assert!(Balances::reserved_balance(ACCOUNT_00).is_zero());

			// Reclaiming the deposit for the same credential again will fail
			assert_noop!(
				PublicCredentials::reclaim_deposit(Origin::signed(ACCOUNT_00), claim_hash),
				Error::<Test>::CredentialNotFound
			);

			// Adding only the attestation without the credential will also fail to reclaim
			// the deposit for the credential.
			Attestation::add(
				DoubleOrigin(ACCOUNT_00, attester.clone()).into(),
				claim_hash,
				attestation.ctype_hash,
				None,
			)
			.expect("Adding the same attestation again should not fail");

			assert_noop!(
				PublicCredentials::reclaim_deposit(Origin::signed(ACCOUNT_00), claim_hash),
				Error::<Test>::CredentialNotFound
			);

			// Check that only the attestation deposit is reserved
			assert_eq!(Balances::reserved_balance(ACCOUNT_00), attestation_deposit);
		});
}

#[test]
fn reclaim_deposit_unauthorized() {
	let attester = sr25519_did_from_seed(&ALICE_SEED);
	let subject_id: <Test as Config>::SubjectId = SUBJECT_ID_00;
	let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_01);
	let attestation: AttestationDetails<Test> = generate_base_attestation(attester.clone(), ACCOUNT_00);
	let new_credential = generate_base_credential_entry(ACCOUNT_00, 0);
	let public_credential_deposit = <Test as Config>::Deposit::get();
	let attestation_deposit = <Test as attestation::Config>::Deposit::get();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, public_credential_deposit + attestation_deposit)])
		.with_ctypes(vec![(attestation.ctype_hash, attester)])
		.with_attestations(vec![(claim_hash, attestation)])
		.with_public_credentials(vec![(subject_id, claim_hash, new_credential)])
		.build()
		.execute_with(|| {
			assert_noop!(
				PublicCredentials::reclaim_deposit(Origin::signed(ACCOUNT_01), claim_hash),
				attestation::Error::<Test>::Unauthorized
			);
		});
}
