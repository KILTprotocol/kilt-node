use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_arithmetic::ArithmeticError;
use sp_std::ops::{AddAssign, BitOrAssign, ShlAssign};
use substrate_fixed::{
	traits::{Fixed, FixedSigned, ToFixed},
	transcendental::{exp, ln},
};

use super::BondingFunction;
use crate::{curves::Operation, PassiveSupply, Precision};

#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct LMSRFunctionParameters<Parameter> {
	pub m: Parameter,
}

impl<Parameter> LMSRFunctionParameters<Parameter>
where
	Parameter: FixedSigned + PartialOrd<Precision> + From<Precision> + ToFixed,
	<Parameter as Fixed>::Bits: Copy + ToFixed + AddAssign + BitOrAssign + ShlAssign,
{
	fn calculate_passive_issuance(&self, x: Parameter) -> Result<Parameter, ArithmeticError> {
		x.checked_div(self.m)
			.ok_or(ArithmeticError::DivisionByZero)
			.and_then(|x| exp::<Parameter, Parameter>(x).map_err(|_| ArithmeticError::Overflow))
	}
}

impl<Parameter> BondingFunction<Parameter> for LMSRFunctionParameters<Parameter>
where
	Parameter: FixedSigned + PartialOrd<Precision> + From<Precision> + ToFixed,
	<Parameter as Fixed>::Bits: Copy + ToFixed + AddAssign + BitOrAssign + ShlAssign,
{
	fn calculate_costs(
		&self,
		low: Parameter,
		high: Parameter,
		op: Operation<PassiveSupply<Parameter>>,
	) -> Result<Parameter, ArithmeticError> {
		let passive_issuance_over_e = op
			.inner_value()
			.iter()
			.map(|x| self.calculate_passive_issuance(*x))
			.collect::<Result<Vec<Parameter>, ArithmeticError>>()?;

		let passive_issuance = passive_issuance_over_e
			.iter()
			.try_fold(Parameter::from_num(0), |acc, x| {
				acc.checked_add(*x).ok_or(ArithmeticError::Overflow)
			})?;

		let exponent_numerator = passive_issuance
			.checked_sub(high)
			.ok_or(ArithmeticError::Underflow)?
			.checked_div(self.m)
			.ok_or(ArithmeticError::DivisionByZero)?;

		let exponent_denominator = passive_issuance
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
