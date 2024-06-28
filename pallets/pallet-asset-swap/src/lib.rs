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

#![cfg_attr(not(feature = "std"), no_std)]

pub mod traits;
pub mod xcm;

mod swap;
pub use swap::{SwapPairInfo, SwapPairStatus};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

use ::xcm::{VersionedAssetId, VersionedMultiAsset, VersionedMultiLocation};
use parity_scale_codec::{Decode, Encode};
use sp_runtime::traits::TrailingZeroInput;
use sp_std::boxed::Box;

pub use crate::pallet::*;

const LOG_TARGET: &str = "runtime::pallet-asset-swap";

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use crate::{
		swap::{SwapPairInfo, SwapPairStatus},
		traits::SwapHooks,
		LOG_TARGET,
	};

	use frame_support::{
		pallet_prelude::*,
		traits::{
			fungible::{Inspect as InspectFungible, Mutate as MutateFungible},
			tokens::{Fortitude, Preservation, Provenance},
			EnsureOrigin,
		},
	};
	use frame_system::{ensure_root, pallet_prelude::*};
	use sp_runtime::traits::TryConvert;
	use sp_std::{boxed::Box, vec};
	use xcm::{
		v3::{
			validate_send, AssetId,
			Instruction::{BuyExecution, DepositAsset, RefundSurplus, SetAppendix, TransferAsset, WithdrawAsset},
			Junction, MultiAsset, MultiAssetFilter, MultiLocation, SendXcm, WeightLimit, WildMultiAsset, Xcm,
		},
		VersionedAssetId, VersionedMultiAsset, VersionedMultiLocation,
	};
	use xcm_executor::traits::TransactAsset;

	pub type LocalCurrencyBalanceOf<T> =
		<<T as Config>::LocalCurrency as InspectFungible<<T as frame_system::Config>::AccountId>>::Balance;
	pub type SwapPairInfoOf<T> = SwapPairInfo<<T as frame_system::Config>::AccountId>;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		const POOL_ADDRESS_GENERATION_ENTROPY: [u8; 8];

		type AccountIdConverter: TryConvert<Self::AccountId, Junction>;
		type AssetTransactor: TransactAsset;
		type FeeOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		type LocalCurrency: MutateFungible<Self::AccountId>;
		type PauseOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type SubmitterOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Self::AccountId>;
		type SwapHooks: SwapHooks<Self>;
		type SwapOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		type XcmRouter: SendXcm;
	}

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		SwapPairCreated {
			circulating_supply: u128,
			pool_account: T::AccountId,
			remote_asset_id: VersionedAssetId,
			remote_asset_reserve_location: VersionedMultiLocation,
			remote_xcm_fee: Box<VersionedMultiAsset>,
			total_issuance: u128,
		},
		SwapPairRemoved {
			remote_asset_id: VersionedAssetId,
		},
		SwapPairResumed {
			remote_asset_id: VersionedAssetId,
		},
		SwapPairPaused {
			remote_asset_id: VersionedAssetId,
		},
		SwapPairFeeUpdated {
			old: VersionedMultiAsset,
			new: VersionedMultiAsset,
		},
		LocalToRemoteSwapExecuted {
			from: T::AccountId,
			to: VersionedMultiLocation,
			amount: LocalCurrencyBalanceOf<T>,
		},
		RemoteToLocalSwapExecuted {
			to: T::AccountId,
			amount: u128,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidInput,
		Hook(u8),
		Liquidity,
		LocalPoolBalance,
		PoolInitialLiquidityRequirement,
		SwapPairAlreadyExisting,
		SwapPairNotEnabled,
		SwapPairNotFound,
		UserSwapBalance,
		UserXcmBalance,
		Xcm,
		Internal,
	}

	#[pallet::storage]
	#[pallet::getter(fn swap_pair)]
	pub(crate) type SwapPair<T> = StorageValue<_, SwapPairInfoOf<T>, OptionQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		LocalCurrencyBalanceOf<T>: Into<u128>,
	{
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
		pub fn set_swap_pair(
			origin: OriginFor<T>,
			reserve_location: Box<VersionedMultiLocation>,
			remote_asset_id: Box<VersionedAssetId>,
			remote_fee: Box<VersionedMultiAsset>,
			total_issuance: u128,
			circulating_supply: u128,
		) -> DispatchResult {
			T::SwapOrigin::ensure_origin(origin)?;

			// 1. Verify swap pair has not already been set.
			ensure!(!SwapPair::<T>::exists(), Error::<T>::SwapPairAlreadyExisting);

			// 2. Verify that total issuance >= circulating supply.
			ensure!(total_issuance >= circulating_supply, Error::<T>::InvalidInput);

			// 3. Verify the pool account has enough local assets to match the circulating
			//    supply of eKILTs to cover for all potential remote -> local swaps.
			let pool_account = Self::pool_account_id_for_remote_asset(&remote_asset_id)?;
			let pool_account_reducible_balance_as_u128: u128 =
				T::LocalCurrency::reducible_balance(&pool_account, Preservation::Expendable, Fortitude::Polite).into();
			ensure!(
				pool_account_reducible_balance_as_u128 >= circulating_supply,
				Error::<T>::PoolInitialLiquidityRequirement
			);

			Self::set_swap_pair_bypass_checks(
				*reserve_location,
				*remote_asset_id,
				*remote_fee,
				total_issuance,
				circulating_supply,
				pool_account,
			);

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		pub fn force_set_swap_pair(
			origin: OriginFor<T>,
			reserve_location: Box<VersionedMultiLocation>,
			remote_asset_id: Box<VersionedAssetId>,
			remote_fee: Box<VersionedMultiAsset>,
			total_issuance: u128,
			circulating_supply: u128,
		) -> DispatchResult {
			ensure_root(origin)?;

			ensure!(total_issuance >= circulating_supply, Error::<T>::InvalidInput);
			let pool_account = Self::pool_account_id_for_remote_asset(&remote_asset_id)?;

			Self::set_swap_pair_bypass_checks(
				*reserve_location,
				*remote_asset_id,
				*remote_fee,
				total_issuance,
				circulating_supply,
				pool_account,
			);

			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(0)]
		pub fn force_unset_swap_pair(origin: OriginFor<T>) -> DispatchResult {
			ensure_root(origin)?;

			Self::unset_swap_pair_bypass_checks();

			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(0)]
		pub fn pause_swap_pair(origin: OriginFor<T>) -> DispatchResult {
			T::PauseOrigin::ensure_origin(origin)?;

			Self::set_swap_pair_status(SwapPairStatus::Paused)?;

			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(0)]
		pub fn resume_swap_pair(origin: OriginFor<T>) -> DispatchResult {
			T::SwapOrigin::ensure_origin(origin)?;

			Self::set_swap_pair_status(SwapPairStatus::Running)?;

			Ok(())
		}

		#[pallet::call_index(5)]
		#[pallet::weight(0)]
		pub fn update_remote_fee(origin: OriginFor<T>, new: Box<VersionedMultiAsset>) -> DispatchResult {
			T::FeeOrigin::ensure_origin(origin)?;

			SwapPair::<T>::try_mutate(|entry| {
				let SwapPairInfoOf::<T> { remote_fee, .. } = entry.as_mut().ok_or(Error::<T>::SwapPairNotFound)?;
				let old_remote_fee = remote_fee.clone();
				*remote_fee = *new.clone();
				if old_remote_fee != *new {
					Self::deposit_event(Event::<T>::SwapPairFeeUpdated {
						old: old_remote_fee,
						new: *new,
					});
				};
				Ok::<_, Error<T>>(())
			})?;

			Ok(())
		}

		#[pallet::call_index(6)]
		#[pallet::weight(0)]
		pub fn swap(
			origin: OriginFor<T>,
			local_asset_amount: LocalCurrencyBalanceOf<T>,
			beneficiary: Box<VersionedMultiLocation>,
		) -> DispatchResult {
			let submitter = T::SubmitterOrigin::ensure_origin(origin)?;

			// 1. Retrieve swap pair info from storage, else fail.
			let swap_pair = SwapPair::<T>::get().ok_or(DispatchError::from(Error::<T>::SwapPairNotFound))?;

			// 2. Check if swaps are enabled.
			ensure!(
				swap_pair.can_swap(),
				DispatchError::from(Error::<T>::SwapPairNotEnabled)
			);

			// 3. Verify the tx submitter has enough local assets for the swap, without
			//    having their balance go to zero.
			T::LocalCurrency::can_withdraw(&submitter, local_asset_amount)
				.into_result(true)
				.map_err(|_| DispatchError::from(Error::<T>::UserSwapBalance))?;

			// 4. Verify the local assets can be transferred to the swap pool account
			T::LocalCurrency::can_deposit(&swap_pair.pool_account, local_asset_amount, Provenance::Extant)
				.into_result()
				.map_err(|_| DispatchError::from(Error::<T>::LocalPoolBalance))?;

			// 5. Verify we have enough balance on the remote location to perform the
			//    transfer
			let remote_asset_amount_as_u128 = local_asset_amount.into();
			ensure!(
				swap_pair.remote_asset_balance >= remote_asset_amount_as_u128,
				Error::<T>::Liquidity
			);

			let asset_id_v3: AssetId = swap_pair.remote_asset_id.clone().try_into().map_err(|e| {
				log::error!(
					target: LOG_TARGET,
					"Failed to convert asset ID {:?} into v3 `AssetId` with error {:?}",
					swap_pair.remote_asset_id,
					e
				);
				DispatchError::from(Error::<T>::Internal)
			})?;
			let remote_asset_fee_v3: MultiAsset = swap_pair.remote_fee.clone().try_into().map_err(|e| {
				log::error!(
					target: LOG_TARGET,
					"Failed to convert remote XCM asset fee {:?} into v3 `MultiAssset` with error {:?}",
					swap_pair.remote_fee,
					e
				);
				DispatchError::from(Error::<T>::Xcm)
			})?;
			let destination_v3: MultiLocation = swap_pair.remote_reserve_location.clone().try_into().map_err(|e| {
				log::error!(
					target: LOG_TARGET,
					"Failed to convert remote reserve location {:?} into v3 `MultiLocation` with error {:?}",
					swap_pair.remote_reserve_location,
					e
				);
				DispatchError::from(Error::<T>::Internal)
			})?;
			let beneficiary_v3: MultiLocation = (*beneficiary.clone()).try_into().map_err(|e| {
				log::info!(
					target: LOG_TARGET,
					"Failed to convert beneficiary {:?} into v3 `MultiLocation` with error {:?}",
					beneficiary,
					e
				);
				DispatchError::from(Error::<T>::Xcm)
			})?;
			// Use the same local `AccountIdConverter` to generate a `MultiLocation` to use
			// to send funds on remote.
			let submitter_as_multilocation = T::AccountIdConverter::try_convert(submitter.clone())
				.map(|j| j.into_location())
				.map_err(|e| {
					log::info!(
						target: LOG_TARGET,
						"Failed to convert account {:?} into `MultiLocation` with error {:?}",
						submitter,
						e
					);
					DispatchError::from(Error::<T>::Xcm)
				})?;

			// 6. Compose and validate XCM message
			let appendix: Xcm<()> = vec![
				RefundSurplus,
				DepositAsset {
					assets: MultiAssetFilter::Wild(WildMultiAsset::All),
					beneficiary: submitter_as_multilocation,
				},
			]
			.into();
			let remote_xcm: Xcm<()> = vec![
				WithdrawAsset(remote_asset_fee_v3.clone().into()),
				BuyExecution {
					weight_limit: WeightLimit::Unlimited,
					fees: remote_asset_fee_v3.clone(),
				},
				TransferAsset {
					assets: (asset_id_v3, remote_asset_amount_as_u128).into(),
					beneficiary: beneficiary_v3,
				},
				SetAppendix(appendix),
			]
			.into();
			let xcm_ticket = validate_send::<T::XcmRouter>(destination_v3, remote_xcm.clone()).map_err(|e| {
				log::info!(
					"Failed to call validate_send for destination {:?} and remote XCM {:?} with error {:?}",
					destination_v3,
					remote_xcm,
					e
				);
				DispatchError::from(Error::<T>::Xcm)
			})?;

			// 7. Call into hook pre-swap checks
			T::SwapHooks::pre_local_to_remote_swap(&submitter, &beneficiary, local_asset_amount)
				.map_err(|e| DispatchError::from(Error::<T>::Hook(e.into())))?;

			// 8. Transfer funds from user to pool
			let transferred_amount = T::LocalCurrency::transfer(
				&submitter,
				&swap_pair.pool_account,
				local_asset_amount,
				Preservation::Preserve,
			)?;
			if transferred_amount != local_asset_amount {
				log::error!(
					"Transferred amount {:?} does not match expected user-specified amount {:?}",
					transferred_amount,
					local_asset_amount
				);
				return Err(Error::<T>::Internal.into());
			}

			// 9. Take XCM fee from submitter.
			let withdrawn_fees =
				T::AssetTransactor::withdraw_asset(&remote_asset_fee_v3, &submitter_as_multilocation, None).map_err(
					|e| {
						log::info!(
							target: LOG_TARGET,
							"Failed to withdraw asset {:?} from location {:?} with error {:?}",
							remote_asset_fee_v3,
							submitter_as_multilocation,
							e
						);
						DispatchError::from(Error::<T>::UserXcmBalance)
					},
				)?;
			if withdrawn_fees != vec![remote_asset_fee_v3.clone()].into() {
				log::error!(
					target: LOG_TARGET,
					"Withdrawn fees {:?} does not match expected fee {:?}.",
					withdrawn_fees,
					remote_asset_fee_v3
				);
				return Err(DispatchError::from(Error::<T>::Internal));
			}

			// 10. Send XCM out
			T::XcmRouter::deliver(xcm_ticket.0).map_err(|e| {
				log::info!("Failed to deliver ticket with error {:?}", e);
				DispatchError::from(Error::<T>::Xcm)
			})?;

			// 11. Update remote asset balance
			SwapPair::<T>::try_mutate(|entry| {
				let Some(SwapPairInfoOf::<T> {
					remote_asset_balance, ..
				}) = entry.as_mut()
				else {
					log::error!(target: LOG_TARGET, "Failed to borrow stored swap pair info as mut.");
					return Err(Error::<T>::Internal);
				};
				let Some(new_balance) = remote_asset_balance.checked_sub(remote_asset_amount_as_u128) else {
					log::error!(target: LOG_TARGET, "Failed to subtract {:?} from stored remote balance {:?}.", transferred_amount, remote_asset_balance);
					return Err(Error::<T>::Internal);
				};
				*remote_asset_balance = new_balance;
				Ok(())
			})?;

			// 12. Call into hook post-swap checks
			T::SwapHooks::post_local_to_remote_swap(&submitter, &beneficiary, local_asset_amount)
				.map_err(|e| DispatchError::from(Error::<T>::Hook(e.into())))?;

			Self::deposit_event(Event::<T>::LocalToRemoteSwapExecuted {
				from: submitter,
				to: *beneficiary,
				amount: local_asset_amount,
			});

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn set_swap_pair_bypass_checks(
		reserve_location: VersionedMultiLocation,
		remote_asset_id: VersionedAssetId,
		remote_fee: VersionedMultiAsset,
		total_issuance: u128,
		circulating_supply: u128,
		pool_account: T::AccountId,
	) {
		debug_assert!(
			total_issuance >= circulating_supply,
			"Provided total issuance smaller than circulating supply."
		);
		let swap_pair_info = SwapPairInfoOf::<T> {
			pool_account: pool_account.clone(),
			// We can do a simple subtraction since all checks are performed in calling functions.
			remote_asset_balance: total_issuance - circulating_supply,
			remote_asset_id: remote_asset_id.clone(),
			remote_fee: remote_fee.clone(),
			remote_reserve_location: reserve_location.clone(),
			status: SwapPairStatus::Paused,
		};

		SwapPair::<T>::set(Some(swap_pair_info));

		Self::deposit_event(Event::<T>::SwapPairCreated {
			circulating_supply,
			pool_account,
			remote_asset_reserve_location: reserve_location,
			remote_asset_id,
			remote_xcm_fee: Box::new(remote_fee),
			total_issuance,
		});
	}

	fn unset_swap_pair_bypass_checks() {
		let swap_pair = SwapPair::<T>::take();
		if let Some(swap_pair) = swap_pair {
			Self::deposit_event(Event::<T>::SwapPairRemoved {
				remote_asset_id: swap_pair.remote_asset_id,
			});
		};
	}

	fn set_swap_pair_status(new_status: SwapPairStatus) -> Result<(), Error<T>> {
		SwapPair::<T>::try_mutate(|entry| {
			let SwapPairInfoOf::<T> {
				remote_asset_id,
				status,
				..
			} = entry.as_mut().ok_or(Error::<T>::SwapPairNotFound)?;
			let relevant_event = match new_status {
				SwapPairStatus::Running => Event::<T>::SwapPairResumed {
					remote_asset_id: remote_asset_id.clone(),
				},
				SwapPairStatus::Paused => Event::<T>::SwapPairPaused {
					remote_asset_id: remote_asset_id.clone(),
				},
			};
			let old_status = status.clone();
			*status = new_status;
			// If state was actually changed, generate an event, otherwise this is a no-op.
			if old_status != *status {
				Self::deposit_event(relevant_event);
			}
			Ok::<_, Error<T>>(())
		})?;
		Ok(())
	}
}

impl<T: Config> Pallet<T> {
	pub fn pool_account_id_for_remote_asset(remote_asset_id: &VersionedAssetId) -> Result<T::AccountId, Error<T>> {
		let hash_input = (T::POOL_ADDRESS_GENERATION_ENTROPY, b'.', remote_asset_id.clone()).encode();
		let hash_output = sp_io::hashing::blake2_256(hash_input.as_slice());
		T::AccountId::decode(&mut TrailingZeroInput::new(hash_output.as_slice())).map_err(|e| {
			log::error!(
				target: LOG_TARGET,
				"Failed to generate pool ID from remote asset {:?} with error: {:?}",
				remote_asset_id,
				e
			);
			Error::<T>::Internal
		})
	}
}
