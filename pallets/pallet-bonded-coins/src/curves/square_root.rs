/// Square Root Bonding Curve Implementation.
///
/// This module provides an implementation of a square root bonding curve, with the integral precomputed for efficiency.
///
/// ### Cost Function
/// The cost function is defined as:
/// ```text
/// c(s) = m * sqrt(s) + n
/// ```
/// This function, `c(s)`, determines the price for purchasing or selling assets at any supply point `s`.
/// The total transaction cost is calculated as the integral of `c(s)` between the start point and `s`.
///
/// ### Antiderivative
/// The indefinite integral of the cost function is:
/// ```text
/// C(s) = (2/3) * m * s^(3/2) + n * s
/// ```
/// Where:
/// - `s` is the supply of assets,
/// - `m` is the coefficient for the square root term,
/// - `n` is the coefficient for the linear term.
///
/// `C(s)` represents the total cost of purchasing or selling assets up to the current supply `s`.
/// To calculate the incremental cost of a transaction, use the formula:
/// ```text
/// Incremental Cost = C(s) - C(s*)
/// ```
/// Here, `s*` represents the initial supply before the transaction, and `s` is the supply after the transaction.
///
/// ### Optimization for Numerical Stability
/// Calculating `s^(3/2)` directly can lead to overflow in fixed-point arithmetic. To mitigate this, the calculation is factored as:
/// ```text
/// sqrt(s^3) = sqrt(s) * s
/// ```
/// By expressing `s^(3/2)` as the product of `sqrt(s)` and `s`, we reduce the risk of overflow while maintaining computational precision.
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_arithmetic::ArithmeticError;
use substrate_fixed::{
	traits::{FixedSigned, FixedUnsigned, ToFixed},
	transcendental::sqrt,
};

use super::{calculate_accumulated_passive_issuance, BondingFunction};
use crate::{PassiveSupply, Precision};

/// A struct representing the unchecked input parameters for a square root bonding curve.
/// This struct is used to convert the input parameters to the correct fixed-point type.
///
/// The input struct assumes that the coefficients are precomputed according to the integral rules of the square root function./// ### Example
///
/// For a square root cost function `c(s) = 3 * s^1/2 + 2
///
/// which is resulting into the antiderivative `C(s) = (6 / 3) * s^(1/2) + 2 * s`
/// the input parameters would be:
/// ```rust, ignore
/// SquareRootParametersInput {
///    m: 2,
///    n: 2,
/// }
/// ```
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct SquareRootParametersInput<Parameter> {
	/// Coefficient for the square root part.
	pub m: Parameter,
	/// Coefficient for the linear part.
	pub n: Parameter,
}

/// A struct representing the validated parameters for a square root bonding curve.
/// This struct is used to store the parameters for a square root bonding curve and to perform calculations using the square root bonding curve.
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct SquareRootParameters<Parameter> {
	/// Coefficient for the square root part.
	pub m: Parameter,
	/// Coefficient for the linear part.
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
	/// Calculate the cost of purchasing/selling assets using the square root bonding curve.
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
