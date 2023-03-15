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
	traits::{ConstU32, Contains, Everything, Nothing},
	weights::{IdentityFee, Weight},
};
use frame_system::EnsureRoot;
use polkadot_parachain::primitives::Sibling;
use xcm::latest::prelude::*;
use xcm_builder::{
	AllowTopLevelPaidExecutionFrom, CurrencyAdapter, FixedWeightBounds, IsConcrete, SiblingParachainAsNative,
	SiblingParachainConvertsVia, UsingComponents,
};
use xcm_executor::XcmExecutor;

use crate::{
	AccountId, AllPalletsWithSystem, Balance, Balances, ParachainInfo, ParachainSystem, Runtime, RuntimeCall,
	RuntimeEvent, RuntimeOrigin, XcmpQueue,
};

parameter_types! {
	pub const HereLocation: MultiLocation = MultiLocation::here();
	pub UnitWeightCost: Weight = Weight::from_ref_time(1_000);
	pub UniversalLocation: InteriorMultiLocation = Parachain(ParachainInfo::parachain_id().into()).into();
}

pub type Barrier = AllowTopLevelPaidExecutionFrom<Everything>;
pub type AssetTransactorLocationConverter = SiblingParachainConvertsVia<Sibling, AccountId>;
pub type LocalAssetTransactor =
	CurrencyAdapter<Balances, IsConcrete<HereLocation>, AssetTransactorLocationConverter, AccountId, ()>;
pub type XcmRouter = (ParentAsUmp<ParachainSystem, (), ()>, XcmpQueue);

pub struct DipTransactSafeCalls;

impl Contains<RuntimeCall> for DipTransactSafeCalls {
	fn contains(t: &RuntimeCall) -> bool {
		matches!(
			t,
			RuntimeCall::DipReceiver(pallet_dip_receiver::Call::process_identity_action { .. })
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
	type OriginConverter = SiblingParachainAsNative<cumulus_pallet_xcm::Origin, RuntimeOrigin>;
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

impl cumulus_pallet_xcm::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type XcmExecutor = XcmExecutor<XcmConfig>;
}