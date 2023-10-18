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

use cumulus_primitives_core::ParaId;
use dip_consumer_runtime_template::{
	AccountId, AuraId, BalancesConfig, CollatorSelectionConfig, ParachainInfoConfig, RuntimeGenesisConfig,
	SessionConfig, SessionKeys, Signature, SudoConfig, SystemConfig, EXISTENTIAL_DEPOSIT, SS58_PREFIX, WASM_BINARY,
};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup, Properties};
use sc_service::{ChainType, GenericChainSpec};
use serde::{Deserialize, Serialize};
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};

const PARA_ID: u32 = 2_001;

pub type ChainSpec = GenericChainSpec<RuntimeGenesisConfig, Extensions>;
type AccountPublic = <Signature as Verify>::Signer;

pub(crate) fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
	pub relay_chain: String,
	pub para_id: u32,
}

impl Extensions {
	pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
		sc_chain_spec::get_extension(chain_spec.extensions())
	}
}

pub fn get_collator_keys_from_seed(seed: &str) -> AuraId {
	get_from_seed::<AuraId>(seed)
}

pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

pub fn template_session_keys(keys: AuraId) -> SessionKeys {
	SessionKeys { aura: keys }
}

fn testnet_genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	endowed_accounts: Vec<AccountId>,
	id: ParaId,
) -> RuntimeGenesisConfig {
	RuntimeGenesisConfig {
		system: SystemConfig {
			code: WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			..Default::default()
		},
		parachain_system: Default::default(),
		parachain_info: ParachainInfoConfig {
			parachain_id: id,
			..Default::default()
		},
		sudo: SudoConfig {
			key: Some(endowed_accounts.first().unwrap().clone()),
		},
		balances: BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|k| (k, 1 << 60)).collect(),
		},
		transaction_payment: Default::default(),
		collator_selection: CollatorSelectionConfig {
			invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond: EXISTENTIAL_DEPOSIT * 16,
			..Default::default()
		},
		session: SessionConfig {
			keys: invulnerables
				.into_iter()
				.map(|(acc, aura)| (acc.clone(), acc, template_session_keys(aura)))
				.collect(),
		},
		aura: Default::default(),
		aura_ext: Default::default(),
	}
}

pub fn development_config() -> ChainSpec {
	let mut properties = Properties::new();
	properties.insert("tokenSymbol".into(), "REILT".into());
	properties.insert("tokenDecimals".into(), 12.into());
	properties.insert("ss58Format".into(), SS58_PREFIX.into());

	ChainSpec::from_genesis(
		"DIP consumer dev",
		"dip-consumer-dev",
		ChainType::Development,
		move || {
			testnet_genesis(
				vec![(
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_collator_keys_from_seed("Alice"),
				)],
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
					get_account_id_from_seed::<sr25519::Public>("Charlie"),
					get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
				],
				PARA_ID.into(),
			)
		},
		Vec::new(),
		None,
		"dip-consumer-dev".into(),
		None,
		None,
		Extensions {
			relay_chain: "rococo-local".into(),
			para_id: PARA_ID,
		},
	)
}
