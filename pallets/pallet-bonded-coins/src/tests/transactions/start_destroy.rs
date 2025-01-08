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
use frame_support::{assert_err, assert_ok, traits::fungibles::Destroy};
use frame_system::{pallet_prelude::OriginFor, RawOrigin};
use sp_runtime::traits::BadOrigin;

use crate::{
	mock::{runtime::*, *},
	types::PoolStatus,
	AccountIdOf, Error, Event, Pools,
};

#[test]
fn start_destroy_works() {
	let pool_details = generate_pool_details(
		vec![DEFAULT_BONDED_CURRENCY_ID],
		get_linear_bonding_curve(),
		true,
		Some(PoolStatus::Active),
		Some(ACCOUNT_00),
		Some(DEFAULT_COLLATERAL_CURRENCY_ID),
		Some(ACCOUNT_00),
		None,
	);
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	ExtBuilder::default()
		.with_pools(vec![(pool_id.clone(), pool_details)])
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.build_and_execute_with_sanity_tests(|| {
			let origin = RawOrigin::Signed(ACCOUNT_00).into();

			assert_ok!(BondingPallet::start_destroy(origin, pool_id.clone(), 1));

			// Verify that the pool state has been updated to destroying
			let updated_pool = Pools::<Test>::get(&pool_id).unwrap();
			assert_eq!(updated_pool.state, PoolStatus::Destroying);

			// Verify the expected event has been deposited
			System::assert_has_event(Event::DestructionStarted { id: pool_id }.into());

			// There is no direct way to check the asset status, so we assert that it has
			// been moved to destroying state by calling a function that would otherwise
			// fail.
			assert_ok!(<Assets as Destroy<AccountId>>::destroy_accounts(
				DEFAULT_BONDED_CURRENCY_ID,
				1
			));
		});
}

#[test]
fn start_destroy_works_when_nothing_to_refund() {
	let pool_details = generate_pool_details(
		vec![DEFAULT_BONDED_CURRENCY_ID],
		get_linear_bonding_curve(),
		true,
		Some(PoolStatus::Active),
		Some(ACCOUNT_00),
		Some(DEFAULT_COLLATERAL_CURRENCY_ID),
		Some(ACCOUNT_00),
		None,
	);
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	ExtBuilder::default()
		.with_pools(vec![(pool_id.clone(), pool_details)])
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![(DEFAULT_COLLATERAL_CURRENCY_ID, pool_id.clone(), u128::MAX / 10)])
		.build_and_execute_with_sanity_tests(|| {
			let origin = RawOrigin::Signed(ACCOUNT_00).into();

			assert_ok!(BondingPallet::start_destroy(origin, pool_id.clone(), 1));

			// Verify that the pool state has been updated to destroying
			let updated_pool = Pools::<Test>::get(&pool_id).unwrap();
			assert_eq!(updated_pool.state, PoolStatus::Destroying);

			// Verify the expected event has been deposited
			System::assert_has_event(Event::DestructionStarted { id: pool_id }.into());
		});
}

#[test]
fn start_destroy_works_when_no_collateral() {
	let pool_details = generate_pool_details(
		vec![DEFAULT_BONDED_CURRENCY_ID],
		get_linear_bonding_curve(),
		true,
		Some(PoolStatus::Active),
		Some(ACCOUNT_00),
		Some(DEFAULT_COLLATERAL_CURRENCY_ID),
		Some(ACCOUNT_00),
		None,
	);
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	ExtBuilder::default()
		.with_pools(vec![(pool_id.clone(), pool_details)])
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_00, u128::MAX / 10)])
		.build_and_execute_with_sanity_tests(|| {
			let origin = RawOrigin::Signed(ACCOUNT_00).into();

			assert_ok!(BondingPallet::start_destroy(origin, pool_id.clone(), 1));

			// Verify that the pool state has been updated to destroying
			let updated_pool = Pools::<Test>::get(&pool_id).unwrap();
			assert_eq!(updated_pool.state, PoolStatus::Destroying);

			// Verify the expected event has been deposited
			System::assert_has_event(Event::DestructionStarted { id: pool_id }.into());
		});
}

#[test]
fn start_destroy_works_when_refunding() {
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
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![(DEFAULT_COLLATERAL_CURRENCY_ID, pool_id.clone(), u128::MAX / 10)])
		.build_and_execute_with_sanity_tests(|| {
			let origin = RawOrigin::Signed(ACCOUNT_00).into();

			assert_ok!(BondingPallet::start_destroy(origin, pool_id.clone(), 1));

			// Verify that the pool state has been updated to destroying
			let updated_pool = Pools::<Test>::get(&pool_id).unwrap();
			assert_eq!(updated_pool.state, PoolStatus::Destroying);

			// Verify the expected event has been deposited
			System::assert_has_event(Event::DestructionStarted { id: pool_id }.into());
		});
}

#[test]
fn start_destroy_fails_when_pool_has_active_currencies() {
	let pool_details = generate_pool_details(
		vec![DEFAULT_BONDED_CURRENCY_ID],
		get_linear_bonding_curve(),
		false,
		Some(PoolStatus::Refunding),
		Some(ACCOUNT_00),
		None,
		None,
		None,
	);
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT)])
		.with_pools(vec![(pool_id.clone(), pool_details)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![
			(DEFAULT_COLLATERAL_CURRENCY_ID, pool_id.clone(), ONE_HUNDRED_KILT),
			(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_00, ONE_HUNDRED_KILT),
		])
		.build_and_execute_with_sanity_tests(|| {
			let origin: OriginFor<Test> = RawOrigin::Signed(ACCOUNT_00).into();

			// Ensure the start_destroy call fails due to pool being actively used
			assert_err!(
				BondingPallet::start_destroy(origin, pool_id.clone(), 1),
				Error::<Test>::LivePool
			);

			<Assets as Destroy<AccountId>>::destroy_accounts(DEFAULT_BONDED_CURRENCY_ID, 1).unwrap_err();
		});
}

#[test]
fn start_destroy_fails_when_pool_destroying() {
	let pool_details = generate_pool_details(
		vec![DEFAULT_BONDED_CURRENCY_ID],
		get_linear_bonding_curve(),
		false,
		Some(PoolStatus::Destroying),
		Some(ACCOUNT_00),
		None,
		None,
		None,
	);
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT)])
		.with_pools(vec![(pool_id.clone(), pool_details)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.build_and_execute_with_sanity_tests(|| {
			let origin: OriginFor<Test> = RawOrigin::Signed(ACCOUNT_00).into();

			// Ensure the start_destroy call fails due to pool not being active
			assert_err!(
				BondingPallet::start_destroy(origin, pool_id.clone(), 1),
				Error::<Test>::PoolNotLive
			);
		});
}

#[test]
fn start_destroy_fails_when_currency_no_low() {
	let currencies = vec![
		DEFAULT_BONDED_CURRENCY_ID,
		DEFAULT_BONDED_CURRENCY_ID + 1,
		DEFAULT_BONDED_CURRENCY_ID + 2,
	];
	let pool_details = generate_pool_details(
		currencies.clone(),
		get_linear_bonding_curve(),
		false,
		Some(PoolStatus::Active),
		Some(ACCOUNT_00),
		None,
		None,
		None,
	);
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&currencies);

	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT)])
		.with_pools(vec![(pool_id.clone(), pool_details)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![(
			DEFAULT_COLLATERAL_CURRENCY_ID,
			pool_id.clone(),
			ONE_HUNDRED_KILT,
		)])
		.build_and_execute_with_sanity_tests(|| {
			let origin: OriginFor<Test> = RawOrigin::Signed(ACCOUNT_00).into();

			assert_err!(
				BondingPallet::start_destroy(origin.clone(), pool_id.clone(), 1),
				Error::<Test>::CurrencyCount
			);

			assert_err!(
				BondingPallet::start_destroy(origin.clone(), pool_id.clone(), 2),
				Error::<Test>::CurrencyCount
			);

			assert_ok!(BondingPallet::start_destroy(origin, pool_id.clone(), 3),);
		});
}

#[test]
fn force_start_destroy_works() {
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
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT)])
		.with_pools(vec![(pool_id.clone(), pool_details)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![(
			DEFAULT_COLLATERAL_CURRENCY_ID,
			pool_id.clone(),
			ONE_HUNDRED_KILT,
		)])
		.build_and_execute_with_sanity_tests(|| {
			let origin = RawOrigin::Root.into();

			assert_ok!(BondingPallet::force_start_destroy(origin, pool_id.clone(), 1));

			// Verify that the pool state has been updated to destroying
			let updated_pool = Pools::<Test>::get(&pool_id).unwrap();
			assert_eq!(updated_pool.state, PoolStatus::Destroying);

			// Verify the expected event has been deposited
			System::assert_has_event(Event::DestructionStarted { id: pool_id }.into());
		});
}

#[test]
fn force_start_destroy_works_even_with_nonzero_supply() {
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
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT)])
		.with_pools(vec![(pool_id.clone(), pool_details)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![
			(DEFAULT_COLLATERAL_CURRENCY_ID, pool_id.clone(), ONE_HUNDRED_KILT),
			(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_00, ONE_HUNDRED_KILT),
		])
		.build_and_execute_with_sanity_tests(|| {
			let origin = RawOrigin::Root.into();

			assert_ok!(BondingPallet::force_start_destroy(origin, pool_id.clone(), 1));

			// Verify that the pool state has been updated to destroying
			let updated_pool = Pools::<Test>::get(&pool_id).unwrap();
			assert_eq!(updated_pool.state, PoolStatus::Destroying);

			// Verify the expected event has been deposited
			System::assert_has_event(Event::DestructionStarted { id: pool_id }.into());
		});
}

#[test]
fn force_start_destroy_fails_when_not_root() {
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
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT)])
		.with_pools(vec![(pool_id.clone(), pool_details)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![
			(DEFAULT_COLLATERAL_CURRENCY_ID, pool_id.clone(), ONE_HUNDRED_KILT),
			(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_00, ONE_HUNDRED_KILT),
		])
		.build_and_execute_with_sanity_tests(|| {
			let origin = RawOrigin::Signed(ACCOUNT_00).into();

			// Ensure the force_start_destroy call fails due to non-root origin
			assert_err!(BondingPallet::force_start_destroy(origin, pool_id, 1), BadOrigin);
		});
}

#[test]
fn force_start_destroy_fails_when_currency_no_low() {
	let currencies = vec![
		DEFAULT_BONDED_CURRENCY_ID,
		DEFAULT_BONDED_CURRENCY_ID + 1,
		DEFAULT_BONDED_CURRENCY_ID + 2,
	];
	let pool_details = generate_pool_details(
		currencies.clone(),
		get_linear_bonding_curve(),
		false,
		Some(PoolStatus::Active),
		Some(ACCOUNT_00),
		None,
		None,
		None,
	);
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&currencies);

	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT)])
		.with_pools(vec![(pool_id.clone(), pool_details)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![(
			DEFAULT_COLLATERAL_CURRENCY_ID,
			pool_id.clone(),
			ONE_HUNDRED_KILT,
		)])
		.build_and_execute_with_sanity_tests(|| {
			let origin: OriginFor<Test> = RawOrigin::Root.into();

			assert_err!(
				BondingPallet::force_start_destroy(origin.clone(), pool_id.clone(), 1),
				Error::<Test>::CurrencyCount
			);

			assert_err!(
				BondingPallet::force_start_destroy(origin.clone(), pool_id.clone(), 2),
				Error::<Test>::CurrencyCount
			);

			assert_ok!(BondingPallet::force_start_destroy(origin, pool_id.clone(), 3),);
		});
}
