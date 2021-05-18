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
use kilt_parachain_runtime::{
	BalancesConfig, CouncilConfig, GenesisConfig, KiltLaunchConfig, ParachainInfoConfig, SudoConfig, SystemConfig,
	TechnicalCommitteeConfig, VestingConfig, WASM_BINARY,
};
use kilt_primitives::{constants::MONTHS, AccountId, AccountPublic, Balance, BlockNumber};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::{ChainType, Properties};
use serde::{Deserialize, Serialize};
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::IdentifyAccount;

use hex_literal::hex;

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
	/// The relay chain of the Parachain.
	pub relay_chain: String,
	/// The id of the Parachain.
	pub para_id: u32,
}

impl Extensions {
	/// Try to get the extension from the given `ChainSpec`.
	pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
		sc_chain_spec::get_extension(chain_spec.extensions())
	}
}

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

pub fn get_properties(symbol: &str, decimals: u32, ss58format: u32) -> Properties {
	let mut properties = Properties::new();
	properties.insert("tokenSymbol".into(), symbol.into());
	properties.insert("tokenDecimals".into(), decimals.into());
	properties.insert("ss58Format".into(), ss58format.into());

	properties
}

pub fn get_chain_spec(id: ParaId) -> Result<ChainSpec, String> {
	let properties = get_properties("KILT", 15, 38);
	let wasm = WASM_BINARY.ok_or("No WASM")?;

	Ok(ChainSpec::from_genesis(
		"KILT Collator Local Testnet",
		"kilt_parachain_local_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				wasm,
				get_account_id_from_seed::<sr25519::Public>("Alice"),
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

pub fn staging_test_net(id: ParaId) -> Result<ChainSpec, String> {
	let properties = get_properties("KILT", 15, 38);
	let wasm = WASM_BINARY.ok_or("No WASM")?;

	Ok(ChainSpec::from_genesis(
		"KILT Collator Staging Testnet",
		"kilt_parachain_staging_testnet",
		ChainType::Live,
		move || {
			testnet_genesis(
				wasm,
				hex!["d206033ba2eadf615c510f2c11f32d931b27442e5cfb64884afa2241dfa66e70"].into(),
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

pub fn peregrine_test_net(id: ParaId) -> Result<ChainSpec, String> {
	let properties = get_properties("PKILT", 15, 38);
	let wasm = WASM_BINARY.ok_or("No WASM")?;

	Ok(ChainSpec::from_genesis(
		"KILT Collator Peregrine Testnet",
		"peregrine_kilt",
		ChainType::Live,
		move || {
			testnet_genesis(
				wasm,
				hex!["6419c4046cff92703299e9fa37fc100f2664677e6ee3d841735665005345f710"].into(),
				vec![
					hex!["6419c4046cff92703299e9fa37fc100f2664677e6ee3d841735665005345f710"].into(),
					hex!["369669429a18b273fb686bd9335c387bb5e8d98abfa33cda946fc7313483ed3f"].into(),
				],
				id,
			)
		},
		Vec::new(),
		None,
		None,
		Some(properties),
		Extensions {
			relay_chain: "peregrine_relay_testnet".into(),
			para_id: id.into(),
		},
	))
}

pub fn rococo_net() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../res/kilt-prod.json")[..])
}

fn testnet_genesis(
	wasm_binary: &[u8],
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	id: ParaId,
) -> GenesisConfig {
	type VestingPeriod = BlockNumber;
	type LockingPeriod = BlockNumber;

	// vesting and locks as initially designed
	let airdrop_accounts_json = &include_bytes!("../res/genesis-testing/genesis_accounts.json")[..];
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
				.map(|(who, amount, _, locking_length)| (who, locking_length * MONTHS, amount))
				.collect(),
			vesting: airdrop_accounts
				.iter()
				.cloned()
				.map(|(who, amount, vesting_length, _)| (who, vesting_length * MONTHS, amount))
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
	}
}
