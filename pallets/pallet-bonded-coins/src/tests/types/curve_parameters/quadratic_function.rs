use frame_support::assert_err;
use sp_arithmetic::{ArithmeticError, FixedI128, FixedU128};
use sp_runtime::FixedPointNumber;

use crate::curves_parameters::{BondingFunction, PolynomialFunctionParameters};

#[test]
fn test_all_zero() {
	let params = PolynomialFunctionParameters {
		m: FixedU128::from(0),
		n: FixedU128::from(0),
		o: FixedU128::from(0),
	};
	let x = FixedU128::from(0);
	assert_eq!(params.get_value(x), Ok(FixedU128::from(0)));
}

#[test]
fn test_basic_test() {
	let m = FixedU128::from_u32(1);
	let n = FixedU128::from_u32(2);
	let o = FixedU128::from_u32(3);
	let x = FixedU128::from_u32(1);

	let curve = PolynomialFunctionParameters { m, n, o };

	// 1*1^3 + 2*1^2 + 3* 1 = 6
	let result = curve.get_value(x).unwrap();
	assert_eq!(result, FixedU128::from_u32(6));
}

#[test]
fn test_fraction() {
	let m = FixedU128::from_rational(1, 2);
	let n = FixedU128::from_u32(2);
	let o = FixedU128::from_u32(3);
	let x = FixedU128::from_u32(1);

	let curve = PolynomialFunctionParameters { m, n, o };

	// 0.5*1^3 + 2*1^2 + 3* 1 = 5.5
	let result = curve.get_value(x).unwrap();
	assert_eq!(result, FixedU128::from_rational(11, 2));
}

#[test]
fn test_large_values() {
	let params = PolynomialFunctionParameters {
		m: FixedU128::from_u32(1000000),
		n: FixedU128::from_u32(1000000),
		o: FixedU128::from_u32(1000000),
	};
	let x = FixedU128::from_u32(1);
	// 1000000 * 1^3 + 1000000 * 1^2 + 1000000 * 1 = 3000000
	assert_eq!(params.get_value(x), Ok(FixedU128::from(3000000)));
}

#[test]
fn test_large_x() {
	let params = PolynomialFunctionParameters {
		m: FixedU128::from(2),
		n: FixedU128::from(3),
		o: FixedU128::from(4),
	};
	let x = FixedU128::from(1000000);

	// 2*1000000^3 + 3*1000000^2 + 4*1000000 = 2000003000004000000
	assert_eq!(params.get_value(x), Ok(FixedU128::from(2000003000004000000)));
}

#[test]
fn test_negative() {
	let params = PolynomialFunctionParameters {
		m: FixedI128::from(2),
		n: FixedI128::from(3),
		o: FixedI128::from(4),
	};
	let x = FixedI128::from(-4);

	// 2*(-4)^3 + 3*(-4)^2 + 4 * -4 = -96
	assert_err!(params.get_value(x), ArithmeticError::Underflow);
}

#[test]
fn test_negative_m() {
	let params = PolynomialFunctionParameters {
		m: FixedI128::from(-2),
		n: FixedI128::from(3),
		o: FixedI128::from(4),
	};
	let x = FixedI128::from(4);
	// -2*4^3 + 3*4^2 + 4 * 4 = -128 + 48 + 16 = -64
	assert_err!(params.get_value(x), ArithmeticError::Underflow);
}

#[test]
fn test_negative_m_and_n() {
	let params = PolynomialFunctionParameters {
		m: FixedI128::from(-2),
		n: FixedI128::from(-3),
		o: FixedI128::from(4),
	};
	let x = FixedI128::from(4);

	// -2*4^3 - 3*4^2 + 4 *4 = -128 - 48 + 16  = -160
	assert_err!(params.get_value(x), ArithmeticError::Underflow);
}

#[test]
fn test_negative_m_n_and_o() {
	let params = PolynomialFunctionParameters {
		m: FixedI128::from(-2),
		n: FixedI128::from(-3),
		o: FixedI128::from(-4),
	};
	let x = FixedI128::from(4);

	// -2*4^3 - 3*4^2 - 4*4 = -128 - 48 - 16 = -192
	assert_err!(params.get_value(x), ArithmeticError::Underflow);
}

#[test]
fn test_negative_n() {
	let params = PolynomialFunctionParameters {
		m: FixedI128::from(2),
		n: FixedI128::from(-3),
		o: FixedI128::from(4),
	};
	let x = FixedI128::from(4);
	// 2*4^3 - 3*4^2 + 4 * 6 = 128 - 48 + 16 = 96
	assert_eq!(params.get_value(x), Ok(FixedI128::from(96)));
}

#[test]
fn test_overflow_m() {
	let m = FixedU128::from_inner(u128::MAX);
	let n = FixedU128::from_inner(1);
	let o = FixedU128::from_inner(1);
	let x = FixedU128::from_u32(2);

	let curve = PolynomialFunctionParameters { m, n, o };

	let result = curve.get_value(x);
	assert_err!(result, ArithmeticError::Overflow);
}

#[test]
fn test_overflow_n() {
	let m = FixedU128::from_u32(1);
	let n = FixedU128::from_inner(u128::MAX);
	let o = FixedU128::from_u32(1);
	let x = FixedU128::from_u32(2);

	let curve = PolynomialFunctionParameters { m, n, o };

	let result = curve.get_value(x);
	assert_err!(result, ArithmeticError::Overflow);
}

#[test]
fn test_overflow_o() {
	let m = FixedU128::from_u32(1);
	let n = FixedU128::from_u32(1);
	let o = FixedU128::from_inner(u128::MAX);
	let x = FixedU128::from_u32(2);

	let curve = PolynomialFunctionParameters { m, n, o };

	let result = curve.get_value(x);
	assert_err!(result, ArithmeticError::Overflow);
}

#[test]
fn test_precision_large_fraction() {
	let m = FixedU128::from_rational(999999, 1000000); // 0.999999
	let n = FixedU128::from_rational(999999, 1000000); // 0.999999
	let o = FixedU128::from_rational(999999, 1000000); // 0.999999
	let x = FixedU128::from_rational(999999, 1000000); // 0.999999

	let curve = PolynomialFunctionParameters { m, n, o };

	// 0.999999*(0.999999^3) + 0.999999*0.999999^2 + 0.999999*0.999999
	// = 2.999991000009999995000001
	let result = curve.get_value(x).unwrap();
	assert_eq!(result, FixedU128::from_rational(2999991000009999995, FixedU128::DIV));
}

#[test]
fn test_precision_mixed_fraction() {
	// 0.3
	let m = FixedU128::from_rational(3, 10);
	// 0.75
	let n = FixedU128::from_rational(3, 4);
	// 0.5
	let o = FixedU128::from_rational(1, 2);
	let x = FixedU128::from_u32(1); // 1

	let curve = PolynomialFunctionParameters { m, n, o };

	// 0.3*(1^3) + 0.75*1^2 + 0.5*1
	// = 1.55
	let result = curve.get_value(x).unwrap();
	assert_eq!(result, FixedU128::from_rational(155, 100));
}

#[test]
fn test_precision_small_fraction() {
	// 0.001
	let m = FixedU128::from_rational(1, 1000);
	let n = FixedU128::from_rational(1, 1000);
	let o = FixedU128::from_rational(1, 1000);
	// 1
	let x = FixedU128::from_u32(1);

	let curve = PolynomialFunctionParameters { m, n, o };

	// 0.001*(1^3) + 0.001*1^2 + 0.001*1
	// = 0.003
	let result = curve.get_value(x).unwrap();
	assert_eq!(result, FixedU128::from_rational(3, 1000));
}

#[test]
fn test_zero_x() {
	let params = PolynomialFunctionParameters {
		m: FixedI128::from(2),
		n: FixedI128::from(3),
		o: FixedI128::from(4),
	};
	let x = FixedI128::from(0);

	// 2*0^3 + 3*0^2 + 4*0 = 0
	assert_eq!(params.get_value(x), Ok(FixedI128::from(0)));
}
