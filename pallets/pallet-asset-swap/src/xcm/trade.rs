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

pub use xcm_fee_asset::UsingComponentsForXcmFeeAsset;
mod xcm_fee_asset {
	use frame_support::{ensure, weights::WeightToFee as WeightToFeeT};
	use sp_runtime::traits::Zero;
	use sp_std::marker::PhantomData;
	use xcm::v3::{AssetId, Error, MultiAsset, Weight, XcmContext, XcmHash};
	use xcm_executor::{traits::WeightTrader, Assets};

	use crate::{Config, SwapPair};

	const LOG_TARGET: &str = "xcm::pallet-asset-swap::UsingComponentsForXcmFeeAsset";

	/// Type implementing `WeightTrader` that allows to pay for XCM fees when
	/// reserve transferring the XCM fee asset for the on-chain swap pair.
	///
	/// This trader is required in case there is no other mechanism to pay for
	/// fees when transferring such an asset to this chain.
	pub struct UsingComponentsForXcmFeeAsset<T, WeightToFee>
	where
		T: Config,
	{
		remaining_weight: Weight,
		remaining_fungible_balance: u128,
		consumed_xcm_hash: Option<XcmHash>,
		_phantom: PhantomData<(T, WeightToFee)>,
	}

	impl<T, WeightToFee> WeightTrader for UsingComponentsForXcmFeeAsset<T, WeightToFee>
	where
		T: Config,

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

		fn buy_weight(&mut self, weight: Weight, payment: Assets, context: &XcmContext) -> Result<Assets, Error> {
			log::info!(target: LOG_TARGET, "buy_weight {:?}, {:?}, {:?}", weight, payment, context);

			// Prevent re-using the same trader more than once.
			ensure!(self.consumed_xcm_hash.is_none(), Error::NotWithdrawable);
			// Asset not relevant if no swap pair is set.
			let swap_pair = SwapPair::<T>::get().ok_or(Error::AssetNotFound)?;

			let amount = WeightToFee::weight_to_fee(&weight);

			let xcm_fee_asset_v3: MultiAsset = swap_pair.remote_fee.clone().try_into().map_err(|e| {
				log::error!(target: LOG_TARGET, "Failed to convert stored asset ID {:?} into v3 MultiAsset with error {:?}", swap_pair.remote_fee, e);
				Error::FailedToTransactAsset("Failed to convert swap pair asset ID into required version.")
			})?;

			let required: MultiAsset = (xcm_fee_asset_v3.id, amount).into();
			let unused = payment.checked_sub(required.clone()).map_err(|_| Error::TooExpensive)?;

			// Set link to XCM message ID only if this is the trader used.
			log::trace!(target: LOG_TARGET, "Required {:?} - unused {:?}", required, unused);
			self.consumed_xcm_hash = Some(context.message_id);
			self.remaining_fungible_balance = self.remaining_fungible_balance.saturating_add(amount);
			self.remaining_weight = self.remaining_weight.saturating_add(weight);

			Ok(unused)
		}

		fn refund_weight(&mut self, weight: Weight, context: &XcmContext) -> Option<MultiAsset> {
			log::info!(target: LOG_TARGET, "refund_weight weight: {:?} {:?}", weight, context);

			// Ensure we refund in the same trader we took fees from.
			if Some(context.message_id) != self.consumed_xcm_hash {
				return None;
			};

			let Some(swap_pair) = SwapPair::<T>::get() else {
				log::error!(target: LOG_TARGET, "Stored swap pair should not be None, but it is.");
				return None;
			};

			let remote_asset_id_v3: AssetId = swap_pair.remote_asset_id.clone().try_into().map_err(|e| {
				log::error!(target: LOG_TARGET, "Failed to convert stored asset ID {:?} into v3 AssetId with error {:?}", swap_pair.remote_asset_id, e);
				e
			}).ok()?;

			let weight = weight.min(self.remaining_weight);
			let amount = WeightToFee::weight_to_fee(&weight);

			self.consumed_xcm_hash = None;
			self.remaining_fungible_balance = self
				.remaining_fungible_balance
				.saturating_sub(self.remaining_fungible_balance);
			self.remaining_weight = self.remaining_weight.saturating_sub(weight);

			if amount > 0 {
				log::trace!(target: LOG_TARGET, "Refund amount {:?}", (remote_asset_id_v3, amount));
				Some((remote_asset_id_v3, amount).into())
			} else {
				log::trace!(target: LOG_TARGET, "No refund");
				None
			}
		}
	}

	// We burn whatever surplus we have since we know we control it at destination.
	impl<T, WeightToFee> Drop for UsingComponentsForXcmFeeAsset<T, WeightToFee>
	where
		T: Config,
	{
		fn drop(&mut self) {
			log::trace!(target: LOG_TARGET, "Drop with remaining {:?}", (self.consumed_xcm_hash, self.remaining_fungible_balance, self.remaining_weight));
		}
	}
}

pub use swap_pair_remote_asset::UsingComponentsForSwapPairRemoteAsset;
mod swap_pair_remote_asset {
	use frame_support::{
		ensure,
		traits::{fungible::Mutate, tokens::Preservation},
		weights::WeightToFee as WeightToFeeT,
	};
	use sp_core::Get;
	use sp_runtime::traits::Zero;
	use sp_std::marker::PhantomData;
	use xcm::v3::{AssetId, Error, MultiAsset, Weight, XcmContext, XcmHash};
	use xcm_executor::{traits::WeightTrader, Assets};

	use crate::{Config, LocalCurrencyBalanceOf, SwapPair, SwapPairInfoOf};

	const LOG_TARGET: &str = "xcm::pallet-asset-swap::UsingComponentsForSwapPairRemoteAsset";

	/// Type implementing `WeightTrader` that allows to pay for XCM fees when
	/// reserve transferring the remote asset of the on-chain swap pair.
	///
	/// This trader is required in case there is no other mechanism to pay for
	/// fees when transferring such an asset to this chain.
	///
	/// Any unused fee is transferred from the swap pair pool account to the
	/// specified account.
	#[derive(Default)]
	pub struct UsingComponentsForSwapPairRemoteAsset<T, WeightToFee, FeeDestinationAccount>
	where
		T: Config,
		FeeDestinationAccount: Get<T::AccountId>,
	{
		remaining_weight: Weight,
		remaining_fungible_balance: u128,
		consumed_xcm_hash: Option<XcmHash>,
		swap_pair: Option<SwapPairInfoOf<T>>,
		_phantom: PhantomData<(WeightToFee, FeeDestinationAccount)>,
	}

	impl<T, WeightToFee, FeeDestinationAccount> WeightTrader
		for UsingComponentsForSwapPairRemoteAsset<T, WeightToFee, FeeDestinationAccount>
	where
		T: Config,
		FeeDestinationAccount: Get<T::AccountId>,

		WeightToFee: WeightToFeeT<Balance = u128>,
	{
		fn new() -> Self {
			let swap_pair = SwapPair::<T>::get();
			Self {
				consumed_xcm_hash: None,
				remaining_fungible_balance: Zero::zero(),
				remaining_weight: Zero::zero(),
				swap_pair,
				_phantom: PhantomData,
			}
		}

		fn buy_weight(&mut self, weight: Weight, payment: Assets, context: &XcmContext) -> Result<Assets, Error> {
			log::info!(target: LOG_TARGET, "buy_weight {:?}, {:?}, {:?}", weight, payment, context);

			// Prevent re-using the same trader more than once.
			ensure!(self.consumed_xcm_hash.is_none(), Error::NotWithdrawable);
			// Asset not relevant if no swap pair is set.
			let swap_pair = self.swap_pair.as_ref().ok_or(Error::AssetNotFound)?;

			let amount = WeightToFee::weight_to_fee(&weight);

			let swap_pair_remote_asset_v3: AssetId = swap_pair.remote_asset_id.clone().try_into().map_err(|e| {
				log::error!(target: LOG_TARGET, "Failed to convert stored asset ID {:?} into v3 AssetId with error {:?}", swap_pair.remote_asset_id, e);
				Error::FailedToTransactAsset("Failed to convert swap pair asset ID into required version.")
			})?;

			let required: MultiAsset = (swap_pair_remote_asset_v3, amount).into();
			let unused = payment.checked_sub(required.clone()).map_err(|_| Error::TooExpensive)?;

			// Set link to XCM message ID only if this is the trader used.
			log::trace!(target: LOG_TARGET, "Required {:?} - unused {:?}", required, unused);
			self.consumed_xcm_hash = Some(context.message_id);
			self.remaining_fungible_balance = self.remaining_fungible_balance.saturating_add(amount);
			self.remaining_weight = self.remaining_weight.saturating_add(weight);

			Ok(unused)
		}

		fn refund_weight(&mut self, weight: Weight, context: &XcmContext) -> Option<MultiAsset> {
			log::trace!(target: LOG_TARGET, "UsingComponents::refund_weight weight: {:?}, context: {:?}", weight, context);

			// Ensure we refund in the same trader we took fees from.
			if Some(context.message_id) != self.consumed_xcm_hash {
				return None;
			};

			let Some(ref swap_pair) = self.swap_pair else {
				log::error!(target: LOG_TARGET, "Stored swap pair should not be None, but it is.");
				return None;
			};

			let swap_pair_remote_asset_v3: AssetId = swap_pair.remote_asset_id.clone().try_into().map_err(|e| {
				log::error!(target: LOG_TARGET, "Failed to convert stored asset ID {:?} into v3 AssetId with error {:?}", swap_pair.remote_asset_id, e);
				Error::FailedToTransactAsset("Failed to convert swap pair asset ID into required version.")
			}).ok()?;

			let weight = weight.min(self.remaining_weight);
			let amount = WeightToFee::weight_to_fee(&weight);

			self.consumed_xcm_hash = None;
			self.remaining_fungible_balance = self
				.remaining_fungible_balance
				.saturating_sub(self.remaining_fungible_balance);
			self.remaining_weight = self.remaining_weight.saturating_sub(weight);

			if amount > 0 {
				log::trace!(target: LOG_TARGET, "Refund amount {:?}", (swap_pair_remote_asset_v3, amount));
				Some((swap_pair_remote_asset_v3, amount).into())
			} else {
				log::trace!(target: LOG_TARGET, "No refund");
				None
			}
		}
	}

	// Move any unused asset from the swap pool account to the specified account,
	// and update the remote balance with the difference since we know we control
	// the full amount on the remote location.
	impl<T, WeightToFee, FeeDestinationAccount> Drop
		for UsingComponentsForSwapPairRemoteAsset<T, WeightToFee, FeeDestinationAccount>
	where
		T: Config,
		FeeDestinationAccount: Get<T::AccountId>,
	{
		fn drop(&mut self) {
			log::trace!(target: LOG_TARGET, "Drop with remaining {:?}", (self.consumed_xcm_hash, self.remaining_fungible_balance, self.remaining_weight, &self.swap_pair));
			match (self.remaining_fungible_balance, &self.swap_pair) {
				(remaining_balance, Some(swap_pair)) if remaining_balance > Zero::zero() => {
					let Ok(remaining_balance_as_local_currency) = LocalCurrencyBalanceOf::<T>::try_from(remaining_balance).map_err(|e| {
						log::error!(target: LOG_TARGET, "Failed to convert remaining balance {:?} to local currency balance", remaining_balance);
						e
					}) else { return; };

					// No error should ever be thrown from inside this block.
					let _ = <T::LocalCurrency as Mutate<T::AccountId>>::transfer(
						&swap_pair.pool_account,
						&FeeDestinationAccount::get(),
						remaining_balance_as_local_currency,
						Preservation::Expendable,
					).map_err(|e| {
						log::error!(target: LOG_TARGET, "Failed to transfer unused balance {:?} from swap pair pool account {:?} to specified account {:?}", remaining_balance_as_local_currency, swap_pair.pool_account, FeeDestinationAccount::get());
						e
					});

					// No error should ever be thrown from inside this block.
					SwapPair::<T>::mutate(|entry| {
						let Some(entry) = entry.as_mut() else {
							log::error!(target: LOG_TARGET, "Stored swap pair should not be None but it is.");
							return;
						};
						entry.remote_asset_balance = entry
							.remote_asset_balance
							.saturating_add(self.remaining_fungible_balance);
					});
				}
				_ => {}
			}
		}
	}
}