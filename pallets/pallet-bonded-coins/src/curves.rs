use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_arithmetic::ArithmeticError;
use sp_std::ops::{AddAssign, BitOrAssign, ShlAssign};
use substrate_fixed::{
	traits::{Fixed, FixedSigned, ToFixed},
	transcendental::{exp, ln, sqrt},
};

use crate::{PassiveSupply, Precision};

#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub enum Curve<Parameter> {
	Polynomial(PolynomialParameters<Parameter>),
	SquareRoot(SquareRootParameters<Parameter>),
	LMSR(LMSRFunctionParameters<Parameter>),
}

pub enum Operation<PassiveSupply> {
	Mint(PassiveSupply),
	Burn(PassiveSupply),
}

impl<Balance> Operation<PassiveSupply<Balance>> {
	pub fn inner_value(&self) -> &PassiveSupply<Balance> {
		match self {
			Operation::Mint(x) => x,
			Operation::Burn(x) => x,
		}
	}
}

impl<Parameter> Curve<Parameter>
where
	Parameter: FixedSigned + PartialOrd<Precision> + From<Precision> + ToFixed,
	<Parameter as Fixed>::Bits: Copy + ToFixed + AddAssign + BitOrAssign + ShlAssign,
{
	fn calculate_accumulated_passive_issuance<Balance: Fixed>(passive_issuance: &[Balance]) -> Balance {
		passive_issuance
			.iter()
			.fold(Balance::from_num(0), |sum, x| sum.saturating_add(*x))
	}

	fn calculate_integral_bounds(
		op: Operation<PassiveSupply<Parameter>>,
		active_issuance_pre: Parameter,
		active_issuance_post: Parameter,
	) -> (Parameter, Parameter) {
		match op {
			Operation::Burn(passive) => {
				let accumulated_passive_issuance = Self::calculate_accumulated_passive_issuance(&passive);
				(
					active_issuance_post.saturating_add(accumulated_passive_issuance),
					active_issuance_pre.saturating_add(accumulated_passive_issuance),
				)
			}
			Operation::Mint(passive) => {
				let accumulated_passive_issuance = Self::calculate_accumulated_passive_issuance(&passive);
				(
					active_issuance_pre.saturating_add(accumulated_passive_issuance),
					active_issuance_post.saturating_add(accumulated_passive_issuance),
				)
			}
		}
	}
	pub fn calculate_cost(
		&self,
		active_issuance_pre: Parameter,
		active_issuance_post: Parameter,
		op: Operation<PassiveSupply<Parameter>>,
	) -> Result<Parameter, ArithmeticError> {
		match self {
			Curve::Polynomial(params) => {
				let (low, high) = Self::calculate_integral_bounds(op, active_issuance_pre, active_issuance_post);

				params.calculate_costs(low, high)
			}
			Curve::SquareRoot(params) => {
				let (low, high) = Self::calculate_integral_bounds(op, active_issuance_pre, active_issuance_post);
				params.calculate_costs(low, high)
			}
			Curve::LMSR(params) => {
				let passive_issuance_over_e = op
					.inner_value()
					.iter()
					.map(|x| params.calculate_passive_issuance(*x))
					.collect::<Result<Vec<Parameter>, ArithmeticError>>()?;

				let passive_issuance = passive_issuance_over_e
					.iter()
					.try_fold(Parameter::from_num(0), |acc, x| {
						acc.checked_add(*x).ok_or(ArithmeticError::Overflow)
					})?;

				let lmsr_calc = LMSRCalculation {
					m: params.m,
					passive_issuance,
				};
				lmsr_calc.calculate_costs(active_issuance_pre, active_issuance_post)
			}
		}
	}
}

pub trait BondingFunction<Parameter: Fixed> {
	fn calculate_costs(&self, low: Parameter, high: Parameter) -> Result<Parameter, ArithmeticError>;

	fn square(x: Parameter) -> Result<Parameter, ArithmeticError> {
		x.checked_mul(x).ok_or(ArithmeticError::Overflow)
	}
}

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
		let high_square = Self::square(high)?;
		let low_square = Self::square(low)?;

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
