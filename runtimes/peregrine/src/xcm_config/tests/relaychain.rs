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

use crate::xcm_config::tests::utils::get_from_seed;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use polkadot_primitives::{AccountId, AssignmentId, BlockNumber, ValidatorId, LOWEST_PUBLIC_ID};
use polkadot_runtime_parachains::configuration::HostConfiguration;
use polkadot_service::chain_spec::get_authority_keys_from_seed_no_beefy;
pub(crate) use rococo_runtime::{
	xcm_config::{LocationConverter, XcmConfig},
	BabeConfig, Balances, ConfigurationConfig, MessageQueue, RegistrarConfig, Runtime as RococoRuntime, RuntimeCall,
	RuntimeEvent, RuntimeGenesisConfig, RuntimeOrigin, SessionConfig, SessionKeys, System, SystemConfig, XcmPallet,
	BABE_GENESIS_EPOCH_CONFIG, WASM_BINARY,
};
use sc_consensus_grandpa::AuthorityId as GrandpaId;
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_consensus_beefy::crypto::AuthorityId as BeefyId;
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

#[allow(clippy::type_complexity)]
fn initial_authorities() -> Vec<(
	AccountId,
	AccountId,
	BabeId,
	GrandpaId,
	ImOnlineId,
	ValidatorId,
	AssignmentId,
	AuthorityDiscoveryId,
)> {
	vec![get_authority_keys_from_seed_no_beefy("Alice")]
}

fn session_keys(
	babe: BabeId,
	grandpa: GrandpaId,
	im_online: ImOnlineId,
	para_validator: ValidatorId,
	para_assignment: AssignmentId,
	authority_discovery: AuthorityDiscoveryId,
	beefy: BeefyId,
) -> SessionKeys {
	SessionKeys {
		babe,
		grandpa,
		im_online,
		para_validator,
		para_assignment,
		authority_discovery,
		beefy,
	}
}

fn genesis() -> Storage {
	RuntimeGenesisConfig {
		system: SystemConfig {
			code: WASM_BINARY.unwrap().to_vec(),
			..Default::default()
		},
		babe: BabeConfig {
			epoch_config: Some(BABE_GENESIS_EPOCH_CONFIG),
			..Default::default()
		},
		session: SessionConfig {
			keys: initial_authorities()
				.iter()
				.map(|x| {
					(
						x.0.clone(),
						x.0.clone(),
						session_keys(
							x.2.clone(),
							x.3.clone(),
							x.4.clone(),
							x.5.clone(),
							x.6.clone(),
							x.7.clone(),
							get_from_seed::<BeefyId>("Alice"),
						),
					)
				})
				.collect::<Vec<_>>(),
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
