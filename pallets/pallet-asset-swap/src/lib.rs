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

pub use crate::pallet::*;

const LOG_TARGET: &str = "runtime::pallet-asset-swap";

#[frame_support::pallet]
pub mod pallet {
	use crate::{
		swap::{SwapPairInfo, SwapPairRatio},
		LOG_TARGET,
	};

	use frame_support::{
		pallet_prelude::*,
		traits::{fungible::Inspect, EnsureOrigin},
		Blake2_128Concat,
	};
	use frame_system::pallet_prelude::*;
	use xcm::{VersionedAssetId, VersionedInteriorMultiLocation, VersionedMultiLocation};

	pub type BalanceOf<T> = <<T as Config>::Currency as Inspect<<T as frame_system::Config>::AccountId>>::Balance;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Currency: Inspect<Self::AccountId>;
		type FeeManager: EnsureOrigin<Self::RuntimeOrigin>;
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		type SwapManager: EnsureOrigin<Self::RuntimeOrigin>;
	}

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		SwapPairCreated {
			local_asset_id: Box<VersionedInteriorMultiLocation>,
			remote_asset_id: Box<VersionedMultiLocation>,
			ratio: SwapPairRatio,
			maximum_issuance: u128,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		LocalAssetExisting,
		LocalAssetNotFound,
		AssetIdMismatch,
		Internal,
	}

	#[pallet::storage]
	#[pallet::getter(fn remote_asset_for_local_asset)]
	pub(crate) type LocalToRemoteAssets<T> =
		StorageMap<_, Blake2_128Concat, VersionedInteriorMultiLocation, SwapPairInfo>;

	#[pallet::storage]
	#[pallet::getter(fn local_asset_for_remote_asset)]
	pub(crate) type RemoteToLocalAssets<T> =
		StorageMap<_, Blake2_128Concat, VersionedAssetId, (VersionedMultiLocation, VersionedInteriorMultiLocation)>;

	#[pallet::storage]
	#[pallet::getter(fn delivery_fee_for_asset)]
	pub(crate) type DeliveryFeeForAsset<T> = StorageMap<_, Blake2_128Concat, VersionedAssetId, ()>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(u64::MAX)]
		pub fn add_swap_pair(
			origin: OriginFor<T>,
			local_asset_id: Box<VersionedInteriorMultiLocation>,
			reserve_location_base: Box<VersionedMultiLocation>,
			remote_asset_id: Box<VersionedAssetId>,
			ratio: SwapPairRatio,
			maximum_issuance: u128,
		) -> DispatchResult {
			T::SwapManager::ensure_origin(origin)?;

			LocalToRemoteAssets::<T>::try_mutate(&(*local_asset_id), |entry| match entry {
				Some(_) => Err(Error::<T>::LocalAssetExisting),
				None => {
					let swap_pair_info = SwapPairInfo {
						ratio,
						remote_asset_balance: maximum_issuance,
						remote_asset_id: *remote_asset_id.clone(),
						running: false,
					};
					*entry = Some(swap_pair_info);
					Ok(())
				}
			})?;

			RemoteToLocalAssets::<T>::try_mutate(remote_asset_id.clone(), |entry| match entry {
				Some(_) => {
					log::error!(target: LOG_TARGET, "Found an entry in `RemoteToLocalAssets` for remote asset ID: {:?} when there should not be any.", remote_asset_id);
					Err(Error::<T>::Internal)
				}
				None => {
					let local_asset_info = (*reserve_location_base, *local_asset_id);
					*entry = Some(local_asset_info);
					Ok(())
				}
			})?;

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(u64::MAX)]
		pub fn remove_swap_pair(
			origin: OriginFor<T>,
			local_asset_id: Box<VersionedInteriorMultiLocation>,
			remote_asset_id: Box<VersionedAssetId>,
		) -> DispatchResult {
			T::SwapManager::ensure_origin(origin)?;

			let swap_pair = LocalToRemoteAssets::<T>::take(local_asset_id).ok_or(Error::<T>::LocalAssetNotFound)?;
			ensure!(
				swap_pair.remote_asset_id == *remote_asset_id,
				Error::<T>::AssetIdMismatch
			);
			if RemoteToLocalAssets::<T>::take(&(*remote_asset_id)).is_some() {
				log::error!(target: LOG_TARGET, "Entry in `RemoteToLocalAssets` for remote asset ID: {:?} not found when there should have been one.", remote_asset_id);
				return Err(Error::<T>::Internal.into());
			}

			Ok(())
		}
	}
}
