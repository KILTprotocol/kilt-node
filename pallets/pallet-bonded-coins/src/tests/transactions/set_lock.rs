use frame_support::assert_ok;

use crate::{
	mock::{runtime::*, *},
	types::{Locks, PoolStatus},
	Pools,
};

#[test]
fn test_set_lock() {
	let currencies = vec![DEFAULT_BONDED_CURRENCY_ID];
	let pool_id = calculate_pool_id(currencies.clone());

	let curve = get_linear_bonding_curve();

	let pool = calculate_pool_details(currencies, ACCOUNT_01, false, curve.clone(), PoolStatus::Active);

	let target_lock: Locks = Default::default();

	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_01, UNIT_NATIVE * 10), (pool_id.clone(), UNIT_NATIVE)])
		.with_collateral_asset_id(DEFAULT_COLLATERAL_CURRENCY_ID)
		.with_currencies(vec![vec![DEFAULT_BONDED_CURRENCY_ID]])
		.with_metadata(vec![
			(DEFAULT_COLLATERAL_CURRENCY_ID, DEFAULT_COLLATERAL_DENOMINATION),
			(DEFAULT_BONDED_CURRENCY_ID, DEFAULT_BONDED_DENOMINATION),
		])
		.with_pools(vec![(pool_id.clone(), pool)])
		.build()
		.execute_with(|| {
			let pool_details = Pools::<Test>::get(&pool_id).expect("Pool should exist");

			assert_eq!(pool_details.state, PoolStatus::Active);
			assert_ok!(BondingPallet::set_lock(
				RuntimeOrigin::signed(ACCOUNT_01),
				pool_id.clone(),
				target_lock.clone()
			));

			let pool_details_after_tx = Pools::<Test>::get(&pool_id).expect("Pool should exist");

			assert_eq!(pool_details_after_tx.state, PoolStatus::Locked(target_lock));

			// check events
			assert_eq!(events(), vec![crate::Event::<Test>::LockSet(pool_id)])
		});
}
