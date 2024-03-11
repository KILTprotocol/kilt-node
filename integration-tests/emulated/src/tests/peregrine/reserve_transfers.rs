use crate::{
	mock::{
		network::MockNetworkRococo,
		para_chains::{AssetHubRococo, Peregrine},
		relay_chains::Rococo,
	},
	utils::get_account_id_from_seed,
};

use frame_support::{assert_ok, dispatch::RawOrigin, traits::fungible::Inspect};
use integration_tests_common::{asset_hub_polkadot, polkadot::ED, ALICE};
use peregrine_runtime::PolkadotXcm as PeregrineXcm;
use rococo_runtime::System as RococoSystem;
use runtime_common::AccountId;
use sp_core::sr25519;
use sp_runtime::traits::Zero;
use xcm::v3::WeightLimit;
use xcm_emulator::{
	assert_expected_events, cumulus_pallet_xcmp_queue::Event as XcmpQueueEvent, Here, Junction, Junctions, Parachain,
	Parent, ParentThen, TestExt, X1,
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

		type RuntimeEvent = <Peregrine as Parachain>::RuntimeEvent;

		assert_expected_events!(
			Peregrine,
			vec![RuntimeEvent::PolkadotXcm(pallet_xcm::Event::Attempted {
				outcome: xcm::latest::Outcome::Error(xcm::latest::Error::Barrier)
			}) => {},]
		);
	});
	// No message should reach the relaychain.
	Rococo::execute_with(|| {
		assert_eq!(RococoSystem::events().len(), 0);
	})
}

#[test]
fn test_reserve_asset_transfer_from_regular_peregrine_account_to_asset_hub() {
	MockNetworkRococo::reset();

	let alice_account_id = get_account_id_from_seed::<sr25519::Public>(ALICE);
	let asset_hub_sovereign_account =
		Peregrine::sovereign_account_id_of(Peregrine::sibling_location_of(AssetHubRococo::para_id()));

	let balance_to_transfer = 1000 * ED;

	Peregrine::execute_with(|| {
		// the sovereign_account of AssetHub should have no coins.

		let balance_before_transfer: u128 =
			<<Peregrine as Parachain>::Balances as Inspect<AccountId>>::balance(&asset_hub_sovereign_account).into();

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

		type RuntimeEvent = <Peregrine as Parachain>::RuntimeEvent;

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
			<<Peregrine as Parachain>::Balances as Inspect<AccountId>>::balance(&asset_hub_sovereign_account).into();

		assert_eq!(balance_after_transfer, balance_to_transfer);
	});
	// No event on the relaychain (message is meant for AssetHub.
	Rococo::execute_with(|| {
		assert_eq!(RococoSystem::events().len(), 0);
	});
	// Fails on AssetHub since peregrine is not a trusted registrar
	AssetHubRococo::execute_with(|| {
		type RuntimeEvent = <AssetHubRococo as Parachain>::RuntimeEvent;

		assert_expected_events!(
			AssetHubRococo,
			vec![RuntimeEvent::XcmpQueue(XcmpQueueEvent::Fail { .. }) => {},]
		);
	});
}
