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

use emulated_integration_tests_common::{
	accounts, impl_accounts_helpers_for_parachain, impl_assert_events_helpers_for_parachain,
	impl_xcm_helpers_for_parachain,
};
use frame_support::traits::OnInitialize;
use rococo_emulated_chain::genesis::ED;
use runtime_common::AuthorityId;
use sp_core::sr25519;
use sp_runtime::{BuildStorage, Storage};
use xcm_emulator::{decl_test_parachains, Parachain};

use crate::utils::{get_account_id_from_seed, get_from_seed};

const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;
pub mod spiritnet {
	use super::*;

	use spiritnet_runtime::{
		BalancesConfig, ParachainInfoConfig, PolkadotXcmConfig, RuntimeGenesisConfig, SessionConfig, SessionKeys,
	};

	pub const PARA_ID: u32 = 2_000;

	pub fn genesis() -> Storage {
		RuntimeGenesisConfig {
			parachain_info: ParachainInfoConfig {
				parachain_id: PARA_ID.into(),
				..Default::default()
			},
			polkadot_xcm: PolkadotXcmConfig {
				safe_xcm_version: Some(SAFE_XCM_VERSION),
				..Default::default()
			},
			session: SessionConfig {
				keys: vec![(
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_from_seed::<AuthorityId>("Alice"),
				)]
				.iter()
				.map(|(acc, key)| (acc.clone(), acc.clone(), SessionKeys { aura: key.clone() }))
				.collect::<Vec<_>>(),
			},
			balances: BalancesConfig {
				balances: accounts::init_balances()
					.iter()
					.cloned()
					.map(|k| (k, ED * 4096))
					.collect(),
			},
			..Default::default()
		}
		.build_storage()
		.unwrap()
	}
}

pub mod peregrine {
	use super::*;

	use peregrine_runtime::{
		BalancesConfig, ParachainInfoConfig, PolkadotXcmConfig, RuntimeGenesisConfig, SessionConfig, SessionKeys,
	};

	pub const PARA_ID: u32 = 2_000;

	pub fn genesis() -> Storage {
		RuntimeGenesisConfig {
			parachain_info: ParachainInfoConfig {
				parachain_id: PARA_ID.into(),
				..Default::default()
			},
			polkadot_xcm: PolkadotXcmConfig {
				safe_xcm_version: Some(SAFE_XCM_VERSION),
				..Default::default()
			},
			session: SessionConfig {
				keys: vec![(
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_from_seed::<AuthorityId>("Alice"),
				)]
				.iter()
				.map(|(acc, key)| (acc.clone(), acc.clone(), SessionKeys { aura: key.clone() }))
				.collect::<Vec<_>>(),
			},
			balances: BalancesConfig {
				balances: accounts::init_balances()
					.iter()
					.cloned()
					.map(|k| (k, ED * 4096))
					.collect(),
			},
			..Default::default()
		}
		.build_storage()
		.unwrap()
	}
}

decl_test_parachains! {
	pub struct Spiritnet {
		genesis = spiritnet::genesis(),
		on_init = {
			spiritnet_runtime::AuraExt::on_initialize(1);
		},
		runtime = spiritnet_runtime,
		core = {
			XcmpMessageHandler: spiritnet_runtime::XcmpQueue,
			LocationToAccountId: spiritnet_runtime::xcm_config::LocationToAccountIdConverter,
			ParachainInfo: spiritnet_runtime::ParachainInfo,
			MessageOrigin: cumulus_primitives_core::AggregateMessageOrigin,
		},
		pallets = {
			Balances: spiritnet_runtime::Balances,
			PolkadotXcm: spiritnet_runtime::PolkadotXcm,
			Did: spiritnet_runtime::Did,
			Ctype: spiritnet_runtime::Ctype,
			Attestation: spiritnet_runtime::Attestation,
			Web3Names: spiritnet_runtime::Web3Names,
			DidLookup: spiritnet_runtime::DidLookup,
			PublicCredentials: spiritnet_runtime::PublicCredentials,
		}
	},
	pub struct Peregrine {
		genesis = peregrine::genesis(),
		on_init = {
			peregrine_runtime::AuraExt::on_initialize(1);
		},
		runtime = peregrine_runtime,
		core = {
			XcmpMessageHandler: peregrine_runtime::XcmpQueue,
			LocationToAccountId: peregrine_runtime::xcm_config::LocationToAccountIdConverter,
			ParachainInfo: peregrine_runtime::ParachainInfo,
			MessageOrigin: cumulus_primitives_core::AggregateMessageOrigin,
		},
		pallets = {
			Balances: peregrine_runtime::Balances,
			PolkadotXcm: peregrine_runtime::PolkadotXcm,
			Did: peregrine_runtime::Did,
			Ctype: peregrine_runtime::Ctype,
			Attestation: peregrine_runtime::Attestation,
			Web3Names: peregrine_runtime::Web3Names,
			DidLookup: peregrine_runtime::DidLookup,
			PublicCredentials: peregrine_runtime::PublicCredentials,
		}
	},
}

impl_accounts_helpers_for_parachain!(Spiritnet);
impl_assert_events_helpers_for_parachain!(Spiritnet);
impl_xcm_helpers_for_parachain!(Spiritnet);

impl_accounts_helpers_for_parachain!(Peregrine);
impl_assert_events_helpers_for_parachain!(Peregrine);
impl_xcm_helpers_for_parachain!(Peregrine);
