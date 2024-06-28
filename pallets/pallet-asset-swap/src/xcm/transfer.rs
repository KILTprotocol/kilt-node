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

pub use xcm_fee_asset::IsSwapPairXcmFeeAsset;
mod xcm_fee_asset {
	use frame_support::traits::ContainsPair;
	use sp_std::marker::PhantomData;
	use xcm::v3::{MultiAsset, MultiLocation};

	use crate::{Config, SwapPair};

	const LOG_TARGET: &str = "xcm::pallet-asset-swap::AllowXcmFeeAsset";

	/// Type implementing `ContainsPair<MultiAsset, MultiLocation>` and returns
	/// `true` if the specified asset matches the swap pair remote XCM fee
	/// asset, which must be reserve transferred to this chain in order to be
	/// withdrawn from the user's balance to pay for XCM fees at destination.
	pub struct IsSwapPairXcmFeeAsset<T, I>(PhantomData<(T, I)>);

	impl<T, I> ContainsPair<MultiAsset, MultiLocation> for IsSwapPairXcmFeeAsset<T, I>
	where
		T: Config<I>,
		I: 'static,
	{
		fn contains(a: &MultiAsset, b: &MultiLocation) -> bool {
			log::info!(target: LOG_TARGET, "contains {:?}, {:?}", a, b);
			// 1. Verify a swap pair has been set.
			let Some(swap_pair) = SwapPair::<T, I>::get() else {
				return false;
			};

			// 2. We only trust the configured remote location.
			let Ok(stored_remote_reserve_location_v3): Result<MultiLocation, _> = swap_pair.remote_reserve_location.clone().try_into().map_err(|e| {
				log::error!(target: LOG_TARGET, "Failed to convert stored remote reserve location {:?} into v3 with error {:?}.", swap_pair.remote_reserve_location, e);
				e
			 }) else { return false; };
			if stored_remote_reserve_location_v3 != *b {
				log::trace!(
					target: LOG_TARGET,
					"Remote origin {:?} does not match expected origin {:?}",
					b,
					stored_remote_reserve_location_v3
				);
				return false;
			}

			// 3. Verify the asset matches the configured XCM fee asset.
			let Ok(stored_remote_asset_fee): Result<MultiAsset, _> = swap_pair.remote_fee.clone().try_into().map_err(|e| {
				log::error!(target: LOG_TARGET, "Failed to convert stored remote asset fee {:?} into v3 with error {:?}.", swap_pair.remote_fee, e);
				e
			 }) else { return false; };

			a.id == stored_remote_asset_fee.id
		}
	}
}

pub use swap_pair_remote_asset::IsSwapPairRemoteAsset;
mod swap_pair_remote_asset {
	use frame_support::traits::ContainsPair;
	use sp_std::marker::PhantomData;
	use xcm::v3::{AssetId, MultiAsset, MultiLocation};

	use crate::{Config, SwapPair};

	const LOG_TARGET: &str = "xcm::barriers::pallet-asset-swap::AllowSwapPairRemoteAsset";

	/// Type implementing `ContainsPair<MultiAsset, MultiLocation>` and returns
	/// `true` if the specified asset matches the swap pair remote asset, which
	/// must be reserve transferred to this chain to be traded back for the
	/// local token.
	pub struct IsSwapPairRemoteAsset<T, I>(PhantomData<(T, I)>);

	impl<T, I> ContainsPair<MultiAsset, MultiLocation> for IsSwapPairRemoteAsset<T, I>
	where
		T: Config<I>,
		I: 'static,
	{
		fn contains(a: &MultiAsset, b: &MultiLocation) -> bool {
			log::info!(target: LOG_TARGET, "contains {:?}, {:?}", a, b);
			// 1. Verify a swap pair has been set.
			let Some(swap_pair) = SwapPair::<T, I>::get() else {
				return false;
			};

			// 2. We only trust the configured remote location.
			let Ok(stored_remote_reserve_location_v3): Result<MultiLocation, _> = swap_pair.remote_reserve_location.clone().try_into().map_err(|e| {
				log::error!(target: LOG_TARGET, "Failed to convert stored remote reserve location {:?} into v3 with error {:?}.", swap_pair.remote_reserve_location, e);
				e
			 }) else { return false; };
			if stored_remote_reserve_location_v3 != *b {
				log::trace!(
					target: LOG_TARGET,
					"Remote origin {:?} does not match expected origin {:?}",
					b,
					stored_remote_reserve_location_v3
				);
				return false;
			}

			// 3. Verify the asset matches the remote asset to swap for local ones.
			let Ok(stored_remote_asset_id): Result<AssetId, _> = swap_pair.remote_asset_id.clone().try_into().map_err(|e| {
				log::error!(target: LOG_TARGET, "Failed to convert stored remote asset ID {:?} into v3 with error {:?}.", swap_pair.remote_asset_id, e);
				e
			 }) else { return false; };

			a.id == stored_remote_asset_id
		}
	}
}
