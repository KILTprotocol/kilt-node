// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2022 BOTLabs GmbH

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
use pallet_balances::WeightInfo;
use pallet_transaction_payment::OnChargeTransaction;
use smallvec::smallvec;
use sp_runtime::Perbill;

use crate::{constants::MILLI_KILT, AccountId, Balance, NegativeImbalanceOf};

/// Split two Imbalances between two unbalanced handlers.
/// The first Imbalance will be split according to the given ratio. The second
/// Imbalance will be handled by the second beneficiary.
///
/// In case of transaction payment, the first Imbalance is the fee and the
/// second imbalance the tip.
pub struct SplitFeesByRatio<R, Ratio, Beneficiary1, Beneficiary2>(
	sp_std::marker::PhantomData<(R, Ratio, Beneficiary1, Beneficiary2)>,
);
impl<R, Ratio, Beneficiary1, Beneficiary2> OnUnbalanced<NegativeImbalanceOf<R>>
	for SplitFeesByRatio<R, Ratio, Beneficiary1, Beneficiary2>
where
	R: pallet_balances::Config,
	Beneficiary1: OnUnbalanced<NegativeImbalanceOf<R>>,
	Beneficiary2: OnUnbalanced<NegativeImbalanceOf<R>>,
	Ratio: Get<(u32, u32)>,
{
	fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = NegativeImbalanceOf<R>>) {
		let ratio = Ratio::get();
		if let Some(fees) = fees_then_tips.next() {
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
	<R as pallet_balances::Config>::Balance: Into<u128>,
{
	fn on_nonzero_unbalanced(amount: NegativeImbalanceOf<R>) {
		if let Some(author) = <pallet_authorship::Pallet<R>>::author() {
			<pallet_balances::Pallet<R>>::resolve_creating(&author, amount);
		}
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
	R: frame_system::Config,
	R: pallet_balances::Config,
	u128: From<<<R as pallet_transaction_payment::Config>::OnChargeTransaction as OnChargeTransaction<R>>::Balance>,
{
	type Balance = Balance;
	fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
		// The should be fee
		let wanted_fee: Balance = 10 * MILLI_KILT;

		// TODO: transfer_keep_alive is 288 byte long?
		let tx_len: u64 = 288;
		let byte_fee: Balance = <R as pallet_transaction_payment::Config>::LengthToFee::calc(&tx_len).into();
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

#[cfg(test)]
mod tests {
	use super::*;
	use frame_support::{parameter_types, traits::FindAuthor, weights::DispatchClass};
	use frame_system::limits;
	use sp_core::H256;
	use sp_runtime::{
		testing::Header,
		traits::{BlakeTwo256, IdentityLookup},
		Perbill,
	};

	use crate::AccountId;

	type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
	type Block = frame_system::mocking::MockBlock<Test>;

	frame_support::construct_runtime!(
		pub enum Test where
			Block = Block,
			NodeBlock = Block,
			UncheckedExtrinsic = UncheckedExtrinsic,
		{
			System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
			Authorship: pallet_authorship::{Pallet, Call, Storage, Inherent},
			Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		}
	);

	parameter_types! {
		pub const BlockHashCount: u64 = 250;
		pub BlockWeights: limits::BlockWeights = limits::BlockWeights::builder()
			.base_block(10)
			.for_class(DispatchClass::all(), |weight| {
				weight.base_extrinsic = 100;
			})
			.for_class(DispatchClass::non_mandatory(), |weight| {
				weight.max_total = Some(1024);
			})
			.build_or_panic();
		pub BlockLength: limits::BlockLength = limits::BlockLength::max(2 * 1024);
		pub const AvailableBlockRatio: Perbill = Perbill::one();
	}

	impl frame_system::Config for Test {
		type BaseCallFilter = frame_support::traits::Everything;
		type Origin = Origin;
		type Index = u64;
		type BlockNumber = u64;
		type Call = Call;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type AccountId = AccountId;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type Event = Event;
		type BlockHashCount = BlockHashCount;
		type BlockLength = BlockLength;
		type BlockWeights = BlockWeights;
		type DbWeight = ();
		type Version = ();
		type PalletInfo = PalletInfo;
		type AccountData = pallet_balances::AccountData<u64>;
		type OnNewAccount = ();
		type OnKilledAccount = ();
		type SystemWeightInfo = ();
		type SS58Prefix = ();
		type OnSetCode = ();
		type MaxConsumers = frame_support::traits::ConstU32<16>;
	}

	impl pallet_balances::Config for Test {
		type Balance = u64;
		type Event = Event;
		type DustRemoval = ();
		type ExistentialDeposit = ();
		type AccountStore = System;
		type MaxLocks = ();
		type MaxReserves = ();
		type ReserveIdentifier = [u8; 8];
		type WeightInfo = ();
	}

	pub const TREASURY_ACC: AccountId = crate::AccountId::new([1u8; 32]);
	const AUTHOR_ACC: AccountId = AccountId::new([2; 32]);

	pub struct ToBeneficiary();
	impl OnUnbalanced<NegativeImbalanceOf<Test>> for ToBeneficiary {
		fn on_nonzero_unbalanced(amount: NegativeImbalanceOf<Test>) {
			// Must resolve into existing but better to be safe.
			<pallet_balances::Pallet<Test>>::resolve_creating(&TREASURY_ACC, amount);
		}
	}

	pub struct OneAuthor;
	impl FindAuthor<AccountId> for OneAuthor {
		fn find_author<'a, I>(_: I) -> Option<AccountId>
		where
			I: 'a,
		{
			Some(AUTHOR_ACC)
		}
	}
	impl pallet_authorship::Config for Test {
		type FindAuthor = OneAuthor;
		type UncleGenerations = ();
		type FilterUncle = ();
		type EventHandler = ();
	}

	pub fn new_test_ext() -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
		// We use default for brevity, but you can configure as desired if needed.
		pallet_balances::GenesisConfig::<Test>::default()
			.assimilate_storage(&mut t)
			.unwrap();
		t.into()
	}

	parameter_types! {
		pub const Ratio: (u32, u32) = (50, 50);
	}

	#[test]
	fn test_fees_and_tip_split() {
		new_test_ext().execute_with(|| {
			let fee = Balances::issue(10);
			let tip = Balances::issue(20);

			assert_eq!(Balances::free_balance(TREASURY_ACC), 0);
			assert_eq!(Balances::free_balance(AUTHOR_ACC), 0);

			SplitFeesByRatio::<Test, Ratio, ToBeneficiary, ToAuthor<Test>>::on_unbalanceds(vec![fee, tip].into_iter());

			assert_eq!(Balances::free_balance(TREASURY_ACC), 5);
			assert_eq!(Balances::free_balance(AUTHOR_ACC), 25);
		});
	}
}
