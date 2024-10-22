use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_arithmetic::ArithmeticError;
use substrate_fixed::{
	traits::{FixedSigned, ToFixed},
	transcendental::sqrt,
};

use super::BondingFunction;
use crate::Precision;

#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct SquareRootParameters<Parameter> {
	pub m: Parameter,
	pub n: Parameter,
}

impl<Parameter> BondingFunction<Parameter> for SquareRootParameters<Parameter>
where
	Parameter: FixedSigned + PartialOrd<Precision> + From<Precision> + ToFixed,
{
	fn calculate_costs(&self, low: Parameter, high: Parameter) -> Result<Parameter, ArithmeticError> {
		// Ensure that high and low are positive (logarithms of negative numbers are undefined)

		// Calculate sqrt(high^3) and sqrt(low^3)
		let sqrt_x3_high: Parameter = sqrt::<Parameter, Parameter>(high)
			.map_err(|_| ArithmeticError::Underflow)?
			.checked_mul(high)
			.ok_or(ArithmeticError::Overflow)?;

		let sqrt_x3_low: Parameter = sqrt::<Parameter, Parameter>(low)
			.map_err(|_| ArithmeticError::Underflow)?
			.checked_mul(low)
			.ok_or(ArithmeticError::Overflow)?;

		let delta_sqrt_x3 = sqrt_x3_high
			.checked_sub(sqrt_x3_low)
			.ok_or(ArithmeticError::Underflow)?;

		let term1 = self.m.checked_mul(delta_sqrt_x3).ok_or(ArithmeticError::Overflow)?;

		// Calculate n * (high - low) (linear term)
		let delta_x = high.checked_sub(low).ok_or(ArithmeticError::Underflow)?;
		let term2 = self.n.checked_mul(delta_x).ok_or(ArithmeticError::Overflow)?;

		// Calculate the final result (sqrt + linear terms)
		term1.checked_add(term2).ok_or(ArithmeticError::Overflow)
	}
}
