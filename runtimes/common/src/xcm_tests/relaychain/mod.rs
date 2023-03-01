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

use frame_support::{
	construct_runtime, parameter_types,
	traits::{Everything, Nothing},
	weights::Weight,
};
use frame_system::EnsureRoot;
use polkadot_parachain::primitives::Id as ParaId;
use polkadot_runtime_parachains::{configuration, origin, shared, ump};
use sp_core::{ConstU32, ConstU64, H256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
	AccountId32,
};
use xcm::latest::prelude::*;
use xcm_builder::{
	Account32Hash, AccountId32Aliases, AllowUnpaidExecutionFrom, ChildParachainAsNative, ChildParachainConvertsVia,
	ChildSystemParachainAsSuperuser, EnsureXcmOrigin, FixedWeightBounds, SignedAccountId32AsNative,
	SignedToAccountId32, SovereignSignedViaLocation,
};
use xcm_executor::XcmExecutor;

parameter_types! {
	pub const BalanceExistentialDeposit: Balance = 1;
	pub const RelayNetworkId: NetworkId = ByGenesis([0; 32]);
	pub const UniversalLocation: InteriorMultiLocation = Here;
	pub const UnitWeightCost: Weight = Weight::from_parts(1, 1);
}
#[cfg(feature = "runtime-benchmarks")]
parameter_types! {
	pub ReachableDest: Option<MultiLocation> = Some(Parachain(1).into());
}

type AccountId = AccountId32;
type Balance = u128;
type Block<Runtime> = frame_system::mocking::MockBlock<Runtime>;
type UncheckedExtrinsic<Runtime> = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;

type LocationToAccountId = (
	ChildParachainConvertsVia<ParaId, AccountId>,
	AccountId32Aliases<RelayNetworkId, AccountId>,
	Account32Hash<(), AccountId>,
);
type LocalOriginConverter = (
	SovereignSignedViaLocation<LocationToAccountId, RuntimeOrigin>,
	ChildParachainAsNative<origin::Origin, RuntimeOrigin>,
	SignedAccountId32AsNative<RelayNetworkId, RuntimeOrigin>,
	ChildSystemParachainAsSuperuser<ParaId, RuntimeOrigin>,
);
type XcmRouter = super::RelayChainXcmRouter;

construct_runtime!(
	pub enum Runtime where
		Block = Block<Runtime>,
		NodeBlock = Block<Runtime>,
		UncheckedExtrinsic = UncheckedExtrinsic<Runtime>,
	{
		System: frame_system::{Pallet, Call, Storage, Config, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		XcmPallet: pallet_xcm::{Pallet, Call, Storage, Event<T>, Origin},
		ParasUmp: ump::{Pallet, Call, Storage, Event},
		ParasOrigin: origin::{Pallet, Origin},
	}
);

impl frame_system::Config for Runtime {
	type AccountData = pallet_balances::AccountData<Balance>;
	type AccountId = AccountId;
	type BaseCallFilter = Everything;
	type BlockHashCount = ConstU64<250>;
	type BlockLength = ();
	type BlockNumber = u64;
	type BlockWeights = ();
	type DbWeight = ();
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type Header = Header;
	type Index = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type MaxConsumers = ConstU32<16>;
	type OnKilledAccount = ();
	type OnNewAccount = ();
	type OnSetCode = ();
	type PalletInfo = PalletInfo;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type SS58Prefix = ();
	type SystemWeightInfo = ();
	type Version = ();
}

impl pallet_balances::Config for Runtime {
	type AccountStore = System;
	type Balance = Balance;
	type DustRemoval = ();
	type ExistentialDeposit = BalanceExistentialDeposit;
	type MaxLocks = ConstU32<50>;
	type MaxReserves = ConstU32<50>;
	type ReserveIdentifier = [u8; 8];
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
}

pub struct XcmConfig;
impl xcm_executor::Config for XcmConfig {
	type AssetClaims = ();
	type AssetExchanger = ();
	type AssetLocker = ();
	type AssetTransactor = ();
	type AssetTrap = ();
	type Barrier = AllowUnpaidExecutionFrom<Everything>;
	type CallDispatcher = RuntimeCall;
	type FeeManager = ();
	type IsReserve = ();
	type IsTeleporter = ();
	type MaxAssetsIntoHolding = ConstU32<64>;
	type MessageExporter = ();
	type OriginConverter = LocalOriginConverter;
	type PalletInstancesInfo = ();
	type ResponseHandler = ();
	type RuntimeCall = RuntimeCall;
	type SafeCallFilter = Everything;
	type SubscriptionService = ();
	type Trader = ();
	type UniversalAliases = Nothing;
	type UniversalLocation = UniversalLocation;
	type XcmSender = XcmRouter;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, ConstU32<100>>;
}

impl shared::Config for Runtime {}

impl configuration::Config for Runtime {
	type WeightInfo = configuration::TestWeightInfo;
}

impl ump::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type UmpSink = ump::XcmSink<XcmExecutor<XcmConfig>, Runtime>;
	type FirstMessageFactorPercent = ConstU64<100>;
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
	type WeightInfo = ump::TestWeightInfo;
}

impl pallet_xcm::Config for Runtime {
	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;

	type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
	type Currency = Balances;
	type CurrencyMatcher = ();
	type ExecuteXcmOrigin =
		EnsureXcmOrigin<RuntimeOrigin, SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetworkId>>;
	type MaxLockers = ConstU32<8>;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetworkId>>;
	type SovereignAccountOf = LocationToAccountId;
	type TrustedLockers = ();
	type UniversalLocation = UniversalLocation;
	type XcmExecuteFilter = Everything;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type XcmReserveTransferFilter = Nothing;
	type XcmRouter = XcmRouter;
	type XcmTeleportFilter = Nothing;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, ConstU32<100>>;
	type WeightInfo = pallet_xcm::TestWeightInfo;

	#[cfg(feature = "runtime-benchmarks")]
	type ReachableDest = ReachableDest;
}

impl origin::Config for Runtime {}
