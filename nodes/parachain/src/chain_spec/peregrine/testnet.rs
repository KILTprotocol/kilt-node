use cumulus_primitives_core::ParaId;
use hex_literal::hex;
use sc_service::ChainType;

use peregrine_runtime::WASM_BINARY;
use runtime_common::constants::{kilt_inflation_config, MAX_COLLATOR_STAKE};

use crate::chain_spec::{get_properties, peregrine::ChainSpec, Extensions};

use super::testnet_genesis;

pub fn make_new_spec() -> Result<ChainSpec, String> {
	let properties = get_properties("PILT", 15, 38);
	let wasm = WASM_BINARY.ok_or("No WASM")?;
	let id: ParaId = 1000.into();

	Ok(ChainSpec::from_genesis(
		"KILT Peregrine Testnet",
		"kilt_peregrine_testnet",
		ChainType::Live,
		move || {
			testnet_genesis(
				wasm,
				vec![],
				kilt_inflation_config(),
				MAX_COLLATOR_STAKE,
				vec![],
				vec![],
				id,
				hex!["d206033ba2eadf615c510f2c11f32d931b27442e5cfb64884afa2241dfa66e70"].into(),
			)
		},
		Vec::new(),
		None,
		None,
		None,
		Some(properties),
		Extensions {
			relay_chain: "rococo_local_testnet".into(),
			para_id: id.into(),
		},
	))
}
