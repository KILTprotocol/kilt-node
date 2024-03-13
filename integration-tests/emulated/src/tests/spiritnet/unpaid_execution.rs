use crate::mock::{
	network::MockNetworkPolkadot,
	para_chains::{spiritnet, AssetHubPolkadot, AssetHubPolkadotPallet, Spiritnet},
	relay_chains::Polkadot,
};
use frame_support::assert_ok;
use xcm::{v3::WeightLimit, VersionedMultiLocation, VersionedXcm};
use xcm_emulator::{
	assert_expected_events, Instruction::UnpaidExecution, Junction, Junctions, Parachain, ParentThen, TestExt, Xcm,
};

#[test]
fn test_unpaid_execution_to_spiritnet() {
	MockNetworkPolkadot::reset();

	let sudo_origin = <AssetHubPolkadot as Parachain>::RuntimeOrigin::root();
	let parachain_destination: VersionedMultiLocation =
		ParentThen(Junctions::X1(Junction::Parachain(spiritnet::PARA_ID))).into();

	let weight_limit = WeightLimit::Unlimited;
	let check_origin = None;

	let xcm = VersionedXcm::from(Xcm(vec![UnpaidExecution {
		weight_limit,
		check_origin,
	}]));

	//Send XCM message from relay chain
	AssetHubPolkadot::execute_with(|| {
		assert_ok!(<AssetHubPolkadot as AssetHubPolkadotPallet>::PolkadotXcm::send(
			sudo_origin,
			Box::new(parachain_destination),
			Box::new(xcm)
		));

		type RuntimeEvent = <AssetHubPolkadot as Parachain>::RuntimeEvent;

		assert_expected_events!(
			AssetHubPolkadot,
			vec![
				RuntimeEvent::PolkadotXcm(pallet_xcm::Event::Sent { .. }) => {},
			]
		);
	});

	Spiritnet::execute_with(|| {
		type SpiritnetRuntimeEvent = <Spiritnet as Parachain>::RuntimeEvent;
		assert_expected_events!(
			Spiritnet,
			vec![
				SpiritnetRuntimeEvent::XcmpQueue(cumulus_pallet_xcmp_queue::Event::Fail {
					error: xcm::v3::Error::Barrier,
					..
				}) => {},
			]
		);
	});

	// No event on the Polkadot message is meant for Spiritnet
	Polkadot::execute_with(|| {
		assert_eq!(Polkadot::events().len(), 0);
	});
}
