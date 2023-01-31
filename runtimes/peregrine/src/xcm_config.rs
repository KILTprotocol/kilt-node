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

use super::{
	AccountId, Balances, ParachainInfo, ParachainSystem, PolkadotXcm, Runtime, RuntimeCall, RuntimeEvent,
	RuntimeOrigin, Treasury, WeightToFee, XcmpQueue,
};

use frame_support::{parameter_types, traits::Nothing};
use pallet_xcm::XcmPassthrough;
use xcm::latest::prelude::*;
use xcm_builder::{
	EnsureXcmOrigin, FixedWeightBounds, LocationInverter, RelayChainAsNative, SiblingParachainAsNative,
	SignedAccountId32AsNative, SignedToAccountId32, SovereignSignedViaLocation, UsingComponents,
};
use xcm_executor::XcmExecutor;

use runtime_common::xcm_config::{
	HereLocation, LocalAssetTransactor, LocationToAccountId, MaxInstructions, UnitWeightCost, XcmBarrier,
};

parameter_types! {
	pub RelayChainOrigin: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();
	pub Ancestry: MultiLocation = Parachain(ParachainInfo::parachain_id().into()).into();
	// TODO: This needs to be updated once we deploy Peregrine on Rococo
	pub const RelayNetworkId: NetworkId = NetworkId::Any;
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

pub struct XcmConfig;
impl xcm_executor::Config for XcmConfig {
	type RuntimeCall = RuntimeCall;
	// How we send Xcm messages.
	type XcmSender = XcmRouter;
	// How to withdraw and deposit an asset.
	type AssetTransactor = LocalAssetTransactor<Balances, RelayNetworkId>;
	type OriginConverter = XcmOriginToTransactDispatchOrigin;
	// Reserving is disabled.
	type IsReserve = ();
	// Teleporting is disabled.
	type IsTeleporter = ();
	// Invert a location.
	// E.g., the relay chain is described as parent (MultiLocation(1, Here)))
	// chain. When we invert that location it would be MultiLocation(0,
	// X1(Parachain(2086))) since we are a child chain with ParaId 2086.
	type LocationInverter = LocationInverter<Ancestry>;
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
}

/// Allows only local `Signed` origins to be converted into `MultiLocation`s by
/// the XCM executor.
pub type LocalOriginToLocation = SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetworkId>;

/// The means for routing XCM messages which are not for local execution into
/// the right message queues.
pub type XcmRouter = (
	// Two routers. Use UMP to communicate with the relay chain:
	cumulus_primitives_utility::ParentAsUmp<ParachainSystem, ()>,
	// .. and XCMP to communicate with the sibling chains.
	XcmpQueue,
);

impl pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	type XcmRouter = XcmRouter;
	type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
	// Disable dispatchable execution on the XCM pallet.
	// NOTE: For local testing this needs to be `Everything`.
	type XcmExecuteFilter = Nothing;
	type XcmTeleportFilter = Nothing;
	type XcmReserveTransferFilter = Nothing;
	type XcmExecutor = XcmExecutor<XcmConfig>;
	type Weigher = FixedWeightBounds<UnitWeightCost, RuntimeCall, MaxInstructions>;
	type LocationInverter = LocationInverter<Ancestry>;
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;

	const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
	// Our latest supported XCM version.
	type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
}

impl cumulus_pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}
