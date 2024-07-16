// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2024 BOTLabs GmbH

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
	construct_runtime,
	sp_runtime::{
		testing::H256,
		traits::{BlakeTwo256, IdentityLookup},
		AccountId32,
	},
	traits::{ConstU128, ConstU16, ConstU32, ConstU64, Currency, Everything, Get},
};
use frame_system::{mocking::MockBlock, EnsureSigned};
use pallet_dip_provider::{DefaultIdentityCommitmentGenerator, DefaultIdentityProvider, IdentityCommitmentVersion};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

use crate::{
	self as storage_deposit_pallet, DepositEntryOf, DepositKeyOf, FixedDepositCollectorViaDepositsPallet, Pallet,
};

pub(crate) type Balance = u128;

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, PartialEq, Eq, RuntimeDebug, Default)]
pub enum DepositNamespaces {
	#[default]
	ExampleNamespace,
}

impl Get<DepositNamespaces> for DepositNamespaces {
	fn get() -> DepositNamespaces {
		Self::ExampleNamespace
	}
}

construct_runtime!(
	pub struct TestRuntime {
		System: frame_system,
		Balances: pallet_balances,
		DipProvider: pallet_dip_provider,
		StorageDepositPallet: storage_deposit_pallet,
	}
);

pub(crate) const SUBJECT: AccountId32 = AccountId32::new([100u8; 32]);
pub(crate) const SUBMITTER: AccountId32 = AccountId32::new([200u8; 32]);

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
	type RuntimeHoldReason = RuntimeHoldReason;
	type MaxFreezes = ConstU32<50>;
	type Balance = Balance;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ConstU128<500>;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ConstU32<50>;
	type MaxReserves = ConstU32<50>;
	type ReserveIdentifier = [u8; 8];
}

pub(crate) type DepositCollectorHook<Runtime> = FixedDepositCollectorViaDepositsPallet<
	DepositNamespaces,
	ConstU128<1_000>,
	(
		<Runtime as pallet_dip_provider::Config>::Identifier,
		AccountId32,
		IdentityCommitmentVersion,
	),
>;

impl pallet_dip_provider::Config for TestRuntime {
	type CommitOrigin = AccountId32;
	type CommitOriginCheck = EnsureSigned<AccountId32>;
	type Identifier = AccountId32;
	type IdentityCommitmentGenerator = DefaultIdentityCommitmentGenerator<u32>;
	type IdentityProvider = DefaultIdentityProvider<u32>;
	type ProviderHooks = DepositCollectorHook<Self>;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
}

impl crate::Config for TestRuntime {
	type CheckOrigin = EnsureSigned<Self::AccountId>;
	type Currency = Balances;
	type DepositHooks = ();
	type RuntimeEvent = RuntimeEvent;
	type RuntimeHoldReason = RuntimeHoldReason;
	type MaxKeyLength = ConstU32<256>;
	type Namespace = DepositNamespaces;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHooks = ();
	type WeightInfo = ();
}

#[derive(Default)]
pub(crate) struct ExtBuilder(
	Vec<(AccountId32, Balance)>,
	Vec<(DepositKeyOf<TestRuntime>, DepositEntryOf<TestRuntime>)>,
);

impl ExtBuilder {
	pub(crate) fn with_balances(mut self, balances: Vec<(AccountId32, Balance)>) -> Self {
		self.0 = balances;
		self
	}

	pub(crate) fn with_deposits(
		mut self,
		deposits: Vec<(DepositKeyOf<TestRuntime>, DepositEntryOf<TestRuntime>)>,
	) -> Self {
		self.1 = deposits;
		self
	}

	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let mut ext = sp_io::TestExternalities::default();

		ext.execute_with(|| {
			for (account, balance) in self.0 {
				Balances::make_free_balance_be(&account, balance);
			}

			for (deposit_key, deposit_entry) in self.1 {
				// Add existential deposit + deposit amount.
				Balances::make_free_balance_be(&deposit_entry.deposit.owner, 500 + deposit_entry.deposit.amount);
				Pallet::<TestRuntime>::add_deposit(DepositNamespaces::get(), deposit_key, deposit_entry).unwrap();
			}
		});

		ext
	}
}
