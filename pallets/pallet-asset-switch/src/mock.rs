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
	construct_runtime, storage_alias,
	traits::{
		fungible::{Mutate, MutateFreeze, MutateHold},
		Everything, VariantCount,
	},
	Twox64Concat,
};
use frame_system::{mocking::MockBlock, EnsureRoot, EnsureSigned};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::{ConstU16, ConstU32, ConstU64, H256};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	AccountId32,
};
use sp_std::sync::Arc;
use xcm::v4::{
	Asset, AssetId, Error as XcmError, Fungibility,
	Junction::{AccountId32 as AccountId32Junction, AccountKey20, GlobalConsensus, Parachain},
	Junctions::{Here, X1, X2},
	Location, NetworkId, SendError, SendResult, SendXcm, Xcm, XcmContext, XcmHash,
};
use xcm_executor::{traits::TransactAsset, AssetsInHolding};

use crate::{xcm::convert::AccountId32ToAccountId32JunctionConverter, Config, NewSwitchPairInfoOf, Pallet};

construct_runtime!(
	pub enum MockRuntime {
		System: frame_system,
		Balances: pallet_balances,
		Assetswitch: crate
	}
);

impl frame_system::Config for MockRuntime {
	type AccountData = pallet_balances::AccountData<u64>;
	type AccountId = AccountId32;
	type BaseCallFilter = Everything;
	type Block = MockBlock<MockRuntime>;
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
	type RuntimeTask = ();
	type RuntimeOrigin = RuntimeOrigin;
	type SS58Prefix = ConstU16<1>;
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
	type ExistentialDeposit = ConstU64<1>;
	type FreezeIdentifier = [u8; 1];
	type MaxFreezes = ConstU32<10>;
	type MaxLocks = ConstU32<10>;
	type MaxReserves = ConstU32<10>;
	type ReserveIdentifier = [u8; 1];
	type RuntimeEvent = RuntimeEvent;
	type RuntimeHoldReason = MockRuntimeHoldReason;
	type RuntimeFreezeReason = ();
	type WeightInfo = ();
}

// Used to temporarily store balances used in the mock.
#[storage_alias]
type BalancesStorage<T: Config> = StorageMap<Pallet<T>, Twox64Concat, Location, u128>;

pub struct MockFungibleAssetTransactor;

impl MockFungibleAssetTransactor {
	pub(super) fn get_balance_for(account: &Location) -> u128 {
		BalancesStorage::<MockRuntime>::get(account).unwrap_or_default()
	}
}

impl TransactAsset for MockFungibleAssetTransactor {
	fn withdraw_asset(
		what: &Asset,
		who: &Location,
		_maybe_context: Option<&XcmContext>,
	) -> Result<AssetsInHolding, XcmError> {
		let Asset {
			fun: Fungibility::Fungible(amount),
			..
		} = *what
		else {
			return Err(XcmError::FailedToTransactAsset("Only fungible assets supported."));
		};
		BalancesStorage::<MockRuntime>::try_mutate(who, |entry| {
			let balance = entry
				.as_mut()
				.ok_or(XcmError::FailedToTransactAsset("No balance found for user."))?;
			let new_balance = balance
				.checked_sub(amount)
				.ok_or(XcmError::FailedToTransactAsset("No enough balance for user."))?;
			*balance = new_balance;
			Ok::<_, XcmError>(())
		})?;
		Ok(vec![what.clone()].into())
	}

	fn deposit_asset(what: &Asset, who: &Location, _context: Option<&XcmContext>) -> Result<(), XcmError> {
		let Asset {
			fun: Fungibility::Fungible(amount),
			..
		} = *what
		else {
			return Err(XcmError::FailedToTransactAsset("Only fungible assets supported."));
		};
		BalancesStorage::<MockRuntime>::mutate(who, |entry| {
			let new_balance = entry
				.unwrap_or_default()
				.checked_add(amount)
				.ok_or(XcmError::FailedToTransactAsset("Balance overflow for destination."))?;
			*entry = Some(new_balance);
			Ok::<_, XcmError>(())
		})?;
		Ok(())
	}
}

pub struct AlwaysSuccessfulXcmRouter;

impl SendXcm for AlwaysSuccessfulXcmRouter {
	type Ticket = ();

	fn validate(_destination: &mut Option<Location>, _message: &mut Option<Xcm<()>>) -> SendResult<Self::Ticket> {
		Ok(((), vec![].into()))
	}

	fn deliver(_ticket: Self::Ticket) -> Result<XcmHash, SendError> {
		Ok(XcmHash::default())
	}
}

impl crate::Config for MockRuntime {
	type AccountIdConverter = AccountId32ToAccountId32JunctionConverter;
	type AssetTransactor = MockFungibleAssetTransactor;
	type FeeOrigin = EnsureRoot<Self::AccountId>;
	type LocalCurrency = Balances;
	type PauseOrigin = EnsureRoot<Self::AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type SubmitterOrigin = EnsureSigned<Self::AccountId>;
	type SwitchHooks = ();
	type SwitchOrigin = EnsureRoot<Self::AccountId>;
	type WeightInfo = ();
	type XcmRouter = AlwaysSuccessfulXcmRouter;

	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
}

#[derive(Default)]
pub(super) struct ExtBuilder(
	Option<NewSwitchPairInfoOf<MockRuntime>>,
	Vec<(AccountId32, u64, u64, u64)>,
	Vec<(AccountId32, Asset)>,
);

pub(super) const FREEZE_REASON: [u8; 1] = *b"1";
pub(super) const HOLD_REASON: MockRuntimeHoldReason = MockRuntimeHoldReason {};

impl ExtBuilder {
	pub(super) fn with_switch_pair_info(mut self, switch_pair_info: NewSwitchPairInfoOf<MockRuntime>) -> Self {
		self.0 = Some(switch_pair_info);
		self
	}

	pub(super) fn with_balances(mut self, balances: Vec<(AccountId32, u64, u64, u64)>) -> Self {
		self.1 = balances;
		self
	}

	pub(super) fn with_fungibles(mut self, fungibles: Vec<(AccountId32, Asset)>) -> Self {
		self.2 = fungibles;
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
			for (account, free, frozen, held) in self.1 {
				<Balances as Mutate<AccountId32>>::set_balance(&account, free);
				<Balances as MutateFreeze<AccountId32>>::set_freeze(&FREEZE_REASON, &account, frozen)
					.expect("Failed to freeze balance on account.");
				<Balances as MutateHold<AccountId32>>::hold(&HOLD_REASON, &account, held)
					.expect("Failed to hold balance on account.");
			}

			for (account, asset) in self.2 {
				MockFungibleAssetTransactor::deposit_asset(
					&asset,
					&Location {
						parents: 0,
						interior: X1([AccountId32Junction {
							network: None,
							id: account.clone().into(),
						}]
						.into()),
					},
					Some(&XcmContext::with_message_id([0; 32])),
				)
				.unwrap_or_else(|_| {
					panic!(
						"Should not fail to deposit asset {:?} into account {:?}",
						asset, account
					)
				});
			}

			// Some setup operations generate events which interfere with our assertions.
			System::reset_events()
		});

		ext
	}

	// Run the specified closure and test the storage invariants afterwards.
	pub(crate) fn build_and_execute_with_sanity_tests(self, run: impl FnOnce()) {
		let mut ext = self.build();
		ext.execute_with(|| {
			run();
			crate::try_state::do_try_state::<MockRuntime, _>(System::block_number()).unwrap();
		});
	}

	#[cfg(all(feature = "runtime-benchmarks", test))]
	pub(crate) fn build_with_keystore(self) -> sp_io::TestExternalities {
		let mut ext = self.build();
		let keystore = sp_keystore::testing::MemoryKeystore::new();
		ext.register_extension(sp_keystore::KeystoreExt(sp_std::sync::Arc::new(keystore)));
		ext
	}
}

pub(super) const XCM_ASSET_FEE: Asset = Asset {
	id: PARENT_NATIVE_CURRENCY,
	fun: Fungibility::Fungible(1_000),
};
const PARENT_NATIVE_CURRENCY: AssetId = AssetId(PARENT_LOCATION);
const PARENT_LOCATION: Location = Location {
	parents: 1,
	interior: Here,
};

pub(super) fn get_asset_hub_location() -> Location {
	Location {
		parents: 1,
		interior: X1(Arc::new([Parachain(1_000)])),
	}
}

pub(super) fn get_remote_erc20_asset_id() -> AssetId {
	AssetId(Location {
		parents: 2,
		interior: X2([
			GlobalConsensus(NetworkId::Ethereum { chain_id: 1 }),
			AccountKey20 {
				network: None,
				key: *b"!!test_eth_address!!",
			},
		]
		.into()),
	})
}
