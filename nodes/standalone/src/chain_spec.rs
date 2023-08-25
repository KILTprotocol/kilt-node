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

use kestrel_runtime::{
	BalancesConfig, GenesisConfig, IndicesConfig, SessionConfig, SudoConfig, SystemConfig, WASM_BINARY,
};
use runtime_common::{AccountId, AccountPublic};

use hex_literal::hex;
use sc_service::{self, ChainType, Properties};
use sp_consensus_aura::ed25519::AuthorityId as AuraId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{crypto::UncheckedInto, ed25519, sr25519, Pair, Public};
use sp_runtime::traits::IdentifyAccount;

// Note this is the URL for the telemetry server
//const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialised `ChainSpec`. This is a specialisation of the general Substrate
/// ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// Helper function to generate a crypto pair from seed
fn get_from_secret<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(seed, None)
		.unwrap_or_else(|_| panic!("Invalid string '{}'", seed))
		.public()
}

/// Helper function to generate an account ID from seed
fn get_account_id_from_secret<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_secret::<TPublic>(seed)).into_account()
}

/// Helper function to generate an authority key for Aura
fn get_authority_keys_from_secret(seed: &str) -> (AccountId, AuraId, GrandpaId) {
	(
		get_account_id_from_secret::<ed25519::Public>(seed),
		get_from_secret::<AuraId>(seed),
		get_from_secret::<GrandpaId>(seed),
	)
}

fn devnet_chain_spec() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm binary not available".to_string())?;

	let mut properties = Properties::new();
	properties.insert("tokenDecimals".into(), 15_i16.into());

	properties.insert("tokenSymbol".into(), "DILT".into());
	Ok(ChainSpec::from_genesis(
		"Development",
		"development",
		ChainType::Development,
		move || {
			devnet_genesis(
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

const YORLIN_ALICE_ACC: [u8; 32] = hex!["e82655d021c27086c4c8a47c29a9094c50c3d09d5ddbb71c01781b4cf6c2dc3f"];
const YORLIN_ALICE_SESSION_SR: [u8; 32] = hex!["ecb26520504cecf51936e8d9df07d1355726bf186f9cd38d35277f918fe3230c"];
const YORLIN_ALICE_SESSION_ED: [u8; 32] = hex!["d600f710ab168414cb29faef92bd570f01c375cb359ec27485b176246ac597a5"];
const YORLIN_BOB_ACC: [u8; 32] = hex!["38621f2de0250bd855fef9ab09fd8b06e6ed67c574ea4ae2b46557a809fab56d"];
const YORLIN_BOB_SESSION_SR: [u8; 32] = hex!["1284c324ac272432b83779886ad66ff74dc6147f4a4a67124218e0b88c27ea7d"];
const YORLIN_BOB_SESSION_ED: [u8; 32] = hex!["5d1040178af44ca8bc598d48c6b0c49e8b5b916315d3d91f953df7623c9c78ae"];
const YORLIN_FAUCET_ACC: [u8; 32] = hex!["a874b37f88e76eefbeb62d4424876004c81f7ae30e2d7c2bb380001a1961fc38"];

fn yorlin_chain_spec() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Yorlin wasm binary not available".to_string())?;

	let mut properties = Properties::new();
	properties.insert("tokenDecimals".into(), 15_i16.into());

	properties.insert("tokenSymbol".into(), "YILT".into());
	Ok(ChainSpec::from_genesis(
		"Yorlin",
		"Yorlin",
		ChainType::Development,
		move || {
			devnet_genesis(
				wasm_binary,
				vec![
					(
						YORLIN_ALICE_ACC.into(),
						YORLIN_ALICE_SESSION_SR.unchecked_into(),
						YORLIN_ALICE_SESSION_ED.unchecked_into(),
					),
					(
						YORLIN_BOB_ACC.into(),
						YORLIN_BOB_SESSION_SR.unchecked_into(),
						YORLIN_BOB_SESSION_ED.unchecked_into(),
					),
				],
				YORLIN_ALICE_ACC.into(),
				vec![YORLIN_ALICE_ACC.into(), YORLIN_BOB_ACC.into(), YORLIN_FAUCET_ACC.into()],
			)
		},
		vec![],
		None,
		Some("yOrLiN"),
		None,
		Some(properties),
		None,
	))
}

fn testnet_chain_spec() -> Result<ChainSpec, String> {
	ChainSpec::from_json_bytes(&include_bytes!("../res/testnet.json")[..])
}

fn devnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AccountId, AuraId, GrandpaId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
) -> GenesisConfig {
	GenesisConfig {
		system: SystemConfig {
			code: wasm_binary.to_vec(),
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

pub fn load_spec(id: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
	Ok(Box::new(match id {
		"kilt-testnet" => testnet_chain_spec()?,
		"dev" => devnet_chain_spec()?,
		"yorlin" => yorlin_chain_spec()?,
		_ => return Err(format!("Unknown spec: {}", id)),
	}))
}
