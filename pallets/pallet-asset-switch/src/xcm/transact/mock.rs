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
	pallet_prelude::TypeInfo,
	traits::{
		fungible::{
			freeze::Mutate as MutateFreeze, hold::Mutate as MutateHold, Inspect as InspectFungible,
			Mutate as MutateFungible,
		},
		Everything, VariantCount,
	},
};
use frame_system::{mocking::MockBlock, EnsureRoot, EnsureSigned};
use pallet_balances::AccountData;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use sp_core::{ConstU16, ConstU32, ConstU64, H256};
use sp_runtime::{
	traits::{BlakeTwo256, CheckedConversion, IdentityLookup},
	AccountId32,
};
use xcm::v4::Location;
use xcm_executor::traits::ConvertLocation;

use crate::{NewSwitchPairInfoOf, Pallet, SwitchPair, SwitchPairInfoOf};

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
	type RuntimeTask = ();
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

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, MaxEncodedLen, Encode, Decode, Debug, TypeInfo, Default)]
pub struct MockRuntimeHoldReason;

impl VariantCount for MockRuntimeHoldReason {
	const VARIANT_COUNT: u32 = 1;
}

impl pallet_balances::Config for MockRuntime {
	type AccountStore = System;
	type Balance = u64;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU64<2>;
	type FreezeIdentifier = [u8; 4];
	type MaxFreezes = ConstU32<1>;
	type MaxLocks = ConstU32<0>;
	type MaxReserves = ConstU32<0>;
	type ReserveIdentifier = ();
	type RuntimeEvent = RuntimeEvent;
	type RuntimeHoldReason = MockRuntimeHoldReason;
	type RuntimeFreezeReason = ();
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

pub(super) const SUCCESSFUL_ACCOUNT_ID: AccountId32 = AccountId32::new([100; 32]);
pub(super) struct SuccessfulAccountIdConverter;

impl ConvertLocation<AccountId32> for SuccessfulAccountIdConverter {
	fn convert_location(_location: &Location) -> Option<AccountId32> {
		Some(SUCCESSFUL_ACCOUNT_ID)
	}
}

pub(super) struct FailingAccountIdConverter;

impl ConvertLocation<AccountId32> for FailingAccountIdConverter {
	fn convert_location(_location: &Location) -> Option<AccountId32> {
		None
	}
}

#[derive(Default)]
pub(super) struct ExtBuilder(
	Option<NewSwitchPairInfoOf<MockRuntime>>,
	Vec<(AccountId32, u64, u64, u64)>,
);

impl ExtBuilder {
	pub(super) fn with_switch_pair_info(mut self, switch_pair_info: NewSwitchPairInfoOf<MockRuntime>) -> Self {
		self.0 = Some(switch_pair_info);
		self
	}

	pub(super) fn with_additional_balance_entries(mut self, balances: Vec<(AccountId32, u64, u64, u64)>) -> Self {
		self.1 = balances;
		self
	}

	pub(super) fn build(self) -> sp_io::TestExternalities {
		let _ = env_logger::try_init();
		let mut ext = sp_io::TestExternalities::default();

		ext.execute_with(|| {
			System::set_block_number(1);

			let local_ed = <Balances as InspectFungible<AccountId32>>::minimum_balance();
			if let Some(switch_pair_info) = &self.0 {
				let switch_pair_info = SwitchPairInfoOf::<MockRuntime>::from_input_unchecked(switch_pair_info.clone());

				// Set pool balance to local ED + circulating supply, to maintain
				// invariants and make them verifiable.
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
			for (account, free, held, frozen) in self.1 {
				<Balances as MutateFungible<AccountId32>>::mint_into(&account, free).unwrap();
				<Balances as MutateHold<AccountId32>>::hold(&MockRuntimeHoldReason {}, &account, held).unwrap();
				<Balances as MutateFreeze<AccountId32>>::set_freeze(b"test", &account, frozen).unwrap();

				// If the specified account is the pool account, remove from the registered
				// circulating supply the amount of tokens that have been marked as held or
				// frozen, to maintain the invariant.
				if Some(account)
					== self
						.0
						.as_ref()
						.map(|new_switch_info| new_switch_info.clone().pool_account)
				{
					// ED can be frozen, so we only need to considered only the ones that go BEYOND
					// the ED.
					let freezes_more_than_ed = frozen.saturating_sub(local_ed);
					SwitchPair::<MockRuntime, _>::mutate(|switch_pair| {
						if let Some(switch_pair) = switch_pair.as_mut() {
							// Calculate all held tokens as not available in the pool, hence not available
							// as circulating supply at destination.
							switch_pair.remote_asset_circulating_supply = switch_pair
								.remote_asset_circulating_supply
								.checked_sub(held as u128)
								.unwrap();
							// Calculate frozen tokens beyond the Ed as not available in the pool, hence not
							// available as circulating supply at destination.
							switch_pair.remote_asset_circulating_supply = switch_pair
								.remote_asset_circulating_supply
								.checked_sub(freezes_more_than_ed as u128)
								.unwrap();
						}
					});
				}
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
