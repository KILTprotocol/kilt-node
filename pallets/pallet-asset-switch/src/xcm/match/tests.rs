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

use frame_support::assert_noop;
use xcm::{
	v4::{Asset, AssetId, AssetInstance, Fungibility, Junction, Junctions, Location},
	IntoVersion, VersionedAsset,
};
use xcm_executor::traits::{Error, MatchesFungibles};

use crate::{
	xcm::{
		r#match::mock::{ExtBuilder, MockRuntime},
		test_utils::get_switch_pair_info_for_remote_location,
		MatchesSwitchPairXcmFeeFungibleAsset,
	},
	SwitchPairStatus,
};

#[test]
fn successful_with_stored_latest() {
	let location = xcm::latest::Location {
		parents: 1,
		interior: xcm::latest::Junctions::X1([xcm::latest::Junction::Parachain(1_000)].into()),
	};
	let new_switch_pair_info = {
		let mut new_switch_pair_info =
			get_switch_pair_info_for_remote_location::<MockRuntime>(&location, SwitchPairStatus::Running);
		// Set XCM fee asset to the latest XCM version.
		new_switch_pair_info.remote_xcm_fee = new_switch_pair_info.remote_xcm_fee.into_latest().unwrap();
		new_switch_pair_info
	};
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info)
		.build_and_execute_with_sanity_tests(|| {
			let (asset_location, asset_amount): (Location, u128) =
				MatchesSwitchPairXcmFeeFungibleAsset::<MockRuntime, _>::matches_fungibles(&Asset {
					id: AssetId(location.clone()),
					fun: Fungibility::Fungible(u128::MAX),
				})
				.unwrap();
			// Asset location should match the one stored in the switch pair.
			assert_eq!(asset_location, location);
			// Asset amount should match the input one.
			assert_eq!(asset_amount, u128::MAX);
		});
}

#[test]
fn successful_with_stored_v4() {
	let location = Location {
		parents: 1,
		interior: Junctions::X1([Junction::Parachain(1_000)].into()),
	};
	let new_switch_pair_info =
		get_switch_pair_info_for_remote_location::<MockRuntime>(&location, SwitchPairStatus::Running);
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info)
		.build_and_execute_with_sanity_tests(|| {
			let (asset_location, asset_amount): (Location, u128) =
				MatchesSwitchPairXcmFeeFungibleAsset::<MockRuntime, _>::matches_fungibles(&Asset {
					id: AssetId(location.clone()),
					fun: Fungibility::Fungible(u128::MAX),
				})
				.unwrap();
			// Asset location should match the one stored in the switch pair.
			assert_eq!(asset_location, location);
			// Asset amount should match the input one.
			assert_eq!(asset_amount, u128::MAX);
		});
}

#[test]
fn successful_with_stored_v3() {
	let location = xcm::v3::MultiLocation {
		parents: 1,
		interior: xcm::v3::Junctions::X1(xcm::v3::Junction::Parachain(1_000)),
	};
	let new_switch_pair_info = get_switch_pair_info_for_remote_location::<MockRuntime>(
		&location.try_into().unwrap(),
		SwitchPairStatus::Running,
	);
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info)
		.build_and_execute_with_sanity_tests(|| {
			let location_v4: Location = location.try_into().unwrap();

			let (asset_location, asset_amount): (Location, u128) =
				MatchesSwitchPairXcmFeeFungibleAsset::<MockRuntime, _>::matches_fungibles(&Asset {
					id: AssetId(location_v4.clone()),
					fun: Fungibility::Fungible(u128::MAX),
				})
				.unwrap();
			// Asset location should match the one stored in the switch pair.
			assert_eq!(asset_location, location_v4);
			// Asset amount should match the input one.
			assert_eq!(asset_amount, u128::MAX);
		});
}

#[test]
fn successful_with_stored_v2() {
	let location = xcm::v2::MultiLocation {
		parents: 1,
		interior: xcm::v2::Junctions::X1(xcm::v2::Junction::Parachain(1_000)),
	};
	let location_v3: xcm::v3::MultiLocation = location.try_into().unwrap();
	let new_switch_pair_info = {
		let mut new_switch_pair_info = get_switch_pair_info_for_remote_location::<MockRuntime>(
			&location_v3.try_into().unwrap(),
			SwitchPairStatus::Running,
		);
		// Set XCM fee asset to an XCM v2.
		new_switch_pair_info.remote_xcm_fee = new_switch_pair_info.remote_xcm_fee.into_version(2).unwrap();
		new_switch_pair_info
	};
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info)
		.build_and_execute_with_sanity_tests(|| {
			let (asset_location, asset_amount): (Location, u128) =
				MatchesSwitchPairXcmFeeFungibleAsset::<MockRuntime, _>::matches_fungibles(&Asset {
					id: AssetId(location_v3.try_into().unwrap()),
					fun: Fungibility::Fungible(u128::MAX),
				})
				.unwrap();
			// Asset location should match the one stored in the switch pair.
			assert_eq!(asset_location, location_v3.try_into().unwrap());
			// Asset amount should match the input one.
			assert_eq!(asset_amount, u128::MAX);
		});
}

#[test]
fn skips_on_switch_pair_not_set() {
	ExtBuilder::default().build_and_execute_with_sanity_tests(|| {
		assert_noop!(
			MatchesSwitchPairXcmFeeFungibleAsset::<MockRuntime, _>::matches_fungibles(&Asset {
				id: AssetId(Location {
					parents: 1,
					interior: Junctions::X1([Junction::Parachain(1_000)].into()),
				}),
				fun: Fungibility::Fungible(u128::MAX),
			}) as Result<(_, u128), _>,
			Error::AssetNotHandled
		);
	});
}

#[test]
fn skips_on_switch_pair_not_enabled() {
	let location = Location {
		parents: 1,
		interior: Junctions::X1([Junction::Parachain(1_000)].into()),
	};
	let new_switch_pair_info =
		get_switch_pair_info_for_remote_location::<MockRuntime>(&location, SwitchPairStatus::Paused);
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info)
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				MatchesSwitchPairXcmFeeFungibleAsset::<MockRuntime, _>::matches_fungibles(&Asset {
					id: AssetId(location),
					fun: Fungibility::Fungible(u128::MAX),
				}) as Result<(_, u128), _>,
				Error::AssetNotHandled
			);
		});
}

#[test]
fn skips_on_different_asset() {
	let location = Location {
		parents: 1,
		interior: Junctions::X1([Junction::Parachain(1_000)].into()),
	};
	let new_switch_pair_info =
		get_switch_pair_info_for_remote_location::<MockRuntime>(&location, SwitchPairStatus::Running);
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info)
		.build_and_execute_with_sanity_tests(|| {
			let different_location = Location {
				parents: 1,
				// Different para ID.
				interior: Junctions::X1([Junction::Parachain(1_001)].into()),
			};
			assert_noop!(
				MatchesSwitchPairXcmFeeFungibleAsset::<MockRuntime, _>::matches_fungibles(&Asset {
					id: AssetId(different_location),
					fun: Fungibility::Fungible(u128::MAX),
				}) as Result<(_, u128), _>,
				Error::AssetNotHandled
			);
		});
}

#[test]
fn skips_on_non_fungible_stored_asset() {
	let location = Location {
		parents: 1,
		interior: Junctions::X1([Junction::Parachain(1_000)].into()),
	};
	let non_fungible_asset_amount = Fungibility::NonFungible(AssetInstance::Index(1));
	let new_switch_pair_info = {
		let mut new_switch_pair_info =
			get_switch_pair_info_for_remote_location::<MockRuntime>(&location, SwitchPairStatus::Running);
		// Set XCM fee asset to one with a non-fungible amount.
		new_switch_pair_info.remote_xcm_fee = VersionedAsset::V4(Asset {
			id: AssetId(location.clone()),
			fun: non_fungible_asset_amount,
		});
		new_switch_pair_info
	};
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info)
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				MatchesSwitchPairXcmFeeFungibleAsset::<MockRuntime, _>::matches_fungibles(&Asset {
					id: AssetId(location),
					fun: Fungibility::Fungible(u128::MAX),
				}) as Result<(_, u128), _>,
				Error::AssetNotHandled
			);
		});
}

#[test]
fn fails_on_non_fungible_input_asset() {
	let location = Location {
		parents: 1,
		interior: Junctions::X1([Junction::Parachain(1_000)].into()),
	};
	let new_switch_pair_info =
		get_switch_pair_info_for_remote_location::<MockRuntime>(&location, SwitchPairStatus::Running);
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info)
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				MatchesSwitchPairXcmFeeFungibleAsset::<MockRuntime, _>::matches_fungibles(&Asset {
					id: AssetId(location),
					fun: Fungibility::NonFungible(AssetInstance::Index(1)),
				}) as Result<(_, u128), _>,
				Error::AmountToBalanceConversionFailed
			);
		});
}
