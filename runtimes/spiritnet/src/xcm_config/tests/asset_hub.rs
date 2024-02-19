// Asset Hub Polkadot

use super::*;
pub const PARA_ID: u32 = 1000;
pub const ED: Balance = parachains_common::polkadot::currency::EXISTENTIAL_DEPOSIT;

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
