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

use frame_support::ensure;
use sp_std::marker::PhantomData;
use xcm::v3::{AssetId, Fungibility, MultiAsset, MultiLocation};
use xcm_executor::traits::{Error as XcmExecutorError, MatchesFungibles};

use crate::{Config, SwitchPair};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

const LOG_TARGET: &str = "xcm::pallet-asset-switch::MatchesSwitchPairXcmFeeFungibleAsset";

/// Type implementing [MatchesFungibles] and returns the provided
/// fungible amount if the specified `MultiLocation` matches the asset used by
/// the switch pallet to pay for XCM fees at the configured remote location
/// (`switch_pair_info.remote_xcm_fee`).
pub struct MatchesSwitchPairXcmFeeFungibleAsset<T, I>(PhantomData<(T, I)>);

impl<T, I, FungiblesBalance> MatchesFungibles<MultiLocation, FungiblesBalance>
	for MatchesSwitchPairXcmFeeFungibleAsset<T, I>
where
	T: Config<I>,
	I: 'static,
	FungiblesBalance: From<u128>,
{
	fn matches_fungibles(a: &MultiAsset) -> Result<(MultiLocation, FungiblesBalance), XcmExecutorError> {
		log::info!(target: LOG_TARGET, "matches_fungibles {:?}", a);
		// 1. Retrieve switch pair from storage.
		let switch_pair = SwitchPair::<T, I>::get().ok_or(XcmExecutorError::AssetNotHandled)?;

		// 2. Ensure switch pair is enabled
		ensure!(switch_pair.is_enabled(), XcmExecutorError::AssetNotHandled);

		// 3. Match stored asset ID with input asset ID.
		let MultiAsset { id, fun } = switch_pair.remote_xcm_fee.clone().try_into().map_err(|e| {
			log::error!(
				target: LOG_TARGET,
				"Failed to convert stored remote fee asset {:?} into v3 MultiLocation with error {:?}.",
				switch_pair.remote_xcm_fee,
				e
			);
			XcmExecutorError::AssetNotHandled
		})?;
		ensure!(id == a.id, XcmExecutorError::AssetNotHandled);
		// 4. Verify the stored asset is a fungible one.
		let Fungibility::Fungible(_) = fun else {
			log::info!(target: LOG_TARGET, "Stored remote fee asset {:?} is not a fungible one.", switch_pair.remote_xcm_fee);
			return Err(XcmExecutorError::AssetNotHandled);
		};

		// After this ensure, we know we need to be transacting with this asset, so any
		// errors thrown from here onwards is a `FailedToTransactAsset` error.

		// 5. Force stored asset as a concrete one.
		let AssetId::Concrete(location) = id else {
			log::error!(target: LOG_TARGET, "Configured XCM fee asset {:?} is supposed to be concrete but it is not.", id);
			return Err(XcmExecutorError::AssetIdConversionFailed);
		};
		// 6. Force input asset as a fungible one and return its amount.
		let Fungibility::Fungible(amount) = a.fun else {
			log::info!(target: LOG_TARGET, "Input asset {:?} is supposed to be fungible but it is not.", a);
			return Err(XcmExecutorError::AmountToBalanceConversionFailed);
		};

		log::trace!(target: LOG_TARGET, "matched {:?}", (location, amount));
		Ok((location, amount.into()))
	}
}
