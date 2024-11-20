/// Square Root bonding curve implementation.
///
/// This module provides an implementation of a square root bonding curve. The current implementation supports a bonding curve where the integral is precomputed.
/// The cost function for the square root bonding curve is defined as:
/// C(s) = m * sqrt(s^3) + n * s,
/// where:
/// - `s` is the supply of assets,
/// - `m` is the coefficient for the square root part,
/// - `n` is the coefficient for the linear part.
/// `C(s)` represents the accumulated cost of purchasing assets up to the current supply `s`.
///
/// To calculate the incremental cost of purchasing the assets, use the formula:
/// `C(s) - C(s*)`, where `s*` is the supply of assets in the market before the purchase.
///
/// The module includes the following components:
///
/// - `SquareRootParametersInput`: A struct representing the input parameters for a square root bonding curve.
/// - `SquareRootParameters`: A struct representing the parameters for a square root bonding curve, used to perform calculations and stored in storage.
/// - `TryFrom<SquareRootParametersInput<I>> for SquareRootParameters<C>`: An implementation to convert input parameters to the correct fixed-point type.
/// - `BondingFunction<Parameter> for SquareRootParameters<Parameter>`: An implementation of the bonding function to calculate costs.
///
/// Optimization
/// The calculation of sqrt(s^3) can quickly overflow the fixed-point type. To avoid this, the calculation is factored into:
/// sqrt(s^3) = sqrt(s) * s,
/// where:
/// - `s` is the supply of assets.
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_arithmetic::ArithmeticError;
use substrate_fixed::{
	traits::{FixedSigned, FixedUnsigned, ToFixed},
	transcendental::sqrt,
};

use super::{calculate_accumulated_passive_issuance, BondingFunction};
use crate::{PassiveSupply, Precision};

/// A struct representing the input parameters for a square root bonding curve.
/// This struct is used to convert the input parameters to the correct fixed-point type.
///
/// # Fields
/// - `m`: Coefficient for the square root part.
/// - `n`: Coefficient for the linear part.
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct SquareRootParametersInput<Parameter> {
	pub m: Parameter,
	pub n: Parameter,
}

/// A struct representing the parameters for a square root bonding curve.
/// This struct is used to store the parameters for a square root bonding curve and to perform calculations using the square root bonding curve.
///
/// # Fields
/// - `m`: Coefficient for the square root part.
/// - `n`: Coefficient for the linear part.
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct SquareRootParameters<Parameter> {
	pub m: Parameter,
	pub n: Parameter,
}

/// Implementation of the TryFrom trait for `SquareRootParametersInput` to convert the input parameters to the correct fixed-point type.
/// The TryFrom implementation for `SquareRootParameters` will fail if the conversion to the fixed-point type fails.
impl<I: FixedUnsigned, C: FixedSigned> TryFrom<SquareRootParametersInput<I>> for SquareRootParameters<C> {
	type Error = ();
	fn try_from(value: SquareRootParametersInput<I>) -> Result<Self, Self::Error> {
		Ok(SquareRootParameters {
			m: C::checked_from_fixed(value.m).ok_or(())?,
			n: C::checked_from_fixed(value.n).ok_or(())?,
		})
	}
}

impl<Parameter> BondingFunction<Parameter> for SquareRootParameters<Parameter>
where
	Parameter: FixedSigned + PartialOrd<Precision> + From<Precision> + ToFixed,
{
	/// Calculate the cost of purchasing assets using the square root bonding curve.
	fn calculate_costs(
		&self,
		low: Parameter,
		high: Parameter,
		passive_supply: PassiveSupply<Parameter>,
	) -> Result<Parameter, ArithmeticError> {
		let accumulated_passive_issuance = calculate_accumulated_passive_issuance(&passive_supply);

		// reassign high and low to include the accumulated passive issuance
		let high = high
			.checked_add(accumulated_passive_issuance)
			.ok_or(ArithmeticError::Overflow)?;

		let low = low
			.checked_add(accumulated_passive_issuance)
			.ok_or(ArithmeticError::Overflow)?;

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
