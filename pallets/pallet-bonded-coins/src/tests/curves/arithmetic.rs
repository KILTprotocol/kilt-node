// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2024 BOTLabs GmbH

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

// If you feel like getting in touch with us, you can do so at info@botlabs.org
use crate::{
	curves::{balance_to_fixed, fixed_to_balance},
	mock::runtime::Float,
	types::Round,
};
use frame_support::assert_ok;
use sp_runtime::ArithmeticError;

const DEFAULT_ROUND_KIND: Round = Round::Down;

#[test]
fn test_balance_to_fixed_basic() {
	let x = 1000u128;
	let denomination = 2u8; // 10^2 = 100

	let result = balance_to_fixed::<u128, Float>(x, denomination, DEFAULT_ROUND_KIND).unwrap();
	// Test runtime uses I75F53 for CurveParameterTypeOf, which is what we'll cover
	// in testing.
	let expected = Float::from_num(10); // 1000 / 100 = 10

	assert_eq!(result, expected);
}

#[test]
fn test_balance_to_fixed_with_remainder() {
	let x = 1050u128;
	let denomination = 2u8; // 10^2 = 100

	let result = balance_to_fixed::<u128, Float>(x, denomination, DEFAULT_ROUND_KIND).unwrap();
	let expected = Float::from_num(10.5); // 1050 / 100 = 10.5

	assert_eq!(result, expected);
}

#[test]
fn test_balance_to_fixed_smaller_than_denomination() {
	let x = 1050u128;
	let denomination = 6u8; // 10^6 = 1000000

	let result = balance_to_fixed::<u128, Float>(x, denomination, DEFAULT_ROUND_KIND).unwrap();
	let expected = Float::from_num(0.00105); // 1050 / 1000000 = 0.00105

	assert_eq!(result, expected);
}

#[test]
fn test_balance_to_fixed_large_value() {
	let x = 1_000_000_000_000_000u128;
	// 10^12 = 1_000_000_000_000
	let denomination = 12u8;

	let result = balance_to_fixed::<u128, Float>(x, denomination, DEFAULT_ROUND_KIND).unwrap();
	let expected = Float::from_num(1000); // 1_000_000_000_000_000 / 1_000_000_000_000 = 1000

	assert_eq!(result, expected);
}

#[test]
fn test_balance_to_fixed_small_denomination() {
	let x = 12345u128;
	// 10^1 = 10
	let denomination = 1u8;

	let result = balance_to_fixed::<u128, Float>(x, denomination, DEFAULT_ROUND_KIND).unwrap();
	let expected = Float::from_num(1234.5); // 12345 / 10 = 1234.5

	assert_eq!(result, expected);
}

#[test]
fn test_balance_to_fixed_overflow() {
	let x = u128::MAX;
	// 10^0 = 1, no scaling
	let denomination = 0u8;

	let result = balance_to_fixed::<u128, Float>(x, denomination, DEFAULT_ROUND_KIND);
	result.unwrap_err();
	assert_eq!(result.unwrap_err(), ArithmeticError::Overflow);
}

#[test]
fn test_balance_to_fixed_denomination_overflow() {
	let x = 1000u128;
	// 10^128 overflows
	let denomination = 128u8;

	let result = balance_to_fixed::<u128, Float>(x, denomination, DEFAULT_ROUND_KIND);
	result.unwrap_err();
	assert_eq!(result.unwrap_err(), ArithmeticError::Overflow);
}

#[test]
fn test_balance_to_fixed_overflow_avoided() {
	let x = u128::MAX; // around 3.4e+38
	let denomination = 17u8; // I75F53 should handle around 1.8e+22, 38 - 23 -> 17

	let result = balance_to_fixed::<u128, Float>(x, denomination, DEFAULT_ROUND_KIND);
	assert_ok!(result);
}

#[test]
fn test_balance_to_fixed_handles_large_denomination() {
	let x = u128::MAX; // around 3.4e+38
	let denomination = 22u8; // I75F53 should handle around 1.8e+22; this is the maximum safe denomination

	let result = balance_to_fixed::<u128, Float>(x, denomination, DEFAULT_ROUND_KIND);
	assert_ok!(result);
}

#[test]
fn test_balance_to_fixed_very_large_denomination() {
	let denomination = 30u8; // I75F53 should handle around 1.8e+22, this can lead to overflow

	// multiple of denomination would not result in remainder = 0
	assert_ok!(balance_to_fixed::<u128, Float>(
		10u128.pow(31),
		denomination,
		DEFAULT_ROUND_KIND
	));

	// non-multiples of denomination could lead to overflow of remainder
	assert_ok!(balance_to_fixed::<u128, Float>(
		11u128.pow(31),
		denomination,
		DEFAULT_ROUND_KIND
	));
	assert_ok!(balance_to_fixed::<u128, Float>(
		10u128.pow(29),
		denomination,
		DEFAULT_ROUND_KIND
	));
}

#[test]
fn test_balance_to_fixed_zero_denomination() {
	let x = 1000u128;
	let denomination = 0u8; // 10^0 = 1

	let result = balance_to_fixed::<u128, Float>(x, denomination, DEFAULT_ROUND_KIND).unwrap();
	let expected = Float::from_num(1000); // 1000 / 1 = 1000

	assert_eq!(result, expected);
}

#[test]
fn test_balance_to_fixed_zero_input() {
	let x = 0u128;
	let denomination = 10u8; // 10^10 = large divisor

	let result = balance_to_fixed::<u128, Float>(x, denomination, DEFAULT_ROUND_KIND).unwrap();
	let expected = Float::from_num(0); // 0 / any number = 0

	assert_eq!(result, expected);
}

#[test]
fn test_balance_to_fixed_round_up() {
	let x = 1001u128;
	let denomination = 4u8; // 10^2 = 100

	let result = balance_to_fixed::<u128, Float>(x, denomination, Round::Up).unwrap();

	assert!(result > Float::from_num(0.1001));

	let result = balance_to_fixed::<u128, Float>(x, denomination, DEFAULT_ROUND_KIND).unwrap();

	assert_eq!(result, Float::from_num(0.1001));
}

#[test]
fn test_balance_to_fixed_round_up_representable() {
	let x = 1125u128;
	let denomination = 3u8; // 10^2 = 100

	let result1 = balance_to_fixed::<u128, Float>(x, denomination, Round::Up).unwrap();

	assert_eq!(result1, Float::from_num(1.125));

	let result2 = balance_to_fixed::<u128, Float>(x, denomination, DEFAULT_ROUND_KIND).unwrap();

	assert_eq!(result1, result2);
}

#[test]
fn test_round_up() {
	let value = Float::from_num(1.1200000000005);
	let result = fixed_to_balance::<u128, Float>(value, 2, Round::Up).unwrap();
	assert_eq!(result, 113u128);
}

#[test]
fn test_round_up_exact_representable() {
	let value = Float::from_num(1.125);
	let result = fixed_to_balance::<u128, Float>(value, 3, Round::Up).unwrap();
	assert_eq!(result, 1125u128);
}
