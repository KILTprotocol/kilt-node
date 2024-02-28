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

use crate::{
	AccountId, AllPalletsWithSystem, Balances, ParachainInfo, ParachainSystem, PolkadotXcm, Runtime, RuntimeCall,
	RuntimeEvent, RuntimeOrigin, Treasury, WeightToFee, XcmpQueue,
};

use frame_support::{
	parameter_types,
	traits::{Contains, Everything, Nothing},
};
use frame_system::EnsureRoot;
use pallet_xcm::XcmPassthrough;
use sp_core::ConstU32;
use xcm::v3::prelude::*;
use xcm_builder::{
	AllowTopLevelPaidExecutionFrom, EnsureXcmOrigin, FixedWeightBounds, NativeAsset, RelayChainAsNative,
	SiblingParachainAsNative, SignedAccountId32AsNative, SignedToAccountId32, SovereignSignedViaLocation,
	TakeWeightCredit, UsingComponents, WithComputedOrigin,
};
use xcm_executor::{traits::WithOriginFilter, XcmExecutor};

use runtime_common::xcm_config::{
	DenyReserveTransferToRelayChain, DenyThenTry, HereLocation, LocalAssetTransactor, LocationToAccountId,
	MaxAssetsIntoHolding, MaxInstructions, ParentLegislative, UnitWeightCost,
};

parameter_types! {
	pub RelayChainOrigin: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();
	pub Ancestry: MultiLocation = Parachain(ParachainInfo::parachain_id().into()).into();
	// TODO: This needs to be updated once we deploy Peregrine on Rococo/Paseo
	pub const RelayNetworkId: Option<NetworkId> = None;
	// TODO: This needs to be updated once we deploy Peregrine on Rococo/Paseo.
	pub UniversalLocation: InteriorMultiLocation =
		Parachain(ParachainInfo::parachain_id().into()).into();
}

/// This is the type we use to convert an (incoming) XCM origin into a local
/// `Origin` instance, ready for dispatching a transaction with Xcm's
/// `Transact`. There is an `OriginKind` which can bias the kind of local
/// `Origin` it will become.
pub type XcmOriginToTransactDispatchOrigin = (
	// Sovereign account converter; this attempts to derive an `AccountId` from the origin location
	// using `LocationToAccountId` and then turn that into the usual `Signed` origin. Useful for
	// foreign chains who want to have a local sovereign account on this chain which they control.
	// In contrast to Spiritnet, it's fine to include this on peregrine for testing.
	SovereignSignedViaLocation<LocationToAccountId<RelayNetworkId>, RuntimeOrigin>,
	// Native converter for Relay-chain (Parent) location which converts to a `Relay` origin when
	// recognized.
	RelayChainAsNative<RelayChainOrigin, RuntimeOrigin>,
	// Native converter for sibling Parachains which converts to a `SiblingPara` origin when
	// recognized.
	SiblingParachainAsNative<cumulus_pallet_xcm::Origin, RuntimeOrigin>,
	// Native signed account converter which just converts an `AccountId32` origin into a normal
	// `RuntimeOrigin::signed` origin of the same 32-byte value.
	SignedAccountId32AsNative<RelayNetworkId, RuntimeOrigin>,
	// Xcm origins can be represented natively under the Xcm pallet's Xcm origin.
	XcmPassthrough<RuntimeOrigin>,
);

pub type XcmBarrier = DenyThenTry<
	DenyReserveTransferToRelayChain,
	// For local XCM execution (i.e., for pallet_xcm::reserve_asset_transfer)
	(
		TakeWeightCredit,
		WithComputedOrigin<
			(
				// We allow everything from the relay chain if it was sent by the relay chain legislative (i.e.,
				// democracy vote). Since the relaychain doesn't own KILTs and missing fees shouldn't prevent calls
				// from the relaychain legislative, we allow unpaid execution.
				AllowTopLevelPaidExecutionFrom<ParentLegislative>,
			),
			UniversalLocation,
			ConstU32<8>,
		>,
	),
>;

/// A call filter for the XCM Transact instruction. This is a temporary measure
/// until we properly account for proof size weights.
///
/// Calls that are allowed through this filter must:
/// 1. Have a fixed weight;
/// 2. Cannot lead to another call being made;
/// 3. Have a defined proof size weight, e.g. no unbounded vecs in call
/// parameters.
pub struct SafeCallFilter;
impl Contains<RuntimeCall> for SafeCallFilter {
	fn contains(_call: &RuntimeCall) -> bool {
		false
	}
}

pub struct XcmConfig;
impl xcm_executor::Config for XcmConfig {
	type RuntimeCall = RuntimeCall;
	// How we send Xcm messages.
	type XcmSender = XcmRouter;
	// How to withdraw and deposit an asset.
	type AssetTransactor = LocalAssetTransactor<Balances, RelayNetworkId>;
	type OriginConverter = XcmOriginToTransactDispatchOrigin;
	// Reserving is disabled.
	type IsReserve = NativeAsset;
	// Teleporting is disabled.
	type IsTeleporter = ();
	type UniversalLocation = UniversalLocation;
	// Which XCM instructions are allowed and which are not on our chain.
	type Barrier = XcmBarrier;
	// How XCM messages are weighted. Each transaction has a weight of
	// `UnitWeightCost`.
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	// How weight is transformed into fees. The fees are not taken out of the
	// Balances pallet here. Balances is only used if fees are dropped without being
	// used. In that case they are put into the treasury.
	type Trader = UsingComponents<WeightToFee<Runtime>, HereLocation, AccountId, Balances, Treasury>;
	type ResponseHandler = PolkadotXcm;
	// What happens with assets that are left in the register after the XCM message
	// was processed. PolkadotXcm has an AssetTrap that stores a hash of the asset
	// location, amount, version, etc.
	type AssetTrap = PolkadotXcm;
	type AssetClaims = PolkadotXcm;
	type SubscriptionService = PolkadotXcm;
	type PalletInstancesInfo = AllPalletsWithSystem;
	type MaxAssetsIntoHolding = MaxAssetsIntoHolding;
	type AssetLocker = ();
	type AssetExchanger = ();
	type FeeManager = ();
	type MessageExporter = ();
	type UniversalAliases = Nothing;
	type CallDispatcher = WithOriginFilter<SafeCallFilter>;
	type SafeCallFilter = SafeCallFilter;
	type Aliasers = Nothing;
}

/// Allows only local `Signed` origins to be converted into `MultiLocation`s by
/// the XCM executor.
pub type LocalOriginToLocation = SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetworkId>;

/// The means for routing XCM messages which are not for local execution into
/// the right message queues.
pub type XcmRouter = (
	// Two routers. Use UMP to communicate with the relay chain:
	cumulus_primitives_utility::ParentAsUmp<ParachainSystem, PolkadotXcm, ()>,
	// .. and XCMP to communicate with the sibling chains.
	XcmpQueue,
);

#[cfg(feature = "runtime-benchmarks")]
parameter_types! {
	pub ReachableDest: Option<MultiLocation> = Some(Parent.into());
}

impl pallet_xcm::Config for Runtime {
	type MaxRemoteLockConsumers = ConstU32<0>;
	type RemoteLockConsumerIdentifier = ();
	type RuntimeEvent = RuntimeEvent;
	// Allows anyone to send XCM messages. For regular origins, a `DescendOrigin` is
	// prepended to the message.
	type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type XcmRouter = XcmRouter;
	// We allow execution of XCM programs because it is required by the
	// `reserve_transfer_assets` operation.
	type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	// Disable calls to pallet_xcm::execute(), by still allowing the other
	// extrinsics to be called.
	// NOTE: For local testing this needs to be `Everything`.
	type XcmExecuteFilter = Nothing;
	type XcmTeleportFilter = Nothing;
	type XcmReserveTransferFilter = Everything;
	type AdminOrigin = EnsureRoot<AccountId>;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;

	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
	// Our latest supported XCM version.
	type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
	type UniversalLocation = UniversalLocation;
	type Currency = Balances;
	type CurrencyMatcher = ();
	type TrustedLockers = ();
	type SovereignAccountOf = LocationToAccountId<RelayNetworkId>;
	type MaxLockers = ConstU32<8>;
	type WeightInfo = crate::weights::pallet_xcm::WeightInfo<Runtime>;
	#[cfg(feature = "runtime-benchmarks")]
	type ReachableDest = ReachableDest;
}

impl cumulus_pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}

impl cumulus_pallet_xcmp_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ChannelInfo = ParachainSystem;
	type VersionWrapper = PolkadotXcm;
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
	type ControllerOrigin = EnsureRoot<AccountId>;
	type ControllerOriginConverter = XcmOriginToTransactDispatchOrigin;
	type WeightInfo = cumulus_pallet_xcmp_queue::weights::SubstrateWeight<Self>;
	// TODO: Most chains use `NoPriceForMessageDelivery`, merged in https://github.com/paritytech/polkadot-sdk/pull/1234.
	type PriceForSiblingDelivery = ();
}

impl cumulus_pallet_dmp_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type ExecuteOverweightOrigin = EnsureRoot<AccountId>;
}