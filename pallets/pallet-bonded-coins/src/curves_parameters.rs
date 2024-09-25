use frame_support::ensure;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_arithmetic::ArithmeticError;
use sp_runtime::{
	traits::{CheckedDiv, Zero},
	DispatchError, FixedPointNumber, FixedU128, SaturatedConversion, Saturating,
};
use sp_std::marker::PhantomData;

use crate::{CollateralCurrencyBalanceOf, Config, CurveParameterTypeOf, Error, FungiblesBalanceOf};

/// Little helper trait to calculate the square root of Fixed, in order to maintain the generic.
pub trait SquareRoot: Sized {
	fn try_sqrt(self) -> Option<Self>;
	fn sqrt(self) -> Self;
}

impl SquareRoot for FixedU128 {
	fn try_sqrt(self) -> Option<Self> {
		self.clone().try_sqrt()
	}

	fn sqrt(self) -> Self {
		self.clone().sqrt()
	}
}

/// A trait to define the bonding curve functions
pub trait BondingFunction<F: FixedPointNumber> {
	/// returns the value of the curve at x.
	/// The bonding curve is already the primitive integral of f(x).
	/// Therefore the costs can be calculated by the difference of the values of the curve at two points.
	fn get_value(&self, x: F) -> Result<F, ArithmeticError>;

	/// static function to calculate the power of 2 of x
	fn get_power_2(x: F) -> Result<F, ArithmeticError> {
		Ok(x.saturating_mul(x))
	}

	/// static function to calculate the power of 3 of x
	fn get_power_3(x: F) -> Result<F, ArithmeticError> {
		Ok(Self::get_power_2(x)?.saturating_mul(x))
	}

	/// calculates the cost of the curve between low and high
	fn calculate_costs(&self, low: F, high: F) -> Result<F, ArithmeticError> {
		let high_val = self.get_value(high)?;
		let low_val = self.get_value(low)?;
		Ok(high_val.saturating_sub(low_val))
	}
}

#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct PolynomialFunctionParameters<F> {
	pub m: F,
	pub n: F,
	pub o: F,
}

impl<F> BondingFunction<F> for PolynomialFunctionParameters<F>
where
	F: FixedPointNumber,
{
	/// F(x) = m * x^3 + n * x^2 + o * x
	fn get_value(&self, x: F) -> Result<F, ArithmeticError> {
		let x2 = Self::get_power_2(x)?;
		let x3 = Self::get_power_3(x)?;

		let mx3 = self.m.clone().checked_mul(&x3).ok_or(ArithmeticError::Overflow)?;
		let nx2 = self.n.clone().checked_mul(&x2).ok_or(ArithmeticError::Overflow)?;
		let ox = self.o.clone().checked_mul(&x).ok_or(ArithmeticError::Overflow)?;

		let result = mx3
			.checked_add(&nx2)
			.ok_or(ArithmeticError::Overflow)?
			.checked_add(&ox)
			.ok_or(ArithmeticError::Overflow)?;

		ensure!(result >= F::zero(), ArithmeticError::Underflow);
		Ok(result)
	}
}

#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct SquareRootBondingFunctionParameters<F> {
	pub m: F,
	pub n: F,
}

impl<F> BondingFunction<F> for SquareRootBondingFunctionParameters<F>
where
	F: FixedPointNumber + SquareRoot,
{
	/// F(x) = m * sqrt(x^3) + n * x
	fn get_value(&self, x: F) -> Result<F, ArithmeticError> {
		let x3 = Self::get_power_3(x)?;
		let sqrt_x3 = x3.try_sqrt().ok_or(ArithmeticError::Underflow)?;
		let mx3 = self.m.clone().checked_mul(&sqrt_x3).ok_or(ArithmeticError::Overflow)?;
		let nx = self.n.clone().checked_mul(&x).ok_or(ArithmeticError::Overflow)?;

		let result = mx3.checked_add(&nx).ok_or(ArithmeticError::Overflow)?;

		ensure!(result >= F::zero(), ArithmeticError::Underflow);
		Ok(result)
	}
}

#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct RationalBondingFunctionParameters<F>(PhantomData<F>);

impl<F> RationalBondingFunctionParameters<F>
where
	F: FixedPointNumber,
{
	fn get_power_2(x: F) -> Result<F, ArithmeticError> {
		Ok(x.saturating_mul(x))
	}

	pub fn calculate_costs(&self, low: (F, F), high: (F, F)) -> Result<F, ArithmeticError> {
		let high_val = self.calculate_ration(high.0, high.1)?;
		let low_val = self.calculate_ration(low.0, high.1)?;
		Ok(high_val.saturating_sub(low_val))
	}

	/// F(a) =  0.5 * (a / (a + b))**2 + 0.5 * (b / (a + b))**2 * (a+b) , where b is the supply of the other assets.
	fn calculate_ration(&self, a: F, b: F) -> Result<F, ArithmeticError> {
		// for the case, that a and b has no supply, we return 0.
		if a.is_zero() && b.is_zero() {
			return Ok(F::zero());
		}

		// Should never happen, but lets be save. If 0.5 can not be represented as a fixed point number, we have an underflow.
		let constant = F::checked_from_rational(1, 2).ok_or(ArithmeticError::Underflow)?;

		let sum_a_b = a.checked_add(&b).ok_or(ArithmeticError::Overflow)?;

		// Should never happen.
		let a_divided_sum = a.checked_div(&sum_a_b).ok_or(ArithmeticError::DivisionByZero)?;

		// Should never happen.
		let b_divided_sum = a.checked_div(&sum_a_b).ok_or(ArithmeticError::DivisionByZero)?;

		let a_divided_sum_squared = Self::get_power_2(a_divided_sum)?;

		let b_divided_sum_squared = Self::get_power_2(b_divided_sum)?;

		let a_divided_sum_squared_multiplied = a_divided_sum_squared
			.checked_mul(&constant)
			.ok_or(ArithmeticError::Overflow)?;

		let b_divided_sum_squared_multiplied = b_divided_sum_squared
			.checked_mul(&constant)
			.ok_or(ArithmeticError::Overflow)?;

		let b_divided_sum_squared_multiplied_multiplied = b_divided_sum_squared_multiplied
			.checked_mul(&sum_a_b)
			.ok_or(ArithmeticError::Overflow)?;

		a_divided_sum_squared_multiplied
			.checked_add(&b_divided_sum_squared_multiplied_multiplied)
			.ok_or(ArithmeticError::Overflow)
	}

	pub fn process_swap<T: Config>(
		&self,
		currencies_metadata: Vec<(FungiblesBalanceOf<T>, u128)>,
		collateral_meta: (CollateralCurrencyBalanceOf<T>, u128),
		to_idx_usize: usize,
	) -> Result<FungiblesBalanceOf<T>, DispatchError> {
		let normalized_denomination = CurveParameterTypeOf::<T>::DIV;

		let (collateral, denomination) = collateral_meta;

		let normalized_collateral = convert_currency_amount::<T>(
			collateral.saturated_into::<u128>(),
			denomination,
			normalized_denomination,
		)?;

		let target_issuance = currencies_metadata
			.get(to_idx_usize)
			.ok_or(Error::<T>::IndexOutOfBounds)?
			.0;

		let normalized_target_issuance = convert_currency_amount::<T>(
			target_issuance.clone().saturated_into(),
			currencies_metadata
				.get(to_idx_usize)
				.ok_or(Error::<T>::IndexOutOfBounds)?
				.1,
			normalized_denomination,
		)?;

		let normalized_total_issuances = currencies_metadata
			.clone()
			.into_iter()
			.map(|(x, d)| convert_currency_amount::<T>(x.saturated_into::<u128>(), d, normalized_denomination))
			.try_fold(CurveParameterTypeOf::<T>::zero(), |sum, result| {
				result.map(|x| sum.saturating_add(x))
			})?;

		let ratio = normalized_target_issuance
			.checked_div(&normalized_total_issuances)
			.ok_or(ArithmeticError::DivisionByZero)?;

		let supply_to_mint = normalized_collateral
			.checked_div(&ratio)
			.ok_or(ArithmeticError::DivisionByZero)?;

		let raw_supply = convert_currency_amount::<T>(
			supply_to_mint.into_inner(),
			normalized_denomination,
			currencies_metadata
				.get(to_idx_usize)
				.ok_or(Error::<T>::IndexOutOfBounds)?
				.1,
		)?;

		Ok(raw_supply.into_inner().saturated_into())
	}
}

pub fn convert_currency_amount<T: Config>(
	amount: u128,
	current_denomination: u128,
	target_denomination: u128,
) -> Result<CurveParameterTypeOf<T>, ArithmeticError> {
	let value = {
		if target_denomination > current_denomination {
			let factor = target_denomination
				.checked_div(current_denomination)
				.ok_or(ArithmeticError::DivisionByZero)?;
			amount.checked_mul(factor).ok_or(ArithmeticError::Overflow)
		} else {
			let factor = current_denomination
				.checked_div(target_denomination)
				.ok_or(ArithmeticError::DivisionByZero)?;
			amount.checked_div(factor).ok_or(ArithmeticError::DivisionByZero)
		}
	}?;

	Ok(CurveParameterTypeOf::<T>::from_inner(value))
}
