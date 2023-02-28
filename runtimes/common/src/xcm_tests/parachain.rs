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

use cumulus_primitives_core::ParaId;
use cumulus_primitives_utility::ParentAsUmp;
use frame_support::{
	construct_runtime, parameter_types,
	traits::Everything,
	weights::{constants::RocksDbWeight, Weight},
};
use sp_core::{ConstU16, ConstU32, ConstU64};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
	MultiSignature,
};
use xcm::v2::{MultiLocation, Parent};
use xcm_builder::{AllowUnpaidExecutionFrom, SignedAccountId32AsNative, SignedToAccountId32};

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

type Barrier = AllowUnpaidExecutionFrom<Everything>;
type LocalOriginToLocation<RuntimeOrigin, RelayNetwork> = SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetwork>;

pub mod sender {
	use frame_support::traits::Nothing;
use pallet_xcm::TestWeightInfo;
	use xcm::v2::{InteriorMultiLocation, Junction::Parachain, NetworkId};
	use xcm_builder::{EnsureXcmOrigin, FixedWeightBounds};
	use xcm_executor::{XcmExecutor, };

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

	parameter_types! {
		pub const PolkadotLocation: MultiLocation = MultiLocation::parent();
		pub const RelayNetworkId: NetworkId = NetworkId::Polkadot;
		pub const UniversalLocation: InteriorMultiLocation = InteriorMultiLocation::X1(Parachain(2000));
		pub const UnitWeightCost: Weight = Weight::from_parts(1, 1);
	}

	#[cfg(feature = "runtime-benchmarks")]
	parameter_types! {
		pub ReachableDest: Option<MultiLocation> = Some(Parent.into());
	}

	impl pallet_xcm::Config for ParachainRuntime {
		const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;

		type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
		type Currency = Balances;
		type CurrencyMatcher = ();
		type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation<RuntimeOrigin, RelayNetworkId>>;
		type MaxLockers = ConstU32<8>;
		type RuntimeCall = RuntimeCall;
		type RuntimeEvent = RuntimeEvent;
		type RuntimeOrigin = RuntimeOrigin;
		type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation<RuntimeOrigin, RelayNetworkId>>;
		type SovereignAccountOf = ();
		type TrustedLockers = ();
		type UniversalLocation = UniversalLocation;
		type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, ConstU32<100>>;
		type WeightInfo = TestWeightInfo;
		type XcmExecuteFilter = Everything;
		type XcmExecutor = XcmExecutor<XcmConfig>;

		#[cfg(feature = "runtime-benchmarks")]
		type ReachableDest = ReachableDest;
	}

	struct XcmConfig;
	impl xcm_executor::Config for XcmConfig {
		type AssetClaims = ();
		type AssetExchanger = ();
		type AssetLocker = ();
		type AssetTransactor = ();
		type AssetTrap = ();
		type Barrier = Barrier;
		type CallDispatcher = RuntimeCall;
		type FeeManager = ();
		type IsReserve = ();
		type IsTeleporter = ();
		type MaxAssetsIntoHolding = ConstU32<10>;
		type MessageExporter = ();
		type OriginConverter = SignedAccountId32AsNative<RelayNetworkId, RuntimeOrigin>;
		type PalletInstancesInfo = ();
		type ResponseHandler = ();
		type RuntimeCall = RuntimeCall;
		type SafeCallFilter = Everything;
		type SubscriptionService = ();
		type Trader = ();
		type UniversalAliases = Nothing;
		type UniversalLocation = UniversalLocation;
		type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, ConstU32<100>>;;
		type XcmSender = ParachainXcmRou;
	}
}
