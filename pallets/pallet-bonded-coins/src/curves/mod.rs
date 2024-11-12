pub(crate) mod lmsr;
pub(crate) mod polynomial;
pub(crate) mod square_root;

use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_arithmetic::ArithmeticError;
use sp_runtime::traits::CheckedConversion;
use sp_std::ops::{AddAssign, BitOrAssign, ShlAssign};
use substrate_fixed::traits::{Fixed, FixedSigned, ToFixed};

use crate::{
	curves::{
		lmsr::{LMSRParameters, LMSRParametersInput},
		polynomial::{PolynomialParameters, PolynomialParametersInput},
		square_root::{SquareRootParameters, SquareRootParametersInput},
	},
	Config, CurveParameterTypeOf, PassiveSupply, Precision,
};

#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub enum Curve<Parameter> {
	Polynomial(PolynomialParameters<Parameter>),
	SquareRoot(SquareRootParameters<Parameter>),
	Lmsr(LMSRParameters<Parameter>),
}

#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub enum CurveInput<Parameter> {
	Polynomial(PolynomialParametersInput<Parameter>),
	SquareRoot(SquareRootParametersInput<Parameter>),
	Lmsr(LMSRParametersInput<Parameter>),
}

impl<I, C> TryFrom<CurveInput<I>> for Curve<C>
where
	LMSRParameters<C>: TryFrom<LMSRParametersInput<I>>,
	PolynomialParameters<C>: TryFrom<PolynomialParametersInput<I>>,
	SquareRootParameters<C>: TryFrom<SquareRootParametersInput<I>>,
{
	type Error = ();
	fn try_from(value: CurveInput<I>) -> Result<Self, Self::Error> {
		match value {
			CurveInput::Lmsr(params) => {
				let checked_param = LMSRParameters::<C>::try_from(params).map_err(|_| ())?;
				Ok(Curve::Lmsr(checked_param))
			}
			CurveInput::Polynomial(params) => {
				let checked_param = PolynomialParameters::<C>::try_from(params).map_err(|_| ())?;
				Ok(Curve::Polynomial(checked_param))
			}
			CurveInput::SquareRoot(params) => {
				let checked_param = SquareRootParameters::<C>::try_from(params).map_err(|_| ())?;
				Ok(Curve::SquareRoot(checked_param))
			}
		}
	}
}

impl<Parameter> Curve<Parameter>
where
	Parameter: FixedSigned + PartialOrd<Precision> + From<Precision> + ToFixed,
	<Parameter as Fixed>::Bits: Copy + ToFixed + AddAssign + BitOrAssign + ShlAssign,
{
	fn as_inner(&self) -> &dyn BondingFunction<Parameter> {
		match self {
			Curve::Polynomial(params) => params,
			Curve::SquareRoot(params) => params,
			Curve::Lmsr(params) => params,
		}
	}
}

impl<Parameter> BondingFunction<Parameter> for Curve<Parameter>
where
	Parameter: FixedSigned + PartialOrd<Precision> + From<Precision> + ToFixed,
	<Parameter as Fixed>::Bits: Copy + ToFixed + AddAssign + BitOrAssign + ShlAssign,
{
	fn calculate_costs(
		&self,
		low: Parameter,
		high: Parameter,
		passive_supply: PassiveSupply<Parameter>,
	) -> Result<Parameter, ArithmeticError> {
		self.as_inner().calculate_costs(low, high, passive_supply)
	}
}

pub trait BondingFunction<Balance> {
	fn calculate_costs(
		&self,
		low: Balance,
		high: Balance,
		passive_supply: PassiveSupply<Balance>,
	) -> Result<Balance, ArithmeticError>;
}

fn square<FixedType: Fixed>(x: FixedType) -> Result<FixedType, ArithmeticError> {
	x.checked_mul(x).ok_or(ArithmeticError::Overflow)
}

fn calculate_accumulated_passive_issuance<Balance: Fixed>(passive_issuance: &[Balance]) -> Balance {
	passive_issuance
		.iter()
		.fold(Balance::from_num(0), |sum, x| sum.saturating_add(*x))
}

pub(crate) fn convert_to_fixed<T: Config>(x: u128, denomination: u8) -> Result<CurveParameterTypeOf<T>, ArithmeticError>
where
	<CurveParameterTypeOf<T> as Fixed>::Bits: TryFrom<u128>,
{
	let decimals = 10u128
		.checked_pow(u32::from(denomination))
		.ok_or(ArithmeticError::Overflow)?;

	// Scale down x by the denomination using integer division, truncating any
	// fractional parts of the result for now. Will overflow if down-scaling is not
	// sufficient to bring x to a range representable in the fixed point format.
	// This can happen if capacity(u128) / 10^denomination > capacity(fixed).
	let mut scaled_x: CurveParameterTypeOf<T> = x
		.checked_div(decimals)
		.ok_or(ArithmeticError::DivisionByZero)?
		.checked_to_fixed()
		.ok_or(ArithmeticError::Overflow)?;

	// Next we handle the remainder of the division
	let remainder = x.checked_rem_euclid(decimals).ok_or(ArithmeticError::DivisionByZero)?;

	// If the remainder is not 0, convert to fixed, scale it down and add the
	// resulting fractional part to the previous result
	if remainder > 0 {
		// Convert remainder to fixed-point and scale it down by `decimals`,
		let fractional = remainder
			// This can overflow if 10^denomination > capacity(fixed)
			.checked_to_fixed::<CurveParameterTypeOf<T>>()
			.ok_or(ArithmeticError::Overflow)?
			// This would overflow if 10^denomination exceeds the capacity of the _underlying integer_ of the fixed
			// (e.g., i128 for an I75F53). However, there is no denomination that is representable in a u128 but not in
			// an i128.
			.checked_div_int(decimals.checked_into().ok_or(ArithmeticError::Overflow)?)
			.ok_or(ArithmeticError::DivisionByZero)?;

		// Combine both parts
		scaled_x = scaled_x
			// Overflow is theoretically impossible as we are adding a number < 1 to a fixed point where the fractional
			// part is 0.
			.checked_add(fractional)
			.ok_or(ArithmeticError::Overflow)?;
	};

	Ok(scaled_x)
}
