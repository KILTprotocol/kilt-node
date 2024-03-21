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
use sp_std::prelude::ToOwned;
use xcm::v3::prelude::*;
use xcm_builder::{
	AllowKnownQueryResponses, AllowSubscriptionsFrom, AllowTopLevelPaidExecutionFrom, AllowUnpaidExecutionFrom,
	EnsureXcmOrigin, FixedWeightBounds, NativeAsset, RelayChainAsNative, SiblingParachainAsNative,
	SignedAccountId32AsNative, SignedToAccountId32, SovereignSignedViaLocation, TakeWeightCredit, TrailingSetTopicAsId,
	UsingComponents, WithComputedOrigin,
};
use xcm_executor::{traits::WithOriginFilter, XcmExecutor};

use runtime_common::xcm_config::{
	DenyReserveTransferToRelayChain, DenyThenTry, HereLocation, LocalAssetTransactor, LocationToAccountId,
	MaxAssetsIntoHolding, MaxInstructions, ParentLocation, ParentOrSiblings, UnitWeightCost,
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

/// This type specifies how a `MultiLocation` can be converted into an
/// `AccountId` within the Peregrine network, which is crucial for determining
/// ownership of accounts for asset transactions and for dispatching XCM
/// `Transact` operations.
pub type LocationToAccountIdConverter = LocationToAccountId<RelayNetworkId>;

/// This is the type we use to convert an (incoming) XCM origin into a local
/// `Origin` instance, ready for dispatching a transaction with Xcm's
/// `Transact`. There is an `OriginKind` which can bias the kind of local
/// `Origin` it will become.
pub type XcmOriginToTransactDispatchOrigin = (
	// Sovereign account converter; this attempts to derive an `AccountId` from the origin location
	// using `LocationToAccountIdConverter` and then turn that into the usual `Signed` origin. Useful for
	// foreign chains who want to have a local sovereign account on this chain which they control.
	// In contrast to Spiritnet, it's fine to include this on peregrine for testing.
	SovereignSignedViaLocation<LocationToAccountIdConverter, RuntimeOrigin>,
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

/// Explicitly deny ReserveTransfer to the relay chain. Allow calls from the
/// relay chain governance.
pub type XcmBarrier = TrailingSetTopicAsId<
	DenyThenTry<
		DenyReserveTransferToRelayChain,
		(
			// For local extrinsics. Takes credit from already paid extrinsic fee. This is outside the computed origin
			// since local accounts don't have a computed origin (the message isn't send by any router etc.)
			TakeWeightCredit,
			// If we request a response we should also allow it to execute.
			AllowKnownQueryResponses<PolkadotXcm>,
			WithComputedOrigin<
				(
					// Allow unpaid execution from the relay chain
					AllowUnpaidExecutionFrom<ParentLocation>,
					// Allow paid execution.
					AllowTopLevelPaidExecutionFrom<Everything>,
					// Subscriptions for XCM version are OK from the relaychain and other parachains.
					AllowSubscriptionsFrom<ParentOrSiblings>,
				),
				UniversalLocation,
				ConstU32<8>,
			>,
		),
	>,
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
	fn contains(c: &RuntimeCall) -> bool {
		fn is_call_allowed(call: &RuntimeCall) -> bool {
			matches!(
				call,
				RuntimeCall::Ctype { .. }
				| RuntimeCall::DidLookup { .. }
				| RuntimeCall::Web3Names { .. }
				| RuntimeCall::PublicCredentials { .. }
				| RuntimeCall::Attestation { .. }
				// we exclude here [dispatch_as] and [submit_did_call]
				| RuntimeCall::Did (
							did::Call::add_key_agreement_key { .. }
							| did::Call::add_service_endpoint { .. }
							| did::Call::create { .. }
							| did::Call::delete { .. }
							| did::Call::remove_attestation_key { .. }
							| did::Call::remove_delegation_key { .. }
							| did::Call::remove_key_agreement_key { .. }
							| did::Call::remove_service_endpoint { .. }
							| did::Call::set_attestation_key { .. }
							| did::Call::set_authentication_key { .. }
							| did::Call::set_delegation_key { .. }
							| did::Call::update_deposit { .. }
							| did::Call::change_deposit_owner { .. }
							| did::Call::reclaim_deposit { .. }
							| did::Call::create_from_account { .. }
						)
			)
		}

		match c {
			RuntimeCall::Did(c) => match c {
				did::Call::dispatch_as { call, .. } => is_call_allowed(call),
				did::Call::submit_did_call { did_call, .. } => is_call_allowed(&did_call.call),
				_ => is_call_allowed(&c.to_owned().into()),
			},
			_ => is_call_allowed(c),
		}
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
	type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, ()>;
	type XcmRouter = XcmRouter;
	type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
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
	type SovereignAccountOf = LocationToAccountIdConverter;
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
