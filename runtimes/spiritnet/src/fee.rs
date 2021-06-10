/// Fee-related.

pub use sp_runtime::Perbill;
use kilt_primitives::constants::MILLI_KILT;
use runtime_common::ExtrinsicBaseWeight;
use frame_support::weights::{
    WeightToFeePolynomial, WeightToFeeCoefficient, WeightToFeeCoefficients,
};
use smallvec::smallvec;
use weights::pallet_balances::WeightInfo::transfer;

/// The block saturation level. Fees will be updates based on this value.
pub const TARGET_BLOCK_FULLNESS: Perbill = Perbill::from_percent(25);

/// Handles converting a weight scalar to a fee value, based on the scale and granularity of the
/// node's balance type.
///
/// This should typically create a mapping between the following ranges:
///   - [0, MAXIMUM_BLOCK_WEIGHT]
///   - [Balance::min, Balance::max]
///
/// Yet, it can be used for any other sort of change to weight-fee. Some examples being:
///   - Setting it to `0` will essentially disable the weight fee.
///   - Setting it to `1` will cause the literal `#[weight = x]` values to be charged.
pub struct WeightToFee;
impl WeightToFeePolynomial for WeightToFee {
    type Balance = Balance;
    fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
        // in Spiritnet, transfer weight is mapped to 0.001 KILT:
        let p = MILLI_KILT;
        let q = 10 * Balance::from(weights::pallet_balances::WeightInfo::transfer());
        smallvec![WeightToFeeCoefficient {
            degree: 1,
            negative: false,
            coeff_frac: Perbill::from_rational(p % q, q),
            coeff_integer: p / q,
        }]
    }
}
