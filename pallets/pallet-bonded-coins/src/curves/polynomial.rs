use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_arithmetic::ArithmeticError;
use substrate_fixed::traits::FixedSigned;

use super::{square, BondingFunction};

#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct PolynomialParameters<Parameter> {
	pub m: Parameter,
	pub n: Parameter,
	pub o: Parameter,
}

impl<Parameter> BondingFunction<Parameter> for PolynomialParameters<Parameter>
where
	Parameter: FixedSigned,
{
	fn calculate_costs(&self, low: Parameter, high: Parameter) -> Result<Parameter, ArithmeticError> {
		// Calculate high - low
		let delta_x = high.checked_sub(low).ok_or(ArithmeticError::Underflow)?;

		let high_low_mul = high.checked_mul(low).ok_or(ArithmeticError::Overflow)?;
		let high_square = square(high)?;
		let low_square = square(low)?;

		// Factorized cubic term:  (high^2 + high * low + low^2)
		let cubic_term = high_square
			.checked_add(high_low_mul)
			.ok_or(ArithmeticError::Overflow)?
			.checked_add(low_square)
			.ok_or(ArithmeticError::Overflow)?;

		// Calculate m * (high^2 + high * low + low^2)
		let term1 = self.m.checked_mul(cubic_term).ok_or(ArithmeticError::Overflow)?;

		let high_plus_low = high.checked_add(low).ok_or(ArithmeticError::Overflow)?;

		// Calculate n * (high + low)
		let term2 = self.n.checked_mul(high_plus_low).ok_or(ArithmeticError::Overflow)?;

		// Final calculation with factored (high - low)
		let result = term1
			.checked_add(term2)
			.ok_or(ArithmeticError::Overflow)?
			.checked_add(self.o)
			.ok_or(ArithmeticError::Overflow)?;

		result.checked_mul(delta_x).ok_or(ArithmeticError::Overflow)
	}
}
