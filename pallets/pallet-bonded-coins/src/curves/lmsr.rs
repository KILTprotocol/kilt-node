/// Implementation of the [Logarithmic Market Scoring Rule](https://mason.gmu.edu/~rhanson/mktscore.pdf) (LMSR) bonding curve.
/// The cost of purchasing an amount of assets from the market is determined using the liquidity parameter of the
/// LMSR model and the current supply of assets in the market.
/// The LMSR bonding curve is defined by the following equation:
///
/// C(s) = m * ln(Σ(e^(s_i/m))),
///
/// where:
/// - `s` is the supply of assets,
/// - `m` is the liquidity parameter of the LMSR model,
/// - `s_i` is the current supply of the assets.
///
/// `C(s)` represents the accumulated cost of purchasing/selling assets up to the current supply `s`.
/// To calculate the incremental cost of purchasing the assets, use the formula:
/// `C(s) - C(s*)`, where `s*` is the supply of assets in the market before the purchase.
///
/// Optimization
/// For numerical stability, the LMSR bonding curve employs the [log-sum-exp trick](https://en.wikipedia.org/wiki/LogSumExp) to calculate the cost.
use frame_support::ensure;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_arithmetic::ArithmeticError;
use sp_std::ops::{AddAssign, BitOrAssign, ShlAssign};
use substrate_fixed::{
	traits::{Fixed, FixedSigned, FixedUnsigned, ToFixed},
	transcendental::{exp, ln},
};

use super::BondingFunction;
use crate::{PassiveSupply, Precision, LOG_TARGET};

/// A struct representing the unchecked input parameters for the LMSR model. This struct is used to convert
/// the input parameters to the correct fixed-point type.
///
/// # Fields
/// - `m`: The liquidity parameter for the LMSR model.
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct LMSRParametersInput<Parameter> {
	pub m: Parameter,
}

/// A struct representing the validated parameters for the LMSR model. This struct is used to store the
/// parameters for the LMSR model and to perform calculations using the LMSR model.
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct LMSRParameters<Parameter> {
	///The liquidity parameter for the LMSR model. This value must be greater than zero and unsigned.
	pub m: Parameter,
}

/// Implementation of the TryFrom trait for `LMSRParametersInput` to convert the input parameters to
/// the correct fixed-point type. The TryFrom implementation for `LMSRParameters` will fail if the
/// conversion to the fixed-point type fails or if the liquidity parameter is less than or equal to
/// zero.
impl<I: FixedUnsigned, C: FixedSigned> TryFrom<LMSRParametersInput<I>> for LMSRParameters<C> {
	type Error = ();
	fn try_from(value: LMSRParametersInput<I>) -> Result<Self, Self::Error> {
		let m = C::checked_from_fixed(value.m).ok_or(())?;
		ensure!(m > C::from_num(0), ());
		Ok(LMSRParameters { m })
	}
}

impl<Parameter> LMSRParameters<Parameter>
where
	Parameter: FixedSigned + PartialOrd<Precision> + From<Precision> + ToFixed,
	<Parameter as Fixed>::Bits: Copy + ToFixed + AddAssign + BitOrAssign + ShlAssign,
{
	/// Calculate the logarithmic sum of the exponentials of the supply values,
	/// using the log-sum-exp trick.
	fn lse(&self, supply: &[Parameter]) -> Result<Parameter, ArithmeticError> {
		// Find the maximum value in the supply for numerical stability
		let max = supply.iter().max().ok_or_else(|| {
			log::error!(target: LOG_TARGET, "Supply is empty. Found pool with no currencies.");
			ArithmeticError::DivisionByZero
		})?;

		// Compute the sum of the exponent terms, adjusted by max for stability
		let e_term_sum = supply.iter().try_fold(Parameter::from_num(0), |acc, x| {
			let exponent = x
				.checked_sub(*max)
				.ok_or(ArithmeticError::Underflow)?
				.checked_div(self.m)
				.ok_or(ArithmeticError::DivisionByZero)?;

			let exp_result = exp::<Parameter, Parameter>(exponent).map_err(|_| ArithmeticError::Overflow)?;
			acc.checked_add(exp_result).ok_or(ArithmeticError::Overflow)
		})?;

		// Compute the logarithm of the sum and scale it by `m`, then add the max term
		ln::<Parameter, Parameter>(e_term_sum)
			.map_err(|_| ArithmeticError::Underflow)
			.and_then(|log_sum| log_sum.checked_mul(self.m).ok_or(ArithmeticError::Overflow))
			.and_then(|scaled_log| scaled_log.checked_add(*max).ok_or(ArithmeticError::Overflow))
	}
}

impl<Parameter> BondingFunction<Parameter> for LMSRParameters<Parameter>
where
	Parameter: FixedSigned + PartialOrd<Precision> + From<Precision> + ToFixed,
	<Parameter as Fixed>::Bits: Copy + ToFixed + AddAssign + BitOrAssign + ShlAssign,
{
	/// Calculate the cost of purchasing a set of assets from the market using the LMSR model.
	fn calculate_costs(
		&self,
		low: Parameter,
		high: Parameter,
		passive_supply: PassiveSupply<Parameter>,
	) -> Result<Parameter, ArithmeticError> {
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
