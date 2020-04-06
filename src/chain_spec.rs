// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019  BOTLabs GmbH

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

use mashnet_node_runtime::{
	AccountId, AuraConfig, BalancesConfig, GenesisConfig, GrandpaConfig, Signature, SudoConfig,
	SystemConfig, WASM_BINARY,
};

use grandpa_primitives::AuthorityId as GrandpaId;
use sc_service;
use sp_consensus_aura::ed25519::AuthorityId as AuraId;
use sp_core::{ed25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};

// Note this is the URL for the telemetry server
//const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialised `ChainSpec`. This is a specialisation of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// The chain specification option. This is expected to come in from the CLI and
/// is little more than one of a number of alternatives which can easily be converted
/// from a string (`--chain=...`) into a `ChainSpec`.
#[derive(Clone, Debug)]
pub enum Alternative {
	/// Whatever the current runtime is, with just Alice as an auth.
	Development,
	/// Whatever the current runtime is, with simple Alice/Bob auths.
	KiltTestnet,
	KiltDevnet,
}

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(seed, None)
		.expect(&format!("Invalid seed '{}'", seed))
		.public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Helper function to generate an authority key for Aura
pub fn get_authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
	(get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
}

impl Alternative {
	/// Get an actual chain config from one of the alternatives.
	pub(crate) fn load(self) -> Result<ChainSpec, String> {
		Ok(match self {
			Alternative::Development => {
				ChainSpec::from_genesis(
					"Development",
					"development",
					|| {
						testnet_genesis(
							vec![get_authority_keys_from_seed("//Alice")],
							get_account_id_from_seed::<ed25519::Public>("//Alice"),
							vec![
					// Dev Faucet account
					get_account_id_from_seed::<ed25519::Public>("receive clutch item involve chaos clutch furnace arrest claw isolate okay together"),
					get_account_id_from_seed::<ed25519::Public>("//Bob"),
					get_account_id_from_seed::<ed25519::Public>("//Alice"),
				],
							true,
						)
					},
					vec![],
					None,
					None,
					None,
					None,
				)
			}
			Alternative::KiltTestnet => {
				ChainSpec::from_genesis(
					"KILT Testnet",
					"kilt_testnet",
					|| {
						testnet_genesis(
							vec![
							get_authority_keys_from_seed("0x58d3bb9e9dd245f3dec8d8fab7b97578c00a10cf3ca9d224caaa46456f91c46c"),
							get_authority_keys_from_seed("0xd660b4470a954ecc99496d4e4b012ee9acac3979e403967ef09de20da9bdeb28"),
							get_authority_keys_from_seed("0x2ecb6a4ce4d9bc0faab70441f20603fcd443d6d866e97c9e238a2fb3e982ae2f"),
						],
							get_account_id_from_seed::<ed25519::Public>(
								"0x58d3bb9e9dd245f3dec8d8fab7b97578c00a10cf3ca9d224caaa46456f91c46c",
							),
							vec![
					// Testnet Faucet accounts
					get_account_id_from_seed::<ed25519::Public>("0x3ba6e1019a22234a9349eb1d76e02f74fecff31da60a0c8fc1e74a4a3a32b925"),
					get_account_id_from_seed::<ed25519::Public>("0xb7f202703a34a034571696f51e95047417956337c596c889bd4d3c1e162310b6"),
					get_account_id_from_seed::<ed25519::Public>("0x5895c421d0fde063e0758610896453aec306f09081cb2caed9649865728e670a")
				],
							true,
						)
					},
					vec![],
					None,
					None,
					None,
					None,
				)
			}
			Alternative::KiltDevnet => {
				ChainSpec::from_genesis(
					"KILT Devnet",
					"kilt_devnet",
					|| {
						testnet_genesis(
							// Initial Authorities
							vec![
						get_authority_keys_from_seed("0xd44da634611d9c26837e3b5114a7d460a4cb7d688119739000632ed2d3794ae9"),
						get_authority_keys_from_seed("0x06815321f16a5ae0fe246ee19285f8d8858fe60d5c025e060922153fcf8e54f9"),
						get_authority_keys_from_seed("0x6d2d775fdc628134e3613a766459ccc57a29fd380cd410c91c6c79bc9c03b344"),
					],
							get_account_id_from_seed::<ed25519::Public>(
								"0xd44da634611d9c26837e3b5114a7d460a4cb7d688119739000632ed2d3794ae9",
							),
							vec![get_account_id_from_seed::<ed25519::Public>(
								"0xd44da634611d9c26837e3b5114a7d460a4cb7d688119739000632ed2d3794ae9",
							)],
							true,
						)
					},
					vec![],
					None,
					None,
					None,
					None,
				)
			}
		})
	}

	pub(crate) fn from(s: &str) -> Option<Self> {
		match s {
			"dev" => Some(Alternative::Development),
			"kilt-testnet" => Some(Alternative::KiltTestnet),
			"kilt-devnet" => Some(Alternative::KiltDevnet),
			_ => None,
		}
	}
}

fn testnet_genesis(
	initial_authorities: Vec<(AuraId, GrandpaId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	_enable_println: bool,
) -> GenesisConfig {
	GenesisConfig {
		system: Some(SystemConfig {
			code: WASM_BINARY.to_vec(),
			changes_trie_config: Default::default(),
		}),
		balances: Some(BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, 1 << 60))
				.collect(),
		}),
		aura: Some(AuraConfig {
			authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
		}),
		grandpa: Some(GrandpaConfig {
			authorities: initial_authorities
				.iter()
				.map(|x| (x.1.clone(), 1))
				.collect(),
		}),
		sudo: Some(SudoConfig { key: root_key }),
	}
}

pub fn load_spec(id: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
	Ok(match Alternative::from(id) {
		Some(spec) => Box::new(spec.load()?),
		None => Box::new(ChainSpec::from_json_file(std::path::PathBuf::from(id))?),
	})
}
