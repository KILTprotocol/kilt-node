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

use kestrel_runtime::{
	BalancesConfig, IndicesConfig, RuntimeGenesisConfig, SessionConfig, SudoConfig, SystemConfig, WASM_BINARY,
};
use runtime_common::{AccountId, AccountPublic};

use sc_service::{self, ChainType, Properties};
use sp_consensus_aura::ed25519::AuthorityId as AuraId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{ed25519, sr25519, Pair, Public};
use sp_runtime::traits::IdentifyAccount;

pub(crate) fn load_spec(id: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
	Ok(Box::new(match id {
		// Dev chainspec, used for SDK integration tests
		"dev" => generate_dev_chain_spec()?,
		_ => return Err(format!("Unknown spec: {}", id)),
	}))
}

type ChainSpec = sc_service::GenericChainSpec<RuntimeGenesisConfig>;

fn generate_dev_chain_spec() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development WASM binary not available".to_string())?;

	let properties = Properties::from_iter(
		[
			("tokenDecimals".into(), 15.into()),
			("tokenSymbol".into(), "DILT".into()),
		]
		.into_iter(),
	);

	Ok(ChainSpec::from_genesis(
		"Standalone Node (Dev)",
		"standalone_node_development",
		ChainType::Development,
		move || {
			generate_devnet_genesis_state(
				wasm_binary,
				vec![get_authority_keys_from_secret("//Alice")],
				get_account_id_from_secret::<ed25519::Public>("//Alice"),
				vec![
					// Dev Faucet account
					get_account_id_from_secret::<ed25519::Public>(
						"receive clutch item involve chaos clutch furnace arrest claw isolate okay together",
					),
					get_account_id_from_secret::<ed25519::Public>("//Alice"),
					get_account_id_from_secret::<ed25519::Public>("//Bob"),
					get_account_id_from_secret::<sr25519::Public>("//Alice"),
					get_account_id_from_secret::<sr25519::Public>("//Bob"),
				],
			)
		},
		vec![],
		None,
		None,
		None,
		Some(properties),
		None,
	))
}

fn generate_devnet_genesis_state(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AccountId, AuraId, GrandpaId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
) -> RuntimeGenesisConfig {
	RuntimeGenesisConfig {
		system: SystemConfig {
			code: wasm_binary.to_vec(),
			..Default::default()
		},
		indices: IndicesConfig { indices: vec![] },
		transaction_payment: Default::default(),
		balances: BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|a| (a, 1u128 << 90)).collect(),
		},
		session: SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| {
					(
						x.0.clone(),
						x.0.clone(),
						kestrel_runtime::opaque::SessionKeys {
							aura: x.1.clone(),
							grandpa: x.2.clone(),
						},
					)
				})
				.collect::<Vec<_>>(),
		},
		aura: Default::default(),
		grandpa: Default::default(),
		sudo: SudoConfig { key: Some(root_key) },
		did_lookup: Default::default(),
	}
}

fn get_authority_keys_from_secret(seed: &str) -> (AccountId, AuraId, GrandpaId) {
	(
		get_account_id_from_secret::<ed25519::Public>(seed),
		get_from_secret::<AuraId>(seed),
		get_from_secret::<GrandpaId>(seed),
	)
}

fn get_account_id_from_secret<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_secret::<TPublic>(seed)).into_account()
}

fn get_from_secret<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(seed, None)
		.unwrap_or_else(|_| panic!("Invalid string '{}'", seed))
		.public()
}
