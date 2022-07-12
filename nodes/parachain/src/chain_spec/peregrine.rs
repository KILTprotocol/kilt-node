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
use peregrine_runtime::{
	BalancesConfig, CouncilConfig, GenesisConfig, InflationInfo, ParachainInfoConfig, ParachainStakingConfig,
	SessionConfig, SudoConfig, SystemConfig, TechnicalCommitteeConfig, VestingConfig, WASM_BINARY,
};
use runtime_common::{
	constants::{kilt_inflation_config, staking::MinCollatorStake, MAX_COLLATOR_STAKE},
	AccountId, AuthorityId, Balance, BlockNumber,
};
use sc_service::ChainType;
use sp_core::sr25519;
use sp_runtime::traits::Zero;

use crate::chain_spec::{get_account_id_from_seed, get_from_seed, get_properties, Extensions, DEFAULT_PARA_ID};

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

pub fn make_dev_spec() -> Result<ChainSpec, String> {
	let properties = get_properties("PILT", 15, 38);
	let wasm = WASM_BINARY.ok_or("No WASM")?;

	Ok(ChainSpec::from_genesis(
		"KILT Peregrine Develop",
		"kilt_peregrine_dev",
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
				kilt_inflation_config(),
				MAX_COLLATOR_STAKE,
				get_account_id_from_seed::<sr25519::Public>("Alice"),
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
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Charlie"),
					get_account_id_from_seed::<sr25519::Public>("Dave"),
					get_account_id_from_seed::<sr25519::Public>("Eve"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
					get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
					get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
					get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
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
				hex!["d206033ba2eadf615c510f2c11f32d931b27442e5cfb64884afa2241dfa66e70"].into(),
				vec![],
				vec![],
				id,
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

#[allow(clippy::too_many_arguments)]
fn testnet_genesis(
	wasm_binary: &[u8],
	stakers: Vec<(AccountId, Option<AccountId>, Balance)>,
	inflation_config: InflationInfo,
	max_candidate_stake: Balance,
	root_key: AccountId,
	initial_authorities: Vec<(AccountId, AuthorityId)>,
	endowed_accounts: Vec<AccountId>,
	id: ParaId,
) -> GenesisConfig {
	type VestingPeriod = BlockNumber;
	type LockingPeriod = BlockNumber;

	// vesting and locks as initially designed
	let airdrop_accounts_json = &include_bytes!("../../res/genesis/claimable-accounts.json")[..];
	let airdrop_accounts: Vec<(AccountId, Balance, VestingPeriod, LockingPeriod)> =
		serde_json::from_slice(airdrop_accounts_json).expect("The file genesis_accounts.json exists and is valid; qed");

	// botlabs account should not be migrated but some have vesting
	let botlabs_accounts_json = &include_bytes!("../../res/genesis/owned-accounts.json")[..];
	let botlabs_accounts: Vec<(AccountId, Balance, VestingPeriod, LockingPeriod)> =
		serde_json::from_slice(botlabs_accounts_json).expect("The file botlabs_accounts.json exists and is valid; qed");

	GenesisConfig {
		system: SystemConfig {
			code: wasm_binary.to_vec(),
		},
		balances: BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, 10000000000000000000000000000_u128))
				.chain(airdrop_accounts.iter().cloned().map(|(who, total, _, _)| (who, total)))
				.chain(botlabs_accounts.iter().cloned().map(|(who, total, _, _)| (who, total)))
				.collect(),
		},
		sudo: SudoConfig { key: Some(root_key) },
		parachain_info: ParachainInfoConfig { parachain_id: id },
		vesting: VestingConfig {
			vesting: botlabs_accounts
				.iter()
				.cloned()
				.filter(|(_, _, vesting_length, _)| !vesting_length.is_zero())
				.map(|(who, _, vesting_length, _)| (who, 0u64, vesting_length, 0))
				.collect(),
		},
		council: CouncilConfig {
			members: vec![],
			phantom: Default::default(),
		},
		technical_committee: TechnicalCommitteeConfig {
			members: vec![],
			phantom: Default::default(),
		},
		treasury: Default::default(),
		tips_membership: Default::default(),
		technical_membership: Default::default(),
		democracy: Default::default(),
		parachain_staking: ParachainStakingConfig {
			stakers,
			inflation_config,
			max_candidate_stake,
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
						peregrine_runtime::SessionKeys { aura: key.clone() },
					)
				})
				.collect::<Vec<_>>(),
		},
	}
}
