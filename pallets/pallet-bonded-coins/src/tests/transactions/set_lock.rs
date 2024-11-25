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
	);
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	ExtBuilder::default()
		.with_pools(vec![(pool_id.clone(), pool_details.clone())])
		.with_native_balances(vec![(ACCOUNT_00, u128::MAX)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![
			(DEFAULT_COLLATERAL_CURRENCY_ID, pool_id.clone(), u128::MAX / 10),
			(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_00, u128::MAX / 10),
		])
		.build()
		.execute_with(|| {
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
			allow_swap: false,
		})),
		Some(ACCOUNT_00),
		Some(DEFAULT_COLLATERAL_CURRENCY_ID),
		Some(ACCOUNT_00),
	);
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	let new_state = Locks {
		allow_mint: false,
		allow_burn: true,
		allow_swap: false,
	};

	ExtBuilder::default()
		.with_pools(vec![(pool_id.clone(), pool_details.clone())])
		.with_native_balances(vec![(ACCOUNT_00, u128::MAX)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![
			(DEFAULT_COLLATERAL_CURRENCY_ID, pool_id.clone(), u128::MAX / 10),
			(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_00, u128::MAX / 10),
		])
		.build()
		.execute_with(|| {
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
fn set_lock_fails_when_not_authorized() {
	let pool_details = generate_pool_details(
		vec![DEFAULT_BONDED_CURRENCY_ID],
		get_linear_bonding_curve(),
		true,
		Some(PoolStatus::Active),
		Some(ACCOUNT_99),
		Some(DEFAULT_COLLATERAL_CURRENCY_ID),
		Some(ACCOUNT_00),
	);
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	ExtBuilder::default()
		.with_pools(vec![(pool_id.clone(), pool_details.clone())])
		.with_native_balances(vec![(ACCOUNT_01, u128::MAX)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![
			(DEFAULT_COLLATERAL_CURRENCY_ID, pool_id.clone(), u128::MAX / 10),
			(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_01, u128::MAX / 10),
		])
		.build()
		.execute_with(|| {
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
	);
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	ExtBuilder::default()
		.with_pools(vec![(pool_id.clone(), pool_details.clone())])
		.with_native_balances(vec![(ACCOUNT_00, u128::MAX)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![
			(DEFAULT_COLLATERAL_CURRENCY_ID, pool_id.clone(), u128::MAX / 10),
			(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_00, u128::MAX / 10),
		])
		.build()
		.execute_with(|| {
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
