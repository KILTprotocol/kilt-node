/// Polynomial bonding curve implementation.
///
/// This module provides an implementation of a polynomial bonding curve. The current implementation supports a cubic bonding curve, where the integral is precomputed.
/// The cost function for the polynomial bonding curve is defined as:
/// C(s) = m * s^3 + n * s^2 + o * s,
/// where:
/// - `s` is the supply of assets,
/// - `m` is the coefficient for the cubic part,
/// - `n` is the coefficient for the quadratic part,
/// - `o` is the coefficient for the linear part.
/// `C(s)` represents the accumulated cost of purchasing assets up to the current supply `s`.
///
/// To calculate the incremental cost of purchasing the assets, use the formula:
/// `C(s) - C(s*)`, where `s*` is the supply of assets in the market before the purchase.
///
/// The module includes the following components:
///
/// - `PolynomialParametersInput`: A struct representing the input parameters for a polynomial bonding curve.
/// - `PolynomialParameters`: A struct representing the parameters for a polynomial bonding curve, used to perform calculations and stored in storage.
/// - `TryFrom<PolynomialParametersInput<I>> for PolynomialParameters<C>`: An implementation to convert input parameters to the correct fixed-point type.
/// - `BondingFunction<Parameter> for PolynomialParameters<Parameter>`: An implementation of the bonding function to calculate costs.
///
/// Optimization
/// The calculation of x^3 can quickly overflow the fixed-point type. To avoid this, the calculation is factored into:
/// x^3 = (x^2 + x * y + y^2) * (x - y),
/// where:
/// - `x` is the upper bound of the integral,
/// - `y` is the lower bound of the integral.
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_arithmetic::ArithmeticError;
use substrate_fixed::traits::{FixedSigned, FixedUnsigned};

use super::{calculate_accumulated_passive_issuance, square, BondingFunction};
use crate::PassiveSupply;

/// A struct representing the input parameters for a polynomial bonding curve.
/// This struct is used to convert the input parameters to the correct fixed-point type.
///
/// # Fields
/// - `m`: Coefficient for the cubic part.
/// - `n`: Coefficient for the quadratic part.
/// - `o`: Coefficient for the linear part.
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct PolynomialParametersInput<Parameter> {
	pub m: Parameter,
	pub n: Parameter,
	pub o: Parameter,
}

/// A struct representing the parameters for a polynomial bonding curve.
/// This struct is used to store the parameters for a polynomial bonding
/// curve and to perform calculations using the polynomial bonding curve.
///
/// # Fields
/// - `m`: Coefficient for the cubic part.
/// - `n`: Coefficient for the quadratic part.
/// - `o`: Coefficient for the linear part.
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct PolynomialParameters<Parameter> {
	pub m: Parameter,
	pub n: Parameter,
	pub o: Parameter,
}

/// Implementation of the TryFrom trait for `PolynomialParametersInput` to convert the input parameters to
/// the correct fixed-point type. The TryFrom implementation for `PolynomialParameters` will fail if the
/// conversion to the fixed-point type fails.
impl<I: FixedUnsigned, C: FixedSigned> TryFrom<PolynomialParametersInput<I>> for PolynomialParameters<C> {
	type Error = ();
	fn try_from(value: PolynomialParametersInput<I>) -> Result<Self, Self::Error> {
		Ok(PolynomialParameters {
			m: C::checked_from_fixed(value.m).ok_or(())?,
			n: C::checked_from_fixed(value.n).ok_or(())?,
			o: C::checked_from_fixed(value.o).ok_or(())?,
		})
	}
}

impl<Parameter> BondingFunction<Parameter> for PolynomialParameters<Parameter>
where
	Parameter: FixedSigned,
{
	/// Calculate the cost of purchasing assets using the polynomial bonding curve.
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

		// Calculate high - low
		let delta_x = high.checked_sub(low).ok_or(ArithmeticError::Underflow)?;

		let high_low_mul = high.checked_mul(low).ok_or(ArithmeticError::Overflow)?;
		let high_square = square(high)?;
		let low_square = square(low)?;

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
