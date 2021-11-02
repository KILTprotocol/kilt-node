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

use frame_support::{
	traits::{Currency, Get, Imbalance, OnUnbalanced},
	weights::{DispatchClass, WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial},
};
use crate::{constants::MILLI_KILT, Balance};
use pallet_balances::WeightInfo;
use smallvec::smallvec;
use sp_runtime::Perbill;

use crate::{AccountId, NegativeImbalanceOf};

pub struct SplitFeesRatio<R, Ratio, Beneficiary1, Beneficiary2>(
	sp_std::marker::PhantomData<(R, Ratio, Beneficiary1, Beneficiary2)>,
);
impl<R, Ratio, Beneficiary1, Beneficiary2> OnUnbalanced<NegativeImbalanceOf<R>>
	for SplitFeesRatio<R, Ratio, Beneficiary1, Beneficiary2>
where
	R: pallet_balances::Config,
	Beneficiary1: OnUnbalanced<NegativeImbalanceOf<R>>,
	Beneficiary2: OnUnbalanced<NegativeImbalanceOf<R>>,
	Ratio: Get<(u32, u32)>,
{
	fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = NegativeImbalanceOf<R>>) {
		let ratio = Ratio::get();
		if let Some(fees) = fees_then_tips.next() {
			// for fees, 80% to treasury, 20% to author
			let mut split = fees.ration(ratio.0, ratio.1);
			if let Some(tips) = fees_then_tips.next() {
				// for tips, if any, 100% to author
				tips.merge_into(&mut split.1);
			}
			Beneficiary1::on_unbalanced(split.0);
			Beneficiary2::on_unbalanced(split.1);
		}
	}
}

/// Logic for the author to get a portion of fees.
pub struct ToAuthor<R>(sp_std::marker::PhantomData<R>);

impl<R> OnUnbalanced<NegativeImbalanceOf<R>> for ToAuthor<R>
where
	R: pallet_balances::Config + pallet_authorship::Config,
	<R as frame_system::Config>::AccountId: From<AccountId>,
	<R as frame_system::Config>::AccountId: Into<AccountId>,
	<R as frame_system::Config>::Event: From<pallet_balances::Event<R>>,
	<R as pallet_balances::Config>::Balance: Into<u128>,
{
	fn on_nonzero_unbalanced(amount: NegativeImbalanceOf<R>) {
		let numeric_amount = amount.peek();
		let author = pallet_authorship::Pallet::<R>::author();
		pallet_balances::Pallet::<R>::resolve_creating(&author, amount);
		frame_system::Pallet::<R>::deposit_event(pallet_balances::Event::Deposit(author, numeric_amount));
	}
}

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
pub struct WeightToFee<R>(sp_std::marker::PhantomData<R>);
impl<R> WeightToFeePolynomial for WeightToFee<R>
where
	R: pallet_transaction_payment::Config,
	<R as pallet_transaction_payment::Config>::TransactionByteFee: Get<Balance>,
	R: frame_system::Config,
	R: pallet_balances::Config,
{
	type Balance = Balance;
	fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
		// The should be fee
		let wanted_fee: Balance = 10 * MILLI_KILT;

		let per_byte_fee: u128 = <R as pallet_transaction_payment::Config>::TransactionByteFee::get();
		// TODO: transfer_keep_alive is 288 byte long?
		let byte_fee: u128 = 288_u128 * per_byte_fee;
		let base_weight: Balance = <R as frame_system::Config>::BlockWeights::get()
			.get(DispatchClass::Normal)
			.base_extrinsic
			.into();
		let tx_weight: Balance = <R as pallet_balances::Config>::WeightInfo::transfer_keep_alive().into();
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
