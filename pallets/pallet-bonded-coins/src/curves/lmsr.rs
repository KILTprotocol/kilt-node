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

/// Implementation of the [Logarithmic Market Scoring Rule (LMSR)](https://mason.gmu.edu/~rhanson/mktscore.pdf) bonding curve.
///
/// This module provides an LMSR bonding curve implementation, which determines
/// the cost of purchasing assets based on the liquidity parameter and the
/// current supply of assets in the market.
///
/// ### Cost Function
/// The LMSR bonding curve is defined by the equation:
/// ```text
/// C(s) = m * ln(Σ(e^(s_i / m)))
/// ```
/// Where:
/// - `s` is the supply of all assets, represented as a vector,
/// - `m` is the liquidity parameter of the LMSR model,
/// - `s_i` is the supply of a single asset.
///
/// `C(s)` represents the accumulated cost of purchasing or selling assets up to
/// the current supply `s`.
///
/// ### Incremental Cost
/// To calculate the incremental cost of a transaction, use the formula:
/// ```text
/// Incremental Cost = C(s) - C(s*)
/// ```
/// Here:
/// - `s*` is the supply of assets in the market before the transaction, and
/// - `s` is the supply of assets after the transaction.
///
/// ### Optimization for Numerical Stability
/// To ensure numerical stability and prevent overflow/underflow during
/// calculations, this implementation uses the [log-sum-exp trick](https://en.wikipedia.org/wiki/LogSumExp).
/// This technique improves precision when handling the summation and
/// exponential terms in the cost function.
use frame_support::ensure;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_arithmetic::ArithmeticError;
use sp_std::ops::{AddAssign, BitOrAssign, ShlAssign};
use substrate_fixed::{
	traits::{Fixed, FixedSigned, FixedUnsigned, ToFixed},
	transcendental::{exp, ln},
};

use crate::{curves::BondingFunction, PassiveSupply, Precision, LOG_TARGET};

/// A struct representing the unchecked input parameters for the LMSR model.
/// This struct is used to convert the input parameters to the correct
/// fixed-point type.
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct LMSRParametersInput<Coefficient> {
	/// The liquidity parameter for the LMSR model
	pub m: Coefficient,
}

/// A struct representing the validated parameters for the LMSR model. This
/// struct is used to store the parameters for the LMSR model and to perform
/// calculations using the LMSR model.
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct LMSRParameters<Coefficient> {
	///The liquidity parameter for the LMSR model. This value must be greater
	/// than zero and unsigned.
	pub m: Coefficient,
}

/// Implementation of the TryFrom trait for `LMSRParametersInput` to convert the
/// input parameters to the correct fixed-point type. The TryFrom implementation
/// for `LMSRParameters` will fail if the conversion to the fixed-point type
/// fails or if the liquidity parameter is less than or equal to zero.
impl<I: FixedUnsigned, C: FixedSigned> TryFrom<LMSRParametersInput<I>> for LMSRParameters<C> {
	type Error = ();
	fn try_from(value: LMSRParametersInput<I>) -> Result<Self, Self::Error> {
		let m = C::checked_from_fixed(value.m).ok_or(())?;
		ensure!(m > C::from_num(0u8), ());
		Ok(LMSRParameters { m })
	}
}

impl<Coefficient> LMSRParameters<Coefficient>
where
	Coefficient: FixedSigned + PartialOrd<Precision> + From<Precision>,
	<Coefficient as Fixed>::Bits: Copy + ToFixed + AddAssign + BitOrAssign + ShlAssign,
{
	/// Calculate the logarithmic sum of the exponentials of the supply values,
	/// using the log-sum-exp trick.
	fn lse(&self, supply: &[Coefficient]) -> Result<Coefficient, ArithmeticError> {
		// Find the maximum value in the supply for numerical stability
		let max = supply.iter().max().ok_or_else(|| {
			log::error!(target: LOG_TARGET, "Supply is empty. Found pool with no currencies.");
			ArithmeticError::DivisionByZero
		})?;

		// Compute the sum of the exponent terms, adjusted by max for stability
		let e_term_sum = supply.iter().try_fold(Coefficient::from_num(0u8), |acc, x| {
			let exponent = x
				.checked_sub(*max)
				.ok_or(ArithmeticError::Underflow)?
				.checked_div(self.m)
				.ok_or(ArithmeticError::DivisionByZero)?;

			let exp_result = exp::<Coefficient, Coefficient>(exponent).map_err(|_| ArithmeticError::Overflow)?;
			acc.checked_add(exp_result).ok_or(ArithmeticError::Overflow)
		})?;

		// Compute the logarithm of the sum and scale it by `m`, then add the max term
		ln::<Coefficient, Coefficient>(e_term_sum)
			.map_err(|_| ArithmeticError::Underflow)
			.and_then(|log_sum| log_sum.checked_mul(self.m).ok_or(ArithmeticError::Overflow))
			.and_then(|scaled_log| scaled_log.checked_add(*max).ok_or(ArithmeticError::Overflow))
	}
}

impl<Coefficient> BondingFunction<Coefficient> for LMSRParameters<Coefficient>
where
	Coefficient: FixedSigned + PartialOrd<Precision> + From<Precision>,
	<Coefficient as Fixed>::Bits: Copy + ToFixed + AddAssign + BitOrAssign + ShlAssign,
{
	/// Calculate the cost of purchasing a set of assets from the market using
	/// the LMSR model.
	fn calculate_costs(
		&self,
		low: Coefficient,
		high: Coefficient,
		passive_supply: PassiveSupply<Coefficient>,
	) -> Result<Coefficient, ArithmeticError> {
		// Clone passive_supply and add low and high to create modified supplies
		let mut low_total_supply = passive_supply.clone();
		low_total_supply.push(low);
		let mut high_total_supply = passive_supply;
		high_total_supply.push(high);

		// Compute LSE for both modified supplies
		let lower_bound_value = self.lse(&low_total_supply)?;
		let high_bound_value = self.lse(&high_total_supply)?;

		// Return the difference between high and low LSE values
		high_bound_value
			.checked_sub(lower_bound_value)
			.ok_or(ArithmeticError::Underflow)
	}
}
