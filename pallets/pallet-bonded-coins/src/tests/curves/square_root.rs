use std::str::FromStr;

use crate::{
	curves::{Curve, DiffKind, SquareRootBondingFunctionParameters},
	mock::{assert_relative_eq, Float},
};

#[test]
fn test_mint_first_coin() {
	// Create curve with shape f(x) = 2x^1/2 + 2, resulting into integral function F(x) = x^3/2 + 2x
	let m = Float::from_num(1);
	let n = Float::from_num(2);
	let curve = Curve::SquareRootBondingFunction(SquareRootBondingFunctionParameters { m, n });

	// Create supply, where denomination is 15. Active issuance is zero.
	let active_issuance_pre = Float::from_num(0);
	let active_issuance_post = Float::from_num(1);

	// single coin in pool. Passive issuance is zero.
	let passive_issuance = Float::from_num(0);

	// Existing supply: 0^3/2 + 3*0 = 0
	// New Supply: 1^3/2 + 2*1 = 3
	// Cost to mint the first coin: 3 - 0 = 3
	let costs = curve
		.calculate_cost(
			active_issuance_pre,
			active_issuance_post,
			passive_issuance,
			DiffKind::Mint,
		)
		.unwrap();

	assert_eq!(costs, Float::from_num(3));
}

#[test]
fn test_mint_coin_with_existing_supply() {
	// Create curve with shape f(x) = 2x^1/2 + 2, resulting into integral function F(x) = x^3/2 + 2x
	let m = Float::from_num(1);
	let n = Float::from_num(2);
	let curve = Curve::SquareRootBondingFunction(SquareRootBondingFunctionParameters { m, n });

	// Create supply. Active issuance is 100. We want to mint 10 additional coins.

	// single coin in pool. Passive issuance is zero.
	let passive_issuance = Float::from_num(0);

	let active_issuance_pre = Float::from_num(100);
	let active_issuance_post = Float::from_num(110);

	// Existing supply: 100^3/2 + 2*100 = 1200
	// New supply: 110^3/2 + 2*110 = 1373.6897329871667016905988650
	// Cost to mint 10 coins: 1373.6897329871667016905988650 - 1200 = 173.689732987166701690598865 -> 173.6897329871667016

	let costs = curve
		.calculate_cost(
			active_issuance_pre,
			active_issuance_post,
			passive_issuance,
			DiffKind::Mint,
		)
		.unwrap();

	let expected_costs = Float::from_str("173.6897329871667016").unwrap();

	assert_relative_eq(costs, expected_costs, Float::from_str("0.0000000000000100").unwrap());
}

#[test]
fn test_mint_coin_with_existing_passive_supply() {
	// Create curve with shape f(x) = 2x^1/2 + 2, resulting into integral function F(x) = x^3/2 + 2x
	let m = Float::from_num(1);
	let n = Float::from_num(2);
	let curve = Curve::SquareRootBondingFunction(SquareRootBondingFunctionParameters { m, n });

	let passive_issuance = Float::from_num(10);
	let active_issuance_pre = Float::from_num(0);
	let active_issuance_post = Float::from_num(1);

	// The passive issuance should influence the price of the new selected currency.
	// Existing supply: (10)^3/2 + (10)*2 = 51.6227766016837933199
	// New supply: (10 + 1)^3/2 + (10 + 1)*2 = 58.4828726939093983402642601
	// Cost to mint 1 coin: 58.4828726939093983402642601 - 51.6227766016837933199 = 6.8600960922256050203642601 -> 6.8600960922256050

	let costs = curve
		.calculate_cost(
			active_issuance_pre,
			active_issuance_post,
			passive_issuance,
			DiffKind::Mint,
		)
		.unwrap();

	let expected_costs = Float::from_str("6.8600960922256050").unwrap();
	assert_relative_eq(costs, expected_costs, Float::from_str("0.0000000000000001").unwrap());
}

#[test]
fn test_mint_coin_with_existing_passive_supply_and_existing_active_supply() {
	// Create curve with shape f(x) = 2x^1/2 + 2, resulting into integral function F(x) = x^3/2 + 2x
	let m = Float::from_num(1);
	let n = Float::from_num(2);
	let curve = Curve::SquareRootBondingFunction(SquareRootBondingFunctionParameters { m, n });

	// Create supply. Active issuance is 10. We mint 10 additional coins.

	let active_issuance_pre = Float::from_num(10);
	let active_issuance_post = Float::from_num(20);
	let passive_issuance = Float::from_num(10);

	// The passive issuance should influence the price of the new selected currency.
	// Existing supply: (20)^(3/2) + (20)*2 = 129.4427190999915878563669467492
	// New supply: (30)^(3/2) + (30)*2 = 224.316767251549834037090934840240640
	// Cost to mint 10 coin: 224.3167672515498 - 129.442719099991 = 94.87404815155824618072398809104064 -> 94.8740481515582461
	let expected_costs = curve
		.calculate_cost(
			active_issuance_pre,
			active_issuance_post,
			passive_issuance,
			DiffKind::Mint,
		)
		.unwrap();

	let costs = Float::from_str("94.8740481515582461").unwrap();

	assert_relative_eq(costs, expected_costs, Float::from_str("0.0000000000000020").unwrap());
}

#[test]
fn test_mint_first_coin_frac_bonding_curve() {
	// Create curve with shape f(x) = x^1/2 + 2, resulting into integral function F(x) = 2/3 x^3/2 + 2x
	let m = Float::from_num(0.6666);
	let n = Float::from_num(2);
	let curve = Curve::SquareRootBondingFunction(SquareRootBondingFunctionParameters { m, n });

	// single coin in pool. Passive issuance is zero.
	let active_issuance_pre = Float::from_num(0);
	let passive_issuance = Float::from_num(0);
	let active_issuance_post = Float::from_num(1);

	// Existing supply: 2/3*(0)^(3/2) + 2/3*(0)*2 = 0
	// New supply: 0.6666*(1)^(3/2) + (1)*2 = 2.6666
	// Cost to mint 10 coin: 2 - 0 = 0
	let costs = curve
		.calculate_cost(
			active_issuance_pre,
			active_issuance_post,
			passive_issuance,
			DiffKind::Mint,
		)
		.unwrap();

	let expected_costs = Float::from_str("2.6666").unwrap();

	assert_eq!(costs, expected_costs);
}

// TODO: more tests for burning and frac.
