use frame_support::assert_err;
use sp_runtime::{ArithmeticError, FixedPointNumber, FixedU128};

use crate::curves_parameters::RationalBondingFunctionParameters;

#[test]
fn test_all_zero() {
	let first_coin_supply = FixedU128::from_u32(0);
	let second_coin_supply = FixedU128::from_u32(0);

	let ratio = RationalBondingFunctionParameters::<FixedU128>::calculate_ration(first_coin_supply, second_coin_supply);

	assert_eq!(ratio, Ok(FixedU128::from_inner(0)));
}

#[test]
fn test_basic() {
	// A total supply of 30. 10 A Coins and 20 B Coins
	let first_coin_supply = FixedU128::from_u32(10);
	let second_coin_supply = FixedU128::from_u32(20);

	//(1/2 * (10^2 + 10* 20^2 + 20^3)) / ((10+30)^2) = 6.7222..
	let ratio = RationalBondingFunctionParameters::<FixedU128>::calculate_ration(first_coin_supply, second_coin_supply);

	assert_eq!(ratio, Ok(FixedU128::from_inner(6722222222222222222)));
}

#[test]
fn test_coin_supply_0() {
	// A total supply of 10. 10 A Coins and 0 B Coins
	let first_coin_supply = FixedU128::from_u32(10);
	let second_coin_supply = FixedU128::from_u32(0);

	//(1/2 * (10^2 + 10* 0^2 + 0^3)) / ((10+0)^2) = 0.5
	let ratio = RationalBondingFunctionParameters::<FixedU128>::calculate_ration(first_coin_supply, second_coin_supply);

	assert_eq!(ratio, Ok(FixedU128::from_rational(1, 2)));
}

#[test]
fn test_large_values() {
	// A total supply of 30. 10 A Coins and 20 B Coins
	let first_coin_supply = FixedU128::from_u32(100000);
	let second_coin_supply = FixedU128::from_u32(200000);

	//(1/2 * (100000^2 + 10* 200000^2 + 200000^3)) / ((100000+200000)^2) = 66666.722222222222222222..
	let ratio = RationalBondingFunctionParameters::<FixedU128>::calculate_ration(first_coin_supply, second_coin_supply);

	assert_eq!(
		ratio,
		Ok(FixedU128::from_rational(66666722222222222222222, FixedU128::DIV))
	);
}

#[test]
fn test_overflow() {
	// A total supply of 30. 10 A Coins and 20 B Coins
	let first_coin_supply = FixedU128::from_inner(u128::MAX);
	let second_coin_supply = FixedU128::from_u32(200000);

	//(1/2 * (100000^2 + 10* 200000^2 + 200000^3)) / ((100000+200000)^2) = 66666.722222222222222222..
	let ratio = RationalBondingFunctionParameters::<FixedU128>::calculate_ration(first_coin_supply, second_coin_supply);

	assert_err!(ratio, ArithmeticError::Overflow);
}
