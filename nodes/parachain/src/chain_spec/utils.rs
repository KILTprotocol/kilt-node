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

use std::path::{Path, PathBuf};

use runtime_common::{AccountId, AccountPublic};
use sc_service::Properties;
use sp_core::{Pair, Public};
use sp_runtime::traits::IdentifyAccount;

use crate::chain_spec::{self, ParachainRuntime, PeregrineRuntime, SpiritnetRuntime};

/// Helper function to generate an account ID from seed
pub(crate) fn get_account_id_from_secret<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_public_key_from_secret::<TPublic>(seed)).into_account()
}

/// Helper function to generate a crypto pair from seed
pub(crate) fn get_public_key_from_secret<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

pub(crate) fn get_properties(symbol: &str, decimals: u32, ss58format: u32) -> Properties {
	Properties::from_iter(
		[
			("tokenSymbol".into(), symbol.into()),
			("tokenDecimals".into(), decimals.into()),
			("ss58Format".into(), ss58format.into()),
		]
		.into_iter(),
	)
}

pub(crate) fn load_spec(id: &str) -> Result<Box<dyn sc_service::ChainSpec>, String> {
	let runtime = id.parse::<ParachainRuntime>()?;

	match runtime {
		ParachainRuntime::Peregrine(pr) => match pr {
			PeregrineRuntime::Dev => Ok(Box::new(chain_spec::peregrine::dev::generate_chain_spec(
				"rococo_local",
			))),
			PeregrineRuntime::New => Ok(Box::new(chain_spec::peregrine::new::generate_chain_spec())),
			PeregrineRuntime::Peregrine => Ok(Box::new(chain_spec::peregrine::load_chain_spec(
				get_chainspec_full_path("peregrine/peregrine-paseo.json")
					.to_str()
					.unwrap(),
			)?)),
			PeregrineRuntime::PeregrineStg => Ok(Box::new(chain_spec::peregrine::load_chain_spec(
				get_chainspec_full_path("peregrine-stg/peregrine-stg.json")
					.to_str()
					.unwrap(),
			)?)),
			PeregrineRuntime::Rilt => Ok(Box::new(chain_spec::peregrine::load_chain_spec(
				get_chainspec_full_path("rilt/peregrine-rilt.json").to_str().unwrap(),
			)?)),
			PeregrineRuntime::Other(s) => Ok(Box::new(chain_spec::peregrine::load_chain_spec(s.as_str())?)),
		},
		ParachainRuntime::Spiritnet(sr) => match sr {
			SpiritnetRuntime::Dev => Ok(Box::new(chain_spec::spiritnet::dev::generate_chain_spec(
				"rococo_local",
			))),
			SpiritnetRuntime::New => Ok(Box::new(chain_spec::spiritnet::new::generate_chain_spec())),
			SpiritnetRuntime::Spiritnet => Ok(Box::new(chain_spec::spiritnet::load_chain_spec(
				get_chainspec_full_path("spiritnet/spiritnet.json").to_str().unwrap(),
			)?)),
			SpiritnetRuntime::Other(s) => Ok(Box::new(chain_spec::spiritnet::load_chain_spec(s.as_str())?)),
		},
	}
}

const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");
const CHAINSPECS_FOLDER: &str = "chainspecs";

// Prepends the given path with the `<workspace_root>/chainspecs` path.
fn get_chainspec_full_path(path: &str) -> PathBuf {
	Path::new(MANIFEST_DIR)
		.join("..")
		.join("..")
		.join(CHAINSPECS_FOLDER)
		.join(path)
		.canonicalize()
		.expect("Invalid path provided.")
}
