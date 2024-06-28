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
	use frame_support::{traits::fungibles::Inspect, weights::WeightToFee as WeightToFeeT};
	use sp_runtime::{
		traits::{Saturating, Zero},
		SaturatedConversion,
	};
	use sp_std::marker::PhantomData;
	use xcm::v3::{AssetId, Error, MultiAsset, Weight, XcmContext};
	use xcm_executor::{traits::WeightTrader, Assets};

	use crate::{Config, SwapPair};

	const LOG_TARGET: &str = "xcm::pallet-asset-swap::UsingComponentsForXcmFeeAsset";

	/// Type implementing `WeightTrader` that allows to pay for XCM fees when
	/// reserve transferring the XCM fee asset for the on-chain swap pair.
	///
	/// This trader is required in case there is no other mechanism to pay for
	/// fees when transferring such an asset to this chain.
	pub struct UsingComponentsForXcmFeeAsset<T: frame_system::Config, WeightToFee, Fungibles: Inspect<T::AccountId>>(
		Weight,
		Fungibles::Balance,
		PhantomData<(T, WeightToFee)>,
	);

	impl<T, WeightToFee, Fungibles> WeightTrader for UsingComponentsForXcmFeeAsset<T, WeightToFee, Fungibles>
	where
		T: Config,
		WeightToFee: WeightToFeeT<Balance = Fungibles::Balance>,
		Fungibles: Inspect<T::AccountId>,
	{
		fn new() -> Self {
			Self(Weight::zero(), Zero::zero(), PhantomData)
		}

		fn buy_weight(&mut self, weight: Weight, payment: Assets, context: &XcmContext) -> Result<Assets, Error> {
			log::info!(target: LOG_TARGET, "buy_weight {:?}, {:?}, {:?}", weight, payment, context);
			let swap_pair = SwapPair::<T>::get().ok_or(Error::AssetNotFound)?;
			let amount = WeightToFee::weight_to_fee(&weight);
			let u128_amount: u128 = amount.try_into().map_err(|_| Error::Overflow)?;
			let xcm_fee_asset_v3: MultiAsset = swap_pair.remote_fee.clone().try_into().map_err(|e| {
				log::error!(target: "xcm::weight", "Failed to convert stored asset ID {:?} into v3 MultiAsset with error {:?}", swap_pair.remote_fee, e);
				Error::FailedToTransactAsset("Failed to convert swap pair asset ID into required version.")
			})?;
			let required: MultiAsset = (xcm_fee_asset_v3.id, u128_amount).into();
			let unused = payment.checked_sub(required.clone()).map_err(|_| Error::TooExpensive)?;
			log::trace!(target: LOG_TARGET, "required {:?} - unused {:?}", required, unused);
			self.0 = self.0.saturating_add(weight);
			self.1 = self.1.saturating_add(amount);
			Ok(unused)
		}

		fn refund_weight(&mut self, weight: Weight, context: &XcmContext) -> Option<MultiAsset> {
			log::info!(target: LOG_TARGET, "refund_weight weight: {:?} {:?}", weight, context);
			let swap_pair = SwapPair::<T>::get()?;
			let remote_asset_id_v3: AssetId = swap_pair.remote_asset_id.clone().try_into().map_err(|e| {
				log::error!(target: LOG_TARGET, "Failed to convert stored asset ID {:?} into v3 AssetId with error {:?}", swap_pair.remote_asset_id, e);
				e
			}).ok()?;
			let weight = weight.min(self.0);
			let amount = WeightToFee::weight_to_fee(&weight);
			self.0 -= weight;
			self.1 = self.1.saturating_sub(amount);
			let amount: u128 = amount.saturated_into();
			if amount > 0 {
				log::trace!(target: LOG_TARGET, "refund amount {:?}", (remote_asset_id_v3, amount));
				Some((remote_asset_id_v3, amount).into())
			} else {
				log::trace!(target: LOG_TARGET, "No refund");
				None
			}
		}
	}
}

pub use swap_pair_remote_asset::UsingComponentsForSwapPairRemoteAsset;
mod swap_pair_remote_asset {
	use frame_support::weights::WeightToFee as WeightToFeeT;
	use sp_runtime::{traits::Zero, SaturatedConversion};
	use sp_std::marker::PhantomData;
	use xcm::v3::{AssetId, Error, MultiAsset, Weight, XcmContext};
	use xcm_executor::{traits::WeightTrader, Assets};

	use crate::{Config, SwapPair};

	const LOG_TARGET: &str = "xcm::pallet-asset-swap::UsingComponentsForSwapPairRemoteAsset";

	/// Type implementing `WeightTrader` that allows to pay for XCM fees when
	/// reserve transferring the remote asset of the on-chain swap pair.
	///
	/// This trader is required in case there is no other mechanism to pay for
	/// fees when transferring such an asset to this chain.
	pub struct UsingComponentsForSwapPairRemoteAsset<T: frame_system::Config, WeightToFee>(
		Weight,
		u128,
		PhantomData<(T, WeightToFee)>,
	);

	impl<T, WeightToFee> WeightTrader for UsingComponentsForSwapPairRemoteAsset<T, WeightToFee>
	where
		T: Config,
		WeightToFee: WeightToFeeT<Balance = u128>,
	{
		fn new() -> Self {
			Self(Weight::zero(), Zero::zero(), PhantomData)
		}

		fn buy_weight(&mut self, weight: Weight, payment: Assets, context: &XcmContext) -> Result<Assets, Error> {
			log::info!(target: LOG_TARGET, "buy_weight {:?}, {:?}, {:?}", weight, payment, context);
			let swap_pair = SwapPair::<T>::get().ok_or(Error::AssetNotFound)?;
			let amount = WeightToFee::weight_to_fee(&weight);
			let swap_pair_remote_asset_v3: AssetId = swap_pair.remote_asset_id.clone().try_into().map_err(|e| {
				log::error!(target: LOG_TARGET, "Failed to convert stored asset ID {:?} into v3 AssetId with error {:?}", swap_pair.remote_asset_id, e);
				Error::FailedToTransactAsset("Failed to convert swap pair asset ID into required version.")
			})?;
			let required: MultiAsset = (swap_pair_remote_asset_v3, amount).into();
			let unused = payment.checked_sub(required.clone()).map_err(|_| Error::TooExpensive)?;
			log::trace!(target: LOG_TARGET, "required {:?} - unused {:?}", required, unused);
			self.0 = self.0.saturating_add(weight);
			self.1 = self.1.saturating_add(amount);
			Ok(unused)
		}

		fn refund_weight(&mut self, weight: Weight, context: &XcmContext) -> Option<MultiAsset> {
			log::trace!(target: LOG_TARGET, "UsingComponents::refund_weight weight: {:?}, context: {:?}", weight, context);
			let swap_pair = SwapPair::<T>::get()?;
			let swap_pair_remote_asset_v3: AssetId = swap_pair.remote_asset_id.clone().try_into().map_err(|e| {
				log::error!(target: LOG_TARGET, "Failed to convert stored asset ID {:?} into v3 AssetId with error {:?}", swap_pair.remote_asset_id, e);
				Error::FailedToTransactAsset("Failed to convert swap pair asset ID into required version.")
			}).ok()?;
			let weight = weight.min(self.0);
			let amount = WeightToFee::weight_to_fee(&weight);
			self.0 -= weight;
			self.1 = self.1.saturating_sub(amount);
			let amount: u128 = amount.saturated_into();
			if amount > 0 {
				log::trace!(target: LOG_TARGET, "refund amount {:?}", (swap_pair_remote_asset_v3, amount));
				Some((swap_pair_remote_asset_v3, amount).into())
			} else {
				log::trace!(target: LOG_TARGET, "No refund");
				None
			}
		}
	}
}
