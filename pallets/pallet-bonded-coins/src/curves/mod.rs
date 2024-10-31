pub(crate) mod lmsr;
pub(crate) mod polynomial;
pub(crate) mod square_root;

use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_arithmetic::ArithmeticError;
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
	LMSR(LMSRParameters<Parameter>),
}

#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub enum CurveInput<Parameter> {
	Polynomial(PolynomialParametersInput<Parameter>),
	SquareRoot(SquareRootParametersInput<Parameter>),
	LMSR(LMSRParametersInput<Parameter>),
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
			CurveInput::LMSR(params) => {
				let checked_param = LMSRParameters::<C>::try_from(params).map_err(|_| ())?;
				Ok(Curve::LMSR(checked_param))
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
		match self {
			Curve::Polynomial(params) => params.calculate_costs(low, high, passive_supply),
			Curve::SquareRoot(params) => params.calculate_costs(low, high, passive_supply),
			Curve::LMSR(params) => params.calculate_costs(low, high, passive_supply),
		}
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

pub(crate) fn convert_to_fixed<T: Config>(
	x: u128,
	denomination: u8,
) -> Result<CurveParameterTypeOf<T>, ArithmeticError> {
	let decimals = 10u128
		.checked_pow(u32::from(denomination))
		.ok_or(ArithmeticError::Overflow)?;
	let scaled_x = x.checked_div(decimals).ok_or(ArithmeticError::DivisionByZero)?;
	scaled_x.checked_to_fixed().ok_or(ArithmeticError::Overflow)
}
