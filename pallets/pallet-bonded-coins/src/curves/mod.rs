///  Curve Module
///
/// This module defines various curve types and their associated parameters used in the system.
/// It includes the following curve types:
/// - Polynomial
/// - SquareRoot
/// - LMSR (Logarithmic Market Scoring Rule)
///
/// The module provides the following key components:
/// - `Curve`: An enum representing different types of curves with their respective parameters. Used to store curve parameters and perform calculations.
/// - `CurveInput`: An enum representing input parameters for different types of curves. Used to convert input parameters to the correct fixed-point type.
/// - `TryFrom<CurveInput<I>> for Curve<C>`: An implementation to convert `CurveInput` into `Curve`.
pub(crate) mod lmsr;
pub(crate) mod polynomial;
pub(crate) mod square_root;

use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_arithmetic::ArithmeticError;
use sp_core::U256;
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

/// An enum representing different types of curves with their respective parameters.
/// Used to store curve parameters and perform calculations.
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub enum Curve<Parameter> {
	Polynomial(PolynomialParameters<Parameter>),
	SquareRoot(SquareRootParameters<Parameter>),
	Lmsr(LMSRParameters<Parameter>),
}

/// An enum representing input parameters for different types of curves.
/// Used to convert into Curve.
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub enum CurveInput<Parameter> {
	Polynomial(PolynomialParametersInput<Parameter>),
	SquareRoot(SquareRootParametersInput<Parameter>),
	Lmsr(LMSRParametersInput<Parameter>),
}

/// Implementation of the TryFrom trait for `CurveInput` to convert the input parameters to
/// the correct fixed-point type. The TryFrom implementation for `Curve` will fail if the
/// conversion to the fixed-point type fails.
/// The conversion is done by converting the input parameters to the correct fixed-point type
/// using the TryFrom implementation for the respective parameters type.
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

/// Implementation of the `BondingFunction` trait for `Curve`.
/// The `BondingFunction` trait is used to calculate the cost of purchasing or selling assets using the curve.
///
/// The implementation forwards the call to the inner bonding function.
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

/// Trait defining the bonding function for a curve.
/// The bonding function is used to calculate the cost of purchasing or selling assets using the curve.
/// The trait is implemented for each curve type.
///
/// Variables:
/// - `low`: The lower bound of integral.
/// - `high`: The upper bound of integral.
/// - `passive_supply`: The passive supply of other assets in the pool, which are not affected by the operation.
pub trait BondingFunction<Balance> {
	fn calculate_costs(
		&self,
		low: Balance,
		high: Balance,
		passive_supply: PassiveSupply<Balance>,
	) -> Result<Balance, ArithmeticError>;
}

/// Helper function to calculate the square of a fixed-point number.
fn square<FixedType: Fixed>(x: FixedType) -> Result<FixedType, ArithmeticError> {
	x.checked_mul(x).ok_or(ArithmeticError::Overflow)
}

/// Helper function to calculate the accumulated passive issuance.
fn calculate_accumulated_passive_issuance<Balance: Fixed>(passive_issuance: &[Balance]) -> Balance {
	passive_issuance
		.iter()
		.fold(Balance::from_num(0), |sum, x| sum.saturating_add(*x))
}

/// TODO. Implementation might change.
pub(crate) fn convert_to_fixed<T: Config>(x: u128, denomination: u8) -> Result<CurveParameterTypeOf<T>, ArithmeticError>
where
	<CurveParameterTypeOf<T> as Fixed>::Bits: TryFrom<U256>, // TODO: make large integer type configurable in runtime
{
	let decimals = U256::from(10)
		.checked_pow(denomination.into())
		.ok_or(ArithmeticError::Overflow)?;
	// Convert to U256 so we have enough bits to perform lossless scaling.
	let mut x_u256 = U256::from(x);
	// Shift left to produce the representation that our fixed type would have (but
	// with extra integer bits that would potentially not fit in the fixed type).
	// This function can panic in theory, but only if frac_nbits() would be larger
	// than 256 - and no Fixed of that size exists.
	x_u256.shl_assign(CurveParameterTypeOf::<T>::frac_nbits());
	// Perform division. Due to the shift the precision/truncation is identical to
	// division on the fixed type.
	x_u256 = x_u256.checked_div(decimals).ok_or(ArithmeticError::DivisionByZero)?;
	// Try conversion to integer type underlying the fixed type (e.g., i128 for a
	// I75F53). If this overflows, there is nothing we can do; even the scaled value
	// does not fit in the fixed type.
	let truncated = x_u256.checked_into().ok_or(ArithmeticError::Overflow)?;
	// Cast the integer as a fixed. We can do this because we've already applied the
	// correct bit shift above.
	let fixed = <CurveParameterTypeOf<T> as Fixed>::from_bits(truncated);
	// Return the result of scaled conversion to fixed.
	Ok(fixed)
}
