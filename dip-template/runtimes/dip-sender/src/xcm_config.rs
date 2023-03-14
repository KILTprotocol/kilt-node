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
	traits::{ConstU32, Nothing},
	weights::{IdentityFee, Weight},
};
use frame_system::EnsureRoot;
use xcm::latest::prelude::*;
use xcm_builder::{FixedWeightBounds, UsingComponents};
use xcm_executor::XcmExecutor;

use crate::{
	AccountId, AllPalletsWithSystem, Balance, Balances, HereLocation, ParachainInfo, ParachainSystem, Runtime,
	RuntimeCall, RuntimeEvent, XcmpQueue,
};

parameter_types! {
	pub UnitWeightCost: Weight = Weight::from_ref_time(1_000);
	pub UniversalLocation: InteriorMultiLocation = Parachain(ParachainInfo::parachain_id().into()).into();
}

pub type XcmRouter = (ParentAsUmp<ParachainSystem, (), ()>, XcmpQueue);

pub struct XcmConfig;
impl xcm_executor::Config for XcmConfig {
	type AssetClaims = ();
	type AssetExchanger = ();
	type AssetLocker = ();
	type AssetTransactor = ();
	type AssetTrap = ();
	type Barrier = ();
	type CallDispatcher = RuntimeCall;
	type FeeManager = ();
	type IsReserve = ();
	type IsTeleporter = ();
	type MaxAssetsIntoHolding = ConstU32<64>;
	type MessageExporter = ();
	type OriginConverter = ();
	type PalletInstancesInfo = AllPalletsWithSystem;
	type ResponseHandler = ();
	type RuntimeCall = RuntimeCall;
	type SafeCallFilter = Nothing;
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
