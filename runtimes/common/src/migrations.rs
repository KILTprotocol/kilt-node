// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2022 BOTLabs GmbH

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
	pallet_prelude::ValueQuery,
	storage_alias,
	traits::{GetStorageVersion, OnRuntimeUpgrade},
};
use sp_runtime::traits::{Get, Zero};
use sp_std::marker::PhantomData;

use ctype::{CtypeCreatorOf, CtypeEntryOf};

#[storage_alias]
type MigrationCounter<T: ctype::Config> = StorageValue<ctype::Pallet<T>, u32, ValueQuery>;

pub struct AddCTypeBlockNumber<R>(PhantomData<R>);

impl<T: ctype::Config> OnRuntimeUpgrade for AddCTypeBlockNumber<T> {
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		assert_eq!(ctype::Pallet::<T>::on_chain_storage_version(), 0,);
		assert!(MigrationCounter::<T>::get().is_zero());

		// Use iter_keys() on new storage so it won't try to decode values.
		let ctypes_to_migrate = ctype::Ctypes::<T>::iter_keys().count();

		log::info!("🪪  CType pallet pre check: {:?} CTypes to migrate", ctypes_to_migrate);

		MigrationCounter::<T>::set(ctypes_to_migrate as u32);
		Ok(())
	}

	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		let current = ctype::Pallet::<T>::current_storage_version();
		let onchain = ctype::Pallet::<T>::on_chain_storage_version();

		log::info!(
			"💰 Running migration with current storage version {:?} / onchain {:?}",
			current,
			onchain
		);

		let mut num_translations = 0u64;
		let default_block_number = <T as frame_system::Config>::BlockNumber::zero();

		ctype::Ctypes::<T>::translate_values(|old: CtypeCreatorOf<T>| {
			num_translations = num_translations.saturating_add(1);
			Some(CtypeEntryOf::<T> {
				creator: old,
				created_at: default_block_number,
			})
		});
		current.put::<ctype::Pallet<T>>();

		// Num translations + old version read and new version write
		T::DbWeight::get().reads_writes(num_translations.saturating_add(1), num_translations.saturating_add(1))
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		assert_eq!(ctype::Pallet::<T>::on_chain_storage_version(), 1);

		// Use iter() on new storage so it also checks that the new values can be
		// decoded after the migration.
		assert_eq!(MigrationCounter::<T>::get(), ctype::Ctypes::<T>::iter().count() as u32);
		if let Some(ctype_entry) = ctype::Ctypes::<T>::iter_values().last() {
			assert!(ctype_entry.creation_block_number.is_zero());
		}

		log::info!(
			"🪪  CType pallet post checks ok, all {:} CTypes have been migrated ✅",
			MigrationCounter::<T>::get()
		);
		Ok(())
	}
}
