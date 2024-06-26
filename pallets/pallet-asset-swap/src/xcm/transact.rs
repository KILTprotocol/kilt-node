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

use frame_support::{
	ensure,
	traits::{fungible::Mutate, tokens::Preservation},
};
use sp_std::marker::PhantomData;
use xcm::prelude::{AssetId, Fungibility, MultiAsset, MultiLocation, XcmContext, XcmError, XcmResult};
use xcm_executor::traits::{ConvertLocation, TransactAsset};

use crate::{Config, LocalCurrencyBalanceOf, SwapPair, SwapPairInfoOf, LOG_TARGET};

// TODO: Add unit tests
pub struct SwapPairRemoteAssetTransactor<AccountIdConverter, T>(PhantomData<(AccountIdConverter, T)>);

impl<AccountIdConverter, T> TransactAsset for SwapPairRemoteAssetTransactor<AccountIdConverter, T>
where
	AccountIdConverter: ConvertLocation<T::AccountId>,
	T: Config,
{
	fn deposit_asset(what: &MultiAsset, who: &MultiLocation, _context: &XcmContext) -> XcmResult {
		// 1. Verify the swap pair exists.
		let swap_pair = SwapPair::<T>::get().ok_or(XcmError::AssetNotFound)?;

		// 2. Verify the swap pair is running.
		ensure!(swap_pair.can_swap(), XcmError::AssetNotFound);

		// 3. Verify the asset matches the other side of the swap pair.
		let stored_asset_id_as_required_version: AssetId =
			swap_pair.remote_asset_id.clone().try_into().map_err(|e| {
				log::error!(
					target: LOG_TARGET,
					"Failed to convert stored asset ID {:?} into required version with error {:?}.",
					swap_pair.remote_asset_id,
					e
				);
				XcmError::AssetNotFound
			})?;
		ensure!(stored_asset_id_as_required_version == what.id, XcmError::AssetNotFound);
		// After this ensure, we know we need to be transacting with this asset, so any
		// errors thrown from here onwards is a `FailedToTransactAsset` error.

		// 4. Perform the local transfer
		let beneficiary = AccountIdConverter::convert_location(who).ok_or(XcmError::FailedToTransactAsset(
			"Failed to convert beneficiary to valid account.",
		))?;
		let Fungibility::Fungible(fungible_amount) = what.fun else {
			return Err(XcmError::FailedToTransactAsset(
				"Deposited token expected to be fungible.",
			));
		};
		let fungible_amount_as_currency_balance: LocalCurrencyBalanceOf<T> =
			fungible_amount.try_into().map_err(|_| {
				XcmError::FailedToTransactAsset("Failed to convert fungible amount to balance of local currency.")
			})?;
		T::LocalCurrency::transfer(
			&swap_pair.pool_account,
			&beneficiary,
			fungible_amount_as_currency_balance,
			Preservation::Expendable,
		)
		.map_err(|e| {
			log::error!(
				target: LOG_TARGET,
				"Failed to transfer assets from pool account with error {:?}",
				e
			);
			XcmError::FailedToTransactAsset("Failed to transfer assets from pool account")
		})?;

		// 5. Increase the balance of the remote asset
		let new_remote_balance =
			swap_pair
				.remote_asset_balance
				.checked_add(fungible_amount)
				.ok_or(XcmError::FailedToTransactAsset(
					"Failed to transfer assets from pool account",
				))?;
		SwapPair::<T>::try_mutate(|entry| {
			let SwapPairInfoOf::<T> {
				remote_asset_balance, ..
			} = entry
				.as_mut()
				.ok_or(XcmError::FailedToTransactAsset("SwapPair should not be None."))?;
			*remote_asset_balance = new_remote_balance;
			Ok::<_, XcmError>(())
		})?;

		Ok(())
	}
}
