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
		fungible::Dust,
		tokens::{
			fungible::{Inspect as InspectFungible, Mutate as MutateFungible, Unbalanced as UnbalancedFungible},
			DepositConsequence, Fortitude, Preservation, Provenance, WithdrawConsequence,
		},
		Everything,
	},
};
use frame_system::{mocking::MockBlock, EnsureRoot, EnsureSigned};
use sp_core::{ConstU16, ConstU32, ConstU64, H256};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	AccountId32, DispatchError,
};

use crate::{NewSwitchPairInfoOf, Pallet};

construct_runtime!(
	pub enum MockRuntime {
		System: frame_system,
		Assetswitch: crate
	}
);

impl frame_system::Config for MockRuntime {
	type AccountData = ();
	type AccountId = AccountId32;
	type RuntimeTask = ();
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
	type SS58Prefix = ConstU16<0>;
	type SystemWeightInfo = ();
	type Version = ();
}

// Currency is not used in this XCM component tests, so we mock the entire
// currency system.
pub struct MockCurrency;

impl MutateFungible<AccountId32> for MockCurrency {}

impl InspectFungible<AccountId32> for MockCurrency {
	type Balance = u64;

	fn active_issuance() -> Self::Balance {
		Self::Balance::default()
	}

	fn balance(_who: &AccountId32) -> Self::Balance {
		Self::Balance::default()
	}

	fn can_deposit(_who: &AccountId32, _amount: Self::Balance, _provenance: Provenance) -> DepositConsequence {
		DepositConsequence::Success
	}

	fn can_withdraw(_who: &AccountId32, _amount: Self::Balance) -> WithdrawConsequence<Self::Balance> {
		WithdrawConsequence::Success
	}

	fn minimum_balance() -> Self::Balance {
		Self::Balance::default()
	}

	fn reducible_balance(_who: &AccountId32, _preservation: Preservation, _force: Fortitude) -> Self::Balance {
		Self::Balance::default()
	}

	fn total_balance(_who: &AccountId32) -> Self::Balance {
		Self::Balance::default()
	}

	fn total_issuance() -> Self::Balance {
		Self::Balance::default()
	}
}

impl UnbalancedFungible<AccountId32> for MockCurrency {
	fn handle_dust(_dust: Dust<AccountId32, Self>) {}

	fn write_balance(_who: &AccountId32, _amount: Self::Balance) -> Result<Option<Self::Balance>, DispatchError> {
		Ok(Some(Self::Balance::default()))
	}

	fn set_total_issuance(_amount: Self::Balance) {}
}

impl crate::Config for MockRuntime {
	type AccountIdConverter = ();
	type AssetTransactor = ();
	type FeeOrigin = EnsureRoot<Self::AccountId>;
	type LocalCurrency = MockCurrency;
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
}
