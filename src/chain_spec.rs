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
    AccountId, BalancesConfig, ConsensusConfig, GenesisConfig, IndicesConfig,
    SudoConfig, TimestampConfig,
};
use substrate_service;

use ed25519::Public as AuthorityId;
use primitives::{ed25519, ed25519 as x25519, Pair};

// Note this is the URL for the telemetry server
//const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialised `ChainSpec`. This is a specialisation of the general Substrate ChainSpec type.
pub type ChainSpec = substrate_service::ChainSpec<GenesisConfig>;

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

fn authority_key(s: &str) -> AuthorityId {
    ed25519::Pair::from_string(&format!("//{}", s), None)
        .expect("static values are valid; qed")
        .public()
}

fn account_key(s: &str) -> AccountId {
    x25519::Pair::from_string(&format!("//{}", s), None)
        .expect("static values are valid; qed")
        .public()
}

impl Alternative {
    /// Get an actual chain config from one of the alternatives.
    pub(crate) fn load(self) -> Result<ChainSpec, String> {
        Ok(match self {
            Alternative::Development => ChainSpec::from_genesis(
                "Development",
                "development",
                || testnet_genesis(vec![
                    authority_key("Alice")
                ], vec![
                    // Dev Faucet account
                    // Seed phrase: "receive clutch item involve chaos clutch furnace arrest claw isolate okay together"
                    x25519::Public::from_raw(hex!("edd46b726279b53ea67dee9eeca1d8193de4d78e7e729a6d11a8dea59905f95e")),
                    account_key("Alice"),
                    account_key("Bob")
                ],
                    account_key("Alice"),
                ),
                vec![],
                None,
                None,
                None,
                None,
            ),
            Alternative::KiltTestnet => ChainSpec::from_genesis(
                "KILT Testnet",
                "kilt_testnet",
                || testnet_genesis(vec![
                    x25519::Public::from_raw(hex!("58d3bb9e9dd245f3dec8d8fab7b97578c00a10cf3ca9d224caaa46456f91c46c")),
                    x25519::Public::from_raw(hex!("d660b4470a954ecc99496d4e4b012ee9acac3979e403967ef09de20da9bdeb28")),
                    x25519::Public::from_raw(hex!("2ecb6a4ce4d9bc0faab70441f20603fcd443d6d866e97c9e238a2fb3e982ae2f")),
                ], vec![
                    // Testnet Faucet accounts
                    x25519::Public::from_raw(hex!("3ba6e1019a22234a9349eb1d76e02f74fecff31da60a0c8fc1e74a4a3a32b925")),
                    x25519::Public::from_raw(hex!("b7f202703a34a034571696f51e95047417956337c596c889bd4d3c1e162310b6")),
                    x25519::Public::from_raw(hex!("5895c421d0fde063e0758610896453aec306f09081cb2caed9649865728e670a"))
                ],
                    x25519::Public::from_raw(hex!("58d3bb9e9dd245f3dec8d8fab7b97578c00a10cf3ca9d224caaa46456f91c46c")),
                ),
                vec![],
                None,
                None,
                None,
                None,
            ),
            Alternative::KiltDevnet => ChainSpec::from_genesis(
                "KILT Devnet",
                "kilt_devnet",
                || testnet_genesis(
                    // Initial Authorities
                    vec![
                        x25519::Public::from_raw(hex!("d44da634611d9c26837e3b5114a7d460a4cb7d688119739000632ed2d3794ae9")),
                        x25519::Public::from_raw(hex!("06815321f16a5ae0fe246ee19285f8d8858fe60d5c025e060922153fcf8e54f9")),
                        x25519::Public::from_raw(hex!("6d2d775fdc628134e3613a766459ccc57a29fd380cd410c91c6c79bc9c03b344")),
                    ], 
                    // Endowed Accounts
                    vec![
                        x25519::Public::from_raw(hex!("d44da634611d9c26837e3b5114a7d460a4cb7d688119739000632ed2d3794ae9")),
                    ],
                    // Root
                        x25519::Public::from_raw(hex!("d44da634611d9c26837e3b5114a7d460a4cb7d688119739000632ed2d3794ae9")),
                ),
                vec![],
                None,
                None,
                None,
                None,
            ),
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

fn testnet_genesis(initial_authorities: Vec<AuthorityId>, endowed_accounts: Vec<AccountId>, root_key: AccountId) -> GenesisConfig {
	GenesisConfig {
		consensus: Some(ConsensusConfig {
			code: include_bytes!("../runtime/wasm/target/wasm32-unknown-unknown/release/mashnet_node_runtime_wasm.compact.wasm").to_vec(),
			authorities: initial_authorities.clone(),
		}),
		system: None,
		timestamp: Some(TimestampConfig {
			minimum_period: 5, // 10 second block time.
		}),
		indices: Some(IndicesConfig {
			ids: endowed_accounts.clone(),
		}),
		balances: Some(BalancesConfig {
			transaction_base_fee: 1000000,
			transaction_byte_fee: 0,
			existential_deposit: 1000000,
			transfer_fee: 0,
			creation_fee: 0,
			balances: endowed_accounts.iter().cloned().map(|k|(k, 1 << 60)).collect(),
			vesting: vec![],
		}),
		sudo: Some(SudoConfig {
			key: root_key,
		}),
	}
}
