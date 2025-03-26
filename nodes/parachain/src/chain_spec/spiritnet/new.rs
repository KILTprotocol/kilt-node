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

// If you feel like getting in touch with us, you can do so at <hello@kilt.io>

//! KILT chain specification

use sc_service::ChainType;
use serde_json::to_value;
use spiritnet_runtime::{ParachainInfoConfig, PolkadotXcmConfig, RuntimeGenesisConfig, WASM_BINARY};

use crate::chain_spec::{
	spiritnet::{ChainSpec, SAFE_XCM_VERSION},
	utils::get_properties,
	Extensions, KILT_PARA_ID,
};

pub(crate) fn generate_chain_spec() -> ChainSpec {
	let wasm_binary = WASM_BINARY.expect("WASM binary not available");
	let genesis_state = to_value(generate_genesis_state()).expect("Creating genesis state failed");

	ChainSpec::builder(
		wasm_binary,
		Extensions {
			relay_chain: "relay".into(),
			para_id: KILT_PARA_ID,
		},
	)
	.with_name("KILT Spiritnet New (change title)")
	.with_id("kilt_spiritnet_new")
	.with_chain_type(ChainType::Live)
	.with_properties(get_properties("KILT", 15, 38))
	.with_genesis_config(genesis_state)
	.build()
}

fn generate_genesis_state() -> RuntimeGenesisConfig {
	RuntimeGenesisConfig {
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
