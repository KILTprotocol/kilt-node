// KILT Blockchain – <https://kilt.io>
// Copyright (C) 2025, KILT Foundation

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

// If you feel like getting in touch with us, you can do so at <hello@kilt.io>

use frame_support::{
	construct_runtime, parameter_types,
	sp_runtime::{
		testing::H256,
		traits::{BlakeTwo256, IdentityLookup},
		AccountId32,
	},
	traits::{ConstU16, ConstU32, ConstU64, Currency, Everything},
};
use frame_system::{mocking::MockBlock, EnsureSigned};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

use crate::{self as storage_deposit_pallet, DepositEntryOf, DepositKeyOf, Pallet};

pub(crate) type Balance = u128;

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, PartialEq, Eq, RuntimeDebug, Default)]
pub enum DepositNamespace {
	#[default]
	ExampleNamespace,
}

construct_runtime!(
	pub struct TestRuntime {
		System: frame_system,
		StorageDepositPallet: storage_deposit_pallet,
		Balances: pallet_balances,
	}
);

impl frame_system::Config for TestRuntime {
	type AccountData = pallet_balances::AccountData<Balance>;
	type AccountId = AccountId32;
	type BaseCallFilter = Everything;
	type Block = MockBlock<TestRuntime>;
	type BlockHashCount = ConstU64<256>;
	type BlockLength = ();
	type BlockWeights = ();
	type DbWeight = ();
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type Lookup = IdentityLookup<Self::AccountId>;
	type MaxConsumers = ConstU32<16>;
	type Nonce = u64;
	type OnKilledAccount = ();
	type OnNewAccount = ();
	type OnSetCode = ();
	type PalletInfo = PalletInfo;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeTask = ();
	type SS58Prefix = ConstU16<1>;
	type SystemWeightInfo = ();
	type Version = ();
	type MultiBlockMigrator = ();
	type SingleBlockMigrations = ();
	type PostInherents = ();
	type PostTransactions = ();
	type PreInherents = ();
}

parameter_types! {
	pub const ExistentialDeposit: Balance = 500;
	pub const MaxLocks: u32 = 50;
	pub const MaxReserves: u32 = 50;
	pub const MaxFreezes: u32 = 50;
}

impl pallet_balances::Config for TestRuntime {
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type FreezeIdentifier = RuntimeFreezeReason;
	type RuntimeHoldReason = RuntimeHoldReason;
	type MaxFreezes = MaxFreezes;
	type Balance = Balance;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = MaxLocks;
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
}

impl crate::Config for TestRuntime {
	type CheckOrigin = EnsureSigned<Self::AccountId>;
	type Currency = Balances;
	type DepositHooks = ();
	type RuntimeEvent = RuntimeEvent;
	type RuntimeHoldReason = RuntimeHoldReason;
	type MaxKeyLength = ConstU32<256>;
	type Namespace = DepositNamespace;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHooks = ();
	type WeightInfo = ();
}

pub(crate) const OWNER: AccountId32 = AccountId32::new([100u8; 32]);
pub(crate) const OTHER_ACCOUNT: AccountId32 = AccountId32::new([101u8; 32]);

#[derive(Default)]
pub(crate) struct ExtBuilder(
	Vec<(AccountId32, Balance)>,
	Vec<(DepositNamespace, DepositKeyOf<TestRuntime>, DepositEntryOf<TestRuntime>)>,
);

impl ExtBuilder {
	pub(crate) fn with_balances(mut self, balances: Vec<(AccountId32, Balance)>) -> Self {
		self.0 = balances;
		self
	}

	pub(crate) fn with_deposits(
		mut self,
		deposits: Vec<(DepositNamespace, DepositKeyOf<TestRuntime>, DepositEntryOf<TestRuntime>)>,
	) -> Self {
		self.1 = deposits;
		self
	}

	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let mut ext = sp_io::TestExternalities::default();

		ext.execute_with(|| {
			for (account_id, amount) in self.0 {
				Balances::make_free_balance_be(&account_id, amount);
			}

			for (namespace, key, entry) in self.1 {
				// Fund each account with ED + deposit amount
				Balances::make_free_balance_be(&entry.deposit.owner, 500 + entry.deposit.amount);
				Pallet::<TestRuntime>::add_deposit(namespace, key, entry).unwrap();
			}
		});

		ext
	}

	pub(crate) fn build_and_execute_with_sanity_tests(self, run: impl FnOnce()) {
		let mut ext = self.build();
		ext.execute_with(|| {
			run();
			crate::try_state::try_state::<TestRuntime>(System::block_number()).unwrap();
		});
	}

	#[cfg(feature = "runtime-benchmarks")]
	pub(crate) fn build_with_keystore(self) -> sp_io::TestExternalities {
		let mut ext = self.build();
		let keystore = sp_keystore::testing::MemoryKeystore::new();
		ext.register_extension(sp_keystore::KeystoreExt(sp_std::sync::Arc::new(keystore)));
		ext
	}
}
