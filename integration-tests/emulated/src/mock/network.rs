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

use xcm_emulator::decl_test_networks;

use crate::mock::{
	para_chains::{AssetHubPolkadot, AssetHubRococo, Peregrine, Spiritnet},
	relay_chains::{Polkadot, Rococo},
};

decl_test_networks! {
	pub struct MockNetworkPolkadot {
		relay_chain = Polkadot,
		parachains = vec![
			AssetHubPolkadot,
			Spiritnet,
		],
		bridge = ()
	},
	pub struct MockNetworkRococo {
		relay_chain = Rococo,
		parachains = vec![
			AssetHubRococo,
			Peregrine,
		],
		bridge = ()
	}
}
