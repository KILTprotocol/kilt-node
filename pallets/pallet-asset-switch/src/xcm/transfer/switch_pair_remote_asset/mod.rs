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
use xcm::v4::{Asset, AssetId, Location};

use crate::{Config, SwitchPair};

#[cfg(test)]
mod tests;

const LOG_TARGET: &str = "xcm::barriers::pallet-asset-switch::AllowSwitchPairRemoteAsset";

/// Type implementing [ContainsPair] and returns
/// `true` if the specified asset ID matches the switch pair remote asset ID,
/// which must be reserve transferred to this chain to be traded back for
/// the local token. The fungibility of either asset is not checked, and that
/// logic is delegated to the other XCM components, such as the asset
/// transactor(s).
pub struct IsSwitchPairRemoteAsset<T, I>(PhantomData<(T, I)>);

impl<T, I> ContainsPair<Asset, Location> for IsSwitchPairRemoteAsset<T, I>
where
	T: Config<I>,
	I: 'static,
{
	fn contains(a: &Asset, b: &Location) -> bool {
		log::info!(target: LOG_TARGET, "contains {:?}, {:?}", a, b);
		// 1. Verify a switch pair has been set. We don't care if it's enabled at this
		//    stage, as we still want the assets to move inside this system.
		let Some(switch_pair) = SwitchPair::<T, I>::get() else {
			return false;
		};

		// 2. We only trust the EXACT configured remote location (no parent is allowed).
		let Ok(stored_remote_reserve_location_v4): Result<Location, _> = switch_pair.remote_reserve_location.clone().try_into().map_err(|e| {
				log::error!(target: LOG_TARGET, "Failed to convert stored remote reserve location {:?} into v4 xcm version with error {:?}.", switch_pair.remote_reserve_location, e);
				e
			 }) else { return false; };
		if stored_remote_reserve_location_v4 != *b {
			log::trace!(
				target: LOG_TARGET,
				"Remote origin {:?} does not match expected origin {:?}",
				b,
				stored_remote_reserve_location_v4
			);
			return false;
		}

		// 3. Verify the asset ID matches the remote asset ID to switch for local ones.
		let Ok(stored_remote_asset_id): Result<AssetId, _> = switch_pair.remote_asset_id.clone().try_into().map_err(|e| {
				log::error!(target: LOG_TARGET, "Failed to convert stored remote asset ID {:?} into v4 xcm version with error {:?}.", switch_pair.remote_asset_id, e);
				e
			 }) else { return false; };

		a.id == stored_remote_asset_id
	}
}