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

use frame_support::assert_err;
use xcm::{
	v3::{AssetId, AssetInstance, Fungibility, Junction, Junctions, MultiAsset, MultiLocation},
	VersionedMultiAsset,
};
use xcm_executor::traits::{Error, MatchesFungibles};

use crate::xcm::{
	r#match::mock::{get_switch_pair_info_for_remote_location, ExtBuilder, MockRuntime},
	MatchesSwitchPairXcmFeeFungibleAsset,
};

// TODO: Not an easy way to test inconsistent XCM versions. Write those tests
// when a new XCM version is added to the codebase.

#[test]
fn successful() {
	let location = MultiLocation {
		parents: 1,
		interior: Junctions::X1(Junction::Parachain(1_000)),
	};
	let new_switch_pair_info = get_switch_pair_info_for_remote_location(&location);
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build()
		.execute_with(|| {
			let (asset_location, asset_amount): (MultiLocation, u128) =
				MatchesSwitchPairXcmFeeFungibleAsset::<MockRuntime, _>::matches_fungibles(&MultiAsset {
					id: AssetId::Concrete(location),
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
fn fails_on_switch_pair_not_set() {
	ExtBuilder::default().build().execute_with(|| {
		assert_err!(
			MatchesSwitchPairXcmFeeFungibleAsset::<MockRuntime, _>::matches_fungibles(&MultiAsset {
				id: AssetId::Concrete(MultiLocation {
					parents: 1,
					interior: Junctions::X1(Junction::Parachain(1_000)),
				}),
				fun: Fungibility::Fungible(u128::MAX),
			}) as Result<(_, u128), _>,
			Error::AssetNotHandled
		);
	});
}

#[test]
fn fails_on_different_asset() {
	let location = MultiLocation {
		parents: 1,
		interior: Junctions::X1(Junction::Parachain(1_000)),
	};
	let new_switch_pair_info = get_switch_pair_info_for_remote_location(&location);
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info)
		.build()
		.execute_with(|| {
			let different_location = MultiLocation {
				parents: 1,
				// Different para ID.
				interior: Junctions::X1(Junction::Parachain(1_001)),
			};
			assert_err!(
				MatchesSwitchPairXcmFeeFungibleAsset::<MockRuntime, _>::matches_fungibles(&MultiAsset {
					id: AssetId::Concrete(different_location),
					fun: Fungibility::Fungible(u128::MAX),
				}) as Result<(_, u128), _>,
				Error::AssetNotHandled
			);
		});
}

#[test]
fn fails_on_not_concrete_stored_asset() {
	let location = MultiLocation {
		parents: 1,
		interior: Junctions::X1(Junction::Parachain(1_000)),
	};
	let abstract_asset_id = AssetId::Abstract([1; 32]);
	let new_switch_pair_info = {
		let mut new_switch_pair_info = get_switch_pair_info_for_remote_location(&location);
		// Set XCM fee asset to one with an abstract ID.
		new_switch_pair_info.remote_xcm_fee = VersionedMultiAsset::V3(MultiAsset {
			id: abstract_asset_id,
			fun: Fungibility::Fungible(10_000),
		});
		new_switch_pair_info
	};
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info)
		.build()
		.execute_with(|| {
			assert_err!(
				MatchesSwitchPairXcmFeeFungibleAsset::<MockRuntime, _>::matches_fungibles(&MultiAsset {
					id: abstract_asset_id,
					fun: Fungibility::Fungible(u128::MAX),
				}) as Result<(_, u128), _>,
				Error::AssetIdConversionFailed
			);
		});
}

// TODO: Resume from here
#[test]
fn fails_on_not_fungible_stored_asset() {
	let location = MultiLocation {
		parents: 1,
		interior: Junctions::X1(Junction::Parachain(1_000)),
	};
	let non_fungible_asset_amount = Fungibility::NonFungible(AssetInstance::Index(1));
	let new_switch_pair_info = {
		let mut new_switch_pair_info = get_switch_pair_info_for_remote_location(&location);
		// Set XCM fee asset to one with an abstract ID.
		new_switch_pair_info.remote_xcm_fee = VersionedMultiAsset::V3(MultiAsset {
			id: AssetId::Concrete(location),
			fun: non_fungible_asset_amount,
		});
		new_switch_pair_info
	};
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info)
		.build()
		.execute_with(|| {
			assert_err!(
				MatchesSwitchPairXcmFeeFungibleAsset::<MockRuntime, _>::matches_fungibles(&MultiAsset {
					id: AssetId::Concrete(location),
					fun: Fungibility::Fungible(u128::MAX),
				}) as Result<(_, u128), _>,
				Error::AssetIdConversionFailed
			);
		});
}

#[test]
fn fails_on_not_fungible_asset() {
	let location = MultiLocation {
		parents: 1,
		interior: Junctions::X1(Junction::Parachain(1_000)),
	};
	let new_switch_pair_info = get_switch_pair_info_for_remote_location(&location);
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info)
		.build()
		.execute_with(|| {
			assert_err!(
				MatchesSwitchPairXcmFeeFungibleAsset::<MockRuntime, _>::matches_fungibles(&MultiAsset {
					id: AssetId::Concrete(location),
					fun: Fungibility::NonFungible(AssetInstance::Index(1)),
				}) as Result<(_, u128), _>,
				Error::AmountToBalanceConversionFailed
			);
		});
}