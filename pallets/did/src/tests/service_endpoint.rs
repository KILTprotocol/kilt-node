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
use sp_core::Pair;
use sp_runtime::{SaturatedConversion, TokenError};
use sp_std::convert::TryInto;

use crate::{
	self as did, did_details::DidVerificationKey, mock::*, mock_utils::*, service_endpoints::DidEndpoint, HoldReason,
};

#[test]
fn check_deposit_change_by_adding_service_endpoint() {
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let mut did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(alice_did.clone()));
	did_details.deposit.amount = <Test as did::Config>::BaseDeposit::get();

	let new_service_endpoint = DidEndpoint::new(b"id".to_vec(), vec![b"type".to_vec()], vec![b"url".to_vec()]);
	let new_service_endpoint2: DidEndpoint<Test> =
		DidEndpoint::new(b"id2".to_vec(), vec![b"type2".to_vec()], vec![b"url2".to_vec()]);

	let balance = <Test as did::Config>::BaseDeposit::get()
		+ <Test as did::Config>::ServiceEndpointDeposit::get()
		+ <<Test as did::Config>::Currency as Inspect<did::AccountIdOf<Test>>>::minimum_balance();

	let origin = build_test_origin(alice_did.clone(), alice_did.clone());

	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), balance)])
		.with_dids(vec![(alice_did.clone(), did_details)])
		.build(None)
		.execute_with(|| {
			assert_ok!(Did::add_service_endpoint(origin.clone(), new_service_endpoint.clone()));

			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &alice_did),
				<Test as did::Config>::ServiceEndpointDeposit::get() + <Test as did::Config>::BaseDeposit::get()
			);

			assert_eq!(
				Balances::balance(&alice_did),
				<<Test as did::Config>::Currency as Inspect<did::AccountIdOf<Test>>>::minimum_balance()
			);

			assert_noop!(
				Did::add_service_endpoint(origin, new_service_endpoint2.clone()),
				TokenError::FundsUnavailable
			);

			assert!(did::ServiceEndpoints::<Test>::get(alice_did.clone(), new_service_endpoint.id).is_some());

			assert!(did::ServiceEndpoints::<Test>::get(alice_did, new_service_endpoint2.id).is_none());
		});
}

#[test]
fn check_service_addition_no_prior_service_successful() {
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let new_service_endpoint = DidEndpoint::new(b"id".to_vec(), vec![b"type".to_vec()], vec![b"url".to_vec()]);

	let old_did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(alice_did.clone()));

	let origin = build_test_origin(alice_did.clone(), alice_did.clone());

	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(alice_did.clone(), old_did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_ok!(Did::add_service_endpoint(origin, new_service_endpoint.clone()));
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
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
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

	let origin = build_test_origin(alice_did.clone(), alice_did.clone());

	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(alice_did.clone(), old_did_details)])
		.with_endpoints(vec![(alice_did.clone(), old_service_endpoints)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_ok!(Did::add_service_endpoint(origin, new_service_endpoint.clone()));
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
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let new_service_endpoint = DidEndpoint::new(b"id".to_vec(), vec![b"type".to_vec()], vec![b"url".to_vec()]);

	let origin = build_test_origin(alice_did.clone(), alice_did.clone());

	ExtBuilder::default()
		.with_balances(vec![(alice_did, DEFAULT_BALANCE)])
		.build(None)
		.execute_with(|| {
			assert_noop!(
				Did::add_service_endpoint(origin, new_service_endpoint),
				did::Error::<Test>::NotFound
			);
		});
}

#[test]
fn check_service_already_present_addition_error() {
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let service_endpoint = DidEndpoint::new(b"id".to_vec(), vec![b"type".to_vec()], vec![b"url".to_vec()]);

	let old_did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(alice_did.clone()));

	let origin = build_test_origin(alice_did.clone(), alice_did.clone());

	ExtBuilder::default()
		.with_dids(vec![(alice_did.clone(), old_did_details)])
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_endpoints(vec![(alice_did, vec![service_endpoint.clone()])])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::add_service_endpoint(origin, service_endpoint),
				did::Error::<Test>::ServiceAlreadyExists
			);
		});
}

#[test]
fn check_max_services_count_addition_error() {
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
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

	let old_did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(alice_did.clone()));

	let origin = build_test_origin(alice_did.clone(), alice_did.clone());

	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(alice_did.clone(), old_did_details)])
		.with_endpoints(vec![(alice_did, old_service_endpoints)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::add_service_endpoint(origin, new_service_endpoint),
				did::Error::<Test>::MaxNumberOfServicesExceeded
			);
		});
}

#[test]
#[should_panic = "Service ID too long."]
fn check_max_service_id_length_addition_error() {
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
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

	let origin = build_test_origin(alice_did.clone(), alice_did.clone());

	ExtBuilder::default()
		.with_dids(vec![(alice_did, old_did_details)])
		.build(None)
		.execute_with(|| {
			assert_noop!(
				Did::add_service_endpoint(origin, new_service_endpoint),
				did::Error::<Test>::MaxServiceIdLengthExceeded
			);
		});
}

#[test]
#[should_panic = "Service type too long."]
fn check_max_service_type_length_addition_error() {
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
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

	let origin = build_test_origin(alice_did.clone(), alice_did.clone());

	ExtBuilder::default()
		.with_dids(vec![(alice_did, old_did_details)])
		.build(None)
		.execute_with(|| {
			assert_noop!(
				Did::add_service_endpoint(origin, new_service_endpoint),
				did::Error::<Test>::MaxServiceTypeLengthExceeded
			);
		});
}

#[test]
#[should_panic = "Too many types for the given service."]
fn check_max_service_type_count_addition_error() {
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
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

	let origin = build_test_origin(alice_did.clone(), alice_did.clone());

	ExtBuilder::default()
		.with_dids(vec![(alice_did, old_did_details)])
		.build(None)
		.execute_with(|| {
			assert_noop!(
				Did::add_service_endpoint(origin, new_service_endpoint),
				did::Error::<Test>::MaxNumberOfTypesPerServiceExceeded
			);
		});
}

#[test]
#[should_panic = "Service URL too long."]
fn check_max_service_url_length_addition_error() {
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
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

	let origin = build_test_origin(alice_did.clone(), alice_did.clone());

	ExtBuilder::default()
		.with_dids(vec![(alice_did, old_did_details)])
		.build(None)
		.execute_with(|| {
			assert_noop!(
				Did::add_service_endpoint(origin, new_service_endpoint),
				did::Error::<Test>::MaxServiceUrlLengthExceeded
			);
		});
}

#[test]
#[should_panic = "Too many URLs for the given service."]
fn check_max_service_url_count_addition_error() {
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
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

	let origin = build_test_origin(alice_did.clone(), alice_did.clone());

	ExtBuilder::default()
		.with_dids(vec![(alice_did, old_did_details)])
		.build(None)
		.execute_with(|| {
			assert_noop!(
				Did::add_service_endpoint(origin, new_service_endpoint),
				did::Error::<Test>::MaxNumberOfUrlsPerServiceExceeded
			);
		});
}

#[test]
fn character_addition_error() {
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let new_service_details = DidEndpoint::new("å".bytes().collect(), vec![b"type".to_vec()], vec![b"url".to_vec()]);

	let old_did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(alice_did.clone()));

	let origin = build_test_origin(alice_did.clone(), alice_did.clone());

	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(alice_did, old_did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::add_service_endpoint(origin, new_service_details),
				did::Error::<Test>::InvalidServiceEncoding
			);
		});
}

#[test]
fn check_invalid_service_type_character_addition_error() {
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let new_service_details = DidEndpoint::new(b"id".to_vec(), vec!["å".bytes().collect()], vec![b"url".to_vec()]);

	let old_did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(alice_did.clone()));

	let origin = build_test_origin(alice_did.clone(), alice_did.clone());

	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(alice_did, old_did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::add_service_endpoint(origin, new_service_details),
				did::Error::<Test>::InvalidServiceEncoding
			);
		});
}

#[test]
fn check_invalid_service_url_character_addition_error() {
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let new_service_details = DidEndpoint::new(b"id".to_vec(), vec![b"type".to_vec()], vec!["å".bytes().collect()]);

	let old_did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(alice_did.clone()));

	let origin = build_test_origin(alice_did.clone(), alice_did.clone());

	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(alice_did, old_did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::add_service_endpoint(origin, new_service_details),
				did::Error::<Test>::InvalidServiceEncoding
			);
		});
}

// remove_service_endpoint

#[test]
fn check_service_deletion_successful() {
	initialize_logger();
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let old_service_endpoint = DidEndpoint::new(b"id".to_vec(), vec![b"type".to_vec()], vec![b"url".to_vec()]);

	let old_did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(alice_did.clone()));

	let origin = build_test_origin(alice_did.clone(), alice_did.clone());

	ExtBuilder::default()
		.with_dids(vec![(alice_did.clone(), old_did_details)])
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_endpoints(vec![(alice_did.clone(), vec![old_service_endpoint.clone()])])
		.build_and_execute_with_sanity_tests(None, || {
			assert_ok!(Did::remove_service_endpoint(origin, old_service_endpoint.id));
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
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let service_id = b"id".to_vec();

	let old_did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(alice_did.clone()));

	let origin = build_test_origin(alice_did.clone(), alice_did.clone());

	ExtBuilder::default()
		.with_dids(vec![(alice_did.clone(), old_did_details)])
		.with_balances(vec![(alice_did, DEFAULT_BALANCE)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::remove_service_endpoint(origin, service_id.try_into().expect("Service ID to delete too long")),
				did::Error::<Test>::ServiceNotFound
			);
		});
}
