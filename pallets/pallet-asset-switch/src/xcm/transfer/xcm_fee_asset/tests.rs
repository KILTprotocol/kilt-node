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

use frame_support::traits::ContainsPair;
use xcm::{
	v3::{AssetId, AssetInstance, Fungibility, Junction, Junctions, MultiAsset, MultiLocation},
	IntoVersion,
};

use crate::{
	xcm::{
		test_utils::get_switch_pair_info_for_remote_location,
		transfer::mock::{ExtBuilder, MockRuntime},
		IsSwitchPairXcmFeeAsset,
	},
	SwitchPairStatus,
};

#[test]
fn true_with_stored_xcm_fee_asset_latest() {
	let location = xcm::latest::MultiLocation {
		parents: 1,
		interior: xcm::latest::Junctions::X1(xcm::latest::Junction::Parachain(1_000)),
	};
	let new_switch_pair_info = {
		let mut new_switch_pair_info =
			get_switch_pair_info_for_remote_location::<MockRuntime>(&location, SwitchPairStatus::Running);
		// Set XCM fee asset to the latest XCM version.
		new_switch_pair_info.remote_xcm_fee = new_switch_pair_info.remote_xcm_fee.into_latest().unwrap();
		new_switch_pair_info
	};
	// Works with XCM fungible asset.
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build()
		.execute_with(|| {
			assert!(IsSwitchPairXcmFeeAsset::<MockRuntime, _>::contains(
				&MultiAsset {
					id: MultiAsset::try_from(new_switch_pair_info.clone().remote_xcm_fee)
						.unwrap()
						.id,
					fun: Fungibility::Fungible(1)
				},
				new_switch_pair_info.clone().remote_reserve_location.try_as().unwrap()
			));
		});
	// Works with XCM non-fungible asset.
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build()
		.execute_with(|| {
			assert!(IsSwitchPairXcmFeeAsset::<MockRuntime, _>::contains(
				&MultiAsset {
					id: MultiAsset::try_from(new_switch_pair_info.clone().remote_xcm_fee)
						.unwrap()
						.id,
					fun: Fungibility::NonFungible(AssetInstance::Index(1))
				},
				new_switch_pair_info.remote_reserve_location.try_as().unwrap()
			));
		});
}

#[test]
fn true_with_stored_xcm_fee_asset_v3() {
	let location = MultiLocation {
		parents: 1,
		interior: Junctions::X1(Junction::Parachain(1_000)),
	};
	let new_switch_pair_info =
		get_switch_pair_info_for_remote_location::<MockRuntime>(&location, SwitchPairStatus::Running);
	// Works with remote fungible asset.
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build()
		.execute_with(|| {
			assert!(IsSwitchPairXcmFeeAsset::<MockRuntime, _>::contains(
				&MultiAsset {
					id: MultiAsset::try_from(new_switch_pair_info.clone().remote_xcm_fee)
						.unwrap()
						.id,
					fun: Fungibility::Fungible(1)
				},
				new_switch_pair_info.clone().remote_reserve_location.try_as().unwrap()
			));
		});
	// Works with remote non-fungible asset.
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build()
		.execute_with(|| {
			assert!(IsSwitchPairXcmFeeAsset::<MockRuntime, _>::contains(
				&MultiAsset {
					id: MultiAsset::try_from(new_switch_pair_info.clone().remote_xcm_fee)
						.unwrap()
						.id,
					fun: Fungibility::NonFungible(AssetInstance::Index(1))
				},
				new_switch_pair_info.remote_reserve_location.try_as().unwrap()
			));
		});
}

#[test]
fn true_with_stored_remote_location_latest() {
	let location = xcm::latest::MultiLocation {
		parents: 1,
		interior: xcm::latest::Junctions::X1(xcm::latest::Junction::Parachain(1_000)),
	};
	let new_switch_pair_info =
		get_switch_pair_info_for_remote_location::<MockRuntime>(&location, SwitchPairStatus::Running);
	// Works with remote fungible asset.
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build()
		.execute_with(|| {
			assert!(IsSwitchPairXcmFeeAsset::<MockRuntime, _>::contains(
				&MultiAsset {
					id: MultiAsset::try_from(new_switch_pair_info.clone().remote_xcm_fee)
						.unwrap()
						.id,
					fun: Fungibility::Fungible(1)
				},
				new_switch_pair_info.clone().remote_reserve_location.try_as().unwrap()
			));
		});
	// Works with remote non-fungible asset.
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build()
		.execute_with(|| {
			assert!(IsSwitchPairXcmFeeAsset::<MockRuntime, _>::contains(
				&MultiAsset {
					id: MultiAsset::try_from(new_switch_pair_info.clone().remote_xcm_fee)
						.unwrap()
						.id,
					fun: Fungibility::NonFungible(AssetInstance::Index(1))
				},
				new_switch_pair_info.remote_reserve_location.try_as().unwrap()
			));
		});
}

#[test]
fn true_with_stored_remote_location_v3() {
	let location = MultiLocation {
		parents: 1,
		interior: Junctions::X1(Junction::Parachain(1_000)),
	};
	let new_switch_pair_info =
		get_switch_pair_info_for_remote_location::<MockRuntime>(&location, SwitchPairStatus::Running);
	// Works with remote fungible asset.
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build()
		.execute_with(|| {
			assert!(IsSwitchPairXcmFeeAsset::<MockRuntime, _>::contains(
				&MultiAsset {
					id: MultiAsset::try_from(new_switch_pair_info.clone().remote_xcm_fee)
						.unwrap()
						.id,
					fun: Fungibility::Fungible(1)
				},
				new_switch_pair_info.clone().remote_reserve_location.try_as().unwrap()
			));
		});
	// Works with remote non-fungible asset.
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build()
		.execute_with(|| {
			assert!(IsSwitchPairXcmFeeAsset::<MockRuntime, _>::contains(
				&MultiAsset {
					id: MultiAsset::try_from(new_switch_pair_info.clone().remote_xcm_fee)
						.unwrap()
						.id,
					fun: Fungibility::NonFungible(AssetInstance::Index(1))
				},
				new_switch_pair_info.remote_reserve_location.try_as().unwrap()
			));
		});
}

#[test]
fn true_with_stored_remote_location_v2() {
	let location = xcm::v2::MultiLocation {
		parents: 1,
		interior: xcm::v2::Junctions::X1(xcm::v2::Junction::Parachain(1_000)),
	};
	let new_switch_pair_info = {
		let mut new_switch_pair_info = get_switch_pair_info_for_remote_location::<MockRuntime>(
			&location.try_into().unwrap(),
			SwitchPairStatus::Running,
		);
		// Set remote location to the XCM v2.
		new_switch_pair_info.remote_reserve_location =
			new_switch_pair_info.remote_reserve_location.into_version(2).unwrap();
		new_switch_pair_info
	};
	// Works with remote fungible asset.
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build()
		.execute_with(|| {
			assert!(IsSwitchPairXcmFeeAsset::<MockRuntime, _>::contains(
				&MultiAsset {
					id: MultiAsset::try_from(new_switch_pair_info.clone().remote_xcm_fee)
						.unwrap()
						.id,
					fun: Fungibility::Fungible(1)
				},
				&new_switch_pair_info
					.clone()
					.remote_reserve_location
					.into_version(3)
					.unwrap()
					.try_into()
					.unwrap()
			));
		});
	// Works with remote non-fungible asset.
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build()
		.execute_with(|| {
			assert!(IsSwitchPairXcmFeeAsset::<MockRuntime, _>::contains(
				&MultiAsset {
					id: MultiAsset::try_from(new_switch_pair_info.clone().remote_xcm_fee)
						.unwrap()
						.id,
					fun: Fungibility::NonFungible(AssetInstance::Index(1))
				},
				&new_switch_pair_info
					.clone()
					.remote_reserve_location
					.into_version(3)
					.unwrap()
					.try_into()
					.unwrap()
			));
		});
}

#[test]
fn false_on_switch_pair_not_set() {
	ExtBuilder::default().build().execute_with(|| {
		assert!(!IsSwitchPairXcmFeeAsset::<MockRuntime, _>::contains(
			&MultiAsset {
				id: AssetId::Concrete(MultiLocation {
					parents: 1,
					interior: Junctions::X1(Junction::Parachain(1_000))
				}),
				fun: Fungibility::NonFungible(AssetInstance::Index(1))
			},
			&MultiLocation {
				parents: 1,
				interior: Junctions::X1(Junction::Parachain(1_000))
			}
		));
	});
}

#[test]
fn false_on_different_remote_location() {
	let location = MultiLocation {
		parents: 1,
		interior: Junctions::X1(Junction::Parachain(1_000)),
	};
	let new_switch_pair_info =
		get_switch_pair_info_for_remote_location::<MockRuntime>(&location, SwitchPairStatus::Running);
	// Fails with remote fungible asset.
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build()
		.execute_with(|| {
			assert!(!IsSwitchPairXcmFeeAsset::<MockRuntime, _>::contains(
				&MultiAsset {
					id: MultiAsset::try_from(new_switch_pair_info.clone().remote_xcm_fee)
						.unwrap()
						.id,
					fun: Fungibility::Fungible(1)
				},
				&MultiLocation {
					parents: 1,
					interior: Junctions::X2(Junction::Parachain(1_000), Junction::PalletInstance(1))
				},
			));
		});
	// Fails with remote non-fungible asset.
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build()
		.execute_with(|| {
			assert!(!IsSwitchPairXcmFeeAsset::<MockRuntime, _>::contains(
				&MultiAsset {
					id: MultiAsset::try_from(new_switch_pair_info.clone().remote_xcm_fee)
						.unwrap()
						.id,
					fun: Fungibility::NonFungible(AssetInstance::Index(1))
				},
				// Use a different location that does not match the stored one.
				&MultiLocation {
					parents: 1,
					interior: Junctions::X2(Junction::Parachain(1_000), Junction::PalletInstance(1))
				},
			));
		});
}

#[test]
fn false_on_nested_remote_location() {
	let location = MultiLocation {
		parents: 1,
		interior: Junctions::X1(Junction::Parachain(1_000)),
	};
	let new_switch_pair_info =
		get_switch_pair_info_for_remote_location::<MockRuntime>(&location, SwitchPairStatus::Running);
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build()
		.execute_with(|| {
			assert!(!IsSwitchPairXcmFeeAsset::<MockRuntime, _>::contains(
				new_switch_pair_info.clone().remote_xcm_fee.try_as().unwrap(),
				&MultiLocation {
					parents: 1,
					interior: Junctions::X2(
						Junction::Parachain(1_000),
						Junction::AccountId32 {
							network: None,
							id: [0; 32]
						}
					)
				}
			));
		});
}

#[test]
fn false_on_parent_remote_location() {
	let location = MultiLocation {
		parents: 1,
		interior: Junctions::X2(
			Junction::Parachain(1_000),
			Junction::AccountId32 {
				network: None,
				id: [0; 32],
			},
		),
	};
	let new_switch_pair_info =
		get_switch_pair_info_for_remote_location::<MockRuntime>(&location, SwitchPairStatus::Running);
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build()
		.execute_with(|| {
			assert!(!IsSwitchPairXcmFeeAsset::<MockRuntime, _>::contains(
				new_switch_pair_info.clone().remote_xcm_fee.try_as().unwrap(),
				&MultiLocation {
					parents: 1,
					interior: Junctions::X1(Junction::Parachain(1_000),)
				}
			));
		});
}

#[test]
fn false_on_different_xcm_fee_asset_id() {
	let location = MultiLocation {
		parents: 1,
		interior: Junctions::X1(Junction::Parachain(1_000)),
	};
	let new_switch_pair_info =
		get_switch_pair_info_for_remote_location::<MockRuntime>(&location, SwitchPairStatus::Running);
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build()
		.execute_with(|| {
			assert!(!IsSwitchPairXcmFeeAsset::<MockRuntime, _>::contains(
				&MultiAsset {
					id: AssetId::Abstract([0; 32]),
					fun: Fungibility::Fungible(1)
				},
				new_switch_pair_info.remote_reserve_location.try_as().unwrap()
			));
		});
}

#[test]
fn false_on_nested_xcm_fee_asset_id() {
	let location = MultiLocation {
		parents: 1,
		interior: Junctions::X1(Junction::Parachain(1_000)),
	};
	let new_switch_pair_info =
		get_switch_pair_info_for_remote_location::<MockRuntime>(&location, SwitchPairStatus::Running);
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build()
		.execute_with(|| {
			assert!(!IsSwitchPairXcmFeeAsset::<MockRuntime, _>::contains(
				&MultiAsset {
					// Nested location inside configured remote location
					id: AssetId::Concrete(MultiLocation {
						parents: 1,
						interior: Junctions::X2(
							Junction::Parachain(1_000),
							Junction::AccountId32 {
								network: None,
								id: [0; 32]
							}
						),
					}),
					fun: Fungibility::Fungible(1)
				},
				new_switch_pair_info.remote_reserve_location.try_as().unwrap()
			));
		});
}

#[test]
fn false_on_parent_xcm_fee_asset_id() {
	let location = MultiLocation {
		parents: 1,
		interior: Junctions::X2(
			Junction::Parachain(1_000),
			Junction::AccountId32 {
				network: None,
				id: [0; 32],
			},
		),
	};
	let new_switch_pair_info =
		get_switch_pair_info_for_remote_location::<MockRuntime>(&location, SwitchPairStatus::Running);
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build()
		.execute_with(|| {
			assert!(!IsSwitchPairXcmFeeAsset::<MockRuntime, _>::contains(
				&MultiAsset {
					// Parent location of configured remote location.
					id: AssetId::Concrete(MultiLocation {
						parents: 1,
						interior: Junctions::X1(Junction::Parachain(1_000),),
					}),
					fun: Fungibility::Fungible(1)
				},
				new_switch_pair_info.remote_reserve_location.try_as().unwrap()
			));
		});
}
