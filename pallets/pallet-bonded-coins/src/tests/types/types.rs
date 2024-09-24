use sp_arithmetic::FixedU128;
use sp_runtime::{traits::Zero, FixedPointNumber};

use crate::{
	curves_parameters::{transform_denomination_currency_amount, LinearBondingFunctionParameters},
	mock::runtime::*,
	types::{Curve, DiffKind},
};
// target denomination for collateral currency
const CURRENT_DENOMINATION: u128 = 10u128.pow(15);
const NORMALIZED_DENOMINATION: u128 = FixedU128::DIV;
const ONE_COIN: u128 = CURRENT_DENOMINATION;

#[test]
fn test_mint_first_coin() {
	// Create curve with shape f(x) = 2x + 3, resulting into integral function F(x) = x^2 + 3x
	let m = FixedU128::from_u32(1);
	let n = FixedU128::from_u32(3);
	let curve = Curve::LinearRatioCurve(LinearBondingFunctionParameters { m, n });

	// Create supply, where denomination is 15. Active issuance is zero.
	let active_issuance_pre: u128 = 0;
	let active_issuance_post: u128 = ONE_COIN;

	// single coin in pool. Passive issuance is zero.
	let passive_issuance = FixedU128::zero();

	let normalized_active_issuance_pre = transform_denomination_currency_amount::<Test>(
		active_issuance_pre,
		CURRENT_DENOMINATION,
		NORMALIZED_DENOMINATION,
	)
	.unwrap();
	let normalized_active_issuance_post = transform_denomination_currency_amount::<Test>(
		active_issuance_post,
		CURRENT_DENOMINATION,
		NORMALIZED_DENOMINATION,
	)
	.unwrap();

	// The cost to mint the first coin should be 4.
	let costs = curve
		.calculate_cost(
			normalized_active_issuance_pre,
			normalized_active_issuance_post,
			passive_issuance,
			DiffKind::Mint,
		)
		.unwrap();

	assert_eq!(costs, FixedU128::from_u32(4));
}

#[test]
fn test_mint_coin_with_existing_supply() {
	// Create curve with shape f(x) = 2x + 3, resulting into integral function F(x) = x^2 + 3x
	let m = FixedU128::from_u32(1);
	let n = FixedU128::from_u32(3);
	let curve = Curve::LinearRatioCurve(LinearBondingFunctionParameters { m, n });

	// Create supply. Active issuance is 100. We want to mint 10 additional coins.
	let active_issuance_pre: u128 = ONE_COIN * 100;
	let active_issuance_post: u128 = ONE_COIN * 110;

	// single coin in pool. Passive issuance is zero.
	let passive_issuance = FixedU128::zero();

	let normalized_active_issuance_pre = transform_denomination_currency_amount::<Test>(
		active_issuance_pre,
		CURRENT_DENOMINATION,
		NORMALIZED_DENOMINATION,
	)
	.unwrap();
	let normalized_active_issuance_post = transform_denomination_currency_amount::<Test>(
		active_issuance_post,
		CURRENT_DENOMINATION,
		NORMALIZED_DENOMINATION,
	)
	.unwrap();

	// Existing supply: 100^2 + 3*100 = 10300
	// New supply: 110^2 + 3*110 = 12130
	// Cost to mint 10 coins: 12130 - 10300 = 2130
	let costs = curve
		.calculate_cost(
			normalized_active_issuance_pre,
			normalized_active_issuance_post,
			passive_issuance,
			DiffKind::Mint,
		)
		.unwrap();

	assert_eq!(costs, FixedU128::from_u32(2130));
}

#[test]
fn test_mint_coin_with_existing_passive_supply() {
	// Create curve with shape f(x) = 2x + 3, resulting into integral function F(x) = x^2 + 3x
	let m = FixedU128::from_u32(1);
	let n = FixedU128::from_u32(3);
	let curve = Curve::LinearRatioCurve(LinearBondingFunctionParameters { m, n });

	// Create supply. Active issuance is Zero. We only mint a single coin.
	let active_issuance_pre: u128 = 0;
	let active_issuance_post: u128 = ONE_COIN;

	// Multiple coins in pool. Passive issuance is 10.
	let passive_issuance = ONE_COIN * 10;

	let normalized_active_issuance_pre = transform_denomination_currency_amount::<Test>(
		active_issuance_pre,
		CURRENT_DENOMINATION,
		NORMALIZED_DENOMINATION,
	)
	.unwrap();
	let normalized_active_issuance_post = transform_denomination_currency_amount::<Test>(
		active_issuance_post,
		CURRENT_DENOMINATION,
		NORMALIZED_DENOMINATION,
	)
	.unwrap();
	let normalized_passive_issuance =
		transform_denomination_currency_amount::<Test>(passive_issuance, CURRENT_DENOMINATION, NORMALIZED_DENOMINATION)
			.unwrap();

	// The passive issuance should influence the price of the new selected currency.
	// Existing supply: (10)^2 + (10 )*3 = 130
	// New supply: (10 + 1)^2 + (10 + 1)*3 = 154
	// Cost to mint 1 coin: 154 - 130 = 24
	let costs = curve
		.calculate_cost(
			normalized_active_issuance_pre,
			normalized_active_issuance_post,
			normalized_passive_issuance,
			DiffKind::Mint,
		)
		.unwrap();

	assert_eq!(costs, FixedU128::from_u32(24));
}

#[test]
fn test_mint_coin_with_existing_passive_supply_and_existing_active_supply() {
	// Create curve with shape f(x) = 2x + 3, resulting into integral function F(x) = x^2 + 3x
	let m = FixedU128::from_u32(1);
	let n = FixedU128::from_u32(3);
	let curve = Curve::LinearRatioCurve(LinearBondingFunctionParameters { m, n });

	// Create supply. Active issuance is 10. We mint 10 additional coins.
	let active_issuance_pre: u128 = ONE_COIN * 10;
	let active_issuance_post: u128 = ONE_COIN * 20;

	// Multiple coins in pool. Passive issuance is 10.
	let passive_issuance = ONE_COIN * 10;

	let normalized_active_issuance_pre = transform_denomination_currency_amount::<Test>(
		active_issuance_pre,
		CURRENT_DENOMINATION,
		NORMALIZED_DENOMINATION,
	)
	.unwrap();
	let normalized_active_issuance_post = transform_denomination_currency_amount::<Test>(
		active_issuance_post,
		CURRENT_DENOMINATION,
		NORMALIZED_DENOMINATION,
	)
	.unwrap();
	let normalized_passive_issuance =
		transform_denomination_currency_amount::<Test>(passive_issuance, CURRENT_DENOMINATION, NORMALIZED_DENOMINATION)
			.unwrap();

	// The passive issuance should influence the price of the new selected currency.
	// Existing supply: (20)^2 + (20)*3 = 460
	// New supply: (30)^2 + (30)*3 = 990
	// Cost to mint 10 coin: 990 - 460 = 530
	let costs = curve
		.calculate_cost(
			normalized_active_issuance_pre,
			normalized_active_issuance_post,
			normalized_passive_issuance,
			DiffKind::Mint,
		)
		.unwrap();

	assert_eq!(costs, FixedU128::from_u32(530));
}

#[test]
fn test_mint_first_coin_frac_bonding_curve() {
	// Create curve with shape f(x) = x + 3, resulting into integral function F(x) = 1/2*x^2 + 3x
	let m = FixedU128::from_rational(1, 2);
	let n = FixedU128::from_u32(3);
	let curve = Curve::LinearRatioCurve(LinearBondingFunctionParameters { m, n });

	// Create supply, where denomination is 15. Active issuance is zero.
	let active_issuance_pre: u128 = 0;
	let active_issuance_post: u128 = ONE_COIN;

	// single coin in pool. Passive issuance is zero.
	let passive_issuance = FixedU128::zero();

	let normalized_active_issuance_pre = transform_denomination_currency_amount::<Test>(
		active_issuance_pre,
		CURRENT_DENOMINATION,
		NORMALIZED_DENOMINATION,
	)
	.unwrap();
	let normalized_active_issuance_post = transform_denomination_currency_amount::<Test>(
		active_issuance_post,
		CURRENT_DENOMINATION,
		NORMALIZED_DENOMINATION,
	)
	.unwrap();

	// Existing supply: 1/2*(0)^2 + (0)*3 = 0
	// New supply: 1/2*(1)^2 + (1)*3 = 3.5
	// Cost to mint 10 coin: 3.5 - 0 = 0
	let costs = curve
		.calculate_cost(
			normalized_active_issuance_pre,
			normalized_active_issuance_post,
			passive_issuance,
			DiffKind::Mint,
		)
		.unwrap();

	assert_eq!(costs, FixedU128::from_rational(7, 2));
}

// TODO add more tests for passive and existing active supply.
