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
	constants::{INFLATION_CONFIG, KILT, MAX_COLLATOR_STAKE, MINUTES},
	AccountId, AuthorityId, Balance, BlockNumber,
};
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use sp_core::{crypto::UncheckedInto, sr25519};
use sp_runtime::traits::Zero;
use spiritnet_runtime::{
	BalancesConfig, GenesisConfig, InflationInfo, KiltLaunchConfig, MinCollatorStake, ParachainInfoConfig,
	ParachainStakingConfig, SessionConfig, SudoConfig, SystemConfig, VestingConfig, WASM_BINARY,
};

use crate::chain_spec::{get_account_id_from_seed, get_from_seed, TELEMETRY_URL};

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
						2 * MinCollatorStake::get(),
					),
					(
						// TODO: Change before launch
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
	let id: ParaId = 2078.into();

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
				MAX_COLLATOR_STAKE,
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
		vec![
			"/dns4/bootnode.kilt.io/tcp/30360/p2p/12D3KooWRPR7q1Rgwurd4QGyUUbVnN4nXYNVzbLeuhFsd9eXmHJk"
				.parse()
				.expect("bootnode address is formatted correctly; qed"),
			"/dns4/bootnode.kilt.io/tcp/30361/p2p/12D3KooWDAEqpTRsL76itsabbh4SeaqtCM6v9npQ8eCeqPbbuFE9"
				.parse()
				.expect("bootnode address is formatted correctly; qed"),
		],
		Some(TelemetryEndpoints::new(vec![(TELEMETRY_URL.to_string(), 0)]).expect("KILT telemetry url is valid; qed")),
		None,
		Some(properties),
		Extensions {
			relay_chain: "westend".into(),
			para_id: id.into(),
		},
	))
}

const SPIRIT_COL_ACC_01: [u8; 32] = hex!["c48ed216c1ae656a501016efaaef59f4eb8778c64f84b245da3cc19321d4c22a"];
const SPIRIT_COL_SESSION_01: [u8; 32] = hex!["709dddf36d5741239071b3537421f4ea620ddef1f20f82ca86d290fe0cb1d17e"];
const SPIRIT_COL_ACC_02: [u8; 32] = hex!["628da4055a812ca4145c75c38734138b6c62f0402ff1feae649be54d4c42c32e"];
const SPIRIT_COL_SESSION_02: [u8; 32] = hex!["1cbebf801ded95b160f683469d61c4dc85653bc14d9e2bdcd72eb985aac19943"];

const SPIRIT_COL_ACC_03: [u8; 32] = hex!["ec9b89167a547f11a13ef71c9fea326d66b067d3dfd83744c83fda31d7fd4171"];
const SPIRIT_COL_SESSION_03: [u8; 32] = hex!["30f7db5a399cb77cc55515fb8b85b3d39ac55b5fdd6b236a34248c75fbd90e60"];
const SPIRIT_COL_ACC_04: [u8; 32] = hex!["10abf8ffbb90d92395d891a798e4476ade3d28ae5f8c5c955b9199d995871a62"];
const SPIRIT_COL_SESSION_04: [u8; 32] = hex!["9433021147aae2723b197e25b0089bc2fce0f8e3f5b61178c45ef227abb4c22b"];
const SPIRIT_COL_ACC_05: [u8; 32] = hex!["1664016a5caab8f5b4b1360b1e05b4aa84c1970f30a19d49ae4f40312404a538"];
const SPIRIT_COL_SESSION_05: [u8; 32] = hex!["8a5c403f5cc5ca297b7fef06cad3a4e8bc8a0310d92adb5bbc2fbca116e0441a"];
const SPIRIT_COL_ACC_06: [u8; 32] = hex!["fa53c9aba42da8645c332d15272cae9b939de4181b8ea261e3cb9c2e79e1dd36"];
const SPIRIT_COL_SESSION_06: [u8; 32] = hex!["2ef0912b021321a65e2ab2e146796b1d5ddcd47298864be38edf5c70b869864f"];
const SPIRIT_COL_ACC_07: [u8; 32] = hex!["0000000000000000000000000000000000000000000000000000000000000000"];
const SPIRIT_COL_SESSION_07: [u8; 32] = hex!["4cd52e3d6742d5e090f5977cb5413d61a651b5bb643ac65b0aee61d1c0a68a0c"];
const SPIRIT_COL_ACC_08: [u8; 32] = hex!["5270ec35ba01254d8bff046a1a58f16d3ae615c235efd6e99a35f233b2d9df2c"];
const SPIRIT_COL_SESSION_08: [u8; 32] = hex!["e4acf473fa03cc55ddf05ab58d0816cc38f5a82128c90af7174080b186db2555"];
const SPIRIT_COL_ACC_09: [u8; 32] = hex!["ec4635974882ab477e60fc38ea72f42636d60f0433a5c1ce2e5f3b4c9a879fe9"];
const SPIRIT_COL_SESSION_09: [u8; 32] = hex!["64007bee2fd9d23e6cc6cf7bbb55badee773d5d99665ab0d3f3dd9415ef0585d"];
const SPIRIT_COL_ACC_10: [u8; 32] = hex!["be1dbcf4234b70c81a518378e72b67aa7cd06b122ebb7658562472167e8e231c"];
const SPIRIT_COL_SESSION_10: [u8; 32] = hex!["d08bae71b7184947656646fb5155e874632625f8227f299847d11516ca803e21"];
const SPIRIT_COL_ACC_11: [u8; 32] = hex!["4294c4ffa38dc95fb9c57fc9f82ded3d02336b78f137c08636b849f9eb9ca60f"];
const SPIRIT_COL_SESSION_11: [u8; 32] = hex!["60d1411a7316ac8aeca5750ddfb509c03adf1ac41a259be62bf57d9b9ec5bd29"];
const SPIRIT_COL_ACC_12: [u8; 32] = hex!["6c9783e922b00e288d19b9020ffce919d1ab4ab20fc62d5d0b23ac1e61b2096b"];
const SPIRIT_COL_SESSION_12: [u8; 32] = hex!["18a3abde01c55ee58633668c97c9855a6772f3d97f0b89ce8f60842a65af3212"];
const SPIRIT_COL_ACC_13: [u8; 32] = hex!["eadbf15cd28e209358f6cef3b139a71e428254c2072948782bd94e9e8fcd3608"];
const SPIRIT_COL_SESSION_13: [u8; 32] = hex!["d8e4758a39e623ef7e7841747fa788891bdd3b5f775cec05d0d2dd22fdfa2259"];
const SPIRIT_COL_ACC_14: [u8; 32] = hex!["68a7a05e316865a68968f92b09e9ec6d5d1847e71e520e68e5d306b5730c2468"];
const SPIRIT_COL_SESSION_14: [u8; 32] = hex!["3cf5f477056588cdb198e03852b1f4d1e1abe89f854a271e38e892c14f15182d"];
const SPIRIT_COL_ACC_15: [u8; 32] = hex!["66241b171bd08521a33dc807061373a092faea04252f58b70792e7c7ecfd500e"];
const SPIRIT_COL_SESSION_15: [u8; 32] = hex!["f0d87466a633cf242237b5807262e90f199704c07fc9c3d9bda861dc67ef6a32"];
const SPIRIT_COL_ACC_16: [u8; 32] = hex!["4ebcfdc6cdc35cd99a8bb93c64ea81bb30731c2f01b0464bb400bad3f0d4b61f"];
const SPIRIT_COL_SESSION_16: [u8; 32] = hex!["849769562be9e7f744754e79a74fd3479388e59d278e8fc2fd073f82f0799604"];

const SPIRIT_SUDO_ACC: [u8; 32] = hex!["427b946b2cee9bd4ed03982e6f716d4eeaa5dc8410255e7a8ecf8d0080effe24"];
const SPIRIT_TRANS_ACC: [u8; 32] = hex!["de28ef5b1691663300a2edb97202791e89bb6985ffdaa4c405d68c826b634b76"];

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
					(SPIRIT_COL_ACC_01.into(), None, MAX_COLLATOR_STAKE),
					(SPIRIT_COL_ACC_02.into(), None, MAX_COLLATOR_STAKE),
				],
				kilt_inflation_config(),
				MAX_COLLATOR_STAKE,
				SPIRIT_SUDO_ACC.into(),
				vec![
					(SPIRIT_COL_ACC_01.into(), SPIRIT_COL_SESSION_01.unchecked_into()),
					(SPIRIT_COL_ACC_02.into(), SPIRIT_COL_SESSION_02.unchecked_into()),
				],
				vec![
					(SPIRIT_COL_ACC_01.into(), MAX_COLLATOR_STAKE + 100 * KILT),
					(SPIRIT_COL_ACC_02.into(), MAX_COLLATOR_STAKE + 100 * KILT),
					(SPIRIT_SUDO_ACC.into(), 10000 * KILT),
					(SPIRIT_TRANS_ACC.into(), 10000 * KILT),
				],
				SPIRIT_TRANS_ACC.into(),
				id,
			)
		},
		vec![
			"/dns4/bootnode.kilt.io/tcp/30390/p2p/12D3KooWRPR7q1Rgwurd4QGyUUbVnN4nXYNVzbLeuhFsd9eXmHJk"
				.parse()
				.expect("bootnode address is formatted correctly; qed"),
			"/dns4/bootnode.kilt.io/tcp/30391/p2p/12D3KooWDAEqpTRsL76itsabbh4SeaqtCM6v9npQ8eCeqPbbuFE9"
				.parse()
				.expect("bootnode address is formatted correctly; qed"),
		],
		Some(TelemetryEndpoints::new(vec![(TELEMETRY_URL.to_string(), 0)]).expect("KILT telemetry url is valid; qed")),
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
	InflationInfo::from(INFLATION_CONFIG)
}

#[allow(clippy::too_many_arguments)]
fn testnet_genesis(
	wasm_binary: &[u8],
	stakers: Vec<(AccountId, Option<AccountId>, Balance)>,
	inflation_config: InflationInfo,
	max_candidate_stake: Balance,
	root_key: AccountId,
	initial_authorities: Vec<(AccountId, AuthorityId)>,
	endowed_accounts: Vec<(AccountId, Balance)>,
	transfer_account: AccountId,
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
			changes_trie_config: Default::default(),
		},
		balances: BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.chain(airdrop_accounts.iter().cloned().map(|(who, total, _, _)| (who, total)))
				.chain(botlabs_accounts.iter().cloned().map(|(who, total, _, _)| (who, total)))
				.collect(),
		},
		sudo: SudoConfig { key: root_key },
		parachain_info: ParachainInfoConfig { parachain_id: id },
		kilt_launch: KiltLaunchConfig {
			vesting: airdrop_accounts
				.iter()
				.cloned()
				.map(|(who, amount, vesting_length, _)| (who, vesting_length * MINUTES, amount))
				.collect(),
			balance_locks: airdrop_accounts
				.iter()
				.cloned()
				.map(|(who, amount, _, locking_length)| (who, locking_length * MINUTES, amount))
				.collect(),
			// TODO: Set this to another address (PRE-LAUNCH)
			transfer_account,
		},
		vesting: VestingConfig {
			vesting: botlabs_accounts
				.iter()
				.cloned()
				.filter(|(_, _, vesting_length, _)| !vesting_length.is_zero())
				.map(|(who, amount, vesting_length, _)| (who, 0u64, vesting_length * MINUTES, amount))
				.collect(),
		},
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
						spiritnet_runtime::opaque::SessionKeys { aura: key.clone() },
					)
				})
				.collect::<Vec<_>>(),
		},
	}
}
