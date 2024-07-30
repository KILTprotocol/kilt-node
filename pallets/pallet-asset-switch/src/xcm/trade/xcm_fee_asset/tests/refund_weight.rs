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

use frame_support::assert_storage_noop;
use sp_runtime::traits::Zero;
use xcm::{
	v4::{Asset, AssetInstance, Fungibility, Junction, Junctions, Location, Weight, XcmContext},
	IntoVersion,
};
use xcm_executor::traits::WeightTrader;

use crate::{
	xcm::{
		test_utils::get_switch_pair_info_for_remote_location,
		trade::{
			test_utils::SumTimeAndProofValues,
			xcm_fee_asset::mock::{ExtBuilder, MockRuntime},
		},
		UsingComponentsForXcmFeeAsset,
	},
	SwitchPairStatus,
};

#[test]
fn successful_on_stored_fungible_xcm_fee_asset_latest_with_remaining_balance_and_weight() {
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
	// Results in an amount of `2` local currency tokens.
	let weight_to_refund = Weight::from_parts(1, 1);
	let xcm_context = XcmContext::with_message_id([0u8; 32]);
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build()
		.execute_with(|| {
			let mut weigher = {
				let mut weigher = UsingComponentsForXcmFeeAsset::<MockRuntime, _, SumTimeAndProofValues>::new();
				weigher.remaining_fungible_balance = u128::MAX;
				weigher.remaining_weight = Weight::MAX;
				weigher.consumed_xcm_hash = Some([0u8; 32]);
				weigher
			};
			let amount_refunded = weigher.refund_weight(weight_to_refund, &xcm_context);
			assert_eq!(
				amount_refunded,
				Some(Asset {
					id: Asset::try_from(new_switch_pair_info.clone().remote_xcm_fee).unwrap().id,
					fun: Fungibility::Fungible(2)
				})
			);
			assert_eq!(weigher.remaining_fungible_balance, u128::MAX - 2);
			assert_eq!(weigher.remaining_weight, Weight::MAX - weight_to_refund);
			assert!(weigher.consumed_xcm_hash.is_none());
		});
}

#[test]
fn successful_on_stored_fungible_xcm_fee_asset_latest_with_zero_remaining_balance() {
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
	// Results in an amount of `2` local currency tokens.
	let weight_to_refund = Weight::from_parts(1, 1);
	let xcm_context = XcmContext::with_message_id([0u8; 32]);
	// No balance is refunded, weight is.
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info)
		.build()
		.execute_with(|| {
			let mut weigher = {
				let mut weigher = UsingComponentsForXcmFeeAsset::<MockRuntime, _, SumTimeAndProofValues>::new();
				weigher.remaining_fungible_balance = u128::zero();
				weigher.remaining_weight = Weight::MAX;
				weigher.consumed_xcm_hash = Some([0u8; 32]);
				weigher
			};
			let amount_refunded = weigher.refund_weight(weight_to_refund, &xcm_context);
			assert_eq!(amount_refunded, None);
			assert!(weigher.remaining_fungible_balance.is_zero());
			assert_eq!(weigher.remaining_weight, Weight::MAX - weight_to_refund);
			assert!(weigher.consumed_xcm_hash.is_none());
		});
}

#[test]
fn successful_on_stored_fungible_xcm_fee_asset_latest_with_zero_remaining_weight() {
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
	// Results in an amount of `2` local currency tokens.
	let weight_to_refund = Weight::from_parts(1, 1);
	let xcm_context = XcmContext::with_message_id([0u8; 32]);
	// Nothing is refunded, remaining balance is not changed.
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info)
		.build()
		.execute_with(|| {
			let mut weigher = {
				let mut weigher = UsingComponentsForXcmFeeAsset::<MockRuntime, _, SumTimeAndProofValues>::new();
				weigher.remaining_fungible_balance = u128::MAX;
				weigher.remaining_weight = Weight::zero();
				weigher.consumed_xcm_hash = Some([0u8; 32]);
				weigher
			};
			let amount_refunded = weigher.refund_weight(weight_to_refund, &xcm_context);
			assert_eq!(amount_refunded, None);
			assert_eq!(weigher.remaining_fungible_balance, u128::MAX);
			assert!(weigher.remaining_weight.is_zero());
			assert!(weigher.consumed_xcm_hash.is_none());
		});
}

#[test]
fn successful_on_stored_fungible_xcm_fee_asset_latest_with_zero_remaining_balance_and_weight() {
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
	// Results in an amount of `2` local currency tokens.
	let weight_to_refund = Weight::from_parts(1, 1);
	let xcm_context = XcmContext::with_message_id([0u8; 32]);
	// Nothing is refunded.
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info)
		.build()
		.execute_with(|| {
			let mut weigher = {
				let mut weigher = UsingComponentsForXcmFeeAsset::<MockRuntime, _, SumTimeAndProofValues>::new();
				weigher.remaining_fungible_balance = u128::zero();
				weigher.remaining_weight = Weight::zero();
				weigher.consumed_xcm_hash = Some([0u8; 32]);
				weigher
			};
			let amount_refunded = weigher.refund_weight(weight_to_refund, &xcm_context);
			assert_eq!(amount_refunded, None);
			assert!(weigher.remaining_fungible_balance.is_zero());
			assert!(weigher.remaining_weight.is_zero());
			assert!(weigher.consumed_xcm_hash.is_none());
		});
}

#[test]
fn successful_on_stored_fungible_xcm_fee_asset_v4_with_remaining_balance_and_weight() {
	let location = Location {
		parents: 1,
		interior: Junctions::X1([Junction::Parachain(1_000)].into()),
	};
	let new_switch_pair_info = {
		let mut new_switch_pair_info =
			get_switch_pair_info_for_remote_location::<MockRuntime>(&location, SwitchPairStatus::Running);
		// Set XCM fee asset to the XCM version 3.
		new_switch_pair_info.remote_xcm_fee = new_switch_pair_info.remote_xcm_fee.into_version(3).unwrap();
		new_switch_pair_info
	};
	// Results in an amount of `2` local currency tokens.
	let weight_to_refund = Weight::from_parts(1, 1);
	let xcm_context = XcmContext::with_message_id([0u8; 32]);
	// Case when remaining balance and weight are both higher than refund.
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build()
		.execute_with(|| {
			let mut weigher = {
				let mut weigher = UsingComponentsForXcmFeeAsset::<MockRuntime, _, SumTimeAndProofValues>::new();
				weigher.remaining_fungible_balance = u128::MAX;
				weigher.remaining_weight = Weight::MAX;
				weigher.consumed_xcm_hash = Some([0u8; 32]);
				weigher
			};
			let amount_refunded = weigher.refund_weight(weight_to_refund, &xcm_context);
			assert_eq!(
				amount_refunded,
				Some(Asset {
					id: Asset::try_from(new_switch_pair_info.clone().remote_xcm_fee).unwrap().id,
					fun: Fungibility::Fungible(2)
				})
			);
			assert_eq!(weigher.remaining_fungible_balance, u128::MAX - 2);
			assert_eq!(weigher.remaining_weight, Weight::MAX - weight_to_refund);
			assert!(weigher.consumed_xcm_hash.is_none());
		});
}

#[test]
fn successful_on_stored_fungible_xcm_fee_asset_v4_with_zero_remaining_balance() {
	let location = Location {
		parents: 1,
		interior: Junctions::X1([Junction::Parachain(1_000)].into()),
	};
	let new_switch_pair_info = {
		let mut new_switch_pair_info =
			get_switch_pair_info_for_remote_location::<MockRuntime>(&location, SwitchPairStatus::Running);
		// Set XCM fee asset to the XCM version 3.
		new_switch_pair_info.remote_xcm_fee = new_switch_pair_info.remote_xcm_fee.into_version(3).unwrap();
		new_switch_pair_info
	};
	// Results in an amount of `2` local currency tokens.
	let weight_to_refund = Weight::from_parts(1, 1);
	let xcm_context = XcmContext::with_message_id([0u8; 32]);
	// No balance is refunded, weight is.
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info)
		.build()
		.execute_with(|| {
			let mut weigher = {
				let mut weigher = UsingComponentsForXcmFeeAsset::<MockRuntime, _, SumTimeAndProofValues>::new();
				weigher.remaining_fungible_balance = u128::zero();
				weigher.remaining_weight = Weight::MAX;
				weigher.consumed_xcm_hash = Some([0u8; 32]);
				weigher
			};
			let amount_refunded = weigher.refund_weight(weight_to_refund, &xcm_context);
			assert_eq!(amount_refunded, None);
			assert!(weigher.remaining_fungible_balance.is_zero());
			assert_eq!(weigher.remaining_weight, Weight::MAX - weight_to_refund);
			assert!(weigher.consumed_xcm_hash.is_none());
		});
}

#[test]
fn successful_on_stored_fungible_xcm_fee_asset_v4_with_zero_remaining_weight() {
	let location = Location {
		parents: 1,
		interior: Junctions::X1([Junction::Parachain(1_000)].into()),
	};
	let new_switch_pair_info = {
		let mut new_switch_pair_info =
			get_switch_pair_info_for_remote_location::<MockRuntime>(&location, SwitchPairStatus::Running);
		// Set XCM fee asset to the XCM version 3.
		new_switch_pair_info.remote_xcm_fee = new_switch_pair_info.remote_xcm_fee.into_version(3).unwrap();
		new_switch_pair_info
	};
	// Results in an amount of `2` local currency tokens.
	let weight_to_refund = Weight::from_parts(1, 1);
	let xcm_context = XcmContext::with_message_id([0u8; 32]);
	// Nothing is refunded, remaining balance is not changed.
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info)
		.build()
		.execute_with(|| {
			let mut weigher = {
				let mut weigher = UsingComponentsForXcmFeeAsset::<MockRuntime, _, SumTimeAndProofValues>::new();
				weigher.remaining_fungible_balance = u128::MAX;
				weigher.remaining_weight = Weight::zero();
				weigher.consumed_xcm_hash = Some([0u8; 32]);
				weigher
			};
			let amount_refunded = weigher.refund_weight(weight_to_refund, &xcm_context);
			assert_eq!(amount_refunded, None);
			assert_eq!(weigher.remaining_fungible_balance, u128::MAX);
			assert!(weigher.remaining_weight.is_zero());
			assert!(weigher.consumed_xcm_hash.is_none());
		});
}

#[test]
fn successful_on_stored_fungible_xcm_fee_asset_v4_with_zero_remaining_balance_and_weight() {
	let location = Location {
		parents: 1,
		interior: Junctions::X1([Junction::Parachain(1_000)].into()),
	};
	let new_switch_pair_info = {
		let mut new_switch_pair_info =
			get_switch_pair_info_for_remote_location::<MockRuntime>(&location, SwitchPairStatus::Running);
		// Set XCM fee asset to the XCM version 3.
		new_switch_pair_info.remote_xcm_fee = new_switch_pair_info.remote_xcm_fee.into_version(3).unwrap();
		new_switch_pair_info
	};
	// Results in an amount of `2` local currency tokens.
	let weight_to_refund = Weight::from_parts(1, 1);
	let xcm_context = XcmContext::with_message_id([0u8; 32]);
	// Nothing is refunded.
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info)
		.build()
		.execute_with(|| {
			let mut weigher = {
				let mut weigher = UsingComponentsForXcmFeeAsset::<MockRuntime, _, SumTimeAndProofValues>::new();
				weigher.remaining_fungible_balance = u128::zero();
				weigher.remaining_weight = Weight::zero();
				weigher.consumed_xcm_hash = Some([0u8; 32]);
				weigher
			};
			let amount_refunded = weigher.refund_weight(weight_to_refund, &xcm_context);
			assert_eq!(amount_refunded, None);
			assert!(weigher.remaining_fungible_balance.is_zero());
			assert!(weigher.remaining_weight.is_zero());
			assert!(weigher.consumed_xcm_hash.is_none());
		});
}

#[test]
fn successful_on_stored_fungible_xcm_fee_asset_v3_with_remaining_balance_and_weight() {
	let location = xcm::v3::MultiLocation {
		parents: 1,
		interior: xcm::v3::Junctions::X1(xcm::v3::Junction::Parachain(1_000)),
	};
	let new_switch_pair_info = {
		let mut new_switch_pair_info = get_switch_pair_info_for_remote_location::<MockRuntime>(
			&location.try_into().unwrap(),
			SwitchPairStatus::Running,
		);
		// Set XCM fee asset to the XCM version 3.
		new_switch_pair_info.remote_xcm_fee = new_switch_pair_info.remote_xcm_fee.into_version(3).unwrap();
		new_switch_pair_info
	};
	// Results in an amount of `2` local currency tokens.
	let weight_to_refund = Weight::from_parts(1, 1);
	let xcm_context = XcmContext::with_message_id([0u8; 32]);
	// Case when remaining balance and weight are both higher than refund.
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build()
		.execute_with(|| {
			let mut weigher = {
				let mut weigher = UsingComponentsForXcmFeeAsset::<MockRuntime, _, SumTimeAndProofValues>::new();
				weigher.remaining_fungible_balance = u128::MAX;
				weigher.remaining_weight = Weight::MAX;
				weigher.consumed_xcm_hash = Some([0u8; 32]);
				weigher
			};
			let amount_refunded = weigher.refund_weight(weight_to_refund, &xcm_context);
			assert_eq!(
				amount_refunded,
				Some(Asset {
					id: Asset::try_from(new_switch_pair_info.clone().remote_xcm_fee).unwrap().id,
					fun: Fungibility::Fungible(2)
				})
			);
			assert_eq!(weigher.remaining_fungible_balance, u128::MAX - 2);
			assert_eq!(weigher.remaining_weight, Weight::MAX - weight_to_refund);
			assert!(weigher.consumed_xcm_hash.is_none());
		});
}

#[test]
fn successful_on_stored_fungible_xcm_fee_asset_v3_with_zero_remaining_balance() {
	let location = xcm::v3::MultiLocation {
		parents: 1,
		interior: xcm::v3::Junctions::X1(xcm::v3::Junction::Parachain(1_000)),
	};
	let new_switch_pair_info = {
		let mut new_switch_pair_info = get_switch_pair_info_for_remote_location::<MockRuntime>(
			&location.try_into().unwrap(),
			SwitchPairStatus::Running,
		);
		// Set XCM fee asset to the XCM version 3.
		new_switch_pair_info.remote_xcm_fee = new_switch_pair_info.remote_xcm_fee.into_version(3).unwrap();
		new_switch_pair_info
	};
	// Results in an amount of `2` local currency tokens.
	let weight_to_refund = Weight::from_parts(1, 1);
	let xcm_context = XcmContext::with_message_id([0u8; 32]);
	// No balance is refunded, weight is.
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info)
		.build()
		.execute_with(|| {
			let mut weigher = {
				let mut weigher = UsingComponentsForXcmFeeAsset::<MockRuntime, _, SumTimeAndProofValues>::new();
				weigher.remaining_fungible_balance = u128::zero();
				weigher.remaining_weight = Weight::MAX;
				weigher.consumed_xcm_hash = Some([0u8; 32]);
				weigher
			};
			let amount_refunded = weigher.refund_weight(weight_to_refund, &xcm_context);
			assert_eq!(amount_refunded, None);
			assert!(weigher.remaining_fungible_balance.is_zero());
			assert_eq!(weigher.remaining_weight, Weight::MAX - weight_to_refund);
			assert!(weigher.consumed_xcm_hash.is_none());
		});
}

#[test]
fn successful_on_stored_fungible_xcm_fee_asset_v3_with_zero_remaining_weight() {
	let location = xcm::v3::MultiLocation {
		parents: 1,
		interior: xcm::v3::Junctions::X1(xcm::v3::Junction::Parachain(1_000)),
	};
	let new_switch_pair_info = {
		let mut new_switch_pair_info = get_switch_pair_info_for_remote_location::<MockRuntime>(
			&location.try_into().unwrap(),
			SwitchPairStatus::Running,
		);
		// Set XCM fee asset to the XCM version 3.
		new_switch_pair_info.remote_xcm_fee = new_switch_pair_info.remote_xcm_fee.into_version(3).unwrap();
		new_switch_pair_info
	};
	// Results in an amount of `2` local currency tokens.
	let weight_to_refund = Weight::from_parts(1, 1);
	let xcm_context = XcmContext::with_message_id([0u8; 32]);
	// Nothing is refunded, remaining balance is not changed.
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info)
		.build()
		.execute_with(|| {
			let mut weigher = {
				let mut weigher = UsingComponentsForXcmFeeAsset::<MockRuntime, _, SumTimeAndProofValues>::new();
				weigher.remaining_fungible_balance = u128::MAX;
				weigher.remaining_weight = Weight::zero();
				weigher.consumed_xcm_hash = Some([0u8; 32]);
				weigher
			};
			let amount_refunded = weigher.refund_weight(weight_to_refund, &xcm_context);
			assert_eq!(amount_refunded, None);
			assert_eq!(weigher.remaining_fungible_balance, u128::MAX);
			assert!(weigher.remaining_weight.is_zero());
			assert!(weigher.consumed_xcm_hash.is_none());
		});
}

#[test]
fn successful_on_stored_fungible_xcm_fee_asset_v3_with_zero_remaining_balance_and_weight() {
	let location = xcm::v3::MultiLocation {
		parents: 1,
		interior: xcm::v3::Junctions::X1(xcm::v3::Junction::Parachain(1_000)),
	};
	let new_switch_pair_info = {
		let mut new_switch_pair_info = get_switch_pair_info_for_remote_location::<MockRuntime>(
			&location.try_into().unwrap(),
			SwitchPairStatus::Running,
		);
		// Set XCM fee asset to the XCM version 3.
		new_switch_pair_info.remote_xcm_fee = new_switch_pair_info.remote_xcm_fee.into_version(3).unwrap();
		new_switch_pair_info
	};
	// Results in an amount of `2` local currency tokens.
	let weight_to_refund = Weight::from_parts(1, 1);
	let xcm_context = XcmContext::with_message_id([0u8; 32]);
	// Nothing is refunded.
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info)
		.build()
		.execute_with(|| {
			let mut weigher = {
				let mut weigher = UsingComponentsForXcmFeeAsset::<MockRuntime, _, SumTimeAndProofValues>::new();
				weigher.remaining_fungible_balance = u128::zero();
				weigher.remaining_weight = Weight::zero();
				weigher.consumed_xcm_hash = Some([0u8; 32]);
				weigher
			};
			let amount_refunded = weigher.refund_weight(weight_to_refund, &xcm_context);
			assert_eq!(amount_refunded, None);
			assert!(weigher.remaining_fungible_balance.is_zero());
			assert!(weigher.remaining_weight.is_zero());
			assert!(weigher.consumed_xcm_hash.is_none());
		});
}

#[test]
fn successful_on_stored_fungible_xcm_fee_asset_v2() {
	let location = xcm::v2::MultiLocation {
		parents: 1,
		interior: xcm::v2::Junctions::X1(xcm::v2::Junction::Parachain(1_000)),
	};
	let new_switch_pair_info = {
		let location_v3: xcm::v3::MultiLocation = location.try_into().unwrap();
		let mut new_switch_pair_info = get_switch_pair_info_for_remote_location::<MockRuntime>(
			&location_v3.try_into().unwrap(),
			SwitchPairStatus::Running,
		);
		// Set XCM fee asset to the XCM version 2.
		new_switch_pair_info.remote_xcm_fee = new_switch_pair_info.remote_xcm_fee.into_version(2).unwrap();
		new_switch_pair_info
	};
	// Results in an amount of `2` local currency tokens.
	let weight_to_refund = Weight::from_parts(1, 1);
	let xcm_context = XcmContext::with_message_id([0u8; 32]);
	// Case when remaining balance and weight are both higher than refund.
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build()
		.execute_with(|| {
			let mut weigher = {
				let mut weigher = UsingComponentsForXcmFeeAsset::<MockRuntime, _, SumTimeAndProofValues>::new();
				weigher.remaining_fungible_balance = u128::MAX;
				weigher.remaining_weight = Weight::MAX;
				weigher.consumed_xcm_hash = Some([0u8; 32]);
				weigher
			};
			let amount_refunded = weigher.refund_weight(weight_to_refund, &xcm_context);
			assert_eq!(
				amount_refunded,
				Some(Asset {
					id: Asset::try_from(new_switch_pair_info.clone().remote_xcm_fee).unwrap().id,
					fun: Fungibility::Fungible(2)
				})
			);
			assert_eq!(weigher.remaining_fungible_balance, u128::MAX - 2);
			assert_eq!(weigher.remaining_weight, Weight::MAX - weight_to_refund);
			assert!(weigher.consumed_xcm_hash.is_none());
		});
	// Case when remaining balance is 0 -> Nothing is refunded.
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build()
		.execute_with(|| {
			let mut weigher = {
				let mut weigher = UsingComponentsForXcmFeeAsset::<MockRuntime, _, SumTimeAndProofValues>::new();
				weigher.remaining_fungible_balance = u128::zero();
				weigher.remaining_weight = Weight::MAX;
				weigher.consumed_xcm_hash = Some([0u8; 32]);
				weigher
			};
			let amount_refunded = weigher.refund_weight(weight_to_refund, &xcm_context);
			assert_eq!(amount_refunded, None);
			assert!(weigher.remaining_fungible_balance.is_zero());
			assert_eq!(weigher.remaining_weight, Weight::MAX - weight_to_refund);
			assert!(weigher.consumed_xcm_hash.is_none());
		});
	// Case when remaining weight is 0 -> Nothing is refunded, remaining balance is
	// not changed.
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build()
		.execute_with(|| {
			let mut weigher = {
				let mut weigher = UsingComponentsForXcmFeeAsset::<MockRuntime, _, SumTimeAndProofValues>::new();
				weigher.remaining_fungible_balance = u128::MAX;
				weigher.remaining_weight = Weight::zero();
				weigher.consumed_xcm_hash = Some([0u8; 32]);
				weigher
			};
			let amount_refunded = weigher.refund_weight(weight_to_refund, &xcm_context);
			assert_eq!(amount_refunded, None);
			assert_eq!(weigher.remaining_fungible_balance, u128::MAX);
			assert!(weigher.remaining_weight.is_zero());
			assert!(weigher.consumed_xcm_hash.is_none());
		});
	// Case when both remaining weight and remaining balance are 0 -> Nothing is
	// refunded.
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info)
		.build()
		.execute_with(|| {
			let mut weigher = {
				let mut weigher = UsingComponentsForXcmFeeAsset::<MockRuntime, _, SumTimeAndProofValues>::new();
				weigher.remaining_fungible_balance = u128::zero();
				weigher.remaining_weight = Weight::zero();
				weigher.consumed_xcm_hash = Some([0u8; 32]);
				weigher
			};
			let amount_refunded = weigher.refund_weight(weight_to_refund, &xcm_context);
			assert_eq!(amount_refunded, None);
			assert!(weigher.remaining_fungible_balance.is_zero());
			assert!(weigher.remaining_weight.is_zero());
			assert!(weigher.consumed_xcm_hash.is_none());
		});
}

#[test]
fn skips_on_weight_not_previously_purchased() {
	let location = Location {
		parents: 1,
		interior: Junctions::X1([Junction::Parachain(1_000)].into()),
	};
	let new_switch_pair_info = {
		let mut new_switch_pair_info =
			get_switch_pair_info_for_remote_location::<MockRuntime>(&location, SwitchPairStatus::Running);
		// Set XCM fee asset to the XCM version 3.
		new_switch_pair_info.remote_xcm_fee = new_switch_pair_info.remote_xcm_fee.into_version(3).unwrap();
		new_switch_pair_info
	};
	// Results in an amount of `2` local currency tokens.
	let weight_to_refund = Weight::from_parts(1, 1);
	let xcm_context = XcmContext::with_message_id([0u8; 32]);
	// Fails with XCM message hash `None`.
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build()
		.execute_with(|| {
			let mut weigher = {
				let mut weigher = UsingComponentsForXcmFeeAsset::<MockRuntime, _, SumTimeAndProofValues>::new();
				weigher.remaining_fungible_balance = u128::MAX;
				weigher.remaining_weight = Weight::MAX;
				// Setting this to 'None' triggers the "not bought with me" condition.
				weigher.consumed_xcm_hash = None;
				weigher
			};
			let initial_weigher = weigher.clone();
			let amount_refunded = weigher.refund_weight(weight_to_refund, &xcm_context);
			assert!(amount_refunded.is_none());
			assert_eq!(initial_weigher, weigher);
		});
	// Fails with XCM message hash `Some(something_else)`.
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info)
		.build()
		.execute_with(|| {
			let mut weigher = {
				let mut weigher = UsingComponentsForXcmFeeAsset::<MockRuntime, _, SumTimeAndProofValues>::new();
				weigher.remaining_fungible_balance = u128::MAX;
				weigher.remaining_weight = Weight::MAX;
				// Setting this to a different value than expected also triggers the "not bought
				// with me" condition.
				weigher.consumed_xcm_hash = Some([100; 32]);
				weigher
			};
			let initial_weigher = weigher.clone();
			let amount_refunded = weigher.refund_weight(weight_to_refund, &xcm_context);
			assert!(amount_refunded.is_none());
			assert_eq!(initial_weigher, weigher);
		});
}

#[test]
fn skips_on_switch_pair_not_set() {
	ExtBuilder::default().build().execute_with(|| {
		let mut weigher = {
			let mut weigher = UsingComponentsForXcmFeeAsset::<MockRuntime, _, SumTimeAndProofValues>::new();
			weigher.remaining_fungible_balance = u128::MAX;
			weigher.remaining_weight = Weight::MAX;
			weigher.consumed_xcm_hash = Some([0u8; 32]);
			weigher
		};
		let initial_weigher = weigher.clone();
		let amount_refunded = weigher.refund_weight(Weight::from_parts(1, 1), &XcmContext::with_message_id([0u8; 32]));
		assert!(amount_refunded.is_none());
		assert_eq!(initial_weigher, weigher);
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
		.build()
		.execute_with(|| {
			let mut weigher = UsingComponentsForXcmFeeAsset::<MockRuntime, _, SumTimeAndProofValues>::new();
			assert!(weigher
				.refund_weight(Weight::from_parts(1, 1), &XcmContext::with_message_id([0u8; 32]))
				.is_none());
			assert_storage_noop!(drop(weigher));
		});
}

#[test]
fn skips_on_stored_non_fungible_xcm_fee_asset() {
	let location = Location {
		parents: 1,
		interior: Junctions::X1([Junction::Parachain(1_000)].into()),
	};
	let new_switch_pair_info = {
		let mut new_switch_pair_info =
			get_switch_pair_info_for_remote_location::<MockRuntime>(&location, SwitchPairStatus::Running);
		// Set XCM fee asset to the XCM version 3.
		let non_fungible_remote_xcm_fee_v3 = Asset::try_from(new_switch_pair_info.remote_xcm_fee)
			.map(|asset| Asset {
				id: asset.id,
				fun: Fungibility::NonFungible(AssetInstance::Index(1)),
			})
			.unwrap();
		new_switch_pair_info.remote_xcm_fee = non_fungible_remote_xcm_fee_v3.into();
		new_switch_pair_info
	};
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info)
		.build()
		.execute_with(|| {
			let mut weigher = {
				let mut weigher = UsingComponentsForXcmFeeAsset::<MockRuntime, _, SumTimeAndProofValues>::new();
				weigher.remaining_fungible_balance = u128::MAX;
				weigher.remaining_weight = Weight::MAX;
				weigher.consumed_xcm_hash = Some([0u8; 32]);
				weigher
			};
			let initial_weigher = weigher.clone();
			assert!(weigher
				.refund_weight(Weight::from_parts(1, 1), &XcmContext::with_message_id([0u8; 32]))
				.is_none());
			assert_eq!(initial_weigher, weigher);
		});
}
