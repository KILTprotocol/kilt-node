use frame_support::assert_ok;
use sp_runtime::FixedU128;

use crate::{
	curves_parameters::transform_denomination_currency_amount,
	mock::{runtime::*, *},
	types::PoolStatus,
};

#[test]
fn test_mint_into_account() {
	let currencies = vec![DEFAULT_BONDED_CURRENCY_ID];
	let pool_id = calculate_pool_id(currencies.clone());
	let one_bonded_currency = get_currency_unit(DEFAULT_BONDED_DENOMINATION);
	let one_collateral_currency = get_currency_unit(DEFAULT_COLLATERAL_DENOMINATION);

	let curve = get_linear_bonding_curve();

	let pool = calculate_pool_details(currencies, ACCOUNT_01, false, curve.clone(), PoolStatus::Active);

	let active_issuance_pre = FixedU128::from_inner(0);
	let passive_issuance = FixedU128::from_inner(0);
	let active_issuance_post = FixedU128::from_u32(1);

	let expected_costs_normalized = curve
		.calculate_cost(active_issuance_pre, active_issuance_post, passive_issuance)
		.expect("Cost calculation should not fail");

	let expected_raw_costs =
		transform_denomination_currency_amount(expected_costs_normalized.into_inner(), 18, DEFAULT_BONDED_DENOMINATION)
			.expect("Transforming costs should not fail")
			.into_inner();

	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, UNIT_NATIVE * 10)])
		.with_collateral_asset_id(DEFAULT_COLLATERAL_CURRENCY_ID)
		.with_currencies(vec![vec![DEFAULT_BONDED_CURRENCY_ID]])
		.with_metadata(vec![
			(DEFAULT_COLLATERAL_CURRENCY_ID, DEFAULT_COLLATERAL_DENOMINATION),
			(DEFAULT_BONDED_CURRENCY_ID, DEFAULT_BONDED_DENOMINATION),
		])
		.with_pools(vec![(pool_id.clone(), pool)])
		.with_bonded_balance(vec![(
			DEFAULT_COLLATERAL_CURRENCY_ID,
			ACCOUNT_00,
			one_collateral_currency * 10,
		)])
		.build()
		.execute_with(|| {
			assert_ok!(BondingPallet::mint_into(
				RuntimeOrigin::signed(ACCOUNT_00),
				pool_id.clone(),
				0,
				one_bonded_currency,
				one_collateral_currency * 5,
				ACCOUNT_00
			));

			let supply_minted_coins = Assets::balance(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_00);
			assert_eq!(supply_minted_coins, one_bonded_currency);

			let collateral_balance = Assets::balance(DEFAULT_COLLATERAL_CURRENCY_ID, pool_id);
			assert_eq!(collateral_balance, expected_raw_costs);
		});
}
