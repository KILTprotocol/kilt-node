use frame_support::assert_err;
use sp_arithmetic::{ArithmeticError, FixedI128, FixedU128};
use sp_runtime::FixedPointNumber;

use crate::curves_parameters::{BondingFunction, SquareRootBondingFunctionParameters};

#[test]
fn test_all_zero() {
	let params = SquareRootBondingFunctionParameters {
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

	let curve = SquareRootBondingFunctionParameters { m, n };

	// 1*sqrt(1^3) + 2*1 = 1 + 2 = 3
	let result = curve.get_value(x).unwrap();
	assert_eq!(result, FixedU128::from_u32(3));
}

#[test]
fn test_fraction() {
	let m = FixedU128::from_rational(1, 2);
	let n = FixedU128::from_u32(2);
	let x = FixedU128::from_u32(1);

	let curve = SquareRootBondingFunctionParameters { m, n };

	// 0.5*sqrt(1^3) + 2*1 = 0.5*1 + 2 = 2.5
	let result = curve.get_value(x).unwrap();
	assert_eq!(result, FixedU128::from_rational(5, 2));
}

#[test]
fn test_large_values() {
	let params = SquareRootBondingFunctionParameters {
		m: FixedI128::from_u32(1000000),
		n: FixedI128::from_u32(1000000),
	};
	let x = FixedI128::from_u32(1000000);
	// 1000000*sqrt(1000000^3) + 1000000*1000000 = 1001000000000000
	assert_eq!(params.get_value(x), Ok(FixedI128::from(1001000000000000)));
}

#[test]
fn test_negative() {
	let params = SquareRootBondingFunctionParameters {
		m: FixedI128::from(2),
		n: FixedI128::from(3),
	};
	let x = FixedI128::from(-4);
	// 2*sqrt((-4)^3) + 3*(-4) = 2*sqrt(-64) - 12 = 2*8i - 12 (complex number)
	// Since sqrt of negative number is not defined in real numbers, it should return an error
	let result = params.get_value(x);
	assert_err!(result, ArithmeticError::Underflow);
}

#[test]
fn test_negative_m() {
	let params = SquareRootBondingFunctionParameters {
		m: FixedI128::from(-2),
		n: FixedI128::from(3),
	};
	let x = FixedI128::from(4);
	// -2*sqrt(4^3) + 3*4 = -2*sqrt(64) + 12 = -2*8 + 12 = -16 + 12 = -4
	assert_err!(params.get_value(x), ArithmeticError::Underflow);
}

#[test]
fn test_negative_m_and_n() {
	let params = SquareRootBondingFunctionParameters {
		m: FixedI128::from(-2),
		n: FixedI128::from(-3),
	};
	let x = FixedI128::from(4);
	// -2*sqrt(4^3) - 3*4 = -2*sqrt(64) - 12 = -2*8 - 12 = -16 - 12 = -28
	assert_err!(params.get_value(x), ArithmeticError::Underflow);
}

#[test]
fn test_negative_n() {
	let params = SquareRootBondingFunctionParameters {
		m: FixedI128::from(2),
		n: FixedI128::from(-3),
	};
	let x = FixedI128::from(4);
	// 2*sqrt(4^3) - 3*4 = 2*sqrt(64) - 12 = 2*8 - 12 = 16 - 12 = 4
	assert_eq!(params.get_value(x), Ok(FixedI128::from(4)));
}

#[test]
fn test_overflow_m() {
	let m = FixedU128::from_inner(u128::MAX);
	let n = FixedU128::from_inner(1);
	let x = FixedU128::from_u32(2);

	let curve = SquareRootBondingFunctionParameters { m, n };

	let result = curve.get_value(x);
	assert_err!(result, ArithmeticError::Overflow);
}

#[test]
fn test_overflow_n() {
	let m = FixedU128::from_u32(1);
	let n = FixedU128::from_inner(u128::MAX);
	let x = FixedU128::from_u32(2);

	let curve = SquareRootBondingFunctionParameters { m, n };

	let result = curve.get_value(x);
	assert_err!(result, ArithmeticError::Overflow);
}

#[test]
fn test_precision_large_fraction() {
	let m = FixedU128::from_rational(999999, 1000000); // 0.999999
	let n = FixedU128::from_rational(999999, 1000000); // 0.999999
	let x = FixedU128::from_rational(999999, 1000000); // 0.999999

	let curve = SquareRootBondingFunctionParameters { m, n };

	// 0.999999*sqrt(0.999999^3) + 0.999999*0.999999
	// = 1.999995500002874999
	let result = curve.get_value(x).unwrap();
	assert_eq!(result, FixedU128::from_rational(1999995500002874999, FixedU128::DIV));
}
#[test]
fn test_precision_mixed_fraction() {
	let m = FixedU128::from_rational(3, 10); // 0.3
	let n = FixedU128::from_rational(3, 4); // 0.75
	let x = FixedU128::from_rational(1, 2); // 0.5

	let curve = SquareRootBondingFunctionParameters { m, n };

	// 0.3*sqrt(0.5^3) + 0.75*0.5
	// = 0.481066017177982128
	let result = curve.get_value(x).unwrap();
	assert_eq!(result, FixedU128::from_rational(481066017177982128, FixedU128::DIV));
}

#[test]
fn test_precision_small_fraction() {
	let m = FixedU128::from_rational(1, 1000); // 0.001
	let n = FixedU128::from_rational(1, 1000); // 0.001
	let x = FixedU128::from_rational(1, 1000); // 0.001

	let curve = SquareRootBondingFunctionParameters { m, n };

	// 0.001*sqrt(0.001^3) + 0.001*0.001
	// = 0.000001031622776601
	let result = curve.get_value(x).unwrap();
	assert_eq!(result, FixedU128::from_rational(1031622776601, FixedU128::DIV));
}

#[test]
fn test_zero_m_and_n() {
	let params = SquareRootBondingFunctionParameters {
		m: FixedI128::from(0),
		n: FixedI128::from(0),
	};
	let x = FixedI128::from(4);
	// 0*sqrt(4^3) + 0*4 = 0
	assert_eq!(params.get_value(x), Ok(FixedI128::from(0)));
}

#[test]
fn test_zero_x() {
	let params = SquareRootBondingFunctionParameters {
		m: FixedI128::from(2),
		n: FixedI128::from(3),
	};
	let x = FixedI128::from(0);
	// 2*sqrt(0^3) + 3*0 = 0
	assert_eq!(params.get_value(x), Ok(FixedI128::from(0)));
}
