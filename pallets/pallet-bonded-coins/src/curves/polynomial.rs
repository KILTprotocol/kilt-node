// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2024 BOTLabs GmbH

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

// If you feel like getting in touch with us, you can do so at info@botlabs.org

/// Polynomial Bonding Curve Implementation.
///
/// This module provides an implementation of a polynomial bonding curve.
/// The current implementation supports a polynomial function of order 2, with
/// the integral precomputed for efficiency.
///
/// ### Cost Function
/// The cost function is defined as:
/// ```text
/// c(s) = m * s^2 + n * s + o
/// ```
/// This function, `c(s)`, determines the price for purchasing or selling assets
/// at any supply point `s`. The total cost of transactions is computed as the
/// integral of `c(s)` between the start point and `s`.
///
/// ### Antiderivative
/// The indefinite integral of the cost function is:
/// ```text
/// C(s) = (m / 3) * s^3 + (n / 2) * s^2 + o * s
/// ```
/// Where:
/// - `m` is the coefficient for the quadratic term,
/// - `n` is the coefficient for the linear term,
/// - `o` is the constant term.
///
///
/// `C(s)` represents the accumulated cost of purchasing or selling assets up to
/// the current supply `s`. The integral between two supply points, `s*`
/// (initial supply) and `s` (current supply), gives the incremental cost:
/// ```text
/// Incremental Cost = C(s) - C(s*)
/// ```
/// This captures the total cost for changing the supply from `s*` to `s`.
///
/// ### Optimization for Numerical Stability
/// The computation of `s^3` can cause overflow in fixed-point arithmetic. To
/// mitigate this, the calculation is factored as:
/// ```text
/// x^3 - y^3 = (x^2 + x * y + y^2) * (x - y)
/// ```
/// Where:
/// - `x` is the upper limit of the integral,
/// - `y` is the lower limit of the integral.
///
/// By breaking down the computation in this way, we reduce the risk of overflow
/// while maintaining precision.
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_arithmetic::ArithmeticError;
use substrate_fixed::traits::{FixedSigned, FixedUnsigned};

use super::{calculate_accumulated_passive_issuance, square, BondingFunction};
use crate::PassiveSupply;

/// A struct representing the unchecked input parameters for a polynomial
/// bonding curve. This struct is used to convert the input parameters to the
/// correct fixed-point type.
///
/// The input struct assumes that the coefficients are precomputed according to
/// the integral rules of the polynomial function.
///
/// ### Example
///
/// For a polynomial cost function `c(s) = 3 * s^2 + 2 * s + 2`
///
/// which is resulting into the antiderivative
/// `C(s) = (3 / 3) * s^3 + (2 / 2) * s^2 + 2 * s`
/// the input parameters would be:
/// ```rust, ignore
/// PolynomialParametersInput {
///    m: 1,
///    n: 1,
///    o: 2,
/// }
/// ```
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct PolynomialParametersInput<Coefficient> {
	/// Coefficient for the cubic part.
	pub m: Coefficient,
	/// Coefficient for the quadratic part.
	pub n: Coefficient,
	/// Coefficient for the linear part.
	pub o: Coefficient,
}

/// A struct representing the validated parameters for a polynomial bonding
/// curve. This struct is used to store the parameters for a polynomial bonding
/// curve and to perform calculations using the polynomial bonding curve.
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct PolynomialParameters<Coefficient> {
	/// Coefficient for the cubic part.
	pub m: Coefficient,
	/// Coefficient for the quadratic part.
	pub n: Coefficient,
	/// Coefficient for the linear part.
	pub o: Coefficient,
}

/// Implementation of the TryFrom trait for `PolynomialParametersInput` to
/// convert the input parameters to the correct fixed-point type. The TryFrom
/// implementation for `PolynomialParameters` will fail if the conversion to the
/// fixed-point type fails.
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

impl<Coefficient> BondingFunction<Coefficient> for PolynomialParameters<Coefficient>
where
	Coefficient: FixedSigned,
{
	/// Calculate the cost of purchasing/selling assets using the polynomial
	/// bonding curve.
	fn calculate_costs(
		&self,
		low_without_passive: Coefficient,
		high_without_passive: Coefficient,
		passive_supply: PassiveSupply<Coefficient>,
	) -> Result<Coefficient, ArithmeticError> {
		let accumulated_passive_issuance = calculate_accumulated_passive_issuance(&passive_supply);

		// reassign high and low to include the accumulated passive issuance
		let high = high_without_passive
			.checked_add(accumulated_passive_issuance)
			.ok_or(ArithmeticError::Overflow)?;

		let low = low_without_passive
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
