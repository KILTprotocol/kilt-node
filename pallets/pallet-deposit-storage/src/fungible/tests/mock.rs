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
	construct_runtime,
	sp_runtime::{
		testing::H256,
		traits::{BlakeTwo256, IdentityLookup},
		AccountId32,
	},
	traits::{ConstU128, ConstU16, ConstU32, ConstU64, Currency, Everything, VariantCount},
};
use frame_system::{mocking::MockBlock, EnsureSigned};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

use crate::{self as storage_deposit_pallet, HoldReason};

construct_runtime!(
	pub struct TestRuntime {
		System: frame_system,
		StorageDepositPallet: storage_deposit_pallet,
		Balances: pallet_balances,
	}
);

pub(crate) type Balance = u128;

// Required to test more than a single hold reason without introducing any new
// pallets.
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum TestRuntimeHoldReason {
	Deposit,
	Else,
}

impl From<HoldReason> for TestRuntimeHoldReason {
	fn from(_value: HoldReason) -> Self {
		Self::Deposit
	}
}

// This value is used by the `Balances` pallet to create the limit for the
// `BoundedVec` of holds. By returning `1` here, we make it possible to hit to
// hold limit.
impl VariantCount for TestRuntimeHoldReason {
	const VARIANT_COUNT: u32 = 1;
}

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
}

impl pallet_balances::Config for TestRuntime {
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type FreezeIdentifier = RuntimeFreezeReason;
	type RuntimeHoldReason = TestRuntimeHoldReason;
	type MaxFreezes = ConstU32<1>;
	type Balance = Balance;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ConstU128<1>;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ConstU32<1>;
	type MaxReserves = ConstU32<1>;
	type ReserveIdentifier = [u8; 8];
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, PartialEq, Eq, RuntimeDebug, Default)]
pub enum DepositNamespace {
	#[default]
	ExampleNamespace,
}

impl crate::Config for TestRuntime {
	type CheckOrigin = EnsureSigned<Self::AccountId>;
	type Currency = Balances;
	type DepositHooks = ();
	type RuntimeEvent = RuntimeEvent;
	type RuntimeHoldReason = TestRuntimeHoldReason;
	type MaxKeyLength = ConstU32<256>;
	type Namespace = DepositNamespace;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHooks = ();
	type WeightInfo = ();
}

pub(crate) const OWNER: AccountId32 = AccountId32::new([100u8; 32]);
pub(crate) const OTHER_ACCOUNT: AccountId32 = AccountId32::new([101u8; 32]);

#[derive(Default)]
pub(crate) struct ExtBuilder(Vec<(AccountId32, Balance)>);

impl ExtBuilder {
	pub(crate) fn with_balances(mut self, balances: Vec<(AccountId32, Balance)>) -> Self {
		self.0 = balances;
		self
	}

	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let mut ext = sp_io::TestExternalities::default();

		ext.execute_with(|| {
			for (account_id, amount) in self.0 {
				Balances::make_free_balance_be(&account_id, amount);
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
}
