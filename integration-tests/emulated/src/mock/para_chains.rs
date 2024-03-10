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

use integration_tests_common::constants::{accounts, asset_hub_polkadot, polkadot::ED};
use runtime_common::{xcm_config::LocationToAccountId, AuthorityId};
use sp_core::sr25519;
use sp_runtime::{BuildStorage, Storage};
use spiritnet_runtime::{
	xcm_config::RelayNetworkId, BalancesConfig, ParachainInfoConfig, PolkadotXcmConfig, RuntimeGenesisConfig,
	SessionConfig, SessionKeys, SystemConfig, WASM_BINARY,
};
use xcm_emulator::{decl_test_parachains, BridgeMessageHandler, Parachain, TestExt};

use crate::utils::{get_account_id_from_seed, get_from_seed};

const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;

pub mod spiritnet {
	use super::*;

	pub const PARA_ID: u32 = 2_000;

	pub fn genesis() -> Storage {
		RuntimeGenesisConfig {
			system: SystemConfig {
				code: WASM_BINARY
					.expect("WASM binary was not build, please build it!")
					.to_vec(),
				..Default::default()
			},
			parachain_info: ParachainInfoConfig {
				parachain_id: PARA_ID.into(),
				..Default::default()
			},
			polkadot_xcm: PolkadotXcmConfig {
				safe_xcm_version: Some(SAFE_XCM_VERSION),
				..Default::default()
			},
			session: SessionConfig {
				keys: vec![(
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_from_seed::<AuthorityId>("Alice"),
				)]
				.iter()
				.map(|(acc, key)| (acc.clone(), acc.clone(), SessionKeys { aura: key.clone() }))
				.collect::<Vec<_>>(),
			},
			balances: BalancesConfig {
				balances: accounts::init_balances()
					.iter()
					.cloned()
					.map(|k| (k, ED * 4096))
					.collect(),
			},
			..Default::default()
		}
		.build_storage()
		.unwrap()
	}
}

decl_test_parachains! {
	pub struct Spiritnet {
		genesis = spiritnet::genesis(),
		on_init = (),
		runtime = {
			Runtime: spiritnet_runtime::Runtime,
			RuntimeOrigin: spiritnet_runtime::RuntimeOrigin,
			RuntimeCall: spiritnet_runtime::RuntimeCall,
			RuntimeEvent: spiritnet_runtime::RuntimeEvent,
			XcmpMessageHandler: spiritnet_runtime::XcmpQueue,
			DmpMessageHandler: spiritnet_runtime::DmpQueue,
			LocationToAccountId: LocationToAccountId<RelayNetworkId>,
			System: spiritnet_runtime::System,
			Balances: spiritnet_runtime::Balances,
			ParachainSystem: spiritnet_runtime::ParachainSystem,
			ParachainInfo: spiritnet_runtime::ParachainInfo,
		},
		pallets_extra = {
			Did: spiritnet_runtime::Did,
		}
	},
	pub struct AssetHub {
		genesis = asset_hub_polkadot::genesis(),
		on_init = (),
		runtime = {
			Runtime: asset_hub_polkadot_runtime::Runtime,
			RuntimeOrigin: asset_hub_polkadot_runtime::RuntimeOrigin,
			RuntimeCall: asset_hub_polkadot_runtime::RuntimeCall,
			RuntimeEvent: asset_hub_polkadot_runtime::RuntimeEvent,
			XcmpMessageHandler: asset_hub_polkadot_runtime::XcmpQueue,
			DmpMessageHandler: asset_hub_polkadot_runtime::DmpQueue,
			LocationToAccountId: asset_hub_polkadot_runtime::xcm_config::LocationToAccountId,
			System: asset_hub_polkadot_runtime::System,
			Balances: asset_hub_polkadot_runtime::Balances,
			ParachainSystem: asset_hub_polkadot_runtime::ParachainSystem,
			ParachainInfo: asset_hub_polkadot_runtime::ParachainInfo,
		},
		pallets_extra = {
			PolkadotXcm: asset_hub_polkadot_runtime::PolkadotXcm,
			Assets: asset_hub_polkadot_runtime::Assets,
		}
	}
}
