use sp_arithmetic::FixedU128;
use sp_runtime::{traits::Zero, FixedPointNumber};

use crate::{
	curves_parameters::{convert_currency_amount, SquareRootBondingFunctionParameters},
	mock::runtime::*,
	tests::types::curves::{CURRENT_DENOMINATION, NORMALIZED_DENOMINATION},
	types::{Curve, DiffKind},
};

#[test]
fn test_mint_first_coin() {
	// Create curve with shape f(x) = 2x^1/2 + 2, resulting into integral function F(x) = x^3/2 + 2x
	let m = FixedU128::from_u32(1);
	let n = FixedU128::from_u32(2);
	let curve = Curve::SquareRootBondingFunction(SquareRootBondingFunctionParameters { m, n });

	// Create supply, where denomination is 15. Active issuance is zero.
	let active_issuance_pre: u128 = 0;
	let active_issuance_post: u128 = CURRENT_DENOMINATION;

	// single coin in pool. Passive issuance is zero.
	let passive_issuance = FixedU128::zero();

	let normalized_active_issuance_pre =
		convert_currency_amount::<Test>(active_issuance_pre, CURRENT_DENOMINATION, NORMALIZED_DENOMINATION).unwrap();
	let normalized_active_issuance_post =
		convert_currency_amount::<Test>(active_issuance_post, CURRENT_DENOMINATION, NORMALIZED_DENOMINATION).unwrap();

	// Existing supply: 0^3/2 + 3*0 = 0
	// New Supply: 1^3/2 + 2*1 = 3
	// Cost to mint the first coin: 3 - 0 = 3
	let costs = curve
		.calculate_cost(
			normalized_active_issuance_pre,
			normalized_active_issuance_post,
			passive_issuance,
			DiffKind::Mint,
		)
		.unwrap();

	assert_eq!(costs, FixedU128::from_u32(3));
}

#[test]
fn test_mint_coin_with_existing_supply() {
	// Create curve with shape f(x) = 2x^1/2 + 2, resulting into integral function F(x) = x^3/2 + 2x
	let m = FixedU128::from_u32(1);
	let n = FixedU128::from_u32(2);
	let curve = Curve::SquareRootBondingFunction(SquareRootBondingFunctionParameters { m, n });

	// Create supply. Active issuance is 100. We want to mint 10 additional coins.
	let active_issuance_pre: u128 = CURRENT_DENOMINATION * 100;
	let active_issuance_post: u128 = CURRENT_DENOMINATION * 110;

	// single coin in pool. Passive issuance is zero.
	let passive_issuance = FixedU128::zero();

	let normalized_active_issuance_pre =
		convert_currency_amount::<Test>(active_issuance_pre, CURRENT_DENOMINATION, NORMALIZED_DENOMINATION).unwrap();
	let normalized_active_issuance_post =
		convert_currency_amount::<Test>(active_issuance_post, CURRENT_DENOMINATION, NORMALIZED_DENOMINATION).unwrap();

	// Existing supply: 100^3/2 + 2*100 = 1200
	// New supply: 110^2 + 3*110 = 1373.689732987
	// Cost to mint 10 coins: 1373.689732987 - 1200 = 173.689732987
	let costs = curve
		.calculate_cost(
			normalized_active_issuance_pre,
			normalized_active_issuance_post,
			passive_issuance,
			DiffKind::Mint,
		)
		.unwrap();

	assert_eq!(costs, FixedU128::from_rational(173689732987, 1_000_000_000));
}

#[test]
fn test_mint_coin_with_existing_passive_supply() {
	// Create curve with shape f(x) = 2x^1/2 + 2, resulting into integral function F(x) = x^3/2 + 2x
	let m = FixedU128::from_u32(1);
	let n = FixedU128::from_u32(2);
	let curve = Curve::SquareRootBondingFunction(SquareRootBondingFunctionParameters { m, n });

	// Create supply. Active issuance is Zero. We only mint a single coin.
	let active_issuance_pre: u128 = 0;
	let active_issuance_post: u128 = CURRENT_DENOMINATION;

	// Multiple coins in pool. Passive issuance is 10.
	let passive_issuance = CURRENT_DENOMINATION * 10;

	let normalized_active_issuance_pre =
		convert_currency_amount::<Test>(active_issuance_pre, CURRENT_DENOMINATION, NORMALIZED_DENOMINATION).unwrap();
	let normalized_active_issuance_post =
		convert_currency_amount::<Test>(active_issuance_post, CURRENT_DENOMINATION, NORMALIZED_DENOMINATION).unwrap();
	let normalized_passive_issuance =
		convert_currency_amount::<Test>(passive_issuance, CURRENT_DENOMINATION, NORMALIZED_DENOMINATION).unwrap();

	// The passive issuance should influence the price of the new selected currency.
	// Existing supply: (10)^3/2 + (10)*2 = 51.6227766016837933199
	// New supply: (10 + 1)^3/2 + (10 + 1)*2 = 58.4828726939093983402642601
	// Cost to mint 1 coin: 58.4828726939093983402642601 - 51.6227766016837933199 = 6.860096092
	let costs = curve
		.calculate_cost(
			normalized_active_issuance_pre,
			normalized_active_issuance_post,
			normalized_passive_issuance,
			DiffKind::Mint,
		)
		.unwrap();

	assert_eq!(costs, FixedU128::from_rational(6860096092, 1_000_000_000));
}

#[test]
fn test_mint_coin_with_existing_passive_supply_and_existing_active_supply() {
	// Create curve with shape f(x) = 2x^1/2 + 2, resulting into integral function F(x) = x^3/2 + 2x
	let m = FixedU128::from_u32(1);
	let n = FixedU128::from_u32(2);
	let curve = Curve::SquareRootBondingFunction(SquareRootBondingFunctionParameters { m, n });

	// Create supply. Active issuance is 10. We mint 10 additional coins.
	let active_issuance_pre: u128 = CURRENT_DENOMINATION * 10;
	let active_issuance_post: u128 = CURRENT_DENOMINATION * 20;

	// Multiple coins in pool. Passive issuance is 10.
	let passive_issuance = CURRENT_DENOMINATION * 10;

	let normalized_active_issuance_pre =
		convert_currency_amount::<Test>(active_issuance_pre, CURRENT_DENOMINATION, NORMALIZED_DENOMINATION).unwrap();
	let normalized_active_issuance_post =
		convert_currency_amount::<Test>(active_issuance_post, CURRENT_DENOMINATION, NORMALIZED_DENOMINATION).unwrap();
	let normalized_passive_issuance =
		convert_currency_amount::<Test>(passive_issuance, CURRENT_DENOMINATION, NORMALIZED_DENOMINATION).unwrap();

	// The passive issuance should influence the price of the new selected currency.
	// Existing supply: (20)^(3/2) + (20)*2 = 129.442719099991
	// New supply: (30)^(3/2) + (30)*2 = 224.3167672515498
	// Cost to mint 10 coin: 224.3167672515498 - 129.442719099991 = 94.874048152
	let costs = curve
		.calculate_cost(
			normalized_active_issuance_pre,
			normalized_active_issuance_post,
			normalized_passive_issuance,
			DiffKind::Mint,
		)
		.unwrap();

	assert_eq!(costs, FixedU128::from_rational(94874048152, 1_000_000_000));
}

#[test]
fn test_mint_first_coin_frac_bonding_curve() {
	// Create curve with shape f(x) = x^1/2 + 2, resulting into integral function F(x) = 2/3 x^3/2 + 2x
	let m = FixedU128::from_rational(2, 3);
	let n = FixedU128::from_u32(2);
	let curve = Curve::SquareRootBondingFunction(SquareRootBondingFunctionParameters { m, n });

	// Create supply, where denomination is 15. Active issuance is zero.
	let active_issuance_pre: u128 = 0;
	let active_issuance_post: u128 = CURRENT_DENOMINATION;

	// single coin in pool. Passive issuance is zero.
	let passive_issuance = FixedU128::zero();

	let normalized_active_issuance_pre =
		convert_currency_amount::<Test>(active_issuance_pre, CURRENT_DENOMINATION, NORMALIZED_DENOMINATION).unwrap();
	let normalized_active_issuance_post =
		convert_currency_amount::<Test>(active_issuance_post, CURRENT_DENOMINATION, NORMALIZED_DENOMINATION).unwrap();

	// Existing supply: 2/3*(0)^(3/2) + 2/3*(0)*2 = 0
	// New supply: 2/3*(1)^(3/2) + (1)*2 = 2.666666666..
	// Cost to mint 10 coin: 2 - 0 = 0
	let costs = curve
		.calculate_cost(
			normalized_active_issuance_pre,
			normalized_active_issuance_post,
			passive_issuance,
			DiffKind::Mint,
		)
		.unwrap();

	assert_eq!(costs, FixedU128::from_rational(2666666666666666667, FixedU128::DIV));
}

// TODO: more tests for burning and frac.
