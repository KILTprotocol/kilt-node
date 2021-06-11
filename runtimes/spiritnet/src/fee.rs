// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2021 BOTLabs GmbH

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

use frame_support::weights::{WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial};
use kilt_primitives::{constants::KILT, Balance};
use pallet_balances::WeightInfo;
use smallvec::smallvec;
pub use sp_runtime::Perbill;
use sp_std::marker::PhantomData;

/// Handles converting a weight scalar to a fee value, based on the scale and
/// granularity of the node's balance type.
///
/// This should typically create a mapping between the following ranges:
///   - [0, MAXIMUM_BLOCK_WEIGHT]
///   - [Balance::min, Balance::max]
///
/// Yet, it can be used for any other sort of change to weight-fee. Some
/// examples being:
///   - Setting it to `0` will essentially disable the weight fee.
///   - Setting it to `1` will cause the literal `#[weight = x]` values to be
///     charged.
pub struct WeightToFee<T>(PhantomData<T>)
where
	T: frame_system::Config;
impl<T: frame_system::Config> WeightToFeePolynomial for WeightToFee<T> {
	type Balance = Balance;
	fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
		// in Spiritnet, transfer weight is mapped to 0.01 KILT:
		let p = KILT / 100;
		let q = Balance::from(crate::weights::pallet_balances::WeightInfo::<T>::transfer());

		// f(w) = MILLI_KILT / WeightInfo::transfer() * w
		smallvec![WeightToFeeCoefficient {
			degree: 1,
			negative: false,
			coeff_frac: Perbill::from_rational(p % q, q),
			coeff_integer: p / q,
		}]
	}
}

#[cfg(test)]
mod tests {
	use super::WeightToFee;
	use frame_support::weights::WeightToFeePolynomial;
	use kilt_primitives::constants::KILT;
	use pallet_balances::WeightInfo;

	#[test]
	fn transaction_fee_is_correct_runtime() {
		assert_eq!(
			WeightToFee::<crate::Runtime>::calc(
				&crate::weights::pallet_balances::WeightInfo::<crate::Runtime>::transfer()
			),
			KILT / 100
		);
	}

	// TODO: Add test for full block weight once attestation weights have been
	// calculated
}
