use crate::mock::{
	network::{MockNetwork, Polkadot},
	parachains::{AssetHub, Spiritnet},
	utils::get_account_id_from_seed,
};
use asset_hub_polkadot_runtime::{RuntimeEvent as AssetHubRuntimeEvent, System as AssetHubSystem};
use frame_support::dispatch::RawOrigin;
use frame_support::{assert_err, assert_ok};
use integration_tests_common::{asset_hub_polkadot, polkadot::ED, ALICE, BOB};
use polkadot_runtime::System as PolkadotSystem;
use sp_core::sr25519;
use sp_runtime::{DispatchError, ModuleError};
use spiritnet_runtime::{
	PolkadotXcm as SpiritnetXcm, RuntimeEvent as SpiritnetRuntimeEvent, System as SpiritnetSystem,
};
use xcm::v3::WeightLimit;
use xcm_emulator::{
	cumulus_pallet_xcmp_queue::Event as XcmpQueueEvent, Here, Junction, Junctions, Parent, ParentThen, TestExt, X1,
};

#[test]
fn test_reserve_asset_transfer_from_regular_account_to_relay() {
	MockNetwork::reset();

	let alice_account_id_on_peregrine = get_account_id_from_seed::<sr25519::Public>(ALICE);

	Spiritnet::execute_with(|| {
		assert_ok!(SpiritnetXcm::limited_reserve_transfer_assets(
			RawOrigin::Signed(alice_account_id_on_peregrine.clone()).into(),
			Box::new(Parent.into()),
			Box::new(
				X1(Junction::AccountId32 {
					network: None,
					id: alice_account_id_on_peregrine.into()
				})
				.into()
			),
			Box::new((Here, 1_000_000).into()),
			0,
			WeightLimit::Unlimited,
		));
		assert!(matches!(
			SpiritnetSystem::events()
				.first()
				.expect("An event should be emitted when sending an XCM message.")
				.event,
			SpiritnetRuntimeEvent::PolkadotXcm(pallet_xcm::Event::Attempted {
				outcome: xcm::latest::Outcome::Error(xcm::latest::Error::Barrier)
			})
		));
	});
	// No message should reach the relaychain.
	Polkadot::execute_with(|| {
		assert_eq!(PolkadotSystem::events().len(), 0);
	})
}

/// Test that a reserved transfer to the relaychain is failing. We don't want to
/// allow transfers to the relaychain since the funds might be lost.
#[test]
fn test_reserve_asset_transfer_from_regular_account_to_asset_hub() {
	MockNetwork::reset();

	let alice_account_id = get_account_id_from_seed::<sr25519::Public>(ALICE);
	let bob_account_id = get_account_id_from_seed::<sr25519::Public>(BOB);

	Spiritnet::execute_with(|| {
		assert_ok!(SpiritnetXcm::limited_reserve_transfer_assets(
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
		));

		assert!(
			matches!(
				SpiritnetSystem::events()
					.last()
					.expect("An event should be emitted when sending an XCM message.")
					.event,
				SpiritnetRuntimeEvent::PolkadotXcm(pallet_xcm::Event::Attempted {
					outcome: xcm::latest::Outcome::Complete(_)
				})
			),
			"Didn't match {:?}",
			SpiritnetSystem::events().last()
		);
	});
	// No event on the relaychain (message is meant for asset hub)
	Polkadot::execute_with(|| {
		assert_eq!(PolkadotSystem::events().len(), 0);
	});
	// Fails on AsssetHub since spiritnet is not a trusted registrar.
	AssetHub::execute_with(|| {
		assert!(
			matches!(
				AssetHubSystem::events()
					.last()
					.expect("An event should be emitted when sending an XCM message.")
					.event,
				AssetHubRuntimeEvent::XcmpQueue(XcmpQueueEvent::Fail { .. })
			),
			"Didn't match {:?}",
			AssetHubSystem::events().last()
		);
	});
}

#[test]
fn test_teleport_asset_from_regular_account_to_asset_hub() {
	MockNetwork::reset();

	let alice_account_id = get_account_id_from_seed::<sr25519::Public>(ALICE);
	let bob_account_id = get_account_id_from_seed::<sr25519::Public>(BOB);

	Spiritnet::execute_with(|| {
		assert_err!(
			SpiritnetXcm::limited_teleport_assets(
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
			DispatchError::Module(ModuleError {
				index: 83,
				error: [2, 0, 0, 0],
				message: Some("Filtered")
			})
		);
	});
	// No event on the relaychain (message is meant for asset hub)
	Polkadot::execute_with(|| {
		assert_eq!(PolkadotSystem::events().len(), 0);
	});
	// Fails on AsssetHub since spiritnet is not a trusted registrar.
	AssetHub::execute_with(|| {
		assert_eq!(AssetHubSystem::events().len(), 0);
	});
}

// TODO: Receive funds from assetHub
// TODO: Disallow root calls from other chains.
// TODO: create a DID from another chain
// TODO: use a DID (e.g. CType creation)
