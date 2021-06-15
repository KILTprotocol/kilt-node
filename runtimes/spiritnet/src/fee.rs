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

use codec::Encode;
use frame_support::{
	traits::Get,
	weights::{WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial},
};
use kilt_primitives::{constants::KILT, AccountId, Balance};
use pallet_balances::WeightInfo;
use pallet_transaction_payment::OnChargeTransaction;
use smallvec::smallvec;
use sp_runtime::traits::Zero;
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
	T: frame_system::Config + pallet_transaction_payment::Config,
	Balance: From<<<T as pallet_transaction_payment::Config>::OnChargeTransaction as OnChargeTransaction<T>>::Balance>;
impl<T: frame_system::Config + pallet_transaction_payment::Config> WeightToFeePolynomial for WeightToFee<T>
where
	Balance: From<<<T as pallet_transaction_payment::Config>::OnChargeTransaction as OnChargeTransaction<T>>::Balance>,
{
	type Balance = Balance;
	fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
		// In Spiritnet, transfer_keep_alive should cost 0.01 KILT in an empty block
		let wanted_fee: Balance = KILT / 100;
		let transfer_weight: Balance = crate::weights::pallet_balances::WeightInfo::<T>::transfer_keep_alive().into();

		// Subtract byte fee for transfer_keep_alive transaction
		let per_byte: Balance = Balance::from(<T as pallet_transaction_payment::Config>::TransactionByteFee::get());
		let tx = crate::Call::Balances(pallet_balances::Call::transfer_keep_alive(
			AccountId::default().into(),
			Balance::zero(),
		));
		let tx_byte_fee = tx.encode().len() as u128 * per_byte;

		// f(x) = 0.01 KILT / weight(transfer_keep_alive()) * x -
		// byte_fee(transfer_keep_alive())
		smallvec![
			WeightToFeeCoefficient {
				degree: 0,
				negative: true,
				coeff_frac: Perbill::zero(),
				coeff_integer: tx_byte_fee,
			},
			WeightToFeeCoefficient {
				degree: 1,
				negative: false,
				coeff_frac: Perbill::from_rational(wanted_fee % transfer_weight, transfer_weight),
				coeff_integer: wanted_fee / transfer_weight,
			}
		]
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{AccountId, Call, Runtime, TransactionPayment};
	use codec::Encode;
	use sp_runtime::traits::Zero;

	#[test]
	fn transaction_fee_is_correct() {
		let storage = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();
		let mut ext = sp_io::TestExternalities::new(storage);
		ext.execute_with(|| {
			let tx = Call::Balances(pallet_balances::Call::transfer_keep_alive(
				AccountId::default().into(),
				Balance::zero(),
			));

			let info = TransactionPayment::query_fee_details(tx.clone(), tx.encode().len() as u32);
			let incl_fee = info.inclusion_fee.unwrap();
			assert_eq!(
				incl_fee.base_fee + incl_fee.len_fee + incl_fee.adjusted_weight_fee,
				KILT / 100,
				"base fee: {:?} byte fee: {:?} adjusted fee {:?}",
				incl_fee.base_fee,
				incl_fee.len_fee,
				incl_fee.adjusted_weight_fee
			);
		})
	}

	// TODO: Add test for full block weight once attestation weights have been
	// calculated
}
