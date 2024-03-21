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
	assert_expected_events, Instruction::UnpaidExecution, Junction, Junctions, Outcome, Parachain, ParentThen,
	RelayChain, TestExt, Xcm,
};

use crate::mock::{
	network::MockNetworkPolkadot,
	para_chains::{spiritnet, AssetHubPolkadot, AssetHubPolkadotPallet, Spiritnet},
	relay_chains::{Polkadot, PolkadotPallet},
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

	//Send XCM message from Parachain
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

#[test]
fn test_unpaid_execution_from_polkadot_to_spiritnet() {
	MockNetworkPolkadot::reset();

	let sudo_origin = <Polkadot as RelayChain>::RuntimeOrigin::root();
	let parachain_destination: VersionedMultiLocation = Junctions::X1(Junction::Parachain(spiritnet::PARA_ID)).into();

	let weight_limit = WeightLimit::Unlimited;
	let check_origin = None;

	let xcm = VersionedXcm::from(Xcm(vec![UnpaidExecution {
		weight_limit,
		check_origin,
	}]));

	//Send XCM message from relay chain
	Polkadot::execute_with(|| {
		assert_ok!(<Polkadot as PolkadotPallet>::XcmPallet::send(
			sudo_origin,
			Box::new(parachain_destination),
			Box::new(xcm)
		));

		type RuntimeEvent = <Polkadot as RelayChain>::RuntimeEvent;

		assert_expected_events!(
			Polkadot,
			vec![
				RuntimeEvent::XcmPallet(pallet_xcm::Event::Sent { .. }) => {},
			]
		);
	});

	Spiritnet::execute_with(|| {
		type SpiritnetRuntimeEvent = <Spiritnet as Parachain>::RuntimeEvent;
		assert_expected_events!(
			Spiritnet,
			vec![
				SpiritnetRuntimeEvent::DmpQueue(cumulus_pallet_dmp_queue::Event::ExecutedDownward {
					outcome: Outcome::Complete(_),
					..
				}) => {},
			]
		);
	});

	// No event on AssetHubPolkadot. message is meant for Spiritnet
	AssetHubPolkadot::execute_with(|| {
		assert_eq!(AssetHubPolkadot::events().len(), 0);
	});
}
