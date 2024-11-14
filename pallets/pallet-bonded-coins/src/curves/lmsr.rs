use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_arithmetic::ArithmeticError;
use sp_std::ops::{AddAssign, BitOrAssign, ShlAssign};
use sp_std::vec::Vec;
use substrate_fixed::{
	traits::{Fixed, FixedSigned, FixedUnsigned, ToFixed},
	transcendental::{exp, ln},
};

use super::BondingFunction;
use crate::{PassiveSupply, Precision};

#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct LMSRParametersInput<Parameter> {
	pub m: Parameter,
}

#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct LMSRParameters<Parameter> {
	pub m: Parameter,
}

impl<I: FixedUnsigned, C: FixedSigned> TryFrom<LMSRParametersInput<I>> for LMSRParameters<C> {
	type Error = ();
	fn try_from(value: LMSRParametersInput<I>) -> Result<Self, Self::Error> {
		Ok(LMSRParameters {
			m: C::checked_from_fixed(value.m).ok_or(())?,
		})
	}
}

impl<Parameter> LMSRParameters<Parameter>
where
	Parameter: FixedSigned + PartialOrd<Precision> + From<Precision> + ToFixed,
	<Parameter as Fixed>::Bits: Copy + ToFixed + AddAssign + BitOrAssign + ShlAssign,
{
	fn calculate_exp_term(&self, supply: Parameter, x: Parameter) -> Result<Parameter, ArithmeticError> {
		supply
			.checked_sub(x)
			.ok_or(ArithmeticError::Underflow)?
			.checked_div(self.m)
			.ok_or(ArithmeticError::DivisionByZero)
			.and_then(|exponent| exp::<Parameter, Parameter>(exponent).map_err(|_| ArithmeticError::Overflow))
	}
}

impl<Parameter> BondingFunction<Parameter> for LMSRParameters<Parameter>
where
	Parameter: FixedSigned + PartialOrd<Precision> + From<Precision> + ToFixed,
	<Parameter as Fixed>::Bits: Copy + ToFixed + AddAssign + BitOrAssign + ShlAssign,
{
	// c(a, c) = (a - c) + b * ln((1 + SUM_i e^((q_i - a)/b)) / (1 + SUM_i e^((q_i - c)/b)))
	fn calculate_costs(
		&self,
		low: Parameter,
		high: Parameter,
		passive_supply: PassiveSupply<Parameter>,
	) -> Result<Parameter, ArithmeticError> {
		let e_term_numerator = passive_supply
			.iter()
			.map(|x| self.calculate_exp_term(*x, high))
			.collect::<Result<Vec<Parameter>, ArithmeticError>>()?;

		let term1 = e_term_numerator.iter().try_fold(Parameter::from_num(0), |acc, x| {
			acc.checked_add(*x).ok_or(ArithmeticError::Overflow)
		})?;

		let numerator = Parameter::from_num(1)
			.checked_add(term1)
			.ok_or(ArithmeticError::Overflow)?;

		let e_term_denominator = passive_supply
			.iter()
			.map(|x| self.calculate_exp_term(*x, low))
			.collect::<Result<Vec<Parameter>, ArithmeticError>>()?;

		let term2 = e_term_denominator.iter().try_fold(Parameter::from_num(0), |acc, x| {
			acc.checked_add(*x).ok_or(ArithmeticError::Overflow)
		})?;

		let denominator = Parameter::from_num(1)
			.checked_add(term2)
			.ok_or(ArithmeticError::Overflow)?;

		let log_value = numerator
			.checked_div(denominator)
			.ok_or(ArithmeticError::DivisionByZero)
			.and_then(|x| ln::<Parameter, Parameter>(x).map_err(|_| ArithmeticError::Overflow))?;

		let high_low_diff = high.checked_sub(low).ok_or(ArithmeticError::Underflow)?;

		let m_log_value = self.m.checked_mul(log_value).ok_or(ArithmeticError::Overflow)?;

		high_low_diff.checked_add(m_log_value).ok_or(ArithmeticError::Overflow)
	}
}
