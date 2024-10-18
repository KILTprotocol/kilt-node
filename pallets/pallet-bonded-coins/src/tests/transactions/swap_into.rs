use frame_support::assert_ok;

use crate::{
	mock::{runtime::*, *},
	pool_details::PoolStatus,
};

#[ignore]
#[test]
fn test_swap_into_non_ratio_function() {
	let second_currency_id = 1;
	let currencies = vec![DEFAULT_BONDED_CURRENCY_ID, second_currency_id];
	let pool_id = calculate_pool_id(currencies.clone());

	let one_bonded_currency = get_currency_unit(DEFAULT_BONDED_DENOMINATION);
	let one_collateral_currency = get_currency_unit(DEFAULT_COLLATERAL_DENOMINATION);

	let curve = get_linear_bonding_curve();

	let pool_details = calculate_pool_details(currencies, ACCOUNT_01, curve.clone(), PoolStatus::Active, 10);

	let collateral_balance_supply = one_collateral_currency * 10;

	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, UNIT_NATIVE * 10), (pool_id.clone(), UNIT_NATIVE)])
		.with_collateral_asset_id(DEFAULT_COLLATERAL_CURRENCY_ID)
		.with_currencies(vec![vec![DEFAULT_BONDED_CURRENCY_ID, second_currency_id]])
		.with_metadata(vec![
			(DEFAULT_COLLATERAL_CURRENCY_ID, DEFAULT_COLLATERAL_DENOMINATION),
			(DEFAULT_BONDED_CURRENCY_ID, DEFAULT_BONDED_DENOMINATION),
			(second_currency_id, 10),
		])
		.with_pools(vec![(pool_id.clone(), pool_details)])
		.with_bonded_balance(vec![
			(DEFAULT_COLLATERAL_CURRENCY_ID, ACCOUNT_00, collateral_balance_supply),
			(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_00, one_bonded_currency),
		])
		.build()
		.execute_with(|| {
			let funds_bonded_coin_before_tx = Assets::balance(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_00);
			assert_eq!(funds_bonded_coin_before_tx, one_bonded_currency);

			let collateral_asset_pool_before_tx = Assets::balance(DEFAULT_COLLATERAL_CURRENCY_ID, pool_id.clone());

			assert_ok!(BondingPallet::swap_into(
				RuntimeOrigin::signed(ACCOUNT_00),
				pool_id.clone(),
				0,
				1,
				one_bonded_currency,
				ACCOUNT_00,
			));

			// Collateral should have not change.
			let collateral_asset_pool_after_tx = Assets::balance(DEFAULT_COLLATERAL_CURRENCY_ID, pool_id.clone());
			assert_eq!(collateral_asset_pool_after_tx, collateral_asset_pool_before_tx);

			// Bonded should have not change.
			let funds_bonded_coin_after_tx = Assets::balance(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_00);
			assert_eq!(funds_bonded_coin_after_tx, 0);

			let funds_target_bonded_coin_after_tx = Assets::balance(second_currency_id, ACCOUNT_00);
			assert_eq!(funds_target_bonded_coin_after_tx, one_bonded_currency);
		});
}
