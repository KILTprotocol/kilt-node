use frame_support::assert_ok;

use crate::{
	mock::{runtime::*, *},
	types::{DiffKind, PoolStatus},
};

#[ignore]
#[test]
fn test_mint_into_account() {
	let currencies = vec![DEFAULT_BONDED_CURRENCY_ID];
	let pool_id = calculate_pool_id(currencies.clone());

	let curve = get_linear_bonding_curve();

	let denomination = 10;

	let pool = calculate_pool_details(
		currencies,
		ACCOUNT_01,
		false,
		curve.clone(),
		PoolStatus::Active,
		denomination,
	);

	let active_issuance_pre = Float::from_num(0);
	let passive_issuance = Float::from_num(0);
	let active_issuance_post = Float::from_num(1);

	let expected_costs_normalized = curve
		.calculate_cost(
			active_issuance_pre,
			active_issuance_post,
			passive_issuance,
			DiffKind::Mint,
		)
		.expect("Cost calculation should not fail");

	let expected_raw_costs = expected_costs_normalized * Float::from_num(denomination);

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
		.with_bonded_balance(vec![(
			DEFAULT_COLLATERAL_CURRENCY_ID,
			ACCOUNT_00,
			collateral_balance_supply,
		)])
		.build()
		.execute_with(|| {
			assert_ok!(BondingPallet::mint_into(
				RuntimeOrigin::signed(ACCOUNT_00),
				pool_id.clone(),
				0,
				DEFAULT_BONDED_UNIT,
				DEFAULT_COLLATERAL_UNIT * 5,
				ACCOUNT_00
			));

			// user should have the requested bonded coin
			let supply_minted_coins = Assets::balance(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_00);
			assert_eq!(supply_minted_coins, DEFAULT_BONDED_UNIT);

			// pool should have the required collateral for minting the coin
			let collateral_balance = Assets::balance(DEFAULT_COLLATERAL_CURRENCY_ID, pool_id);
			assert_eq!(collateral_balance, expected_raw_costs);

			// the total supply should be one
			let bonded_currency_total_supply = Assets::total_supply(DEFAULT_BONDED_CURRENCY_ID);
			assert_eq!(bonded_currency_total_supply, DEFAULT_BONDED_UNIT);

			// the submitter should have the collateral balance reduced by the minting cost
			let collateral_balance_submitter = Assets::balance(DEFAULT_COLLATERAL_CURRENCY_ID, ACCOUNT_00);
			assert_eq!(
				collateral_balance_submitter,
				collateral_balance_supply - expected_raw_costs.to_num::<u128>()
			);
		});
}
