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

use frame_support::traits::{fungible::Mutate, tokens::Preservation};
use xcm::v4::{AssetId, Junctions::Here, Location, Response, Weight, XcmContext};
use xcm_executor::traits::OnResponse;

use crate::{
	traits::SwitchHooks, Config, Event, LocalCurrencyBalanceOf, Pallet, PendingSwitchConfirmations, SwitchPair,
	SwitchPairInfo, UnconfirmedSwitchInfoOf,
};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

const LOG_TARGET: &str = "runtime::pallet-asset-switch::OnResponse";

impl<T: Config<I>, I: 'static> OnResponse for Pallet<T, I> {
	fn expecting_response(origin: &Location, query_id: u64, querier: Option<&Location>) -> bool {
		// Verify we are the original queriers.
		if querier != Some(&Here.into_location()) {
			log::trace!(
				target: LOG_TARGET,
				"Querier for query ID {:?} = {:?}, which is not us.",
				query_id, querier
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
				"Did not find a switch pair set.",
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
		// This order is important. We want to check if the origin is nested inside the
		// configure reserve location, and not the reverse. Hence, an `X2` origin would
		// fail this check if the reserve location is an `X1`. Conversely, if the origin
		// is an `X1(Parachain(1000))` and the configured reserve location is, for
		// instance, a `X2(Parachain(1000), AccountId32(<acc>))`, this check will pass,
		// as we make te same assumption Polkadot does, that nested locations are always
		// under the control of parent locations.
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
		if !Self::expecting_response(origin, query_id, querier) {
			log::error!(
				target: LOG_TARGET,
				"`on_response` called even tho `expecting_response` returned `false`.",
			);
			return Weight::zero();
		}
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
		let Some(mut switch_pair) = SwitchPair::<T, I>::get() else {
			log::error!(
				target: LOG_TARGET,
				"Cannot fetch switch pair from storage. This should not happen if `expecting_response` returned `true`.",
			);
			return Weight::zero();
		};
		let Ok(remote_asset_id_v4) = AssetId::try_from(switch_pair.remote_asset_id.clone()) else {
			log::error!(
				target: LOG_TARGET,
				"Failed to convert stored remote asset ID to v4.",
			);
			return Weight::zero();
		};

		let Some(UnconfirmedSwitchInfoOf::<T> { from, to, amount }) = PendingSwitchConfirmations::<T, I>::get(query_id)
		else {
			log::error!(
				target: LOG_TARGET,
				"Cannot fetch pending confirmation from storage. This should not happen if `expecting_response` returned `true`.",
			);
			return Weight::zero();
		};
		let Ok(local_amount) = LocalCurrencyBalanceOf::<T, I>::try_from(amount) else {
			log::error!(
				target: LOG_TARGET,
				"Failed to convert input amount {:?} into local balance.", amount
			);
			return Weight::zero();
		};

		let mut assets_of_kind = holding_assets
			.inner()
			.iter()
			.filter(|a| a.id == remote_asset_id_v4)
			.peekable();
		// No assets are reported or there's no asset of interest for us. Happy case.
		if holding_assets.is_none() || assets_of_kind.peek().is_none() {
			log::trace!(
				target: LOG_TARGET,
				"Switch was successful. Removing pending query {:?} from storage upon receiving holding report: {:?}.",
				query_id, holding_assets
			);
			PendingSwitchConfirmations::<T, I>::remove(query_id);
			T::SwitchHooks::post_local_to_remote_finalized(&from, &to, local_amount);
			Self::deposit_event(Event::<T, I>::LocalToRemoteSwitchFinalized { amount, from, to });
		// Sad case, we need to revert the user's transfer.
		} else if assets_of_kind.any(|a| a.fun == amount.into()) {
			let Ok(fungible_amount_as_currency_balance) = LocalCurrencyBalanceOf::<T, I>::try_from(amount) else {
				log::error!(
					target: LOG_TARGET,
					"Failed to convert fungible amount to balance of local currency."
				);
				return Weight::zero();
			};
			log::trace!(
				target: LOG_TARGET,
				"Switch failed. Reverting the transfer because of holding asset reported: {:?}.", holding_assets
			);
			let Ok(_) = T::LocalCurrency::transfer(
				&switch_pair.pool_account,
				&from,
				fungible_amount_as_currency_balance,
				Preservation::Preserve,
			) else {
				log::error!(
					target: LOG_TARGET,
					"Failed to transfer assets from pool account into original payer.",
				);
				return Weight::zero();
			};
			// We act like we received an incoming switch when updating the switch pair.
			let Ok(_) = switch_pair.try_process_incoming_switch(amount) else {
				log::error!(
					target: LOG_TARGET,
					"Failed to update the switch pair storage to account for the reverted operation.",
				);
				return Weight::zero();
			};
			SwitchPair::<T, I>::set(Some(switch_pair));
			PendingSwitchConfirmations::<T, I>::remove(query_id);
			T::SwitchHooks::post_local_to_remote_transfer_revert(&from, &to, local_amount);
			Self::deposit_event(Event::<T, I>::LocalToRemoteSwitchReverted { amount, from, to });
		// Weird case where the transfer has partially completed. We don't
		// explicitly handle this for now, but simply generate some error
		// logs, as this is definitely not expected.
		} else {
			log::error!(
				target: LOG_TARGET,
				"Transfer was partially completed, which is currently not expected nor handled. Content of the holding register: {:?}",
				holding_assets
			);
		}
		Weight::zero()
	}
}
