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

use integration_tests_common::{polkadot, rococo};
use xcm_emulator::{decl_test_relay_chains, RelayChain, TestExt, XcmHash};

decl_test_relay_chains! {
	#[api_version(5)]
	pub struct Polkadot {
		genesis = polkadot::genesis(),
		on_init = (),
		runtime = {
			Runtime: polkadot_runtime::Runtime,
			RuntimeOrigin: polkadot_runtime::RuntimeOrigin,
			RuntimeCall: polkadot_runtime::RuntimeCall,
			RuntimeEvent: polkadot_runtime::RuntimeEvent,
			MessageQueue: polkadot_runtime::MessageQueue,
			XcmConfig: polkadot_runtime::xcm_config::XcmConfig,
			SovereignAccountOf: polkadot_runtime::xcm_config::SovereignAccountOf,
			System: polkadot_runtime::System,
			Balances: polkadot_runtime::Balances,
		},
		pallets_extra = {
			XcmPallet: polkadot_runtime::XcmPallet,
		}
	},
	#[api_version(5)]
	pub struct Rococo {
		genesis = rococo::genesis(),
		on_init = (),
		runtime = {
			Runtime: rococo_runtime::Runtime,
			RuntimeOrigin: rococo_runtime::RuntimeOrigin,
			RuntimeCall: rococo_runtime::RuntimeCall,
			RuntimeEvent: rococo_runtime::RuntimeEvent,
			MessageQueue: rococo_runtime::MessageQueue,
			XcmConfig: rococo_runtime::xcm_config::XcmConfig,
			SovereignAccountOf: rococo_runtime::xcm_config::LocationConverter,
			System: rococo_runtime::System,
			Balances: rococo_runtime::Balances,
		},
		pallets_extra = {
			XcmPallet: rococo_runtime::XcmPallet,
		}
	}
}
