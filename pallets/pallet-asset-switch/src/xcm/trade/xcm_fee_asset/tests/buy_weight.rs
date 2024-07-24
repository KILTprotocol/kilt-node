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
	v3::{AssetInstance, Error, Fungibility, MultiAsset, Weight, XcmContext},
	IntoVersion,
};
use xcm_executor::{traits::WeightTrader, Assets};

use crate::xcm::{
	trade::mock::{
		get_switch_pair_info_for_remote_location, is_weigher_unchanged, ExtBuilder, MockRuntime, SumTimeAndProofValues,
	},
	UsingComponentsForXcmFeeAsset,
};

#[test]
fn successful_on_stored_fungible_xcm_fee_asset_latest() {
	let location = xcm::latest::MultiLocation {
		parents: 1,
		interior: xcm::latest::Junctions::X1(xcm::latest::Junction::Parachain(1_000)),
	};
	let new_switch_pair_info = {
		let mut new_switch_pair_info = get_switch_pair_info_for_remote_location(&location);
		// Set XCM fee asset to the latest XCM version.
		new_switch_pair_info.remote_xcm_fee = new_switch_pair_info.remote_xcm_fee.into_latest().unwrap();
		new_switch_pair_info
	};
	// Results in a required amount of `2` local currency tokens.
	let weight_to_buy = Weight::from_parts(1, 1);
	let xcm_context = XcmContext::with_message_id([0u8; 32]);
	// Works with an input fungible amount.
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build()
		.execute_with(|| {
			let mut weigher = UsingComponentsForXcmFeeAsset::<MockRuntime, _, SumTimeAndProofValues>::new();
			let payment: Assets = vec![MultiAsset {
				id: MultiAsset::try_from(new_switch_pair_info.clone().remote_xcm_fee)
					.unwrap()
					.id,
				fun: Fungibility::Fungible(2),
			}]
			.into();
			let unused_weight = weigher.buy_weight(weight_to_buy, payment, &xcm_context).unwrap();
			assert!(unused_weight.is_empty());
			assert_eq!(weigher.consumed_xcm_hash, Some(xcm_context.message_id));
			assert_eq!(weigher.remaining_fungible_balance, 2);
			assert_eq!(weigher.remaining_weight, weight_to_buy);
		});
	// Fails with an input non-fungible amount.
	ExtBuilder::default()
		.with_switch_pair_info(new_switch_pair_info.clone())
		.build()
		.execute_with(|| {
			let mut weigher = UsingComponentsForXcmFeeAsset::<MockRuntime, _, SumTimeAndProofValues>::new();
			let payment: Assets = vec![MultiAsset {
				id: MultiAsset::try_from(new_switch_pair_info.clone().remote_xcm_fee)
					.unwrap()
					.id,
				fun: Fungibility::NonFungible(AssetInstance::Index(1)),
			}]
			.into();

			assert_err!(
				weigher.buy_weight(weight_to_buy, payment, &xcm_context),
				Error::TooExpensive
			);
			assert!(is_weigher_unchanged(&weigher));
		});
}

#[test]
fn successful_on_stored_fungible_xcm_fee_asset_v3() {}

#[test]
fn successful_on_stored_fungible_xcm_fee_asset_v2() {}

#[test]
fn successful_on_stored_non_fungible_xcm_fee_asset_latest() {}

#[test]
fn successful_on_stored_non_fungible_xcm_fee_asset_v3() {}

#[test]
fn successful_on_stored_non_fungible_xcm_fee_asset_v2() {}

#[test]
fn skips_on_switch_pair_not_set() {}

#[test]
fn fails_on_too_expensive() {}
