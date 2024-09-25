use frame_support::assert_err;
use sp_arithmetic::{ArithmeticError, FixedI128, FixedU128};
use sp_runtime::FixedPointNumber;

use crate::curves_parameters::{BondingFunction, LinearBondingFunctionParameters};

#[test]
fn test_all_zero() {
	let params = LinearBondingFunctionParameters {
		m: FixedU128::from(0),
		n: FixedU128::from(0),
	};
	let x = FixedU128::from(0);
	assert_eq!(params.get_value(x), Ok(FixedU128::from(0)));
}

#[test]
fn test_basic_test() {
	let m = FixedU128::from_u32(1);
	let n = FixedU128::from_u32(2);
	let x = FixedU128::from_u32(1);

	let curve = LinearBondingFunctionParameters { m, n };

	// 1*1^2 + 2*1 = 3
	let result = curve.get_value(x).unwrap();
	assert_eq!(result, FixedU128::from_u32(3));
}

#[test]
fn test_fraction() {
	let m = FixedU128::from_rational(1, 2);
	let n = FixedU128::from_u32(2);
	let x = FixedU128::from_u32(1);

	let curve = LinearBondingFunctionParameters { m, n };

	// 0.5*1^2 + 2*1 = 2.5
	let result = curve.get_value(x).unwrap();
	assert_eq!(result, FixedU128::from_rational(5, 2));
}

#[test]
fn test_large_values() {
	let params = LinearBondingFunctionParameters {
		m: FixedU128::from_u32(1000000),
		n: FixedU128::from_u32(1000000),
	};
	let x = FixedU128::from_u32(1);
	// 1000000 * 1^2 + 1000000 * 1  = 2000000
	assert_eq!(params.get_value(x), Ok(FixedU128::from(2000000)));
}

#[test]
fn test_large_x() {
	let params = LinearBondingFunctionParameters {
		m: FixedU128::from(2),
		n: FixedU128::from(3),
	};
	let x = FixedU128::from_u32(1000000000);

	// 2*1000000000^2 + 3*1000000000 = 2000000003000000000
	assert_eq!(params.get_value(x), Ok(FixedU128::from(2000000003000000000)));
}

#[test]
fn test_negative() {
	let params = LinearBondingFunctionParameters {
		m: FixedI128::from(2),
		n: FixedI128::from(3),
	};
	let x = FixedI128::from(-4);
	// 2*(-4)^2 + 3*(-4) = 32 - 12 = 20
	assert_eq!(params.get_value(x), Ok(FixedI128::from(20)));
}

#[test]
fn test_negative_m() {
	let params = LinearBondingFunctionParameters {
		m: FixedI128::from(-2),
		n: FixedI128::from(3),
	};
	let x = FixedI128::from(4);
	// -2*4^2 + 3*4 = -32 + 12 = -20
	assert_err!(params.get_value(x), ArithmeticError::Underflow);
}

#[test]
fn test_negative_m_and_n() {
	let params = LinearBondingFunctionParameters {
		m: FixedI128::from(-2),
		n: FixedI128::from(-3),
	};
	let x = FixedI128::from(4);
	// -2*4^2 - 3*4 = -32 - 12 = -44
	assert_err!(params.get_value(x), ArithmeticError::Underflow);
}

#[test]
fn test_negative_n() {
	let params = LinearBondingFunctionParameters {
		m: FixedI128::from(2),
		n: FixedI128::from(-3),
	};
	let x = FixedI128::from(4);
	// 2*4^2 - 3*4 = 32 - 12 = 20
	assert_eq!(params.get_value(x), Ok(FixedI128::from(20)));
}

#[test]
fn test_overflow_m() {
	let m = FixedU128::from_inner(u128::MAX);
	let n = FixedU128::from_inner(1);
	let x = FixedU128::from_u32(2);

	let curve = LinearBondingFunctionParameters { m, n };

	let result = curve.get_value(x);
	assert_err!(result, ArithmeticError::Overflow);
}

#[test]
fn test_overflow_n() {
	let m = FixedU128::from_u32(1);
	let n = FixedU128::from_inner(u128::MAX);
	let x = FixedU128::from_u32(2);

	let curve = LinearBondingFunctionParameters { m, n };

	let result = curve.get_value(x);
	assert_err!(result, ArithmeticError::Overflow);
}

#[test]
fn test_precision_large_fraction() {
	let m = FixedU128::from_rational(999999, 1000000); // 0.999999
	let n = FixedU128::from_rational(999999, 1000000); // 0.999999
	let x = FixedU128::from_rational(999999, 1000000); // 0.999999

	let curve = LinearBondingFunctionParameters { m, n };

	// 0.999999*(0.999999^2) + 0.999999*0.999999
	// = 1.999995000003999999
	let result = curve.get_value(x).unwrap();
	assert_eq!(result, FixedU128::from_rational(1999995000003999999, FixedU128::DIV));
}

#[test]
fn test_precision_mixed_fraction() {
	// 0.3
	let m = FixedU128::from_rational(3, 10);
	// 0.75
	let n = FixedU128::from_rational(3, 4);
	let x = FixedU128::from_u32(1); // 1

	let curve = LinearBondingFunctionParameters { m, n };

	// 0.3*(1) + 0.75*1
	// = 1.05
	let result = curve.get_value(x).unwrap();
	assert_eq!(result, FixedU128::from_rational(105, 100));
}

#[test]
fn test_precision_small_fraction() {
	// 0.001
	let m = FixedU128::from_rational(1, 1000);
	let n = FixedU128::from_rational(1, 1000);
	// 1
	let x = FixedU128::from_u32(1);

	let curve = LinearBondingFunctionParameters { m, n };

	// 0.001*(1^2) + 0.001*1
	// = 0.002
	let result = curve.get_value(x).unwrap();
	assert_eq!(result, FixedU128::from_rational(2, 1000));
}

#[test]
fn test_zero_x() {
	let params = LinearBondingFunctionParameters {
		m: FixedI128::from(2),
		n: FixedI128::from(3),
	};
	let x = FixedI128::from(0);

	// 2*0^2 + 3*0  = 0
	assert_eq!(params.get_value(x), Ok(FixedI128::from(0)));
}
