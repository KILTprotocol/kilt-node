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

mod peregrine;
mod relaychain;
mod utils;

use crate::PolkadotXcm as PeregrineXcm;
use frame_system::RawOrigin;
use peregrine::{Runtime as PeregrineRuntime, System as PeregrineSystem};
use polkadot_primitives::{AccountId, Balance};
use relaychain::{Runtime as RococoRuntime, System as RococoSystem};
use sp_core::{sr25519, Get};
use xcm::prelude::*;
use xcm_emulator::{decl_test_networks, BridgeMessageHandler, Parachain, RelayChain, TestExt};
use xcm_executor::traits::ConvertLocation;

decl_test_networks! {
	pub struct RococoNetwork {
		relay_chain = RococoRuntime,
		parachains = vec![
			PeregrineRuntime,
		],
		bridge = ()
	}
}

#[test]
fn example() {
	env_logger::init();
	let parent_location: MultiLocation = Parent.into();
	let message: Xcm<()> = vec![
		Instruction::WithdrawAsset((Here, 2_000_000).into()),
		Instruction::BuyExecution {
			fees: (Here, 2_000_000).into(),
			weight_limit: WeightLimit::Unlimited,
		},
	]
	.into();
	PeregrineRuntime::execute_with(|| {
		let res = PeregrineXcm::send(
			RawOrigin::Root.into(),
			Box::new(parent_location.into()),
			Box::new(VersionedXcm::from(message)),
		);
		println!("{:?}", res);
		println!("{:?}", PeregrineSystem::events());
	});
	RococoRuntime::execute_with(|| {
		println!("{:?}", RococoSystem::events());
	})
}
