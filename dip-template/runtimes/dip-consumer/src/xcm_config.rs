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

use cumulus_primitives_utility::ParentAsUmp;
use frame_support::{
	parameter_types,
	traits::{ConstU32, Contains, Nothing},
	weights::{IdentityFee, Weight},
};
use frame_system::EnsureRoot;
use kilt_dip_support::xcm::{AccountIdJunctionAsParachain, OkOrElseCheckForParachainProvider};
use pallet_xcm::TestWeightInfo;
use xcm::latest::prelude::*;
use xcm_builder::{
	Account32Hash, AllowTopLevelPaidExecutionFrom, CurrencyAdapter, EnsureXcmOrigin, FixedWeightBounds, IsConcrete,
	SignedToAccountId32, UsingComponents,
};
use xcm_executor::XcmExecutor;

use crate::{
	AccountId, AllPalletsWithSystem, Balance, Balances, ParachainInfo, ParachainSystem, Runtime, RuntimeCall,
	RuntimeEvent, RuntimeOrigin, XcmpQueue,
};

parameter_types! {
	pub HereLocation: MultiLocation = MultiLocation::here();
	pub NoneNetworkId: Option<NetworkId> = None;
	pub UnitWeightCost: Weight = Weight::from_ref_time(1_000);
	pub UniversalLocation: InteriorMultiLocation = Parachain(ParachainInfo::parachain_id().into()).into();
}

pub type Barrier = OkOrElseCheckForParachainProvider<AllowTopLevelPaidExecutionFrom<Nothing>, ConstU32<2_000>>;
pub type AssetTransactorLocationConverter = Account32Hash<NoneNetworkId, AccountId>;
pub type LocalAssetTransactor =
	CurrencyAdapter<Balances, IsConcrete<HereLocation>, AssetTransactorLocationConverter, AccountId, ()>;
pub type XcmRouter = (ParentAsUmp<ParachainSystem, (), ()>, XcmpQueue);

pub struct DipTransactSafeCalls;

impl Contains<RuntimeCall> for DipTransactSafeCalls {
	fn contains(t: &RuntimeCall) -> bool {
		matches!(
			t,
			RuntimeCall::DipConsumer(pallet_dip_consumer::Call::process_identity_action { .. })
		)
	}
}

pub struct XcmConfig;
impl xcm_executor::Config for XcmConfig {
	type AssetClaims = ();
	type AssetExchanger = ();
	type AssetLocker = ();
	type AssetTransactor = LocalAssetTransactor;
	type AssetTrap = ();
	type Barrier = Barrier;
	type CallDispatcher = RuntimeCall;
	type FeeManager = ();
	type IsReserve = ();
	type IsTeleporter = ();
	type MaxAssetsIntoHolding = ConstU32<64>;
	type MessageExporter = ();
	type OriginConverter = AccountIdJunctionAsParachain<ConstU32<2_000>, cumulus_pallet_xcm::Origin, RuntimeOrigin>;
	type PalletInstancesInfo = AllPalletsWithSystem;
	type ResponseHandler = ();
	type RuntimeCall = RuntimeCall;
	type SafeCallFilter = DipTransactSafeCalls;
	type SubscriptionService = ();
	type UniversalAliases = Nothing;
	type UniversalLocation = UniversalLocation;
	type Trader = UsingComponents<IdentityFee<Balance>, HereLocation, AccountId, Balances, ()>;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, ConstU32<100>>;
	type XcmSender = XcmRouter;
}

impl cumulus_pallet_xcmp_queue::Config for Runtime {
	type ChannelInfo = ParachainSystem;
	type ControllerOrigin = EnsureRoot<AccountId>;
	type ControllerOriginConverter = ();
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
	type PriceForSiblingDelivery = ();
	type RuntimeEvent = RuntimeEvent;
	type VersionWrapper = ();
	type WeightInfo = ();
	type XcmExecutor = XcmExecutor<XcmConfig>;
}

impl cumulus_pallet_dmp_queue::Config for Runtime {
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}

const MAX_INSTRUCTIONS: u32 = 100;

parameter_types! {
	pub RelayNetwork: Option<NetworkId> = None;
}
#[cfg(feature = "runtime-benchmarks")]
parameter_types! {
	pub ReachableDest: Option<MultiLocation> = Some(Parent.into());
}

pub type XcmPalletToRemoteLocationConverter = SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetwork>;

impl pallet_xcm::Config for Runtime {
	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;

	type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
	type Currency = Balances;
	type CurrencyMatcher = ();
	type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, XcmPalletToRemoteLocationConverter>;
	type MaxLockers = ConstU32<8>;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, XcmPalletToRemoteLocationConverter>;
	type SovereignAccountOf = ();
	type TrustedLockers = ();
	type UniversalLocation = UniversalLocation;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, ConstU32<MAX_INSTRUCTIONS>>;
	type WeightInfo = TestWeightInfo;
	type XcmExecuteFilter = Nothing;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type XcmReserveTransferFilter = Nothing;
	type XcmRouter = XcmRouter;
	type XcmTeleportFilter = Nothing;
	#[cfg(feature = "runtime-benchmarks")]
	type ReachableDest = ReachableDest;
}

impl cumulus_pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}
