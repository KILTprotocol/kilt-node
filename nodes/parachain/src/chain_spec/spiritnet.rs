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
use runtime_common::{
	constants::{kilt_inflation_config, staking::MinCollatorStake, KILT, MAX_COLLATOR_STAKE},
	AccountId, AuthorityId, Balance, BlockNumber,
};
use sc_service::ChainType;
use sc_telemetry::TelemetryEndpoints;
use sp_core::{crypto::UncheckedInto, sr25519};
use sp_runtime::traits::Zero;
use spiritnet_runtime::{
	BalancesConfig, CouncilConfig, GenesisConfig, InflationInfo, ParachainInfoConfig, ParachainStakingConfig,
	SessionConfig, SystemConfig, TechnicalCommitteeConfig, VestingConfig, WASM_BINARY,
};

use crate::chain_spec::{get_account_id_from_seed, get_from_seed, DEFAULT_PARA_ID, TELEMETRY_URL};

use super::{get_properties, Extensions};

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig, Extensions>;

pub fn get_chain_spec_dev() -> Result<ChainSpec, String> {
	let properties = get_properties("KILT", 15, 38);
	let wasm = WASM_BINARY.ok_or("No WASM")?;

	Ok(ChainSpec::from_genesis(
		"KILT Spiritnet Develop",
		"kilt_spiritnet_dev",
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

const WILT_COL_ACC_1: [u8; 32] = hex!["e6cf13c86a5f174acba79ca361dc429d89eb704c6a407af83f30b11ab8bc5045"];
const WILT_COL_SESSION_1: [u8; 32] = hex!["e29df39b74777495ca00cd7a316ce98c5225d7088ae924b122fe0e2e6a4b5569"];
const WILT_COL_ACC_2: [u8; 32] = hex!["e8ed0c2a40fb5a0bbb24c38f5c8cd83d79498ac029ac9f87497677f5701e3d2c"];
const WILT_COL_SESSION_2: [u8; 32] = hex!["7cacfbce640321ba84a85f41dfb43c2a2ea14ed789c096ad62ee0491599b0f44"];

pub fn get_chain_spec_wilt() -> Result<ChainSpec, String> {
	let properties = get_properties("WILT", 15, 38);
	let wasm = WASM_BINARY.ok_or("No WASM")?;
	let id: ParaId = 2085.into();

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
				vec![
					(WILT_COL_ACC_1.into(), WILT_COL_SESSION_1.unchecked_into()),
					(WILT_COL_ACC_2.into(), WILT_COL_SESSION_2.unchecked_into()),
				],
				vec![
					(WILT_COL_ACC_1.into(), 40000 * KILT),
					(WILT_COL_ACC_2.into(), 40000 * KILT),
				],
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
		Some(TelemetryEndpoints::new(vec![(TELEMETRY_URL.to_string(), 0)]).expect("WILT telemetry url is valid; qed")),
		None,
		None,
		Some(properties),
		Extensions {
			relay_chain: "westend".into(),
			para_id: id.into(),
		},
	))
}

const RILT_COL_ACC_1: [u8; 32] = hex!["6a5c355bca369a54c334542fd91cf70822be92f215a1049ceb04f36baba9b87b"];
const RILT_COL_SESSION_1: [u8; 32] = hex!["66c4ca0710c2c8a92504f281d992000508ce255543016545014cf0bfbbe71429"];
const RILT_COL_ACC_2: [u8; 32] = hex!["768538a941d1e4730c31830ab85a54ff34aaaad1f81bdd246db11802a57a5412"];
const RILT_COL_SESSION_2: [u8; 32] = hex!["7cff6c7a53c4630a0a35f8793a04b663681575bbfa43dbe5848b220bc4bd1963"];

pub fn get_chain_spec_rilt() -> Result<ChainSpec, String> {
	let properties = get_properties("RILT", 15, 38);
	let wasm = WASM_BINARY.ok_or("No WASM")?;
	let id: ParaId = 2086.into();

	Ok(ChainSpec::from_genesis(
		"RILT",
		"kilt_rococo",
		ChainType::Live,
		move || {
			testnet_genesis(
				wasm,
				vec![
					(RILT_COL_ACC_1.into(), None, 200_000 * KILT),
					(RILT_COL_ACC_2.into(), None, 200_000 * KILT),
				],
				kilt_inflation_config(),
				MAX_COLLATOR_STAKE,
				vec![
					(RILT_COL_ACC_1.into(), RILT_COL_SESSION_1.unchecked_into()),
					(RILT_COL_ACC_2.into(), RILT_COL_SESSION_2.unchecked_into()),
				],
				vec![
					(RILT_COL_ACC_1.into(), 1_000_000 * KILT),
					(RILT_COL_ACC_2.into(), 1_000_000 * KILT),
				],
				id,
			)
		},
		vec![
			"/dns4/bootnode.kilt.io/tcp/30365/p2p/12D3KooWS2h3rxqEC9bzrFNKVgrT1iaGz2UAWA1jVG1EB6dEoeJm"
				.parse()
				.expect("bootnode address is formatted correctly; qed"),
			"/dns4/bootnode.kilt.io/tcp/30366/p2p/12D3KooWMSF7Vefmpf67iGMkPrUgvXw38HoxaLmTNpYGYikFS7DZ"
				.parse()
				.expect("bootnode address is formatted correctly; qed"),
		],
		Some(TelemetryEndpoints::new(vec![(TELEMETRY_URL.to_string(), 0)]).expect("RILT telemetry url is valid; qed")),
		None,
		None,
		Some(properties),
		Extensions {
			relay_chain: "rococo".into(),
			para_id: id.into(),
		},
	))
}

pub fn load_rilt_spec() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../../res/rilt.json")[..])
}

pub fn load_spiritnet_spec() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../../res/spiritnet.json")[..])
}

#[allow(clippy::too_many_arguments)]
fn testnet_genesis(
	wasm_binary: &[u8],
	stakers: Vec<(AccountId, Option<AccountId>, Balance)>,
	inflation_config: InflationInfo,
	max_candidate_stake: Balance,
	initial_authorities: Vec<(AccountId, AuthorityId)>,
	endowed_accounts: Vec<(AccountId, Balance)>,
	id: ParaId,
) -> GenesisConfig {
	type VestingPeriod = BlockNumber;
	type LockingPeriod = BlockNumber;

	// vesting and locks as initially designed
	let claimable_accounts_json = &include_bytes!("../../res/genesis/claimable-accounts.json")[..];
	let claimable_accounts: Vec<(AccountId, Balance, VestingPeriod, LockingPeriod)> =
		serde_json::from_slice(claimable_accounts_json)
			.expect("The file genesis_accounts.json exists and is valid; qed");

	// botlabs account should not be migrated but some have vesting
	let owned_accounts_json = &include_bytes!("../../res/genesis/owned-accounts.json")[..];
	let owned_accounts: Vec<(AccountId, Balance, VestingPeriod, LockingPeriod)> =
		serde_json::from_slice(owned_accounts_json).expect("The file botlabs_accounts.json exists and is valid; qed");

	GenesisConfig {
		system: SystemConfig {
			code: wasm_binary.to_vec(),
		},
		balances: BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.chain(
					claimable_accounts
						.iter()
						.cloned()
						.map(|(who, total, _, _)| (who, total)),
				)
				.chain(owned_accounts.iter().cloned().map(|(who, total, _, _)| (who, total)))
				.collect(),
		},
		parachain_info: ParachainInfoConfig { parachain_id: id },
		vesting: VestingConfig {
			vesting: owned_accounts
				.iter()
				.cloned()
				.filter(|(_, _, vesting_length, _)| !vesting_length.is_zero())
				.map(|(who, _, vesting_length, _)| (who, 0u64, vesting_length, 0))
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
						spiritnet_runtime::SessionKeys { aura: key.clone() },
					)
				})
				.collect::<Vec<_>>(),
		},
		council: CouncilConfig {
			members: initial_authorities.iter().map(|(acc, _)| acc).cloned().collect(),
			phantom: Default::default(),
		},
		technical_committee: TechnicalCommitteeConfig {
			members: initial_authorities.iter().map(|(acc, _)| acc).cloned().collect(),
			phantom: Default::default(),
		},
		treasury: Default::default(),
		technical_membership: Default::default(),
		tips_membership: Default::default(),
		democracy: Default::default(),
	}
}
