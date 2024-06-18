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

mod swap;
mod xcm;

use ::xcm::{VersionedAssetId, VersionedMultiLocation};
use parity_scale_codec::{Decode, Encode};
use sp_runtime::traits::TrailingZeroInput;

pub use crate::pallet::*;
use crate::swap::{SwapPairRatio, SwapPairStatus};

const LOG_TARGET: &str = "runtime::pallet-asset-swap";

#[frame_support::pallet]
pub mod pallet {
	use crate::{
		swap::{SwapPairInfo, SwapPairRatio, SwapPairStatus},
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
	use xcm::{
		v3::{
			validate_send, AssetId,
			Instruction::{SetFeesMode, TransferAsset, WithdrawAsset},
			Junctions, MultiLocation, SendXcm, Xcm,
		},
		VersionedAssetId, VersionedMultiLocation,
	};

	pub type CurrencyBalanceOf<T> =
		<<T as Config>::Currency as InspectFungible<<T as frame_system::Config>::AccountId>>::Balance;
	pub type SwapPairInfoOf<T> = SwapPairInfo<<T as frame_system::Config>::AccountId>;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		const PALLET_ID: [u8; 8];

		type Currency: MutateFungible<Self::AccountId>;
		type SwapOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		type PauseOrigin: EnsureOrigin<Self::RuntimeOrigin>;
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type SubmitterOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Self::AccountId>;
		type XcmRouter: SendXcm;
	}

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		SwapPairCreated {
			remote_asset_id: VersionedAssetId,
			ratio: SwapPairRatio,
			maximum_issuance: u128,
			pool_account: T::AccountId,
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
	}

	#[pallet::error]
	pub enum Error<T> {
		AlreadyExisting,
		InvalidInput,
		LiquidityNotMet,
		NotEnabled,
		NotFound,
		RemotePoolBalance,
		UserBalance,
		Xcm,
		Internal,
	}

	#[pallet::storage]
	#[pallet::getter(fn swap_pair)]
	pub(crate) type SwapPair<T> = StorageValue<_, SwapPairInfoOf<T>, OptionQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(u64::MAX)]
		pub fn set_swap_pair(
			origin: OriginFor<T>,
			reserve_location: Box<VersionedMultiLocation>,
			remote_asset_id: Box<VersionedAssetId>,
			ratio: SwapPairRatio,
			total_issuance: u128,
			circulating_supply: u128,
		) -> DispatchResult {
			T::SwapOrigin::ensure_origin(origin)?;

			// 1. Verify swap pair has not already been set.
			ensure!(!SwapPair::<T>::exists(), Error::<T>::AlreadyExisting);
			// 2. Verify that total issuance >= circulating supply and take the difference
			//    as the amount of assets we control on destination.
			let locked_supply = total_issuance
				.checked_sub(circulating_supply)
				.ok_or(Error::<T>::InvalidInput)?;
			// 3. Verify the pool account has enough local assets to cover for all potential
			//    remote -> local swaps, according to the specified swap ratio.
			let pool_account = Self::pool_account_id_for_remote_asset(&remote_asset_id)?;
			// Local assets are calculated from the circulating supply * (local/remote) swap
			// rate.
			let minimum_local_assets_required = (circulating_supply / ratio.remote_asset) * ratio.local_asset;
			let pool_account_reducible_balance_as_u128: u128 =
				T::Currency::reducible_balance(&pool_account, Preservation::Expendable, Fortitude::Polite)
					.try_into()
					.map_err(|_| {
						log::error!(target: LOG_TARGET, "Failed to cast pool account reducible balance to u128.");
						Error::<T>::Internal
					})?;
			ensure!(
				pool_account_reducible_balance_as_u128 >= minimum_local_assets_required,
				Error::<T>::LiquidityNotMet
			);

			Self::set_swap_pair_bypass_checks(reserve_location, remote_asset_id, ratio, locked_supply, pool_account);

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(u64::MAX)]
		pub fn force_set_swap_pair(
			origin: OriginFor<T>,
			reserve_location: Box<VersionedMultiLocation>,
			remote_asset_id: Box<VersionedAssetId>,
			ratio: SwapPairRatio,
			maximum_issuance: u128,
		) -> DispatchResult {
			ensure_root(origin)?;

			let pool_account = Self::pool_account_id_for_remote_asset(&remote_asset_id)?;
			Self::set_swap_pair_bypass_checks(reserve_location, remote_asset_id, ratio, maximum_issuance, pool_account);

			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(u64::MAX)]
		pub fn force_unset_swap_pair(origin: OriginFor<T>) -> DispatchResult {
			ensure_root(origin)?;

			Self::unset_swap_pair_bypass_checks();

			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(u64::MAX)]
		pub fn pause_swap_pair(origin: OriginFor<T>) -> DispatchResult {
			T::PauseOrigin::ensure_origin(origin)?;

			Self::set_swap_pair_status(SwapPairStatus::Paused)?;

			Ok(())
		}

		#[pallet::call_index(4)]
		#[pallet::weight(u64::MAX)]
		pub fn resume_swap_pair(origin: OriginFor<T>) -> DispatchResult {
			T::SwapOrigin::ensure_origin(origin)?;

			Self::set_swap_pair_status(SwapPairStatus::Running)?;

			Ok(())
		}

		#[pallet::call_index(5)]
		#[pallet::weight(u64::MAX)]
		pub fn swap(
			origin: OriginFor<T>,
			local_asset_amount: CurrencyBalanceOf<T>,
			beneficiary: Box<VersionedMultiLocation>,
		) -> DispatchResult {
			let submitter = T::SubmitterOrigin::ensure_origin(origin).map_err(DispatchError::from)?;

			// 1. Retrieve swap pair info from storage, else fail.
			let swap_pair = SwapPair::<T>::get().ok_or(DispatchError::from(Error::<T>::NotFound))?;

			// 2. Check if swaps are enabled.
			ensure!(swap_pair.can_swap(), DispatchError::from(Error::<T>::NotEnabled));

			// 3. Verify the tx submitter has enough local assets for the swap.
			let balance_to_withdraw = T::Currency::can_withdraw(&submitter, local_asset_amount).into_result(true)?;
			ensure!(balance_to_withdraw == local_asset_amount, Error::<T>::UserBalance);

			// 4. Verify the local assets can be transferred to the swap pool account
			T::Currency::can_deposit(&swap_pair.pool_account, local_asset_amount, Provenance::Extant).into_result()?;

			// 5. Verify we have enough balance on the remote location to perform the
			//    transfer
			let local_asset_amount_as_u128: u128 = local_asset_amount.try_into().map_err(|_| {
				log::error!(target: LOG_TARGET, "Failed to cast user-specified account to u128.");
				DispatchError::from(Error::<T>::Internal)
			})?;
			let corresponding_remote_assets =
				(local_asset_amount_as_u128 / swap_pair.ratio.local_asset) * swap_pair.ratio.remote_asset;
			ensure!(
				swap_pair.remote_asset_balance >= corresponding_remote_assets,
				Error::<T>::RemotePoolBalance
			);
			// TODO: Convert to version based on destination
			let asset_id_v3: AssetId = swap_pair.remote_asset_id.clone().try_into().map_err(|e| {
				log::error!(target: LOG_TARGET, "Failed to convert asset ID {:?} into v3 `AssetId` with error {:?}", swap_pair.remote_asset_id, e);
				DispatchError::from(Error::<T>::Internal)
			})?;
			let beneficiary_v3: MultiLocation = (*beneficiary.clone()).try_into().map_err(|e| {
				log::error!(target: LOG_TARGET, "Failed to convert beneficiary {:?} into v3 `Multilocation` with error {:?}", beneficiary, e);
				DispatchError::from(Error::<T>::Internal)
			})?;
			let destination_v3: MultiLocation = swap_pair.remote_reserve_location.clone().try_into().map_err(|e| {
				log::error!(target: LOG_TARGET, "Failed to convert remote reserve location {:?} into v3 `Multilocation` with error {:?}", swap_pair.remote_reserve_location, e);
				DispatchError::from(Error::<T>::Internal)
			})?;

			// 6. Compose and validate XCM message
			let remote_xcm: Xcm<()> = vec![
				WithdrawAsset((Junctions::Here, 1_000_000_000).into()),
				SetFeesMode { jit_withdraw: true },
				TransferAsset {
					assets: (asset_id_v3, corresponding_remote_assets).into(),
					beneficiary: beneficiary_v3,
				},
				// TODO: Add try-catch and asset refund
			]
			.into();
			let xcm_ticket = validate_send::<T::XcmRouter>(destination_v3, remote_xcm.clone()).map_err(|e| {
				log::error!(
					"Failed to call validate_send for destination {:?} and remote XCM {:?} with error {:?}",
					destination_v3,
					remote_xcm,
					e
				);
				DispatchError::from(Error::<T>::Xcm)
			})?;

			// 7. Transfer funds from user to pool
			let transferred_amount = T::Currency::transfer(
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

			// 8. Send XCM out
			T::XcmRouter::deliver(xcm_ticket.0).map_err(|e| {
				log::error!("Failed to deliver ticket with error {:?}", e);
				DispatchError::from(Error::<T>::Xcm)
			})?;

			// TODO: Get right XCM version based on destination (we could default to always
			// use v3 for now, and v4 when we update to a newer polkadot-sdk version).
			// TODO: Delegate XCM message composition to a trait Config as well, depending
			// on the destination (choosing which asset to use for payments, what amount,
			// etc).
			// TODO: Add hook to check the swap parameters (restricting
			// where remote assets can be sent to).
			// TODO: Think about XCM fees management.

			Ok(())
		}
	}
}

impl<T: Config> Pallet<T> {
	fn set_swap_pair_bypass_checks(
		reserve_location: Box<VersionedMultiLocation>,
		remote_asset_id: Box<VersionedAssetId>,
		ratio: SwapPairRatio,
		maximum_issuance: u128,
		pool_account: T::AccountId,
	) {
		let swap_pair_info = SwapPairInfoOf::<T> {
			pool_account: pool_account.clone(),
			ratio: ratio.clone(),
			remote_asset_balance: maximum_issuance,
			remote_asset_id: *remote_asset_id.clone(),
			remote_reserve_location: *reserve_location,
			status: SwapPairStatus::Paused,
		};

		SwapPair::<T>::set(Some(swap_pair_info));

		Self::deposit_event(Event::<T>::SwapPairCreated {
			maximum_issuance,
			pool_account,
			ratio,
			remote_asset_id: *remote_asset_id,
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
		let event = SwapPair::<T>::try_mutate(|entry| {
			let swap_pair = entry.as_mut().ok_or(Error::<T>::NotFound)?;
			let event = match new_status {
				SwapPairStatus::Running => Event::<T>::SwapPairResumed {
					remote_asset_id: swap_pair.remote_asset_id.clone(),
				},
				SwapPairStatus::Paused => Event::<T>::SwapPairPaused {
					remote_asset_id: swap_pair.remote_asset_id.clone(),
				},
			};
			swap_pair.status = new_status;
			Ok::<_, Error<T>>(event)
		})?;
		Self::deposit_event(event);
		Ok(())
	}
}

impl<T: Config> Pallet<T> {
	fn pool_account_id_for_remote_asset(remote_asset_id: &VersionedAssetId) -> Result<T::AccountId, Error<T>> {
		let hash_input = (T::PALLET_ID, b'.', remote_asset_id.clone()).encode();
		let hash_output = sp_io::hashing::blake2_256(hash_input.as_slice());
		T::AccountId::decode(&mut TrailingZeroInput::new(hash_output.as_slice())).map_err(|e| {
			log::error!(target: LOG_TARGET, "Failed to generate pool ID from remote asset {:?} with error: {:?}", remote_asset_id, e);
			Error::<T>::Internal
		})
	}
}
