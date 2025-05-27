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

use frame_support::ensure;
use xcm::v4::{Asset, AssetId, Fungibility, Location};
use xcm_executor::traits::{Error as XcmExecutorError, MatchesFungibles};

use super::get_remote_asset_id;

const LOG_TARGET: &str = "xcm::matcher::MatchesBkiltAsset";

pub struct MatchesBkiltAsset;

impl<FungiblesBalance> MatchesFungibles<Location, FungiblesBalance> for MatchesBkiltAsset
where
	FungiblesBalance: From<u128>,
{
	fn matches_fungibles(a: &Asset) -> Result<(Location, FungiblesBalance), XcmExecutorError> {
		log::info!(target: LOG_TARGET, "matches_fungibles {:?}", a);

		let asset_id = get_remote_asset_id();
		ensure!(asset_id == a.id, XcmExecutorError::AssetNotHandled);

		let AssetId(location) = asset_id;

		let Fungibility::Fungible(amount) = a.fun else {
			log::info!(target: LOG_TARGET, "Input asset {:?} is supposed to be fungible but it is not.", a);
			return Err(XcmExecutorError::AmountToBalanceConversionFailed);
		};

		log::trace!(target: LOG_TARGET, "matched {:?}", (location.clone(), amount));
		Ok((location, amount.into()))
	}
}
