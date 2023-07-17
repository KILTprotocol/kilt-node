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

// TODO: Pallet description

#![cfg_attr(not(feature = "std"), no_std)]

mod relay;

pub use crate::pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	use frame_support::{
		pallet_prelude::{ValueQuery, *},
		BoundedVec,
	};
	use frame_system::pallet_prelude::*;
	use sp_core::H256;

	use crate::relay::RelayParentInfo;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(0);

	#[pallet::storage]
	#[pallet::getter(fn latest_relay_head_for_block)]
	pub(crate) type LatestRelayHeads<T: Config> = StorageMap<_, Twox64Concat, H256, RelayParentInfo<u32, H256>>;

	// TODO: Use a better data structure for lookups
	#[pallet::storage]
	pub(crate) type LatestBlockHashes<T: Config> = StorageValue<_, BoundedVec<H256, T::MaxBlocks>, ValueQuery>;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		#[pallet::constant]
		type MaxBlocks: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T>
	where
		T: cumulus_pallet_parachain_system::Config,
	{
		fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
			// Reserve weight to update the last relay state root
			<T as frame_system::Config>::DbWeight::get().writes(2)
		}
		fn on_finalize(_n: BlockNumberFor<T>) {
			// Called before the validation data is cleaned in the
			// parachain_system::on_finalize hook
			let Some(new_validation_data) = cumulus_pallet_parachain_system::Pallet::<T>::validation_data() else { return; };
			// Remove old relay block from both storage entries.
			let mut latest_block_hashes = LatestBlockHashes::<T>::get();
			if latest_block_hashes.is_full() {
				let oldest_block_hash = latest_block_hashes.remove(0);
				LatestRelayHeads::<T>::remove(oldest_block_hash);
				log::trace!(
					"Relay block queue full. Removing oldest block with hash {:#02x?}",
					oldest_block_hash
				);
			}
			// Set the new relay block in storage.
			let relay_block_hash = new_validation_data.parent_head.hash();
			log::trace!(
				"Adding new relay block hash {:#02x?} with state root {:#02x?} and number {:#02x?}",
				relay_block_hash,
				new_validation_data.relay_parent_storage_root,
				new_validation_data.relay_parent_number,
			);
			LatestRelayHeads::<T>::insert(
				relay_block_hash,
				RelayParentInfo {
					relay_parent_number: new_validation_data.relay_parent_number,
					relay_parent_storage_root: new_validation_data.relay_parent_storage_root,
				},
			);
			latest_block_hashes
				.try_push(relay_block_hash)
				.expect("Should never fail to push a new object on the BoundedVec of relay block hashes.");
			LatestBlockHashes::<T>::set(latest_block_hashes);
		}
	}
}
