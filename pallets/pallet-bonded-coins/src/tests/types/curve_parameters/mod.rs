mod polynomial_parameters;
mod square_root_parameters;

mod ratio_parameters;

use frame_support::assert_err;
use sp_arithmetic::{ArithmeticError, FixedI128, FixedU128};
use sp_runtime::traits::Zero;

use crate::{
	curves_parameters::{convert_currency_amount, utils::*},
	mock::runtime::*,
};

#[test]
fn test_increase_denomination_currency_amount() {
	let amount = 100;
	let current_denomination = 100;
	let target_denomination = 1000;

	let result = convert_currency_amount::<Test>(amount, current_denomination, target_denomination).unwrap();
	assert_eq!(result, FixedU128::from_inner(1000));
}

#[test]
fn test_decrease_denomination_currency_amount() {
	let amount = 1000;
	let current_denomination = 1000;
	let target_denomination = 100;

	let result = convert_currency_amount::<Test>(amount, current_denomination, target_denomination).unwrap();
	assert_eq!(result, FixedU128::from_inner(100));
}

#[test]
fn test_increase_denomination_overflow() {
	let amount = u128::MAX;
	let current_denomination = 10000000000;

	// just increase denomination by one. This should overflow
	let target_denomination = 100000000000;

	let result = convert_currency_amount::<Test>(amount, current_denomination, target_denomination);
	assert_err!(result, ArithmeticError::Overflow);
}

#[test]
fn test_decrease_denomination_underflow() {
	let amount = 1;
	let current_denomination = 100000;

	// just increase
	let target_denomination = 10000;

	// we should have dropped all relevant bits. This should gives use an Ok with zero
	let result = convert_currency_amount::<Test>(amount, current_denomination, target_denomination).unwrap();
	assert_eq!(result, FixedU128::zero())
}

#[test]
fn test_get_power_2_positive() {
	let x = FixedU128::from_u32(2);
	let result = get_power_2(x).unwrap();
	assert_eq!(result, FixedU128::from_u32(4));
}

#[test]
fn test_get_power_2_zero() {
	let x = FixedU128::from_u32(0);
	let result = get_power_2(x).unwrap();
	assert_eq!(result, FixedU128::from_u32(0));
}

#[test]
fn test_get_power_2_negative() {
	let x = FixedI128::from(-2);
	let result = get_power_2(x).unwrap();
	assert_eq!(result, FixedI128::from_u32(4));
}

#[test]
fn test_get_power_2_overflow() {
	let x = FixedU128::from_inner(u128::MAX);
	let result = get_power_2(x);
	assert!(result.is_err());
}

#[test]
fn test_get_power_3_positive() {
	let x = FixedU128::from_u32(2);
	let result = get_power_3(x).unwrap();
	assert_eq!(result, FixedU128::from_u32(8));
}

#[test]
fn test_get_power_3_zero() {
	let x = FixedU128::from_u32(0);
	let result = get_power_3(x).unwrap();
	assert_eq!(result, FixedU128::from_u32(0));
}

#[test]
fn test_get_power_3_negative() {
	let x = FixedI128::from(-2);
	let result = get_power_3(x).unwrap();
	assert_eq!(result, FixedI128::from(-8));
}

#[test]
fn test_get_power_3_overflow() {
	let x = FixedU128::from_inner(u128::MAX);
	let result = get_power_3(x);
	assert!(result.is_err());
}
