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

use runtime_common::{AccountId, AccountPublic};
use sc_service::Properties;
use sp_core::{Pair, Public};
use sp_runtime::traits::IdentifyAccount;

use crate::chain_spec;

/// Helper function to generate an account ID from seed
pub(crate) fn get_account_id_from_secret<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_secret::<TPublic>(seed)).into_account()
}

/// Helper function to generate a crypto pair from seed
pub(crate) fn get_from_secret<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
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
	let runtime = id.parse::<chain_spec::ChainRuntime>()?;

	log::trace!(target: chain_spec::LOG_TARGET, "Loading spec id: {id}.");
	log::trace!(target: chain_spec::LOG_TARGET, "The following runtime was chosen based on the spec id: {runtime}.");

	match (id, runtime) {
		// Peregrine development
		("dev", _) => Ok(Box::new(chain_spec::peregrine::dev::generate_chain_spec(
			"rococo_local",
		))),
		// New blank Peregrine chainspec
		("peregrine-new", _) => Ok(Box::new(chain_spec::peregrine::new::generate_chain_spec())),
		// Peregrine chainspec
		("peregrine", _) => Ok(Box::new(chain_spec::peregrine::load_chain_spec(
			"chain_spec/peregrine/specs/peregrine.json",
		)?)),
		// Peregrine staging chainspec
		("peregrine-stg", _) => Ok(Box::new(chain_spec::peregrine::load_chain_spec(
			"chain_spec/peregrine/specs/peregrine-stg.json",
		)?)),
		// RILT chainspec
		("rilt", _) => Ok(Box::new(chain_spec::peregrine::load_chain_spec(
			"chain_spec/peregrine/specs/peregrine-rilt.json",
		)?)),
		// Any other Peregrine-based chainspec
		(s, chain_spec::ChainRuntime::Peregrine) => Ok(Box::new(chain_spec::peregrine::load_chain_spec(s)?)),

		// Spiritnet development
		("spiritnet-dev", _) => Ok(Box::new(chain_spec::spiritnet::dev::generate_chain_spec(
			"rococo_local",
		))),
		// New blank Spiritnet chainspec
		("spiritnet-new", _) => Ok(Box::new(chain_spec::spiritnet::new::generate_chain_spec())),
		// Spiritnet chainspec
		("spiritnet", _) => Ok(Box::new(chain_spec::spiritnet::load_chain_spec(
			"chain_spec/spiritnet/specs/spiritnet.json",
		)?)),
		// Any other Spiritnet-based chainspec
		(s, chain_spec::ChainRuntime::Spiritnet) => Ok(Box::new(chain_spec::spiritnet::load_chain_spec(s)?)),
	}
}
