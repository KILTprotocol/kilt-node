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

use frame_support::assert_ok;
use xcm::{v3::WeightLimit, VersionedMultiLocation, VersionedXcm};
use xcm_emulator::{
	assert_expected_events, BodyId, BodyPart, Instruction::UnpaidExecution, Junction, Junctions, MultiLocation,
	Outcome, Parachain, ParentThen, Plurality, RelayChain, TestExt, Xcm, X1,
};

use crate::mock::{
	network::MockNetworkRococo,
	para_chains::{peregrine, AssetHubRococo, AssetHubRococoPallet, Peregrine},
	relay_chains::{Rococo, RococoPallet},
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

	//Send XCM message from Parachain
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

// TODO: Check why test is passing. Unpaid execution should work now.
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
