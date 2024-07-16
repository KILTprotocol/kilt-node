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

use frame_support::{construct_runtime, traits::Everything};
use frame_system::{mocking::MockBlock, EnsureSigned, RawOrigin};
use pallet_dip_provider::{DefaultIdentityCommitmentGenerator, DefaultIdentityProvider, IdentityCommitmentVersion};
use sp_core::{ConstU128, ConstU32};
use sp_runtime::traits::IdentityLookup;

use crate::{
	constants::{deposit_storage::MAX_DEPOSIT_PALLET_KEY_LENGTH, KILT},
	dip::deposit::{DepositHooks, DepositNamespace},
	AccountId, Balance, BlockHashCount, BlockLength, BlockWeights, Hash, Hasher, Nonce,
};

construct_runtime!(
	pub struct TestRuntime {
		System: frame_system,
		Balances: pallet_balances,
		DipProvider: pallet_dip_provider,
		StorageDepositPallet: pallet_deposit_storage,
	}
);

pub(crate) const SUBJECT: AccountId = AccountId::new([100u8; 32]);
pub(crate) const SUBMITTER: AccountId = AccountId::new([200u8; 32]);

impl frame_system::Config for TestRuntime {
	type AccountData = pallet_balances::AccountData<Balance>;
	type AccountId = AccountId;
	type BaseCallFilter = Everything;
	type Block = MockBlock<TestRuntime>;
	type BlockHashCount = BlockHashCount;
	type BlockLength = BlockLength;
	type BlockWeights = BlockWeights;
	type DbWeight = ();
	type Hash = Hash;
	type Hashing = Hasher;
	type Lookup = IdentityLookup<Self::AccountId>;
	type MaxConsumers = ConstU32<16>;
	type Nonce = Nonce;
	type OnKilledAccount = ();
	type OnNewAccount = ();
	type OnSetCode = ();
	type PalletInfo = PalletInfo;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeTask = ();
	type SS58Prefix = ();
	type SystemWeightInfo = ();
	type Version = ();
}

impl pallet_balances::Config for TestRuntime {
	type RuntimeFreezeReason = RuntimeFreezeReason;
	type FreezeIdentifier = RuntimeFreezeReason;
	type RuntimeHoldReason = RuntimeHoldReason;
	type MaxFreezes = ConstU32<10>;
	type Balance = Balance;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = ConstU128<KILT>;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ConstU32<10>;
	type MaxReserves = ConstU32<10>;
	type ReserveIdentifier = [u8; 8];
}

impl pallet_dip_provider::Config for TestRuntime {
	type CommitOrigin = AccountId;
	type CommitOriginCheck = EnsureSigned<AccountId>;
	type Identifier = AccountId;
	type IdentityCommitmentGenerator = DefaultIdentityCommitmentGenerator<u32>;
	type IdentityProvider = DefaultIdentityProvider<u32>;
	type ProviderHooks = ();
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
}

impl pallet_deposit_storage::Config for TestRuntime {
	type CheckOrigin = EnsureSigned<Self::AccountId>;
	type Currency = Balances;
	type DepositHooks = DepositHooks;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeHoldReason = RuntimeHoldReason;
	type MaxKeyLength = ConstU32<MAX_DEPOSIT_PALLET_KEY_LENGTH>;
	type Namespace = DepositNamespace;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHooks = ();
	type WeightInfo = ();
}

#[derive(Default)]
pub(crate) struct ExtBuilder(Vec<(AccountId, IdentityCommitmentVersion, AccountId)>);

impl ExtBuilder {
	pub(crate) fn with_commitments(
		mut self,
		commitments: Vec<(AccountId, IdentityCommitmentVersion, AccountId)>,
	) -> Self {
		self.0 = commitments;
		self
	}

	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let mut ext = sp_io::TestExternalities::default();

		ext.execute_with(|| {
			for (subject, version, submitter) in self.0 {
				DipProvider::commit_identity(RawOrigin::Signed(submitter).into(), subject, Some(version)).unwrap();
			}
		});

		ext
	}
}
