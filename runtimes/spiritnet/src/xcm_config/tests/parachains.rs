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

use crate::xcm_config::tests::relaychain::{accounts, collators, polkadot::ED};
pub(crate) use crate::{
	xcm_config::{
		tests::utils::{get_account_id_from_seed, get_from_seed},
		RelayNetworkId,
	},
	AuthorityId, Balances, BalancesConfig, DmpQueue, ParachainInfo, ParachainInfoConfig, ParachainSystem,
	PolkadotXcmConfig, Runtime as PeregrineRuntime, RuntimeCall, RuntimeEvent, RuntimeGenesisConfig, RuntimeOrigin,
	SessionConfig, SessionKeys, System, SystemConfig, XcmpQueue, WASM_BINARY,
};
use runtime_common::constants::EXISTENTIAL_DEPOSIT;
pub(crate) use runtime_common::{xcm_config::LocationToAccountId, AccountPublic};
use sp_core::sr25519;
use sp_runtime::{BuildStorage, Storage};
use xcm_emulator::{decl_test_parachains, BridgeMessageHandler, Parachain, TestExt};

const PARA_ID: u32 = 2_000;
const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;

fn genesis() -> Storage {
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
				get_account_id_from_seed::<AccountPublic, sr25519::Public>("Alice"),
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
				.map(|k| (k, EXISTENTIAL_DEPOSIT * 4096))
				.collect(),
		},
		..Default::default()
	}
	.build_storage()
	.unwrap()
}

pub mod asset_hub_polkadot {

	use super::*;
	pub const PARA_ID: u32 = 1000;

	pub fn genesis() -> Storage {
		let genesis_config = asset_hub_polkadot_runtime::RuntimeGenesisConfig {
			system: asset_hub_polkadot_runtime::SystemConfig {
				code: asset_hub_polkadot_runtime::WASM_BINARY
					.expect("WASM binary was not build, please build it!")
					.to_vec(),
				..Default::default()
			},
			balances: asset_hub_polkadot_runtime::BalancesConfig {
				balances: accounts::init_balances()
					.iter()
					.cloned()
					.map(|k| (k, ED * 4096))
					.collect(),
			},
			parachain_info: asset_hub_polkadot_runtime::ParachainInfoConfig {
				parachain_id: PARA_ID.into(),
				..Default::default()
			},
			collator_selection: asset_hub_polkadot_runtime::CollatorSelectionConfig {
				invulnerables: collators::invulnerables_asset_hub_polkadot()
					.iter()
					.cloned()
					.map(|(acc, _)| acc)
					.collect(),
				candidacy_bond: ED * 16,
				..Default::default()
			},
			session: asset_hub_polkadot_runtime::SessionConfig {
				keys: collators::invulnerables_asset_hub_polkadot()
					.into_iter()
					.map(|(acc, aura)| {
						(
							acc.clone(),                                      // account id
							acc,                                              // validator id
							asset_hub_polkadot_runtime::SessionKeys { aura }, // session keys
						)
					})
					.collect(),
			},
			polkadot_xcm: asset_hub_polkadot_runtime::PolkadotXcmConfig {
				safe_xcm_version: Some(SAFE_XCM_VERSION),
				..Default::default()
			},
			..Default::default()
		};

		genesis_config.build_storage().unwrap()
	}
}

decl_test_parachains! {
	pub struct AssetHubPolkadot {
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
	},
	pub struct SpiritnetPolkadot {
		genesis = genesis(),
		on_init = (),
		runtime = {
			Runtime: PeregrineRuntime,
			RuntimeOrigin: RuntimeOrigin,
			RuntimeCall: RuntimeCall,
			RuntimeEvent: RuntimeEvent,
			XcmpMessageHandler: XcmpQueue,
			DmpMessageHandler: DmpQueue,
			LocationToAccountId: LocationToAccountId<RelayNetworkId>,
			System: System,
			Balances: Balances,
			ParachainSystem: ParachainSystem,
			ParachainInfo: ParachainInfo,
		},
		pallets_extra = {}
	}
}
