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

use frame_support::{assert_noop, assert_storage_noop};
use xcm::{
	v4::{Asset, AssetId, AssetInstance, Error, Fungibility, Junction, Junctions, Location, Weight, XcmContext},
	IntoVersion,
};
use xcm_executor::{traits::WeightTrader, AssetsInHolding};

use crate::{
	xcm::{
		test_utils::get_switch_pair_info_for_remote_location_with_pool_usable_balance,
		trade::{
			switch_pair_remote_asset::mock::{ExtBuilder, MockRuntime, ToDestinationAccount},
			test_utils::SumTimeAndProofValues,
		},
		UsingComponentsForSwitchPairRemoteAsset,
	},
	SwitchPairStatus,
};

#[test]
fn successful_on_stored_remote_asset_latest_with_input_fungible() {
	let location = xcm::latest::Location {
		parents: 1,
		interior: xcm::latest::Junctions::X1([xcm::latest::Junction::Parachain(1_000)].into()),
	};
	let new_switch_pair_info =
		{
			// Give to pool amount same amount that is being purchased in the test case +
			// ED.
			let mut new_switch_pair_info = get_switch_pair_info_for_remote_location_with_pool_usable_balance::<
				MockRuntime,
			>(&location, 2, SwitchPairStatus::Running);
			// Set remote asset to the latest XCM version.
			new_switch_pair_info.remote_asset_id = new_switch_pair_info.remote_asset_id.into_latest().unwrap();
			new_switch_pair_info
		};
	// Results in a required amount of `2` local currency tokens.
	let weight_to_buy = Weight::from_parts(1, 1);
	let xcm_context = XcmContext::with_message_id([0u8; 32]);
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build_and_execute_with_sanity_tests(|| {
			let mut weigher = UsingComponentsForSwitchPairRemoteAsset::<
				MockRuntime,
				_,
				SumTimeAndProofValues,
				ToDestinationAccount,
			>::new();
			let payment: AssetsInHolding = vec![Asset {
				id: new_switch_pair_info.clone().remote_asset_id.try_into().unwrap(),
				fun: Fungibility::Fungible(2),
			}]
			.into();
			let unused_weight = weigher.buy_weight(weight_to_buy, payment, &xcm_context).unwrap();
			assert!(unused_weight.is_empty());
			assert_eq!(weigher.consumed_xcm_hash, Some(xcm_context.message_id));
			assert_eq!(weigher.remaining_fungible_balance, 2);
			assert_eq!(weigher.remaining_weight, weight_to_buy);
		});
}

#[test]
fn successful_on_stored_remote_asset_latest_with_input_non_fungible() {
	let location = xcm::latest::Location {
		parents: 1,
		interior: xcm::latest::Junctions::X1([xcm::latest::Junction::Parachain(1_000)].into()),
	};
	let new_switch_pair_info =
		{
			// Give to pool amount same amount that is being purchased in the test case +
			// ED.
			let mut new_switch_pair_info = get_switch_pair_info_for_remote_location_with_pool_usable_balance::<
				MockRuntime,
			>(&location, 2, SwitchPairStatus::Running);
			// Set remote asset to the latest XCM version.
			new_switch_pair_info.remote_asset_id = new_switch_pair_info.remote_asset_id.into_latest().unwrap();
			new_switch_pair_info
		};
	// Results in a required amount of `2` local currency tokens.
	let weight_to_buy = Weight::from_parts(1, 1);
	let xcm_context = XcmContext::with_message_id([0u8; 32]);
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build_and_execute_with_sanity_tests(|| {
			let mut weigher = UsingComponentsForSwitchPairRemoteAsset::<
				MockRuntime,
				_,
				SumTimeAndProofValues,
				ToDestinationAccount,
			>::new();
			let payment: AssetsInHolding = vec![Asset {
				id: new_switch_pair_info.clone().remote_asset_id.try_into().unwrap(),
				fun: Fungibility::NonFungible(AssetInstance::Index(1)),
			}]
			.into();

			assert_noop!(
				weigher.buy_weight(weight_to_buy, payment, &xcm_context),
				Error::TooExpensive
			);
			assert_storage_noop!(drop(weigher));
		});
}

#[test]
fn successful_on_stored_remote_asset_latest_with_input_fungible_and_non_fungible() {
	let location = xcm::latest::Location {
		parents: 1,
		interior: xcm::latest::Junctions::X1([xcm::latest::Junction::Parachain(1_000)].into()),
	};
	let new_switch_pair_info =
		{
			// Give to pool amount same amount that is being purchased in the test case +
			// ED.
			let mut new_switch_pair_info = get_switch_pair_info_for_remote_location_with_pool_usable_balance::<
				MockRuntime,
			>(&location, 2, SwitchPairStatus::Running);
			// Set remote asset to the latest XCM version.
			new_switch_pair_info.remote_asset_id = new_switch_pair_info.remote_asset_id.into_latest().unwrap();
			new_switch_pair_info
		};
	// Results in a required amount of `2` local currency tokens.
	let weight_to_buy = Weight::from_parts(1, 1);
	let xcm_context = XcmContext::with_message_id([0u8; 32]);
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build_and_execute_with_sanity_tests(|| {
			let mut weigher = UsingComponentsForSwitchPairRemoteAsset::<
				MockRuntime,
				_,
				SumTimeAndProofValues,
				ToDestinationAccount,
			>::new();
			let payment: AssetsInHolding = vec![
				Asset {
					id: new_switch_pair_info.clone().remote_asset_id.try_into().unwrap(),
					fun: Fungibility::Fungible(2),
				},
				Asset {
					id: new_switch_pair_info.clone().remote_asset_id.try_into().unwrap(),
					fun: Fungibility::NonFungible(AssetInstance::Index(1)),
				},
			]
			.into();

			let unused_weight = weigher.buy_weight(weight_to_buy, payment, &xcm_context).unwrap();
			// The non-fungible asset is left in the registry.
			assert_eq!(
				unused_weight,
				vec![Asset {
					id: new_switch_pair_info.clone().remote_asset_id.try_into().unwrap(),
					fun: Fungibility::NonFungible(AssetInstance::Index(1)),
				},]
				.into()
			);
			assert_eq!(weigher.consumed_xcm_hash, Some(xcm_context.message_id));
			assert_eq!(weigher.remaining_fungible_balance, 2);
			assert_eq!(weigher.remaining_weight, weight_to_buy);
		});
}

#[test]
fn successful_on_stored_remote_asset_v4_with_input_fungible() {
	let location = Location {
		parents: 1,
		interior: Junctions::X1([Junction::Parachain(1_000)].into()),
	};
	// Give to pool amount same amount that is being purchased in the test case +
	// ED.
	let new_switch_pair_info = get_switch_pair_info_for_remote_location_with_pool_usable_balance::<MockRuntime>(
		&location,
		3,
		SwitchPairStatus::Running,
	);
	// Results in a required amount of `2` local currency tokens.
	let weight_to_buy = Weight::from_parts(1, 1);
	let xcm_context = XcmContext::with_message_id([0u8; 32]);
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build_and_execute_with_sanity_tests(|| {
			let mut weigher = UsingComponentsForSwitchPairRemoteAsset::<
				MockRuntime,
				_,
				SumTimeAndProofValues,
				ToDestinationAccount,
			>::new();
			let payment: AssetsInHolding = vec![Asset {
				id: new_switch_pair_info.clone().remote_asset_id.try_into().unwrap(),
				fun: Fungibility::Fungible(2),
			}]
			.into();
			let unused_weight = weigher.buy_weight(weight_to_buy, payment, &xcm_context).unwrap();
			assert!(unused_weight.is_empty());
			assert_eq!(weigher.consumed_xcm_hash, Some(xcm_context.message_id));
			assert_eq!(weigher.remaining_fungible_balance, 2);
			assert_eq!(weigher.remaining_weight, weight_to_buy);
		});
}

#[test]
fn successful_on_stored_remote_asset_v4_with_input_non_fungible() {
	let location = Location {
		parents: 1,
		interior: Junctions::X1([Junction::Parachain(1_000)].into()),
	};
	// Give to pool amount same amount that is being purchased in the test case +
	// ED.
	let new_switch_pair_info = get_switch_pair_info_for_remote_location_with_pool_usable_balance::<MockRuntime>(
		&location,
		3,
		SwitchPairStatus::Running,
	);
	// Results in a required amount of `2` local currency tokens.
	let weight_to_buy = Weight::from_parts(1, 1);
	let xcm_context = XcmContext::with_message_id([0u8; 32]);
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build_and_execute_with_sanity_tests(|| {
			let mut weigher = UsingComponentsForSwitchPairRemoteAsset::<
				MockRuntime,
				_,
				SumTimeAndProofValues,
				ToDestinationAccount,
			>::new();
			let payment: AssetsInHolding = vec![Asset {
				id: new_switch_pair_info.clone().remote_asset_id.try_into().unwrap(),
				fun: Fungibility::NonFungible(AssetInstance::Index(1)),
			}]
			.into();

			assert_noop!(
				weigher.buy_weight(weight_to_buy, payment, &xcm_context),
				Error::TooExpensive
			);
			assert_storage_noop!(drop(weigher));
		});
}

#[test]
fn successful_on_stored_remote_asset_v4_with_input_fungible_and_non_fungible() {
	let location = Location {
		parents: 1,
		interior: Junctions::X1([Junction::Parachain(1_000)].into()),
	};
	// Give to pool amount same amount that is being purchased in the test case +
	// ED.
	let new_switch_pair_info = get_switch_pair_info_for_remote_location_with_pool_usable_balance::<MockRuntime>(
		&location,
		3,
		SwitchPairStatus::Running,
	);
	// Results in a required amount of `2` local currency tokens.
	let weight_to_buy = Weight::from_parts(1, 1);
	let xcm_context = XcmContext::with_message_id([0u8; 32]);
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build_and_execute_with_sanity_tests(|| {
			let mut weigher = UsingComponentsForSwitchPairRemoteAsset::<
				MockRuntime,
				_,
				SumTimeAndProofValues,
				ToDestinationAccount,
			>::new();
			let payment: AssetsInHolding = vec![
				Asset {
					id: new_switch_pair_info.clone().remote_asset_id.try_into().unwrap(),
					fun: Fungibility::Fungible(2),
				},
				Asset {
					id: new_switch_pair_info.clone().remote_asset_id.try_into().unwrap(),
					fun: Fungibility::NonFungible(AssetInstance::Index(1)),
				},
			]
			.into();

			let unused_weight = weigher.buy_weight(weight_to_buy, payment, &xcm_context).unwrap();
			// The non-fungible asset is left in the registry.
			assert_eq!(
				unused_weight,
				vec![Asset {
					id: new_switch_pair_info.clone().remote_asset_id.try_into().unwrap(),
					fun: Fungibility::NonFungible(AssetInstance::Index(1)),
				},]
				.into()
			);
			assert_eq!(weigher.consumed_xcm_hash, Some(xcm_context.message_id));
			assert_eq!(weigher.remaining_fungible_balance, 2);
			assert_eq!(weigher.remaining_weight, weight_to_buy);
		});
}

#[test]
fn successful_on_stored_remote_asset_v3_with_input_fungible() {
	let location = xcm::v3::MultiLocation {
		parents: 1,
		interior: xcm::v3::Junctions::X1(xcm::v3::Junction::Parachain(1_000)),
	};
	// Give to pool amount same amount that is being purchased in the test case +
	// ED.
	let new_switch_pair_info = get_switch_pair_info_for_remote_location_with_pool_usable_balance::<MockRuntime>(
		&location.try_into().unwrap(),
		3,
		SwitchPairStatus::Running,
	);
	// Results in a required amount of `2` local currency tokens.
	let weight_to_buy = Weight::from_parts(1, 1);
	let xcm_context = XcmContext::with_message_id([0u8; 32]);
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build_and_execute_with_sanity_tests(|| {
			let mut weigher = UsingComponentsForSwitchPairRemoteAsset::<
				MockRuntime,
				_,
				SumTimeAndProofValues,
				ToDestinationAccount,
			>::new();
			let payment: AssetsInHolding = vec![Asset {
				id: new_switch_pair_info.clone().remote_asset_id.try_into().unwrap(),
				fun: Fungibility::Fungible(2),
			}]
			.into();
			let unused_weight = weigher.buy_weight(weight_to_buy, payment, &xcm_context).unwrap();
			assert!(unused_weight.is_empty());
			assert_eq!(weigher.consumed_xcm_hash, Some(xcm_context.message_id));
			assert_eq!(weigher.remaining_fungible_balance, 2);
			assert_eq!(weigher.remaining_weight, weight_to_buy);
		});
}

#[test]
fn successful_on_stored_remote_asset_v3_with_input_non_fungible() {
	let location = xcm::v3::MultiLocation {
		parents: 1,
		interior: xcm::v3::Junctions::X1(xcm::v3::Junction::Parachain(1_000)),
	};
	// Give to pool amount same amount that is being purchased in the test case +
	// ED.
	let new_switch_pair_info = get_switch_pair_info_for_remote_location_with_pool_usable_balance::<MockRuntime>(
		&location.try_into().unwrap(),
		3,
		SwitchPairStatus::Running,
	);
	// Results in a required amount of `2` local currency tokens.
	let weight_to_buy = Weight::from_parts(1, 1);
	let xcm_context = XcmContext::with_message_id([0u8; 32]);
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build_and_execute_with_sanity_tests(|| {
			let mut weigher = UsingComponentsForSwitchPairRemoteAsset::<
				MockRuntime,
				_,
				SumTimeAndProofValues,
				ToDestinationAccount,
			>::new();
			let payment: AssetsInHolding = vec![Asset {
				id: new_switch_pair_info.clone().remote_asset_id.try_into().unwrap(),
				fun: Fungibility::NonFungible(AssetInstance::Index(1)),
			}]
			.into();

			assert_noop!(
				weigher.buy_weight(weight_to_buy, payment, &xcm_context),
				Error::TooExpensive
			);
			assert_storage_noop!(drop(weigher));
		});
}

#[test]
fn successful_on_stored_remote_asset_v3_with_input_fungible_and_non_fungible() {
	let location = xcm::v3::MultiLocation {
		parents: 1,
		interior: xcm::v3::Junctions::X1(xcm::v3::Junction::Parachain(1_000)),
	};
	// Give to pool amount same amount that is being purchased in the test case +
	// ED.
	let new_switch_pair_info = get_switch_pair_info_for_remote_location_with_pool_usable_balance::<MockRuntime>(
		&location.try_into().unwrap(),
		3,
		SwitchPairStatus::Running,
	);
	// Results in a required amount of `2` local currency tokens.
	let weight_to_buy = Weight::from_parts(1, 1);
	let xcm_context = XcmContext::with_message_id([0u8; 32]);
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build_and_execute_with_sanity_tests(|| {
			let mut weigher = UsingComponentsForSwitchPairRemoteAsset::<
				MockRuntime,
				_,
				SumTimeAndProofValues,
				ToDestinationAccount,
			>::new();
			let payment: AssetsInHolding = vec![
				Asset {
					id: new_switch_pair_info.clone().remote_asset_id.try_into().unwrap(),
					fun: Fungibility::Fungible(2),
				},
				Asset {
					id: new_switch_pair_info.clone().remote_asset_id.try_into().unwrap(),
					fun: Fungibility::NonFungible(AssetInstance::Index(1)),
				},
			]
			.into();

			let unused_weight = weigher.buy_weight(weight_to_buy, payment, &xcm_context).unwrap();
			// The non-fungible asset is left in the registry.
			assert_eq!(
				unused_weight,
				vec![Asset {
					id: new_switch_pair_info.clone().remote_asset_id.try_into().unwrap(),
					fun: Fungibility::NonFungible(AssetInstance::Index(1)),
				},]
				.into()
			);
			assert_eq!(weigher.consumed_xcm_hash, Some(xcm_context.message_id));
			assert_eq!(weigher.remaining_fungible_balance, 2);
			assert_eq!(weigher.remaining_weight, weight_to_buy);
		});
}

#[test]
fn fails_on_rerun() {
	let location = Location {
		parents: 1,
		interior: Junctions::X1([Junction::Parachain(1_000)].into()),
	};
	let new_switch_pair_info = get_switch_pair_info_for_remote_location_with_pool_usable_balance::<MockRuntime>(
		&location,
		0,
		SwitchPairStatus::Running,
	);
	// Results in a required amount of `2` local currency tokens.
	let weight_to_buy = Weight::from_parts(1, 1);
	let xcm_context = XcmContext::with_message_id([0u8; 32]);
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build_and_execute_with_sanity_tests(|| {
			let mut weigher = {
				let mut weigher = UsingComponentsForSwitchPairRemoteAsset::<
					MockRuntime,
					_,
					SumTimeAndProofValues,
					ToDestinationAccount,
				>::new();
				weigher.consumed_xcm_hash = Some([0; 32]);
				weigher
			};
			let payment: AssetsInHolding = vec![Asset {
				id: new_switch_pair_info.clone().remote_asset_id.try_into().unwrap(),
				fun: Fungibility::Fungible(2),
			}]
			.into();
			assert_noop!(
				weigher.buy_weight(weight_to_buy, payment, &xcm_context),
				Error::NotWithdrawable
			);
			assert_storage_noop!(drop(weigher));
		});
}

#[test]
fn skips_on_switch_pair_not_set() {
	let weight_to_buy = Weight::from_parts(1, 1);
	let xcm_context = XcmContext::with_message_id([0u8; 32]);
	ExtBuilder::default().build_and_execute_with_sanity_tests(|| {
		let mut weigher = UsingComponentsForSwitchPairRemoteAsset::<
			MockRuntime,
			_,
			SumTimeAndProofValues,
			ToDestinationAccount,
		>::new();
		let payment: AssetsInHolding = vec![Asset {
			id: AssetId(Location::here()),
			fun: Fungibility::Fungible(1),
		}]
		.into();
		assert_noop!(
			weigher.buy_weight(weight_to_buy, payment, &xcm_context),
			Error::AssetNotFound
		);
		assert_storage_noop!(drop(weigher));
	});
}

#[test]
fn skips_on_switch_pair_not_enabled() {
	let location = Location {
		parents: 1,
		interior: Junctions::X1([Junction::Parachain(1_000)].into()),
	};
	let new_switch_pair_info = get_switch_pair_info_for_remote_location_with_pool_usable_balance::<MockRuntime>(
		&location,
		1,
		SwitchPairStatus::Paused,
	);
	let weight_to_buy = Weight::from_parts(1, 1);
	let xcm_context = XcmContext::with_message_id([0u8; 32]);
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build_and_execute_with_sanity_tests(|| {
			let mut weigher = UsingComponentsForSwitchPairRemoteAsset::<
				MockRuntime,
				_,
				SumTimeAndProofValues,
				ToDestinationAccount,
			>::new();
			let payment: AssetsInHolding = vec![Asset {
				id: new_switch_pair_info.clone().remote_asset_id.try_into().unwrap(),
				fun: Fungibility::Fungible(1),
			}]
			.into();
			assert_noop!(
				weigher.buy_weight(weight_to_buy, payment, &xcm_context),
				Error::AssetNotFound
			);
			assert_storage_noop!(drop(weigher));
		});
}

#[test]
fn fails_on_too_expensive() {
	let location = Location {
		parents: 1,
		interior: Junctions::X1([Junction::Parachain(1_000)].into()),
	};
	let new_switch_pair_info = get_switch_pair_info_for_remote_location_with_pool_usable_balance::<MockRuntime>(
		&location,
		0,
		SwitchPairStatus::Running,
	);
	// Results in a required amount of `2` local currency tokens.
	let weight_to_buy = Weight::from_parts(1, 1);
	let xcm_context = XcmContext::with_message_id([0u8; 32]);
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build_and_execute_with_sanity_tests(|| {
			let mut weigher = UsingComponentsForSwitchPairRemoteAsset::<
				MockRuntime,
				_,
				SumTimeAndProofValues,
				ToDestinationAccount,
			>::new();
			let payment: AssetsInHolding = vec![Asset {
				id: new_switch_pair_info.clone().remote_asset_id.try_into().unwrap(),
				// Using only `1` asset is not sufficient.
				fun: Fungibility::Fungible(1),
			}]
			.into();
			assert_noop!(
				weigher.buy_weight(weight_to_buy, payment, &xcm_context),
				Error::TooExpensive
			);
			assert_storage_noop!(drop(weigher));
		});
}
