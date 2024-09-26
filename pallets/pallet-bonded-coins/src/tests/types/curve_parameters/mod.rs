mod polynomial_function;
mod square_root_function;

mod ratio_function;

use frame_support::assert_err;
use sp_arithmetic::{ArithmeticError, FixedU128};
use sp_runtime::traits::Zero;

use crate::{curves_parameters::convert_currency_amount, mock::runtime::*};

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
