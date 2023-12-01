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

use cumulus_pallet_parachain_system::{ParachainSetCode, RelayNumberStrictlyIncreases};
use cumulus_primitives_core::ParaId;
use frame_support::{
	construct_runtime, parameter_types,
	sp_runtime::{
		testing::H256,
		traits::{BlakeTwo256, IdentityLookup},
		AccountId32,
	},
	traits::{ConstU16, ConstU32, ConstU64, Everything},
};
use frame_system::mocking::MockBlock;

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
	type SS58Prefix = ConstU16<1>;
	type SystemWeightInfo = ();
	type Version = ();
}

parameter_types! {
	pub const ParachainId: ParaId = ParaId::new(2_000);
}

impl cumulus_pallet_parachain_system::Config for TestRuntime {
	type CheckAssociatedRelayNumber = RelayNumberStrictlyIncreases;
	type DmpMessageHandler = ();
	type OnSystemEvent = ();
	type OutboundXcmpMessageSource = ();
	type ReservedDmpWeight = ();
	type ReservedXcmpWeight = ();
	type RuntimeEvent = RuntimeEvent;
	type SelfParaId = ParachainId;
	type XcmpMessageHandler = ();
}

impl crate::Config for TestRuntime {
	type MaxRelayBlocksStored = ConstU32<5>;
	type WeightInfo = ();
}

#[derive(Default)]
pub(crate) struct ExtBuilder;

impl ExtBuilder {
	pub fn _build(self) -> sp_io::TestExternalities {
		sp_io::TestExternalities::default()
	}

	#[cfg(feature = "runtime-benchmarks")]
	pub fn build_with_keystore(self) -> sp_io::TestExternalities {
		let mut ext = self._build();
		let keystore = sp_keystore::testing::MemoryKeystore::new();
		ext.register_extension(sp_keystore::KeystoreExt(sp_std::sync::Arc::new(keystore)));
		ext
	}
}
