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

use kilt_support::traits::InspectMetadata;
use peregrine_runtime::{MetadataProvider, SS_58_PREFIX, WASM_BINARY};
use sc_service::ChainType;

use crate::chain_spec::{peregrine::ChainSpec, utils::get_properties, Extensions, KILT_PARA_ID};

pub(crate) fn generate_chain_spec() -> ChainSpec {
	let wasm_binary = WASM_BINARY.expect("WASM binary not available");
	let genesis_config = peregrine_runtime::genesis_state::production::generate_genesis_state();
	let currency_symbol = String::from_utf8(MetadataProvider::symbol()).expect("Creating currency symbol failed");
	let denomination = MetadataProvider::decimals();

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
	.with_properties(get_properties(
		&currency_symbol,
		denomination.into(),
		SS_58_PREFIX.into(),
	))
	.with_genesis_config(genesis_config)
	.build()
}
