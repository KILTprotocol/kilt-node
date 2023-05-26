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

use frame_support::{assert_err, assert_noop, assert_ok, traits::Currency};
use parity_scale_codec::Encode;
use sp_core::{ed25519, Pair};
use sp_runtime::{
	traits::{BadOrigin, Hash, Zero},
	SaturatedConversion,
};
use sp_std::{
	collections::btree_set::BTreeSet,
	convert::{TryFrom, TryInto},
};

use crate::{
	self as did,
	did_details::{DidEncryptionKey, DidNewKeyAgreementKeySet, DidVerificationKey, DidVerificationKeyRelationship},
	mock::*,
	mock_utils::*,
	service_endpoints::DidEndpoint,
	Error,
};

// create

#[test]
fn check_successful_simple_ed25519_creation() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let auth_did_key = DidVerificationKey::from(auth_key.public());
	let details = generate_base_did_creation_details::<Test>(alice_did.clone(), ACCOUNT_00);

	let signature = auth_key.sign(details.encode().as_ref());

	let balance = <Test as did::Config>::BaseDeposit::get()
		+ <Test as did::Config>::Fee::get()
		+ <<Test as did::Config>::Currency as Currency<did::AccountIdOf<Test>>>::minimum_balance();
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
				Balances::reserved_balance(ACCOUNT_00),
				<Test as did::Config>::BaseDeposit::get()
			);

			assert_eq!(Balances::free_balance(ACCOUNT_FEE), <Test as did::Config>::Fee::get());
		});
}

#[test]
fn check_successful_simple_sr25519_creation() {
	let auth_key = get_sr25519_authentication_key(true);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	let auth_did_key = DidVerificationKey::from(auth_key.public());
	let details = generate_base_did_creation_details::<Test>(alice_did.clone(), ACCOUNT_00);

	let signature = auth_key.sign(details.encode().as_ref());

	let balance = <Test as did::Config>::BaseDeposit::get()
		+ <Test as did::Config>::Fee::get()
		+ <<Test as did::Config>::Currency as Currency<did::AccountIdOf<Test>>>::minimum_balance();
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
				Balances::reserved_balance(ACCOUNT_00),
				<Test as did::Config>::BaseDeposit::get()
			);
			assert_eq!(Balances::free_balance(ACCOUNT_FEE), <Test as did::Config>::Fee::get());
		});
}

#[test]
fn check_successful_simple_ecdsa_creation() {
	let auth_key = get_ecdsa_authentication_key(true);
	let alice_did = get_did_identifier_from_ecdsa_key(auth_key.public());
	let auth_did_key = DidVerificationKey::from(auth_key.public());
	let details = generate_base_did_creation_details::<Test>(alice_did.clone(), ACCOUNT_00);

	let signature = auth_key.sign(details.encode().as_ref());

	let balance = <Test as did::Config>::BaseDeposit::get()
		+ <Test as did::Config>::Fee::get()
		+ <<Test as did::Config>::Currency as Currency<did::AccountIdOf<Test>>>::minimum_balance();
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
				Balances::reserved_balance(ACCOUNT_00),
				<Test as did::Config>::BaseDeposit::get()
			);
			assert_eq!(Balances::free_balance(ACCOUNT_FEE), <Test as did::Config>::Fee::get());
		});
}

#[test]
fn check_successful_complete_creation() {
	let auth_key = get_sr25519_authentication_key(true);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	let auth_did_key = DidVerificationKey::from(auth_key.public());
	let enc_keys = DidNewKeyAgreementKeySet::<Test>::try_from(
		vec![get_x25519_encryption_key(true), get_x25519_encryption_key(false)]
			.iter()
			.copied()
			.collect::<BTreeSet<DidEncryptionKey>>(),
	)
	.expect("Exceeded BoundedBTreeSet bounds when creating new key agreement keys");
	let del_key = get_sr25519_delegation_key(true);
	let att_key = get_ecdsa_attestation_key(true);
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
		+ <<Test as did::Config>::Currency as Currency<did::AccountIdOf<Test>>>::minimum_balance();

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
				Balances::reserved_balance(ACCOUNT_00),
				<Test as did::Config>::BaseDeposit::get()
					+ required_balance_for_endpoint
					+ required_balance_for_key_agreement
					+ required_balance_for_keys
			);
			assert_eq!(Balances::free_balance(ACCOUNT_FEE), <Test as did::Config>::Fee::get());
		});
}

#[test]
fn check_deposit_change_by_adding_service_endpoint() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let mut did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(alice_did.clone()));
	did_details.deposit.amount = <Test as did::Config>::BaseDeposit::get();

	let new_service_endpoint = DidEndpoint::new(b"id".to_vec(), vec![b"type".to_vec()], vec![b"url".to_vec()]);
	let new_service_endpoint2: DidEndpoint<Test> =
		DidEndpoint::new(b"id2".to_vec(), vec![b"type2".to_vec()], vec![b"url2".to_vec()]);

	let balance = <Test as did::Config>::BaseDeposit::get() + <Test as did::Config>::ServiceEndpointDeposit::get();

	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), balance)])
		.with_dids(vec![(alice_did.clone(), did_details)])
		.build(None)
		.execute_with(|| {
			assert_ok!(Did::add_service_endpoint(
				RuntimeOrigin::signed(alice_did.clone()),
				new_service_endpoint.clone()
			));

			assert_eq!(
				Balances::reserved_balance(alice_did.clone()),
				<Test as did::Config>::ServiceEndpointDeposit::get() + <Test as did::Config>::BaseDeposit::get()
			);

			assert_eq!(Balances::free_balance(alice_did.clone()), Zero::zero());

			assert_noop!(
				Did::add_service_endpoint(RuntimeOrigin::signed(alice_did.clone()), new_service_endpoint2.clone()),
				pallet_balances::Error::<Test>::InsufficientBalance
			);

			assert!(did::ServiceEndpoints::<Test>::get(alice_did.clone(), new_service_endpoint.id).is_some());

			assert!(did::ServiceEndpoints::<Test>::get(alice_did, new_service_endpoint2.id).is_none());
		});
}

#[test]
fn check_duplicate_did_creation() {
	let auth_key = get_sr25519_authentication_key(true);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	let auth_did_key = DidVerificationKey::from(auth_key.public());
	let mock_did = generate_base_did_details::<Test>(auth_did_key, None);
	let details = generate_base_did_creation_details::<Test>(alice_did.clone(), ACCOUNT_00);

	let signature = auth_key.sign(details.encode().as_ref());

	let balance = <Test as did::Config>::BaseDeposit::get() * 20
		+ <Test as did::Config>::Fee::get()
		+ <<Test as did::Config>::Currency as Currency<did::AccountIdOf<Test>>>::minimum_balance();
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
	let auth_key = get_sr25519_authentication_key(true);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	let auth_did_key = DidVerificationKey::from(auth_key.public());
	let mock_did = generate_base_did_details::<Test>(auth_did_key, None);
	// Use ACCOUNT_01 to generate the DID creation operation
	let details = generate_base_did_creation_details::<Test>(alice_did.clone(), ACCOUNT_01);

	let signature = auth_key.sign(details.encode().as_ref());

	let balance = <Test as did::Config>::BaseDeposit::get()
		+ <Test as did::Config>::Fee::get()
		+ <<Test as did::Config>::Currency as Currency<did::AccountIdOf<Test>>>::minimum_balance();
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
	let auth_key = get_sr25519_authentication_key(true);
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
	let auth_key = get_sr25519_authentication_key(true);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	let details = generate_base_did_creation_details::<Test>(alice_did.clone(), ACCOUNT_00);

	let signature = auth_key.sign(details.encode().as_ref());

	let balance = <Test as did::Config>::BaseDeposit::get()
		+ <Test as did::Config>::Fee::get()
		+ <<Test as did::Config>::Currency as Currency<did::AccountIdOf<Test>>>::minimum_balance();
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
	let auth_key = get_sr25519_authentication_key(true);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	// Using an Ed25519 key where an Sr25519 is expected
	let invalid_key = get_ed25519_authentication_key(true);
	// DID creation contains auth_key, but signature is generated using invalid_key
	let details = generate_base_did_creation_details::<Test>(alice_did, ACCOUNT_00);

	let signature = invalid_key.sign(details.encode().as_ref());

	let balance = <Test as did::Config>::BaseDeposit::get()
		+ <Test as did::Config>::Fee::get()
		+ <<Test as did::Config>::Currency as Currency<did::AccountIdOf<Test>>>::minimum_balance();
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
	let auth_key = get_sr25519_authentication_key(true);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	let alternative_key = get_sr25519_authentication_key(false);
	let details = generate_base_did_creation_details::<Test>(alice_did, ACCOUNT_00);

	let signature = alternative_key.sign(details.encode().as_ref());

	let balance = <Test as did::Config>::BaseDeposit::get()
		+ <Test as did::Config>::Fee::get()
		+ <<Test as did::Config>::Currency as Currency<did::AccountIdOf<Test>>>::minimum_balance();
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
	let auth_key = get_sr25519_authentication_key(true);
	let swapped_key = get_sr25519_authentication_key(false);
	let swapped_did = get_did_identifier_from_sr25519_key(swapped_key.public());
	let details = generate_base_did_creation_details::<Test>(swapped_did, ACCOUNT_00);

	let signature = auth_key.sign(details.encode().as_ref());

	let balance = <Test as did::Config>::BaseDeposit::get()
		+ <Test as did::Config>::Fee::get()
		+ <<Test as did::Config>::Currency as Currency<did::AccountIdOf<Test>>>::minimum_balance();
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
	let auth_key = get_sr25519_authentication_key(true);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	// Max keys allowed + 1
	let enc_keys = get_key_agreement_keys::<Test>(MaxNewKeyAgreementKeys::get().saturating_add(1));
	let mut details = generate_base_did_creation_details::<Test>(alice_did, ACCOUNT_00);
	details.new_key_agreement_keys = enc_keys;

	let signature = auth_key.sign(details.encode().as_ref());

	let balance = <Test as did::Config>::BaseDeposit::get()
		+ <Test as did::Config>::Fee::get()
		+ <<Test as did::Config>::Currency as Currency<did::AccountIdOf<Test>>>::minimum_balance();
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
	let auth_key = get_sr25519_authentication_key(true);
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
		+ <<Test as did::Config>::Currency as Currency<did::AccountIdOf<Test>>>::minimum_balance();
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
	let auth_key = get_sr25519_authentication_key(true);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	let mut details = generate_base_did_creation_details::<Test>(alice_did, ACCOUNT_00);
	details.new_service_details =
		get_service_endpoints(1, <Test as did::Config>::MaxServiceIdLength::get() + 1, 1, 1, 1, 1);

	let signature = auth_key.sign(details.encode().as_ref());

	let balance = <Test as did::Config>::BaseDeposit::get()
		+ <Test as did::Config>::Fee::get()
		+ <<Test as did::Config>::Currency as Currency<did::AccountIdOf<Test>>>::minimum_balance();
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
	let auth_key = get_sr25519_authentication_key(true);
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
		+ <<Test as did::Config>::Currency as Currency<did::AccountIdOf<Test>>>::minimum_balance();
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
	let auth_key = get_sr25519_authentication_key(true);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	let mut details = generate_base_did_creation_details::<Test>(alice_did, ACCOUNT_00);
	details.new_service_details =
		get_service_endpoints(1, 1, 1, <Test as did::Config>::MaxServiceTypeLength::get() + 1, 1, 1);

	let signature = auth_key.sign(details.encode().as_ref());

	let balance = <Test as did::Config>::BaseDeposit::get()
		+ <Test as did::Config>::Fee::get()
		+ <<Test as did::Config>::Currency as Currency<did::AccountIdOf<Test>>>::minimum_balance();
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
	let auth_key = get_sr25519_authentication_key(true);
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
		+ <<Test as did::Config>::Currency as Currency<did::AccountIdOf<Test>>>::minimum_balance();
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
	let auth_key = get_sr25519_authentication_key(true);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	let mut details = generate_base_did_creation_details::<Test>(alice_did, ACCOUNT_00);
	details.new_service_details =
		get_service_endpoints(1, 1, 1, 1, 1, <Test as did::Config>::MaxServiceUrlLength::get() + 1);

	let signature = auth_key.sign(details.encode().as_ref());

	let balance = <Test as did::Config>::BaseDeposit::get()
		+ <Test as did::Config>::Fee::get()
		+ <<Test as did::Config>::Currency as Currency<did::AccountIdOf<Test>>>::minimum_balance();
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
	let auth_key = get_sr25519_authentication_key(true);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	let new_service_details = DidEndpoint::new("å".bytes().collect(), vec![b"type".to_vec()], vec![b"url".to_vec()]);
	let mut details = generate_base_did_creation_details::<Test>(alice_did, ACCOUNT_00);
	details.new_service_details = vec![new_service_details];

	let signature = auth_key.sign(details.encode().as_ref());

	let balance = <Test as did::Config>::BaseDeposit::get()
		+ <Test as did::Config>::Fee::get()
		+ <Test as did::Config>::ServiceEndpointDeposit::get()
		+ <<Test as did::Config>::Currency as Currency<did::AccountIdOf<Test>>>::minimum_balance();
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
	let auth_key = get_sr25519_authentication_key(true);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	let new_service_details = DidEndpoint::new(b"id".to_vec(), vec!["å".bytes().collect()], vec![b"url".to_vec()]);
	let mut details = generate_base_did_creation_details::<Test>(alice_did, ACCOUNT_00);
	details.new_service_details = vec![new_service_details];

	let signature = auth_key.sign(details.encode().as_ref());

	let balance = <Test as did::Config>::BaseDeposit::get()
		+ <Test as did::Config>::Fee::get()
		+ <Test as did::Config>::ServiceEndpointDeposit::get()
		+ <<Test as did::Config>::Currency as Currency<did::AccountIdOf<Test>>>::minimum_balance();
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
	let auth_key = get_sr25519_authentication_key(true);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	let new_service_details = DidEndpoint::new(b"id".to_vec(), vec![b"type".to_vec()], vec!["å".bytes().collect()]);
	let mut details = generate_base_did_creation_details::<Test>(alice_did, ACCOUNT_00);
	details.new_service_details = vec![new_service_details];

	let signature = auth_key.sign(details.encode().as_ref());

	let balance = <Test as did::Config>::BaseDeposit::get()
		+ <Test as did::Config>::ServiceEndpointDeposit::get()
		+ <Test as did::Config>::Fee::get()
		+ <<Test as did::Config>::Currency as Currency<did::AccountIdOf<Test>>>::minimum_balance();
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

// updates

#[test]
fn check_successful_authentication_key_update() {
	let old_auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(old_auth_key.public());
	let new_auth_key = get_ed25519_authentication_key(false);

	let old_did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(old_auth_key.public()), Some(alice_did.clone()));

	let new_block_number: BlockNumber = 1;

	// Update authentication key. The old one should be removed.
	ExtBuilder::default()
		.with_dids(vec![(alice_did.clone(), old_did_details)])
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.build_and_execute_with_sanity_tests(None, || {
			System::set_block_number(new_block_number);
			assert_ok!(Did::set_authentication_key(
				RuntimeOrigin::signed(alice_did.clone()),
				DidVerificationKey::from(new_auth_key.public())
			));
			let new_did_details = Did::get_did(&alice_did).expect("ALICE_DID should be present on chain.");
			assert_eq!(
				new_did_details.authentication_key,
				generate_key_id(&DidVerificationKey::from(new_auth_key.public()).into())
			);
			let public_keys = new_did_details.public_keys;
			// Total is +1 for the new auth key, -1 for the old auth key (replaced) = 1
			assert_eq!(public_keys.len(), 1);
			// Check for new authentication key
			assert!(public_keys.contains_key(&generate_key_id(
				&DidVerificationKey::from(new_auth_key.public()).into()
			)));
		});
}

#[test]
fn check_successful_authentication_key_max_public_keys_update() {
	let old_auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(old_auth_key.public());
	let new_auth_key = get_ed25519_authentication_key(false);
	let key_agreement_keys = get_key_agreement_keys::<Test>(MaxTotalKeyAgreementKeys::get());

	let mut did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(old_auth_key.public()), Some(alice_did.clone()));
	assert_ok!(did_details.add_key_agreement_keys(key_agreement_keys, 0u64,));

	// Fill public key map to its max by adding
	// MaxPublicKeysPerDid - MaxTotalKeyAgreementKeys many keys
	did_details = fill_public_keys(did_details);

	let new_block_number: BlockNumber = 1;

	// Update authentication key. The old one should be removed.
	ExtBuilder::default()
		.with_dids(vec![(alice_did.clone(), did_details)])
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.build_and_execute_with_sanity_tests(None, || {
			System::set_block_number(new_block_number);
			assert_ok!(Did::set_authentication_key(
				RuntimeOrigin::signed(alice_did.clone()),
				DidVerificationKey::from(new_auth_key.public())
			));

			let new_did_details = Did::get_did(&alice_did).expect("ALICE_DID should be present on chain.");
			assert_eq!(
				new_did_details.authentication_key,
				generate_key_id(&DidVerificationKey::from(new_auth_key.public()).into())
			);
			let public_keys = new_did_details.public_keys;
			// Total is the maximum allowed
			assert_eq!(public_keys.len(), MaxPublicKeysPerDid::get().saturated_into::<usize>());
			// Check for new authentication key
			assert!(public_keys.contains_key(&generate_key_id(
				&DidVerificationKey::from(new_auth_key.public()).into()
			)));
		});
}

#[test]
fn check_reused_key_authentication_key_update() {
	let old_auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(old_auth_key.public());
	let old_delegation_key = old_auth_key;
	let new_auth_key = get_ed25519_authentication_key(false);

	let mut old_did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(old_auth_key.public()), Some(alice_did.clone()));
	// Same key for auth and del key
	assert_ok!(old_did_details.update_delegation_key(DidVerificationKey::from(old_delegation_key.public()), 0u64));

	let new_block_number: BlockNumber = 1;

	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(alice_did.clone(), old_did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			System::set_block_number(new_block_number);
			assert_ok!(Did::set_authentication_key(
				RuntimeOrigin::signed(alice_did.clone()),
				DidVerificationKey::from(new_auth_key.public())
			));

			let new_did_details = Did::get_did(&alice_did).expect("ALICE_DID should be present on chain.");
			assert_eq!(
				new_did_details.authentication_key,
				generate_key_id(&DidVerificationKey::from(new_auth_key.public()).into())
			);
			let public_keys = new_did_details.public_keys;
			// Total is +1 for the new auth key (the old key is still used as delegation
			// key, so it is not removed)
			assert_eq!(public_keys.len(), 2);
			// Check for new authentication key
			assert!(public_keys.contains_key(&generate_key_id(
				&DidVerificationKey::from(new_auth_key.public()).into()
			)));
			// Check for old authentication key (delegation key)
			assert!(public_keys.contains_key(&generate_key_id(
				&DidVerificationKey::from(old_auth_key.public()).into()
			)));
		});
}

#[test]
fn check_max_keys_authentication_key_update_error() {
	let old_auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(old_auth_key.public());
	let delegation_key = old_auth_key;
	let new_auth_key = get_ed25519_authentication_key(false);
	let key_agreement_keys = get_key_agreement_keys::<Test>(MaxTotalKeyAgreementKeys::get());

	let mut did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(old_auth_key.public()), Some(alice_did.clone()));
	assert_ok!(did_details.add_key_agreement_keys(key_agreement_keys, 0u64,));
	assert_ok!(did_details.update_delegation_key(DidVerificationKey::from(delegation_key.public()), 0u64));

	// Fill public key map to its max by adding
	// MaxPublicKeysPerDid - MaxTotalKeyAgreementKeys many keys
	did_details = fill_public_keys(did_details);

	let new_block_number: BlockNumber = 1;

	// Update authentication key. Since the old one is not removed because it is the
	// same as the delegation key, the update should fail as the max number of
	// public keys is already present.
	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(alice_did.clone(), did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			System::set_block_number(new_block_number);
			assert_noop!(
				Did::set_authentication_key(
					RuntimeOrigin::signed(alice_did.clone()),
					DidVerificationKey::from(new_auth_key.public())
				),
				did::Error::<Test>::MaxPublicKeysExceeded
			);
		});
}

#[test]
fn check_did_not_present_authentication_key_update_error() {
	let old_auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(old_auth_key.public());
	let new_auth_key = get_ed25519_authentication_key(false);

	let new_block_number: BlockNumber = 1;

	// Update authentication key. The old one should be removed.
	ExtBuilder::default().build(None).execute_with(|| {
		System::set_block_number(new_block_number);
		assert_noop!(
			Did::set_authentication_key(
				RuntimeOrigin::signed(alice_did.clone()),
				DidVerificationKey::from(new_auth_key.public())
			),
			did::Error::<Test>::NotFound
		);
	});
}

#[test]
fn check_successful_delegation_key_update() {
	let auth_key = get_ed25519_authentication_key(true);
	let old_del_key = get_sr25519_delegation_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let new_del_key = get_sr25519_delegation_key(false);

	let mut old_did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(alice_did.clone()));
	assert_ok!(old_did_details.update_delegation_key(DidVerificationKey::from(old_del_key.public()), 0u64));

	let new_block_number: BlockNumber = 1;

	// Update delegation key. The old one should be removed.
	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(alice_did.clone(), old_did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			System::set_block_number(new_block_number);
			assert_ok!(Did::set_delegation_key(
				RuntimeOrigin::signed(alice_did.clone()),
				DidVerificationKey::from(new_del_key.public())
			));

			let new_did_details = Did::get_did(&alice_did).expect("ALICE_DID should be present on chain.");
			assert_eq!(
				new_did_details.delegation_key,
				Some(generate_key_id(&DidVerificationKey::from(new_del_key.public()).into()))
			);
			let public_keys = new_did_details.public_keys;
			// Total is +1 for the new del key, -1 for the old del key (replaced) + auth key
			// = 2
			assert_eq!(public_keys.len(), 2);
			// Check for new delegation key
			assert!(public_keys.contains_key(&generate_key_id(&DidVerificationKey::from(new_del_key.public()).into())));
		});
}

#[test]
fn check_successful_delegation_key_max_public_keys_update() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let old_del_key = get_sr25519_delegation_key(true);
	let new_del_key = get_sr25519_delegation_key(false);
	let key_agreement_keys = get_key_agreement_keys::<Test>(MaxTotalKeyAgreementKeys::get());

	let mut did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(alice_did.clone()));
	assert_ok!(did_details.add_key_agreement_keys(key_agreement_keys, 0u64,));
	assert_ok!(did_details.update_delegation_key(DidVerificationKey::from(old_del_key.public()), 0u64));

	// Fill public key map to its max by adding
	// MaxPublicKeysPerDid - MaxTotalKeyAgreementKeys many keys
	did_details = fill_public_keys(did_details);

	let new_block_number: BlockNumber = 1;

	// Update delegation key. The old one should be removed.
	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(alice_did.clone(), did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			System::set_block_number(new_block_number);
			assert_ok!(Did::set_delegation_key(
				RuntimeOrigin::signed(alice_did.clone()),
				DidVerificationKey::from(new_del_key.public())
			));

			let new_did_details = Did::get_did(&alice_did).expect("ALICE_DID should be present on chain.");
			assert_eq!(
				new_did_details.delegation_key,
				Some(generate_key_id(&DidVerificationKey::from(new_del_key.public()).into()))
			);
			let public_keys = new_did_details.public_keys;
			// Total is the maximum allowed
			assert_eq!(public_keys.len(), MaxPublicKeysPerDid::get().saturated_into::<usize>());
			// Check for new delegation key
			assert!(public_keys.contains_key(&generate_key_id(&DidVerificationKey::from(new_del_key.public()).into())));
		});
}

#[test]
fn check_reused_key_delegation_key_update() {
	let old_auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(old_auth_key.public());
	let old_del_key = old_auth_key;
	let new_del_key = get_sr25519_delegation_key(true);

	let mut old_did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(old_auth_key.public()), Some(alice_did.clone()));
	// Same key for auth and del key
	assert_ok!(old_did_details.update_delegation_key(DidVerificationKey::from(old_del_key.public()), 0u64));

	let new_block_number: BlockNumber = 1;

	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(alice_did.clone(), old_did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			System::set_block_number(new_block_number);
			assert_ok!(Did::set_delegation_key(
				RuntimeOrigin::signed(alice_did.clone()),
				DidVerificationKey::from(new_del_key.public())
			));

			let new_did_details = Did::get_did(&alice_did).expect("ALICE_DID should be present on chain.");
			assert_eq!(
				new_did_details.delegation_key,
				Some(generate_key_id(&DidVerificationKey::from(new_del_key.public()).into()))
			);
			let public_keys = new_did_details.public_keys;
			// Total is +1 for the new del key (the old key is still used as authentication
			// key, so it is not removed)
			assert_eq!(public_keys.len(), 2);
			// Check for new delegation key
			assert!(public_keys.contains_key(&generate_key_id(&DidVerificationKey::from(new_del_key.public()).into())));
			// Check for old delegation key (authentication key)
			assert!(public_keys.contains_key(&generate_key_id(&DidVerificationKey::from(old_del_key.public()).into())));
		});
}

#[test]
fn check_max_public_keys_delegation_key_addition_error() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let new_del_key = get_sr25519_delegation_key(false);
	let key_agreement_keys = get_key_agreement_keys::<Test>(MaxTotalKeyAgreementKeys::get());

	let mut did_details = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);
	assert_ok!(did_details.add_key_agreement_keys(key_agreement_keys, 0u64,));

	// Fill public key map to its max by adding
	// MaxPublicKeysPerDid - MaxTotalKeyAgreementKeys many keys
	did_details = fill_public_keys(did_details);

	let new_block_number: BlockNumber = 1;

	// Update delegation key. The old one should be removed.
	ExtBuilder::default()
		.with_dids(vec![(alice_did.clone(), did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			System::set_block_number(new_block_number);
			assert_noop!(
				Did::set_delegation_key(
					RuntimeOrigin::signed(alice_did.clone()),
					DidVerificationKey::from(new_del_key.public())
				),
				did::Error::<Test>::MaxPublicKeysExceeded
			);
		});
}

#[test]
fn check_max_public_keys_reused_key_delegation_key_update_error() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let old_del_key = auth_key;
	let new_del_key = get_sr25519_delegation_key(true);
	let key_agreement_keys = get_key_agreement_keys::<Test>(MaxTotalKeyAgreementKeys::get());

	let mut did_details = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);
	assert_ok!(did_details.add_key_agreement_keys(key_agreement_keys, 0u64,));
	// Same key for auth and delegation
	assert_ok!(did_details.update_delegation_key(DidVerificationKey::from(old_del_key.public()), 0u64));

	// Fill public key map to its max by adding
	// MaxPublicKeysPerDid - MaxTotalKeyAgreementKeys many keys
	did_details = fill_public_keys(did_details);

	let new_block_number: BlockNumber = 1;

	// Update delegation key. The old one should not be removed as it is still used
	// as authentication key.
	ExtBuilder::default()
		.with_dids(vec![(alice_did.clone(), did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			System::set_block_number(new_block_number);
			assert_noop!(
				Did::set_delegation_key(
					RuntimeOrigin::signed(alice_did.clone()),
					DidVerificationKey::from(new_del_key.public())
				),
				did::Error::<Test>::MaxPublicKeysExceeded
			);
		});
}

#[test]
fn check_did_not_present_delegation_key_update_error() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let new_del_key = get_sr25519_delegation_key(false);

	let new_block_number: BlockNumber = 1;

	// Update delegation key. The old one should be removed.
	ExtBuilder::default().build(None).execute_with(|| {
		System::set_block_number(new_block_number);
		assert_noop!(
			Did::set_delegation_key(
				RuntimeOrigin::signed(alice_did.clone()),
				DidVerificationKey::from(new_del_key.public())
			),
			did::Error::<Test>::NotFound
		);
	});
}

#[test]
fn check_successful_delegation_key_deletion() {
	let auth_key = get_ed25519_authentication_key(true);
	let old_del_key = get_sr25519_delegation_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());

	let mut old_did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(alice_did.clone()));
	assert_ok!(old_did_details.update_delegation_key(DidVerificationKey::from(old_del_key.public()), 0u64));

	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(alice_did.clone(), old_did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_ok!(Did::remove_delegation_key(RuntimeOrigin::signed(alice_did.clone())));

			let new_did_details = Did::get_did(&alice_did).expect("ALICE_DID should be present on chain.");
			assert!(new_did_details.delegation_key.is_none());
			let public_keys = new_did_details.public_keys;
			// Total is -1 for the removal + auth key = 1
			assert_eq!(public_keys.len(), 1);
			// Check for new delegation key
			assert!(!public_keys.contains_key(&generate_key_id(&DidVerificationKey::from(old_del_key.public()).into())));
		});
}

#[test]
fn check_successful_reused_delegation_key_deletion() {
	let auth_key = get_ed25519_authentication_key(true);
	let old_del_key = auth_key;
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());

	let mut old_did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(alice_did.clone()));
	assert_ok!(old_did_details.update_delegation_key(DidVerificationKey::from(old_del_key.public()), 0u64));

	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(alice_did.clone(), old_did_details.clone())])
		.build_and_execute_with_sanity_tests(None, || {
			assert_ok!(Did::remove_delegation_key(RuntimeOrigin::signed(alice_did.clone())));

			let new_did_details = Did::get_did(&alice_did).expect("ALICE_DID should be present on chain.");
			assert!(new_did_details.delegation_key.is_none());
			let public_keys = new_did_details.public_keys;
			// Total should be unchanged as the key was re-used so it is not completely
			// deleted
			assert_eq!(public_keys.len(), old_did_details.public_keys.len());
			// Check for presence of old delegation key (authentication key)
			assert!(public_keys.contains_key(&generate_key_id(&DidVerificationKey::from(old_del_key.public()).into())));
		});
}

#[test]
fn check_did_not_present_delegation_key_deletion_error() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());

	ExtBuilder::default().build(None).execute_with(|| {
		assert_noop!(
			Did::remove_delegation_key(RuntimeOrigin::signed(alice_did.clone())),
			did::Error::<Test>::NotFound
		);
	});
}

#[test]
fn check_key_not_present_delegation_key_deletion_error() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());

	let old_did_details = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);

	ExtBuilder::default()
		.with_dids(vec![(alice_did.clone(), old_did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::remove_delegation_key(RuntimeOrigin::signed(alice_did.clone())),
				did::Error::<Test>::VerificationKeyNotFound
			);
		});
}

#[test]
fn check_successful_attestation_key_update() {
	let auth_key = get_ed25519_authentication_key(true);
	let old_att_key = get_sr25519_attestation_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let new_att_key = get_sr25519_attestation_key(false);

	let mut old_did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(alice_did.clone()));
	assert_ok!(old_did_details.update_attestation_key(DidVerificationKey::from(old_att_key.public()), 0u64));

	let new_block_number: BlockNumber = 1;

	// Update attestation key. The old one should be removed.
	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(alice_did.clone(), old_did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			System::set_block_number(new_block_number);
			assert_ok!(Did::set_attestation_key(
				RuntimeOrigin::signed(alice_did.clone()),
				DidVerificationKey::from(new_att_key.public())
			));
			let new_did_details = Did::get_did(&alice_did).expect("ALICE_DID should be present on chain.");
			assert_eq!(
				new_did_details.attestation_key,
				Some(generate_key_id(&DidVerificationKey::from(new_att_key.public()).into()))
			);
			let public_keys = new_did_details.public_keys;
			// Total is +1 for the new att key, -1 for the old att key (replaced) + auth key
			// = 2
			assert_eq!(public_keys.len(), 2);
			// Check for new attestation key
			assert!(public_keys.contains_key(&generate_key_id(&DidVerificationKey::from(new_att_key.public()).into())));
		});
}

#[test]
fn check_successful_attestation_key_max_public_keys_update() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let old_att_key = get_sr25519_attestation_key(true);
	let key_agreement_keys = get_key_agreement_keys::<Test>(MaxTotalKeyAgreementKeys::get());
	let new_att_key = get_sr25519_attestation_key(false);

	let mut did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(alice_did.clone()));
	assert_ok!(did_details.add_key_agreement_keys(key_agreement_keys, 0u64,));
	assert_ok!(did_details.update_attestation_key(DidVerificationKey::from(old_att_key.public()), 0u64));

	// Fill public key map to its max by adding
	// MaxPublicKeysPerDid - MaxTotalKeyAgreementKeys many keys
	did_details = fill_public_keys(did_details);

	let new_block_number: BlockNumber = 1;

	// Update attestation key. The old one should be removed.
	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(alice_did.clone(), did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			System::set_block_number(new_block_number);
			assert_ok!(Did::set_attestation_key(
				RuntimeOrigin::signed(alice_did.clone()),
				DidVerificationKey::from(new_att_key.public())
			));
			let new_did_details = Did::get_did(&alice_did).expect("ALICE_DID should be present on chain.");
			assert_eq!(
				new_did_details.attestation_key,
				Some(generate_key_id(&DidVerificationKey::from(new_att_key.public()).into()))
			);
			let public_keys = new_did_details.public_keys;
			// Total is the maximum allowed
			assert_eq!(public_keys.len(), MaxPublicKeysPerDid::get().saturated_into::<usize>());
			// Check for new attestation key
			assert!(public_keys.contains_key(&generate_key_id(&DidVerificationKey::from(new_att_key.public()).into())));
		});
}

#[test]
fn check_reused_key_attestation_key_update() {
	let old_auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(old_auth_key.public());
	let old_att_key = old_auth_key;
	let new_att_key = get_sr25519_attestation_key(true);

	let mut old_did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(old_auth_key.public()), Some(alice_did.clone()));
	// Same key for auth and att key
	assert_ok!(old_did_details.update_attestation_key(DidVerificationKey::from(old_att_key.public()), 0u64));

	let new_block_number: BlockNumber = 1;

	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(alice_did.clone(), old_did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			System::set_block_number(new_block_number);
			assert_ok!(Did::set_attestation_key(
				RuntimeOrigin::signed(alice_did.clone()),
				DidVerificationKey::from(new_att_key.public())
			));

			let new_did_details = Did::get_did(&alice_did).expect("ALICE_DID should be present on chain.");
			assert_eq!(
				new_did_details.attestation_key,
				Some(generate_key_id(&DidVerificationKey::from(new_att_key.public()).into()))
			);
			let public_keys = new_did_details.public_keys;
			// Total is +1 for the new att key (the old key is still used as authentication
			// key, so it is not removed)
			assert_eq!(public_keys.len(), 2);
			// Check for new attestation key
			assert!(public_keys.contains_key(&generate_key_id(&DidVerificationKey::from(new_att_key.public()).into())));
			// Check for old attestation key (authentication key)
			assert!(public_keys.contains_key(&generate_key_id(&DidVerificationKey::from(old_att_key.public()).into())));
		});
}

#[test]
fn check_max_public_keys_attestation_key_addition_error() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let new_att_key = get_sr25519_attestation_key(false);
	let key_agreement_keys = get_key_agreement_keys::<Test>(MaxTotalKeyAgreementKeys::get());

	let mut did_details = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);
	assert_ok!(did_details.add_key_agreement_keys(key_agreement_keys, 0u64,));

	// Fill public key map to its max by adding
	// MaxPublicKeysPerDid - MaxTotalKeyAgreementKeys many keys
	did_details = fill_public_keys(did_details);

	let new_block_number: BlockNumber = 1;

	// Update attestation key. The old one should be removed.
	ExtBuilder::default()
		.with_dids(vec![(alice_did.clone(), did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			System::set_block_number(new_block_number);
			assert_noop!(
				Did::set_attestation_key(
					RuntimeOrigin::signed(alice_did.clone()),
					DidVerificationKey::from(new_att_key.public())
				),
				did::Error::<Test>::MaxPublicKeysExceeded
			);
		});
}

#[test]
fn check_max_public_keys_reused_key_attestation_key_update_error() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let old_att_key = auth_key;
	let new_att_key = get_sr25519_delegation_key(true);
	let key_agreement_keys = get_key_agreement_keys::<Test>(MaxTotalKeyAgreementKeys::get());

	let mut did_details = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);
	assert_ok!(did_details.add_key_agreement_keys(key_agreement_keys, 0u64,));
	// Same key for auth and attestation
	assert_ok!(did_details.update_attestation_key(DidVerificationKey::from(old_att_key.public()), 0u64));

	// Fill public key map to its max by adding
	// MaxPublicKeysPerDid - MaxTotalKeyAgreementKeys many keys
	did_details = fill_public_keys(did_details);

	let new_block_number: BlockNumber = 1;

	// Update attestation key. The old one should not be removed as it is still used
	// as authentication key.
	ExtBuilder::default()
		.with_dids(vec![(alice_did.clone(), did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			System::set_block_number(new_block_number);
			assert_noop!(
				Did::set_attestation_key(
					RuntimeOrigin::signed(alice_did.clone()),
					DidVerificationKey::from(new_att_key.public())
				),
				did::Error::<Test>::MaxPublicKeysExceeded
			);
		});
}

#[test]
fn check_did_not_present_attestation_key_update_error() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let new_att_key = get_sr25519_delegation_key(false);

	let new_block_number: BlockNumber = 1;

	// Update delegation key. The old one should be removed.
	ExtBuilder::default().build(None).execute_with(|| {
		System::set_block_number(new_block_number);
		assert_noop!(
			Did::set_delegation_key(
				RuntimeOrigin::signed(alice_did.clone()),
				DidVerificationKey::from(new_att_key.public())
			),
			did::Error::<Test>::NotFound
		);
	});
}

#[test]
fn check_successful_attestation_key_deletion() {
	let auth_key = get_ed25519_authentication_key(true);
	let old_att_key = get_sr25519_attestation_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());

	let mut old_did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(alice_did.clone()));
	assert_ok!(old_did_details.update_attestation_key(DidVerificationKey::from(old_att_key.public()), 0u64));

	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(alice_did.clone(), old_did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_ok!(Did::remove_attestation_key(RuntimeOrigin::signed(alice_did.clone())));

			let new_did_details = Did::get_did(&alice_did).expect("ALICE_DID should be present on chain.");
			assert!(new_did_details.attestation_key.is_none());
			let public_keys = new_did_details.public_keys;
			// Total is -1 for the removal + auth key = 1
			assert_eq!(public_keys.len(), 1);
			// Check for new attestation key
			assert!(!public_keys.contains_key(&generate_key_id(&DidVerificationKey::from(old_att_key.public()).into())));
		});
}

#[test]
fn check_successful_reused_attestation_key_deletion() {
	let auth_key = get_ed25519_authentication_key(true);
	let old_att_key = auth_key;
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());

	let mut old_did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(alice_did.clone()));
	assert_ok!(old_did_details.update_attestation_key(DidVerificationKey::from(old_att_key.public()), 0u64));

	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(alice_did.clone(), old_did_details.clone())])
		.build_and_execute_with_sanity_tests(None, || {
			assert_ok!(Did::remove_attestation_key(RuntimeOrigin::signed(alice_did.clone())));
			let new_did_details = Did::get_did(&alice_did).expect("ALICE_DID should be present on chain.");
			assert!(new_did_details.attestation_key.is_none());
			let public_keys = new_did_details.public_keys;
			// Total should be unchanged as the key was re-used so it is not completely
			// deleted
			assert_eq!(public_keys.len(), old_did_details.public_keys.len());
			// Check for presence of old delegation key (authentication key)
			assert!(public_keys.contains_key(&generate_key_id(&DidVerificationKey::from(old_att_key.public()).into())));
		});
}

#[test]
fn check_did_not_present_attestation_key_deletion_error() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());

	ExtBuilder::default().build(None).execute_with(|| {
		assert_noop!(
			Did::remove_attestation_key(RuntimeOrigin::signed(alice_did.clone())),
			did::Error::<Test>::NotFound
		);
	});
}

#[test]
fn check_key_not_present_attestation_key_deletion_error() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());

	let old_did_details = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);

	ExtBuilder::default()
		.with_dids(vec![(alice_did.clone(), old_did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::remove_attestation_key(RuntimeOrigin::signed(alice_did.clone())),
				did::Error::<Test>::VerificationKeyNotFound
			);
		});
}

#[test]
fn check_successful_key_agreement_key_addition() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let new_key_agreement_key = get_x25519_encryption_key(true);

	let old_did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(alice_did.clone()));

	let new_block_number: BlockNumber = 1;

	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(alice_did.clone(), old_did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			System::set_block_number(new_block_number);
			assert_ok!(Did::add_key_agreement_key(
				RuntimeOrigin::signed(alice_did.clone()),
				new_key_agreement_key,
			));
			let new_did_details = Did::get_did(&alice_did).expect("ALICE_DID should be present on chain.");
			assert_eq!(new_did_details.key_agreement_keys.len(), 1);
			assert_eq!(
				new_did_details.key_agreement_keys.iter().next().unwrap(),
				&generate_key_id(&new_key_agreement_key.into())
			);
			let public_keys = new_did_details.public_keys;
			// Total is +1 for the new enc key + auth key = 2
			assert_eq!(public_keys.len(), 2);
			// Check for new key agreement key
			assert!(public_keys.contains_key(&generate_key_id(&new_key_agreement_key.into())));
		});
}

#[test]
fn check_max_public_keys_key_agreement_key_addition_error() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());

	let key_agreement_keys = get_key_agreement_keys::<Test>(MaxTotalKeyAgreementKeys::get());
	let new_key_agreement_key = get_x25519_encryption_key(true);

	let mut did_details = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);
	assert_ok!(did_details.add_key_agreement_keys(key_agreement_keys, 0u64,));

	// Fill public key map to its max by adding
	// MaxPublicKeysPerDid - MaxTotalKeyAgreementKeys many keys
	did_details = fill_public_keys(did_details);

	let new_block_number: BlockNumber = 1;

	ExtBuilder::default()
		.with_dids(vec![(alice_did.clone(), did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			System::set_block_number(new_block_number);
			assert_noop!(
				Did::add_key_agreement_key(RuntimeOrigin::signed(alice_did.clone()), new_key_agreement_key,),
				did::Error::<Test>::MaxPublicKeysExceeded
			);
		});
}

#[test]
fn check_did_not_present_key_agreement_key_addition_error() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let new_enc_key = get_x25519_encryption_key(true);

	let new_block_number: BlockNumber = 1;

	// Update delegation key. The old one should be removed.
	ExtBuilder::default().build(None).execute_with(|| {
		System::set_block_number(new_block_number);
		assert_noop!(
			Did::add_key_agreement_key(RuntimeOrigin::signed(alice_did.clone()), new_enc_key),
			did::Error::<Test>::NotFound
		);
	});
}

#[test]
fn check_successful_key_agreement_key_deletion() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let old_enc_key = get_x25519_encryption_key(true);

	let mut old_did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(alice_did.clone()));
	assert_ok!(old_did_details.add_key_agreement_key(old_enc_key, 0u64));

	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(alice_did.clone(), old_did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_ok!(Did::remove_key_agreement_key(
				RuntimeOrigin::signed(alice_did.clone()),
				generate_key_id(&old_enc_key.into()),
			));
			let new_did_details = Did::get_did(&alice_did).expect("ALICE_DID should be present on chain.");
			assert!(new_did_details.key_agreement_keys.is_empty());
			let public_keys = new_did_details.public_keys;
			// Total is -1 for the enc key removal + auth key = 1
			assert_eq!(public_keys.len(), 1);
			// Check for new key agreement key
			assert!(!public_keys.contains_key(&generate_key_id(&old_enc_key.into())));
		});
}

#[test]
fn check_did_not_found_key_agreement_key_deletion_error() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let test_enc_key = get_x25519_encryption_key(true);

	ExtBuilder::default().build(None).execute_with(|| {
		assert_noop!(
			Did::remove_key_agreement_key(
				RuntimeOrigin::signed(alice_did.clone()),
				generate_key_id(&test_enc_key.into())
			),
			did::Error::<Test>::NotFound
		);
	});
}

#[test]
fn check_key_not_found_key_agreement_key_deletion_error() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let test_enc_key = get_x25519_encryption_key(true);

	// No enc key added
	let old_did_details = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);

	ExtBuilder::default()
		.with_dids(vec![(alice_did.clone(), old_did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::remove_key_agreement_key(
					RuntimeOrigin::signed(alice_did.clone()),
					generate_key_id(&test_enc_key.into())
				),
				did::Error::<Test>::VerificationKeyNotFound
			);
		});
}

// add_service_endpoint

#[test]
fn check_service_addition_no_prior_service_successful() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let new_service_endpoint = DidEndpoint::new(b"id".to_vec(), vec![b"type".to_vec()], vec![b"url".to_vec()]);

	let old_did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(alice_did.clone()));

	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(alice_did.clone(), old_did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_ok!(Did::add_service_endpoint(
				RuntimeOrigin::signed(alice_did.clone()),
				new_service_endpoint.clone()
			));
			let stored_endpoint = did::pallet::ServiceEndpoints::<Test>::get(&alice_did, &new_service_endpoint.id)
				.expect("Service endpoint should be stored.");
			assert_eq!(stored_endpoint, new_service_endpoint);
			assert_eq!(
				did::pallet::ServiceEndpoints::<Test>::iter_prefix(&alice_did).count(),
				1
			);
			assert_eq!(did::pallet::DidEndpointsCount::<Test>::get(&alice_did), 1);
		});
}

#[test]
fn check_service_addition_one_from_full_successful() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let old_service_endpoints = get_service_endpoints(
		// -1 from the max number
		<Test as did::Config>::MaxNumberOfServicesPerDid::get() - 1,
		<Test as did::Config>::MaxServiceIdLength::get(),
		<Test as did::Config>::MaxNumberOfTypesPerService::get(),
		<Test as did::Config>::MaxServiceTypeLength::get(),
		<Test as did::Config>::MaxNumberOfUrlsPerService::get(),
		<Test as did::Config>::MaxServiceUrlLength::get(),
	);
	let new_service_endpoint = DidEndpoint::new(b"id".to_vec(), vec![b"type".to_vec()], vec![b"url".to_vec()]);

	let old_did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(alice_did.clone()));

	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(alice_did.clone(), old_did_details)])
		.with_endpoints(vec![(alice_did.clone(), old_service_endpoints)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_ok!(Did::add_service_endpoint(
				RuntimeOrigin::signed(alice_did.clone()),
				new_service_endpoint.clone()
			));
			assert_eq!(
				did::pallet::DidEndpointsCount::<Test>::get(&alice_did),
				<Test as did::Config>::MaxNumberOfServicesPerDid::get()
			);
			assert_eq!(
				did::pallet::ServiceEndpoints::<Test>::iter_prefix(&alice_did).count(),
				<Test as did::Config>::MaxNumberOfServicesPerDid::get().saturated_into::<usize>()
			);
			let stored_endpoint = did::pallet::ServiceEndpoints::<Test>::get(&alice_did, &new_service_endpoint.id)
				.expect("Service endpoint should be stored.");
			assert_eq!(stored_endpoint, new_service_endpoint);
		});
}

#[test]
fn check_did_not_present_services_addition_error() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let new_service_endpoint = DidEndpoint::new(b"id".to_vec(), vec![b"type".to_vec()], vec![b"url".to_vec()]);

	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.build(None)
		.execute_with(|| {
			assert_noop!(
				Did::add_service_endpoint(RuntimeOrigin::signed(alice_did.clone()), new_service_endpoint),
				did::Error::<Test>::NotFound
			);
		});
}

#[test]
fn check_service_already_present_addition_error() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let service_endpoint = DidEndpoint::new(b"id".to_vec(), vec![b"type".to_vec()], vec![b"url".to_vec()]);

	let old_did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(alice_did.clone()));

	ExtBuilder::default()
		.with_dids(vec![(alice_did.clone(), old_did_details)])
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_endpoints(vec![(alice_did.clone(), vec![service_endpoint.clone()])])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::add_service_endpoint(RuntimeOrigin::signed(alice_did.clone()), service_endpoint),
				did::Error::<Test>::ServiceAlreadyExists
			);
		});
}

#[test]
fn check_max_services_count_addition_error() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let old_service_endpoints = get_service_endpoints(
		<Test as did::Config>::MaxNumberOfServicesPerDid::get(),
		<Test as did::Config>::MaxServiceIdLength::get(),
		<Test as did::Config>::MaxNumberOfTypesPerService::get(),
		<Test as did::Config>::MaxServiceTypeLength::get(),
		<Test as did::Config>::MaxNumberOfUrlsPerService::get(),
		<Test as did::Config>::MaxServiceUrlLength::get(),
	);
	let new_service_endpoint = DidEndpoint::new(b"id".to_vec(), vec![b"type".to_vec()], vec![b"url".to_vec()]);

	let old_did_details = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);

	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(alice_did.clone(), old_did_details)])
		.with_endpoints(vec![(alice_did.clone(), old_service_endpoints)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::add_service_endpoint(RuntimeOrigin::signed(alice_did.clone()), new_service_endpoint),
				did::Error::<Test>::MaxNumberOfServicesExceeded
			);
		});
}

#[test]
#[should_panic = "Service ID too long."]
fn check_max_service_id_length_addition_error() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let new_service_endpoint = get_service_endpoints(
		1,
		<Test as did::Config>::MaxServiceIdLength::get() + 1,
		<Test as did::Config>::MaxNumberOfTypesPerService::get(),
		<Test as did::Config>::MaxServiceTypeLength::get(),
		<Test as did::Config>::MaxNumberOfUrlsPerService::get(),
		<Test as did::Config>::MaxServiceUrlLength::get(),
	)[0]
	.clone();

	let old_did_details = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);

	ExtBuilder::default()
		.with_dids(vec![(alice_did.clone(), old_did_details)])
		.build(None)
		.execute_with(|| {
			assert_noop!(
				Did::add_service_endpoint(RuntimeOrigin::signed(alice_did.clone()), new_service_endpoint),
				did::Error::<Test>::MaxServiceIdLengthExceeded
			);
		});
}

#[test]
#[should_panic = "Service type too long."]
fn check_max_service_type_length_addition_error() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let new_service_endpoint = get_service_endpoints(
		1,
		<Test as did::Config>::MaxServiceIdLength::get(),
		<Test as did::Config>::MaxNumberOfTypesPerService::get(),
		<Test as did::Config>::MaxServiceTypeLength::get() + 1,
		<Test as did::Config>::MaxNumberOfUrlsPerService::get(),
		<Test as did::Config>::MaxServiceUrlLength::get(),
	)[0]
	.clone();

	let old_did_details = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);

	ExtBuilder::default()
		.with_dids(vec![(alice_did.clone(), old_did_details)])
		.build(None)
		.execute_with(|| {
			assert_noop!(
				Did::add_service_endpoint(RuntimeOrigin::signed(alice_did.clone()), new_service_endpoint),
				did::Error::<Test>::MaxServiceTypeLengthExceeded
			);
		});
}

#[test]
#[should_panic = "Too many types for the given service."]
fn check_max_service_type_count_addition_error() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let new_service_endpoint = get_service_endpoints(
		1,
		<Test as did::Config>::MaxServiceIdLength::get(),
		<Test as did::Config>::MaxNumberOfTypesPerService::get() + 1,
		<Test as did::Config>::MaxServiceTypeLength::get(),
		<Test as did::Config>::MaxNumberOfUrlsPerService::get(),
		<Test as did::Config>::MaxServiceUrlLength::get(),
	)[0]
	.clone();

	let old_did_details = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);

	ExtBuilder::default()
		.with_dids(vec![(alice_did.clone(), old_did_details)])
		.build(None)
		.execute_with(|| {
			assert_noop!(
				Did::add_service_endpoint(RuntimeOrigin::signed(alice_did.clone()), new_service_endpoint),
				did::Error::<Test>::MaxNumberOfTypesPerServiceExceeded
			);
		});
}

#[test]
#[should_panic = "Service URL too long."]
fn check_max_service_url_length_addition_error() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let new_service_endpoint = get_service_endpoints(
		1,
		<Test as did::Config>::MaxServiceIdLength::get(),
		<Test as did::Config>::MaxNumberOfTypesPerService::get(),
		<Test as did::Config>::MaxServiceTypeLength::get(),
		<Test as did::Config>::MaxNumberOfUrlsPerService::get(),
		<Test as did::Config>::MaxServiceUrlLength::get() + 1,
	)[0]
	.clone();

	let old_did_details = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);

	ExtBuilder::default()
		.with_dids(vec![(alice_did.clone(), old_did_details)])
		.build(None)
		.execute_with(|| {
			assert_noop!(
				Did::add_service_endpoint(RuntimeOrigin::signed(alice_did.clone()), new_service_endpoint),
				did::Error::<Test>::MaxServiceUrlLengthExceeded
			);
		});
}

#[test]
#[should_panic = "Too many URLs for the given service."]
fn check_max_service_url_count_addition_error() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let new_service_endpoint = get_service_endpoints(
		1,
		<Test as did::Config>::MaxServiceIdLength::get(),
		<Test as did::Config>::MaxNumberOfTypesPerService::get(),
		<Test as did::Config>::MaxServiceTypeLength::get(),
		<Test as did::Config>::MaxNumberOfUrlsPerService::get() + 1,
		<Test as did::Config>::MaxServiceUrlLength::get(),
	)[0]
	.clone();

	let old_did_details = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);

	ExtBuilder::default()
		.with_dids(vec![(alice_did.clone(), old_did_details)])
		.build(None)
		.execute_with(|| {
			assert_noop!(
				Did::add_service_endpoint(RuntimeOrigin::signed(alice_did.clone()), new_service_endpoint),
				did::Error::<Test>::MaxNumberOfUrlsPerServiceExceeded
			);
		});
}

#[test]
fn character_addition_error() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let new_service_details = DidEndpoint::new("å".bytes().collect(), vec![b"type".to_vec()], vec![b"url".to_vec()]);

	let old_did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(alice_did.clone()));

	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(alice_did.clone(), old_did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::add_service_endpoint(RuntimeOrigin::signed(alice_did.clone()), new_service_details),
				did::Error::<Test>::InvalidServiceEncoding
			);
		});
}

#[test]
fn check_invalid_service_type_character_addition_error() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let new_service_details = DidEndpoint::new(b"id".to_vec(), vec!["å".bytes().collect()], vec![b"url".to_vec()]);

	let old_did_details = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);

	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(alice_did.clone(), old_did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::add_service_endpoint(RuntimeOrigin::signed(alice_did.clone()), new_service_details),
				did::Error::<Test>::InvalidServiceEncoding
			);
		});
}

#[test]
fn check_invalid_service_url_character_addition_error() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let new_service_details = DidEndpoint::new(b"id".to_vec(), vec![b"type".to_vec()], vec!["å".bytes().collect()]);

	let old_did_details = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);

	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(alice_did.clone(), old_did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::add_service_endpoint(RuntimeOrigin::signed(alice_did.clone()), new_service_details),
				did::Error::<Test>::InvalidServiceEncoding
			);
		});
}

// remove_service_endpoint

#[test]
fn check_service_deletion_successful() {
	initialize_logger();
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let old_service_endpoint = DidEndpoint::new(b"id".to_vec(), vec![b"type".to_vec()], vec![b"url".to_vec()]);

	let old_did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(alice_did.clone()));

	ExtBuilder::default()
		.with_dids(vec![(alice_did.clone(), old_did_details)])
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_endpoints(vec![(alice_did.clone(), vec![old_service_endpoint.clone()])])
		.build_and_execute_with_sanity_tests(None, || {
			assert_ok!(Did::remove_service_endpoint(
				RuntimeOrigin::signed(alice_did.clone()),
				old_service_endpoint.id
			));
			// Counter should be deleted from the storage.
			assert_eq!(did::pallet::DidEndpointsCount::<Test>::get(&alice_did), 0);
			assert_eq!(
				did::pallet::ServiceEndpoints::<Test>::iter_prefix(&alice_did).count(),
				0
			);
		});
}

#[test]
fn check_service_not_present_deletion_error() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let service_id = b"id".to_vec();

	let old_did_details = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);

	ExtBuilder::default()
		.with_dids(vec![(alice_did.clone(), old_did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::remove_service_endpoint(
					RuntimeOrigin::signed(alice_did.clone()),
					service_id.try_into().expect("Service ID to delete too long")
				),
				did::Error::<Test>::ServiceNotFound
			);
		});
}

// delete

#[test]
fn check_successful_deletion_no_endpoints() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());

	let mut did_details = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);
	did_details.deposit.owner = ACCOUNT_00;
	did_details.deposit.amount = <Test as did::Config>::BaseDeposit::get();

	let balance = <Test as did::Config>::BaseDeposit::get() * 2
		+ <Test as did::Config>::Fee::get() * 2
		+ <<Test as did::Config>::Currency as Currency<did::AccountIdOf<Test>>>::minimum_balance();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, balance)])
		.with_dids(vec![(alice_did.clone(), did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_eq!(did::pallet::DidEndpointsCount::<Test>::get(&alice_did), 0);
			assert_eq!(
				Balances::reserved_balance(ACCOUNT_00),
				<Test as did::Config>::BaseDeposit::get()
			);
			assert_ok!(Did::delete(RuntimeOrigin::signed(alice_did.clone()), 0));
			assert!(Did::get_did(alice_did.clone()).is_none());
			assert!(Did::get_deleted_did(alice_did.clone()).is_some());
			assert!(Balances::reserved_balance(ACCOUNT_00).is_zero());

			assert_eq!(did::pallet::DidEndpointsCount::<Test>::get(&alice_did), 0);

			// Re-adding the same DID identifier should fail.
			let details = generate_base_did_creation_details::<Test>(alice_did.clone(), ACCOUNT_00);

			let signature = auth_key.sign(details.encode().as_ref());

			assert_noop!(
				Did::create(
					RuntimeOrigin::signed(ACCOUNT_00.clone()),
					Box::new(details),
					did::DidSignature::from(signature),
				),
				did::Error::<Test>::AlreadyDeleted
			);
		});
}

#[test]
fn check_successful_deletion_with_endpoints() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let service_endpoint = DidEndpoint::new(b"id".to_vec(), vec![b"type".to_vec()], vec![b"url".to_vec()]);

	let mut did_details = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);
	did_details.deposit.owner = ACCOUNT_00;
	did_details.deposit.amount = <Test as did::Config>::BaseDeposit::get();

	let balance = <Test as did::Config>::BaseDeposit::get() * 2
		+ <Test as did::Config>::Fee::get() * 2
		+ <<Test as did::Config>::Currency as Currency<did::AccountIdOf<Test>>>::minimum_balance();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, balance)])
		.with_dids(vec![(alice_did.clone(), did_details)])
		.with_endpoints(vec![(alice_did.clone(), vec![service_endpoint])])
		.build_and_execute_with_sanity_tests(None, || {
			assert_eq!(did::pallet::DidEndpointsCount::<Test>::get(&alice_did), 1);
			assert_eq!(
				Balances::reserved_balance(ACCOUNT_00),
				<Test as did::Config>::BaseDeposit::get()
			);
			assert_ok!(Did::delete(RuntimeOrigin::signed(alice_did.clone()), 1));
			assert!(Did::get_did(alice_did.clone()).is_none());
			assert!(Did::get_deleted_did(alice_did.clone()).is_some());
			assert!(Balances::reserved_balance(ACCOUNT_00).is_zero());

			assert_eq!(did::pallet::DidEndpointsCount::<Test>::get(&alice_did), 0);

			// Re-adding the same DID identifier should fail.
			let details = generate_base_did_creation_details::<Test>(alice_did.clone(), ACCOUNT_00);

			let signature = auth_key.sign(details.encode().as_ref());

			assert_noop!(
				Did::create(
					RuntimeOrigin::signed(ACCOUNT_00.clone()),
					Box::new(details),
					did::DidSignature::from(signature),
				),
				did::Error::<Test>::AlreadyDeleted
			);
		});
}

#[test]
fn check_did_not_present_deletion() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());

	let balance = <Test as did::Config>::BaseDeposit::get()
		+ <Test as did::Config>::Fee::get()
		+ <<Test as did::Config>::Currency as Currency<did::AccountIdOf<Test>>>::minimum_balance();
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, balance)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::delete(RuntimeOrigin::signed(alice_did), 0),
				did::Error::<Test>::NotFound
			);
		});
}

#[test]
fn check_service_count_too_small_deletion_error() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let service_endpoint = DidEndpoint::new(b"id".to_vec(), vec![b"type".to_vec()], vec![b"url".to_vec()]);

	let mut did_details = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);
	did_details.deposit.owner = ACCOUNT_00;
	did_details.deposit.amount = <Test as did::Config>::BaseDeposit::get();

	let balance = <Test as did::Config>::BaseDeposit::get() * 2
		+ <Test as did::Config>::Fee::get() * 2
		+ <<Test as did::Config>::Currency as Currency<did::AccountIdOf<Test>>>::minimum_balance();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, balance)])
		.with_dids(vec![(alice_did.clone(), did_details)])
		.with_endpoints(vec![(alice_did.clone(), vec![service_endpoint])])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::delete(RuntimeOrigin::signed(alice_did.clone()), 0),
				did::Error::<Test>::MaxStoredEndpointsCountExceeded
			);
		});
}

// reclaim_deposit

#[test]
fn check_successful_reclaiming() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let old_service_endpoint = DidEndpoint::new(b"id".to_vec(), vec![b"type".to_vec()], vec![b"url".to_vec()]);
	let mut did_details = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);
	did_details.deposit.owner = ACCOUNT_00;
	did_details.deposit.amount = <Test as did::Config>::BaseDeposit::get();

	let balance = <Test as did::Config>::BaseDeposit::get() * 2
		+ <Test as did::Config>::Fee::get() * 2
		+ <<Test as did::Config>::Currency as Currency<did::AccountIdOf<Test>>>::minimum_balance();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, balance)])
		.with_dids(vec![(alice_did.clone(), did_details)])
		.with_endpoints(vec![(alice_did.clone(), vec![old_service_endpoint])])
		.build_and_execute_with_sanity_tests(None, || {
			assert_eq!(did::pallet::DidEndpointsCount::<Test>::get(&alice_did), 1);
			assert_eq!(
				Balances::reserved_balance(ACCOUNT_00),
				<Test as did::Config>::BaseDeposit::get()
			);
			assert_ok!(Did::reclaim_deposit(
				RuntimeOrigin::signed(ACCOUNT_00.clone()),
				alice_did.clone(),
				1
			));
			assert!(Did::get_did(alice_did.clone()).is_none());
			assert!(Did::get_deleted_did(alice_did.clone()).is_some());
			assert!(Balances::reserved_balance(ACCOUNT_00).is_zero());
			assert_eq!(did::pallet::DidEndpointsCount::<Test>::get(&alice_did), 0);

			// Re-adding the same DID identifier should fail.
			let details = generate_base_did_creation_details::<Test>(alice_did.clone(), ACCOUNT_00);

			let signature = auth_key.sign(details.encode().as_ref());

			assert_noop!(
				Did::create(
					RuntimeOrigin::signed(ACCOUNT_00.clone()),
					Box::new(details),
					did::DidSignature::from(signature),
				),
				did::Error::<Test>::AlreadyDeleted
			);
		});
}

#[test]
fn unauthorized_reclaiming() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let mut did_details = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);
	did_details.deposit.owner = ACCOUNT_00;
	did_details.deposit.amount = <Test as did::Config>::BaseDeposit::get();

	let balance = <Test as did::Config>::BaseDeposit::get()
		+ <Test as did::Config>::Fee::get()
		+ <<Test as did::Config>::Currency as Currency<did::AccountIdOf<Test>>>::minimum_balance();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, balance)])
		.with_dids(vec![(alice_did.clone(), did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_eq!(
				Balances::reserved_balance(ACCOUNT_00),
				<Test as did::Config>::BaseDeposit::get()
			);
			assert_noop!(
				Did::reclaim_deposit(RuntimeOrigin::signed(ACCOUNT_01.clone()), alice_did.clone(), 0),
				did::Error::<Test>::NotOwnerOfDeposit
			);
		});
}

#[test]
fn check_service_count_too_small_reclaim_error() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let service_endpoint = DidEndpoint::new(b"id".to_vec(), vec![b"type".to_vec()], vec![b"url".to_vec()]);

	let mut did_details = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);
	did_details.deposit.owner = ACCOUNT_00;
	did_details.deposit.amount = <Test as did::Config>::BaseDeposit::get();

	let balance = <Test as did::Config>::BaseDeposit::get() * 2
		+ <Test as did::Config>::Fee::get() * 2
		+ <<Test as did::Config>::Currency as Currency<did::AccountIdOf<Test>>>::minimum_balance();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, balance)])
		.with_dids(vec![(alice_did.clone(), did_details)])
		.with_endpoints(vec![(alice_did.clone(), vec![service_endpoint])])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::reclaim_deposit(RuntimeOrigin::signed(ACCOUNT_00.clone()), alice_did.clone(), 0),
				did::Error::<Test>::MaxStoredEndpointsCountExceeded
			);
		});
}

// submit_did_call

#[test]
fn check_did_not_found_call_error() {
	let auth_key = get_sr25519_authentication_key(true);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = ACCOUNT_00;

	let call_operation = generate_test_did_call(DidVerificationKeyRelationship::Authentication, did, caller.clone());
	let signature = auth_key.sign(call_operation.encode().as_ref());

	// No DID added
	ExtBuilder::default().build(None).execute_with(|| {
		assert_noop!(
			Did::submit_did_call(
				RuntimeOrigin::signed(caller),
				Box::new(call_operation.operation),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::NotFound
		);
	});
}

#[test]
fn check_too_small_tx_counter_after_wrap_call_error() {
	let auth_key = get_sr25519_authentication_key(true);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = ACCOUNT_00;
	let mut mock_did = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);
	// After wrapping tx_counter becomes 0 again.
	mock_did.last_tx_counter = 0u64;

	let mut call_operation = generate_test_did_call(
		DidVerificationKeyRelationship::Authentication,
		did.clone(),
		caller.clone(),
	);
	call_operation.operation.tx_counter = u64::MAX;
	let signature = auth_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_dids(vec![(did, mock_did)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::submit_did_call(
					RuntimeOrigin::signed(caller),
					Box::new(call_operation.operation),
					did::DidSignature::from(signature)
				),
				did::Error::<Test>::InvalidNonce
			);
		});
}

#[test]
fn check_too_small_tx_counter_call_error() {
	let auth_key = get_sr25519_authentication_key(true);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = ACCOUNT_00;
	let mut mock_did = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);
	mock_did.last_tx_counter = 1u64;

	let mut call_operation = generate_test_did_call(
		DidVerificationKeyRelationship::Authentication,
		did.clone(),
		caller.clone(),
	);
	call_operation.operation.tx_counter = mock_did.last_tx_counter - 1;
	let signature = auth_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_dids(vec![(did, mock_did)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::submit_did_call(
					RuntimeOrigin::signed(caller),
					Box::new(call_operation.operation),
					did::DidSignature::from(signature)
				),
				did::Error::<Test>::InvalidNonce
			);
		});
}

#[test]
fn check_equal_tx_counter_call_error() {
	let auth_key = get_sr25519_authentication_key(true);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = ACCOUNT_00;
	let mock_did = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);

	let mut call_operation = generate_test_did_call(
		DidVerificationKeyRelationship::Authentication,
		did.clone(),
		caller.clone(),
	);
	call_operation.operation.tx_counter = mock_did.last_tx_counter;
	let signature = auth_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_dids(vec![(did, mock_did)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::submit_did_call(
					RuntimeOrigin::signed(caller),
					Box::new(call_operation.operation),
					did::DidSignature::from(signature)
				),
				did::Error::<Test>::InvalidNonce
			);
		});
}

#[test]
fn check_too_large_tx_counter_call_error() {
	let auth_key = get_sr25519_authentication_key(true);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = ACCOUNT_00;
	let mock_did = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);

	let mut call_operation = generate_test_did_call(
		DidVerificationKeyRelationship::Authentication,
		did.clone(),
		caller.clone(),
	);
	call_operation.operation.tx_counter = mock_did.last_tx_counter + 2u64;
	let signature = auth_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_dids(vec![(did, mock_did)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::submit_did_call(
					RuntimeOrigin::signed(caller),
					Box::new(call_operation.operation),
					did::DidSignature::from(signature)
				),
				did::Error::<Test>::InvalidNonce
			);
		});
}

#[test]
fn check_tx_block_number_too_low_error() {
	let auth_key = get_sr25519_authentication_key(true);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = ACCOUNT_00;

	let mock_did = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);

	let call_operation = generate_test_did_call(
		DidVerificationKeyRelationship::Authentication,
		did.clone(),
		caller.clone(),
	);
	let signature = auth_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_dids(vec![(did, mock_did)])
		.build_and_execute_with_sanity_tests(None, || {
			// System block number 1 past the max block the operation was allowed for.
			System::set_block_number(call_operation.operation.block_number + MaxBlocksTxValidity::get() + 1);
			assert_noop!(
				Did::submit_did_call(
					RuntimeOrigin::signed(caller.clone()),
					Box::new(call_operation.operation.clone()),
					did::DidSignature::from(signature.clone())
				),
				did::Error::<Test>::TransactionExpired
			);

			// But it would work if the system would be one block earlier.
			System::set_block_number(call_operation.operation.block_number + MaxBlocksTxValidity::get());
			assert_ok!(Did::submit_did_call(
				RuntimeOrigin::signed(caller),
				Box::new(call_operation.operation),
				did::DidSignature::from(signature)
			));
		});
}

#[test]
fn check_tx_block_number_too_high_error() {
	let auth_key = get_sr25519_authentication_key(true);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = ACCOUNT_00;

	let mock_did = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);

	let mut call_operation = generate_test_did_call(
		DidVerificationKeyRelationship::Authentication,
		did.clone(),
		caller.clone(),
	);

	call_operation.operation.block_number = MaxBlocksTxValidity::get() + 100;
	let signature = auth_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_dids(vec![(did, mock_did)])
		.build_and_execute_with_sanity_tests(None, || {
			// System block number is still too low, meaning that the block number used in
			// the operation was too high.
			System::set_block_number(call_operation.operation.block_number - MaxBlocksTxValidity::get() - 1);
			assert_noop!(
				Did::submit_did_call(
					RuntimeOrigin::signed(caller.clone()),
					Box::new(call_operation.operation.clone()),
					did::DidSignature::from(signature.clone())
				),
				did::Error::<Test>::TransactionExpired
			);
		});
}

#[test]
fn check_verification_key_not_present_call_error() {
	let auth_key = get_sr25519_authentication_key(true);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = ACCOUNT_00;
	let mock_did = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);

	// The operation requires the delegation key that is currently not stored for
	// the given DID.
	let call_operation = generate_test_did_call(
		DidVerificationKeyRelationship::CapabilityDelegation,
		did.clone(),
		caller.clone(),
	);
	let signature = auth_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_dids(vec![(did, mock_did)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::submit_did_call(
					RuntimeOrigin::signed(caller),
					Box::new(call_operation.operation),
					did::DidSignature::from(signature)
				),
				did::Error::<Test>::VerificationKeyNotFound
			);
		});
}

#[test]
fn check_invalid_signature_format_call_error() {
	let auth_key = get_sr25519_authentication_key(true);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = ACCOUNT_00;
	let alternative_auth_key = get_ed25519_authentication_key(true);
	let mock_did = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);

	let call_operation = generate_test_did_call(
		DidVerificationKeyRelationship::Authentication,
		did.clone(),
		caller.clone(),
	);
	let signature = alternative_auth_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_dids(vec![(did, mock_did)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::submit_did_call(
					RuntimeOrigin::signed(caller),
					Box::new(call_operation.operation),
					did::DidSignature::from(signature)
				),
				did::Error::<Test>::InvalidSignatureFormat
			);
		});
}

#[test]
fn check_bad_submitter_error() {
	let auth_key = get_sr25519_authentication_key(true);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = ACCOUNT_00;
	let alternative_auth_key = get_sr25519_authentication_key(false);
	let mock_did = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);

	let submitter = ACCOUNT_01;

	let call_operation = generate_test_did_call(DidVerificationKeyRelationship::Authentication, did.clone(), submitter);
	let signature = alternative_auth_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_dids(vec![(did, mock_did)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::submit_did_call(
					RuntimeOrigin::signed(caller),
					Box::new(call_operation.operation),
					did::DidSignature::from(signature)
				),
				did::Error::<Test>::BadDidOrigin
			);
		});
}

#[test]
fn check_invalid_signature_call_error() {
	let auth_key = get_sr25519_authentication_key(true);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = ACCOUNT_00;
	let alternative_auth_key = get_sr25519_authentication_key(false);
	let mock_did = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);

	let call_operation = generate_test_did_call(
		DidVerificationKeyRelationship::Authentication,
		did.clone(),
		caller.clone(),
	);
	let signature = alternative_auth_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_dids(vec![(did, mock_did)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::submit_did_call(
					RuntimeOrigin::signed(caller),
					Box::new(call_operation.operation),
					did::DidSignature::from(signature)
				),
				did::Error::<Test>::InvalidSignature
			);
		});
}

#[test]
fn check_call_attestation_key_successful() {
	let auth_key = get_sr25519_authentication_key(true);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = ACCOUNT_00;
	let attestation_key = get_ed25519_attestation_key(true);

	let mut mock_did = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);
	assert_ok!(mock_did.update_attestation_key(DidVerificationKey::from(attestation_key.public()), 0));

	let call_operation = generate_test_did_call(
		DidVerificationKeyRelationship::AssertionMethod,
		did.clone(),
		caller.clone(),
	);
	let signature = attestation_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_dids(vec![(did, mock_did)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_ok!(Did::submit_did_call(
				RuntimeOrigin::signed(caller),
				Box::new(call_operation.operation),
				did::DidSignature::from(signature)
			));
		});
}

#[test]
fn check_call_attestation_key_error() {
	let auth_key = get_sr25519_authentication_key(true);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = ACCOUNT_00;
	let attestation_key = get_ed25519_attestation_key(true);

	let mut mock_did = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);
	assert_ok!(mock_did.update_attestation_key(DidVerificationKey::from(attestation_key.public()), 0));

	let call_operation = generate_test_did_call(
		DidVerificationKeyRelationship::AssertionMethod,
		did.clone(),
		caller.clone(),
	);
	let signature = attestation_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_dids(vec![(did.clone(), mock_did)])
		.with_ctypes(vec![(
			<Test as frame_system::Config>::Hashing::hash(&get_attestation_key_test_input()[..]),
			did,
		)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_err!(
				Did::submit_did_call(
					RuntimeOrigin::signed(caller),
					Box::new(call_operation.operation),
					did::DidSignature::from(signature)
				),
				ctype::Error::<Test>::AlreadyExists
			);
		});
}

#[test]
fn check_call_delegation_key_successful() {
	let auth_key = get_sr25519_authentication_key(true);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = ACCOUNT_00;
	let delegation_key = get_ed25519_delegation_key(true);

	let mut mock_did = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);
	assert_ok!(mock_did.update_delegation_key(DidVerificationKey::from(delegation_key.public()), 0));

	let call_operation = generate_test_did_call(
		DidVerificationKeyRelationship::CapabilityDelegation,
		did.clone(),
		caller.clone(),
	);
	let signature = delegation_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_dids(vec![(did, mock_did)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_ok!(Did::submit_did_call(
				RuntimeOrigin::signed(caller),
				Box::new(call_operation.operation),
				did::DidSignature::from(signature)
			));
		});
}

#[test]
fn check_call_delegation_key_error() {
	let auth_key = get_sr25519_authentication_key(true);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = ACCOUNT_00;
	let delegation_key = get_ed25519_delegation_key(true);

	let mut mock_did = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);
	assert_ok!(mock_did.update_delegation_key(DidVerificationKey::from(delegation_key.public()), 0));

	let call_operation = generate_test_did_call(
		DidVerificationKeyRelationship::CapabilityDelegation,
		did.clone(),
		caller.clone(),
	);
	let signature = delegation_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_dids(vec![(did.clone(), mock_did)])
		.with_ctypes(vec![(
			<Test as frame_system::Config>::Hashing::hash(&get_delegation_key_test_input()[..]),
			did,
		)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_err!(
				Did::submit_did_call(
					RuntimeOrigin::signed(caller),
					Box::new(call_operation.operation),
					did::DidSignature::from(signature)
				),
				ctype::Error::<Test>::AlreadyExists
			);
		});
}

#[test]
fn check_call_authentication_key_successful() {
	let auth_key = get_sr25519_authentication_key(true);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = ACCOUNT_00;

	let mock_did = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);

	let call_operation = generate_test_did_call(
		DidVerificationKeyRelationship::Authentication,
		did.clone(),
		caller.clone(),
	);
	let signature = auth_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_dids(vec![(did, mock_did)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_ok!(Did::submit_did_call(
				RuntimeOrigin::signed(caller),
				Box::new(call_operation.operation),
				did::DidSignature::from(signature)
			));
		});
}

#[test]
fn check_call_authentication_key_error() {
	let auth_key = get_sr25519_authentication_key(true);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = ACCOUNT_00;

	let mock_did = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);

	let call_operation = generate_test_did_call(
		DidVerificationKeyRelationship::Authentication,
		did.clone(),
		caller.clone(),
	);
	let signature = auth_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_dids(vec![(did.clone(), mock_did)])
		.with_ctypes(vec![(
			<Test as frame_system::Config>::Hashing::hash(&get_authentication_key_test_input()[..]),
			did,
		)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_err!(
				Did::submit_did_call(
					RuntimeOrigin::signed(caller),
					Box::new(call_operation.operation),
					did::DidSignature::from(signature)
				),
				ctype::Error::<Test>::AlreadyExists
			);
		});
}

#[test]
fn check_null_key_error() {
	let auth_key = get_sr25519_authentication_key(true);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = ACCOUNT_00;

	// CapabilityInvocation is not supported at the moment, so it should return no
	// key and hence the operation fail.
	let call_operation = generate_test_did_call(
		DidVerificationKeyRelationship::CapabilityInvocation,
		did,
		caller.clone(),
	);
	let signature = ed25519::Signature::from_raw([0u8; 64]);

	ExtBuilder::default().build(None).execute_with(|| {
		assert_noop!(
			Did::submit_did_call(
				RuntimeOrigin::signed(caller),
				Box::new(call_operation.operation),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::UnsupportedDidAuthorizationCall
		);
	});
}

// Internal function: verify_did_operation_signature_and_increase_nonce

#[test]
fn check_authentication_successful_operation_verification() {
	let auth_key = get_sr25519_authentication_key(true);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());

	let mock_did = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);

	let call_operation =
		generate_test_did_call(DidVerificationKeyRelationship::Authentication, did.clone(), ACCOUNT_00);
	let signature = auth_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_dids(vec![(did, mock_did.clone())])
		.build_and_execute_with_sanity_tests(None, || {
			assert_ok!(Did::verify_did_operation_signature_and_increase_nonce(
				&call_operation,
				&did::DidSignature::from(signature)
			));
			// Verify that the DID tx counter has increased
			let did_details = Did::get_did(&call_operation.operation.did).expect("DID should be present on chain.");
			assert_eq!(did_details.last_tx_counter, mock_did.last_tx_counter + 1u64);
		});
}

#[test]
fn check_attestation_successful_operation_verification() {
	let auth_key = get_sr25519_authentication_key(true);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let attestation_key = get_ed25519_attestation_key(true);

	let mut mock_did = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);
	assert_ok!(mock_did.update_attestation_key(DidVerificationKey::from(attestation_key.public()), 0));

	let call_operation =
		generate_test_did_call(DidVerificationKeyRelationship::AssertionMethod, did.clone(), ACCOUNT_00);
	let signature = attestation_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_dids(vec![(did, mock_did.clone())])
		.build_and_execute_with_sanity_tests(None, || {
			assert_ok!(Did::verify_did_operation_signature_and_increase_nonce(
				&call_operation,
				&did::DidSignature::from(signature)
			));
			// Verify that the DID tx counter has increased
			let did_details = Did::get_did(&call_operation.operation.did).expect("DID should be present on chain.");
			assert_eq!(did_details.last_tx_counter, mock_did.last_tx_counter + 1u64);
		});
}

#[test]
fn check_delegation_successful_operation_verification() {
	let auth_key = get_sr25519_authentication_key(true);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let delegation_key = get_ecdsa_delegation_key(true);

	let mut mock_did = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);
	assert_ok!(mock_did.update_delegation_key(DidVerificationKey::from(delegation_key.public()), 0));

	let call_operation = generate_test_did_call(
		DidVerificationKeyRelationship::CapabilityDelegation,
		did.clone(),
		ACCOUNT_00,
	);
	let signature = delegation_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_dids(vec![(did, mock_did.clone())])
		.build_and_execute_with_sanity_tests(None, || {
			assert_ok!(Did::verify_did_operation_signature_and_increase_nonce(
				&call_operation,
				&did::DidSignature::from(signature)
			));
			// Verify that the DID tx counter has increased
			let did_details = Did::get_did(&call_operation.operation.did).expect("DID should be present on chain.");
			assert_eq!(did_details.last_tx_counter, mock_did.last_tx_counter + 1u64);
		});
}

#[test]
fn check_did_not_present_operation_verification() {
	let auth_key = get_sr25519_authentication_key(true);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());

	let call_operation = generate_test_did_call(DidVerificationKeyRelationship::CapabilityDelegation, did, ACCOUNT_00);
	let signature = auth_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default().build(None).execute_with(|| {
		assert_noop!(
			Did::verify_did_operation_signature_and_increase_nonce(
				&call_operation,
				&did::DidSignature::from(signature)
			),
			did::errors::StorageError::NotFound(did::errors::NotFoundKind::Did)
		);
	});
}

#[test]
fn check_tx_counter_wrap_operation_verification() {
	let auth_key = get_sr25519_authentication_key(true);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());

	let mut mock_did = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);
	mock_did.last_tx_counter = u64::MAX;

	let mut call_operation =
		generate_test_did_call(DidVerificationKeyRelationship::Authentication, did.clone(), ACCOUNT_00);
	// Counter should wrap, so 0 is now expected.
	call_operation.operation.tx_counter = 0u64;
	let signature = auth_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_dids(vec![(did, mock_did)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_ok!(Did::verify_did_operation_signature_and_increase_nonce(
				&call_operation,
				&did::DidSignature::from(signature)
			));
			// Verify that the DID tx counter has wrapped around
			let did_details = Did::get_did(&call_operation.operation.did).expect("DID should be present on chain.");
			assert_eq!(did_details.last_tx_counter, 0u64);
		});
}

#[test]
fn check_smaller_counter_operation_verification() {
	let auth_key = get_ed25519_authentication_key(true);
	let did = get_did_identifier_from_ed25519_key(auth_key.public());

	let mut mock_did = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);
	mock_did.last_tx_counter = 1;

	let mut call_operation = generate_test_did_call(
		DidVerificationKeyRelationship::CapabilityDelegation,
		did.clone(),
		ACCOUNT_00,
	);
	call_operation.operation.tx_counter = 0u64;
	let signature = auth_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_dids(vec![(did, mock_did)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::verify_did_operation_signature_and_increase_nonce(
					&call_operation,
					&did::DidSignature::from(signature)
				),
				did::errors::DidError::Signature(did::errors::SignatureError::InvalidNonce)
			);
		});
}

#[test]
fn check_equal_counter_operation_verification() {
	let auth_key = get_ed25519_authentication_key(true);
	let did = get_did_identifier_from_ed25519_key(auth_key.public());

	let mock_did = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);

	let mut call_operation = generate_test_did_call(
		DidVerificationKeyRelationship::CapabilityDelegation,
		did.clone(),
		ACCOUNT_00,
	);
	call_operation.operation.tx_counter = mock_did.last_tx_counter;
	let signature = auth_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_dids(vec![(did, mock_did)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::verify_did_operation_signature_and_increase_nonce(
					&call_operation,
					&did::DidSignature::from(signature)
				),
				did::errors::DidError::Signature(did::errors::SignatureError::InvalidNonce)
			);
		});
}

#[test]
fn check_too_large_counter_operation_verification() {
	let auth_key = get_ed25519_authentication_key(true);
	let did = get_did_identifier_from_ed25519_key(auth_key.public());

	let mock_did = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);

	let mut call_operation = generate_test_did_call(
		DidVerificationKeyRelationship::CapabilityDelegation,
		did.clone(),
		ACCOUNT_00,
	);
	call_operation.operation.tx_counter = mock_did.last_tx_counter + 2;
	let signature = auth_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_dids(vec![(did, mock_did)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::verify_did_operation_signature_and_increase_nonce(
					&call_operation,
					&did::DidSignature::from(signature)
				),
				did::errors::DidError::Signature(did::errors::SignatureError::InvalidNonce)
			);
		});
}

#[test]
fn check_verification_key_not_present_operation_verification() {
	let auth_key = get_ed25519_authentication_key(true);
	let did = get_did_identifier_from_ed25519_key(auth_key.public());

	let mock_did = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);

	let call_operation =
		generate_test_did_call(DidVerificationKeyRelationship::AssertionMethod, did.clone(), ACCOUNT_00);
	let signature = auth_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_dids(vec![(did, mock_did)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::verify_did_operation_signature_and_increase_nonce(
					&call_operation,
					&did::DidSignature::from(signature)
				),
				did::errors::DidError::Storage(did::errors::StorageError::NotFound(did::errors::NotFoundKind::Key(
					did::errors::KeyType::AssertionMethod
				)))
			);
		});
}

#[test]
fn check_invalid_signature_format_operation_verification() {
	let auth_key = get_sr25519_authentication_key(true);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	// Expected an Sr25519, given an Ed25519
	let invalid_key = get_ed25519_authentication_key(true);

	let mock_did = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);

	let call_operation =
		generate_test_did_call(DidVerificationKeyRelationship::Authentication, did.clone(), ACCOUNT_00);
	let signature = invalid_key.sign(call_operation.encode().as_ref());

	ExtBuilder::default()
		.with_dids(vec![(did, mock_did)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::verify_did_operation_signature_and_increase_nonce(
					&call_operation,
					&did::DidSignature::from(signature)
				),
				did::errors::DidError::Signature(did::errors::SignatureError::InvalidFormat)
			);
		});
}

#[test]
fn check_invalid_signature_operation_verification() {
	let auth_key = get_sr25519_authentication_key(true);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	// Using same key type but different seed (default = false)
	let alternative_key = get_sr25519_authentication_key(false);

	let mock_did = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);

	let call_operation =
		generate_test_did_call(DidVerificationKeyRelationship::Authentication, did.clone(), ACCOUNT_00);
	let signature = alternative_key.sign(&call_operation.encode());

	ExtBuilder::default()
		.with_dids(vec![(did, mock_did)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::verify_did_operation_signature_and_increase_nonce(
					&call_operation,
					&did::DidSignature::from(signature)
				),
				did::errors::DidError::Signature(did::errors::SignatureError::InvalidData)
			);
		});
}

// #############################################################################
// transfer deposit

#[test]
fn test_change_deposit_owner() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let mut did_details = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);
	did_details.deposit.owner = ACCOUNT_00;
	did_details.deposit.amount = <Test as did::Config>::BaseDeposit::get();

	let balance = <Test as did::Config>::BaseDeposit::get() * 2
		+ <Test as did::Config>::Fee::get() * 2
		+ <<Test as did::Config>::Currency as Currency<did::AccountIdOf<Test>>>::minimum_balance();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, balance), (alice_did.clone(), balance)])
		.with_dids(vec![(alice_did.clone(), did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_eq!(
				Balances::reserved_balance(ACCOUNT_00),
				<Test as did::Config>::BaseDeposit::get()
			);
			assert_ok!(Did::change_deposit_owner(RuntimeOrigin::signed(alice_did.clone())));
			assert!(Balances::reserved_balance(ACCOUNT_00).is_zero());
			assert_eq!(
				Balances::reserved_balance(alice_did),
				<Test as did::Config>::BaseDeposit::get()
			);
		});
}

#[test]
fn test_change_deposit_owner_insufficient_balance() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let mut did_details = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);
	did_details.deposit.owner = ACCOUNT_00;
	did_details.deposit.amount = <Test as did::Config>::BaseDeposit::get();

	let balance = <Test as did::Config>::BaseDeposit::get() * 2
		+ <Test as did::Config>::Fee::get() * 2
		+ <<Test as did::Config>::Currency as Currency<did::AccountIdOf<Test>>>::minimum_balance();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, balance)])
		.with_dids(vec![(alice_did.clone(), did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::change_deposit_owner(RuntimeOrigin::signed(alice_did.clone())),
				pallet_balances::Error::<Test>::InsufficientBalance
			);
		});
}

#[test]
fn test_change_deposit_owner_not_found() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());

	let balance = <Test as did::Config>::BaseDeposit::get() * 2
		+ <Test as did::Config>::Fee::get() * 2
		+ <<Test as did::Config>::Currency as Currency<did::AccountIdOf<Test>>>::minimum_balance();

	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), balance)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::change_deposit_owner(RuntimeOrigin::signed(alice_did.clone())),
				crate::Error::<Test>::NotFound
			);
		});
}

#[test]
fn test_change_deposit_owner_not_authorized() {
	let alice_auth_key = get_ed25519_authentication_key(true);
	let bob_auth_key = get_ed25519_authentication_key(false);
	let alice_did = get_did_identifier_from_ed25519_key(alice_auth_key.public());
	let bob_did = get_did_identifier_from_ed25519_key(bob_auth_key.public());

	let balance = <Test as did::Config>::BaseDeposit::get() * 2
		+ <Test as did::Config>::Fee::get() * 2
		+ <<Test as did::Config>::Currency as Currency<did::AccountIdOf<Test>>>::minimum_balance();

	ExtBuilder::default()
		.with_balances(vec![(alice_did, balance), (bob_did.clone(), balance)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::change_deposit_owner(RuntimeOrigin::signed(bob_did.clone())),
				crate::Error::<Test>::NotFound
			);
		});
}

/// Update the deposit amount
#[test]
fn test_update_deposit() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let mut did_details = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);
	did_details.deposit.owner = alice_did.clone();
	did_details.deposit.amount = <Test as did::Config>::BaseDeposit::get() * 2;

	let balance = <Test as did::Config>::BaseDeposit::get() * 2
		+ <Test as did::Config>::Fee::get() * 2
		+ <<Test as did::Config>::Currency as Currency<did::AccountIdOf<Test>>>::minimum_balance();

	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), balance)])
		.with_dids(vec![(alice_did.clone(), did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_eq!(
				Balances::reserved_balance(alice_did.clone()),
				<Test as did::Config>::BaseDeposit::get() * 2
			);

			assert_ok!(Did::update_deposit(
				RuntimeOrigin::signed(alice_did.clone()),
				alice_did.clone()
			));

			assert_eq!(
				Balances::reserved_balance(alice_did.clone()),
				<Test as did::Config>::BaseDeposit::get()
			);

			assert_eq!(
				Did::get_did(&alice_did)
					.expect("DID should be present on chain.")
					.deposit,
				kilt_support::deposit::Deposit {
					owner: alice_did,
					amount: <Test as did::Config>::BaseDeposit::get(),
				}
			);
		});
}

#[test]
fn test_update_deposit_unauthorized() {
	let alice_auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(alice_auth_key.public());

	let bob_auth_key = get_ed25519_authentication_key(false);
	let bob_did = get_did_identifier_from_ed25519_key(bob_auth_key.public());

	let mut did_details = generate_base_did_details::<Test>(DidVerificationKey::from(alice_auth_key.public()), None);
	did_details.deposit.owner = alice_did.clone();
	did_details.deposit.amount = <Test as did::Config>::BaseDeposit::get() * 2;

	let balance = <Test as did::Config>::BaseDeposit::get() * 2
		+ <Test as did::Config>::Fee::get() * 2
		+ <<Test as did::Config>::Currency as Currency<did::AccountIdOf<Test>>>::minimum_balance();

	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), balance)])
		.with_dids(vec![(alice_did.clone(), did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_eq!(
				Balances::reserved_balance(alice_did.clone()),
				<Test as did::Config>::BaseDeposit::get() * 2
			);
			assert_noop!(
				Did::update_deposit(RuntimeOrigin::signed(bob_did.clone()), alice_did.clone()),
				Error::<Test>::BadDidOrigin
			);
		});
}
