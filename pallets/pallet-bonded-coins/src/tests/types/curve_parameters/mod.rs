mod polynomial_parameters;
mod square_root_parameters;

use crate::{
	mock::{runtime::*, Float},
	types::convert_balance_to_parameter,
};

#[test]
fn test_scale_down_balance_by_denomination() {
	let amount = 10 * 10u128.pow(10);
	let current_denomination = 10;

	let result = convert_balance_to_parameter::<Test>(amount, &current_denomination).unwrap();
	assert_eq!(result, Float::from_num(10));
}

#[test]
fn test_decrease_denomination_underflow() {
	let amount = 1;
	let current_denomination = 10;

	// we should have dropped all relevant bits. This should gives use an Ok with zero
	let result = convert_balance_to_parameter::<Test>(amount, &current_denomination).unwrap();
	assert_eq!(result, Float::from_num(0))
}
