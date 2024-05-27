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

use frame_support::traits::OnInitialize;
use integration_tests_common::constants::{accounts, asset_hub_polkadot, polkadot::ED};
use runtime_common::AuthorityId;
use sp_core::sr25519;
use sp_runtime::{BuildStorage, Storage};
use xcm_emulator::decl_test_parachains;

use crate::utils::{get_account_id_from_seed, get_from_seed};

const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;
pub mod spiritnet {
	use super::*;

	use spiritnet_runtime::{
		BalancesConfig, ParachainInfoConfig, PolkadotXcmConfig, RuntimeGenesisConfig, SessionConfig, SessionKeys,
		SystemConfig, WASM_BINARY,
	};

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

pub mod peregrine {
	use super::*;

	use peregrine_runtime::{
		BalancesConfig, ParachainInfoConfig, PolkadotXcmConfig, RuntimeGenesisConfig, SessionConfig, SessionKeys,
	};

	pub const PARA_ID: u32 = 2_000;

	pub fn genesis() -> Storage {
		RuntimeGenesisConfig {
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
		on_init = {
			spiritnet_runtime::AuraExt::on_initialize(1);
		},
		runtime = spiritnet_runtime,
		core = {
			XcmpMessageHandler: spiritnet_runtime::XcmpQueue,
			DmpMessageHandler: spiritnet_runtime::DmpQueue,
			LocationToAccountId: spiritnet_runtime::xcm_config::LocationToAccountIdConverter,
			ParachainInfo: spiritnet_runtime::ParachainInfo,
		},
		pallets = {
			Balances: spiritnet_runtime::Balances,
			PolkadotXcm: spiritnet_runtime::PolkadotXcm,
			Did: spiritnet_runtime::Did,
			Ctype: spiritnet_runtime::Ctype,
			Attestation: spiritnet_runtime::Attestation,
			Web3Names: spiritnet_runtime::Web3Names,
			DidLookup: spiritnet_runtime::DidLookup,
			PublicCredentials: spiritnet_runtime::PublicCredentials,
		}
	},
	pub struct Peregrine {
		genesis = peregrine::genesis(),
		on_init = {
			peregrine_runtime::AuraExt::on_initialize(1);
		},
		runtime = peregrine_runtime,
		core = {
			XcmpMessageHandler: peregrine_runtime::XcmpQueue,
			DmpMessageHandler: peregrine_runtime::DmpQueue,
			LocationToAccountId: peregrine_runtime::xcm_config::LocationToAccountIdConverter,
			ParachainInfo: peregrine_runtime::ParachainInfo,
		},
		pallets = {
			Balances: peregrine_runtime::Balances,
			PolkadotXcm: peregrine_runtime::PolkadotXcm,
			Did: peregrine_runtime::Did,
			Ctype: peregrine_runtime::Ctype,
			Attestation: peregrine_runtime::Attestation,
			Web3Names: peregrine_runtime::Web3Names,
			DidLookup: peregrine_runtime::DidLookup,
			PublicCredentials: peregrine_runtime::PublicCredentials,
		}
	},
	pub struct AssetHubPolkadot {
		genesis = asset_hub_polkadot::genesis(),
		on_init = {
			asset_hub_polkadot_runtime::AuraExt::on_initialize(1);
		},
		runtime = asset_hub_polkadot_runtime,
		core = {
			XcmpMessageHandler: asset_hub_polkadot_runtime::XcmpQueue,
			DmpMessageHandler: asset_hub_polkadot_runtime::DmpQueue,
			LocationToAccountId: asset_hub_polkadot_runtime::xcm_config::LocationToAccountId,
			ParachainInfo: asset_hub_polkadot_runtime::ParachainInfo,
		},
		pallets = {
			Balances: asset_hub_polkadot_runtime::Balances,
			PolkadotXcm: asset_hub_polkadot_runtime::PolkadotXcm,
			Assets: asset_hub_polkadot_runtime::Assets,
		}
	},
	pub struct AssetHubRococo {
		genesis = asset_hub_polkadot::genesis(),
		on_init = {
			asset_hub_polkadot_runtime::AuraExt::on_initialize(1);
		},
		runtime = asset_hub_polkadot_runtime,
		core = {
			XcmpMessageHandler: asset_hub_polkadot_runtime::XcmpQueue,
			DmpMessageHandler: asset_hub_polkadot_runtime::DmpQueue,
			LocationToAccountId: asset_hub_polkadot_runtime::xcm_config::LocationToAccountId,
			ParachainInfo: asset_hub_polkadot_runtime::ParachainInfo,
		},
		pallets = {
			Balances: asset_hub_polkadot_runtime::Balances,
			PolkadotXcm: asset_hub_polkadot_runtime::PolkadotXcm,
			Assets: asset_hub_polkadot_runtime::Assets,
		}
	},

}
