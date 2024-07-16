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
		Everything,
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
use xcm::{
	v3::{
		AssetId, Error as XcmError, Fungibility,
		Junction::{AccountId32 as AccountId32Junction, AccountKey20, GlobalConsensus, Parachain},
		Junctions::{Here, X1, X2},
		MultiAsset, MultiLocation, NetworkId, SendError, SendResult, SendXcm, Xcm, XcmContext, XcmHash,
	},
	VersionedAssetId, VersionedMultiAsset, VersionedMultiLocation,
};
use xcm_executor::{traits::TransactAsset, Assets};

use crate::{
	switch::SwitchPairStatus, xcm::convert::AccountId32ToAccountId32JunctionConverter, Config, Pallet, SwitchPair,
	SwitchPairInfoOf,
};

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
	type RuntimeOrigin = RuntimeOrigin;
	type SS58Prefix = ConstU16<1>;
	type SystemWeightInfo = ();
	type Version = ();
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, MaxEncodedLen, Encode, Decode, Debug, TypeInfo, Default)]
pub struct MockRuntimeHoldReason;

impl pallet_balances::Config for MockRuntime {
	type AccountStore = System;
	type Balance = u64;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU64<1>;
	type FreezeIdentifier = [u8; 1];
	type MaxFreezes = ConstU32<10>;
	type MaxHolds = ConstU32<10>;
	type MaxLocks = ConstU32<10>;
	type MaxReserves = ConstU32<10>;
	type ReserveIdentifier = [u8; 1];
	type RuntimeEvent = RuntimeEvent;
	type RuntimeHoldReason = MockRuntimeHoldReason;
	type WeightInfo = ();
}

// Used to temporarily store balances used in the mock.
#[storage_alias]
type BalancesStorage<T: Config> = StorageMap<Pallet<T>, Twox64Concat, MultiLocation, u128>;

pub struct MockFungibleAssetTransactor;

impl MockFungibleAssetTransactor {
	pub(crate) fn get_balance_for(account: &MultiLocation) -> u128 {
		BalancesStorage::<MockRuntime>::get(account).unwrap_or_default()
	}
}

impl TransactAsset for MockFungibleAssetTransactor {
	fn withdraw_asset(
		what: &MultiAsset,
		who: &MultiLocation,
		_maybe_context: Option<&XcmContext>,
	) -> Result<Assets, XcmError> {
		let MultiAsset {
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

	fn deposit_asset(what: &MultiAsset, who: &MultiLocation, _context: &XcmContext) -> Result<(), XcmError> {
		let MultiAsset {
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

	fn validate(_destination: &mut Option<MultiLocation>, _message: &mut Option<Xcm<()>>) -> SendResult<Self::Ticket> {
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
	type XcmRouter = AlwaysSuccessfulXcmRouter;
}

#[derive(Clone)]
pub(crate) struct NewSwitchPairInfo {
	pub(crate) circulating_supply: u128,
	pub(crate) pool_account: AccountId32,
	pub(crate) remote_asset_id: VersionedAssetId,
	pub(crate) remote_fee: VersionedMultiAsset,
	pub(crate) remote_reserve_location: VersionedMultiLocation,
	pub(crate) status: SwitchPairStatus,
	pub(crate) total_issuance: u128,
}

impl From<NewSwitchPairInfo> for SwitchPairInfoOf<MockRuntime> {
	fn from(new_switch_pair_info: NewSwitchPairInfo) -> Self {
		let remote_asset_balance = new_switch_pair_info.total_issuance - new_switch_pair_info.circulating_supply;
		Self {
			remote_asset_balance,
			pool_account: new_switch_pair_info.pool_account,
			remote_asset_id: new_switch_pair_info.remote_asset_id,
			remote_fee: new_switch_pair_info.remote_fee,
			remote_reserve_location: new_switch_pair_info.remote_reserve_location,
			status: new_switch_pair_info.status,
		}
	}
}

#[derive(Default)]
pub(crate) struct ExtBuilder(
	Option<NewSwitchPairInfo>,
	Vec<(AccountId32, u64, u64, u64)>,
	Vec<(AccountId32, MultiAsset)>,
);

pub(crate) const FREEZE_REASON: [u8; 1] = *b"1";
pub(crate) const HOLD_REASON: MockRuntimeHoldReason = MockRuntimeHoldReason {};

impl ExtBuilder {
	pub(crate) fn with_switch_pair_info(mut self, switch_pair_info: NewSwitchPairInfo) -> Self {
		self.0 = Some(switch_pair_info);
		self
	}

	pub(crate) fn with_balances(mut self, balances: Vec<(AccountId32, u64, u64, u64)>) -> Self {
		self.1 = balances;
		self
	}

	pub(crate) fn with_fungibles(mut self, fungibles: Vec<(AccountId32, MultiAsset)>) -> Self {
		self.2 = fungibles;
		self
	}

	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let _ = env_logger::try_init();
		let mut ext = sp_io::TestExternalities::default();

		ext.execute_with(|| {
			System::set_block_number(1);

			if let Some(switch_pair_info) = self.0 {
				Pallet::<MockRuntime>::set_switch_pair_bypass_checks(
					switch_pair_info.remote_reserve_location,
					switch_pair_info.remote_asset_id,
					switch_pair_info.remote_fee,
					switch_pair_info.total_issuance,
					switch_pair_info.circulating_supply,
					switch_pair_info.pool_account,
				);
				SwitchPair::<MockRuntime>::mutate(|entry| entry.as_mut().unwrap().status = switch_pair_info.status);
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
					&MultiLocation {
						parents: 0,
						interior: X1(AccountId32Junction {
							network: None,
							id: account.clone().into(),
						}),
					},
					&XcmContext::with_message_id([0; 32]),
				)
				.unwrap_or_else(|_| {
					panic!(
						"Should not fail to deposit asset {:?} into account {:?}",
						asset, account
					)
				});
			}
		});

		ext
	}

	#[cfg(all(feature = "runtime-benchmarks", test))]
	pub(crate) fn build_with_keystore(self) -> sp_io::TestExternalities {
		let mut ext = self.build();
		let keystore = sp_keystore::testing::MemoryKeystore::new();
		ext.register_extension(sp_keystore::KeystoreExt(sp_std::sync::Arc::new(keystore)));
		ext
	}
}

pub(crate) const XCM_ASSET_FEE: MultiAsset = MultiAsset {
	id: PARENT_NATIVE_CURRENCY,
	fun: Fungibility::Fungible(1_000),
};
const PARENT_NATIVE_CURRENCY: AssetId = AssetId::Concrete(PARENT_LOCATION);
const PARENT_LOCATION: MultiLocation = MultiLocation {
	parents: 1,
	interior: Here,
};

pub(crate) const ASSET_HUB_LOCATION: MultiLocation = MultiLocation {
	parents: 1,
	interior: X1(Parachain(1_000)),
};

pub(crate) const REMOTE_ERC20_ASSET_ID: AssetId = AssetId::Concrete(MultiLocation {
	parents: 2,
	interior: X2(
		GlobalConsensus(NetworkId::Ethereum { chain_id: 1 }),
		AccountKey20 {
			network: None,
			key: *b"!!test_eth_address!!",
		},
	),
});
