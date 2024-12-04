// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2024 BOTLabs GmbH

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
	assert_err, assert_ok,
	traits::fungibles::{Create, Inspect, Mutate},
};
use frame_system::{pallet_prelude::OriginFor, RawOrigin};
use sp_runtime::TokenError;

use crate::{
	mock::{runtime::*, *},
	traits::FreezeAccounts,
	types::PoolStatus,
	AccountIdOf, Error, Event, Pools,
};

#[test]
fn refund_account_works() {
	let pool_details = generate_pool_details(
		vec![DEFAULT_BONDED_CURRENCY_ID],
		get_linear_bonding_curve(),
		true,
		Some(PoolStatus::Refunding),
		Some(ACCOUNT_00),
		Some(DEFAULT_COLLATERAL_CURRENCY_ID),
		Some(ACCOUNT_00),
		None,
	);
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	let total_collateral = 10u128.pow(10);

	ExtBuilder::default()
		.with_pools(vec![(pool_id.clone(), pool_details)])
		.with_native_balances(vec![(ACCOUNT_01, ONE_HUNDRED_KILT), (ACCOUNT_00, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![
			(DEFAULT_COLLATERAL_CURRENCY_ID, pool_id.clone(), total_collateral),
			(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_01, total_collateral * 10),
		])
		.build_and_execute_with_sanity_tests(|| {
			let origin = RawOrigin::Signed(ACCOUNT_01).into();

			assert_ok!(BondingPallet::refund_account(origin, pool_id.clone(), ACCOUNT_01, 0, 1));

			assert_eq!(Assets::total_balance(DEFAULT_BONDED_CURRENCY_ID, &ACCOUNT_01), 0);

			assert_eq!(
				Assets::total_balance(DEFAULT_COLLATERAL_CURRENCY_ID, &ACCOUNT_01),
				total_collateral
			);

			// There's only one account, so this should complete the refund
			System::assert_has_event(Event::<Test>::RefundComplete { id: pool_id }.into());
		});
}

#[test]
fn refund_account_works_on_frozen() {
	let pool_details = generate_pool_details(
		vec![DEFAULT_BONDED_CURRENCY_ID],
		get_linear_bonding_curve(),
		false,
		Some(PoolStatus::Refunding),
		Some(ACCOUNT_00),
		Some(DEFAULT_COLLATERAL_CURRENCY_ID),
		Some(ACCOUNT_00),
		None,
	);
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	let total_collateral = 10u128.pow(10);

	ExtBuilder::default()
		.with_pools(vec![(pool_id.clone(), pool_details)])
		.with_native_balances(vec![(ACCOUNT_01, ONE_HUNDRED_KILT), (ACCOUNT_00, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![
			(DEFAULT_COLLATERAL_CURRENCY_ID, pool_id.clone(), total_collateral),
			(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_01, total_collateral * 10),
		])
		.build_and_execute_with_sanity_tests(|| {
			<Assets as FreezeAccounts<_, _>>::freeze(&DEFAULT_BONDED_CURRENCY_ID, &ACCOUNT_01)
				.expect("failed to freeze account prior to testing");

			let origin = RawOrigin::Signed(ACCOUNT_01).into();

			assert_ok!(BondingPallet::refund_account(origin, pool_id.clone(), ACCOUNT_01, 0, 1));

			assert_eq!(Assets::total_balance(DEFAULT_BONDED_CURRENCY_ID, &ACCOUNT_01), 0);

			assert_eq!(
				Assets::total_balance(DEFAULT_COLLATERAL_CURRENCY_ID, &ACCOUNT_01),
				total_collateral
			);

			// There's only one account, so this should complete the refund
			System::assert_has_event(Event::<Test>::RefundComplete { id: pool_id }.into());
		});
}

#[test]
fn refund_account_works_with_large_supply() {
	let currencies = vec![DEFAULT_BONDED_CURRENCY_ID, DEFAULT_BONDED_CURRENCY_ID + 1];
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&currencies);
	let pool_details = generate_pool_details(
		currencies.clone(),
		get_linear_bonding_curve(),
		true,
		Some(PoolStatus::Refunding),
		Some(ACCOUNT_00),
		Some(DEFAULT_COLLATERAL_CURRENCY_ID),
		Some(ACCOUNT_00),
		None,
	);

	let total_collateral = u128::MAX / 2;

	ExtBuilder::default()
		.with_pools(vec![(pool_id.clone(), pool_details)])
		.with_native_balances(vec![(ACCOUNT_01, ONE_HUNDRED_KILT), (ACCOUNT_00, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![
			(DEFAULT_COLLATERAL_CURRENCY_ID, pool_id.clone(), total_collateral),
			(currencies[0], ACCOUNT_01, u128::MAX / 3 * 2),
			(currencies[1], ACCOUNT_01, u128::MAX / 3 * 2),
		])
		.build_and_execute_with_sanity_tests(|| {
			let origin: OriginFor<Test> = RawOrigin::Signed(ACCOUNT_01).into();

			assert_ok!(BondingPallet::refund_account(
				origin.clone(),
				pool_id.clone(),
				ACCOUNT_01,
				0,
				2
			));

			assert_eq!(Assets::total_balance(currencies[0], &ACCOUNT_01), 0);

			assert_eq!(
				Assets::total_balance(DEFAULT_COLLATERAL_CURRENCY_ID, &ACCOUNT_01),
				total_collateral / 2
			);

			// At this point we've only refunded one currency, not the other
			assert_eq!(
				events()
					.into_iter()
					.find(|ev| { matches!(ev, Event::<Test>::RefundComplete { .. }) }),
				None
			);

			assert_ok!(BondingPallet::refund_account(origin, pool_id.clone(), ACCOUNT_01, 1, 2));

			assert_eq!(Assets::total_balance(currencies[1], &ACCOUNT_01), 0);

			assert_eq!(Assets::total_balance(DEFAULT_COLLATERAL_CURRENCY_ID, &pool_id), 0);

			System::assert_has_event(Event::<Test>::RefundComplete { id: pool_id }.into());
		});
}

#[test]
fn balance_is_burnt_even_if_no_collateral_received() {
	let pool_details = generate_pool_details(
		vec![DEFAULT_BONDED_CURRENCY_ID],
		get_linear_bonding_curve(),
		true,
		Some(PoolStatus::Refunding),
		Some(ACCOUNT_00),
		Some(DEFAULT_COLLATERAL_CURRENCY_ID),
		Some(ACCOUNT_00),
		None,
	);
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	let total_collateral = 10u128;

	ExtBuilder::default()
		.with_pools(vec![(pool_id.clone(), pool_details)])
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT), (ACCOUNT_01, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![
			(DEFAULT_COLLATERAL_CURRENCY_ID, pool_id.clone(), total_collateral),
			(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_00, 20),
			(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_01, 1), // 10 / 21 = 0.48 -> no collateral for you
		])
		.build_and_execute_with_sanity_tests(|| {
			let origin = RawOrigin::Signed(ACCOUNT_01).into();

			assert_ok!(BondingPallet::refund_account(origin, pool_id.clone(), ACCOUNT_01, 0, 1));

			assert_eq!(Assets::total_balance(DEFAULT_BONDED_CURRENCY_ID, &ACCOUNT_01), 0);

			assert_eq!(Assets::total_balance(DEFAULT_COLLATERAL_CURRENCY_ID, &ACCOUNT_01), 0);

			// At this point we've only refunded one of two accounts
			assert_eq!(
				events()
					.into_iter()
					.find(|ev| { matches!(ev, Event::<Test>::RefundComplete { .. }) }),
				None
			);
		});
}

#[test]
fn refund_below_min_balance() {
	let pool_details = generate_pool_details(
		vec![DEFAULT_BONDED_CURRENCY_ID],
		get_linear_bonding_curve(),
		true,
		Some(PoolStatus::Refunding),
		Some(ACCOUNT_00),
		Some(DEFAULT_COLLATERAL_CURRENCY_ID),
		Some(ACCOUNT_00),
		None,
	);
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	ExtBuilder::default()
		.with_pools(vec![(pool_id.clone(), pool_details)])
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT), (ACCOUNT_01, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![
			(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_00, 2000),
			(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_01, 2000),
		])
		.build_and_execute_with_sanity_tests(|| {
			// change collateral to one that has a minimum balance
			let collateral_id = 101;
			assert_ok!(<Assets as Create<_>>::create(
				collateral_id,
				pool_id.clone(),
				true,
				1000
			));
			Pools::<Test>::mutate(&pool_id, |details| {
				details.as_mut().unwrap().collateral_id = collateral_id
			});
			// put less than 2*min balance in pool account
			let total_collateral = 1500;
			assert_ok!(Assets::mint_into(collateral_id, &pool_id.clone(), total_collateral));

			let origin = RawOrigin::Signed(ACCOUNT_01).into();

			assert_ok!(BondingPallet::refund_account(origin, pool_id.clone(), ACCOUNT_01, 0, 1));

			assert_eq!(Assets::total_balance(DEFAULT_BONDED_CURRENCY_ID, &ACCOUNT_01), 0);
			// each would get half, which is below minimum - should not get transferred
			assert_eq!(Assets::total_balance(collateral_id, &pool_id), total_collateral);
		});
}

#[test]
fn refund_account_fails_when_pool_not_refunding() {
	let pool_details = generate_pool_details(
		vec![DEFAULT_BONDED_CURRENCY_ID],
		get_linear_bonding_curve(),
		false,
		Some(PoolStatus::Active),
		Some(ACCOUNT_00),
		None,
		None,
		None,
	);
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_01, ONE_HUNDRED_KILT)])
		.with_pools(vec![(pool_id.clone(), pool_details)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![
			(DEFAULT_COLLATERAL_CURRENCY_ID, pool_id.clone(), ONE_HUNDRED_KILT),
			(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_01, ONE_HUNDRED_KILT),
		])
		.build_and_execute_with_sanity_tests(|| {
			let origin: OriginFor<Test> = RawOrigin::Signed(ACCOUNT_01).into();

			// Ensure the refund_account call fails due to pool not being in refunding state
			assert_err!(
				BondingPallet::refund_account(origin, pool_id.clone(), ACCOUNT_01, 0, 1),
				Error::<Test>::NotRefunding
			);
		});
}

#[test]
fn refund_account_no_balance() {
	let pool_details = generate_pool_details(
		vec![DEFAULT_BONDED_CURRENCY_ID],
		get_linear_bonding_curve(),
		true,
		Some(PoolStatus::Refunding),
		Some(ACCOUNT_00),
		Some(DEFAULT_COLLATERAL_CURRENCY_ID),
		Some(ACCOUNT_00),
		None,
	);
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	ExtBuilder::default()
		.with_pools(vec![(pool_id.clone(), pool_details)])
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT), (ACCOUNT_01, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![
			(DEFAULT_COLLATERAL_CURRENCY_ID, pool_id.clone(), ONE_HUNDRED_KILT),
			(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_00, ONE_HUNDRED_KILT),
		])
		.build_and_execute_with_sanity_tests(|| {
			let origin = RawOrigin::Signed(ACCOUNT_01).into();

			// Ensure the refund_account call fails when there is no balance to be
			// refunded
			assert_err!(
				BondingPallet::refund_account(origin, pool_id.clone(), ACCOUNT_01, 0, 1),
				TokenError::FundsUnavailable
			);
		});
}

#[test]
fn nothing_to_refund() {
	let pool_details = generate_pool_details(
		vec![DEFAULT_BONDED_CURRENCY_ID],
		get_linear_bonding_curve(),
		true,
		Some(PoolStatus::Refunding),
		Some(ACCOUNT_00),
		Some(DEFAULT_COLLATERAL_CURRENCY_ID),
		Some(ACCOUNT_00),
		None,
	);
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	// no collateral left
	ExtBuilder::default()
		.with_pools(vec![(pool_id.clone(), pool_details)])
		.with_native_balances(vec![(ACCOUNT_01, ONE_HUNDRED_KILT), (ACCOUNT_00, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![
			(DEFAULT_COLLATERAL_CURRENCY_ID, ACCOUNT_00, 100_000),
			(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_01, 100_000),
		])
		.build()
		.execute_with(|| {
			let origin = RawOrigin::Signed(ACCOUNT_01).into();

			assert_err!(
				BondingPallet::refund_account(origin, pool_id.clone(), ACCOUNT_01, 0, 1),
				Error::<Test>::NothingToRefund
			);
		});
}

#[test]
fn unknown_pool_or_currency() {
	let pool_details = generate_pool_details(
		vec![DEFAULT_BONDED_CURRENCY_ID],
		get_linear_bonding_curve(),
		true,
		Some(PoolStatus::Refunding),
		Some(ACCOUNT_00),
		Some(DEFAULT_COLLATERAL_CURRENCY_ID),
		Some(ACCOUNT_00),
		None,
	);

	let pool_id: AccountIdOf<Test> = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	let total_collateral = 10u128.pow(10);

	ExtBuilder::default()
		.with_pools(vec![(pool_id.clone(), pool_details)])
		.with_native_balances(vec![(ACCOUNT_01, ONE_HUNDRED_KILT), (ACCOUNT_00, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![
			(DEFAULT_COLLATERAL_CURRENCY_ID, pool_id.clone(), total_collateral),
			(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_01, total_collateral * 10),
		])
		.build_and_execute_with_sanity_tests(|| {
			let origin: OriginFor<Test> = RawOrigin::Signed(ACCOUNT_01).into();

			// using some other pool id
			assert_err!(
				BondingPallet::refund_account(
					origin.clone(),
					calculate_pool_id(&[DEFAULT_COLLATERAL_CURRENCY_ID]),
					ACCOUNT_01,
					0,
					1
				),
				Error::<Test>::PoolUnknown
			);

			// use the right pool id but asset idx that is too large
			assert_err!(
				BondingPallet::refund_account(origin, pool_id, ACCOUNT_01, 10, 1),
				Error::<Test>::IndexOutOfBounds
			);
		});
}
