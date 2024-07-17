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

use peregrine_runtime::WASM_BINARY;
use sc_service::ChainType;

use crate::chain_spec::{
	peregrine::{ChainSpec, SAFE_XCM_VERSION},
	utils::get_properties,
	Extensions, KILT_PARA_ID,
};

pub(crate) fn generate_chain_spec() -> ChainSpec {
	let wasm_binary = WASM_BINARY.expect("WASM binary not available");

	ChainSpec::builder(
		wasm_binary,
		Extensions {
			relay_chain: "relay".into(),
			para_id: KILT_PARA_ID,
		},
	)
	.with_name("KILT Peregrine New (change title)")
	.with_id("kilt_peregrine_new")
	.with_chain_type(ChainType::Live)
	.with_properties(get_properties("PILT", 15, 38))
	.with_genesis_config(generate_genesis_state())
	.build()
}

fn generate_genesis_state() -> serde_json::Value {
	serde_json::json!({
		"parachainInfo": {
			"parachainId": KILT_PARA_ID,
		},
		"polkadot_xcm": {
			"safeXcmVersion": SAFE_XCM_VERSION,
		},
	})
}
