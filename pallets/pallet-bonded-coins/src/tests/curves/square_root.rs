// KILT Blockchain – <https://kilt.io>
// Copyright (C) 2025, KILT Foundation

// The KILT Blockchain is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The KILT Blockchain is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

// If you feel like getting in touch with us, you can do so at <hello@kilt.io>
use std::str::FromStr;

use crate::{
	curves::{square_root::SquareRootParameters, BondingFunction},
	mock::runtime::{assert_relative_eq, Float},
};

#[test]
fn mint_first_coin() {
	// Create curve with shape f(x) = 2x^1/2 + 2, resulting in integral function
	// F(x) = x^3/2 + 2x
	let m = Float::from_num(1);
	let n = Float::from_num(2);
	let curve = SquareRootParameters { m, n };

	let low = Float::from_num(0);
	let high = Float::from_num(1);

	// Existing supply: 0^3/2 + 2*0 = 0
	// New Supply: 1^3/2 + 2*1 = 3
	// Cost to mint the first coin: 3 - 0 = 3
	let costs = curve.calculate_costs(low, high, vec![]).unwrap();

	assert_eq!(costs, Float::from_num(3));
}

#[test]
fn high_supply() {
	// Create curve with shape f(x) = 2x^1/2 + 2, resulting in integral function
	// F(x) = x^3/2 + 2x
	let m = Float::from_num(1);
	let n = Float::from_num(2);
	let curve = SquareRootParameters { m, n };

	let low = Float::from_num(100_000_000_000_000u128);
	let high = Float::from_num(100_000_000_100_000u128);

	// Existing supply:
	// 100_000_000_000_000^3/2 + 2*100_000_000_000_000 = 1000000200000000000000

	// New Supply:
	// 100_000_000_100_000^3/2 + 2*100_000_000_100_000 =
	// 1000000201500000200374.
	// 9999999375000000234374999882812500068359374956054687530212

	// Cost to mint the first coin:
	// 1000000201500000200374.
	// 9999999375000000234374999882812500068359374956054687530212 -
	// 1000000200000000000000 =
	// 1500000200374.9999999375000000234374999882812500068359 -> 1500000200374.
	// 9999999375000000
	let costs = curve.calculate_costs(low, high, vec![]).unwrap();

	let expected_costs = Float::from_str("1500000200374.9999999375000000").unwrap();

	assert_relative_eq(costs, expected_costs, Float::from_str("0.02").unwrap());
}

#[test]
fn mint_coin_with_existing_supply() {
	// Create curve with shape f(x) = 2x^1/2 + 2, resulting in integral function
	// F(x) = x^3/2 + 2x
	let m = Float::from_num(1);
	let n = Float::from_num(2);
	let curve = SquareRootParameters { m, n };

	let low = Float::from_num(100);
	let high = Float::from_num(110);

	// Existing supply:
	// 100^3/2 + 2*100 = 1200
	// New supply:
	// 110^3/2 + 2*110 = 1373.6897329871667016905988650
	// Cost to mint 10 coins:
	// 1373.6897329871667016905988650 - 1200 = 173.689732987166701690598865 ->
	// 173.6897329871667016
	let costs = curve.calculate_costs(low, high, vec![]).unwrap();

	let expected_costs = Float::from_str("173.6897329871667016").unwrap();

	assert_relative_eq(costs, expected_costs, Float::from_str("0.0000000000000100").unwrap());
}

#[test]
fn mint_first_coin_frac_bonding_curve() {
	// Create curve with shape f(x) = x^1/2 + 2, resulting in integral function
	// F(x) = 2/3 x^3/2 + 2x
	let m = Float::from_num(0.6666);
	let n = Float::from_num(2);
	let curve = SquareRootParameters { m, n };

	// single coin in pool. Passive issuance is zero.
	let low = Float::from_num(0);
	let high = Float::from_num(1);

	// Existing supply: 0.6666 * (0)^(3/2) + (0)*2 = 0
	// New supply: 0.6666 * (1)^(3/2) + (1) * 2 = 2.6666
	// Cost to mint 10 coin: 2 - 0 = 0
	let costs = curve.calculate_costs(low, high, vec![]).unwrap();

	let expected_costs = Float::from_str("2.6666").unwrap();

	assert_eq!(costs, expected_costs);
}

#[test]
fn zero_coefficients() {
	// Create curve with shape f(x) = x^1/2 + 2, resulting in integral function
	// F(x) = 2/3 x^3/2 + 2x
	let m = Float::from_num(0);
	let n = Float::from_num(0);
	let curve = SquareRootParameters { m, n };

	// single coin in pool. Passive issuance is zero.
	let low = Float::from_num(100);

	let high = Float::from_num(101);

	// Existing supply: 0 * (100)^(3/2) + (100)*0 = 0
	// New supply: 0 * (101)^(3/2) + (101) * 0 = 0
	// Cost to mint 10 coin: 2 - 0 = 0
	let costs = curve.calculate_costs(low, high, vec![]).unwrap();

	assert_eq!(costs, 0);
}
