use frame_support::assert_ok;
use sp_runtime::{traits::Zero, FixedPointNumber, FixedU128};

use crate::{
	curves_parameters::convert_currency_amount,
	mock::{runtime::*, *},
	types::{DiffKind, PoolStatus},
};

#[test]
fn test_burn_into_account() {
	let currencies = vec![DEFAULT_BONDED_CURRENCY_ID];
	let pool_id = calculate_pool_id(currencies.clone());

	let curve = get_linear_bonding_curve();

	let pool = calculate_pool_details(currencies, ACCOUNT_01, false, curve.clone(), PoolStatus::Active);

	let active_issuance_pre = FixedU128::from_u32(1);
	let passive_issuance = FixedU128::from_inner(0);
	let active_issuance_post = FixedU128::from_u32(0);

	let expected_costs_normalized = curve
		.calculate_cost(
			active_issuance_pre,
			active_issuance_post,
			passive_issuance,
			DiffKind::Burn,
		)
		.expect("Cost calculation should not fail");

	let expected_raw_return = convert_currency_amount::<Test>(
		expected_costs_normalized.into_inner(),
		FixedU128::DIV,
		10u128.pow(DEFAULT_BONDED_DENOMINATION.into()),
	)
	.expect("Transforming costs should not fail")
	.into_inner();

	let collateral_balance_supply = DEFAULT_COLLATERAL_UNIT * 10;

	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, UNIT_NATIVE * 10), (pool_id.clone(), UNIT_NATIVE)])
		.with_collateral_asset_id(DEFAULT_COLLATERAL_CURRENCY_ID)
		.with_currencies(vec![vec![DEFAULT_BONDED_CURRENCY_ID]])
		.with_metadata(vec![
			(DEFAULT_COLLATERAL_CURRENCY_ID, DEFAULT_COLLATERAL_DENOMINATION),
			(DEFAULT_BONDED_CURRENCY_ID, DEFAULT_BONDED_DENOMINATION),
		])
		.with_pools(vec![(pool_id.clone(), pool)])
		.with_bonded_balance(vec![
			(
				DEFAULT_COLLATERAL_CURRENCY_ID,
				pool_id.clone(),
				collateral_balance_supply,
			),
			(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_00, DEFAULT_BONDED_UNIT),
		])
		.build()
		.execute_with(|| {
			assert_ok!(BondingPallet::burn_into(
				RuntimeOrigin::signed(ACCOUNT_00),
				pool_id.clone(),
				0,
				DEFAULT_BONDED_UNIT,
				0,
				ACCOUNT_00
			));

			// User should have no bonded coins
			let supply_bonded_coins_user = Assets::balance(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_00);
			assert_eq!(supply_bonded_coins_user, Zero::zero());

			// user should have some collateral
			let collateral_balance_submitter = Assets::balance(DEFAULT_COLLATERAL_CURRENCY_ID, ACCOUNT_00);
			assert_eq!(collateral_balance_submitter, expected_raw_return);

			let collateral_balance_pool = Assets::balance(DEFAULT_COLLATERAL_CURRENCY_ID, pool_id);
			assert_eq!(collateral_balance_pool, collateral_balance_supply - expected_raw_return);

			// The total supply should be zero
			assert_eq!(Assets::total_supply(DEFAULT_BONDED_CURRENCY_ID), Zero::zero());
		});
}
