// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2022 BOTLabs GmbH

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
use runtime_common::{constants::KILT, AccountId, AuthorityId, Balance};
use sc_service::ChainType;
use sp_core::{crypto::UncheckedInto, sr25519};

use void_runtime::{
	BalancesConfig, GenesisConfig, ParachainInfoConfig, PolkadotXcmConfig, SessionConfig, SudoConfig, SystemConfig,
	WASM_BINARY,
};

use crate::chain_spec::{get_account_id_from_seed, get_from_seed, DEFAULT_PARA_ID};

use super::{get_properties, Extensions};

const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

pub fn get_chain_spec_dev() -> Result<ChainSpec, String> {
	let properties = get_properties("VOID", 15, 38);
	let wasm = WASM_BINARY.ok_or("No WASM")?;

	Ok(ChainSpec::from_genesis(
		"VOID Local",
		"VOID_parachain_local_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				get_account_id_from_seed::<sr25519::Public>("Alice"),
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
					(get_account_id_from_seed::<sr25519::Public>("Alice"), 10000000 * KILT),
					(get_account_id_from_seed::<sr25519::Public>("Bob"), 10000000 * KILT),
					(get_account_id_from_seed::<sr25519::Public>("Charlie"), 10000000 * KILT),
					(get_account_id_from_seed::<sr25519::Public>("Dave"), 10000000 * KILT),
					(get_account_id_from_seed::<sr25519::Public>("Eve"), 10000000 * KILT),
					(get_account_id_from_seed::<sr25519::Public>("Ferdie"), 10000000 * KILT),
					(
						get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
						10000000 * KILT,
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
						10000000 * KILT,
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
						10000000 * KILT,
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
						10000000 * KILT,
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
						10000000 * KILT,
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
						10000000 * KILT,
					),
				],
				DEFAULT_PARA_ID,
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

const VOID_COL_ACC_1: [u8; 32] = hex!["d66c57ee2e3a5003c56083cddd2601e6e84e80887e8521a3cc2d1870c37a3e39"];
const VOID_COL_SESSION_1: [u8; 32] = hex!["26bde6c2cbd60beac843d9afd63e63e35bcee3ca1e70ee706e41213394cfed00"];

pub fn get_chain_spec_void() -> Result<ChainSpec, String> {
	let properties = get_properties("VOID", 15, 38);
	let wasm = WASM_BINARY.ok_or("No WASM")?;
	let id: ParaId = 2108.into();

	Ok(ChainSpec::from_genesis(
		"VOID",
		"void",
		ChainType::Live,
		move || {
			testnet_genesis(
				VOID_COL_ACC_1.into(),
				wasm,
				vec![(VOID_COL_ACC_1.into(), VOID_COL_SESSION_1.unchecked_into())],
				vec![(VOID_COL_ACC_1.into(), 9_999_999 * KILT)],
				id,
			)
		},
		vec![],
		None,
		None,
		None,
		Some(properties),
		Extensions {
			relay_chain: "kusama".into(),
			para_id: id.into(),
		},
	))
}

pub fn load_void_spec() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../../res/void.json")[..])
}

#[allow(clippy::too_many_arguments)]
fn testnet_genesis(
	sudo: AccountId,
	wasm_binary: &[u8],
	initial_authorities: Vec<(AccountId, AuthorityId)>,
	endowed_accounts: Vec<(AccountId, Balance)>,
	id: ParaId,
) -> GenesisConfig {
	GenesisConfig {
		sudo: SudoConfig { key: Some(sudo) },
		system: SystemConfig {
			code: wasm_binary.to_vec(),
		},
		balances: BalancesConfig {
			balances: endowed_accounts,
		},
		parachain_info: ParachainInfoConfig { parachain_id: id },
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
						void_runtime::SessionKeys { aura: key.clone() },
					)
				})
				.collect::<Vec<_>>(),
		},
		polkadot_xcm: PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
		},
	}
}
