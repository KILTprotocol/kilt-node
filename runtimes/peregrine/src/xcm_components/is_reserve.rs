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

// If you feel like getting in touch with us, you can do so at <hello@kilt.io>

use frame_support::traits::ContainsPair;
use xcm::v4::{Asset, Location};

use super::{get_remote_asset_id, get_remote_reserve_location};

const LOG_TARGET: &str = "xcm::barriers::bkilt::AllowBkiltAsset";

pub struct IsBKilt;

impl ContainsPair<Asset, Location> for IsBKilt {
	fn contains(a: &Asset, b: &Location) -> bool {
		log::info!(target: LOG_TARGET, "contains {:?}, {:?}", a, b);

		// 1 get asset location.
		let asset_location = get_remote_reserve_location();
		if asset_location != *b {
			log::trace!(
				target: LOG_TARGET,
				"Remote origin {:?} does not match expected origin {:?}",
				b,
				asset_location
			);
			return false;
		}

		let target_asset_id = get_remote_asset_id();

		let is_target_asset_id = target_asset_id == a.id;

		if !is_target_asset_id {
			log::trace!(target: LOG_TARGET, "Asset ID does not match the expected asset ID. Expected: {:?}, Actual: {:?}", target_asset_id, a.id);
		}

		is_target_asset_id
	}
}
