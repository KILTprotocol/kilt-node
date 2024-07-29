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
	traits::{
		fungible::{Inspect as InspectFungible, Mutate as MutateFungible},
		Everything,
	},
};
use frame_system::{mocking::MockBlock, EnsureRoot, EnsureSigned};
use pallet_balances::AccountData;
use sp_core::{ConstU16, ConstU32, ConstU64, H256};
use sp_runtime::{
	traits::{BlakeTwo256, CheckedConversion, IdentityLookup},
	AccountId32,
};

use crate::{NewSwitchPairInfoOf, Pallet, SwitchPairInfoOf};

construct_runtime!(
	pub enum MockRuntime {
		System: frame_system,
		Balances: pallet_balances,
		Assetswitch: crate
	}
);

impl frame_system::Config for MockRuntime {
	type AccountData = AccountData<u64>;
	type AccountId = AccountId32;
	type BaseCallFilter = Everything;
	type Block = MockBlock<MockRuntime>;
	type BlockHashCount = ConstU64<0>;
	type BlockLength = ();
	type BlockWeights = ();
	type DbWeight = ();
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type Lookup = IdentityLookup<Self::AccountId>;
	type MaxConsumers = ConstU32<1>;
	type Nonce = u64;
	type OnKilledAccount = ();
	type OnNewAccount = ();
	type OnSetCode = ();
	type PalletInfo = PalletInfo;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeTask = ();
	type SS58Prefix = ConstU16<0>;
	type SystemWeightInfo = ();
	type Version = ();
}

impl pallet_balances::Config for MockRuntime {
	type AccountStore = System;
	type Balance = u64;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU64<1>;
	type FreezeIdentifier = ();
	type MaxFreezes = ConstU32<0>;
	type MaxLocks = ConstU32<0>;
	type MaxReserves = ConstU32<0>;
	type ReserveIdentifier = ();
	type RuntimeEvent = RuntimeEvent;
	type RuntimeFreezeReason = ();
	type RuntimeHoldReason = ();
	type WeightInfo = ();
}

impl crate::Config for MockRuntime {
	type AccountIdConverter = ();
	type AssetTransactor = ();
	type FeeOrigin = EnsureRoot<Self::AccountId>;
	type LocalCurrency = Balances;
	type PauseOrigin = EnsureRoot<Self::AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type SubmitterOrigin = EnsureSigned<Self::AccountId>;
	type SwitchHooks = ();
	type SwitchOrigin = EnsureRoot<Self::AccountId>;
	type XcmRouter = ();
	type WeightInfo = ();

	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
}

#[derive(Default)]
pub(super) struct ExtBuilder(Option<NewSwitchPairInfoOf<MockRuntime>>);

impl ExtBuilder {
	pub(super) fn with_switch_pair_info(mut self, switch_pair_info: NewSwitchPairInfoOf<MockRuntime>) -> Self {
		self.0 = Some(switch_pair_info);
		self
	}

	pub(super) fn build(self) -> sp_io::TestExternalities {
		let _ = env_logger::try_init();
		let mut ext = sp_io::TestExternalities::default();

		ext.execute_with(|| {
			System::set_block_number(1);

			if let Some(switch_pair_info) = self.0 {
				let switch_pair_info = SwitchPairInfoOf::<MockRuntime>::from_input_unchecked(switch_pair_info);

				// Set pool balance to local ED + circulating supply, to maintain
				// invariants and make them verifiable.
				let local_ed = <Balances as InspectFungible<AccountId32>>::minimum_balance();
				<Balances as MutateFungible<AccountId32>>::mint_into(
					&switch_pair_info.pool_account,
					local_ed + u64::checked_from(switch_pair_info.remote_asset_circulating_supply).unwrap(),
				)
				.unwrap();
				Pallet::<MockRuntime>::set_switch_pair_bypass_checks(
					switch_pair_info.remote_asset_total_supply,
					switch_pair_info.remote_asset_id,
					switch_pair_info.remote_asset_circulating_supply,
					switch_pair_info.remote_reserve_location,
					switch_pair_info.remote_asset_ed,
					switch_pair_info.remote_xcm_fee,
					switch_pair_info.pool_account,
				);
				Pallet::<MockRuntime>::set_switch_pair_status(switch_pair_info.status).unwrap();
			}

			System::reset_events()
		});

		ext
	}

	pub(super) fn build_and_execute_with_sanity_tests(self, run: impl FnOnce()) {
		let mut ext = self.build();
		ext.execute_with(|| {
			run();
			crate::try_state::do_try_state::<MockRuntime, _>(System::block_number()).unwrap();
		});
	}
}
