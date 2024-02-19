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

pub(crate) use crate::{
	xcm_config::{
		tests::utils::{get_account_id_from_seed, get_from_seed},
		RelayNetworkId,
	},
	AuthorityId, Balances, DmpQueue, ParachainInfo, ParachainInfoConfig, ParachainSystem, PolkadotXcmConfig,
	Runtime as PeregrineRuntime, RuntimeCall, RuntimeEvent, RuntimeGenesisConfig, RuntimeOrigin, SessionConfig,
	SessionKeys, System, SystemConfig, XcmpQueue, WASM_BINARY,
};
pub(crate) use runtime_common::{xcm_config::LocationToAccountId, AccountPublic};
use sp_core::sr25519;
use sp_runtime::{BuildStorage, Storage};
use xcm_emulator::{decl_test_parachains, BridgeMessageHandler, Parachain, TestExt};

const PARA_ID: u32 = 2_000;
const SAFE_XCM_VERSION: u32 = xcm::prelude::XCM_VERSION;

fn genesis() -> Storage {
	RuntimeGenesisConfig {
		system: SystemConfig {
			code: WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			..Default::default()
		},
		parachain_info: ParachainInfoConfig {
			parachain_id: PARA_ID.into(),
			..Default::default()
		},
		polkadot_xcm: PolkadotXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
			..Default::default()
		},
		session: SessionConfig {
			keys: vec![(
				get_account_id_from_seed::<AccountPublic, sr25519::Public>("Alice"),
				get_from_seed::<AuthorityId>("Alice"),
			)]
			.iter()
			.map(|(acc, key)| (acc.clone(), acc.clone(), SessionKeys { aura: key.clone() }))
			.collect::<Vec<_>>(),
		},
		..Default::default()
	}
	.build_storage()
	.unwrap()
}

decl_test_parachains! {
	pub struct Peregrine {
		genesis = genesis(),
		on_init = (),
		runtime = {
			Runtime: PeregrineRuntime,
			RuntimeOrigin: RuntimeOrigin,
			RuntimeCall: RuntimeCall,
			RuntimeEvent: RuntimeEvent,
			XcmpMessageHandler: XcmpQueue,
			DmpMessageHandler: DmpQueue,
			LocationToAccountId: LocationToAccountId<RelayNetworkId>,
			System: System,
			Balances: Balances,
			ParachainSystem: ParachainSystem,
			ParachainInfo: ParachainInfo,
		},
		pallets_extra = {}
	}
}
