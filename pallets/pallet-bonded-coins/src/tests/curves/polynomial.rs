use crate::{
	curves::{BondingFunction, PolynomialParameters},
	mock::Float,
};

// linear function
#[test]
fn mint_first_coin_linear_function() {
	// Create curve with shape f(x) = 2x + 3, resulting into integral function F(x) = x^2 + 3x
	let m = Float::from_num(0);
	let n = Float::from_num(1);
	let o = Float::from_num(3);
	let curve = PolynomialParameters { m, n, o };

	let low = Float::from_num(0);
	let high = Float::from_num(1);

	// Existing supply: 0^2 + 3*0 = 0
	// New Supply: 1^2 + 3*1 = 4
	// Cost to mint the first coin: 4 - 0 = 4
	let costs = curve.calculate_costs(low, high).unwrap();

	assert_eq!(costs, 4);
}

#[test]
fn high_supply_linear_function() {
	// Create curve with shape f(x) = 2x + 3, resulting into integral function F(x) = x^2 + 3x
	let m = Float::from_num(0);
	let n = Float::from_num(1);
	let o = Float::from_num(3);
	let curve = PolynomialParameters { m, n, o };

	let low = Float::from_num(10_000_000_000u128);
	let high = Float::from_num(10_000_100_000u128);

	// Existing supply: 10_000_000^2 + 3*10_000_000 = 100000030000000
	// New Supply: 10_100_000^2 + 3*10_100_000 = 102010030300000
	// Cost to mint the first coin: 102010030300000 - 100000030000000 = 2010000300000
	let costs = curve.calculate_costs(low, high).unwrap();

	assert_eq!(costs, 2000010000300000u128);
}

#[test]
fn mint_coin_with_existing_supply_linear_function() {
	// Create curve with shape f(x) = 2x + 3, resulting into integral function F(x) = x^2 + 3x
	let m = Float::from_num(0);
	let n = Float::from_num(1);
	let o = Float::from_num(3);
	let curve = PolynomialParameters { m, n, o };

	let low = Float::from_num(1000);
	let high = Float::from_num(1010);

	// Existing supply: 1000^2 + 3*1000 = 1003000
	// New supply: 1010^2 + 3*1010 = 1023130
	// Cost to mint 10 coins: 1023130 - 10300 = 20130
	let costs = curve.calculate_costs(low, high).unwrap();

	assert_eq!(costs, 20130);
}

#[test]
fn mint_first_coin_frac_bonding_linear_function() {
	// Create curve with shape f(x) = x + 3, resulting into integral function F(x) = 1/2*x^2 + 3x
	let m = Float::from_num(0);
	let n = Float::from_num(0.5);
	let o = Float::from_num(3);
	let curve = PolynomialParameters { m, n, o };

	let low = Float::from_num(0);
	let high = Float::from_num(1);

	// Existing supply: 1/2*(0)^2 + (0)*3 = 0
	// New supply: 1/2*(1)^2 + (1)*3 = 3.5
	// Cost to mint 10 coin: 3.5 - 0 = 0
	let costs = curve.calculate_costs(low, high).unwrap();

	assert_eq!(costs, 3.5);
}

// quadratic function

#[test]
fn mint_first_coin_quadratic_function() {
	// Create curve with shape f(x) = 3*x^2 + 2x + 3, resulting into integral function F(x) = x^3 +  x^2 + 3x
	let m = Float::from_num(1);
	let n = Float::from_num(1);
	let o = Float::from_num(3);
	let curve = PolynomialParameters { m, n, o };

	let low = Float::from_num(0);
	let high = Float::from_num(1);

	// Existing supply: 1*0^3 + 0^2 + 3*0 = 0
	// New Supply: 1^3 1^2 + 3*1 = 5
	// Cost to mint the first coin: 5 - 0 = 5
	let costs = curve.calculate_costs(low, high).unwrap();

	assert_eq!(costs, 5);
}

#[test]
fn high_supply_quadratic_function() {
	// Create curve with shape f(x) = 3x² + 2x + 3, resulting into integral function F(x) = x³ + x^2 + 3x
	let m = Float::from_num(1);
	let n = Float::from_num(1);
	let o = Float::from_num(3);
	let curve = PolynomialParameters { m, n, o };

	let low = Float::from_num(100_000_000);
	let high = Float::from_num(100_100_000);

	// Existing supply: (1) * 100000000^3 + (1)*(100000000)^2 + (100000000)*3 = 1000000010000000300000000
	// New supply: (1)*(100100000)^3 + (1)*(100100000)^2 + (100100000)*3 = 1003003011020010300300000
	// Cost to mint 10 coin: 1003003011020010300300000 - 1000000010000000300000000 = 3003001020010000300000
	let costs = curve.calculate_costs(low, high).unwrap();
	assert_eq!(costs, 3003001020010000300000u128);
}

#[test]
fn mint_coin_with_existing_supply_quadratic_function() {
	// Create curve with shape f(x) = 3x² + 2x + 3, resulting into integral function F(x) = x³ + x^2 + 3x
	let m = Float::from_num(1);
	let n = Float::from_num(1);
	let o = Float::from_num(3);
	let curve = PolynomialParameters { m, n, o };

	let low = Float::from_num(1000);
	let high = Float::from_num(1010);

	// Existing supply: 1000^3 + 1000^2 + 3*1000 = 1001003000
	// New supply: 1010^3 + 1010^2 + 3*1010 = 1031324130
	// Cost to mint 10 coins: 1031324130 - 1001003000 = 30321130
	let costs = curve.calculate_costs(low, high).unwrap();

	assert_eq!(costs, Float::from_num(30321130));
}

#[test]
fn mint_first_coin_frac_bonding_quadratic_function() {
	// Create curve with shape f(x) = x + 3, resulting into integral function F(x) = 1/2*x^2 + 3x
	let m = Float::from_num(0.5);
	let n = Float::from_num(0.5);
	let o = Float::from_num(3);
	let curve = PolynomialParameters { m, n, o };

	let low = Float::from_num(0);
	let high = Float::from_num(1);

	// Existing supply: 1/2 *0^3 1/2*(0)^2 + (0)*3 = 0
	// New supply: 1/2*(1)^3 1/2*(1)^2 + (1)*3 = 4
	// Cost to mint 10 coin: 3.5 - 0 = 0
	let costs = curve.calculate_costs(low, high).unwrap();

	assert_eq!(costs, 4);
}

#[test]
fn zero_coefficients() {
	let m = Float::from_num(0);
	let n = Float::from_num(0);
	let o = Float::from_num(0);
	let curve = PolynomialParameters { m, n, o };

	let low = Float::from_num(0);
	let high = Float::from_num(1);

	// Existing supply: 1/2*(0)^2 + (0)*3 = 0
	// New supply: 0*(1)^2 + 0*3 = 0
	// Cost to mint 10 coin: 3.5 - 0 = 0
	let costs = curve.calculate_costs(low, high).unwrap();

	assert_eq!(costs, 0);
}
