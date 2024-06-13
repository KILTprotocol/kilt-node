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

use peregrine_runtime::{SessionKeys, WASM_BINARY};
use runtime_common::{
	constants::{kilt_inflation_config, staking::MinCollatorStake, KILT, MAX_COLLATOR_STAKE},
	AccountId, AuthorityId,
};
use sc_service::ChainType;
use sp_core::sr25519;

use crate::chain_spec::{
	peregrine::{ChainSpec, SAFE_XCM_VERSION},
	utils::{get_account_id_from_secret, get_properties, get_public_key_from_secret},
	Extensions, KILT_PARA_ID,
};

pub(crate) fn generate_chain_spec(relaychain_name: &str) -> ChainSpec {
	let wasm_binary = WASM_BINARY.expect("Development WASM binary not available");

	ChainSpec::builder(
		wasm_binary,
		Extensions {
			relay_chain: relaychain_name.into(),
			para_id: KILT_PARA_ID,
		},
	)
	.with_name("KILT Peregrine Develop")
	.with_id("kilt_peregrine_dev")
	.with_chain_type(ChainType::Development)
	.with_properties(get_properties("PILT", 15, 38))
	.with_genesis_config_patch(get_genesis_config())
	.build()
}

fn get_genesis_config() -> serde_json::Value {
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

	let initial_authorities = vec![alice.clone(), bob.clone()];

	let stakers = [alice.clone(), bob.clone()]
		.into_iter()
		.map(|(acc, _)| -> (AccountId, Option<AccountId>, u128) { (acc, None, 2 * MinCollatorStake::get()) })
		.collect::<Vec<_>>();

	let balances = endowed_accounts
		.iter()
		.cloned()
		.map(|acc| (acc, 1_000_000 * KILT))
		.collect::<Vec<_>>();

	let keys = initial_authorities
		.into_iter()
		.map(|(acc, aura)| (acc.clone(), acc, SessionKeys { aura }))
		.collect::<Vec<_>>();

	let members = vec![alice.clone().0, bob.clone().0];

	serde_json::json!({
		"balances": {
			"balances": balances,
		},
		"session": {
			"keys": keys,
		},
		"sudo": { "key": Some(alice.0) },
		"parachainInfo": {
			"parachainId": KILT_PARA_ID,
		},
		"parachainStaking": {
			"stakers": stakers,
			"inflationConfig": kilt_inflation_config(),
			"maxCandidateStake": MAX_COLLATOR_STAKE,
		},
		"council": {
			"members": members,
		},
		"technicalCommittee": {
			"members": members,
		},
		"polkadotXcm": {
			"safeXcmVersion": Some(SAFE_XCM_VERSION),
		}
	})
}
