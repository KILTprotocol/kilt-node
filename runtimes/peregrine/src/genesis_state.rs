// KILT Blockchain – <https://kilt.io>
// Copyright (C) 2025, KILT Foundation

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

// If you feel like getting in touch with us, you can do so at <hello@kilt.io>
#![allow(clippy::expect_used)]

use crate::{
	BalancesConfig, CouncilConfig, ParachainInfoConfig, ParachainStakingConfig, PolkadotXcmConfig,
	RuntimeGenesisConfig, SessionConfig, SessionKeys, SudoConfig, TechnicalCommitteeConfig,
};
use runtime_common::{
	constants::{kilt_inflation_config, staking::MinCollatorStake, KILT, MAX_COLLATOR_STAKE},
	get_account_id_from_secret, get_public_key_from_secret, AccountId, AuthorityId, Balance,
};
use sp_core::sr25519;
use sp_genesis_builder::PresetId;
use sp_std::{vec, vec::Vec};

pub const KILT_PARA_ID: u32 = 2_086;
const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;
const NEW_RUNTIME_PRESET: &str = "new";

pub mod development {

	use super::*;

	pub fn generate_genesis_state() -> serde_json::Value {
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

		let config = RuntimeGenesisConfig {
			balances: BalancesConfig {
				balances: endowed_accounts.map(|acc| (acc, 10_000_000 * KILT)).to_vec(),
			},
			session: SessionConfig {
				keys: [alice.clone(), bob.clone()]
					.map(|(acc, key)| (acc.clone(), acc, SessionKeys { aura: key }))
					.to_vec(),
				..Default::default()
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
					.map(|(acc, _)| -> (AccountId, Option<AccountId>, Balance) {
						(acc, None, 2u128.saturating_mul(MinCollatorStake::get()))
					})
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
		};

		serde_json::to_value(config).expect("Could not build genesis config.")
	}
}

pub mod production {
	use super::*;

	pub fn generate_genesis_state() -> serde_json::Value {
		let config = RuntimeGenesisConfig {
			parachain_info: ParachainInfoConfig {
				parachain_id: KILT_PARA_ID.into(),
				..Default::default()
			},
			polkadot_xcm: PolkadotXcmConfig {
				safe_xcm_version: Some(SAFE_XCM_VERSION),
				..Default::default()
			},
			..Default::default()
		};

		serde_json::to_value(config).expect("Could not build genesis config.")
	}
}

/// Provides the JSON representation of predefined genesis config for given
/// `id`.
pub fn get_preset(id: &PresetId) -> Option<vec::Vec<u8>> {
	let patch = match id.try_into() {
		Ok(sp_genesis_builder::DEV_RUNTIME_PRESET) | Ok(sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET) => {
			development::generate_genesis_state()
		}
		Ok(NEW_RUNTIME_PRESET) => production::generate_genesis_state(),
		_ => return None,
	};

	Some(
		serde_json::to_string(&patch)
			.expect("serialization to json is expected to work. qed.")
			.into_bytes(),
	)
}

/// List of supported presets.
pub fn preset_names() -> Vec<PresetId> {
	vec![
		PresetId::from(sp_genesis_builder::DEV_RUNTIME_PRESET),
		PresetId::from(sp_genesis_builder::LOCAL_TESTNET_RUNTIME_PRESET),
		PresetId::from(NEW_RUNTIME_PRESET),
	]
}
