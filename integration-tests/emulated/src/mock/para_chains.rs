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

use emulated_integration_tests_common::accounts;
use frame_support::traits::OnInitialize;
use runtime_common::{constants::KILT, AuthorityId};
use sp_core::sr25519;
use sp_runtime::{BuildStorage, Storage};
use std::iter::once;
use xcm_emulator::decl_test_parachains;

use crate::utils::{get_account_id_from_seed, get_from_seed};

const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;
pub mod spiritnet {
	use super::*;

	use spiritnet_runtime::{
		BalancesConfig, ParachainInfoConfig, PolkadotXcmConfig, RuntimeGenesisConfig, SessionConfig, SessionKeys,
	};

	pub const PARA_ID: u32 = 2_001;

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
				keys: once(&(
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_from_seed::<AuthorityId>("Alice"),
				))
				.map(|(acc, key)| (acc.clone(), acc.clone(), SessionKeys { aura: key.clone() }))
				.collect::<Vec<_>>(),
			},
			balances: BalancesConfig {
				balances: accounts::init_balances()
					.iter()
					.cloned()
					.map(|k| (k, KILT * 1_000))
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
				keys: once(&(
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_from_seed::<AuthorityId>("Alice"),
				))
				.map(|(acc, key)| (acc.clone(), acc.clone(), SessionKeys { aura: key.clone() }))
				.collect::<Vec<_>>(),
			},
			balances: BalancesConfig {
				balances: accounts::init_balances()
					.iter()
					.cloned()
					.map(|k| (k, KILT * 1_000))
					.collect(),
			},
			..Default::default()
		}
		.build_storage()
		.unwrap()
	}
}

decl_test_parachains! {
	pub struct SpiritnetParachain {
		genesis = spiritnet::genesis(),
		on_init = {
			spiritnet_runtime::AuraExt::on_initialize(1);
		},
		runtime = spiritnet_runtime,
		core = {
			XcmpMessageHandler: spiritnet_runtime::XcmpQueue,
			LocationToAccountId: spiritnet_runtime::xcm::LocationToAccountIdConverter,
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
	pub struct PeregrineParachain {
		genesis = peregrine::genesis(),
		on_init = {
			peregrine_runtime::AuraExt::on_initialize(1);
		},
		runtime = peregrine_runtime,
		core = {
			XcmpMessageHandler: peregrine_runtime::XcmpQueue,
			LocationToAccountId: peregrine_runtime::xcm::LocationToAccountIdConverter,
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
