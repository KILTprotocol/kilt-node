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

use clone_runtime::{
	BalancesConfig, CollatorSelectionConfig, GenesisConfig, ParachainInfoConfig, PolkadotXcmConfig, SessionConfig,
	SudoConfig, SystemConfig, WASM_BINARY,
};
use cumulus_primitives_core::ParaId;
use hex_literal::hex;
use runtime_common::{
	constants::{staking::MinCollatorStake, KILT},
	AccountId, AuthorityId, Balance,
};
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use sp_core::{crypto::UncheckedInto, sr25519};

use crate::chain_spec::{get_account_id_from_seed, get_from_seed, DEFAULT_PARA_ID, TELEMETRY_URL};

use super::{get_properties, Extensions};

const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

pub fn get_chain_spec_dev() -> Result<ChainSpec, String> {
	let properties = get_properties("KILT", 15, 38);
	let wasm = WASM_BINARY.ok_or("No WASM")?;

	Ok(ChainSpec::from_genesis(
		"KILT clone Develop",
		"cln_kilt_dev",
		ChainType::Local,
		move || {
			testnet_genesis(
				wasm,
				vec![
					(
						get_account_id_from_seed::<sr25519::Public>("Alice"),
						None,
						2 * MinCollatorStake::get(),
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Bob"),
						None,
						2 * MinCollatorStake::get(),
					),
				],
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
				get_account_id_from_seed::<sr25519::Public>("Alice"),
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

const CLN_SUDO: [u8; 32] = hex!["14ab94d42fb790854e7c4813af55722e2007ce2070177bbe93d64cabe5f6ca6f"];
const CLN_COL_ACC_1: [u8; 32] = hex!["d8f775301891bc245f2cbf2d64cf1c0e64d16632c02268fd2199c84b09ff7f7b"];
const CLN_COL_SESSION_1: [u8; 32] = hex!["88245cdf5b5b517c48b0057e17c94c7ff71eeb7ba4665b3d07accdc0c3064915"];
const CLN_COL_ACC_2: [u8; 32] = hex!["5c7c70470cb16b4702921f0b4e2a7109277354bd3d8e11b63bd7ed70510cf57f"];
const CLN_COL_SESSION_2: [u8; 32] = hex!["487cf837b45261c45c45a38e66be1fb80dc7d755094b44661632ec30d3a5db01"];

pub fn get_chain_spec_cln() -> Result<ChainSpec, String> {
	let properties = get_properties("KILT", 15, 38);
	let wasm = WASM_BINARY.ok_or("No WASM")?;
	let id: ParaId = 2086.into();

	Ok(ChainSpec::from_genesis(
		"Clone2",
		"cln_kilt2",
		ChainType::Live,
		move || {
			testnet_genesis(
				wasm,
				vec![
					(CLN_COL_ACC_1.into(), None, 30000 * KILT),
					(CLN_COL_ACC_2.into(), None, 30000 * KILT),
				],
				vec![
					(CLN_COL_ACC_1.into(), CLN_COL_SESSION_1.unchecked_into()),
					(CLN_COL_ACC_2.into(), CLN_COL_SESSION_2.unchecked_into()),
				],
				vec![
					(CLN_COL_ACC_1.into(), 40000 * KILT),
					(CLN_COL_ACC_2.into(), 40000 * KILT),
					(CLN_SUDO.into(), 40000 * KILT),
				],
				CLN_SUDO.into(),
				id,
			)
		},
		vec![],
		Some(TelemetryEndpoints::new(vec![(TELEMETRY_URL.to_string(), 0)]).expect("telemetry url is valid; qed")),
		None,
		None,
		Some(properties),
		Extensions {
			relay_chain: "polkadot".into(),
			para_id: id.into(),
		},
	))
}

pub fn load_clone_spec() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../../res/clone.json")[..])
}

pub fn load_clone2_spec() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../../res/clone2.json")[..])
}

pub fn load_clone3_spec() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../../res/clone3.json")[..])
}

#[allow(clippy::too_many_arguments)]
fn testnet_genesis(
	wasm_binary: &[u8],
	stakers: Vec<(AccountId, Option<AccountId>, Balance)>,
	initial_authorities: Vec<(AccountId, AuthorityId)>,
	endowed_accounts: Vec<(AccountId, Balance)>,
	sudo: AccountId,
	id: ParaId,
) -> GenesisConfig {
	GenesisConfig {
		system: SystemConfig {
			code: wasm_binary.to_vec(),
		},
		balances: BalancesConfig {
			balances: endowed_accounts.to_vec(),
		},
		parachain_info: ParachainInfoConfig { parachain_id: id },
		aura: Default::default(),
		aura_ext: Default::default(),
		parachain_system: Default::default(),
		collator_selection: CollatorSelectionConfig {
			invulnerables: stakers.iter().map(|(acc, _, _)| acc).cloned().collect(),
			candidacy_bond: MinCollatorStake::get(),
			desired_candidates: 2,
		},
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
		sudo: SudoConfig { key: Some(sudo) },
		polkadot_xcm: PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
		},
	}
}
