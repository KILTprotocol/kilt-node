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

use asset_hub_rococo_emulated_chain::AssetHubRococo as AssetHubParachain;
use rococo_emulated_chain::Rococo as RococoChain;
use xcm_emulator::decl_test_networks;

use crate::mock::para_chains::{PeregrineParachain, SpiritnetParachain};

#[cfg(not(feature = "runtime-benchmarks"))]
#[cfg(test)]
pub mod chains {

	use super::*;

	pub type Rococo = RococoChain<MockNetwork>;
	pub type Spiritnet = SpiritnetParachain<MockNetwork>;
	pub type Peregrine = PeregrineParachain<MockNetwork>;
	pub type AssetHub = AssetHubParachain<MockNetwork>;
}

decl_test_networks! {
	pub struct MockNetwork {
		relay_chain = RococoChain,
		parachains = vec![
			AssetHubParachain,
			SpiritnetParachain,
			PeregrineParachain,
		],
		bridge = ()
	},
}
