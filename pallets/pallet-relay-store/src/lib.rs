// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2023 BOTLabs GmbH

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

//! Pallet to store the last N (configurable) relay chain state roots to be used
//! for cross-chain state proof verification. The pallet relies on the
//! cumulus_parachain_system hook to populate the block `ValidationData` with
//! the latest relay chain state root.

#![cfg_attr(not(feature = "std"), no_std)]

mod default_weights;
mod relay;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[cfg(test)]
mod mock;

pub use crate::{default_weights::WeightInfo, pallet::*};

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	use frame_support::{pallet_prelude::*, BoundedVec};
	use frame_system::pallet_prelude::*;
	use sp_core::H256;

	use crate::relay::RelayParentInfo;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	/// Maps from a relaychain block height to its related information,
	/// including the state root.
	#[pallet::storage]
	#[pallet::getter(fn latest_relay_head_for_block)]
	pub(crate) type LatestRelayHeads<T: Config> = StorageMap<_, Twox64Concat, u32, RelayParentInfo<H256>>;

	// TODO: Replace this with a fixed-length array once support for const generics
	// is fully supported in Substrate.
	/// Storage value complimentary to [`LatestRelayHeads`] implementing a FIFO
	/// queue of the last N relay chain blocks info.
	#[pallet::storage]
	pub(crate) type LatestBlockHeights<T: Config> =
		StorageValue<_, BoundedVec<u32, T::MaxRelayBlocksStored>, ValueQuery>;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The maximum number of relaychain block details to store. When the
		/// limit is reached, oldest blocks are overridden with new ones.
		#[pallet::constant]
		type MaxRelayBlocksStored: Get<u32>;
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T>
	where
		T: cumulus_pallet_parachain_system::Config,
	{
		fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
			<T as Config>::WeightInfo::on_finalize()
		}

		fn on_finalize(n: BlockNumberFor<T>) {
			Self::on_finalize_internal(n)
		}
	}

	impl<T: Config> Pallet<T>
	where
		T: cumulus_pallet_parachain_system::Config,
	{
		pub(crate) fn on_finalize_internal(_n: BlockNumberFor<T>) {
			// Called before the validation data is cleaned in the
			// parachain_system::on_finalize hook
			let Some(new_validation_data) = cumulus_pallet_parachain_system::Pallet::<T>::validation_data() else {
				return;
			};
			let mut latest_block_heights = LatestBlockHeights::<T>::get();
			// Remove old relay block from both storage entries.
			if latest_block_heights.is_full() {
				let oldest_block_height = latest_block_heights.remove(0);
				LatestRelayHeads::<T>::remove(oldest_block_height);
				log::trace!(
					"Relay block queue full. Removing oldest block at height {:?}",
					oldest_block_height
				);
			}
			// Set the new relay block in storage.
			let relay_block_height = new_validation_data.relay_parent_number;
			log::trace!(
				"Adding new relay block with state root {:#02x?} and number {:?}",
				new_validation_data.relay_parent_storage_root,
				new_validation_data.relay_parent_number,
			);
			let push_res = latest_block_heights.try_push(relay_block_height);
			if let Err(err) = push_res {
				log::error!(
					"Pushing a new relay block to the queue should not fail but it did when adding relay block n. {:?}",
					err
				);
			} else {
				LatestBlockHeights::<T>::set(latest_block_heights);
				LatestRelayHeads::<T>::insert(
					relay_block_height,
					RelayParentInfo {
						relay_parent_storage_root: new_validation_data.relay_parent_storage_root,
					},
				);
			}
		}
	}
}
