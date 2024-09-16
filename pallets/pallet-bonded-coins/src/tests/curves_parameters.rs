use frame_support::assert_err;
use sp_arithmetic::{ArithmeticError, FixedU128};
use sp_runtime::traits::Zero;

use crate::curves_parameters::{
	transform_denomination_currency_amount, BondingFunction, LinearBondingFunctionParameters,
};

#[test]
fn test_linear_bonding_function_basic_test() {
	let m = FixedU128::from_u32(1);
	let n = FixedU128::from_u32(2);
	let x = FixedU128::from_u32(1);

	let curve = LinearBondingFunctionParameters { m, n };

	// 1*1^2 + 2*1 = 3
	let result = curve.get_value(x).unwrap();
	assert_eq!(result, FixedU128::from_u32(3));
}

#[test]
fn test_linear_bonding_function_fraction() {
	let m = FixedU128::from_rational(1, 2);
	let n = FixedU128::from_u32(2);
	let x = FixedU128::from_u32(1);

	let curve = LinearBondingFunctionParameters { m, n };

	// 0.5*1^2 + 2*1 = 2
	let result = curve.get_value(x).unwrap();
	assert_eq!(result, FixedU128::from_u32(2));
}

#[test]
fn test_linear_bonding_overflow_n() {
	let m = FixedU128::from_u32(1);
	let n = FixedU128::from_inner(u128::MAX);
	let x = FixedU128::from_u32(2);

	let curve = LinearBondingFunctionParameters { m, n };

	let result = curve.get_value(x);
	assert_err!(result, ArithmeticError::Overflow);
}

#[test]
fn test_linear_bonding_overflow_m() {
	let m = FixedU128::from_inner(u128::MAX);
	let n = FixedU128::from_inner(1);
	let x = FixedU128::from_u32(2);

	let curve = LinearBondingFunctionParameters { m, n };

	let result = curve.get_value(x);
	assert_err!(result, ArithmeticError::Overflow);
}

#[test]
fn test_increase_denomination_currency_amount() {
	let amount = 100;
	let current_denomination = 2;
	let target_denomination = 3;

	let result = transform_denomination_currency_amount(amount, current_denomination, target_denomination).unwrap();
	assert_eq!(result, FixedU128::from_inner(1000));
}

#[test]
fn test_decrease_denomination_currency_amount() {
	let amount = 1000;
	let current_denomination = 3;
	let target_denomination = 2;

	let result = transform_denomination_currency_amount(amount, current_denomination, target_denomination).unwrap();
	assert_eq!(result, FixedU128::from_inner(100));
}

#[test]
fn test_increase_denomination_overflow() {
	let amount = u128::MAX;
	let current_denomination = 10;

	// just increase denomination by one. This should overflow
	let target_denomination = 11;

	let result = transform_denomination_currency_amount(amount, current_denomination, target_denomination);
	assert_err!(result, ArithmeticError::Overflow);
}

#[test]
fn test_decrease_denomination_underflow() {
	let amount = 1;
	let current_denomination = 5;

	// just increase
	let target_denomination = 4;

	// we should have dropped all relevant bits. This should gives use an Ok with zero
	let result = transform_denomination_currency_amount(amount, current_denomination, target_denomination).unwrap();
	assert_eq!(result, FixedU128::zero())
}
