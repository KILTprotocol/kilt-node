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

use codec::{Decode, Encode};
use frame_support::parameter_types;
use scale_info::TypeInfo;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	AccountId32,
};

use crate as pallet_did_lookup;
use kilt_primitives::{AccountId, AccountPublic, Balance, BlockHashCount, BlockNumber, Hash, Index, Signature};

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Event<T>},
		DidLookup: pallet_did_lookup::{Pallet, Storage, Call, Event<T>},
		MockOrigin: mock_origin::{Pallet, Origin},
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
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = SS58Prefix;
	type OnSetCode = ();
}

parameter_types! {
	pub const ExistentialDeposit: Balance = 10;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Test {
	type Balance = Balance;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type MaxLocks = MaxLocks;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
	type WeightInfo = ();
}

parameter_types! {
	pub const DidLookupDeposit: Balance = 10;
}

impl pallet_did_lookup::Config for Test {
	type Event = Event;
	type Signature = Signature;
	type Signer = AccountPublic;

	type Currency = Balances;
	type Deposit = DidLookupDeposit;

	type EnsureOrigin = mock_origin::EnsureDoubleOrigin;
	type OriginSuccess = mock_origin::DoubleOrigin;
	type DidIdentifier = DidIdentifier;

	type WeightInfo = ();
}

impl mock_origin::Config for Test {
	type Origin = Origin;
}

pub(crate) const ACCOUNT_00: kilt_primitives::AccountId = kilt_primitives::AccountId::new([0u8; 32]);
pub(crate) const ACCOUNT_01: kilt_primitives::AccountId = kilt_primitives::AccountId::new([1u8; 32]);
pub(crate) const DID_00: DidIdentifier = DidIdentifier(ACCOUNT_00);
pub(crate) const DID_01: DidIdentifier = DidIdentifier(ACCOUNT_01);

#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TypeInfo, Default)]
pub struct DidIdentifier(AccountId32);

impl From<AccountId32> for DidIdentifier {
	fn from(acc: AccountId32) -> Self {
		DidIdentifier(acc)
	}
}

#[frame_support::pallet]
#[allow(dead_code)]
pub mod mock_origin {
	use super::{AccountId, DidIdentifier};
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
	pub struct DoubleOrigin(pub AccountId, pub DidIdentifier);
	impl CallSources<AccountId, DidIdentifier> for DoubleOrigin {
		fn sender(&self) -> AccountId {
			self.0.clone()
		}

		fn subject(&self) -> DidIdentifier {
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

	#[cfg(feature = "runtime-benchmarks")]
	impl<OuterOrigin> kilt_support::traits::GenerateBenchmarkOrigin<OuterOrigin, AccountId, DidIdentifier>
		for EnsureDoubleOrigin
	where
		OuterOrigin: Into<Result<DoubleOrigin, OuterOrigin>> + From<DoubleOrigin>,
	{
		fn generate_origin(sender: AccountId, subject: DidIdentifier) -> OuterOrigin {
			OuterOrigin::from(DoubleOrigin(sender, subject))
		}
	}
}

#[derive(Clone, Default)]
pub struct ExtBuilder {
	balances: Vec<(AccountId, Balance)>,
	connections: Vec<(AccountId, DidIdentifier, AccountId)>,
}

impl ExtBuilder {
	pub fn with_balances(mut self, balances: Vec<(AccountId, Balance)>) -> Self {
		self.balances = balances;
		self
	}

	pub fn with_connections(mut self, connections: Vec<(AccountId, DidIdentifier, AccountId)>) -> Self {
		self.connections = connections;
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
		pallet_balances::GenesisConfig::<Test> {
			balances: self.balances.clone(),
		}
		.assimilate_storage(&mut storage)
		.expect("assimilate should not fail");
		let mut ext = sp_io::TestExternalities::new(storage);

		ext.execute_with(|| {
			for (sender, did, account) in self.connections {
				pallet_did_lookup::Pallet::<Test>::add_association(sender, did, account)
					.expect("Should create connection");
			}
		});
		ext
	}

	// allowance only required for clippy, this function is actually used
	#[cfg(feature = "runtime-benchmarks")]
	pub fn build_with_keystore(self) -> sp_io::TestExternalities {
		let mut ext = self.build();

		let keystore = sp_keystore::testing::KeyStore::new();
		ext.register_extension(sp_keystore::KeystoreExt(std::sync::Arc::new(keystore)));

		ext
	}
}
