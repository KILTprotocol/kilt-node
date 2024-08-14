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

use frame_support::{ensure, weights::WeightToFee as WeightToFeeT};
use sp_runtime::traits::Zero;
use sp_std::marker::PhantomData;
use xcm::v4::{Asset, Error, Fungibility, Weight, XcmContext, XcmHash};
use xcm_executor::{traits::WeightTrader, AssetsInHolding};

use crate::{Config, SwitchPair};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

const LOG_TARGET: &str = "xcm::pallet-asset-switch::UsingComponentsForXcmFeeAsset";

/// Type implementing [WeightTrader] that allows
/// paying for XCM fees when reserve transferring the XCM fee asset for the
/// on-chain switch pair.
///
/// This trader is required in case there is no other mechanism to pay for
/// fees when transferring such an asset to this chain.
///
/// Currently, this trader treats the XCM fee asset as if it were 1:1 with the
/// local currency asset. For cases where the XCM fee asset is considered of
/// greater value than the local currency, this is typically fine. For the other
/// cases, using this trader is not recommended.
#[derive(Default, Debug, Clone)]
pub struct UsingComponentsForXcmFeeAsset<T, I, WeightToFee>
where
	T: Config<I>,
	I: 'static,
{
	remaining_weight: Weight,
	remaining_fungible_balance: u128,
	consumed_xcm_hash: Option<XcmHash>,
	_phantom: PhantomData<(T, I, WeightToFee)>,
}

impl<T, I, WeightToFee> PartialEq for UsingComponentsForXcmFeeAsset<T, I, WeightToFee>
where
	T: Config<I>,
	I: 'static,
{
	fn eq(&self, other: &Self) -> bool {
		self.remaining_weight == other.remaining_weight
			&& self.remaining_fungible_balance == other.remaining_fungible_balance
			&& self.consumed_xcm_hash == other.consumed_xcm_hash
	}
}

impl<T, I, WeightToFee> WeightTrader for UsingComponentsForXcmFeeAsset<T, I, WeightToFee>
where
	T: Config<I>,
	I: 'static,

	WeightToFee: WeightToFeeT<Balance = u128>,
{
	fn new() -> Self {
		Self {
			consumed_xcm_hash: None,
			remaining_fungible_balance: Zero::zero(),
			remaining_weight: Zero::zero(),
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
		// Asset not relevant if no switch pair is set, or not enabled.
		let switch_pair = SwitchPair::<T, I>::get().ok_or(Error::AssetNotFound)?;
		ensure!(switch_pair.is_enabled(), Error::AssetNotFound);

		let amount = WeightToFee::weight_to_fee(&weight);

		let xcm_fee_asset_v4: Asset = switch_pair.remote_xcm_fee.clone().try_into().map_err(|e| {
			log::error!(
				target: LOG_TARGET,
				"Failed to convert stored asset ID {:?} into v4 Asset with error {:?}",
				switch_pair.remote_xcm_fee,
				e
			);
			Error::FailedToTransactAsset("Failed to convert switch pair asset ID into required version.")
		})?;
		// Asset not relevant if the stored XCM fee asset is not fungible.
		let Fungibility::Fungible(_) = xcm_fee_asset_v4.fun else {
			log::info!(target: LOG_TARGET, "Stored XCM fee asset is not fungible.");
			return Err(Error::AssetNotFound);
		};

		let required: Asset = (xcm_fee_asset_v4.id, amount).into();
		let unused = payment.checked_sub(required.clone()).map_err(|_| Error::TooExpensive)?;

		// Set link to XCM message ID only if this is the trader used.
		log::trace!(target: LOG_TARGET, "Required {:?} - unused {:?}", required, unused);
		self.consumed_xcm_hash = Some(context.message_id);
		self.remaining_fungible_balance = self.remaining_fungible_balance.saturating_add(amount);
		self.remaining_weight = self.remaining_weight.saturating_add(weight);

		Ok(unused)
	}

	fn refund_weight(&mut self, weight: Weight, context: &XcmContext) -> Option<Asset> {
		log::info!(target: LOG_TARGET, "refund_weight weight: {:?} {:?}", weight, context);

		// Ensure we refund in the same trader we took fees from.
		if Some(context.message_id) != self.consumed_xcm_hash {
			return None;
		};

		let Some(switch_pair) = SwitchPair::<T, I>::get() else {
			log::error!(target: LOG_TARGET, "Stored switch pair should not be None, but it is.");
			return None;
		};
		if !switch_pair.is_enabled() {
			return None;
		}

		let xcm_fee_asset_v4: Asset = switch_pair
			.remote_xcm_fee
			.clone()
			.try_into()
			.map_err(|e| {
				log::error!(
					target: LOG_TARGET,
					"Failed to convert stored asset ID {:?} into v4 AssetId with error {:?}",
					switch_pair.remote_xcm_fee,
					e
				);
				e
			})
			.ok()?;
		// Double check the store asset fungibility type, in case it changes between
		// weight purchase and weight refund.
		let Fungibility::Fungible(_) = xcm_fee_asset_v4.fun else {
			log::info!(target: LOG_TARGET, "Stored XCM fee asset is not fungible.");
			return None;
		};

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
				(xcm_fee_asset_v4.clone().id, amount_to_refund)
			);

			Some((xcm_fee_asset_v4.id, amount_to_refund).into())
		} else {
			log::trace!(target: LOG_TARGET, "No refund");
			None
		}
	}
}

// We burn whatever surplus we have since we know we control it at destination.
impl<T, I, WeightToFee> Drop for UsingComponentsForXcmFeeAsset<T, I, WeightToFee>
where
	T: Config<I>,
	I: 'static,
{
	fn drop(&mut self) {
		log::trace!(
			target: LOG_TARGET,
			"Drop with remaining {:?}",
			(
				self.consumed_xcm_hash,
				self.remaining_fungible_balance,
				self.remaining_weight
			)
		);
	}
}
