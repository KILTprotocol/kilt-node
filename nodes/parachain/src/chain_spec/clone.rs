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
use hex_literal::hex;
use runtime_common::constants::KILT;
use sc_chain_spec::ChainType;
use sp_core::crypto::UncheckedInto;
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

const CLN_COL1_ACC: [u8; 32] = hex!["34c6801027ff9c1d8700b06f1f8598c07a1ca66aaa7c2829a1801e9bcf72b132"];
const CLN_COL1_SESSION: [u8; 32] = hex!["aee85fda658762e82ccc1cd83d95cadc1d3375208d4d16d9b932427290905e56"];
const CLN_COL2_ACC: [u8; 32] = hex!["2e259a331de128fc889ed3c9d4b03f2f95600715fd21754c3da885b7ee75bf18"];
const CLN_COL2_SESSION: [u8; 32] = hex!["82e6381d3241dc5e864c22f44f2a24cc2a51e09c9ca72d93061cdfbe22e38036"];

pub fn new_chain_spec() -> Result<ChainSpec, String> {
	let properties = get_properties("CLN", 15, 38);
	let wasm = WASM_BINARY.ok_or("No WASM")?;
	let id: ParaId = 3358.into();

	Ok(ChainSpec::from_genesis(
		"Clone",
		"clone3",
		ChainType::Live,
		move || {
			testnet_genesis(
				wasm,
				vec![
					(CLN_COL1_ACC.into(), CLN_COL1_SESSION.unchecked_into()),
					(CLN_COL2_ACC.into(), CLN_COL2_SESSION.unchecked_into()),
				],
				vec![
					(CLN_COL1_ACC.into(), 4_000_000 * KILT),
					(CLN_COL2_ACC.into(), 4_000_000 * KILT),
				],
				id,
				CLN_COL1_ACC.into(),
			)
		},
		vec![],
		None,
		None,
		None,
		Some(properties),
		Extensions {
			relay_chain: "polkadot".into(),
			para_id: id.into(),
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
