// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2023 BOTLabs GmbH

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

use cumulus_primitives_core::ParaId;
use runtime_common::constants::KILT;
use sc_chain_spec::ChainType;
use sp_core::sr25519;

use clone_runtime::{
	BalancesConfig, ParachainInfoConfig, PolkadotXcmConfig, RuntimeGenesisConfig, SessionConfig, SudoConfig,
	SystemConfig,
};
use clone_runtime::{CollatorSelectionConfig, WASM_BINARY};
use runtime_common::{AccountId, AuthorityId, Balance};

use super::{get_account_id_from_seed, get_from_seed, get_properties, DEFAULT_PARA_ID};
use crate::chain_spec::Extensions;

const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<RuntimeGenesisConfig, Extensions>;

pub fn get_chain_spec_dev() -> Result<ChainSpec, String> {
	let properties = get_properties("CLN", 15, 38);
	let wasm = WASM_BINARY.ok_or("No WASM")?;

	Ok(ChainSpec::from_genesis(
		"Clone",
		"clone_dev",
		ChainType::Local,
		move || {
			testnet_genesis(
				wasm,
				vec![
					(
						get_account_id_from_seed::<sr25519::Public>("Alice"),
						get_from_seed::<AuthorityId>("Alice"),
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Bob"),
						get_from_seed::<AuthorityId>("Bob"),
					),
				],
				vec![
					(get_account_id_from_seed::<sr25519::Public>("Alice"), 2_000 * KILT),
					(get_account_id_from_seed::<sr25519::Public>("Bob"), 2_000 * KILT),
				],
				DEFAULT_PARA_ID,
				get_account_id_from_seed::<sr25519::Public>("Alice"),
			)
		},
		vec![],
		None,
		None,
		None,
		Some(properties),
		Extensions {
			relay_chain: "rococo_local_testnet".into(),
			para_id: DEFAULT_PARA_ID.into(),
		},
	))
}

#[allow(clippy::too_many_arguments)]
fn testnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AccountId, AuthorityId)>,
	endowed_accounts: Vec<(AccountId, Balance)>,
	id: ParaId,
	root_key: AccountId,
) -> RuntimeGenesisConfig {
	RuntimeGenesisConfig {
		system: SystemConfig {
			code: wasm_binary.to_vec(),
			..Default::default()
		},
		balances: BalancesConfig {
			balances: endowed_accounts,
		},
		sudo: SudoConfig { key: Some(root_key) },
		parachain_info: ParachainInfoConfig {
			parachain_id: id,
			..Default::default()
		},
		aura: Default::default(),
		aura_ext: Default::default(),
		parachain_system: Default::default(),
		session: SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|(acc, key)| {
					(
						acc.clone(),
						acc.clone(),
						clone_runtime::SessionKeys { aura: key.clone() },
					)
				})
				.collect::<Vec<_>>(),
		},
		collator_selection: CollatorSelectionConfig {
			invulnerables: initial_authorities.iter().map(|(acc, _)| acc).cloned().collect(),
			candidacy_bond: 100,
			desired_candidates: 2,
		},
		polkadot_xcm: PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
			..Default::default()
		},
	}
}
