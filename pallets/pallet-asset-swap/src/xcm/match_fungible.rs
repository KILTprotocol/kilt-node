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

use crate::{Config, SwapPair, SwapPairInfoOf, LOG_TARGET};

pub struct MatchesSwapPairXcmFeeAsset<T>(PhantomData<T>);

impl<T, FungiblesBalance> MatchesFungibles<MultiLocation, FungiblesBalance> for MatchesSwapPairXcmFeeAsset<T>
where
	T: Config,
	FungiblesBalance: From<u128>,
{
	fn matches_fungibles(a: &MultiAsset) -> Result<(MultiLocation, FungiblesBalance), XcmExecutorError> {
		// 1. Retrieve swap pair from storage.
		let SwapPairInfoOf::<T> { remote_fee, .. } = SwapPair::<T>::get().ok_or(XcmExecutorError::AssetNotHandled)?;

		// 2. Match stored asset ID with input asset ID.
		let MultiAsset { id, .. } = remote_fee.clone().try_into().map_err(|e| {
			log::error!(target: LOG_TARGET, "Failed to convert stored remote fee asset {:?} into v3 MultiLocation with error {:?}.", remote_fee, e);
			XcmExecutorError::AssetNotHandled
		})?;
		ensure!(id == a.id, XcmExecutorError::AssetNotHandled);

		// 3. Force stored asset as a concrete and fungible one and return its amount.
		let AssetId::Concrete(location) = id else {
			log::error!(target: LOG_TARGET, "Configured XCM fee asset {:?} is supposed to be concrete but it is not.", id);
			// TODO: Change error to something else, now that we now the asset is the right
			// asset (based on the ensure! above)
			return Err(XcmExecutorError::AssetNotHandled);
		};
		let Fungibility::Fungible(amount) = a.fun else {
			log::error!(target: LOG_TARGET, "Input asset {:?} is supposed to be fungible but it is not.", a);
			// TODO: Change error to something else, now that we now the asset is the right
			// asset (based on the ensure! above)
			return Err(XcmExecutorError::AssetNotHandled);
		};

		Ok((location, amount.into()))
	}
}
