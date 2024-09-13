use sp_arithmetic::{fixed_point::FixedPointNumber, ArithmeticError};

pub trait BondingFunction<F : FixedPointNumber>
{
    fn get_value(&self, x: F) -> Result<F, ArithmeticError>;

    fn get_power_2(x: F) -> Result<F, ArithmeticError> {
        Ok(x.saturating_mul(x))
    }
    fn get_power_3(x: F) -> Result<F, ArithmeticError> {
        Ok(Self::get_power_2(x)?.saturating_mul(x))
    }

    // Change naming
    fn calculate_integral(&self, low: F, high: F) -> Result<F, ArithmeticError> {
        let high_val = self.get_value(high)?;
        let low_val = self.get_value(low)?;
        Ok(high_val.saturating_sub(low_val))
    }
}

#[derive(Debug, Clone)]
pub struct LinearBondingFunctionParameters<F> {
    m: F,
    n: F,
}

impl<F> BondingFunction<F> for LinearBondingFunctionParameters<F>
where
    F:  FixedPointNumber,
{
    // f(x) = m * x^2 + n * x
    fn get_value(&self, x: F) -> Result<F, ArithmeticError> {
        let x2 = Self::get_power_2(x)?;
        // can also be a Underflow. TODO: CHECK how I can figure out the error
        let mx2 = self.m.clone().checked_mul(&x2).ok_or(ArithmeticError::Overflow)?;

        let nx = self.n.clone().checked_mul(&x).ok_or(ArithmeticError::Overflow)?;

        let result = mx2.checked_add(&nx).ok_or(ArithmeticError::Overflow)?;

        // we do not need the fractions here. So we truncate the result
        Ok(result.trunc())
    }
}





#[cfg(test)]
mod tests {
    use super::*;

    use frame_support::assert_err;
    use sp_arithmetic::FixedU128;

    #[test]
    fn test_linear_bonding_function_basic_test() {
        let m = FixedU128::from_u32(1);
        let n = FixedU128::from_u32(2);
        let x = FixedU128::from_u32(1);


        let curve = LinearBondingFunctionParameters { m, n };

        // 1*1^2 + 2*1 = 3
        let result = curve.get_value(x).unwrap();
        assert_eq!(result, FixedU128::from_u32(3));
    }
    
    #[test]
    fn test_linear_bonding_function_fraction() {
        
        let m = FixedU128::from_rational(1, 2);
        let n = FixedU128::from_u32(2);
        let x = FixedU128::from_u32(1);

        let curve = LinearBondingFunctionParameters { m, n };

        // 0.5*1^2 + 2*1 = 2
        let result = curve.get_value(x).unwrap();
        assert_eq!(result, FixedU128::from_u32(2));
    }
    
    #[test]
    fn test_linear_bonding_overflow_n() {
        let m = FixedU128::from_u32(1);
        let n = FixedU128::from_inner(u128::MAX);
        let x = FixedU128::from_u32(2);

        let curve = LinearBondingFunctionParameters { m, n };

        let result = curve.get_value(x);
        assert_err!(result, ArithmeticError::Overflow);
    }

    #[test]
    fn test_linear_bonding_overflow_m() {
        let m = FixedU128::from_inner(u128::MAX);
        let n = FixedU128::from_inner(1);
        let x = FixedU128::from_u32(2);

        let curve = LinearBondingFunctionParameters { m, n };

        let result = curve.get_value(x);
        assert_err!(result, ArithmeticError::Overflow);
    }
}

 
