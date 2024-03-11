use integration_tests_common::Get;
use polkadot_primitives::Balance;
use sp_core::sr25519;
use xcm_emulator::{
	decl_test_networks, AccountId, Ancestor, BridgeMessageHandler, MultiLocation, Parachain, Parent, RelayChain,
	TestExt, X1,
};
use xcm_executor::traits::ConvertLocation;

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
