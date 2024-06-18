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
use frame_system::{mocking::MockBlock, EnsureRoot, EnsureSigned};
use sp_core::{ConstU16, ConstU32, ConstU64, H256};
use sp_runtime::{
	traits::{BlakeTwo256, IdentityLookup},
	AccountId32,
};

use crate::{Config, Pallet, SwapPairInfoOf};

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

impl pallet_balances::Config for MockRuntime {
	type AccountStore = System;
	type Balance = u64;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU64<1>;
	type FreezeIdentifier = [u8; 8];
	type MaxFreezes = ConstU32<10>;
	type MaxHolds = ConstU32<10>;
	type MaxLocks = ConstU32<10>;
	type MaxReserves = ConstU32<10>;
	type ReserveIdentifier = [u8; 8];
	type RuntimeEvent = RuntimeEvent;
	type RuntimeHoldReason = RuntimeHoldReason;
	type WeightInfo = ();
}

impl crate::Config for MockRuntime {
	const PALLET_ID: [u8; 8] = *b"lcl_crcy";
	type Currency = Balances;
	type PauseOrigin = EnsureRoot<Self::AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type SubmitterOrigin = EnsureSigned<Self::AccountId>;
	type SwapOrigin = EnsureRoot<Self::AccountId>;
	type XcmRouter = ();
}

#[derive(Default)]
pub(crate) struct ExtBuilder<T: Config>(Option<SwapPairInfoOf<T>>);

impl<T> ExtBuilder<T>
where
	T: Config,
{
	pub(crate) fn with_swap_pair_info(mut self, swap_pair_info: SwapPairInfoOf<T>) -> Self {
		self.0 = Some(swap_pair_info);
		self
	}

	pub(crate) fn build(self) -> sp_io::TestExternalities {
		let mut ext = sp_io::TestExternalities::default();

		ext.execute_with(|| {
			if let Some(swap_pair_info) = self.0 {
				let SwapPairInfoOf::<T> {
					pool_account,
					ratio,
					remote_asset_balance,
					remote_asset_id,
					remote_reserve_location,
					..
				} = swap_pair_info;
				Pallet::<T>::set_swap_pair_bypass_checks(
					remote_reserve_location,
					remote_asset_id,
					ratio,
					remote_asset_balance,
					pool_account,
				);
			}
		});

		ext
	}
}
