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

use peregrine_runtime::{
	BalancesConfig, CouncilConfig, ParachainInfoConfig, ParachainStakingConfig, PolkadotXcmConfig,
	RuntimeGenesisConfig, SessionConfig, SessionKeys, SudoConfig, SystemConfig, TechnicalCommitteeConfig, WASM_BINARY,
};
use runtime_common::{
	constants::{kilt_inflation_config, staking::MinCollatorStake, KILT, MAX_COLLATOR_STAKE},
	AccountId, AuthorityId, Balance,
};
use sc_service::ChainType;
use sp_core::sr25519;

use crate::chain_spec::{
	peregrine::{ChainSpec, SAFE_XCM_VERSION},
	utils::{get_account_id_from_secret, get_properties, get_public_key_from_secret},
	Extensions, KILT_PARA_ID,
};

pub(crate) fn generate_chain_spec(relaychain_name: &str) -> ChainSpec {
	ChainSpec::from_genesis(
		"KILT Peregrine Develop",
		"kilt_peregrine_dev",
		ChainType::Development,
		generate_genesis_state,
		vec![],
		None,
		None,
		None,
		Some(get_properties("PILT", 15, 38)),
		Extensions {
			relay_chain: relaychain_name.into(),
			para_id: KILT_PARA_ID,
		},
	)
}

fn generate_genesis_state() -> RuntimeGenesisConfig {
	let wasm_binary = WASM_BINARY.expect("Development WASM binary not available");
	let alice = (
		get_account_id_from_secret::<sr25519::Public>("Alice"),
		get_public_key_from_secret::<AuthorityId>("Alice"),
	);
	let bob = (
		get_account_id_from_secret::<sr25519::Public>("Bob"),
		get_public_key_from_secret::<AuthorityId>("Bob"),
	);
	let endowed_accounts = [
		alice.0.clone(),
		bob.0.clone(),
		get_account_id_from_secret::<sr25519::Public>("Charlie"),
		get_account_id_from_secret::<sr25519::Public>("Dave"),
		get_account_id_from_secret::<sr25519::Public>("Eve"),
		get_account_id_from_secret::<sr25519::Public>("Ferdie"),
	];

	RuntimeGenesisConfig {
		system: SystemConfig {
			code: wasm_binary.to_vec(),
			..Default::default()
		},
		balances: BalancesConfig {
			balances: endowed_accounts.map(|acc| (acc, 10_000_000 * KILT)).to_vec(),
		},
		session: SessionConfig {
			keys: [alice.clone(), bob.clone()]
				.map(|(acc, key)| (acc.clone(), acc, SessionKeys { aura: key }))
				.to_vec(),
		},
		sudo: SudoConfig {
			key: Some(alice.0.clone()),
		},
		parachain_info: ParachainInfoConfig {
			parachain_id: KILT_PARA_ID.into(),
			..Default::default()
		},
		parachain_staking: ParachainStakingConfig {
			stakers: [alice.clone(), bob.clone()]
				.map(|(acc, _)| -> (AccountId, Option<AccountId>, Balance) { (acc, None, 2 * MinCollatorStake::get()) })
				.to_vec(),
			inflation_config: kilt_inflation_config(),
			max_candidate_stake: MAX_COLLATOR_STAKE,
		},
		council: CouncilConfig {
			members: [alice.clone(), bob.clone()].map(|(acc, _)| acc).to_vec(),
			phantom: Default::default(),
		},
		technical_committee: TechnicalCommitteeConfig {
			members: [alice, bob].map(|(acc, _)| acc).to_vec(),
			phantom: Default::default(),
		},
		polkadot_xcm: PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
			..Default::default()
		},
		..Default::default()
	}
}
