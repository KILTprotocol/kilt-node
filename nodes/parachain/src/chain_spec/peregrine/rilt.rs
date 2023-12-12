use cumulus_primitives_core::ParaId;
use hex_literal::hex;
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use sp_core::crypto::UncheckedInto;

use peregrine_runtime::WASM_BINARY;
use runtime_common::constants::{kilt_inflation_config, KILT, MAX_COLLATOR_STAKE};

use crate::chain_spec::{get_properties, peregrine::ChainSpec, Extensions, TELEMETRY_URL};

use super::testnet_genesis;

const RILT_COL_ACC_1: [u8; 32] = hex!["6a5c355bca369a54c334542fd91cf70822be92f215a1049ceb04f36baba9b87b"];
const RILT_COL_SESSION_1: [u8; 32] = hex!["66c4ca0710c2c8a92504f281d992000508ce255543016545014cf0bfbbe71429"];
const RILT_COL_ACC_2: [u8; 32] = hex!["768538a941d1e4730c31830ab85a54ff34aaaad1f81bdd246db11802a57a5412"];
const RILT_COL_SESSION_2: [u8; 32] = hex!["7cff6c7a53c4630a0a35f8793a04b663681575bbfa43dbe5848b220bc4bd1963"];

pub fn get_chain_spec_rilt() -> Result<ChainSpec, String> {
	let properties = get_properties("RILT", 15, 38);
	let wasm = WASM_BINARY.ok_or("No WASM")?;
	let id: ParaId = 2086.into();

	Ok(ChainSpec::from_genesis(
		"RILT",
		"kilt_rococo",
		ChainType::Live,
		move || {
			testnet_genesis(
				wasm,
				vec![
					(RILT_COL_ACC_1.into(), None, 200_000 * KILT),
					(RILT_COL_ACC_2.into(), None, 200_000 * KILT),
				],
				kilt_inflation_config(),
				MAX_COLLATOR_STAKE,
				vec![
					(RILT_COL_ACC_1.into(), RILT_COL_SESSION_1.unchecked_into()),
					(RILT_COL_ACC_2.into(), RILT_COL_SESSION_2.unchecked_into()),
				],
				vec![
					(RILT_COL_ACC_1.into(), 1_000_000 * KILT),
					(RILT_COL_ACC_2.into(), 1_000_000 * KILT),
				],
				id,
				RILT_COL_ACC_1.into(),
			)
		},
		vec![
			"/dns4/bootnode.kilt.io/tcp/30365/p2p/12D3KooWS2h3rxqEC9bzrFNKVgrT1iaGz2UAWA1jVG1EB6dEoeJm"
				.parse()
				.expect("bootnode address is formatted correctly; qed"),
			"/dns4/bootnode.kilt.io/tcp/30366/p2p/12D3KooWMSF7Vefmpf67iGMkPrUgvXw38HoxaLmTNpYGYikFS7DZ"
				.parse()
				.expect("bootnode address is formatted correctly; qed"),
		],
		Some(TelemetryEndpoints::new(vec![(TELEMETRY_URL.to_string(), 0)]).expect("RILT telemetry url is valid; qed")),
		None,
		None,
		Some(properties),
		Extensions {
			relay_chain: "rococo".into(),
			para_id: id.into(),
		},
	))
}

pub fn load_rilt_spec() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../../../res/rilt.json")[..])
}
