use crate::{
	curves_parameters::PolynomialFunctionParameters,
	mock::Float,
	types::{Curve, DiffKind},
};

#[test]
fn test_mint_first_coin() {
	// Create curve with shape f(x) = 2x + 3, resulting into integral function F(x) = x^2 + 3x
	let m = Float::from_num(0);
	let n = Float::from_num(1);
	let o = Float::from_num(3);
	let curve = Curve::PolynomialFunction(PolynomialFunctionParameters { m, n, o });

	let active_issuance_pre = Float::from_num(0);
	let active_issuance_post = Float::from_num(1);

	// single coin in pool. Passive issuance is zero.
	let passive_issuance = Float::from_num(0);

	// Existing supply: 0^2 + 3*0 = 0
	// New Supply: 1^2 + 3*1 = 4
	// Cost to mint the first coin: 4 - 0 = 4
	let row_costs = curve
		.calculate_cost(
			active_issuance_pre,
			active_issuance_post,
			passive_issuance,
			DiffKind::Mint,
		)
		.unwrap();

	assert_eq!(row_costs, 4);
}

#[test]
fn test_high_supply() {
	// Create curve with shape f(x) = 2x + 3, resulting into integral function F(x) = x^2 + 3x
	let m = Float::from_num(0);
	let n = Float::from_num(1);
	let o = Float::from_num(3);
	let curve = Curve::PolynomialFunction(PolynomialFunctionParameters { m, n, o });

	// Create supply. Active issuance is 10_000_000. We mint 100_000 additional coins
	let active_issuance_pre: u128 = 10_000_000;
	let active_issuance_post: u128 = 10_100_000;

	// single coin in pool. Passive issuance is zero.
	let passive_issuance = 0;

	// Existing supply: 10_000_000^2 + 3*10_000_000 = 100000030000000
	// New Supply: 10_100_000^2 + 3*10_100_000 = 102010030300000
	// Cost to mint the first coin: 102010030300000 - 100000030000000 = 2010000300000
	let row_costs = curve
		.calculate_cost(
			Float::from_num(active_issuance_pre),
			Float::from_num(active_issuance_post),
			Float::from_num(passive_issuance),
			DiffKind::Mint,
		)
		.unwrap();

	assert_eq!(row_costs, 2010000300000u128);
}

#[test]
fn test_mint_coin_with_existing_supply() {
	// Create curve with shape f(x) = 2x + 3, resulting into integral function F(x) = x^2 + 3x
	let m = Float::from_num(0);
	let n = Float::from_num(1);
	let o = Float::from_num(3);
	let curve = Curve::PolynomialFunction(PolynomialFunctionParameters { m, n, o });

	// Two coins in Pool. Existing supply is 100. We mint 10 additional coins.
	let passive_issuance = Float::from_num(0);
	let active_issuance_pre = Float::from_num(1000);
	let active_issuance_post = Float::from_num(1010);

	// Existing supply: 1000^2 + 3*1000 = 1003000
	// New supply: 1010^2 + 3*1010 = 1023130
	// Cost to mint 10 coins: 1023130 - 10300 = 20130
	let costs = curve
		.calculate_cost(
			active_issuance_pre,
			active_issuance_post,
			passive_issuance,
			DiffKind::Mint,
		)
		.unwrap();

	assert_eq!(costs, Float::from_num(20130));
}

#[test]
fn test_mint_coin_with_existing_passive_supply() {
	// Create curve with shape f(x) = 2x + 3, resulting into integral function F(x) = x^2 + 3x
	let m = Float::from_num(0);
	let n = Float::from_num(1);
	let o = Float::from_num(3);
	let curve = Curve::PolynomialFunction(PolynomialFunctionParameters { m, n, o });

	// Two coins in Pool. Existing supply is 100. We mint 10 additional coins.
	let passive_issuance = Float::from_num(1000);
	let active_issuance_pre = Float::from_num(0);
	let active_issuance_post = Float::from_num(10);

	// Existing supply: 1000^2 + 3*1000 = 1003000
	// New supply: 1010^2 + 3*1010 = 1023130
	// Cost to mint 10 coins: 1023130 - 10300 = 20130
	let costs = curve
		.calculate_cost(
			active_issuance_pre,
			active_issuance_post,
			passive_issuance,
			DiffKind::Mint,
		)
		.unwrap();

	assert_eq!(costs, Float::from_num(20130));
}

#[test]
fn test_mint_coin_with_existing_passive_supply_and_existing_active_supply() {
	// Create curve with shape f(x) = 2x + 3, resulting into integral function F(x) = x^2 + 3x
	let m = Float::from_num(0);
	let n = Float::from_num(1);
	let o = Float::from_num(3);
	let curve = Curve::PolynomialFunction(PolynomialFunctionParameters { m, n, o });

	// Two coins in Pool. Existing supply is 100. We mint 10 additional coins.
	let passive_issuance = Float::from_num(1000);
	let active_issuance_pre = Float::from_num(1000);
	let active_issuance_post = Float::from_num(1010);

	// Existing supply: 2000^2 + 3*2000 = 4006000
	// New supply: 2010^2 + 3*2010 = 4046130
	// Cost to mint 10 coins: 4046130 - 4006000 = 40130
	let costs = curve
		.calculate_cost(
			active_issuance_pre,
			active_issuance_post,
			passive_issuance,
			DiffKind::Mint,
		)
		.unwrap();

	assert_eq!(costs, Float::from_num(40130));
}

#[test]
fn test_mint_first_coin_frac_bonding_curve() {
	// Create curve with shape f(x) = x + 3, resulting into integral function F(x) = 1/2*x^2 + 3x
	let m = Float::from_num(0);
	let n = Float::from_num(0.5);
	let o = Float::from_num(3);
	let curve = Curve::PolynomialFunction(PolynomialFunctionParameters { m, n, o });

	// Create supply, where denomination is 15. Active issuance is zero.
	let active_issuance_pre = Float::from_num(0);
	let active_issuance_post = Float::from_num(1);

	// single coin in pool. Passive issuance is zero.
	let passive_issuance = Float::from_num(0);

	// Existing supply: 1/2*(0)^2 + (0)*3 = 0
	// New supply: 1/2*(1)^2 + (1)*3 = 3.5
	// Cost to mint 10 coin: 3.5 - 0 = 0
	let costs = curve
		.calculate_cost(
			active_issuance_pre,
			active_issuance_post,
			passive_issuance,
			DiffKind::Mint,
		)
		.unwrap();

	assert_eq!(costs, Float::from_num(3.5));
}

// // TODO: tests for burning
