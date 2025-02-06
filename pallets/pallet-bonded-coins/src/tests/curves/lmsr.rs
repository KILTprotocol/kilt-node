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

// If you feel like getting in touch with us, you can do so at <hello@kilt.org>

use std::str::FromStr;

use crate::{
	curves::{lmsr::LMSRParameters, BondingFunction},
	mock::runtime::{assert_relative_eq, Float},
};

#[test]
fn mint_first_coin() {
	// Create curve with liquidity parameter b=100_000_000, and passive issuance=0
	let m = Float::from_num(100_000_000);

	let curve = LMSRParameters { m };

	let low = Float::from_num(0);
	let high = Float::from_num(1);

	let passive_issuance = Float::from_num(0);

	// Costs for existing supply:
	// 100000000 * ln(e^(0/100000000) + e^(0/100000000)) = 69314718.055994530942

	// Costs for new supply:
	// 100000000 * ln(e^(1/100000000) + e^(0/100000000)) = 69314718.555994532192

	// Costs to mint the first coin:
	// 69314718.555994532192 - 69314718.055994530942 = 0.50000000124972321215 ->
	// 0.50000000124972321215
	let costs = curve.calculate_costs(low, high, vec![passive_issuance]).unwrap();

	let expected_costs = Float::from_str("0.50000000124972321215").unwrap();

	assert_relative_eq(costs, expected_costs, Float::from_str("0.00000002").unwrap());
}

#[test]
fn high_supply_with_no_passive_issuance() {
	// Create curve with liquidity parameter b=100_000_000, and passive issuance=0
	let m = Float::from_num(100_000_000);
	let curve = LMSRParameters { m };

	// we mint 100 coins.
	let low = Float::from_num(100_000_000u128);
	let high = Float::from_num(100_000_100u128);

	let passive_issuance = Float::from_num(0);

	// Costs for existing supply:
	// 100000000 * ln(e^(100000000/100000000) + e^(0/100000000)) =
	// 131326168.7518222834

	// Costs for new supply:
	// 100000000 * ln(e^(100000100/100000000) + e^(0/100000000)) =
	// 131326241.857689977

	// Costs to mint the first coin:
	// 131326241.857689977 - 131326168.7518222834 = 73.1058676936
	let costs = curve.calculate_costs(low, high, vec![passive_issuance]).unwrap();

	let expected_costs = Float::from_str("73.1058676936").unwrap();

	assert_relative_eq(costs, expected_costs, Float::from_str("0.000001").unwrap());
}

#[test]
fn high_supply_with_passive_issuance() {
	// Create curve with liquidity parameter b=100_000_000, and passive
	// issuance=1_000_000_000
	let m = Float::from_num(100_000_000);
	let curve = LMSRParameters { m };

	// we mint 100 coins.
	let low = Float::from_num(100_000_000u128);
	let high = Float::from_num(100_000_100u128);

	let passive_issuance = Float::from_num(1_000_000_000);

	// Costs for existing supply:
	// 100000000 * ln(e^(100000000/100000000) + e^(1000000000/100000000)) =
	// 1000012340.2189723259

	// Costs for new supply:
	// 100000000 * ln(e^(100000100/100000000) + e^(1000000000/100000000)) =
	// 1000012340.2313117896

	// Costs to mint the first coin:
	// 1000012340.2313117896 - 1000012340.2189723259 = 0.0123394637
	let costs = curve.calculate_costs(low, high, vec![passive_issuance]).unwrap();

	let expected_costs = Float::from_str("0.0123394637").unwrap();

	assert_relative_eq(costs, expected_costs, Float::from_str("0.0000001").unwrap());
}

// The main consequences of the low liquidity parameter is a lack of
// representable coins. e^40 goes beyond the representable range of the [Float]
// type.
#[test]
fn low_liquidity_parameter() {
	// Create curve with liquidity parameter b=100, and passive issuance=0
	let m = Float::from_num(100);
	let curve = LMSRParameters { m };

	// we mint 100 coins.
	let low = Float::from_num(1_000u128);
	let high = Float::from_num(1_100u128);

	let passive_issuance = Float::from_num(0);

	// Costs for existing supply:
	// 100 * ln(e^(1000/100) + e^(0/100)) = 1000.0045398899216865

	// Costs for new supply:
	// 100 * ln(e^(1100/100) + e^(0/100)) = 1100.0016701561318394

	// Costs to mint the first coin:
	// 1100.0016701561318394 - 1000.0045398899216865 = 99.9971302662101529
	let costs = curve.calculate_costs(low, high, vec![passive_issuance]).unwrap();

	let expected_costs = Float::from_str("99.9971302662101529").unwrap();

	assert_relative_eq(costs, expected_costs, Float::from_str("0.0000000001").unwrap());
}

#[test]
fn mint_coin_with_existing_supply_and_no_passive_issuance() {
	// Create curve with liquidity parameter b=100_000_000, and passive issuance=0
	let m = Float::from_num(100_000_000);

	let curve = LMSRParameters { m };

	// we mint 100 coins.
	let low = Float::from_num(100u128);
	let high = Float::from_num(101u128);

	let passive_issuance = Float::from_num(0);

	// Costs for existing supply:
	// 100000000 * ln(e^(100/100000000) + e^(0/100000000)) =
	// 69314768.056007030941723211624

	// Costs for new supply:
	// 100000000 * ln(e^(101/100000000) + e^(0/100000000)) =
	// 69314768.55600728219172321160383640

	//Costs to mint the first coin:
	// 69314768.55600728219172321160383640 - 69314768.55600728219172321160383640 =
	// 0.5000002412499999999798364
	let costs = curve.calculate_costs(low, high, vec![passive_issuance]).unwrap();

	let expected_costs = Float::from_str("0.50000024125").unwrap();

	assert_relative_eq(costs, expected_costs, Float::from_str("0.00000001").unwrap());
}

#[test]
fn mint_coin_with_existing_supply_and_passive_issuance() {
	// Create curve with liquidity parameter b=100_000_000, and passive issuance=100
	let m = Float::from_num(100_000_000);

	let curve = LMSRParameters { m };

	// we mint 100 coins.
	let low = Float::from_num(100u128);
	let high = Float::from_num(101u128);

	let passive_issuance = Float::from_num(100);

	// Costs for existing supply:
	// 100000000 * ln(e^(100/100000000) + e^(100/100000000)) = 69314818.055994530942

	// Costs for new supply:
	// 100000000 * ln(e^(101/100000000) + e^(100/100000000)) = 69314818.555994532192

	// Costs to mint the first coin:
	// 69314818.555994532192 - 69314818.055994530942 = 0.50000000125
	let costs = curve.calculate_costs(low, high, vec![passive_issuance]).unwrap();

	let expected_costs = Float::from_str("0.50000000125").unwrap();

	assert_relative_eq(costs, expected_costs, Float::from_str("0.00000001").unwrap());
}
