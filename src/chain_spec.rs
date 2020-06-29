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
use sc_service::{self, ChainType};
use sp_consensus_aura::ed25519::AuthorityId as AuraId;
use sp_core::{crypto::UncheckedInto, ed25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};

use hex;

// Note this is the URL for the telemetry server
//const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

type AccountPublic = <Signature as Verify>::Signer;

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
fn get_authority_keys_from_secret(s: &str) -> (AuraId, GrandpaId) {
	(
		get_from_secret::<AuraId>(s),
		get_from_secret::<GrandpaId>(s),
	)
}

/// Build a pair of public keys from a given hex string. This method will panic if the hex string is malformed.
///
/// public_key – the public key formatted as a hex string
fn as_authority_key(sr_public_key: [u8; 32], ed_public_key: [u8; 32]) -> (AuraId, GrandpaId) {
	(
		sr_public_key.unchecked_into(),
		ed_public_key.unchecked_into(),
	)
}

const AUTH_A_SR: [u8; 32] = hex!("06813719bd07babb9683dbbc899cdfa1322fcac995090976f38bd82cb6945a37");
const AUTH_A_ED: [u8; 32] = hex!("a4cc7a000c48e9f3e37113d1ec291fe7b7b52c63a445fab2f37c96d05d20030d");

const AUTH_B_SR: [u8; 32] = hex!("ec5a43ac7191357c152724af94d9e594c24b15cfee0e274d212872604c86bc3b");
const AUTH_B_ED: [u8; 32] = hex!("2277c5f1bc8c60eb7bdad1d41e7157eec56ded7feefba7b01f4f6d97a5b1be9d");

impl Alternative {
	/// Get an actual chain config from one of the alternatives.
	pub(crate) fn load(self) -> Result<ChainSpec, String> {
		Ok(match self {
			Alternative::Development => {
				ChainSpec::from_genesis(
					"Development",
					"development",
					ChainType::Development,
					|| {
						testnet_genesis(
							vec![get_authority_keys_from_secret("//Alice")],
							get_account_id_from_secret::<ed25519::Public>("//Alice"),
							vec![
					// Dev Faucet account
					get_account_id_from_secret::<ed25519::Public>("receive clutch item involve chaos clutch furnace arrest claw isolate okay together"),
					get_account_id_from_secret::<ed25519::Public>("//Bob"),
					get_account_id_from_secret::<ed25519::Public>("//Alice"),
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
					ChainType::Live,
					|| {
						testnet_genesis(
							vec![
								as_authority_key(AUTH_A_SR, AUTH_A_ED),
								as_authority_key(AUTH_B_SR, AUTH_B_ED),
						],

							AUTH_A_SR.into(),
							vec![
					// Testnet Faucet accounts
					AUTH_B_SR.into(),
					AUTH_B_ED.into(),
					AUTH_A_SR.into(),
					AUTH_A_ED.into(),
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
					ChainType::Live,
					|| {
						testnet_genesis(
							// Initial Authorities
							vec![
								as_authority_key(AUTH_A_SR, AUTH_A_ED),
								as_authority_key(AUTH_B_SR, AUTH_B_ED),
					],
							AUTH_A_ED.into(),
							vec![AUTH_A_ED.into(),
							AUTH_A_ED.into()],
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
				.map(|k| (k, 1u128 << 60))
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
