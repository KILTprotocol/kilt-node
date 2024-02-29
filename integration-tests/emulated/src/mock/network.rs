use integration_tests_common::{polkadot, Get};
use polkadot_primitives::Balance;
use sp_core::sr25519;
use xcm_emulator::{
	decl_test_networks, decl_test_relay_chains, AccountId, Ancestor, BridgeMessageHandler, MultiLocation, Parachain,
	Parent, RelayChain, TestExt, XcmHash, X1,
};
use xcm_executor::traits::ConvertLocation;

use crate::mock::parachains::{AssetHub, Spiritnet};

decl_test_relay_chains! {
	#[api_version(5)]
	pub struct Polkadot {
		genesis = polkadot::genesis(),
		on_init = (),
		runtime = {
			Runtime: polkadot_runtime::Runtime,
			RuntimeOrigin: polkadot_runtime::RuntimeOrigin,
			RuntimeCall: polkadot_runtime::RuntimeCall,
			RuntimeEvent: polkadot_runtime::RuntimeEvent,
			MessageQueue: polkadot_runtime::MessageQueue,
			XcmConfig: polkadot_runtime::xcm_config::XcmConfig,
			SovereignAccountOf: polkadot_runtime::xcm_config::SovereignAccountOf,
			System: polkadot_runtime::System,
			Balances: polkadot_runtime::Balances,
		},
		pallets_extra = {
			XcmPallet: polkadot_runtime::XcmPallet,
		}
	}
}

decl_test_networks! {
	pub struct MockNetwork {
		relay_chain = Polkadot,
		parachains = vec![
			AssetHub,
			Spiritnet,
		],
		bridge = ()
	}
}
