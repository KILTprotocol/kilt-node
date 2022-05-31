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

use mashnet_node_runtime::{
	BalancesConfig, GenesisConfig, IndicesConfig, SessionConfig, SudoConfig, SystemConfig, VestingConfig, WASM_BINARY,
};
use runtime_common::{AccountId, AccountPublic, Balance, BlockNumber};

use hex_literal::hex;

use sc_service::{self, ChainType, Properties};
use sc_telemetry::TelemetryEndpoints;
use sp_consensus_aura::ed25519::AuthorityId as AuraId;
use sp_core::{crypto::UncheckedInto, ed25519, sr25519, Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::IdentifyAccount;

// Note this is the URL for the telemetry server
//const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialised `ChainSpec`. This is a specialisation of the general Substrate
/// ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// The chain specification option. This is expected to come in from the CLI and
/// is little more than one of a number of alternatives which can easily be
/// converted from a string (`--chain=...`) into a `ChainSpec`.
#[derive(Clone, Debug)]
pub enum Alternative {
	/// Whatever the current runtime is, with just Alice as an auth.
	Development,
	/// Whatever the current runtime is, with simple Alice/Bob auths.
	KiltTestnet,
	/// Sporran Testnet
	SporranTestnet,
}

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

const TELEMETRY_URL: &str = "wss://telemetry-backend.kilt.io:8080/submit";

const SPORRAN_AUTHORITY_ACC: [u8; 32] = hex!("2c94fbcfe0a7db40579e12bc74d0f7215fe91ba51b3eade92799788ca549f373");
const SPORRAN_AUTHORITY_SESSION: [u8; 32] = hex!("3bbaa842650064362767a1d9dd8899f531c80dc42eafb9599f4df0965e4a5299");
const SPORRAN_FAUCET: [u8; 32] = hex!("2c9e9c40e15a2767e2d04dc1f05d824dd76d1d37bada3d7bb1d40eca29f3a4ff");

impl Alternative {
	/// Get an actual chain config from one of the alternatives.
	pub(crate) fn load(self) -> Result<ChainSpec, String> {
		let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm binary not available".to_string())?;

		let mut properties = Properties::new();
		properties.insert("tokenDecimals".into(), 15_i16.into());

		Ok(match self {
			Alternative::Development => {
				properties.insert("tokenSymbol".into(), "KILT".into());
				ChainSpec::from_genesis(
					"Development",
					"development",
					ChainType::Development,
					move || {
						testnet_genesis(
							wasm_binary,
							vec![get_authority_keys_from_secret("//Alice")],
							get_account_id_from_secret::<ed25519::Public>("//Alice"),
							vec![
								// Dev Faucet account
								get_account_id_from_secret::<ed25519::Public>("receive clutch item involve chaos clutch furnace arrest claw isolate okay together"),
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
				)
			}
			Alternative::SporranTestnet => {
				properties.insert("tokenSymbol".into(), "SILT".into());
				ChainSpec::from_genesis(
					"Sporran",
					"sporran",
					ChainType::Development,
					move || {
						testnet_genesis(
							wasm_binary,
							vec![(
								SPORRAN_AUTHORITY_ACC.into(),
								SPORRAN_AUTHORITY_SESSION.unchecked_into(),
								SPORRAN_AUTHORITY_SESSION.unchecked_into(),
							)],
							SPORRAN_AUTHORITY_ACC.into(),
							vec![SPORRAN_FAUCET.into(), SPORRAN_AUTHORITY_ACC.into()],
						)
					},
					vec![
						"/dns4/bootnode.kilt.io/tcp/30340/p2p/12D3KooWGXaTjB6KmPHxyCx2dQLpS7p9vnXAYmprQMsVNnxYAWYa"
							.parse()
							.expect("bootnode address is formatted correctly; qed"),
					],
					Some(
						TelemetryEndpoints::new(vec![(TELEMETRY_URL.to_string(), 0)])
							.expect("SILT telemetry url is valid; qed"),
					),
					Some("SILT"),
					None,
					Some(properties),
					None,
				)
			}
			Alternative::KiltTestnet => ChainSpec::from_json_bytes(&include_bytes!("../res/testnet.json")[..])?,
		})
	}

	pub(crate) fn from(s: &str) -> Option<Self> {
		match s {
			"dev" => Some(Alternative::Development),
			"kilt-testnet" => Some(Alternative::KiltTestnet),
			"sporran-new" => Some(Alternative::SporranTestnet),
			_ => None,
		}
	}
}

fn testnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AccountId, AuraId, GrandpaId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
) -> GenesisConfig {
	type VestingPeriod = BlockNumber;
	type LockingPeriod = BlockNumber;

	// vesting and locks as initially designed
	let airdrop_accounts_json = &include_bytes!("../res/genesis-testing/genesis_accounts.json")[..];
	let airdrop_accounts: Vec<(AccountId, Balance, VestingPeriod, LockingPeriod)> =
		serde_json::from_slice(airdrop_accounts_json).expect("Could not read from genesis_accounts.json");

	GenesisConfig {
		system: SystemConfig {
			code: wasm_binary.to_vec(),
		},
		indices: IndicesConfig { indices: vec![] },
		transaction_payment: Default::default(),
		balances: BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|a| (a, 1u128 << 90))
				.chain(airdrop_accounts.iter().cloned().map(|(who, total, _, _)| (who, total)))
				.collect(),
		},
		session: SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| {
					(
						x.0.clone(),
						x.0.clone(),
						mashnet_node_runtime::opaque::SessionKeys {
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
		vesting: VestingConfig { vesting: vec![] },
	}
}

pub fn load_spec(id: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
	Ok(match Alternative::from(id) {
		Some(spec) => Box::new(spec.load()?),
		None => Box::new(ChainSpec::from_json_file(std::path::PathBuf::from(id))?),
	})
}
