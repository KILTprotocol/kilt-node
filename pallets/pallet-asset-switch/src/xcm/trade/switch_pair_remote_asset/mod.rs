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
	weights::WeightToFee as WeightToFeeT,
};
use sp_core::Get;
use sp_runtime::traits::Zero;
use sp_std::marker::PhantomData;
use xcm::v4::{Asset, AssetId, Error, Weight, XcmContext, XcmHash};
use xcm_executor::{traits::WeightTrader, AssetsInHolding};

use crate::{Config, LocalCurrencyBalanceOf, SwitchPair, SwitchPairInfoOf};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

const LOG_TARGET: &str = "xcm::pallet-asset-switch::UsingComponentsForSwitchPairRemoteAsset";

/// Type implementing [WeightTrader] that allows paying for XCM fees when
/// reserve transferring the remote asset of the on-chain switch pair.
///
/// This trader is required in case there is no other mechanism to pay for
/// fees when transferring such an asset to this chain.
///
/// Any unused fee is transferred from the switch pair pool account to the
/// specified account.
#[derive(Default, Debug, Clone)]
pub struct UsingComponentsForSwitchPairRemoteAsset<T, I, WeightToFee, FeeDestinationAccount>
where
	T: Config<I>,
	I: 'static,
	FeeDestinationAccount: Get<T::AccountId>,
{
	remaining_weight: Weight,
	remaining_fungible_balance: u128,
	consumed_xcm_hash: Option<XcmHash>,
	switch_pair: Option<SwitchPairInfoOf<T>>,
	_phantom: PhantomData<(WeightToFee, I, FeeDestinationAccount)>,
}

impl<T, I, WeightToFee, FeeDestinationAccount> PartialEq
	for UsingComponentsForSwitchPairRemoteAsset<T, I, WeightToFee, FeeDestinationAccount>
where
	T: Config<I>,
	I: 'static,
	FeeDestinationAccount: Get<T::AccountId>,
{
	fn eq(&self, other: &Self) -> bool {
		self.remaining_weight == other.remaining_weight
			&& self.remaining_fungible_balance == other.remaining_fungible_balance
			&& self.consumed_xcm_hash == other.consumed_xcm_hash
			&& self.switch_pair == other.switch_pair
	}
}

impl<T, I, WeightToFee, FeeDestinationAccount> WeightTrader
	for UsingComponentsForSwitchPairRemoteAsset<T, I, WeightToFee, FeeDestinationAccount>
where
	T: Config<I>,
	I: 'static,
	FeeDestinationAccount: Get<T::AccountId>,

	WeightToFee: WeightToFeeT<Balance = u128>,
{
	fn new() -> Self {
		let switch_pair = SwitchPair::<T, I>::get();
		Self {
			consumed_xcm_hash: None,
			remaining_fungible_balance: Zero::zero(),
			remaining_weight: Zero::zero(),
			switch_pair,
			_phantom: PhantomData,
		}
	}

	fn buy_weight(
		&mut self,
		weight: Weight,
		payment: AssetsInHolding,
		context: &XcmContext,
	) -> Result<AssetsInHolding, Error> {
		log::info!(
			target: LOG_TARGET,
			"buy_weight {:?}, {:?}, {:?}",
			weight,
			payment,
			context
		);

		// Prevent re-using the same trader more than once.
		ensure!(self.consumed_xcm_hash.is_none(), Error::NotWithdrawable);
		// Asset not relevant if no switch pair is set or if not enabled.
		let switch_pair = self.switch_pair.as_ref().ok_or(Error::AssetNotFound)?;
		ensure!(switch_pair.is_enabled(), Error::AssetNotFound);

		let amount = WeightToFee::weight_to_fee(&weight);

		let switch_pair_remote_asset_v4: AssetId = switch_pair.remote_asset_id.clone().try_into().map_err(|e| {
			log::error!(
				target: LOG_TARGET,
				"Failed to convert stored asset ID {:?} into v4 AssetId with error {:?}",
				switch_pair.remote_asset_id,
				e
			);
			Error::FailedToTransactAsset("Failed to convert switch pair asset ID into required version.")
		})?;

		let required: Asset = (switch_pair_remote_asset_v4, amount).into();
		let unused = payment.checked_sub(required.clone()).map_err(|_| Error::TooExpensive)?;

		// Set link to XCM message ID only if this is the trader used.
		log::trace!(target: LOG_TARGET, "Required {:?} - unused {:?}", required, unused);
		self.consumed_xcm_hash = Some(context.message_id);
		self.remaining_fungible_balance = self.remaining_fungible_balance.saturating_add(amount);
		self.remaining_weight = self.remaining_weight.saturating_add(weight);

		Ok(unused)
	}

	fn refund_weight(&mut self, weight: Weight, context: &XcmContext) -> Option<Asset> {
		log::trace!(
			target: LOG_TARGET,
			"UsingComponents::refund_weight weight: {:?}, context: {:?}",
			weight,
			context
		);

		// Ensure we refund in the same trader we took fees from.
		if Some(context.message_id) != self.consumed_xcm_hash {
			return None;
		};

		let Some(ref switch_pair) = self.switch_pair else {
			log::error!(target: LOG_TARGET, "Stored switch pair should not be None, but it is.");
			return None;
		};
		if !switch_pair.is_enabled() {
			return None;
		}

		let switch_pair_remote_asset_v4: AssetId = switch_pair
			.remote_asset_id
			.clone()
			.try_into()
			.map_err(|e| {
				log::error!(
					target: LOG_TARGET,
					"Failed to convert stored asset ID {:?} into v4 AssetId with error {:?}",
					switch_pair.remote_asset_id,
					e
				);
				Error::FailedToTransactAsset("Failed to convert switch pair asset ID into required version.")
			})
			.ok()?;

		let weight_to_refund: Weight = weight.min(self.remaining_weight);
		let amount_for_weight_to_refund = WeightToFee::weight_to_fee(&weight_to_refund);
		// We can only refund up to the remaining balance of this weigher.
		let amount_to_refund = amount_for_weight_to_refund.min(self.remaining_fungible_balance);

		self.consumed_xcm_hash = None;
		self.remaining_fungible_balance = self.remaining_fungible_balance.saturating_sub(amount_to_refund);
		self.remaining_weight = self.remaining_weight.saturating_sub(weight_to_refund);

		if amount_to_refund > 0 {
			log::trace!(
				target: LOG_TARGET,
				"Refund amount {:?}",
				(switch_pair_remote_asset_v4.clone(), amount_to_refund)
			);
			Some((switch_pair_remote_asset_v4, amount_to_refund).into())
		} else {
			log::trace!(target: LOG_TARGET, "No refund");
			None
		}
	}
}

// Move any unused asset from the switch pool account to the specified account,
// and update the remote balance with the difference since we know we control
// the full amount on the remote location.
impl<T, I, WeightToFee, FeeDestinationAccount> Drop
	for UsingComponentsForSwitchPairRemoteAsset<T, I, WeightToFee, FeeDestinationAccount>
where
	T: Config<I>,
	I: 'static,
	FeeDestinationAccount: Get<T::AccountId>,
{
	fn drop(&mut self) {
		log::trace!(
			target: LOG_TARGET,
			"Drop with remaining {:?}",
			(
				self.consumed_xcm_hash,
				self.remaining_fungible_balance,
				self.remaining_weight,
				&self.switch_pair
			)
		);

		// Nothing to refund if this trader was not called or if the leftover balance is
		// zero.
		if let Some(switch_pair) = &self.switch_pair {
			// We don't care if the pool is enabled, since we're sending all non-refunded
			// weight to the configured destination account (e.g., treasury).
			if self.remaining_fungible_balance > Zero::zero() {
				let Ok(remaining_balance_as_local_currency) = LocalCurrencyBalanceOf::<T, I>::try_from(self.remaining_fungible_balance).map_err(|e| {
					log::error!(target: LOG_TARGET, "Failed to convert remaining balance {:?} to local currency balance", self.remaining_fungible_balance);
					e
				}) else { return; };

				// No error should ever be thrown from inside this block.
				let transfer_result = <T::LocalCurrency as Mutate<T::AccountId>>::transfer(
					&switch_pair.pool_account,
					&FeeDestinationAccount::get(),
					remaining_balance_as_local_currency,
					Preservation::Preserve,
				).map_err(|e| {
					log::error!(target: LOG_TARGET, "Failed to transfer unused balance {:?} from switch pair pool account {:?} to specified account {:?}", remaining_balance_as_local_currency, switch_pair.pool_account, FeeDestinationAccount::get());
					e
				});

				debug_assert!(
					transfer_result.is_ok(),
					"Transferring from pool account to fee destination failed."
				);

				// No error should ever be thrown from inside this block.
				SwitchPair::<T, I>::mutate(|entry| {
					let Some(entry) = entry.as_mut() else {
						log::error!(target: LOG_TARGET, "Stored switch pair should not be None but it is.");
						return;
					};
					entry
						.try_process_incoming_switch(self.remaining_fungible_balance)
						.unwrap_or_else(|_| {
							log::error!(
								target: LOG_TARGET,
								"Failed to increase balance of remote sovereign account due to overflow."
							);
						});
				});
			}
		}
	}
}
