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

mod develop;
mod rilt;
mod testnet;

use cumulus_primitives_core::ParaId;
use sp_runtime::traits::Zero;

use crate::chain_spec::Extensions;
use peregrine_runtime::{
	BalancesConfig, CouncilConfig, InflationInfo, ParachainInfoConfig, ParachainStakingConfig, PolkadotXcmConfig,
	RuntimeGenesisConfig, SessionConfig, SudoConfig, SystemConfig, TechnicalCommitteeConfig, VestingConfig,
};
use runtime_common::{AccountId, AuthorityId, Balance, BlockNumber};

pub use develop::get_chain_spec_dev;
pub use rilt::{get_chain_spec_rilt, load_rilt_spec};
pub use testnet::make_new_spec;

const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<RuntimeGenesisConfig, Extensions>;

#[allow(clippy::too_many_arguments)]
fn testnet_genesis(
	wasm_binary: &[u8],
	stakers: Vec<(AccountId, Option<AccountId>, Balance)>,
	inflation_config: InflationInfo,
	max_candidate_stake: Balance,
	initial_authorities: Vec<(AccountId, AuthorityId)>,
	endowed_accounts: Vec<(AccountId, Balance)>,
	id: ParaId,
	root_key: AccountId,
) -> RuntimeGenesisConfig {
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

	RuntimeGenesisConfig {
		system: SystemConfig {
			code: wasm_binary.to_vec(),
			..Default::default()
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
		sudo: SudoConfig { key: Some(root_key) },
		parachain_info: ParachainInfoConfig {
			parachain_id: id,
			..Default::default()
		},
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
						peregrine_runtime::SessionKeys { aura: key.clone() },
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
		polkadot_xcm: PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
			..Default::default()
		},
		did_lookup: Default::default(),
	}
}
