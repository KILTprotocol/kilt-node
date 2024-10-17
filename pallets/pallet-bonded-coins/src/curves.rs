use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_arithmetic::ArithmeticError;
use substrate_fixed::{
	traits::{FixedSigned, ToFixed},
	transcendental::sqrt,
	types::I9F23,
};

/// An enumeration representing different types of bonding curves.
///
/// This enum is generic over the type `F`, which represents the type of the coefficients used in the bonding functions.
///
/// # Variants
/// - `PolynomialFunction`: Represents a polynomial bonding function with parameters of type `PolynomialFunctionParameters<F>`.
/// - `SquareRootBondingFunction`: Represents a square root bonding function with parameters of type `SquareRootFunctionParameters<F>`.
///
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub enum Curve<F> {
	PolynomialFunction(PolynomialFunctionParameters<F>),
	SquareRootBondingFunction(SquareRootFunctionParameters<F>),
}

/// An enumeration representing the type of operation on the bonding curve.
///
/// # Variants
/// - `Mint`: Represents a minting operation, where new tokens are created.
/// - `Burn`: Represents a burning operation, where existing tokens are destroyed.
pub enum DiffKind {
	Mint,
	Burn,
}

impl<F> Curve<F>
where
	F: FixedSigned + PartialOrd<I9F23> + From<I9F23>,
{
	/// Calculates the cost of a bonding curve operation.
	///
	/// This method computes the cost based on the type of bonding curve
	/// and the difference in active issuance before and after the operation, adjusted by passive issuance.
	///
	/// # Parameters
	/// - `active_issuance_pre`: The active issuance before the operation.
	/// - `active_issuance_post`: The active issuance after the operation.
	/// - `passive_issuance`: The passive issuance.
	/// - `kind`: The type of operation, either `DiffKind::Burn` or `DiffKind::Mint`.
	///
	/// # Returns
	/// - `Result<F, ArithmeticError>`: The calculated cost or an arithmetic error if an overflow or underflow occurs.
	///
	/// # Errors
	/// - `ArithmeticError::Underflow`: If subtraction results in an underflow.
	/// - `ArithmeticError::Overflow`: If any arithmetic operation results in an overflow.
	pub fn calculate_cost(
		&self,
		active_issuance_pre: F,
		active_issuance_post: F,
		passive_issuance: F,
		kind: DiffKind,
	) -> Result<F, ArithmeticError> {
		let (low, high) = match kind {
			DiffKind::Burn => (
				active_issuance_post.saturating_add(passive_issuance),
				active_issuance_pre.saturating_add(passive_issuance),
			),
			DiffKind::Mint => (
				active_issuance_pre.saturating_add(passive_issuance),
				active_issuance_post.saturating_add(passive_issuance),
			),
		};

		match self {
			Curve::PolynomialFunction(params) => params.calculate_costs(low, high),
			Curve::SquareRootBondingFunction(params) => params.calculate_costs(low, high),
		}
	}
}

// BondingFunction trait
pub trait BondingFunction<F: FixedSigned + PartialOrd> {
	/// calculates the cost of the curve between low and high
	fn calculate_costs(&self, low: F, high: F) -> Result<F, ArithmeticError>;
}

/// A struct representing the parameters of a polynomial function F(x) = mx³ + nx² + ox.
///
/// This struct is generic over the type `F`, which represents the type of the coefficients.
///
/// # Attributes
/// - `m`: The coefficient for the quadratic term.
/// - `n`: The coefficient for the linear term.
/// - `o`: The constant term.
///
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct PolynomialFunctionParameters<F> {
	pub m: F,
	pub n: F,
	pub o: F,
}

impl<F> BondingFunction<F> for PolynomialFunctionParameters<F>
where
	F: FixedSigned,
{
	/// Calculates the cost for a polynomial bonding function within a given range.
	///
	/// The polynomial function is defined as `F(x) = m * x^3 + n * x^2 + o * x`.
	/// This method calculates the difference `F(high) - F(low)` using a factored form to improve performance and reduce overflow risk.
	///
	/// The calculation follows these steps:
	/// 1. Compute `delta_x = high - low`.
	/// 2. Compute intermediate terms to reduce overflow risk:
	///    - `high_plus_low = high + low`
	///    - `high_low_mul = high * low`
	///    - `high2 = high * high`
	///    - `low2 = low * low`
	/// 3. Compute the factorized cubic term: `cubic_term = high^2 + high * low + low^2`.
	/// 4. Calculate the terms:
	///    - `term1 = m * cubic_term`
	///    - `term2 = n * high_plus_low`
	/// 5. Sum the terms and multiply by `delta_x` to get the final result.
	///
	/// # Parameters
	/// - `low`: The lower bound of the range.
	/// - `high`: The upper bound of the range.
	///
	/// # Returns
	/// - `Result<F, ArithmeticError>`: The calculated cost or an arithmetic error if an overflow or underflow occurs.
	///
	/// # Errors
	/// - `ArithmeticError::Underflow`: If subtraction results in an underflow.
	/// - `ArithmeticError::Overflow`: If any arithmetic operation results in an overflow.
	fn calculate_costs(&self, low: F, high: F) -> Result<F, ArithmeticError> {
		if high == F::from_num(0) && low == F::from_num(0) {
			return Ok(F::from_num(0));
		}

		// Calculate high - low
		let delta_x = high.checked_sub(low).ok_or(ArithmeticError::Underflow)?;

		let high_low_mul = high.checked_mul(low).ok_or(ArithmeticError::Overflow)?;
		let high2 = high.checked_mul(high).ok_or(ArithmeticError::Overflow)?;
		let low2 = low.checked_mul(low).ok_or(ArithmeticError::Overflow)?;

		// Factorized cubic term:  (high^2 + high * low + low^2)
		let cubic_term = high2
			.checked_add(high_low_mul)
			.ok_or(ArithmeticError::Overflow)?
			.checked_add(low2)
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
			.checked_add(self.o) // Add constant term o
			.ok_or(ArithmeticError::Overflow)?;

		result
			.checked_mul(delta_x) // Multiply by (high - low)
			.ok_or(ArithmeticError::Overflow)
	}
}

/// A struct representing the parameters of a square root bonding function.
///
/// This struct is generic over the type `F`, which represents the type of the coefficients.
///
/// # Attributes
/// - `m`: The coefficient for the square root term.
/// - `n`: The constant term.
///
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct SquareRootFunctionParameters<F> {
	pub m: F,
	pub n: F,
}

impl<F> BondingFunction<F> for SquareRootFunctionParameters<F>
where
	F: FixedSigned + PartialOrd<I9F23> + From<I9F23> + ToFixed,
{
	/// Calculates the cost for a square root bonding function within a given range.
	///
	/// The square root bonding function is defined as `F(x) = m * sqrt(x^3) + n * x`.
	/// This method calculates the difference `F(high) - F(low)` using the square root and linear terms.
	///
	/// # Parameters
	/// - `low`: The lower bound of the range.
	/// - `high`: The upper bound of the range.
	///
	/// # Returns
	/// - `Result<F, ArithmeticError>`: The calculated cost or an arithmetic error if an overflow or underflow occurs.
	///
	/// # Errors
	/// - `ArithmeticError::Underflow`: If subtraction or square root results in an underflow.
	/// - `ArithmeticError::Overflow`: If any arithmetic operation results in an overflow.
	fn calculate_costs(&self, low: F, high: F) -> Result<F, ArithmeticError> {
		if low == high {
			return Ok(F::from_num(0));
		}

		// Ensure that high and low are positive (logarithms of negative numbers are undefined)

		// Calculate sqrt(high^3) and sqrt(low^3)
		let sqrt_x3_high: F = {
			if high == F::from_num(0) {
				F::from_num(0)
			} else {
				sqrt::<F, F>(high)
					.map_err(|_| ArithmeticError::Underflow)?
					.checked_mul(high)
					.ok_or(ArithmeticError::Overflow)?
			}
		};
		let sqrt_x3_low: F = {
			if low == F::from_num(0) {
				F::from_num(0)
			} else {
				sqrt::<F, F>(low)
					.map_err(|_| ArithmeticError::Underflow)?
					.checked_mul(low)
					.ok_or(ArithmeticError::Overflow)?
			}
		};

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
