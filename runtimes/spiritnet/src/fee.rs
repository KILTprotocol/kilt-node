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

use crate::Runtime;
use frame_support::weights::{DispatchClass, WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial};
use kilt_primitives::{constants::MILLI_KILT, Balance};
use pallet_balances::WeightInfo;
use smallvec::smallvec;
use sp_runtime::Perbill;

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
pub struct WeightToFee;
impl WeightToFeePolynomial for WeightToFee {
	type Balance = Balance;
	fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
		// The should be fee
		let wanted_fee: Balance = 10 * MILLI_KILT;

		let per_byte_fee: u128 = <Runtime as pallet_transaction_payment::Config>::TransactionByteFee::get();
		// TODO: transfer_keep_alive is 288 byte long?
		let byte_fee: u128 = 288_u128 * per_byte_fee;
		let base_weight: Balance = <Runtime as frame_system::Config>::BlockWeights::get()
			.get(DispatchClass::Normal)
			.base_extrinsic
			.into();
		let tx_weight: Balance = <Runtime as pallet_balances::Config>::WeightInfo::transfer_keep_alive().into();
		let unbalanced_fee: Balance = base_weight + tx_weight;

		let wanted_weight_fee: Balance = wanted_fee - byte_fee;

		smallvec![WeightToFeeCoefficient {
			degree: 1,
			negative: false,
			coeff_frac: Perbill::from_rational(wanted_weight_fee % unbalanced_fee, unbalanced_fee),
			coeff_integer: wanted_weight_fee / unbalanced_fee,
		}]
	}
}
