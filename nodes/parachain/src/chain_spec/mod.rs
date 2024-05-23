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

use std::str::FromStr;

use runtime_common::{AccountId, AccountPublic};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_cli::RuntimeVersion;
use sc_service::Properties;
use serde::{Deserialize, Serialize};
use sp_core::{Pair, Public};
use sp_runtime::traits::IdentifyAccount;

pub(crate) mod peregrine;
pub(crate) mod spiritnet;

const KILT_PARA_ID: u32 = 2_086;

/// Helper function to generate an account ID from seed
fn get_account_id_from_secret<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_secret::<TPublic>(seed)).into_account()
}

/// Helper function to generate a crypto pair from seed
fn get_from_secret<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// The extensions for the `ChainSpec`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
	/// The relay chain of the Parachain.
	pub relay_chain: String,
	/// The id of the Parachain.
	pub para_id: u32,
}

impl Extensions {
	/// Try to get the extension from the given `ChainSpec`.
	pub(crate) fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
		sc_chain_spec::get_extension(chain_spec.extensions())
	}
}

fn get_properties(symbol: &str, decimals: u32, ss58format: u32) -> Properties {
	Properties::from_iter(
		[
			("tokenSymbol".into(), symbol.into()),
			("tokenDecimals".into(), decimals.into()),
			("ss58Format".into(), ss58format.into()),
		]
		.into_iter(),
	)
}

pub(crate) enum ChainRuntime {
	Peregrine,
	Spiritnet,
}

impl ChainRuntime {
	pub(crate) fn native_version(&self) -> &'static RuntimeVersion {
		match self {
			Self::Peregrine => &peregrine_runtime::VERSION,
			Self::Spiritnet => &spiritnet_runtime::VERSION,
		}
	}
}

impl std::fmt::Display for ChainRuntime {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Peregrine => write!(f, "peregrine"),
			Self::Spiritnet => write!(f, "spiritnet"),
		}
	}
}

impl FromStr for ChainRuntime {
	type Err = &'static str;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if s.contains("peregrine") {
			Ok(ChainRuntime::Peregrine)
		} else if s.contains("spiritnet") {
			Ok(ChainRuntime::Spiritnet)
		} else {
			Err("Unknown chain_spec id provided")
		}
	}
}
