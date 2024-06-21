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

// TODO: Add unit tests
pub struct AllowOnlySwapAssetFromConfiguredLocation<T>(PhantomData<T>);

impl<T> ContainsPair<MultiAsset, MultiLocation> for AllowOnlySwapAssetFromConfiguredLocation<T>
where
	T: Config,
{
	fn contains(a: &MultiAsset, b: &MultiLocation) -> bool {
		// 1. Verify a swap pair has been set.
		let Some(swap_pair) = SwapPair::<T>::get() else {
			log::trace!(target: LOG_TARGET, "No swap pair configured.");
			return false;
		};

		// 2. Verify the expected reserve location matches exactly the XCM origin
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

		// 3. Verify the asset matches the expected one from the other side of the swap
		//    pair
		let Ok(stored_remote_asset_id): Result<AssetId, _> = swap_pair.remote_asset_id.clone().try_into().map_err(|e| {
			log::error!(target: LOG_TARGET, "Failed to convert stored remote asset ID {:?} into required version with error {:?}.", swap_pair.remote_asset_id, e);
			e
		 }) else { return false; };
		if stored_remote_asset_id != a.id {
			log::trace!(
				target: LOG_TARGET,
				"Remote asset {:?} does not match expected one {:?}",
				a.id,
				stored_remote_asset_id
			);
			return false;
		}

		true
	}
}

// TODO: Could all be unified into one so that we access storage only once,
// although with caching this should not really be an issue.
// TODO: Add unit tests
pub struct AllowOnlyXcmFeeAssetFromConfiguredLocation<T>(PhantomData<T>);

impl<T> ContainsPair<MultiAsset, MultiLocation> for AllowOnlyXcmFeeAssetFromConfiguredLocation<T>
where
	T: Config,
{
	fn contains(a: &MultiAsset, b: &MultiLocation) -> bool {
		// 1. Verify a swap pair has been set.
		let Some(swap_pair) = SwapPair::<T>::get() else {
			log::trace!(target: LOG_TARGET, "No swap pair configured.");
			return false;
		};

		// 2. Verify the expected reserve location matches exactly the XCM origin
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

		// 3. Verify the asset matches the one configured as fee asset.
		let Ok(stored_remote_asset_fee): Result<MultiAsset, _> = swap_pair.remote_fee.clone().try_into().map_err(|e| {
			log::error!(target: LOG_TARGET, "Failed to convert stored remote asset fee {:?} into required version with error {:?}.", swap_pair.remote_fee, e);
			e
		 }) else { return false; };
		if stored_remote_asset_fee != *a {
			log::trace!(
				target: LOG_TARGET,
				"Remote asset {:?} does not match expected one for fee payment {:?}",
				a,
				stored_remote_asset_fee
			);
			return false;
		}

		true
	}
}
