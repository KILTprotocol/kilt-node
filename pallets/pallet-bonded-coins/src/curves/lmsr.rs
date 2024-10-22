use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_arithmetic::ArithmeticError;
use sp_std::ops::{AddAssign, BitOrAssign, ShlAssign};
use substrate_fixed::{
	traits::{Fixed, FixedSigned, ToFixed},
	transcendental::{exp, ln},
};

use super::BondingFunction;
use crate::Precision;

#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct LMSRFunctionParameters<Parameter> {
	pub m: Parameter,
}

impl<Parameter> LMSRFunctionParameters<Parameter>
where
	Parameter: FixedSigned + PartialOrd<Precision> + From<Precision> + ToFixed,
	<Parameter as Fixed>::Bits: Copy + ToFixed + AddAssign + BitOrAssign + ShlAssign,
{
	pub(crate) fn calculate_passive_issuance(&self, x: Parameter) -> Result<Parameter, ArithmeticError> {
		x.checked_div(self.m)
			.ok_or(ArithmeticError::DivisionByZero)
			.and_then(|x| exp::<Parameter, Parameter>(x).map_err(|_| ArithmeticError::Overflow))
	}
}

pub struct LMSRCalculation<Parameter> {
	pub m: Parameter,
	pub passive_issuance: Parameter,
}

impl<Parameter> BondingFunction<Parameter> for LMSRCalculation<Parameter>
where
	Parameter: FixedSigned + PartialOrd<Precision> + From<Precision> + ToFixed,
	<Parameter as Fixed>::Bits: Copy + ToFixed + AddAssign + BitOrAssign + ShlAssign,
{
	fn calculate_costs(&self, low: Parameter, high: Parameter) -> Result<Parameter, ArithmeticError> {
		let exponent_numerator = self
			.passive_issuance
			.checked_sub(high)
			.ok_or(ArithmeticError::Underflow)?
			.checked_div(self.m)
			.ok_or(ArithmeticError::DivisionByZero)?;

		let exponent_denominator = self
			.passive_issuance
			.checked_sub(low)
			.ok_or(ArithmeticError::Underflow)?
			.checked_div(self.m)
			.ok_or(ArithmeticError::DivisionByZero)?;

		let e_term_numerator = exp::<Parameter, Parameter>(exponent_numerator)
			.map_err(|_| ArithmeticError::Overflow)
			.and_then(|x| x.checked_add(Parameter::from_num(1)).ok_or(ArithmeticError::Overflow))?;

		let e_term_denominator = exp::<Parameter, Parameter>(exponent_denominator)
			.map_err(|_| ArithmeticError::Overflow)
			.and_then(|x| x.checked_add(Parameter::from_num(1)).ok_or(ArithmeticError::Overflow))?;

		let e_term = e_term_numerator
			.checked_div(e_term_denominator)
			.ok_or(ArithmeticError::DivisionByZero)?;

		let term1 = self
			.m
			.checked_mul(ln::<Parameter, Parameter>(e_term).map_err(|_| ArithmeticError::Overflow)?)
			.ok_or(ArithmeticError::Underflow)?;

		let high_low_diff = high.checked_sub(low).ok_or(ArithmeticError::Underflow)?;

		high_low_diff.checked_add(term1).ok_or(ArithmeticError::Overflow)
	}
}
