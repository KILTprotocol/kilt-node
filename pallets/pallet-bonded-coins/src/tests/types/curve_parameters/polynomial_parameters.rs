use crate::{
	curves_parameters::{BondingFunction, PolynomialFunctionParameters},
	mock::Float,
};
use frame_support::assert_err;
use sp_arithmetic::ArithmeticError;

#[test]
fn test_all_zero() {
	let params = PolynomialFunctionParameters {
		m: Float::from(0),
		n: Float::from(0),
		o: Float::from(0),
	};
	let x = Float::from(0);
	assert_eq!(params.get_value(x), Ok(Float::from(0)));
}

#[test]
fn test_basic_test() {
	let m = Float::from_num(1);
	let n = Float::from_num(2);
	let o = Float::from_num(3);
	let x = Float::from_num(1);

	let curve = PolynomialFunctionParameters { m, n, o };

	// 1*1^3 + 2*1^2 + 3* 1 = 6
	let result = curve.get_value(x).unwrap();
	assert_eq!(result, Float::from_num(6));
}

#[test]
fn test_fraction() {
	let m = Float::from_num(0.5);
	let n = Float::from_num(2);
	let o = Float::from_num(3);
	let x = Float::from_num(1);

	let curve = PolynomialFunctionParameters { m, n, o };

	// 0.5*1^3 + 2*1^2 + 3* 1 = 5.5
	let result = curve.get_value(x).unwrap();
	assert_eq!(result, Float::from_num(5.5));
}

#[test]
fn test_large_values() {
	let params = PolynomialFunctionParameters {
		m: Float::from_num(1000000),
		n: Float::from_num(1000000),
		o: Float::from_num(1000000),
	};
	let x = Float::from_num(1);
	// 1000000 * 1^3 + 1000000 * 1^2 + 1000000 * 1 = 3000000
	assert_eq!(params.get_value(x), Ok(Float::from(3000000)));
}

#[test]
fn test_large_x() {
	let params = PolynomialFunctionParameters {
		m: Float::from(2),
		n: Float::from(3),
		o: Float::from(4),
	};
	let x = Float::from_num(10_000_000u128);

	println!("max value: {:?}", Float::max_value());

	// 2*10_000_000^3 + 3*10_000_000^2 + 4*10_000_000 = 2000000300000040000000
	assert_eq!(params.get_value(x), Ok(Float::from_num(2000000300000040000000u128)));
}

#[test]
fn test_negative() {
	let params = PolynomialFunctionParameters {
		m: Float::from(2),
		n: Float::from(3),
		o: Float::from(4),
	};
	let x = Float::from(-4);

	// 2*(-4)^3 + 3*(-4)^2 + 4 * -4 = -96
	assert_err!(params.get_value(x), ArithmeticError::Underflow);
}

#[test]
fn test_negative_m() {
	let params = PolynomialFunctionParameters {
		m: Float::from(-2),
		n: Float::from(3),
		o: Float::from(4),
	};
	let x = Float::from(4);
	// -2*4^3 + 3*4^2 + 4 * 4 = -128 + 48 + 16 = -64
	assert_err!(params.get_value(x), ArithmeticError::Underflow);
}

#[test]
fn test_negative_m_and_n() {
	let params = PolynomialFunctionParameters {
		m: Float::from(-2),
		n: Float::from(-3),
		o: Float::from(4),
	};
	let x = Float::from(4);

	// -2*4^3 - 3*4^2 + 4 *4 = -128 - 48 + 16  = -160
	assert_err!(params.get_value(x), ArithmeticError::Underflow);
}

#[test]
fn test_negative_m_n_and_o() {
	let params = PolynomialFunctionParameters {
		m: Float::from(-2),
		n: Float::from(-3),
		o: Float::from(-4),
	};
	let x = Float::from(4);

	// -2*4^3 - 3*4^2 - 4*4 = -128 - 48 - 16 = -192
	assert_err!(params.get_value(x), ArithmeticError::Underflow);
}

#[test]
fn test_negative_n() {
	let params = PolynomialFunctionParameters {
		m: Float::from(2),
		n: Float::from(-3),
		o: Float::from(4),
	};
	let x = Float::from(4);
	// 2*4^3 - 3*4^2 + 4 * 6 = 128 - 48 + 16 = 96
	assert_eq!(params.get_value(x), Ok(Float::from(96)));
}

#[test]
fn test_overflow_m() {
	let m = Float::from_num(Float::max_value());
	let n = Float::from_num(1);
	let o = Float::from_num(1);
	let x = Float::from_num(2);

	let curve = PolynomialFunctionParameters { m, n, o };

	let result = curve.get_value(x);
	assert_err!(result, ArithmeticError::Overflow);
}

#[test]
fn test_overflow_n() {
	let m = Float::from_num(1);
	let n = Float::from_num(Float::max_value());
	let o = Float::from_num(1);
	let x = Float::from_num(2);

	let curve = PolynomialFunctionParameters { m, n, o };

	let result = curve.get_value(x);
	assert_err!(result, ArithmeticError::Overflow);
}

#[test]
fn test_overflow_o() {
	let m = Float::from_num(1);
	let n = Float::from_num(1);
	let o = Float::from_num(Float::max_value());
	let x = Float::from_num(2);

	let curve = PolynomialFunctionParameters { m, n, o };

	let result = curve.get_value(x);
	assert_err!(result, ArithmeticError::Overflow);
}

#[test]
fn test_precision_large_fraction() {
	let m = Float::from_num(0.999999); // 0.999999
	let n = Float::from_num(0.999999); // 0.999999
	let o = Float::from_num(0.999999); // 0.999999
	let x = Float::from_num(0.999999); // 0.999999

	let curve = PolynomialFunctionParameters { m, n, o };

	// 0.999999*(0.999999^3) + 0.999999*0.999999^2 + 0.999999*0.999999
	// = 2.999991000009999995000001 >  2.9999910000099999. Error Here. We get 2.9999910000099995

	let result = curve.get_value(x).unwrap();
	assert_eq!(result, Float::from_num(2.9999910000099999));
}

#[test]
fn test_precision_mixed_fraction() {
	// 0.3
	let m = Float::from_num(0.3);
	// 0.75
	let n = Float::from_num(0.75);
	// 0.5
	let o = Float::from_num(0.5);
	let x = Float::from_num(1);

	let curve = PolynomialFunctionParameters { m, n, o };

	// 0.3*(1^3) + 0.75*1^2 + 0.5*1
	// = 1.55
	let result = curve.get_value(x).unwrap();
	assert_eq!(result, Float::from_num(1.55));
}

#[test]
fn test_precision_small_fraction() {
	// 0.001
	let m = Float::from_num(0.001);
	let n = Float::from_num(0.001);
	let o = Float::from_num(0.001);
	// 1
	let x = Float::from_num(1);

	let curve = PolynomialFunctionParameters { m, n, o };

	// 0.001*(1^3) + 0.001*1^2 + 0.001*1
	// = 0.003
	let result = curve.get_value(x).unwrap();
	assert_eq!(result, Float::from_num(0.003));
}

#[test]
fn test_zero_x() {
	let params = PolynomialFunctionParameters {
		m: Float::from(2),
		n: Float::from(3),
		o: Float::from(4),
	};
	let x = Float::from(0);

	// 2*0^3 + 3*0^2 + 4*0 = 0
	assert_eq!(params.get_value(x), Ok(Float::from(0)));
}
