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
use sp_runtime::traits::Zero;

use crate::{
	self as did, did_details::DidVerificationKey, mock::*, mock_utils::*, service_endpoints::DidEndpoint, HoldReason,
};

#[test]
fn check_successful_deletion_no_endpoints() {
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());

	let mut did_details = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);
	did_details.deposit.owner = ACCOUNT_00;
	did_details.deposit.amount = <Test as did::Config>::BaseDeposit::get();

	let balance = <Test as did::Config>::BaseDeposit::get() * 2
		+ <Test as did::Config>::Fee::get() * 2
		+ <<Test as did::Config>::Currency as Inspect<did::AccountIdOf<Test>>>::minimum_balance();

	let origin = build_test_origin(alice_did.clone(), alice_did.clone());

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, balance)])
		.with_dids(vec![(alice_did.clone(), did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_eq!(did::pallet::DidEndpointsCount::<Test>::get(&alice_did), 0);
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				<Test as did::Config>::BaseDeposit::get()
			);
			assert_ok!(Did::delete(origin, 0));
			assert!(Did::get_did(alice_did.clone()).is_none());
			assert!(Did::get_deleted_did(alice_did.clone()).is_some());
			assert!(Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00).is_zero());

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
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let service_endpoint = DidEndpoint::new(b"id".to_vec(), vec![b"type".to_vec()], vec![b"url".to_vec()]);

	let mut did_details = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);
	did_details.deposit.owner = ACCOUNT_00;
	did_details.deposit.amount = <Test as did::Config>::BaseDeposit::get();

	let balance = <Test as did::Config>::BaseDeposit::get() * 2
		+ <Test as did::Config>::Fee::get() * 2
		+ <<Test as did::Config>::Currency as Inspect<did::AccountIdOf<Test>>>::minimum_balance();

	let origin = build_test_origin(alice_did.clone(), alice_did.clone());

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, balance)])
		.with_dids(vec![(alice_did.clone(), did_details)])
		.with_endpoints(vec![(alice_did.clone(), vec![service_endpoint])])
		.build_and_execute_with_sanity_tests(None, || {
			assert_eq!(did::pallet::DidEndpointsCount::<Test>::get(&alice_did), 1);
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				<Test as did::Config>::BaseDeposit::get()
			);
			assert_ok!(Did::delete(origin, 1));
			assert!(Did::get_did(alice_did.clone()).is_none());
			assert!(Did::get_deleted_did(alice_did.clone()).is_some());
			assert!(Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00).is_zero());

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
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());

	let balance = <Test as did::Config>::BaseDeposit::get()
		+ <Test as did::Config>::Fee::get()
		+ <<Test as did::Config>::Currency as Inspect<did::AccountIdOf<Test>>>::minimum_balance();

	let origin = build_test_origin(alice_did.clone(), alice_did);

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, balance)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(Did::delete(origin, 0), did::Error::<Test>::NotFound);
		});
}

#[test]
fn check_service_count_too_small_deletion_error() {
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let service_endpoint = DidEndpoint::new(b"id".to_vec(), vec![b"type".to_vec()], vec![b"url".to_vec()]);

	let mut did_details = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), None);
	did_details.deposit.owner = ACCOUNT_00;
	did_details.deposit.amount = <Test as did::Config>::BaseDeposit::get();

	let balance = <Test as did::Config>::BaseDeposit::get() * 2
		+ <Test as did::Config>::Fee::get() * 2
		+ <<Test as did::Config>::Currency as Inspect<did::AccountIdOf<Test>>>::minimum_balance();

	let origin = build_test_origin(alice_did.clone(), alice_did.clone());

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, balance)])
		.with_dids(vec![(alice_did.clone(), did_details)])
		.with_endpoints(vec![(alice_did, vec![service_endpoint])])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::delete(origin, 0),
				did::Error::<Test>::MaxStoredEndpointsCountExceeded
			);
		});
}
