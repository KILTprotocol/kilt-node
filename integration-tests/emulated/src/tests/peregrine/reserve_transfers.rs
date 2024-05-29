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

use cumulus_pallet_xcmp_queue::Event as XcmpQueueEvent;
use frame_support::{assert_ok, dispatch::RawOrigin, traits::fungible::Inspect};
use integration_tests_common::constants::{
	accounts::ALICE,
	asset_hub_polkadot::{self, ED},
};
use peregrine_runtime::PolkadotXcm as PeregrineXcm;
use runtime_common::AccountId;
use sp_core::sr25519;
use sp_runtime::traits::Zero;
use xcm::v3::prelude::{Here, Junction, Junctions, Parent, ParentThen, WeightLimit, X1};
use xcm_emulator::{assert_expected_events, Chain, Network, Parachain, TestExt};

use crate::{
	mock::{
		network::MockNetworkRococo,
		para_chains::{AssetHubRococo, Peregrine, PeregrinePallet},
		relay_chains::Rococo,
	},
	utils::get_account_id_from_seed,
};

/// Test that a reserved transfer to the relaychain is failing. We don't want to
/// allow transfers to the relaychain since the funds might be lost.
#[test]
fn test_reserve_asset_transfer_from_regular_peregrine_account_to_relay() {
	MockNetworkRococo::reset();

	let alice_account = get_account_id_from_seed::<sr25519::Public>(ALICE);

	Peregrine::execute_with(|| {
		assert_ok!(PeregrineXcm::limited_reserve_transfer_assets(
			RawOrigin::Signed(alice_account.clone()).into(),
			Box::new(Parent.into()),
			Box::new(
				X1(Junction::AccountId32 {
					network: None,
					id: alice_account.into()
				})
				.into()
			),
			Box::new((Here, 1_000_000).into()),
			0,
			WeightLimit::Unlimited,
		));

		type RuntimeEvent = <Peregrine as Chain>::RuntimeEvent;

		assert_expected_events!(
			Peregrine,
			vec![RuntimeEvent::PolkadotXcm(pallet_xcm::Event::Attempted {
				outcome: xcm::latest::Outcome::Error(xcm::latest::Error::Barrier)
			}) => {},]
		);
	});
	// No message should reach the relaychain.
	Rococo::execute_with(|| {
		assert_eq!(Rococo::events().len(), 0);
	})
}

#[test]
fn test_reserve_asset_transfer_from_regular_peregrine_account_to_asset_hub() {
	MockNetworkRococo::reset();

	let alice_account_id = get_account_id_from_seed::<sr25519::Public>(ALICE);
	let asset_hub_sovereign_account =
		Peregrine::sovereign_account_id_of(Peregrine::sibling_location_of(AssetHubRococo::para_id()));

	let balance_to_transfer = 10000 * ED;

	Peregrine::execute_with(|| {
		// the sovereign_account of AssetHub should have no coins.
		let balance_before_transfer: u128 =
			<peregrine_runtime::Balances as Inspect<AccountId>>::balance(&asset_hub_sovereign_account);

		assert!(balance_before_transfer.is_zero());

		// submit xcm message
		assert_ok!(PeregrineXcm::limited_reserve_transfer_assets(
			RawOrigin::Signed(alice_account_id.clone()).into(),
			Box::new(ParentThen(Junctions::X1(Junction::Parachain(asset_hub_polkadot::PARA_ID))).into()),
			Box::new(
				X1(Junction::AccountId32 {
					network: None,
					id: asset_hub_sovereign_account.clone().into()
				})
				.into()
			),
			Box::new((Here, balance_to_transfer).into()),
			0,
			WeightLimit::Unlimited,
		));

		type RuntimeEvent = <Peregrine as Chain>::RuntimeEvent;

		// we expect to have the [Complete] event.
		assert_expected_events!(
			Peregrine,
			vec![RuntimeEvent::PolkadotXcm(pallet_xcm::Event::Attempted {
				outcome: xcm::latest::Outcome::Complete(_)
			}) => {},
			]
		);

		// we also expect that the sovereignAccount of AssetHub has some coins now
		let balance_after_transfer: u128 =
			<<Peregrine as PeregrinePallet>::Balances as Inspect<AccountId>>::balance(&asset_hub_sovereign_account);

		assert_eq!(balance_after_transfer, balance_to_transfer);
	});
	// No event on the relaychain (message is meant for AssetHub.
	Rococo::execute_with(|| {
		assert_eq!(Rococo::events().len(), 0);
	});
	// Fails on AssetHub since peregrine is not a trusted registrar
	AssetHubRococo::execute_with(|| {
		type RuntimeEvent = <AssetHubRococo as Chain>::RuntimeEvent;

		assert_expected_events!(
			AssetHubRococo,
			vec![RuntimeEvent::XcmpQueue(XcmpQueueEvent::Fail { .. }) => {},]
		);
	});
}
