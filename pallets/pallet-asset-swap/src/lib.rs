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

use parity_scale_codec::{Decode, Encode};
use sp_runtime::traits::TrailingZeroInput;
use xcm::{VersionedAssetId, VersionedMultiLocation};

pub use crate::pallet::*;
use crate::swap::{SwapPairRatio, SwapPairStatus};

const LOG_TARGET: &str = "runtime::pallet-asset-swap";

#[frame_support::pallet]
pub mod pallet {
	use crate::swap::{SwapPairInfo, SwapPairRatio, SwapPairStatus};

	use frame_support::{
		pallet_prelude::*,
		traits::{
			fungible::{Inspect as InspectFungible, Mutate as MutateFungible},
			EnsureOrigin,
		},
	};
	use frame_system::{ensure_root, pallet_prelude::*};
	use xcm::{VersionedAssetId, VersionedMultiLocation};

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
		SwapPairAlreadyCreated,
		SwapPairNotFound,
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
			maximum_issuance: u128,
		) -> DispatchResult {
			T::SwapOrigin::ensure_origin(origin)?;

			ensure!(!SwapPair::<T>::exists(), Error::<T>::SwapPairAlreadyCreated);

			Self::set_swap_pair_bypass_checks(reserve_location, remote_asset_id, ratio, maximum_issuance)?;

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

			Self::set_swap_pair_bypass_checks(reserve_location, remote_asset_id, ratio, maximum_issuance)?;

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
			let submitter = T::SubmitterOrigin::ensure_origin(origin)?;

			// let SwapRequestLocalAssetOf::<T> {
			// 	local_asset_id,
			// 	local_asset_amount,
			// } = *local_asset;

			// // 1. Verify pool for (local asset, remote asset) exists.
			// let swap_pair =
			// LocalToRemoteAssets::<T>::get(&local_asset_id).
			// ok_or(Error::<T>::LocalAssetNotFound)?; ensure!(
			// 	swap_pair.remote_asset_id == *remote_asset_id,
			// 	Error::<T>::AssetIdMismatch
			// );

			// // 2. Verify pool is running.
			// ensure!(swap_pair.running, Error::<T>::PoolNotEnabled);

			// // 3. Verify tx submitter has enough of the specified asset.
			// let can_withdraw_from_submitter = matches!(
			// 	T::LocalAssets::can_withdraw(local_asset_id.clone().into(), &submitter,
			// local_asset_amount), 	WithdrawConsequenceOf::<T>::Success
			// );
			// ensure!(can_withdraw_from_submitter,
			// Error::<T>::CannotWithdrawFromSubmitter);

			// // 4. Verify we can transfer those tokens into the swap pool account.
			// let can_deposit_into_pool = matches!(
			// 	T::LocalAssets::can_deposit(
			// 		local_asset_id.into(),
			// 		&swap_pair.pool_account,
			// 		local_asset_amount,
			// 		Provenance::Extant
			// 	),
			// 	DepositConsequence::Success
			// );
			// ensure!(can_deposit_into_pool, Error::<T>::CannotDepositIntoSwapPool);

			// // 5. Verify we have enough balance on the remote location to perform the
			// //    transfer.
			// let can_send_from_remote_reserve = {
			// 	let local_asset_amount_as_u128: u128 = local_asset_amount
			// 		.try_into()
			// 		.map_err(|_| Error::<T>::LocalAssetAmountOverflow)?;

			// 	// TODO: This probably has to change
			// 	let remote_asset_amount =
			// 		(local_asset_amount_as_u128 / swap_pair.ratio.local_asset) *
			// swap_pair.ratio.remote_asset; 	remote_asset_amount <=
			// swap_pair.remote_asset_balance };
			// ensure!(can_send_from_remote_reserve, Error::<T>::RemoteReserveDrained);

			// // TODO: Perform transfer, and compose XCM message.
			// // TODO: Think about XCM fees management.

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
	) -> Result<(), Error<T>> {
		let pool_account = Self::pool_account_id_for_remote_asset(&remote_asset_id)?;

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

		Ok(())
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
			let swap_pair = entry.as_mut().ok_or(Error::<T>::SwapPairNotFound)?;
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
