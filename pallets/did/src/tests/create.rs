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
use parity_scale_codec::Encode;
use sp_core::Pair;
use sp_runtime::{traits::BadOrigin, SaturatedConversion};
use sp_std::{collections::btree_set::BTreeSet, convert::TryFrom};

use crate::{
	self as did,
	did_details::{DidEncryptionKey, DidVerificationKey},
	mock::*,
	mock_utils::*,
	service_endpoints::DidEndpoint,
	HoldReason,
};

#[test]
fn check_successful_simple_ed25519_creation() {
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let auth_did_key = DidVerificationKey::from(auth_key.public());
	let details = generate_base_did_creation_details::<Test>(alice_did.clone(), ACCOUNT_00);

	let signature = auth_key.sign(details.encode().as_ref());

	let balance = <Test as did::Config>::BaseDeposit::get()
		+ <Test as did::Config>::Fee::get()
		+ <<Test as did::Config>::Currency as Inspect<did::AccountIdOf<Test>>>::minimum_balance();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, balance)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_ok!(Did::create(
				RuntimeOrigin::signed(ACCOUNT_00),
				Box::new(details),
				did::DidSignature::from(signature),
			));
			let stored_did = Did::get_did(&alice_did).expect("ALICE_DID should be present on chain.");
			assert_eq!(
				stored_did.authentication_key,
				generate_key_id(&auth_did_key.clone().into())
			);
			assert_eq!(stored_did.key_agreement_keys.len(), 0);
			assert_eq!(stored_did.delegation_key, None);
			assert_eq!(stored_did.attestation_key, None);
			assert_eq!(stored_did.public_keys.len(), 1);
			assert!(stored_did
				.public_keys
				.contains_key(&generate_key_id(&auth_did_key.into())));
			assert_eq!(stored_did.last_tx_counter, 0u64);

			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				<Test as did::Config>::BaseDeposit::get()
			);

			assert_eq!(Balances::balance(&ACCOUNT_FEE), <Test as did::Config>::Fee::get());
		});
}

#[test]
fn check_successful_simple_sr25519_creation() {
	let auth_key = get_sr25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	let auth_did_key = DidVerificationKey::from(auth_key.public());
	let details = generate_base_did_creation_details::<Test>(alice_did.clone(), ACCOUNT_00);

	let signature = auth_key.sign(details.encode().as_ref());

	let balance = <Test as did::Config>::BaseDeposit::get()
		+ <Test as did::Config>::Fee::get()
		+ <<Test as did::Config>::Currency as Inspect<did::AccountIdOf<Test>>>::minimum_balance();
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, balance)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_ok!(Did::create(
				RuntimeOrigin::signed(ACCOUNT_00),
				Box::new(details),
				did::DidSignature::from(signature),
			));
			let stored_did = Did::get_did(&alice_did).expect("ALICE_DID should be present on chain.");
			assert_eq!(
				stored_did.authentication_key,
				generate_key_id(&auth_did_key.clone().into())
			);
			assert_eq!(stored_did.key_agreement_keys.len(), 0);
			assert_eq!(stored_did.delegation_key, None);
			assert_eq!(stored_did.attestation_key, None);
			assert_eq!(stored_did.public_keys.len(), 1);
			assert!(stored_did
				.public_keys
				.contains_key(&generate_key_id(&auth_did_key.into())));
			assert_eq!(stored_did.last_tx_counter, 0u64);

			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				<Test as did::Config>::BaseDeposit::get()
			);
			assert_eq!(Balances::balance(&ACCOUNT_FEE), <Test as did::Config>::Fee::get());
		});
}

#[test]
fn check_successful_simple_ecdsa_creation() {
	let auth_key = get_ecdsa_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_ecdsa_key(auth_key.public());
	let auth_did_key = DidVerificationKey::from(auth_key.public());
	let details = generate_base_did_creation_details::<Test>(alice_did.clone(), ACCOUNT_00);

	let signature = auth_key.sign(details.encode().as_ref());

	let balance = <Test as did::Config>::BaseDeposit::get()
		+ <Test as did::Config>::Fee::get()
		+ <<Test as did::Config>::Currency as Inspect<did::AccountIdOf<Test>>>::minimum_balance();
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, balance)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_ok!(Did::create(
				RuntimeOrigin::signed(ACCOUNT_00),
				Box::new(details),
				did::DidSignature::from(signature),
			));
			let stored_did = Did::get_did(&alice_did).expect("ALICE_DID should be present on chain.");
			assert_eq!(
				stored_did.authentication_key,
				generate_key_id(&auth_did_key.clone().into())
			);
			assert_eq!(stored_did.key_agreement_keys.len(), 0);
			assert_eq!(stored_did.delegation_key, None);
			assert_eq!(stored_did.attestation_key, None);
			assert_eq!(stored_did.public_keys.len(), 1);
			assert!(stored_did
				.public_keys
				.contains_key(&generate_key_id(&auth_did_key.into())));
			assert_eq!(stored_did.last_tx_counter, 0u64);

			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				<Test as did::Config>::BaseDeposit::get()
			);
			assert_eq!(Balances::balance(&ACCOUNT_FEE), <Test as did::Config>::Fee::get());
		});
}

#[test]
fn check_successful_complete_creation() {
	let auth_key = get_sr25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	let auth_did_key = DidVerificationKey::from(auth_key.public());
	let enc_keys = DidNewKeyAgreementKeySetOf::<Test>::try_from(
		vec![
			get_x25519_encryption_key(&ENC_SEED_0),
			get_x25519_encryption_key(&ENC_SEED_1),
		]
		.iter()
		.copied()
		.collect::<BTreeSet<DidEncryptionKey>>(),
	)
	.expect("Exceeded BoundedBTreeSet bounds when creating new key agreement keys");
	let del_key = get_sr25519_delegation_key(&DEL_SEED_0);
	let att_key = get_ecdsa_attestation_key(&AUTH_SEED_0);
	let mut details = generate_base_did_creation_details::<Test>(alice_did.clone(), ACCOUNT_00);
	details.new_key_agreement_keys = enc_keys.clone();
	details.new_attestation_key = Some(DidVerificationKey::from(att_key.public()));
	details.new_delegation_key = Some(DidVerificationKey::from(del_key.public()));
	details.new_service_details = get_service_endpoints(
		<Test as did::Config>::MaxNumberOfServicesPerDid::get(),
		<Test as did::Config>::MaxServiceIdLength::get(),
		<Test as did::Config>::MaxNumberOfTypesPerService::get(),
		<Test as did::Config>::MaxServiceTypeLength::get(),
		<Test as did::Config>::MaxNumberOfUrlsPerService::get(),
		<Test as did::Config>::MaxServiceUrlLength::get(),
	);
	let signature = auth_key.sign(details.encode().as_ref());

	let required_balance_for_endpoint = <Test as did::Config>::ServiceEndpointDeposit::get()
		* <Test as did::Config>::MaxNumberOfServicesPerDid::get() as u128;

	let required_balance_for_keys = <Test as did::Config>::KeyDeposit::get() * 2;

	let required_balance_for_key_agreement = <Test as did::Config>::KeyDeposit::get() * enc_keys.len() as u128;

	let balance = required_balance_for_endpoint
		+ required_balance_for_keys
		+ required_balance_for_key_agreement
		+ <Test as did::Config>::BaseDeposit::get()
		+ <Test as did::Config>::Fee::get()
		+ <<Test as did::Config>::Currency as Inspect<did::AccountIdOf<Test>>>::minimum_balance();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, balance)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_ok!(Did::create(
				RuntimeOrigin::signed(ACCOUNT_00),
				Box::new(details.clone()),
				did::DidSignature::from(signature),
			));

			let stored_did = Did::get_did(&alice_did).expect("ALICE_DID should be present on chain.");
			assert_eq!(
				stored_did.authentication_key,
				generate_key_id(&auth_did_key.clone().into())
			);
			assert_eq!(stored_did.key_agreement_keys.len(), 2);
			for key in enc_keys.iter().copied() {
				assert!(stored_did.key_agreement_keys.contains(&generate_key_id(&key.into())))
			}
			assert_eq!(
				stored_did.delegation_key,
				Some(generate_key_id(&details.new_delegation_key.clone().unwrap().into()))
			);
			assert_eq!(
				stored_did.attestation_key,
				Some(generate_key_id(&details.new_attestation_key.clone().unwrap().into()))
			);
			// Authentication key + 2 * Encryption key + Delegation key + Attestation key =
			// 5
			assert_eq!(stored_did.public_keys.len(), 5);
			assert!(stored_did
				.public_keys
				.contains_key(&generate_key_id(&auth_did_key.into())));
			let mut key_agreement_keys_iterator = details.new_key_agreement_keys.iter().copied();
			assert!(stored_did
				.public_keys
				.contains_key(&generate_key_id(&key_agreement_keys_iterator.next().unwrap().into())));
			assert!(stored_did
				.public_keys
				.contains_key(&generate_key_id(&key_agreement_keys_iterator.next().unwrap().into())));
			assert!(stored_did
				.public_keys
				.contains_key(&generate_key_id(&details.new_attestation_key.clone().unwrap().into())));
			assert!(stored_did
				.public_keys
				.contains_key(&generate_key_id(&details.new_delegation_key.clone().unwrap().into())));

			// We check that the service details in the creation operation have been all
			// stored in the storage...
			details.new_service_details.iter().for_each(|new_service| {
				let stored_service = Did::get_service_endpoints(&alice_did, &new_service.id)
					.expect("Service endpoint should be stored.");
				assert_eq!(stored_service.id, new_service.id);
				assert_eq!(stored_service.urls, new_service.urls);
				assert_eq!(stored_service.service_types, new_service.service_types);
			});
			// ... and that the number of elements in the creation operation is the same as
			// the number of elements stored in `ServiceEndpoints` and `DidEndpointsCount`.
			assert_eq!(
				did::pallet::ServiceEndpoints::<Test>::iter_prefix(&alice_did).count(),
				details.new_service_details.len()
			);
			assert_eq!(
				did::pallet::DidEndpointsCount::<Test>::get(&alice_did).saturated_into::<usize>(),
				details.new_service_details.len()
			);

			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				<Test as did::Config>::BaseDeposit::get()
					+ required_balance_for_endpoint
					+ required_balance_for_key_agreement
					+ required_balance_for_keys
			);
			assert_eq!(Balances::balance(&ACCOUNT_FEE), <Test as did::Config>::Fee::get());
		});
}

#[test]
fn check_duplicate_did_creation() {
	let auth_key = get_sr25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	let auth_did_key = DidVerificationKey::from(auth_key.public());
	let mock_did = generate_base_did_details::<Test>(auth_did_key, Some(ACCOUNT_00));
	let details = generate_base_did_creation_details::<Test>(alice_did.clone(), ACCOUNT_00);

	let signature = auth_key.sign(details.encode().as_ref());

	let balance = <Test as did::Config>::BaseDeposit::get() * 20
		+ <Test as did::Config>::Fee::get()
		+ <<Test as did::Config>::Currency as Inspect<did::AccountIdOf<Test>>>::minimum_balance();
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, balance)])
		.with_dids(vec![(alice_did, mock_did)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::create(
					RuntimeOrigin::signed(ACCOUNT_00),
					Box::new(details),
					did::DidSignature::from(signature)
				),
				did::Error::<Test>::AlreadyExists
			);
		});
}

#[test]
fn check_unauthorised_submitter_did_creation_error() {
	let auth_key = get_sr25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	let auth_did_key = DidVerificationKey::from(auth_key.public());
	let mock_did = generate_base_did_details::<Test>(auth_did_key, Some(ACCOUNT_00));
	// Use ACCOUNT_01 to generate the DID creation operation
	let details = generate_base_did_creation_details::<Test>(alice_did.clone(), ACCOUNT_01);

	let signature = auth_key.sign(details.encode().as_ref());

	let balance = <Test as did::Config>::BaseDeposit::get() * 2
		+ <Test as did::Config>::Fee::get()
		+ <<Test as did::Config>::Currency as Inspect<did::AccountIdOf<Test>>>::minimum_balance();
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, balance)])
		.with_dids(vec![(alice_did, mock_did)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				// Use ACCOUNT_00 to submit the transaction
				Did::create(
					RuntimeOrigin::signed(ACCOUNT_00),
					Box::new(details),
					did::DidSignature::from(signature)
				),
				BadOrigin
			);
		});
}

#[test]
fn create_fail_insufficient_balance() {
	let auth_key = get_sr25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	let details = generate_base_did_creation_details::<Test>(alice_did, ACCOUNT_00);

	let signature = auth_key.sign(details.encode().as_ref());

	ExtBuilder::default().build(None).execute_with(|| {
		assert_noop!(
			Did::create(
				RuntimeOrigin::signed(ACCOUNT_00),
				Box::new(details),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::UnableToPayFees
		);
	});
}

#[test]
fn check_did_already_deleted_creation() {
	let auth_key = get_sr25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	let details = generate_base_did_creation_details::<Test>(alice_did.clone(), ACCOUNT_00);

	let signature = auth_key.sign(details.encode().as_ref());

	let balance = <Test as did::Config>::BaseDeposit::get()
		+ <Test as did::Config>::Fee::get()
		+ <<Test as did::Config>::Currency as Inspect<did::AccountIdOf<Test>>>::minimum_balance();
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, balance)])
		.with_deleted_dids(vec![alice_did])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::create(
					RuntimeOrigin::signed(ACCOUNT_00),
					Box::new(details),
					did::DidSignature::from(signature)
				),
				did::Error::<Test>::AlreadyDeleted
			);
		});
}

#[test]
fn check_invalid_signature_format_did_creation() {
	let auth_key = get_sr25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	// Using an Ed25519 key where an Sr25519 is expected
	let invalid_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	// DID creation contains auth_key, but signature is generated using invalid_key
	let details = generate_base_did_creation_details::<Test>(alice_did, ACCOUNT_00);

	let signature = invalid_key.sign(details.encode().as_ref());

	let balance = <Test as did::Config>::BaseDeposit::get()
		+ <Test as did::Config>::Fee::get()
		+ <<Test as did::Config>::Currency as Inspect<did::AccountIdOf<Test>>>::minimum_balance();
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, balance)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::create(
					RuntimeOrigin::signed(ACCOUNT_00),
					Box::new(details),
					did::DidSignature::from(signature)
				),
				did::Error::<Test>::InvalidSignature
			);
		});
}

#[test]
fn check_invalid_signature_did_creation() {
	let auth_key = get_sr25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	let alternative_key = get_sr25519_authentication_key(&AUTH_SEED_1);
	let details = generate_base_did_creation_details::<Test>(alice_did, ACCOUNT_00);

	let signature = alternative_key.sign(details.encode().as_ref());

	let balance = <Test as did::Config>::BaseDeposit::get()
		+ <Test as did::Config>::Fee::get()
		+ <<Test as did::Config>::Currency as Inspect<did::AccountIdOf<Test>>>::minimum_balance();
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, balance)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::create(
					RuntimeOrigin::signed(ACCOUNT_00),
					Box::new(details),
					did::DidSignature::from(signature)
				),
				did::Error::<Test>::InvalidSignature
			);
		});
}

#[test]
fn check_swapped_did_subject_did_creation() {
	let auth_key = get_sr25519_authentication_key(&AUTH_SEED_0);
	let swapped_key = get_sr25519_authentication_key(&AUTH_SEED_1);
	let swapped_did = get_did_identifier_from_sr25519_key(swapped_key.public());
	let details = generate_base_did_creation_details::<Test>(swapped_did, ACCOUNT_00);

	let signature = auth_key.sign(details.encode().as_ref());

	let balance = <Test as did::Config>::BaseDeposit::get()
		+ <Test as did::Config>::Fee::get()
		+ <<Test as did::Config>::Currency as Inspect<did::AccountIdOf<Test>>>::minimum_balance();
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, balance)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::create(
					RuntimeOrigin::signed(ACCOUNT_00),
					Box::new(details),
					did::DidSignature::from(signature)
				),
				did::Error::<Test>::InvalidSignature
			);
		});
}

#[test]
#[should_panic = "Failed to convert key_agreement_keys to BoundedBTreeSet"]
fn check_max_limit_key_agreement_keys_did_creation() {
	let auth_key = get_sr25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	// Max keys allowed + 1
	let enc_keys = get_key_agreement_keys::<Test>(MaxNewKeyAgreementKeys::get().saturating_add(1));
	let mut details = generate_base_did_creation_details::<Test>(alice_did, ACCOUNT_00);
	details.new_key_agreement_keys = enc_keys;

	let signature = auth_key.sign(details.encode().as_ref());

	let balance = <Test as did::Config>::BaseDeposit::get()
		+ <Test as did::Config>::Fee::get()
		+ <<Test as did::Config>::Currency as Inspect<did::AccountIdOf<Test>>>::minimum_balance();
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, balance)])
		.build(None)
		.execute_with(|| {
			assert_noop!(
				Did::create(
					RuntimeOrigin::signed(ACCOUNT_00),
					Box::new(details),
					did::DidSignature::from(signature)
				),
				did::Error::<Test>::MaxNewKeyAgreementKeysLimitExceeded
			);
		});
}

#[test]
fn check_max_limit_service_endpoints_count_did_creation() {
	let auth_key = get_sr25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	let mut details = generate_base_did_creation_details::<Test>(alice_did, ACCOUNT_00);
	details.new_service_details = get_service_endpoints(
		<Test as did::Config>::MaxNumberOfServicesPerDid::get() + 1,
		1,
		1,
		1,
		1,
		1,
	);

	let signature = auth_key.sign(details.encode().as_ref());
	let required_balance_for_service_endpoints = <Test as did::Config>::ServiceEndpointDeposit::get()
		* (<Test as did::Config>::MaxNumberOfServicesPerDid::get() as u128 + 1);
	let balance = required_balance_for_service_endpoints
		+ <Test as did::Config>::BaseDeposit::get()
		+ <Test as did::Config>::Fee::get()
		+ <<Test as did::Config>::Currency as Inspect<did::AccountIdOf<Test>>>::minimum_balance();
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, balance)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::create(
					RuntimeOrigin::signed(ACCOUNT_00),
					Box::new(details),
					did::DidSignature::from(signature)
				),
				did::Error::<Test>::MaxNumberOfServicesExceeded
			);
		});
}

#[test]
#[should_panic = "Service ID too long."]
fn check_max_limit_service_id_length_did_creation() {
	let auth_key = get_sr25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	let mut details = generate_base_did_creation_details::<Test>(alice_did, ACCOUNT_00);
	details.new_service_details =
		get_service_endpoints(1, <Test as did::Config>::MaxServiceIdLength::get() + 1, 1, 1, 1, 1);

	let signature = auth_key.sign(details.encode().as_ref());

	let balance = <Test as did::Config>::BaseDeposit::get()
		+ <Test as did::Config>::Fee::get()
		+ <<Test as did::Config>::Currency as Inspect<did::AccountIdOf<Test>>>::minimum_balance();
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, balance)])
		.build(None)
		.execute_with(|| {
			assert_noop!(
				Did::create(
					RuntimeOrigin::signed(ACCOUNT_00),
					Box::new(details),
					did::DidSignature::from(signature)
				),
				did::Error::<Test>::MaxServiceIdLengthExceeded
			);
		});
}

#[test]
#[should_panic = "Too many types for the given service."]
fn check_max_limit_service_type_count_did_creation() {
	let auth_key = get_sr25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	let mut details = generate_base_did_creation_details::<Test>(alice_did, ACCOUNT_00);
	details.new_service_details = get_service_endpoints(
		1,
		1,
		<Test as did::Config>::MaxNumberOfTypesPerService::get() + 1,
		1,
		1,
		1,
	);

	let signature = auth_key.sign(details.encode().as_ref());

	let balance = <Test as did::Config>::BaseDeposit::get()
		+ <Test as did::Config>::Fee::get()
		+ <<Test as did::Config>::Currency as Inspect<did::AccountIdOf<Test>>>::minimum_balance();
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, balance)])
		.build(None)
		.execute_with(|| {
			assert_noop!(
				Did::create(
					RuntimeOrigin::signed(ACCOUNT_00),
					Box::new(details),
					did::DidSignature::from(signature)
				),
				did::Error::<Test>::MaxNumberOfTypesPerServiceExceeded
			);
		});
}

#[test]
#[should_panic = "Service type too long."]
fn check_max_limit_service_type_length_did_creation() {
	let auth_key = get_sr25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	let mut details = generate_base_did_creation_details::<Test>(alice_did, ACCOUNT_00);
	details.new_service_details =
		get_service_endpoints(1, 1, 1, <Test as did::Config>::MaxServiceTypeLength::get() + 1, 1, 1);

	let signature = auth_key.sign(details.encode().as_ref());

	let balance = <Test as did::Config>::BaseDeposit::get()
		+ <Test as did::Config>::Fee::get()
		+ <<Test as did::Config>::Currency as Inspect<did::AccountIdOf<Test>>>::minimum_balance();
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, balance)])
		.build(None)
		.execute_with(|| {
			assert_noop!(
				Did::create(
					RuntimeOrigin::signed(ACCOUNT_00),
					Box::new(details),
					did::DidSignature::from(signature)
				),
				did::Error::<Test>::MaxServiceTypeLengthExceeded
			);
		});
}

#[test]
#[should_panic = "Too many URLs for the given service."]
fn check_max_limit_service_url_count_did_creation() {
	let auth_key = get_sr25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	let mut details = generate_base_did_creation_details::<Test>(alice_did, ACCOUNT_00);
	details.new_service_details = get_service_endpoints(
		1,
		1,
		1,
		1,
		<Test as did::Config>::MaxNumberOfUrlsPerService::get() + 1,
		1,
	);

	let signature = auth_key.sign(details.encode().as_ref());

	let balance = <Test as did::Config>::BaseDeposit::get()
		+ <Test as did::Config>::Fee::get()
		+ <<Test as did::Config>::Currency as Inspect<did::AccountIdOf<Test>>>::minimum_balance();
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, balance)])
		.build(None)
		.execute_with(|| {
			assert_noop!(
				Did::create(
					RuntimeOrigin::signed(ACCOUNT_00),
					Box::new(details),
					did::DidSignature::from(signature)
				),
				did::Error::<Test>::MaxNumberOfUrlsPerServiceExceeded
			);
		});
}

#[test]
#[should_panic = "URL too long."]
fn check_max_limit_service_url_length_did_creation() {
	let auth_key = get_sr25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	let mut details = generate_base_did_creation_details::<Test>(alice_did, ACCOUNT_00);
	details.new_service_details =
		get_service_endpoints(1, 1, 1, 1, 1, <Test as did::Config>::MaxServiceUrlLength::get() + 1);

	let signature = auth_key.sign(details.encode().as_ref());

	let balance = <Test as did::Config>::BaseDeposit::get()
		+ <Test as did::Config>::Fee::get()
		+ <<Test as did::Config>::Currency as Inspect<did::AccountIdOf<Test>>>::minimum_balance();
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, balance)])
		.build(None)
		.execute_with(|| {
			assert_noop!(
				Did::create(
					RuntimeOrigin::signed(ACCOUNT_00),
					Box::new(details),
					did::DidSignature::from(signature)
				),
				did::Error::<Test>::MaxServiceUrlLengthExceeded
			);
		});
}

#[test]
fn check_invalid_service_id_character_did_creation() {
	let auth_key = get_sr25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	let new_service_details = DidEndpoint::new("å".bytes().collect(), vec![b"type".to_vec()], vec![b"url".to_vec()]);
	let mut details = generate_base_did_creation_details::<Test>(alice_did, ACCOUNT_00);
	details.new_service_details = vec![new_service_details];

	let signature = auth_key.sign(details.encode().as_ref());

	let balance = <Test as did::Config>::BaseDeposit::get()
		+ <Test as did::Config>::Fee::get()
		+ <Test as did::Config>::ServiceEndpointDeposit::get()
		+ <<Test as did::Config>::Currency as Inspect<did::AccountIdOf<Test>>>::minimum_balance();
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, balance)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::create(
					RuntimeOrigin::signed(ACCOUNT_00),
					Box::new(details),
					did::DidSignature::from(signature)
				),
				did::Error::<Test>::InvalidServiceEncoding
			);
		});
}

#[test]
fn check_invalid_service_type_character_did_creation() {
	let auth_key = get_sr25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	let new_service_details = DidEndpoint::new(b"id".to_vec(), vec!["å".bytes().collect()], vec![b"url".to_vec()]);
	let mut details = generate_base_did_creation_details::<Test>(alice_did, ACCOUNT_00);
	details.new_service_details = vec![new_service_details];

	let signature = auth_key.sign(details.encode().as_ref());

	let balance = <Test as did::Config>::BaseDeposit::get()
		+ <Test as did::Config>::Fee::get()
		+ <Test as did::Config>::ServiceEndpointDeposit::get()
		+ <<Test as did::Config>::Currency as Inspect<did::AccountIdOf<Test>>>::minimum_balance();
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, balance)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::create(
					RuntimeOrigin::signed(ACCOUNT_00),
					Box::new(details),
					did::DidSignature::from(signature)
				),
				did::Error::<Test>::InvalidServiceEncoding
			);
		});
}

#[test]
fn check_invalid_service_url_character_did_creation() {
	let auth_key = get_sr25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	let new_service_details = DidEndpoint::new(b"id".to_vec(), vec![b"type".to_vec()], vec!["å".bytes().collect()]);
	let mut details = generate_base_did_creation_details::<Test>(alice_did, ACCOUNT_00);
	details.new_service_details = vec![new_service_details];

	let signature = auth_key.sign(details.encode().as_ref());

	let balance = <Test as did::Config>::BaseDeposit::get()
		+ <Test as did::Config>::ServiceEndpointDeposit::get()
		+ <Test as did::Config>::Fee::get()
		+ <<Test as did::Config>::Currency as Inspect<did::AccountIdOf<Test>>>::minimum_balance();
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, balance)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::create(
					RuntimeOrigin::signed(ACCOUNT_00),
					Box::new(details),
					did::DidSignature::from(signature)
				),
				did::Error::<Test>::InvalidServiceEncoding
			);
		});
}
