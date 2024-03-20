use crate::mock::{
	network::MockNetworkRococo,
	para_chains::{peregrine, AssetHubRococo, AssetHubRococoPallet, Peregrine},
	relay_chains::{Rococo, RococoPallet},
};
use frame_support::assert_ok;
use xcm::{v3::WeightLimit, VersionedMultiLocation, VersionedXcm};
use xcm_emulator::{
	assert_expected_events, BodyId, BodyPart, Instruction::UnpaidExecution, Junction, Junctions, MultiLocation,
	Outcome, Parachain, ParentThen, Plurality, RelayChain, TestExt, Xcm, X1,
};

#[test]
fn test_unpaid_execution_from_asset_hub_to_peregrine() {
	MockNetworkRococo::reset();

	let sudo_origin = <AssetHubRococo as Parachain>::RuntimeOrigin::root();
	let parachain_destination: VersionedMultiLocation =
		ParentThen(Junctions::X1(Junction::Parachain(peregrine::PARA_ID))).into();

	let weight_limit = WeightLimit::Unlimited;
	let check_origin = None;

	let xcm = VersionedXcm::from(Xcm(vec![UnpaidExecution {
		weight_limit,
		check_origin,
	}]));

	//Send XCM message from relay chain
	AssetHubRococo::execute_with(|| {
		assert_ok!(<AssetHubRococo as AssetHubRococoPallet>::PolkadotXcm::send(
			sudo_origin,
			Box::new(parachain_destination),
			Box::new(xcm)
		));

		type RuntimeEvent = <AssetHubRococo as Parachain>::RuntimeEvent;

		assert_expected_events!(
			AssetHubRococo,
			vec![
				RuntimeEvent::PolkadotXcm(pallet_xcm::Event::Sent { .. }) => {},
			]
		);
	});

	Peregrine::execute_with(|| {
		type PeregrineRuntimeEvent = <Peregrine as Parachain>::RuntimeEvent;
		assert_expected_events!(
			Peregrine,
			vec![
				PeregrineRuntimeEvent::XcmpQueue(cumulus_pallet_xcmp_queue::Event::Fail {
					error: xcm::v3::Error::Barrier,
					..
				}) => {},
			]
		);
	});

	// No event on Rococo. message is meant for Peregrine
	Rococo::execute_with(|| {
		assert_eq!(Rococo::events().len(), 0);
	});
}

#[test]
fn test_unpaid_execution_from_rococo_to_peregrine() {
	MockNetworkRococo::reset();

	let sudo_origin = <Rococo as RelayChain>::RuntimeOrigin::root();
	let parachain_destination: VersionedMultiLocation = Junctions::X1(Junction::Parachain(peregrine::PARA_ID)).into();

	let weight_limit = WeightLimit::Unlimited;
	let check_origin = Some(MultiLocation {
		parents: 1,
		interior: X1(Plurality {
			id: BodyId::Legislative,
			part: BodyPart::Voice,
		}),
	});

	let xcm = VersionedXcm::from(Xcm(vec![UnpaidExecution {
		weight_limit,
		check_origin,
	}]));

	//Send XCM message from relay chain
	Rococo::execute_with(|| {
		assert_ok!(<Rococo as RococoPallet>::XcmPallet::send(
			sudo_origin,
			Box::new(parachain_destination),
			Box::new(xcm)
		));

		type RuntimeEvent = <Rococo as RelayChain>::RuntimeEvent;

		assert_expected_events!(
			Rococo,
			vec![
				RuntimeEvent::XcmPallet(pallet_xcm::Event::Sent { .. }) => {},
			]
		);
	});

	Peregrine::execute_with(|| {
		type PeregrineRuntimeEvent = <Peregrine as Parachain>::RuntimeEvent;
		assert_expected_events!(
			Peregrine,
			vec![
				PeregrineRuntimeEvent::DmpQueue(cumulus_pallet_dmp_queue::Event::ExecutedDownward {
					outcome: Outcome::Error(xcm::v3::Error::Barrier),
					..
				}) => {},
			]
		);
	});

	// No event on AssetHubRococo. message is meant for Peregrine
	AssetHubRococo::execute_with(|| {
		assert_eq!(AssetHubRococo::events().len(), 0);
	});
}
