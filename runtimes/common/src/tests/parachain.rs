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

use frame_support::{construct_runtime, parameter_types, traits::Everything, weights::constants::RocksDbWeight};
use sp_core::{ConstU16, ConstU32, ConstU64};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
	MultiSignature,
};
use xcm::v2::{MultiLocation, Parent};

type Block<Runtime> = frame_system::mocking::MockBlock<Runtime>;
type Hash = sp_core::H256;
type UncheckedExtrinsic<Runtime> = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Balance = u128;
type BlockNumber = u64;
type Index = u32;
type ReserveIdentifier = [u8; 8];
type Signature = MultiSignature;
type AccountPublic = <Signature as Verify>::Signer;
type AccountId = <AccountPublic as IdentifyAccount>::AccountId;

pub mod sender {
	use super::*;

	construct_runtime!(
		pub enum ParachainRuntime where
			Block = Block<ParachainRuntime>,
			NodeBlock = Block<ParachainRuntime>,
			UncheckedExtrinsic = UncheckedExtrinsic<ParachainRuntime>,
		{
			System: frame_system::{Pallet, Call, Storage, Config, Event<T>},
			Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
			PolkadotXcm: pallet_xcm::{Pallet, Call, Event<T>, Origin},
		}
	);

	impl frame_system::Config for ParachainRuntime {
		type AccountData = pallet_balances::AccountData<Balance>;
		type AccountId = AccountId;
		type BaseCallFilter = Everything;
		type BlockHashCount = ConstU64<20>;
		type BlockLength = ();
		type BlockNumber = BlockNumber;
		type BlockWeights = ();
		type DbWeight = RocksDbWeight;
		type Hash = Hash;
		type Hashing = BlakeTwo256;
		type Header = Header;
		type MaxConsumers = ConstU32<16>;
		type OnKilledAccount = ();
		type OnNewAccount = ();
		type OnSetCode = ();
		type PalletInfo = PalletInfo;
		type Index = Index;
		type Lookup = IdentityLookup<Self::AccountId>;
		type RuntimeCall = RuntimeCall;
		type RuntimeOrigin = RuntimeOrigin;
		type RuntimeEvent = RuntimeEvent;
		type SS58Prefix = ConstU16<38>;
		type SystemWeightInfo = ();
		type Version = ();
	}

	parameter_types! {
		const ExistentialDeposit: Balance = 0;
	}

	impl pallet_balances::Config for ParachainRuntime {
		type AccountStore = System;
		type Balance = Balance;
		type DustRemoval = ();
		type ExistentialDeposit = ExistentialDeposit;
		type MaxLocks = ConstU32<50>;
		type MaxReserves = ConstU32<50>;
		type ReserveIdentifier = ReserveIdentifier;
		type RuntimeEvent = RuntimeEvent;
		type WeightInfo = ();
	}

	#[cfg(feature = "runtime-benchmarks")]
	parameter_types! {
		pub ReachableDest: Option<MultiLocation> = Some(Parent.into());
	}

	parameter_types! {
		UniversalLocation<ParachainId>: InteriorMultiLocation = Parachain()
	}

	impl pallet_xcm::Config for ParachainRuntime {
		const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;

		type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
		type Currency = Balances;
		type CurrencyMatcher = ();
		type ExecuteXcmOrigin = ();
		type MaxLockers = ConstU32<8>;
		type RuntimeCall = RuntimeCall;
		type RuntimeEvent = RuntimeEvent;
		type RuntimeOrigin = RuntimeOrigin;
		type SendXcmOrigin = ();
		type SovereignAccountOf = ();
		type TrustedLockers = ();

		#[cfg(feature = "runtime-benchmarks")]
		type ReachableDest = ReachableDest;
	}
}
