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

// If you feel like getting in touch with us, you can do so at <hello@kilt.org>

use cumulus_pallet_aura_ext::FixedVelocityConsensusHook;
use cumulus_pallet_parachain_system::RelayNumberMonotonicallyIncreases;
use frame_support::{parameter_types, traits::EnqueueWithOrigin, weights::Weight};
use runtime_common::{constants, xcm_config::RelayOrigin};

use crate::{weights, MessageQueue, Runtime, RuntimeEvent, XcmpQueue};

parameter_types! {
	pub const ReservedXcmpWeight: Weight = constants::MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
	pub const ReservedDmpWeight: Weight = constants::MAXIMUM_BLOCK_WEIGHT.saturating_div(4);
}

pub type ConsensusHook = FixedVelocityConsensusHook<
	Runtime,
	{ constants::RELAY_CHAIN_SLOT_DURATION_MILLIS },
	{ constants::BLOCK_PROCESSING_VELOCITY },
	{ constants::UNINCLUDED_SEGMENT_CAPACITY },
>;

impl cumulus_pallet_parachain_system::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnSystemEvent = ();
	type SelfParaId = parachain_info::Pallet<Runtime>;
	type OutboundXcmpMessageSource = XcmpQueue;
	type DmpQueue = EnqueueWithOrigin<MessageQueue, RelayOrigin>;
	type ReservedDmpWeight = ReservedDmpWeight;
	type XcmpMessageHandler = XcmpQueue;
	type ReservedXcmpWeight = ReservedXcmpWeight;
	type CheckAssociatedRelayNumber = RelayNumberMonotonicallyIncreases;
	type ConsensusHook = ConsensusHook;
	type WeightInfo = weights::cumulus_pallet_parachain_system::WeightInfo<Runtime>;
}

impl parachain_info::Config for Runtime {}

impl cumulus_pallet_aura_ext::Config for Runtime {}
