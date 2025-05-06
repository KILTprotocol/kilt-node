// KILT Blockchain – <https://kilt.io>
// Copyright (C) 2025, KILT Foundation

// The KILT Blockchain is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The KILT Blockchain is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

// If you feel like getting in touch with us, you can do so at <hello@kilt.org>

///  Curve Module
///
/// This module defines various curve types and their associated parameters used
/// in the system. It includes the following curve types:
/// - Polynomial
/// - SquareRoot
/// - LMSR (Logarithmic Market Scoring Rule)
pub mod lmsr;
pub mod polynomial;
pub mod square_root;

use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_arithmetic::ArithmeticError;
use sp_core::U256;
use sp_runtime::traits::CheckedConversion;
use sp_std::ops::{AddAssign, BitOrAssign, ShlAssign, ShrAssign};
use substrate_fixed::traits::{Fixed, FixedSigned, ToFixed};

use crate::{
	curves::{
		lmsr::{LMSRParameters, LMSRParametersInput},
		polynomial::{PolynomialParameters, PolynomialParametersInput},
		square_root::{SquareRootParameters, SquareRootParametersInput},
	},
	types::Round,
	PassiveSupply, Precision,
};

/// An enum representing different types of curves with their respective
/// parameters. Used to store curve parameters and perform calculations.
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub enum Curve<Coefficient> {
	Polynomial(PolynomialParameters<Coefficient>),
	SquareRoot(SquareRootParameters<Coefficient>),
	Lmsr(LMSRParameters<Coefficient>),
}

/// An enum representing input parameters for different types of curves.
/// Used to convert into Curve.
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub enum CurveInput<Coefficient> {
	Polynomial(PolynomialParametersInput<Coefficient>),
	SquareRoot(SquareRootParametersInput<Coefficient>),
	Lmsr(LMSRParametersInput<Coefficient>),
}

/// Implementation of the TryFrom trait for `CurveInput` to convert the input
/// parameters to the correct fixed-point type. The TryFrom implementation for
/// `Curve` will fail if the conversion to the fixed-point type fails.
/// The conversion is done by converting the input parameters to the correct
/// fixed-point type using the TryFrom implementation for the respective
/// parameters type.
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

impl<Coefficient> Curve<Coefficient>
where
	Coefficient: FixedSigned + PartialOrd<Precision> + From<Precision>,
	<Coefficient as Fixed>::Bits: Copy + ToFixed + AddAssign + BitOrAssign + ShlAssign,
{
	fn as_inner(&self) -> &dyn BondingFunction<Coefficient> {
		match self {
			Curve::Polynomial(params) => params,
			Curve::SquareRoot(params) => params,
			Curve::Lmsr(params) => params,
		}
	}
}

/// Implementation of the `BondingFunction` trait for `Curve`.
/// The `BondingFunction` trait is used to calculate the cost of purchasing or
/// selling assets using the curve.
///
/// The implementation forwards the call to the inner bonding function.
impl<Coefficient> BondingFunction<Coefficient> for Curve<Coefficient>
where
	Coefficient: FixedSigned + PartialOrd<Precision> + From<Precision>,
	<Coefficient as Fixed>::Bits: Copy + ToFixed + AddAssign + BitOrAssign + ShlAssign,
{
	fn calculate_costs(
		&self,
		low: Coefficient,
		high: Coefficient,
		passive_supply: PassiveSupply<Coefficient>,
	) -> Result<Coefficient, ArithmeticError> {
		self.as_inner().calculate_costs(low, high, passive_supply)
	}
}

/// Trait defining the bonding function for a curve.
/// The bonding function is used to calculate the cost of purchasing or selling
/// assets using the curve. The trait is implemented for each curve type.
///
/// Parameters:
/// - `low`: The lower bound of integral.
/// - `high`: The upper bound of integral.
/// - `passive_supply`: The passive supply of other assets in the pool, which
///   are not affected by the operation.
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
		.fold(Balance::from_num(0u8), |sum, x| sum.saturating_add(*x))
}

/// Converts an integer balance type to a fixed type by scaling the balance down
/// by its denomination.
pub fn balance_to_fixed<Balance, FixedType: Fixed>(
	balance: Balance,
	denomination: u8,
	round_kind: Round,
) -> Result<FixedType, ArithmeticError>
where
	FixedType::Bits: TryFrom<U256>, // TODO: make large integer type configurable in runtime
	Balance: TryInto<U256>,
{
	let decimals = U256::from(10u8)
		.checked_pow(denomination.into())
		.ok_or(ArithmeticError::Overflow)?;
	// Convert to U256 so we have enough bits to perform lossless scaling.
	let mut x_u256 = balance.checked_into().ok_or(ArithmeticError::Overflow)?;
	// Shift left to produce the representation that our fixed type would have (but
	// with extra integer bits that would potentially not fit in the fixed type).
	// This function can panic in theory, but only if frac_nbits() would be larger
	// than 256 - and no Fixed of that size exists.
	x_u256.shl_assign(FixedType::frac_nbits());

	// adding the scaling factor (decimal) - 1 ensures the result of the division
	// below is rounded up
	if round_kind == Round::Up {
		x_u256 = x_u256
			.checked_add(decimals.saturating_sub(1u8.into()))
			.ok_or(ArithmeticError::Overflow)?;
	}

	// Perform division. Due to the shift the precision/truncation is identical to
	// division on the fixed type.
	x_u256 = x_u256.checked_div(decimals).ok_or(ArithmeticError::DivisionByZero)?;
	// Try conversion to integer type underlying the fixed type (e.g., i128 for a
	// I75F53). If this overflows, there is nothing we can do; even the scaled value
	// does not fit in the fixed type.
	let truncated = x_u256.checked_into().ok_or(ArithmeticError::Overflow)?;
	// Cast the integer as a fixed. We can do this because we've already applied the
	// correct bit shift above.
	let fixed = FixedType::from_bits(truncated);
	// Return the result of scaled conversion to fixed.
	Ok(fixed)
}

/// Converts a fixed type representation of a balance back to an integer type
/// via scaling up by its denomination.
pub fn fixed_to_balance<Balance, FixedType: Fixed>(
	fixed: FixedType,
	denomination: u8,
	round_kind: Round,
) -> Result<Balance, ArithmeticError>
where
	FixedType::Bits: TryInto<U256>,
	Balance: TryFrom<U256>,
{
	// Convert to U256 so we have enough bits to perform lossless scaling.
	let mut value_u256: U256 = fixed.to_bits().try_into().map_err(|_| ArithmeticError::Overflow)?;

	let decimals = U256::from(10u8)
		.checked_pow(denomination.into())
		.ok_or(ArithmeticError::Overflow)?;

	// calculate the actual value by multiplying with the denomination. By using th
	// U256 type we can ensure that the multiplication does not overflow.
	value_u256 = value_u256.checked_mul(decimals).ok_or(ArithmeticError::Overflow)?;

	// Calculate the number of trailing zeros in the value.
	let trailing_zeros = value_u256.trailing_zeros();

	// Calculate the number of fractional bits in the fixed-point type.
	let frac_bits: u32 = FixedType::frac_nbits();

	// Shift the value to the right by the number of fractional bits.
	value_u256.shr_assign(frac_bits);

	// If the number of trailing zeros is less than the number of fractional bits,
	// the value is not rounded and we can return it directly.
	if round_kind == Round::Up && trailing_zeros < frac_bits {
		value_u256 = value_u256
			.checked_add(U256::from(1u8))
			.ok_or(ArithmeticError::Overflow)?;
	}

	// Convert the value back to the collateral representation
	value_u256.try_into().map_err(|_| ArithmeticError::Overflow)
}
