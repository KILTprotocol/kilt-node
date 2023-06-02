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

use xcm_emulator::{decl_test_network, decl_test_parachain};

use crate::relay::RococoChain;

mod para;
mod relay;

#[cfg(test)]
mod tests;

decl_test_parachain! {
	pub struct ProviderParachain {
		Runtime = para::provider::Runtime,
		RuntimeOrigin = para::provider::RuntimeOrigin,
		XcmpMessageHandler = para::provider::XcmpQueue,
		DmpMessageHandler = para::provider::DmpQueue,
		new_ext = para::provider::para_ext(),
	}
}

decl_test_parachain! {
	pub struct ConsumerParachain {
		Runtime = para::consumer::Runtime,
		RuntimeOrigin = para::consumer::RuntimeOrigin,
		XcmpMessageHandler = para::consumer::XcmpQueue,
		DmpMessageHandler = para::consumer::DmpQueue,
		new_ext = para::consumer::para_ext(),
	}
}

decl_test_network! {
	pub struct Network {
		relay_chain = RococoChain,
		parachains = vec![
			// TODO: Change when and if the macro will allow arbitrary expressions.
			// Until then, these have to match the PARA_ID consts in the para submodules.
			(2_000, ProviderParachain),
			(2_001, ConsumerParachain),
		],
	}
}