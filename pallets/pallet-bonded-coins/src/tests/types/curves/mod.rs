mod polynomial_curve;
mod square_root;

use sp_runtime::FixedPointNumber;

use crate::{mock::runtime::*, CurveParameterTypeOf};

const NORMALIZED_DENOMINATION: u128 = CurveParameterTypeOf::<Test>::DIV;
const CURRENT_DENOMINATION: u128 = 10u128.pow(15);
