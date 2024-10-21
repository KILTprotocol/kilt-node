use core::ops::{AddAssign, BitOrAssign, ShlAssign};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_arithmetic::ArithmeticError;
use substrate_fixed::{
	traits::{Fixed, FixedSigned, ToFixed},
	transcendental::{exp, ln, sqrt},
	types::I9F23,
};

#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub enum Curve<F> {
	PolynomialFunction(PolynomialFunctionParameters<F>),
	SquareRootBondingFunction(SquareRootFunctionParameters<F>),
	LMSR(LMSRFunctionParameters<F>),
}

pub enum Operation<F> {
	Mint(Vec<F>),
	Burn(Vec<F>),
}

impl<F> Operation<F> {
	pub fn inner_value(&self) -> &[F] {
		match self {
			Operation::Mint(x) => x,
			Operation::Burn(x) => x,
		}
	}
}

impl<F> Curve<F>
where
	F: FixedSigned + PartialOrd<I9F23> + From<I9F23> + ToFixed,
	<F as Fixed>::Bits: Copy + ToFixed + AddAssign + BitOrAssign + ShlAssign,
{
	fn calculate_accumulated_passive_issuance(passive_issuance: &[F]) -> F {
		passive_issuance
			.iter()
			.fold(F::from_num(0), |sum, x| sum.saturating_add(*x))
	}

	fn calculate_low_high(op: Operation<F>, active_issuance_pre: F, active_issuance_post: F) -> (F, F) {
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
		active_issuance_pre: F,
		active_issuance_post: F,
		op: Operation<F>,
	) -> Result<F, ArithmeticError> {
		match self {
			Curve::PolynomialFunction(params) => {
				let (low, high) = Self::calculate_low_high(op, active_issuance_pre, active_issuance_post);

				params.calculate_costs(low, high)
			}
			Curve::SquareRootBondingFunction(params) => {
				let (low, high) = Self::calculate_low_high(op, active_issuance_pre, active_issuance_post);
				params.calculate_costs(low, high)
			}
			Curve::LMSR(params) => {
				let passive_issuance_over_e = op
					.inner_value()
					.iter()
					.map(|x| params.calculate_passive_issuance(*x))
					.collect::<Result<Vec<F>, ArithmeticError>>()?;

				let passive_issuance = passive_issuance_over_e.iter().try_fold(F::from_num(0), |acc, x| {
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

pub trait BondingFunction<F: FixedSigned + PartialOrd> {
	fn calculate_costs(&self, low: F, high: F) -> Result<F, ArithmeticError>;

	fn square(x: F) -> Result<F, ArithmeticError> {
		x.checked_mul(x).ok_or(ArithmeticError::Overflow)
	}
}

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
	fn calculate_costs(&self, low: F, high: F) -> Result<F, ArithmeticError> {
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
pub struct SquareRootFunctionParameters<F> {
	pub m: F,
	pub n: F,
}

impl<F> BondingFunction<F> for SquareRootFunctionParameters<F>
where
	F: FixedSigned + PartialOrd<I9F23> + From<I9F23> + ToFixed,
{
	fn calculate_costs(&self, low: F, high: F) -> Result<F, ArithmeticError> {
		// Ensure that high and low are positive (logarithms of negative numbers are undefined)

		// Calculate sqrt(high^3) and sqrt(low^3)
		let sqrt_x3_high: F = sqrt::<F, F>(high)
			.map_err(|_| ArithmeticError::Underflow)?
			.checked_mul(high)
			.ok_or(ArithmeticError::Overflow)?;

		let sqrt_x3_low: F = sqrt::<F, F>(low)
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
pub struct LMSRFunctionParameters<F> {
	pub m: F,
}

impl<F> LMSRFunctionParameters<F>
where
	F: FixedSigned + PartialOrd<I9F23> + From<I9F23> + ToFixed,
	<F as Fixed>::Bits: Copy + ToFixed + AddAssign + BitOrAssign + ShlAssign,
{
	fn calculate_passive_issuance(&self, x: F) -> Result<F, ArithmeticError> {
		x.checked_div(self.m)
			.ok_or(ArithmeticError::DivisionByZero)
			.and_then(|x| exp::<F, F>(x).map_err(|_| ArithmeticError::Overflow))
	}
}

pub struct LMSRCalculation<F> {
	pub m: F,
	pub passive_issuance: F,
}

impl<F> BondingFunction<F> for LMSRCalculation<F>
where
	F: FixedSigned + PartialOrd<I9F23> + From<I9F23> + ToFixed,
	<F as Fixed>::Bits: Copy + ToFixed + AddAssign + BitOrAssign + ShlAssign,
{
	fn calculate_costs(&self, low: F, high: F) -> Result<F, ArithmeticError> {
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

		let e_term_numerator = exp::<F, F>(exponent_numerator)
			.map_err(|_| ArithmeticError::Overflow)
			.and_then(|x| x.checked_add(F::from_num(1)).ok_or(ArithmeticError::Overflow))?;

		let e_term_denominator = exp::<F, F>(exponent_denominator)
			.map_err(|_| ArithmeticError::Overflow)
			.and_then(|x| x.checked_add(F::from_num(1)).ok_or(ArithmeticError::Overflow))?;

		let e_term = e_term_numerator
			.checked_div(e_term_denominator)
			.ok_or(ArithmeticError::DivisionByZero)?;

		let term1 = self
			.m
			.checked_mul(ln::<F, F>(e_term).map_err(|_| ArithmeticError::Overflow)?)
			.ok_or(ArithmeticError::Underflow)?;

		let high_low_diff = high.checked_sub(low).ok_or(ArithmeticError::Underflow)?;

		high_low_diff.checked_add(term1).ok_or(ArithmeticError::Overflow)
	}
}
