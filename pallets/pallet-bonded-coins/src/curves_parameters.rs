use frame_support::ensure;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_arithmetic::ArithmeticError;
use substrate_fixed::{
	traits::{FixedSigned, ToFixed},
	transcendental::sqrt,
	types::I9F23,
};

// SquareRoot trait and implementation

// BondingFunction trait
pub trait BondingFunction<F: FixedSigned + PartialOrd> {
	/// returns the value of the curve at x.
	/// The bonding curve is already the primitive integral of f(x).
	/// Therefore the costs can be calculated by the difference of the values of the curve at two points.
	fn get_value(&self, x: F) -> Result<F, ArithmeticError>;

	/// calculates the cost of the curve between low and high
	fn calculate_costs(&self, low: F, high: F) -> Result<F, ArithmeticError> {
		let high_val = self.get_value(high)?;
		let low_val = self.get_value(low)?;
		let result = high_val.checked_sub(low_val).ok_or(ArithmeticError::Underflow)?;
		Ok(result)
	}
}

// PolynomialFunctionParameters struct and implementation
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct PolynomialFunctionParameters<F> {
	pub m: F,
	pub n: F,
	pub o: F,
}

impl<F> BondingFunction<F> for PolynomialFunctionParameters<F>
where
	F: FixedSigned,
{
	/// F(x) = m * x^3 + n * x^2 + o * x
	fn get_value(&self, x: F) -> Result<F, ArithmeticError> {
		let x2 = x.checked_mul(x).ok_or(ArithmeticError::Overflow)?;
		let x3 = x2.checked_mul(x).ok_or(ArithmeticError::Overflow)?;

		let mx3 = self.m.clone().checked_mul(x3).ok_or(ArithmeticError::Overflow)?;
		let nx2 = self.n.clone().checked_mul(x2).ok_or(ArithmeticError::Overflow)?;
		let ox = self.o.clone().checked_mul(x).ok_or(ArithmeticError::Overflow)?;

		let result = mx3
			.checked_add(nx2)
			.ok_or(ArithmeticError::Overflow)?
			.checked_add(ox)
			.ok_or(ArithmeticError::Overflow)?;

		ensure!(result >= F::from_num(0), ArithmeticError::Underflow);
		Ok(result)
	}
}

// SquareRootBondingFunctionParameters struct and implementation
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct SquareRootBondingFunctionParameters<F> {
	pub m: F,
	pub n: F,
}

impl<F> BondingFunction<F> for SquareRootBondingFunctionParameters<F>
where
	F: FixedSigned + PartialOrd<I9F23> + From<I9F23> + ToFixed,
{
	/// F(x) = m * sqrt(x^3) + n * x
	fn get_value(&self, x: F) -> Result<F, ArithmeticError> {
		let x2 = x.checked_mul(x).ok_or(ArithmeticError::Overflow)?;
		let x3 = x2.checked_mul(x).ok_or(ArithmeticError::Overflow)?;

		let sqrt_x3 = sqrt(x3).map_err(|_| ArithmeticError::Overflow)?;
		let mx3 = self.m.clone().checked_mul(sqrt_x3).ok_or(ArithmeticError::Overflow)?;
		let nx = self.n.clone().checked_mul(x).ok_or(ArithmeticError::Overflow)?;

		let result = mx3.checked_add(nx).ok_or(ArithmeticError::Overflow)?;

		ensure!(result >= F::from_num(0), ArithmeticError::Underflow);
		Ok(result)
	}
}
