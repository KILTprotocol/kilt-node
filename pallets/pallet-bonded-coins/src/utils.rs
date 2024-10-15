use sp_arithmetic::ArithmeticError;
use substrate_fixed::traits::ToFixed;

use crate::{Config, CurveParameterTypeOf};

pub fn convert_balance_to_parameter<T: Config>(
	x: u128,
	denomination: &u8,
) -> Result<CurveParameterTypeOf<T>, ArithmeticError> {
	let decimals = 10u128
		.checked_pow(u32::from(*denomination))
		.ok_or(ArithmeticError::Overflow)?;
	let scaled_x = x.checked_div(decimals).ok_or(ArithmeticError::DivisionByZero)?;
	scaled_x.checked_to_fixed().ok_or(ArithmeticError::Overflow)
}
