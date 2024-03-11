use crate::mock::{
	network::MockNetworkRococo,
	para_chains::{peregrine, AssetHubRococo, AssetHubRococoPallet, Peregrine},
	relay_chains::Rococo,
};
use frame_support::assert_ok;
use parity_scale_codec::Encode;
use xcm::{v3::WeightLimit, DoubleEncoded, VersionedMultiLocation, VersionedXcm};
use xcm_emulator::{
	assert_expected_events,
	Instruction::{Transact, UnpaidExecution},
	Junction, Junctions, OriginKind, Parachain, ParentThen, TestExt, Weight, Xcm,
};

#[test]
fn test_unpaid_execution_to_peregrine() {
	MockNetworkRococo::reset();

	let code = vec![];

	let call: DoubleEncoded<()> = <Peregrine as Parachain>::RuntimeCall::System(frame_system::Call::set_code { code })
		.encode()
		.into();
	let sudo_origin = <AssetHubRococo as Parachain>::RuntimeOrigin::root();
	let parachain_destination: VersionedMultiLocation =
		ParentThen(Junctions::X1(Junction::Parachain(peregrine::PARA_ID))).into();

	let weight_limit = WeightLimit::Unlimited;
	let require_weight_at_most = Weight::from_parts(1000000000, 200000);
	let origin_kind = OriginKind::Superuser;
	let check_origin = None;

	let xcm = VersionedXcm::from(Xcm(vec![
		UnpaidExecution {
			weight_limit,
			check_origin,
		},
		Transact {
			origin_kind,
			require_weight_at_most,
			call,
		},
	]));

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

		let a = Peregrine::events();
		println!("{:?}", a);

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

	// No event on Rococo. message is meant for AssetHub
	Rococo::execute_with(|| {
		assert_eq!(Rococo::events().len(), 0);
	});
}
