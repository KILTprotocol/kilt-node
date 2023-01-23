// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2023 BOTLabs GmbH

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
	use frame_support::{ord_parameter_types, parameter_types, weights::constants::RocksDbWeight};
	use frame_system::EnsureSignedBy;
	use kilt_support::mock::{mock_origin, SubjectId};
	use sp_runtime::{
		testing::Header,
		traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
		AccountId32, MultiSignature,
	};

	use crate::{BalanceOf, CtypeEntryOf, Ctypes};

	use super::*;

	pub type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
	pub type Block = frame_system::mocking::MockBlock<Test>;
	pub type Hash = sp_core::H256;
	pub type Balance = u128;
	pub type Signature = MultiSignature;
	pub type AccountPublic = <Signature as Verify>::Signer;
	pub type AccountId = <AccountPublic as IdentifyAccount>::AccountId;

	pub const UNIT: Balance = 10u128.pow(15);
	pub const MILLI_UNIT: Balance = 10u128.pow(12);

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
		type RuntimeOrigin = RuntimeOrigin;
		type RuntimeCall = RuntimeCall;
		type Index = u64;
		type BlockNumber = u64;
		type Hash = Hash;
		type Hashing = BlakeTwo256;
		type AccountId = AccountId;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type RuntimeEvent = ();
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
		type RuntimeEvent = ();
		type ExistentialDeposit = ExistentialDeposit;
		type AccountStore = System;
		type WeightInfo = ();
		type MaxLocks = MaxLocks;
		type MaxReserves = MaxReserves;
		type ReserveIdentifier = [u8; 8];
	}

	impl mock_origin::Config for Test {
		type RuntimeOrigin = RuntimeOrigin;
		type AccountId = AccountId;
		type SubjectId = SubjectId;
	}

	parameter_types! {
		pub const Fee: Balance = 500;
	}

	ord_parameter_types! {
		pub const OverarchingOrigin: AccountId = ACCOUNT_00;
	}

	impl Config for Test {
		type CtypeCreatorId = SubjectId;
		type EnsureOrigin = mock_origin::EnsureDoubleOrigin<AccountId, SubjectId>;
		type OverarchingOrigin = EnsureSignedBy<OverarchingOrigin, AccountId>;
		type OriginSuccess = mock_origin::DoubleOrigin<AccountId, SubjectId>;
		type RuntimeEvent = ();
		type WeightInfo = ();

		type Currency = Balances;
		type Fee = Fee;
		type FeeCollector = ();
	}

	pub(crate) const DID_00: SubjectId = SubjectId(AccountId32::new([1u8; 32]));
	pub(crate) const ACCOUNT_00: AccountId = AccountId::new([1u8; 32]);
	pub(crate) const ACCOUNT_01: AccountId = AccountId::new([2u8; 32]);

	#[derive(Clone, Default)]
	pub(crate) struct ExtBuilder {
		ctypes_stored: Vec<(CtypeHashOf<Test>, SubjectId)>,
		balances: Vec<(AccountId, BalanceOf<Test>)>,
	}

	impl ExtBuilder {
		pub(crate) fn with_ctypes(mut self, ctypes: Vec<(CtypeHashOf<Test>, SubjectId)>) -> Self {
			self.ctypes_stored = ctypes;
			self
		}

		pub(crate) fn with_balances(mut self, balances: Vec<(AccountId, BalanceOf<Test>)>) -> Self {
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
					Ctypes::<Test>::insert(
						ctype_hash,
						CtypeEntryOf::<Test> {
							creator: owner.clone(),
							created_at: System::block_number(),
						},
					);
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
