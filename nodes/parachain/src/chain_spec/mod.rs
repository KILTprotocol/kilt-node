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

use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_cli::RuntimeVersion;
use serde::{Deserialize, Serialize};

pub(crate) use utils::load_spec;

pub(crate) mod peregrine;
pub(crate) mod spiritnet;
pub(crate) mod utils;

const KILT_PARA_ID: u32 = 2_086;

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

pub(crate) enum PeregrineRuntime {
	Dev,
	Peregrine,
	PeregrineStg,
	Rilt,
	New,
	Other(String),
}

impl std::fmt::Display for PeregrineRuntime {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Dev => write!(f, "dev"),
			Self::Peregrine => write!(f, "peregrine"),
			Self::PeregrineStg => write!(f, "peregrine-stg"),
			Self::Rilt => write!(f, "rilt"),
			Self::New => write!(f, "new"),
			Self::Other(path) => write!(f, "other -> {path}"),
		}
	}
}

pub(crate) enum SpiritnetRuntime {
	Dev,
	Spiritnet,
	New,
	Other(String),
}

impl std::fmt::Display for SpiritnetRuntime {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Dev => write!(f, "dev"),
			Self::Spiritnet => write!(f, "spiritnet"),
			Self::New => write!(f, "new"),
			Self::Other(path) => write!(f, "other -> {path}"),
		}
	}
}

pub(crate) enum ParachainRuntime {
	Peregrine(PeregrineRuntime),
	Spiritnet(SpiritnetRuntime),
}

impl ParachainRuntime {
	pub(crate) fn native_version(&self) -> &'static RuntimeVersion {
		match self {
			Self::Peregrine(_) => &peregrine_runtime::VERSION,
			Self::Spiritnet(_) => &spiritnet_runtime::VERSION,
		}
	}
}

impl std::fmt::Display for ParachainRuntime {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Peregrine(p) => write!(f, "peregrine ({p})"),
			Self::Spiritnet(s) => write!(f, "spiritnet ({s})"),
		}
	}
}

impl FromStr for ParachainRuntime {
	type Err = String;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			// Peregrine development
			"dev" => Ok(Self::Peregrine(PeregrineRuntime::Dev)),
			// New blank Peregrine chainspec
			"peregrine-new" => Ok(Self::Peregrine(PeregrineRuntime::New)),
			// Peregrine chainspec
			"peregrine" => Ok(Self::Peregrine(PeregrineRuntime::Peregrine)),
			// Peregrine staging chainspec
			"peregrine-stg" => Ok(Self::Peregrine(PeregrineRuntime::PeregrineStg)),
			// RILT chainspec
			"rilt" => Ok(Self::Peregrine(PeregrineRuntime::Rilt)),
			// Any other Peregrine-based chainspec
			s if s.contains("peregrine") => Ok(Self::Peregrine(PeregrineRuntime::Other(s.to_string()))),

			// Spiritnet development
			"spiritnet-dev" => Ok(Self::Spiritnet(SpiritnetRuntime::Dev)),
			// New blank Spiritnet chainspec
			"spiritnet-new" => Ok(Self::Spiritnet(SpiritnetRuntime::New)),
			// Spiritnet chainspec
			"spiritnet" => Ok(Self::Spiritnet(SpiritnetRuntime::Spiritnet)),
			// Any other Spiritnet-based chainspec
			s if s.contains("spiritnet") => Ok(Self::Spiritnet(SpiritnetRuntime::Other(s.to_string()))),

			_ => Err(format!("Unknown chainspec id provided: {s}")),
		}
	}
}
