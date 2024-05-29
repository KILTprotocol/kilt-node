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

// If you feel like getting in touch with us, you can do so at info@botlabs.org

//! KILT chain specification

use sc_service::ChainType;
use spiritnet_runtime::{ParachainInfoConfig, PolkadotXcmConfig, RuntimeGenesisConfig, SystemConfig, WASM_BINARY};

use crate::chain_spec::{
	spiritnet::{ChainSpec, SAFE_XCM_VERSION},
	utils::get_properties,
	Extensions, KILT_PARA_ID,
};

pub(crate) fn generate_chain_spec() -> ChainSpec {
	ChainSpec::from_genesis(
		"KILT Spiritnet New (change title)",
		"kilt_spiritnet_new",
		ChainType::Live,
		generate_genesis_state,
		vec![],
		None,
		None,
		None,
		Some(get_properties("KILT", 15, 38)),
		Extensions {
			relay_chain: "relay".into(),
			para_id: KILT_PARA_ID,
		},
	)
}

fn generate_genesis_state() -> RuntimeGenesisConfig {
	let wasm_binary = WASM_BINARY.expect("WASM binary not available");

	RuntimeGenesisConfig {
		system: SystemConfig {
			code: wasm_binary.to_vec(),
			..Default::default()
		},
		parachain_info: ParachainInfoConfig {
			parachain_id: KILT_PARA_ID.into(),
			..Default::default()
		},
		polkadot_xcm: PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
			..Default::default()
		},
		..Default::default()
	}
}
