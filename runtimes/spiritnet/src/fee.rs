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

// TODO: Add test

#[cfg(test)]
mod tests {
	use super::WeightToFee;
	use crate::{BlockHashCount, SS58Prefix};
	use frame_support::weights::WeightToFeePolynomial;
	use kilt_primitives::{constants::KILT, AccountId, Balance, BlockNumber, Hash, Index};
	use pallet_balances::WeightInfo;
	use sp_runtime::{
		testing::Header,
		traits::{BlakeTwo256, IdentityLookup},
	};

	type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
	type Block = frame_system::mocking::MockBlock<Test>;

	// Configure a mock runtime to test the pallet.
	frame_support::construct_runtime!(
		pub enum Test where
			Block = Block,
			NodeBlock = Block,
			UncheckedExtrinsic = UncheckedExtrinsic,
		{
			System: frame_system::{Pallet, Call, Config, Storage, Event<T>}
		}
	);

	impl frame_system::Config for Test {
		type BaseCallFilter = ();
		type BlockWeights = ();
		type BlockLength = ();
		type DbWeight = ();
		type Origin = Origin;
		type Call = Call;
		type Index = Index;
		type BlockNumber = BlockNumber;
		type Hash = Hash;
		type Hashing = BlakeTwo256;
		type AccountId = AccountId;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type Event = Event;
		type BlockHashCount = BlockHashCount;
		type Version = ();
		type PalletInfo = PalletInfo;
		type AccountData = pallet_balances::AccountData<Balance>;
		type OnNewAccount = ();
		type OnKilledAccount = ();
		type SystemWeightInfo = ();
		type SS58Prefix = SS58Prefix;
		type OnSetCode = ();
	}

	#[test]
	fn transaction_fee_is_correct() {
		assert_eq!(
			WeightToFee::<Test>::calc(&crate::weights::pallet_balances::WeightInfo::<Test>::transfer()),
			KILT / 100
		);
	}

	// TODO: Add test for full block weight once attestation weights have been
	// calculated
}
