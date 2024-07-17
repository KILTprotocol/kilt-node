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

use kestrel_runtime::{opaque::SessionKeys, RuntimeGenesisConfig, WASM_BINARY};
use runtime_common::{AccountId, AccountPublic};

use sc_service::{self, ChainType, Properties};
use sp_consensus_aura::ed25519::AuthorityId as AuraId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{ed25519, sr25519, Pair, Public};
use sp_runtime::traits::IdentifyAccount;

pub(crate) fn load_spec(id: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
	let chain_spec = match id {
		// Dev chainspec, used for SDK integration tests
		"dev" => Ok::<_, String>(generate_dev_chain_spec()),
		_ => return Err(format!("Unknown spec: {}", id)),
	}?;
	Ok(Box::new(chain_spec))
}

type ChainSpec = sc_service::GenericChainSpec<RuntimeGenesisConfig>;

fn generate_dev_chain_spec() -> ChainSpec {
	let wasm_binary = WASM_BINARY.expect("Development WASM binary not available");
	let properties = Properties::from_iter([
		("tokenDecimals".into(), 15.into()),
		("tokenSymbol".into(), "DILT".into()),
	]);

	ChainSpec::builder(wasm_binary, None)
		.with_name("Standalone Node (Dev)")
		.with_id("standalone_node_development")
		.with_chain_type(ChainType::Development)
		.with_properties(properties)
		.with_genesis_config(generate_genesis_state())
		.build()
}

fn generate_genesis_state() -> serde_json::Value {
	let endowed_accounts = vec![
		// Dev Faucet account
		get_account_id_from_secret::<ed25519::Public>(
			"receive clutch item involve chaos clutch furnace arrest claw isolate okay together",
		),
		get_account_id_from_secret::<ed25519::Public>("//Alice"),
		get_account_id_from_secret::<ed25519::Public>("//Bob"),
		get_account_id_from_secret::<sr25519::Public>("//Alice"),
		get_account_id_from_secret::<sr25519::Public>("//Bob"),
	];

	let initial_authorities = vec![get_authority_keys_from_secret("//Alice")];
	let root_key = get_account_id_from_secret::<ed25519::Public>("//Alice");

	serde_json::json!({
		"balances": {
			"balances": endowed_accounts.iter().cloned().map(|k| (k, 1u64 << 60)).collect::<Vec<_>>(),
		},
		"session": {
			"keys": initial_authorities
				.into_iter()
				.map(|(acc, aura, grandpa)| {
					(
						acc.clone(),                 // account id
						acc,                         // validator id
						template_session_keys(aura, grandpa), // session keys
					)
				})
			.collect::<Vec<_>>(),
		},

		"sudo": { "key": Some(root_key) }
	})
}

fn template_session_keys(aura_keys: AuraId, grandpa_keys: GrandpaId) -> SessionKeys {
	SessionKeys {
		aura: aura_keys,
		grandpa: grandpa_keys,
	}
}

fn get_authority_keys_from_secret(seed: &str) -> (AccountId, AuraId, GrandpaId) {
	(
		get_account_id_from_secret::<ed25519::Public>(seed),
		get_public_key_from_secret::<AuraId>(seed),
		get_public_key_from_secret::<GrandpaId>(seed),
	)
}

fn get_account_id_from_secret<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_public_key_from_secret::<TPublic>(seed)).into_account()
}

fn get_public_key_from_secret<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(seed, None)
		.unwrap_or_else(|_| panic!("Invalid string '{}'", seed))
		.public()
}
