use crate::{
	curves::{convert_to_fixed, round},
	mock::{runtime::Test, Float},
	types::Round,
};
use frame_support::assert_ok;
use sp_runtime::ArithmeticError;

const DEFAULT_ROUND_KIND: Round = Round::Down;

#[test]
fn test_convert_to_fixed_basic() {
	let x = 1000u128;
	let denomination = 2u8; // 10^2 = 100

	let result = convert_to_fixed::<Test>(x, denomination, &DEFAULT_ROUND_KIND).unwrap();
	// Test runtime uses I75F53 for CurveParameterTypeOf, which is what we'll cover
	// in testing.
	let expected = Float::from_num(10); // 1000 / 100 = 10

	assert_eq!(result, expected);
}

#[test]
fn test_convert_to_fixed_with_remainder() {
	let x = 1050u128;
	let denomination = 2u8; // 10^2 = 100

	let result = convert_to_fixed::<Test>(x, denomination, &DEFAULT_ROUND_KIND).unwrap();
	let expected = Float::from_num(10.5); // 1050 / 100 = 10.5

	assert_eq!(result, expected);
}

#[test]
fn test_convert_to_fixed_smaller_than_denomination() {
	let x = 1050u128;
	let denomination = 6u8; // 10^6 = 1000000

	let result = convert_to_fixed::<Test>(x, denomination, &DEFAULT_ROUND_KIND).unwrap();
	let expected = Float::from_num(0.00105); // 1050 / 1000000 = 0.00105

	assert_eq!(result, expected);
}

#[test]
fn test_convert_to_fixed_large_value() {
	let x = 1_000_000_000_000_000u128;
	let denomination = 12u8; // 10^12 = 1_000_000_000_000

	let result = convert_to_fixed::<Test>(x, denomination, &DEFAULT_ROUND_KIND).unwrap();
	let expected = Float::from_num(1000); // 1_000_000_000_000_000 / 1_000_000_000_000 = 1000

	assert_eq!(result, expected);
}

#[test]
fn test_convert_to_fixed_small_denomination() {
	let x = 12345u128;
	let denomination = 1u8; // 10^1 = 10

	let result = convert_to_fixed::<Test>(x, denomination, &DEFAULT_ROUND_KIND).unwrap();
	let expected = Float::from_num(1234.5); // 12345 / 10 = 1234.5

	assert_eq!(result, expected);
}

#[test]
fn test_convert_to_fixed_overflow() {
	let x = u128::MAX;
	let denomination = 0u8; // 10^0 = 1, no scaling

	let result = convert_to_fixed::<Test>(x, denomination, &DEFAULT_ROUND_KIND);
	assert!(result.is_err());
	assert_eq!(result.unwrap_err(), ArithmeticError::Overflow);
}

#[test]
fn test_convert_to_fixed_denomination_overflow() {
	let x = 1000u128;
	let denomination = 128u8; // 10^128 overflows

	let result = convert_to_fixed::<Test>(x, denomination, &DEFAULT_ROUND_KIND);
	assert!(result.is_err());
	assert_eq!(result.unwrap_err(), ArithmeticError::Overflow);
}

#[test]
fn test_convert_to_fixed_overflow_avoided() {
	let x = u128::MAX; // around 3.4e+38
	let denomination = 17u8; // I75F53 should handle around 1.8e+22, 38 - 23 -> 17

	let result = convert_to_fixed::<Test>(x, denomination, &DEFAULT_ROUND_KIND);
	assert_ok!(result);
}

#[test]
fn test_convert_to_fixed_handles_large_denomination() {
	let x = u128::MAX; // around 3.4e+38
	let denomination = 22u8; // I75F53 should handle around 1.8e+22; this is the maximum safe denomination

	let result = convert_to_fixed::<Test>(x, denomination, &DEFAULT_ROUND_KIND);
	assert_ok!(result);
}

#[test]
fn test_convert_to_fixed_very_large_denomination() {
	let denomination = 30u8; // I75F53 should handle around 1.8e+22, this can lead to overflow

	// multiple of denomination would not result in remainder = 0
	assert_ok!(convert_to_fixed::<Test>(
		10u128.pow(31),
		denomination,
		&DEFAULT_ROUND_KIND
	));

	// non-multiples of denomination could lead to overflow of remainder
	assert_ok!(convert_to_fixed::<Test>(
		11u128.pow(31),
		denomination,
		&DEFAULT_ROUND_KIND
	));
	assert_ok!(convert_to_fixed::<Test>(
		10u128.pow(29),
		denomination,
		&DEFAULT_ROUND_KIND
	));
}

#[test]
fn test_convert_to_fixed_zero_denomination() {
	let x = 1000u128;
	let denomination = 0u8; // 10^0 = 1

	let result = convert_to_fixed::<Test>(x, denomination, &DEFAULT_ROUND_KIND).unwrap();
	let expected = Float::from_num(1000); // 1000 / 1 = 1000

	assert_eq!(result, expected);
}

#[test]
fn test_convert_to_fixed_zero_input() {
	let x = 0u128;
	let denomination = 10u8; // 10^10 = large divisor

	let result = convert_to_fixed::<Test>(x, denomination, &DEFAULT_ROUND_KIND).unwrap();
	let expected = Float::from_num(0); // 0 / any number = 0

	assert_eq!(result, expected);
}

#[test]
fn test_convert_to_fixed_round_up() {
	let x = 1001u128;
	let denomination = 4u8; // 10^2 = 100

	let result = convert_to_fixed::<Test>(x, denomination, &Round::Up).unwrap();

	assert!(result > Float::from_num(0.1001));

	let result = convert_to_fixed::<Test>(x, denomination, &DEFAULT_ROUND_KIND).unwrap();

	assert_eq!(result, Float::from_num(0.1001));
}

#[test]
fn test_convert_to_fixed_round_up_representable() {
	let x = 1125u128;
	let denomination = 3u8; // 10^2 = 100

	let result1 = convert_to_fixed::<Test>(x, denomination, &Round::Up).unwrap();

	assert_eq!(result1, Float::from_num(1.125));

	let result2 = convert_to_fixed::<Test>(x, denomination, &DEFAULT_ROUND_KIND).unwrap();

	assert_eq!(result1, result2);
}

#[test]
fn test_round_up() {
	// let value = Float::from_str("1.123456789").unwrap();

	// let b = U256::from(value.frac().to_bits());

	// let result = round::<Test>(value, Round::Up, 2).unwrap();

	// assert_eq!(result, Float::from_str("1.12").unwrap());
}
