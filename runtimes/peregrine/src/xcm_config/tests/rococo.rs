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

use polkadot_primitives::{BlockNumber, LOWEST_PUBLIC_ID};
use polkadot_runtime_parachains::configuration::HostConfiguration;
use rococo_runtime::{
	xcm_config::{LocationConverter, XcmConfig},
	Balances, ConfigurationConfig, MessageQueue, RegistrarConfig, Runtime as RococoRuntime, RuntimeCall, RuntimeEvent,
	RuntimeGenesisConfig, RuntimeOrigin, System, SystemConfig, XcmPallet, WASM_BINARY,
};
use sp_runtime::{BuildStorage, Storage};
use xcm_emulator::{decl_test_relay_chains, RelayChain, TestExt, XcmHash};

fn get_host_config() -> HostConfiguration<BlockNumber> {
	HostConfiguration {
		max_upward_queue_size: 51200,
		max_upward_message_size: 51200,
		max_upward_message_num_per_candidate: 10,
		max_downward_message_size: 51200,
		..Default::default()
	}
}

fn genesis() -> Storage {
	RuntimeGenesisConfig {
		system: SystemConfig {
			code: WASM_BINARY.unwrap().to_vec(),
			..Default::default()
		},
		configuration: ConfigurationConfig {
			config: get_host_config(),
		},
		registrar: RegistrarConfig {
			next_free_para_id: LOWEST_PUBLIC_ID,
			..Default::default()
		},
		..Default::default()
	}
	.build_storage()
	.unwrap()
}

decl_test_relay_chains! {
	#[api_version(5)]
	pub struct Runtime {
		genesis = genesis(),
		on_init = (),
		runtime = {
			Runtime: RococoRuntime,
			RuntimeOrigin: RuntimeOrigin,
			RuntimeCall: RuntimeCall,
			RuntimeEvent: RuntimeEvent,
			MessageQueue: MessageQueue,
			XcmConfig: XcmConfig,
			SovereignAccountOf: LocationConverter,
			System: System,
			Balances: Balances,
		},
		pallets_extra = {
			XcmPallet: XcmPallet,
		}
	}
}
