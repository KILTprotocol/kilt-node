// KILT Blockchain – <https://kilt.io>
// Copyright (C) 2025, KILT Foundation

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

// If you feel like getting in touch with us, you can do so at <hello@kilt.org>

// Required since the pallet `AssetTransactor` will try to deduct the XCM fee
// from the user's balance, and the asset must exist.

use frame_system::RawOrigin;
use pallet_asset_switch::PartialBenchmarkInfo;
use runtime_common::AccountId;
use xcm::v4::{Asset, AssetId, Fungibility, Junction, Junctions, Location, ParentThen};

use crate::{Fungibles, ParachainSystem};

pub struct CreateFungibleForAssetSwitchPool1;

impl pallet_asset_switch::BenchmarkHelper for CreateFungibleForAssetSwitchPool1 {
	fn setup() -> Option<PartialBenchmarkInfo> {
		const DESTINATION_PARA_ID: u32 = 1_000;

		let asset_location: Location = Junctions::Here.into();
		Fungibles::create(
			RawOrigin::Root.into(),
			asset_location.clone(),
			AccountId::from([0; 32]).into(),
			1u32.into(),
		)
		.unwrap();
		let beneficiary = Junctions::X1(
			[Junction::AccountId32 {
				network: None,
				id: [0; 32],
			}]
			.into(),
		)
		.into();
		let destination = Location::from(ParentThen(Junctions::X1(
			[Junction::Parachain(DESTINATION_PARA_ID)].into(),
		)))
		.into();
		let remote_xcm_fee = Asset {
			id: AssetId(asset_location),
			fun: Fungibility::Fungible(1_000),
		}
		.into();

		ParachainSystem::open_outbound_hrmp_channel_for_benchmarks_or_tests(DESTINATION_PARA_ID.into());

		Some(PartialBenchmarkInfo {
			beneficiary: Some(beneficiary),
			destination: Some(destination),
			remote_asset_id: None,
			remote_xcm_fee: Some(remote_xcm_fee),
		})
	}
}
