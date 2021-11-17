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

use crate as pallet_did_lookup;
use frame_support::parameter_types;
use kilt_primitives::{AccountId, AccountPublic, BlockHashCount, BlockNumber, Hash, Index, Signature};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		DidLookup: pallet_did_lookup::{Pallet, Storage, Call, Event<T>},
		MockOrigin: mock_origin::{Pallet, Origin}
	}
);

parameter_types! {
	pub const SS58Prefix: u8 = 38;
}

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
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
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
}

impl pallet_did_lookup::Config for Test {
	type Event = Event;
	type Signature = Signature;
	type Signer = AccountPublic;

	type EnsureOrigin = mock_origin::EnsureDoubleOrigin;
	type OriginSuccess = mock_origin::DoubleOrigin;
	type DidAccount = u64;

	type WeightInfo = ();
}

impl mock_origin::Config for Test {
	type Origin = Origin;
}

#[frame_support::pallet]
mod mock_origin {
	use super::AccountId;
	use kilt_support::traits::CallSources;

	use codec::{Decode, Encode};
	use frame_support::traits::EnsureOrigin;
	use scale_info::TypeInfo;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Origin: From<DoubleOrigin>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::origin]
	pub type Origin = DoubleOrigin;

	#[derive(Debug, Clone, Default, PartialEq, Eq, TypeInfo, Encode, Decode)]
	pub struct DoubleOrigin(AccountId, u64);
	impl CallSources<AccountId, u64> for DoubleOrigin {
		fn sender(&self) -> AccountId {
			self.0.clone()
		}

		fn subject(&self) -> u64 {
			self.1.clone()
		}
	}

	pub struct EnsureDoubleOrigin;

	impl<OuterOrigin> EnsureOrigin<OuterOrigin> for EnsureDoubleOrigin
	where
		OuterOrigin: Into<Result<DoubleOrigin, OuterOrigin>> + From<DoubleOrigin>,
	{
		type Success = DoubleOrigin;

		fn try_origin(o: OuterOrigin) -> Result<Self::Success, OuterOrigin> {
			o.into()
		}

		#[cfg(feature = "runtime-benchmarks")]
		fn successful_origin() -> OuterOrigin {
			OuterOrigin::from(Default::default())
		}
	}
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::default()
		.build_storage::<Test>()
		.unwrap()
		.into()
}
