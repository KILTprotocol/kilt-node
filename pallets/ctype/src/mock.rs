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

use crate::{Config, CtypeHashOf};
use sp_core::H256;

const DEFAULT_CTYPE_HASH_SEED: u64 = 1u64;
const ALTERNATIVE_CTYPE_HASH_SEED: u64 = 2u64;

pub fn get_ctype_hash<T>(default: bool) -> CtypeHashOf<T>
where
	T: Config,
	T::Hash: From<H256>,
{
	if default {
		H256::from_low_u64_be(DEFAULT_CTYPE_HASH_SEED).into()
	} else {
		H256::from_low_u64_be(ALTERNATIVE_CTYPE_HASH_SEED).into()
	}
}

#[cfg(test)]
pub mod runtime {
	use frame_support::parameter_types;
	use kilt_support::mock::{mock_origin, SubjectId};
	use runtime_common::{Balance, Header, RocksDbWeight};
	use sp_runtime::{
		traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
		AccountId32,
	};

	use crate::{BalanceOf, Ctypes};

	use super::*;

	pub type TestCtypeHash = runtime_common::Hash;
	pub type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
	pub type Block = frame_system::mocking::MockBlock<Test>;

	frame_support::construct_runtime!(
		pub enum Test where
			Block = Block,
			NodeBlock = Block,
			UncheckedExtrinsic = UncheckedExtrinsic,
		{
			System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
			Ctype: crate::{Pallet, Call, Storage, Event<T>},
			Balances: pallet_balances::{Pallet, Call, Storage, Event<T>},
			MockOrigin: mock_origin::{Pallet, Origin<T>},
		}
	);

	parameter_types! {
		pub const SS58Prefix: u8 = 38;
		pub const BlockHashCount: u64 = 250;
	}

	impl frame_system::Config for Test {
		type Origin = Origin;
		type Call = Call;
		type Index = u64;
		type BlockNumber = u64;
		type Hash = runtime_common::Hash;
		type Hashing = BlakeTwo256;
		type AccountId = <<runtime_common::Signature as Verify>::Signer as IdentifyAccount>::AccountId;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type Event = ();
		type BlockHashCount = BlockHashCount;
		type DbWeight = RocksDbWeight;
		type Version = ();

		type PalletInfo = PalletInfo;
		type AccountData = pallet_balances::AccountData<Balance>;
		type OnNewAccount = ();
		type OnKilledAccount = ();
		type BaseCallFilter = frame_support::traits::Everything;
		type SystemWeightInfo = ();
		type BlockWeights = ();
		type BlockLength = ();
		type SS58Prefix = SS58Prefix;
		type OnSetCode = ();
		type MaxConsumers = frame_support::traits::ConstU32<16>;
	}

	parameter_types! {
		pub const ExistentialDeposit: Balance = 500;
		pub const MaxLocks: u32 = 50;
		pub const MaxReserves: u32 = 50;
	}

	impl pallet_balances::Config for Test {
		type Balance = Balance;
		type DustRemoval = ();
		type Event = ();
		type ExistentialDeposit = ExistentialDeposit;
		type AccountStore = System;
		type WeightInfo = ();
		type MaxLocks = MaxLocks;
		type MaxReserves = MaxReserves;
		type ReserveIdentifier = [u8; 8];
	}

	impl mock_origin::Config for Test {
		type Origin = Origin;
		type AccountId = runtime_common::AccountId;
		type SubjectId = SubjectId;
	}

	parameter_types! {
		pub const Fee: Balance = 500;
	}

	impl Config for Test {
		type CtypeCreatorId = SubjectId;
		type EnsureOrigin = mock_origin::EnsureDoubleOrigin<runtime_common::AccountId, SubjectId>;
		type OriginSuccess = mock_origin::DoubleOrigin<runtime_common::AccountId, SubjectId>;
		type Event = ();
		type WeightInfo = ();

		type Currency = Balances;
		type Fee = Fee;
		type FeeCollector = ();
	}

	pub(crate) const DID_00: SubjectId = SubjectId(AccountId32::new([1u8; 32]));
	pub(crate) const ACCOUNT_00: runtime_common::AccountId = runtime_common::AccountId::new([1u8; 32]);

	#[derive(Clone, Default)]
	pub(crate) struct ExtBuilder {
		ctypes_stored: Vec<(TestCtypeHash, SubjectId)>,
		balances: Vec<(runtime_common::AccountId, BalanceOf<Test>)>,
	}

	impl ExtBuilder {
		pub(crate) fn with_ctypes(mut self, ctypes: Vec<(TestCtypeHash, SubjectId)>) -> Self {
			self.ctypes_stored = ctypes;
			self
		}

		pub(crate) fn with_balances(mut self, balances: Vec<(runtime_common::AccountId, BalanceOf<Test>)>) -> Self {
			self.balances = balances;
			self
		}

		pub(crate) fn build(self) -> sp_io::TestExternalities {
			let mut storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
			pallet_balances::GenesisConfig::<Test> {
				balances: self.balances.clone(),
			}
			.assimilate_storage(&mut storage)
			.expect("assimilate should not fail");
			let mut ext = sp_io::TestExternalities::new(storage);

			ext.execute_with(|| {
				for (ctype_hash, owner) in self.ctypes_stored.iter() {
					Ctypes::<Test>::insert(ctype_hash, owner);
				}
			});

			ext
		}

		#[cfg(feature = "runtime-benchmarks")]
		pub(crate) fn build_with_keystore(self) -> sp_io::TestExternalities {
			use sp_keystore::{testing::KeyStore, KeystoreExt};
			use sp_std::sync::Arc;

			let mut ext = self.build();

			let keystore = KeyStore::new();
			ext.register_extension(KeystoreExt(Arc::new(keystore)));

			ext
		}
	}
}
