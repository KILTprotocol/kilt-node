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

use frame_support::{assert_noop, dispatch::RawOrigin};
use integration_tests_common::constants::{
	accounts::{ALICE, BOB},
	asset_hub_polkadot,
	polkadot::ED,
};
use peregrine_runtime::PolkadotXcm as PeregrineXcm;
use sp_core::sr25519;
use xcm::v3::prelude::{Here, Junction, Junctions, ParentThen, WeightLimit, X1};
use xcm_emulator::{Chain, Network, TestExt};

use crate::{
	mock::{
		network::MockNetworkRococo,
		para_chains::{AssetHubRococo, Peregrine},
		relay_chains::Rococo,
	},
	utils::get_account_id_from_seed,
};

#[test]
fn test_teleport_asset_from_regular_peregrine_account_to_asset_hub() {
	MockNetworkRococo::reset();

	let alice_account_id = get_account_id_from_seed::<sr25519::Public>(ALICE);
	let bob_account_id = get_account_id_from_seed::<sr25519::Public>(BOB);

	Peregrine::execute_with(|| {
		assert_noop!(
			PeregrineXcm::limited_teleport_assets(
				RawOrigin::Signed(alice_account_id.clone()).into(),
				Box::new(ParentThen(Junctions::X1(Junction::Parachain(asset_hub_polkadot::PARA_ID))).into()),
				Box::new(
					X1(Junction::AccountId32 {
						network: None,
						id: bob_account_id.into()
					})
					.into()
				),
				Box::new((Here, 1000 * ED).into()),
				0,
				WeightLimit::Unlimited,
			),
			pallet_xcm::Error::<peregrine_runtime::Runtime>::Filtered
		);
	});
	// No event on the relaychain Message is for AssetHub
	Rococo::execute_with(|| {
		assert_eq!(Rococo::events().len(), 0);
	});
	// AssetHub should not receive any message, since the message is filtered out.
	AssetHubRococo::execute_with(|| {
		assert_eq!(AssetHubRococo::events().len(), 0);
	});
}
