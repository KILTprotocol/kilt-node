// KILT Blockchain – <https://kilt.io>
// Copyright (C) 2025, KILT Foundation

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

// If you feel like getting in touch with us, you can do so at <hello@kilt.io>

use crate::{
	kilt::{CheckingAccount, KiltToEKiltSwitchPallet},
	xcm_components, AllPalletsWithSystem, AssetSwitchPool1, Balances, Fungibles, MessageQueue, ParachainInfo,
	ParachainSystem, PolkadotXcm, Runtime, RuntimeCall, RuntimeEvent, RuntimeOrigin, Treasury, WeightToFee, XcmpQueue,
};

use cumulus_primitives_core::{AggregateMessageOrigin, ParaId};
use frame_support::{
	parameter_types,
	traits::{Contains, Everything, Nothing, TransformOrigin},
};
use frame_system::EnsureRoot;
use kilt_support::xcm::EitherOr;
use pallet_asset_switch::xcm::{
	IsSwitchPairRemoteAsset, IsSwitchPairXcmFeeAsset, MatchesSwitchPairXcmFeeFungibleAsset,
	SwitchPairRemoteAssetTransactor, UsingComponentsForSwitchPairRemoteAsset, UsingComponentsForXcmFeeAsset,
};
use pallet_xcm::XcmPassthrough;
use parachains_common::message_queue::{NarrowOriginToSibling, ParaIdToSibling};
use polkadot_runtime_common::xcm_sender::NoPriceForMessageDelivery;
use sp_core::ConstU32;
use sp_std::prelude::ToOwned;
use xcm::v4::prelude::*;
use xcm_builder::{
	AllowKnownQueryResponses, AllowSubscriptionsFrom, AllowTopLevelPaidExecutionFrom, AllowUnpaidExecutionFrom,
	EnsureXcmOrigin, FixedWeightBounds, FrameTransactionalProcessor, FungiblesAdapter, NativeAsset, NoChecking,
	RelayChainAsNative, SiblingParachainAsNative, SignedAccountId32AsNative, SignedToAccountId32,
	SovereignSignedViaLocation, TakeWeightCredit, TrailingSetTopicAsId, UsingComponents, WithComputedOrigin,
};
use xcm_executor::{traits::WithOriginFilter, XcmExecutor};

use runtime_common::{
	constants,
	xcm_config::{
		DenyReserveTransferToRelayChain, DenyThenTry, HeapSize, HereLocation, LocalAssetTransactor,
		LocationToAccountId, MaxAssetsIntoHolding, MaxInstructions, MaxStale, ParentLocation, ParentOrSiblings,
		ServiceWeight, UnitWeightCost,
	},
	AccountId, SendDustAndFeesToTreasury,
};

parameter_types! {
	pub RelayChainOrigin: RuntimeOrigin = cumulus_pallet_xcm::Origin::Relay.into();
	pub Ancestry: Location = Parachain(ParachainInfo::parachain_id().into()).into();
	// TODO: This needs to be updated once we deploy Peregrine on Rococo/Paseo and once we migrate to an SDK version that includes Paseo.
	pub const RelayNetworkId: Option<NetworkId> = None;
	pub UniversalLocation: InteriorLocation =
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
			AllowKnownQueryResponses<EitherOr<PolkadotXcm, AssetSwitchPool1>>,
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
///    parameters.
pub struct SafeCallFilter;
impl Contains<RuntimeCall> for SafeCallFilter {
	fn contains(c: &RuntimeCall) -> bool {
		const fn is_call_allowed(call: &RuntimeCall) -> bool {
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
			RuntimeCall::Did(did_call) => match did_call {
				did::Call::dispatch_as { call, .. } => is_call_allowed(call),
				did::Call::submit_did_call {
					did_call: nested_did_call,
					..
				} => is_call_allowed(&nested_did_call.call),
				_ => is_call_allowed(&did_call.to_owned().into()),
			},
			_ => is_call_allowed(c),
		}
	}
}

parameter_types! {
	pub TreasuryAccountId: AccountId = Treasury::account_id();
}

pub struct XcmConfig;
impl xcm_executor::Config for XcmConfig {
	type RuntimeCall = RuntimeCall;
	// How we send Xcm messages.
	type XcmSender = XcmRouter;
	// How to withdraw and deposit an asset.
	// Until fixed, `LocalAssetTransactor` must be last since it returns an error if
	// the operation does not go through, i.e., it cannot be chained with other
	// transactors.
	type AssetTransactor = (
		// Allow the asset from the other side of the pool to be "deposited" into the current system.
		SwitchPairRemoteAssetTransactor<LocationToAccountIdConverter, Runtime, KiltToEKiltSwitchPallet>,
		// Allow the asset to pay for remote XCM fees to be deposited into the current system.
		FungiblesAdapter<
			Fungibles,
			MatchesSwitchPairXcmFeeFungibleAsset<Runtime, KiltToEKiltSwitchPallet>,
			LocationToAccountIdConverter,
			AccountId,
			NoChecking,
			CheckingAccount,
		>,
		// Transactor for fungibles matching the "Here" location.
		LocalAssetTransactor<Balances, RelayNetworkId>,
		// Transactor for BKilts
		FungiblesAdapter<
			Fungibles,
			xcm_components::matcher::MatchesBkiltAsset,
			LocationToAccountIdConverter,
			AccountId,
			NoChecking,
			CheckingAccount,
		>,
	);
	type OriginConverter = XcmOriginToTransactDispatchOrigin;
	type IsReserve = (
		NativeAsset,
		IsSwitchPairRemoteAsset<Runtime, KiltToEKiltSwitchPallet>,
		IsSwitchPairXcmFeeAsset<Runtime, KiltToEKiltSwitchPallet>,
		xcm_components::is_reserve::IsBKilt,
	);

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

	type Trader = (
		// Can pay for fees with the remote XCM asset fee (when sending it into this system).
		UsingComponentsForXcmFeeAsset<Runtime, KiltToEKiltSwitchPallet, WeightToFee<Runtime>>,
		// Can pay for the remote asset of the switch pair (when "depositing" it into this system).
		UsingComponentsForSwitchPairRemoteAsset<
			Runtime,
			KiltToEKiltSwitchPallet,
			WeightToFee<Runtime>,
			TreasuryAccountId,
		>,
		// Can pay with the fungible that matches the "Here" location.
		UsingComponents<WeightToFee<Runtime>, HereLocation, AccountId, Balances, SendDustAndFeesToTreasury<Runtime>>,
	);

	type ResponseHandler = EitherOr<PolkadotXcm, AssetSwitchPool1>;
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
	type TransactionalProcessor = FrameTransactionalProcessor;
	type HrmpChannelAcceptedHandler = ();
	type HrmpChannelClosingHandler = ();
	type HrmpNewChannelOpenRequestHandler = ();
	type XcmRecorder = PolkadotXcm;
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
}

impl cumulus_pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}

impl cumulus_pallet_xcmp_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type ChannelInfo = ParachainSystem;
	type VersionWrapper = PolkadotXcm;
	type ControllerOrigin = EnsureRoot<AccountId>;
	type ControllerOriginConverter = XcmOriginToTransactDispatchOrigin;
	type WeightInfo = cumulus_pallet_xcmp_queue::weights::SubstrateWeight<Self>;
	type PriceForSiblingDelivery = NoPriceForMessageDelivery<ParaId>;
	type MaxInboundSuspended = ConstU32<1_000>;
	type XcmpQueue = TransformOrigin<MessageQueue, AggregateMessageOrigin, ParaId, ParaIdToSibling>;
	type MaxActiveOutboundChannels = ConstU32<{ constants::pallet_xcmp_queue::MAX_ACTIVE_OUTBOUND_CHANNELS }>;
	type MaxPageSize = ConstU32<{ constants::pallet_xcmp_queue::MAX_PAGE_SIZE }>;
}

impl pallet_message_queue::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = crate::weights::pallet_message_queue::WeightInfo<Runtime>;
	#[cfg(feature = "runtime-benchmarks")]
	type MessageProcessor = pallet_message_queue::mock_helpers::NoopMessageProcessor<AggregateMessageOrigin>;
	#[cfg(not(feature = "runtime-benchmarks"))]
	type MessageProcessor =
		xcm_builder::ProcessXcmMessage<AggregateMessageOrigin, xcm_executor::XcmExecutor<XcmConfig>, RuntimeCall>;
	type Size = u32;
	type QueueChangeHandler = NarrowOriginToSibling<XcmpQueue>;
	type QueuePausedQuery = NarrowOriginToSibling<XcmpQueue>;
	type HeapSize = HeapSize;
	type MaxStale = MaxStale;
	type ServiceWeight = ServiceWeight;
	type IdleMaxServiceWeight = ();
}
