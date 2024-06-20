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
		fungible::{Mutate, MutateFreeze, MutateHold},
		Everything,
	},
};
use frame_system::{mocking::MockBlock, EnsureRoot, EnsureSigned};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::{ConstU16, ConstU32, ConstU64, H256};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	AccountId32,
};
use xcm::v3::{
	AssetId, Error as XcmError, Fungibility,
	Junction::{AccountId32 as AccountId32Junction, AccountKey20, GlobalConsensus, Parachain},
	Junctions::{Here, X1, X2},
	MultiAsset, MultiLocation, NetworkId, XcmContext,
};
use xcm_executor::{traits::TransactAsset, Assets};

use crate::{Pallet, SwapPair, SwapPairInfoOf};

construct_runtime!(
	pub enum MockRuntime {
		System: frame_system,
		Balances: pallet_balances,
		AssetSwap: crate
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

static mut BALANCES: Vec<(MultiLocation, u128)> = vec![];
pub struct MockFungibleAssetTransactor;

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
		unsafe {
			let from_entry = BALANCES
				.iter_mut()
				.find(|e| e.0 == *who)
				.ok_or(XcmError::FailedToTransactAsset("No balance found for user."))?;
			let new_from_balance = from_entry
				.1
				.checked_sub(amount)
				.ok_or(XcmError::FailedToTransactAsset("No enough balance for user."))?;
			from_entry.1 = new_from_balance;
			Ok::<_, XcmError>(())
		}?;
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
		unsafe {
			let to_entry = BALANCES.iter_mut().find(|e| e.0 == *who);
			if let Some(to_entry) = to_entry {
				let new_to_balance = to_entry
					.1
					.checked_add(amount)
					.ok_or(XcmError::FailedToTransactAsset("Balance overflow for destination."))?;
				to_entry.1 = new_to_balance;
			} else {
				BALANCES.push((*who, amount));
			}
			Ok::<_, XcmError>(())
		}?;
		Ok(())
	}
}

impl crate::Config for MockRuntime {
	const PALLET_ID: [u8; 8] = *b"eKILT/AH";

	type AccountIdConverter = ();
	type AssetTransactor = MockFungibleAssetTransactor;
	type Currency = Balances;
	type FeeOrigin = EnsureRoot<Self::AccountId>;
	type PauseOrigin = EnsureRoot<Self::AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type SubmitterOrigin = EnsureSigned<Self::AccountId>;
	type SwapOrigin = EnsureRoot<Self::AccountId>;
	type XcmRouter = ();
}

#[derive(Default)]
pub(crate) struct ExtBuilder(
	Option<SwapPairInfoOf<MockRuntime>>,
	Vec<(AccountId32, u64, u64, u64)>,
	Vec<(AccountId32, MultiAsset)>,
);

impl ExtBuilder {
	pub(crate) fn with_swap_pair_info(mut self, swap_pair_info: SwapPairInfoOf<MockRuntime>) -> Self {
		self.0 = Some(swap_pair_info);
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
		let mut ext = sp_io::TestExternalities::default();

		ext.execute_with(|| {
			unsafe {
				BALANCES = vec![];
			}
			System::set_block_number(1);

			if let Some(swap_pair_info) = self.0 {
				let SwapPairInfoOf::<MockRuntime> {
					pool_account,
					remote_asset_balance,
					remote_asset_id,
					remote_reserve_location,
					remote_fee,
					status,
				} = swap_pair_info;
				Pallet::<MockRuntime>::set_swap_pair_bypass_checks(
					remote_reserve_location,
					remote_asset_id,
					remote_fee,
					remote_asset_balance,
					pool_account,
				);
				SwapPair::<MockRuntime>::mutate(|entry| entry.as_mut().unwrap().status = status);
			}
			for (account, free, frozen, locked) in self.1 {
				<Balances as Mutate<AccountId32>>::set_balance(&account, free);
				<Balances as MutateFreeze<AccountId32>>::set_freeze(b"1", &account, frozen)
					.expect("Failed to freeze balance on account.");
				<Balances as MutateHold<AccountId32>>::hold(&MockRuntimeHoldReason::default(), &account, locked)
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
