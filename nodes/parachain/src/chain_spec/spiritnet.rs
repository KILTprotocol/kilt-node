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
use kilt_primitives::{
	constants::{KILT, MINUTES},
	AccountId, AuthorityId, Balance, BlockNumber,
};
use sc_service::ChainType;
use sp_core::{crypto::UncheckedInto, sr25519};
use sp_runtime::Perquintill;
use spiritnet_runtime::{
	BalancesConfig, GenesisConfig, InflationInfo, KiltLaunchConfig, MaxCollatorCandidateStk, MinCollatorStk,
	ParachainInfoConfig, ParachainStakingConfig, SessionConfig, SudoConfig, SystemConfig, VestingConfig, WASM_BINARY,
};

use crate::chain_spec::{get_account_id_from_seed, get_from_seed};

use super::{get_properties, Extensions};

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

pub fn get_chain_spec_dev(id: ParaId) -> Result<ChainSpec, String> {
	let properties = get_properties("KILT", 15, 38);
	let wasm = WASM_BINARY.ok_or("No WASM")?;

	Ok(ChainSpec::from_genesis(
		"KILT Local",
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
				hex!["6a3c793cec9dbe330b349dc4eea6801090f5e71f53b1b41ad11afb4a313a282c"].into(),
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

const WILT_COL_ACC_1: [u8; 32] = hex!["e6cf13c86a5f174acba79ca361dc429d89eb704c6a407af83f30b11ab8bc5045"];
const WILT_COL_SESSION_1: [u8; 32] = hex!["e29df39b74777495ca00cd7a316ce98c5225d7088ae924b122fe0e2e6a4b5569"];
const WILT_COL_ACC_2: [u8; 32] = hex!["e8ed0c2a40fb5a0bbb24c38f5c8cd83d79498ac029ac9f87497677f5701e3d2c"];
const WILT_COL_SESSION_2: [u8; 32] = hex!["7cacfbce640321ba84a85f41dfb43c2a2ea14ed789c096ad62ee0491599b0f44"];
const WILT_SUDO_ACC: [u8; 32] = hex!["200a316b25b3683459585ec746042f6841640e3b9f111028426ff17e9090005d"];
const WILT_TRANS_ACC: [u8; 32] = hex!["aaf5308b81f962ffdaccaa22352cc95b7bef70033d9d0d5a7023ec5681f05954"];

pub fn get_chain_spec_wilt() -> Result<ChainSpec, String> {
	let properties = get_properties("WILT", 15, 38);
	let wasm = WASM_BINARY.ok_or("No WASM")?;
	let id: ParaId = 2009.into();

	Ok(ChainSpec::from_genesis(
		"WILT",
		"kilt_westend",
		ChainType::Live,
		move || {
			testnet_genesis(
				wasm,
				vec![
					(WILT_COL_ACC_1.into(), None, 30000 * KILT),
					(WILT_COL_ACC_2.into(), None, 30000 * KILT),
				],
				kilt_inflation_config(),
				WILT_SUDO_ACC.into(),
				vec![
					(WILT_COL_ACC_1.into(), WILT_COL_SESSION_1.unchecked_into()),
					(WILT_COL_ACC_2.into(), WILT_COL_SESSION_2.unchecked_into()),
				],
				vec![
					(WILT_COL_ACC_1.into(), 40000 * KILT),
					(WILT_COL_ACC_2.into(), 40000 * KILT),
					(WILT_SUDO_ACC.into(), 10000 * KILT),
					(WILT_TRANS_ACC.into(), 10000 * KILT),
				],
				WILT_TRANS_ACC.into(),
				id,
			)
		},
		vec![],
		None,
		None,
		Some(properties),
		Extensions {
			relay_chain: "westend".into(),
			para_id: id.into(),
		},
	))
}

const SPIRIT_COL_ACC_1: [u8; 32] = hex!["dcae0b0169a344cbb3800ea34f438f5139687922ce34bfaf097a1314f5ee9069"];
const SPIRIT_COL_SESSION_1: [u8; 32] = hex!["c2ef6ae55020c046a76e745133b75db6d10955f78d2224048aae6ac6f763fb6c"];
const SPIRIT_COL_ACC_2: [u8; 32] = hex!["86e876d2aa97cc87a8b83b78b748f8795cac40883e6c6fc023f3fde3a094623d"];
const SPIRIT_COL_SESSION_2: [u8; 32] = hex!["32280e5e31512fd4863bada1ab7f9ae0892bb1f9eb1d3506673c6de1ae90fe40"];
const SPIRIT_SUDO_ACC: [u8; 32] = hex!["c48ed216c1ae656a501016efaaef59f4eb8778c64f84b245da3cc19321d4c22a"];
const SPIRIT_TRANS_ACC: [u8; 32] = hex!["7a8604da79a9f89db6b35efdef3c4c84f9ae679b2fbd397f5bade3105a8a8e00"];

pub fn get_chain_spec_spiritnet() -> Result<ChainSpec, String> {
	let properties = get_properties("KILT", 15, 38);
	let wasm = WASM_BINARY.ok_or("No WASM")?;
	let id: ParaId = 2005.into();

	Ok(ChainSpec::from_genesis(
		"KILT Spiritnet",
		"kilt",
		ChainType::Live,
		move || {
			testnet_genesis(
				wasm,
				vec![
					(SPIRIT_COL_ACC_1.into(), None, MaxCollatorCandidateStk::get()),
					(SPIRIT_COL_ACC_2.into(), None, MaxCollatorCandidateStk::get()),
				],
				kilt_inflation_config(),
				SPIRIT_SUDO_ACC.into(),
				vec![
					(SPIRIT_COL_ACC_1.into(), SPIRIT_COL_SESSION_1.unchecked_into()),
					(SPIRIT_COL_ACC_2.into(), SPIRIT_COL_SESSION_2.unchecked_into()),
				],
				vec![
					(SPIRIT_COL_ACC_1.into(), MaxCollatorCandidateStk::get() + 100 * KILT),
					(SPIRIT_COL_ACC_2.into(), MaxCollatorCandidateStk::get() + 100 * KILT),
					(SPIRIT_SUDO_ACC.into(), 10000 * KILT),
					(SPIRIT_TRANS_ACC.into(), 10000 * KILT),
				],
				SPIRIT_TRANS_ACC.into(),
				id,
			)
		},
		vec![],
		None,
		None,
		Some(properties),
		Extensions {
			relay_chain: "kusama".into(),
			para_id: id.into(),
		},
	))
}

pub fn load_spiritnet_spec() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../../res/spiritnet.json")[..])
}

pub fn kilt_inflation_config() -> InflationInfo {
	InflationInfo::new(
		Perquintill::from_percent(10),
		Perquintill::from_percent(10),
		Perquintill::from_percent(40),
		Perquintill::from_percent(5),
	)
}

#[allow(clippy::too_many_arguments)]
fn testnet_genesis(
	wasm_binary: &[u8],
	stakers: Vec<(AccountId, Option<AccountId>, Balance)>,
	inflation_config: InflationInfo,
	root_key: AccountId,
	initial_authorities: Vec<(AccountId, AuthorityId)>,
	endowed_accounts: Vec<(AccountId, Balance)>,
	transfer_account: AccountId,
	id: ParaId,
) -> GenesisConfig {
	type VestingPeriod = BlockNumber;
	type LockingPeriod = BlockNumber;

	// vesting and locks as initially designed
	let airdrop_accounts_json = &include_bytes!("../../res/genesis/genesis-accounts.json")[..];
	let airdrop_accounts: Vec<(AccountId, Balance, VestingPeriod, LockingPeriod)> =
		serde_json::from_slice(airdrop_accounts_json).expect("Could not read from genesis_accounts.json");

	GenesisConfig {
		system: SystemConfig {
			code: wasm_binary.to_vec(),
			changes_trie_config: Default::default(),
		},
		balances: BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.chain(airdrop_accounts.iter().cloned().map(|(who, total, _, _)| (who, total)))
				.collect(),
		},
		sudo: SudoConfig { key: root_key },
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
			transfer_account,
		},
		vesting: VestingConfig { vesting: vec![] },
		parachain_staking: ParachainStakingConfig {
			stakers,
			inflation_config,
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
						spiritnet_runtime::opaque::SessionKeys { aura: key.clone() },
					)
				})
				.collect::<Vec<_>>(),
		},
	}
}
