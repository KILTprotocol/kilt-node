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
use frame_support::{assert_err, assert_ok};
use frame_system::{pallet_prelude::OriginFor, RawOrigin};

use crate::{
	mock::{runtime::*, *},
	types::{Locks, PoolStatus},
	AccountIdOf, Error, Event, Pools,
};

#[test]
fn set_lock_works() {
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
		.with_bonded_balance(vec![
			(DEFAULT_COLLATERAL_CURRENCY_ID, pool_id.clone(), u128::MAX / 10),
			(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_00, u128::MAX / 10),
		])
		.build_and_execute_with_sanity_tests(|| {
			let origin = RawOrigin::Signed(ACCOUNT_00).into();

			assert_ok!(BondingPallet::set_lock(origin, pool_id.clone(), Default::default()));

			// Verify that the pool state has been updated to locked
			let updated_pool = Pools::<Test>::get(&pool_id).unwrap();
			assert!(matches!(updated_pool.state, PoolStatus::Locked(_)));

			// Verify the expected event has been deposited
			System::assert_last_event(
				Event::LockSet {
					id: pool_id,
					lock: Default::default(),
				}
				.into(),
			);
		});
}

#[test]
fn set_lock_works_when_locked() {
	let pool_details = generate_pool_details(
		vec![DEFAULT_BONDED_CURRENCY_ID],
		get_linear_bonding_curve(),
		true,
		Some(PoolStatus::Locked(Locks {
			allow_mint: true,
			allow_burn: false,
		})),
		Some(ACCOUNT_00),
		Some(DEFAULT_COLLATERAL_CURRENCY_ID),
		Some(ACCOUNT_00),
		None,
	);
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	let new_state = Locks {
		allow_mint: false,
		allow_burn: true,
	};

	ExtBuilder::default()
		.with_pools(vec![(pool_id.clone(), pool_details)])
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![
			(DEFAULT_COLLATERAL_CURRENCY_ID, pool_id.clone(), u128::MAX / 10),
			(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_00, u128::MAX / 10),
		])
		.build_and_execute_with_sanity_tests(|| {
			let origin = RawOrigin::Signed(ACCOUNT_00).into();

			assert_ok!(BondingPallet::set_lock(origin, pool_id.clone(), new_state.clone()));

			// Verify that the pool state has been updated to locked
			let updated_pool = Pools::<Test>::get(&pool_id).unwrap();
			assert_eq!(updated_pool.state, PoolStatus::Locked(new_state.clone()));

			// Verify the expected event has been deposited
			System::assert_last_event(
				Event::LockSet {
					id: pool_id,
					lock: new_state,
				}
				.into(),
			);
		});
}

#[test]
fn set_lock_requires_at_least_one_flag_set() {
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
		.with_bonded_balance(vec![
			(DEFAULT_COLLATERAL_CURRENCY_ID, pool_id.clone(), u128::MAX / 10),
			(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_00, u128::MAX / 10),
		])
		.build_and_execute_with_sanity_tests(|| {
			let origin = RawOrigin::Signed(ACCOUNT_00).into();

			assert_err!(
				BondingPallet::set_lock(
					origin,
					pool_id.clone(),
					Locks {
						allow_burn: true,
						allow_mint: true
					}
				),
				Error::<Test>::InvalidInput
			);
		});
}

#[test]
fn set_lock_fails_when_not_authorized() {
	let pool_details = generate_pool_details(
		vec![DEFAULT_BONDED_CURRENCY_ID],
		get_linear_bonding_curve(),
		true,
		Some(PoolStatus::Active),
		Some(ACCOUNT_99),
		Some(DEFAULT_COLLATERAL_CURRENCY_ID),
		Some(ACCOUNT_00),
		None,
	);
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	ExtBuilder::default()
		.with_pools(vec![(pool_id.clone(), pool_details)])
		.with_native_balances(vec![(ACCOUNT_01, ONE_HUNDRED_KILT), (ACCOUNT_00, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![
			(DEFAULT_COLLATERAL_CURRENCY_ID, pool_id.clone(), u128::MAX / 10),
			(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_01, u128::MAX / 10),
		])
		.build_and_execute_with_sanity_tests(|| {
			// Does not work for owner
			assert_err!(
				BondingPallet::set_lock(
					RawOrigin::Signed(ACCOUNT_00).into(),
					pool_id.clone(),
					Default::default()
				),
				Error::<Test>::NoPermission
			);
			// Does not work for some other account
			assert_err!(
				BondingPallet::set_lock(
					RawOrigin::Signed(ACCOUNT_01).into(),
					pool_id.clone(),
					Default::default()
				),
				Error::<Test>::NoPermission
			);
		});
}

#[test]
fn set_lock_fails_when_not_live() {
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
		.with_bonded_balance(vec![
			(DEFAULT_COLLATERAL_CURRENCY_ID, pool_id.clone(), u128::MAX / 10),
			(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_00, u128::MAX / 10),
		])
		.build_and_execute_with_sanity_tests(|| {
			let origin: OriginFor<Test> = RawOrigin::Signed(ACCOUNT_00).into();

			// Ensure the unlock call fails due to the pool not being in a 'live' state
			assert_err!(
				BondingPallet::set_lock(origin.clone(), pool_id.clone(), Default::default()),
				Error::<Test>::PoolNotLive
			);

			Pools::<Test>::mutate(&pool_id, |details| {
				details.as_mut().unwrap().state.start_destroy();
			});

			assert_err!(
				BondingPallet::set_lock(origin, pool_id.clone(), Default::default()),
				Error::<Test>::PoolNotLive
			);
		});
}
