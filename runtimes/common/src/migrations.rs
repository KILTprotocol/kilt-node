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

use frame_support::{
	storage::unhashed::clear_prefix, traits::OnRuntimeUpgrade, weights::Weight, StorageHasher, Twox128,
};
use sp_core::Get;
use sp_io::MultiRemovalResults;
use sp_std::marker::PhantomData;

const PALLET_RUNTIME_NAME: &[u8] = b"RandomnessCollectiveFlip";
#[cfg(feature = "try-runtime")]
const PALLET_STORAGE_NAME: &[u8] = b"RandomMaterial";

pub struct RemoveInsecureRandomnessPallet<T>(PhantomData<T>);

impl<T> OnRuntimeUpgrade for RemoveInsecureRandomnessPallet<T>
where
	T: frame_system::Config,
{
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<sp_std::vec::Vec<u8>, &'static str> {
		log::info!("RemoveInsecureRandomnessPallet::pre_upgrade() checks 🔎");
		if frame_support::migration::have_storage_value(PALLET_RUNTIME_NAME, PALLET_STORAGE_NAME, b"") {
			Ok(sp_std::vec::Vec::default())
		} else {
			Err("Storage in pallet_insecure_randomness_collective_flip is already empty before migration.")
		}
	}

	fn on_runtime_upgrade() -> Weight {
		let MultiRemovalResults { unique, .. } = clear_prefix(
			&Twox128::hash(PALLET_RUNTIME_NAME),
			// Storage version and `RandomMaterial` vector.
			Some(2),
			None,
		);
		log::info!(
			"Deleted {} elements from the pallet_insecure_randomness_collective_flip pallet storage.",
			unique
		);
		T::DbWeight::get().writes(unique.into())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: sp_std::vec::Vec<u8>) -> Result<(), &'static str> {
		log::info!("RemoveInsecureRandomnessPallet::post_upgrade() checks 🔍");
		if frame_support::migration::have_storage_value(PALLET_RUNTIME_NAME, PALLET_STORAGE_NAME, b"") {
			Err("Storage in pallet_insecure_randomness_collective_flip is not empty after migration.")
		} else {
			Ok(())
		}
	}
}
