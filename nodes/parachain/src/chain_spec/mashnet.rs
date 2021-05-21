// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2021 BOTLabs GmbH

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
use kilt_parachain_runtime::{
	BalancesConfig, CouncilConfig, GenesisConfig, InflationInfo, KiltLaunchConfig, MinCollatorStk, ParachainInfoConfig,
	ParachainStakingConfig, SessionConfig, SudoConfig, SystemConfig, TechnicalCommitteeConfig, VestingConfig,
	WASM_BINARY,
};
use kilt_primitives::{
	constants::{DOLLARS, MINUTES},
	AccountId, AuthorityId, Balance, BlockNumber,
};
use sc_service::ChainType;
use sp_core::{crypto::UncheckedInto, sr25519};
use sp_runtime::Perquintill;

use crate::chain_spec::{get_account_id_from_seed, get_from_seed, get_properties, Extensions};

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

pub fn make_dev_spec(id: ParaId) -> Result<ChainSpec, String> {
	let properties = get_properties("KILT", 15, 38);
	let wasm = WASM_BINARY.ok_or("No WASM")?;

	Ok(ChainSpec::from_genesis(
		"KILT Collator Local Testnet",
		"kilt_parachain_local_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				wasm,
				vec![
					(
						// TODO: Change before launch
						get_account_id_from_seed::<sr25519::Public>("Alice"),
						None,
						2 * MinCollatorStk::get(),
					),
					(
						// TODO: Change before launch
						get_account_id_from_seed::<sr25519::Public>("Bob"),
						None,
						2 * MinCollatorStk::get(),
					),
				],
				kilt_inflation_config(),
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
				id,
			)
		},
		vec![],
		None,
		None,
		Some(properties),
		Extensions {
			relay_chain: "rococo_local_testnet".into(),
			para_id: id.into(),
		},
	))
}

pub fn make_staging_spec(id: ParaId) -> Result<ChainSpec, String> {
	let properties = get_properties("KILT", 15, 38);
	let wasm = WASM_BINARY.ok_or("No WASM")?;

	Ok(ChainSpec::from_genesis(
		"KILT Collator Staging Testnet",
		"kilt_parachain_staging_testnet",
		ChainType::Live,
		move || {
			testnet_genesis(
				wasm,
				vec![
					(
						// TODO: Change before launch
						hex!["d206033ba2eadf615c510f2c11f32d931b27442e5cfb64884afa2241dfa66e70"].into(),
						None,
						10_000 * DOLLARS,
					),
					(
						// TODO: Change before launch
						hex!["b67fe6413ffe5cf91ae38a6475c37deea70a25c6c86b3dd17bb82d09efd9b350"].into(),
						None,
						10_000 * DOLLARS,
					),
				],
				kilt_inflation_config(),
				hex!["d206033ba2eadf615c510f2c11f32d931b27442e5cfb64884afa2241dfa66e70"].into(),
				vec![
					(
						hex!["d206033ba2eadf615c510f2c11f32d931b27442e5cfb64884afa2241dfa66e70"].into(),
						hex!["d206033ba2eadf615c510f2c11f32d931b27442e5cfb64884afa2241dfa66e70"].unchecked_into(),
					),
					(
						hex!["b67fe6413ffe5cf91ae38a6475c37deea70a25c6c86b3dd17bb82d09efd9b350"].into(),
						hex!["b67fe6413ffe5cf91ae38a6475c37deea70a25c6c86b3dd17bb82d09efd9b350"].unchecked_into(),
					),
				],
				vec![
					hex!["d206033ba2eadf615c510f2c11f32d931b27442e5cfb64884afa2241dfa66e70"].into(),
					hex!["b67fe6413ffe5cf91ae38a6475c37deea70a25c6c86b3dd17bb82d09efd9b350"].into(),
				],
				id,
			)
		},
		Vec::new(),
		None,
		None,
		Some(properties),
		Extensions {
			relay_chain: "rococo_local_testnet".into(),
			para_id: id.into(),
		},
	))
}

pub fn load_rococo_spec() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../../res/mashnet-rococo.json")[..])
}

pub fn kilt_inflation_config() -> InflationInfo {
	InflationInfo::new(
		Perquintill::from_percent(10),
		Perquintill::from_percent(10),
		Perquintill::from_percent(40),
		Perquintill::from_percent(5),
	)
}

fn testnet_genesis(
	wasm_binary: &[u8],
	stakers: Vec<(AccountId, Option<AccountId>, Balance)>,
	inflation_config: InflationInfo,
	root_key: AccountId,
	initial_authorities: Vec<(AccountId, AuthorityId)>,
	endowed_accounts: Vec<AccountId>,
	id: ParaId,
) -> GenesisConfig {
	type VestingPeriod = BlockNumber;
	type LockingPeriod = BlockNumber;

	// vesting and locks as initially designed
	let airdrop_accounts_json = &include_bytes!("../../res/genesis-testing/genesis-accounts.json")[..];
	let airdrop_accounts: Vec<(AccountId, Balance, VestingPeriod, LockingPeriod)> =
		serde_json::from_slice(airdrop_accounts_json).expect("Could not read from genesis_accounts.json");

	GenesisConfig {
		frame_system: SystemConfig {
			code: wasm_binary.to_vec(),
			changes_trie_config: Default::default(),
		},
		pallet_balances: BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, 10000000000000000000000000000_u128))
				.chain(airdrop_accounts.iter().cloned().map(|(who, total, _, _)| (who, total)))
				.collect(),
		},
		pallet_sudo: SudoConfig { key: root_key },
		parachain_info: ParachainInfoConfig { parachain_id: id },
		kilt_launch: KiltLaunchConfig {
			balance_locks: airdrop_accounts
				.iter()
				.cloned()
				.map(|(who, amount, _, locking_length)| (who, locking_length * MINUTES, amount))
				.collect(),
			vesting: airdrop_accounts
				.iter()
				.cloned()
				.map(|(who, amount, vesting_length, _)| (who, vesting_length * MINUTES, amount))
				.collect(),
			// TODO: Set this to another address (PRE-LAUNCH)
			transfer_account: hex!["6a3c793cec9dbe330b349dc4eea6801090f5e71f53b1b41ad11afb4a313a282c"].into(),
		},
		pallet_vesting: VestingConfig { vesting: vec![] },
		pallet_collective_Instance1: CouncilConfig {
			members: vec![],
			phantom: Default::default(),
		},
		pallet_collective_Instance2: TechnicalCommitteeConfig {
			members: vec![],
			phantom: Default::default(),
		},
		pallet_treasury: Default::default(),
		pallet_elections_phragmen: Default::default(),
		pallet_membership: Default::default(),
		pallet_democracy: Default::default(),
		parachain_staking: ParachainStakingConfig {
			stakers,
			inflation_config,
		},
		pallet_aura: Default::default(),
		cumulus_pallet_aura_ext: Default::default(),
		pallet_session: SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|(acc, key)| {
					(
						acc.clone(),
						acc.clone(),
						kilt_parachain_runtime::opaque::SessionKeys { aura: key.clone() },
					)
				})
				.collect::<Vec<_>>(),
		},
	}
}
