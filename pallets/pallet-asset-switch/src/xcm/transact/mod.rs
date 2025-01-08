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
use xcm::v4::{Asset, AssetId, Error, Fungibility, Location, Result, XcmContext};
use xcm_executor::traits::{ConvertLocation, TransactAsset};

use crate::{traits::SwitchHooks, Config, Event, LocalCurrencyBalanceOf, Pallet, SwitchPair};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

const LOG_TARGET: &str = "xcm::pallet-asset-switch::SwitchPairRemoteAssetTransactor";

/// Type implementing [TransactAsset] that moves from the switch pair pool
/// account, if present, as many local tokens as remote assets received into
/// the specified `Location` if the incoming asset ID matches the remote
/// asset ID as specified in the switch pair and if they are both fungible.
pub struct SwitchPairRemoteAssetTransactor<AccountIdConverter, T, I>(PhantomData<(AccountIdConverter, T, I)>);

impl<AccountIdConverter, T, I> TransactAsset for SwitchPairRemoteAssetTransactor<AccountIdConverter, T, I>
where
	AccountIdConverter: ConvertLocation<T::AccountId>,
	T: Config<I>,
	I: 'static,
{
	fn deposit_asset(what: &Asset, who: &Location, context: Option<&XcmContext>) -> Result {
		log::info!(target: LOG_TARGET, "deposit_asset {:?} {:?} {:?}", what, who, context);
		// 1. Verify the switch pair exists.
		let switch_pair = SwitchPair::<T, I>::get().ok_or(Error::AssetNotFound)?;

		// 2. Verify the asset matches the other side of the switch pair.
		let remote_asset_id_v4: AssetId = switch_pair.remote_asset_id.clone().try_into().map_err(|e| {
			log::error!(
				target: LOG_TARGET,
				"Failed to convert stored asset ID {:?} into required version with error {:?}.",
				switch_pair.remote_asset_id,
				e
			);
			Error::AssetNotFound
		})?;
		ensure!(remote_asset_id_v4 == what.id, Error::AssetNotFound);
		// 3. Verify the asset being deposited is fungible.
		let Fungibility::Fungible(fungible_amount) = what.fun else {
			return Err(Error::AssetNotFound);
		};
		// After this ensure, we know we need to be transacting with this asset, so any
		// errors thrown from here onwards is a `FailedToTransactAsset` error.

		// 4. Verify the switch pair is running.
		ensure!(
			switch_pair.is_enabled(),
			Error::FailedToTransactAsset("switch pair is not running.",)
		);

		let beneficiary = AccountIdConverter::convert_location(who).ok_or(Error::FailedToTransactAsset(
			"Failed to convert beneficiary to valid account.",
		))?;
		// 5. Call into the pre-switch hook
		T::SwitchHooks::pre_remote_to_local_switch(&beneficiary, fungible_amount).map_err(|e| {
			log::error!(
				target: LOG_TARGET,
				"Hook pre-switch check failed with error code {:?}",
				e.into()
			);
			Error::FailedToTransactAsset("Failed to validate preconditions for remote-to-local switch.")
		})?;

		// 6. Perform the local transfer
		let fungible_amount_as_currency_balance: LocalCurrencyBalanceOf<T, I> =
			fungible_amount.try_into().map_err(|_| {
				Error::FailedToTransactAsset("Failed to convert fungible amount to balance of local currency.")
			})?;
		T::LocalCurrency::transfer(
			&switch_pair.pool_account,
			&beneficiary,
			fungible_amount_as_currency_balance,
			Preservation::Preserve,
		)
		.map_err(|e| {
			log::error!(
				target: LOG_TARGET,
				"Failed to transfer assets from pool account with error {:?}",
				e
			);
			Error::FailedToTransactAsset("Failed to transfer assets from pool account to specified account.")
		})?;

		// 6. Increase the balance of the remote asset
		SwitchPair::<T, I>::try_mutate(|entry| {
			let switch_pair_info = entry
				.as_mut()
				.ok_or(Error::FailedToTransactAsset("SwitchPair should not be None."))?;
			switch_pair_info
				.try_process_incoming_switch(fungible_amount)
				.map_err(|_| {
					Error::FailedToTransactAsset("Failed to apply the transfer outcome to the storage components.")
				})?;
			Ok::<_, Error>(())
		})?;

		// 7. Call into the post-switch hook
		T::SwitchHooks::post_remote_to_local_switch(&beneficiary, fungible_amount).map_err(|e| {
			log::error!(
				target: LOG_TARGET,
				"Hook post-switch check failed with error code {:?}",
				e.into()
			);
			Error::FailedToTransactAsset("Failed to validate postconditions for remote-to-local switch.")
		})?;

		Pallet::<T, I>::deposit_event(Event::<T, I>::RemoteToLocalSwitchExecuted {
			amount: fungible_amount,
			to: beneficiary,
		});

		Ok(())
	}
}
