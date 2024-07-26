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

use cumulus_pallet_parachain_system::{ParachainSetCode, RelayNumberStrictlyIncreases};
use cumulus_primitives_core::{AggregateMessageOrigin, ParaId, PersistedValidationData};
use frame_support::{
	construct_runtime, parameter_types,
	sp_runtime::{
		testing::H256,
		traits::{BlakeTwo256, IdentityLookup},
		AccountId32,
	},
	storage_alias,
	traits::{ConstU16, ConstU32, ConstU64, EnqueueWithOrigin, Everything},
};
use frame_system::mocking::MockBlock;
use sp_runtime::BoundedVec;

use crate::{Config, Pallet};

construct_runtime!(
	pub struct TestRuntime {
		System: frame_system,
		ParachainSystem: cumulus_pallet_parachain_system,
		RelayStore: crate,
	}
);

impl frame_system::Config for TestRuntime {
	type AccountData = ();
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
	type OnSetCode = ParachainSetCode<TestRuntime>;
	type PalletInfo = PalletInfo;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeTask = ();
	type SS58Prefix = ConstU16<1>;
	type SystemWeightInfo = ();
	type Version = ();
}

parameter_types! {
	pub const ParachainId: ParaId = ParaId::new(2_000);
	pub const RelayOrigin: AggregateMessageOrigin = AggregateMessageOrigin::Parent;
}

impl cumulus_pallet_parachain_system::Config for TestRuntime {
	type CheckAssociatedRelayNumber = RelayNumberStrictlyIncreases;
	type OnSystemEvent = ();
	type OutboundXcmpMessageSource = ();
	type ReservedDmpWeight = ();
	type ReservedXcmpWeight = ();
	type RuntimeEvent = RuntimeEvent;
	type SelfParaId = ParachainId;
	type XcmpMessageHandler = ();
	type ConsensusHook = cumulus_pallet_parachain_system::ExpectParentIncluded;
	type WeightInfo = ();
	type DmpQueue = EnqueueWithOrigin<(), RelayOrigin>;
}

impl crate::Config for TestRuntime {
	type MaxRelayBlocksStored = ConstU32<5>;
	type WeightInfo = ();
}

// Alias to the ParachainSystem storage which cannot be modified directly.
#[storage_alias]
type ValidationData = StorageValue<ParachainSystem, PersistedValidationData>;

#[derive(Default)]
pub(crate) struct ExtBuilder(
	Option<(u32, H256)>,
	BoundedVec<(u32, H256), <TestRuntime as Config>::MaxRelayBlocksStored>,
);

impl ExtBuilder {
	pub(crate) fn with_new_relay_state_root(mut self, relay_root: (u32, H256)) -> Self {
		self.0 = Some(relay_root);
		self
	}

	pub(crate) fn with_stored_relay_roots(mut self, relay_roots: Vec<(u32, H256)>) -> Self {
		self.1 = relay_roots.try_into().unwrap();
		self
	}

	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let mut ext = sp_io::TestExternalities::default();
		ext.execute_with(|| {
			if let Some(new_relay_state_root) = self.0 {
				ValidationData::put(PersistedValidationData {
					relay_parent_number: new_relay_state_root.0,
					relay_parent_storage_root: new_relay_state_root.1,
					..Default::default()
				});
			}
			for (stored_relay_block_number, stored_relay_state_root) in self.1 {
				Pallet::<TestRuntime>::store_new_validation_data(PersistedValidationData {
					relay_parent_number: stored_relay_block_number,
					relay_parent_storage_root: stored_relay_state_root,
					..Default::default()
				});
			}
		});

		ext
	}

	#[cfg(feature = "runtime-benchmarks")]
	pub(crate) fn build_with_keystore(self) -> sp_io::TestExternalities {
		let mut ext = self.build();
		let keystore = sp_keystore::testing::MemoryKeystore::new();
		ext.register_extension(sp_keystore::KeystoreExt(sp_std::sync::Arc::new(keystore)));
		ext
	}
}
