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
use sp_std::marker::PhantomData;
use xcm::prelude::{AssetId, MultiAsset, MultiLocation};

use crate::{Config, SwapPair, LOG_TARGET};

pub struct ReserveTransfersOfXcmFeeAssetAndRemoteAsset<T>(PhantomData<T>);

impl<T> ContainsPair<MultiAsset, MultiLocation> for ReserveTransfersOfXcmFeeAssetAndRemoteAsset<T>
where
	T: Config,
{
	fn contains(a: &MultiAsset, b: &MultiLocation) -> bool {
		// 1. Verify a swap pair has been set.
		let Some(swap_pair) = SwapPair::<T>::get() else {
			log::trace!(target: LOG_TARGET, "No swap pair configured.");
			return false;
		};

		// 2. For both XCM fee asset and remote asset, we only trust the configured
		//    remote location.
		let Ok(stored_remote_reserve_location_as_required_version): Result<MultiLocation, _> = swap_pair.remote_reserve_location.clone().try_into().map_err(|e| {
			log::error!(target: LOG_TARGET, "Failed to convert stored remote reserve location {:?} into required version with error {:?}.", swap_pair.remote_reserve_location, e);
			e
		 }) else { return false; };
		if stored_remote_reserve_location_as_required_version != *b {
			log::trace!(
				target: LOG_TARGET,
				"Remote origin {:?} does not match expected origin {:?}",
				b,
				stored_remote_reserve_location_as_required_version
			);
			return false;
		}

		// 3. Verify the asset matches either the configured XCM fee asset or remote
		//    asset to swap for local ones.
		let Ok(stored_remote_asset_id): Result<AssetId, _> = swap_pair.remote_asset_id.clone().try_into().map_err(|e| {
			log::error!(target: LOG_TARGET, "Failed to convert stored remote asset ID {:?} into required version with error {:?}.", swap_pair.remote_asset_id, e);
			e
		 }) else { return false; };
		let Ok(stored_remote_asset_fee): Result<MultiAsset, _> = swap_pair.remote_fee.clone().try_into().map_err(|e| {
			log::error!(target: LOG_TARGET, "Failed to convert stored remote asset fee {:?} into required version with error {:?}.", swap_pair.remote_fee, e);
			e
		 }) else { return false; };

		match a.id {
			remote_asset_id if remote_asset_id == stored_remote_asset_id => true,
			xcm_fee_asset_id if xcm_fee_asset_id == stored_remote_asset_fee.id => true,
			_ => {
				log::info!(target: LOG_TARGET, "Received asset ID {:?} does not match neither the expected remote asset ID {:?} nor the XCM fee asset ID {:?}", a.id, stored_remote_asset_id, stored_remote_asset_fee.id);
				false
			}
		}
	}
}
