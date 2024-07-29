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
	pallet_prelude::StorageVersion,
	traits::{GetStorageVersion, OnRuntimeUpgrade},
	weights::Weight,
};
use sp_core::Get;
use sp_std::marker::PhantomData;

const LOG_TARGET: &str = "migration::BumpStorageVersion";

/// There are some pallets without a storage version.
/// Based on the changes in the PR <https://github.com/paritytech/substrate/pull/13417>,
/// pallets without a storage version or with a wrong version throw an error
/// in the try state tests.
pub struct BumpStorageVersion<T>(PhantomData<T>);

const TARGET_PALLET_ASSETS_STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

impl<T> OnRuntimeUpgrade for BumpStorageVersion<T>
where
	T: pallet_assets::Config,
{
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<sp_std::vec::Vec<u8>, sp_runtime::TryRuntimeError> {
		if pallet_assets::Pallet::<T>::on_chain_storage_version() < TARGET_PALLET_ASSETS_STORAGE_VERSION {
			log::trace!(target: LOG_TARGET, "pallet_assets to be migrated to v1.");
		} else {
			log::trace!(target: LOG_TARGET, "pallet_assets already on v1. No migration will run.");
		}
		Ok([].into())
	}

	fn on_runtime_upgrade() -> Weight {
		log::info!(target: LOG_TARGET, "Initiating migration.");

		if pallet_assets::Pallet::<T>::on_chain_storage_version() < TARGET_PALLET_ASSETS_STORAGE_VERSION {
			log::info!(target: LOG_TARGET, "pallet_assets to be migrated to v1.");
			TARGET_PALLET_ASSETS_STORAGE_VERSION.put::<pallet_assets::Pallet<T>>();
			<T as frame_system::Config>::DbWeight::get().reads_writes(1, 1)
		} else {
			log::info!(target: LOG_TARGET, "pallet_assets already on v1. No migration will run.");
			<T as frame_system::Config>::DbWeight::get().reads(1)
		}
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: sp_std::vec::Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
		if pallet_assets::Pallet::<T>::on_chain_storage_version() < TARGET_PALLET_ASSETS_STORAGE_VERSION {
			Err(sp_runtime::TryRuntimeError::Other(
				"pallet_assets storage version was not updated to v1.",
			))
		} else {
			Ok(())
		}
	}
}
