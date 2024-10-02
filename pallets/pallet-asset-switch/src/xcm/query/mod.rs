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

use frame_support::traits::{fungible::Mutate, tokens::Preservation, DefensiveSaturating};
use xcm::v4::{AssetId, Location, Response, Weight, XcmContext};
use xcm_executor::traits::OnResponse;

use crate::{
	traits::QueryIdProvider, Config, Event, LocalCurrencyBalanceOf, Pallet, PendingSwitchConfirmations, SwitchPair,
	SwitchPairInfo,
};

const LOG_TARGET: &str = "runtime::pallet-asset-switch::OnResponse";

impl<T: Config<I>, I: 'static> OnResponse for Pallet<T, I> {
	fn expecting_response(origin: &Location, query_id: u64, querier: Option<&Location>) -> bool {
		// Verify we are the original queriers.
		if querier != Some(&T::UNIVERSAL_LOCATION.into_location()) {
			log::trace!(
				target: LOG_TARGET,
				"Querier for query ID {:?} {:?} is different than configured universal location {:?}",
				query_id, querier, T::UNIVERSAL_LOCATION
			);
			return false;
		}
		let Some(SwitchPairInfo {
			remote_reserve_location,
			..
		}) = SwitchPair::<T, I>::get()
		else {
			log::trace!(
				target: LOG_TARGET,
				"Querier for query ID {:?} {:?} is different than configured universal location {:?}",
				query_id, querier, T::UNIVERSAL_LOCATION
			);
			return false;
		};
		// Verify the response comes from the configured reserve location (or a
		// descendent).
		let Ok(remote_reserve_location_v4) = Location::try_from(remote_reserve_location.clone()) else {
			log::error!(
				target: LOG_TARGET,
				"Failed to convert remote reserve location {:?} into v4 `Location`",
				remote_reserve_location,
			);
			return false;
		};
		if !remote_reserve_location_v4.starts_with(origin) {
			log::trace!(
				target: LOG_TARGET,
				"Origin of query {:?} is not contained in configured trusted reserve location {:?}",
				origin, remote_reserve_location_v4
			);
			return false;
		}
		// Verify we were expecting such an answer.
		if !PendingSwitchConfirmations::<T, I>::contains_key(query_id) {
			log::trace!(
				target: LOG_TARGET,
				"No query with ID {:?} stored in storage.",
				query_id
			);
			return false;
		}
		true
	}

	fn on_response(
		origin: &Location,
		query_id: u64,
		querier: Option<&Location>,
		response: Response,
		max_weight: Weight,
		context: &XcmContext,
	) -> Weight {
		log::info!(
			target: LOG_TARGET,
			"Processing query with origin = {:?}, ID = {:?}, querier = {:?}, response = {:?}, max_weight = {:?}, context: {:?}",
			origin, query_id, querier, response, max_weight, context
		);
		let Response::Assets(holding_assets) = response else {
			log::trace!(
				target: LOG_TARGET,
				"Wrong type of response received: {:?}",
				response
			);
			return Weight::zero();
		};
		let Some(SwitchPairInfo {
			pool_account,
			remote_asset_id,
			..
		}) = SwitchPair::<T, I>::get()
		else {
			log::error!(
				target: LOG_TARGET,
				"Cannot fetch switch pair from storage. This should not happen if `expecting_response` returned `true`.",
			);
			return Weight::zero();
		};
		let Ok(remote_asset_id_v4) = AssetId::try_from(remote_asset_id) else {
			log::error!(
				target: LOG_TARGET,
				"Failed to convert stored remote asset ID to v4.",
			);
			return Weight::zero();
		};

		let Some((source, destination, amount)) = PendingSwitchConfirmations::<T, I>::get(query_id) else {
			log::error!(
				target: LOG_TARGET,
				"Cannot fetch pending confirmation from storage. This should not happen if `expecting_response` returned `true`.",
			);
			return Weight::zero();
		};

		let is_transfer_present = holding_assets.contains(&(remote_asset_id_v4, amount).into());
		// Happy case, let's remove the pending query from the storage.
		if holding_assets.is_none() || !is_transfer_present {
			log::trace!(
				target: LOG_TARGET,
				"Switch was successful. Removing pending query {:?} from storage.",
				query_id
			);
			PendingSwitchConfirmations::<T, I>::remove(query_id);
		// Sad case, we need to revert the user's transfer.
		} else if is_transfer_present {
			// TODO: Factor this out into its own function to be used with the remote ->
			// local switch processing as well.
			let Ok(fungible_amount_as_currency_balance) = LocalCurrencyBalanceOf::<T, I>::try_from(amount) else {
				log::error!(
					target: LOG_TARGET,
					"Failed to convert fungible amount to balance of local currency."
				);
				return Weight::zero();
			};
			let Ok(_) = T::LocalCurrency::transfer(
				&pool_account,
				&source,
				fungible_amount_as_currency_balance,
				Preservation::Preserve,
			) else {
				log::error!(
					target: LOG_TARGET,
					"Failed to transfer assets from pool account into original payer.",
				);
				return Weight::zero();
			};
			SwitchPair::<T, I>::mutate(|entry| {
				let Some(entry) = entry else {
					log::error!(
						target: LOG_TARGET,
						"Switch pair cannot be `None` at this point.",
					);
					// Do not change anything
					return;
				};
				// Defensive operations log something on their own.
				let new_balance = entry.remote_asset_circulating_supply.defensive_saturating_sub(amount);
				entry.remote_asset_circulating_supply = new_balance;
			});
			PendingSwitchConfirmations::<T, I>::remove(query_id);
			T::QueryIdProvider::remove_id(&query_id);
			Self::deposit_event(Event::<T, I>::SwitchReverted {
				amount,
				from: source,
				to: destination,
			});
		// Weird case where the transfer has partially completed. We don't
		// explicitly handle this for now, but simply generate some error
		// logs, as this is definitely not expected.
		} else {
			log::error!(
				target: LOG_TARGET,
				"Transfer was partially completed. Content of the holding register: {:?}",
				holding_assets
			);
		}
		Weight::zero()
	}
}
