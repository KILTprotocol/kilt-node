use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_arithmetic::{fixed_point::FixedPointNumber, ArithmeticError};
use sp_runtime::FixedU128;

/// A trait to define the bonding curve functions
pub trait BondingFunction<F: FixedPointNumber> {
	/// returns the value of the curve at x.
	/// The bonding curve is already the primitive integral of f(x).
	/// Therefore the costs can be calculated by the difference of the values of the curve at two points.
	fn get_value(&self, x: F) -> Result<F, ArithmeticError>;

	/// static function to calculate the power of 2 of x
	fn get_power_2(x: F) -> Result<F, ArithmeticError> {
		Ok(x.saturating_mul(x))
	}

	/// static function to calculate the power of 3 of x
	fn get_power_3(x: F) -> Result<F, ArithmeticError> {
		Ok(Self::get_power_2(x)?.saturating_mul(x))
	}

	/// calculates the cost of the curve between low and high
	fn calculate_costs(&self, low: F, high: F) -> Result<F, ArithmeticError> {
		let high_val = self.get_value(high)?;
		let low_val = self.get_value(low)?;
		Ok(high_val.saturating_sub(low_val))
	}
}

/// A linear bonding function with the shape of f(x) = mx + n,
///  which results in the primitive integral F(x) = m' * x^2 + n * x.
/// It is expected that the user provides the correct parameters for the curve.
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct LinearBondingFunctionParameters<F> {
	pub m: F,
	pub n: F,
}

impl<F> BondingFunction<F> for LinearBondingFunctionParameters<F>
where
	F: FixedPointNumber,
{
	/// we
	fn get_value(&self, x: F) -> Result<F, ArithmeticError> {
		let x2 = Self::get_power_2(x)?;

		let mx2 = self.m.clone().checked_mul(&x2).ok_or(ArithmeticError::Overflow)?;
		let nx = self.n.clone().checked_mul(&x).ok_or(ArithmeticError::Overflow)?;

		let result = mx2.checked_add(&nx).ok_or(ArithmeticError::Overflow)?;

		// we do not need the fractions here. So we truncate the result
		Ok(result)
	}
}

pub fn transform_denomination_currency_amount(
	amount: u128,
	current_denomination: u8,
	target_denomination: u8,
) -> Result<FixedU128, ArithmeticError> {
	let diff = target_denomination as i8 - current_denomination as i8;
	let value = {
		if diff > 0 {
			let factor = 10u128.pow(diff as u32);
			amount.checked_mul(factor).ok_or(ArithmeticError::Overflow)
		} else {
			let factor = 10u128.pow(diff.abs() as u32);
			// Dividing by zero can never happen 10^0 = 1. Lets just be save.
			amount.checked_div(factor).ok_or(ArithmeticError::DivisionByZero)
		}
	}?;

	Ok(FixedU128::from_inner(value))
}
