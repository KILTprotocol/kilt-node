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

use crate::{Config, ConnectedAccounts, ConnectedDids, Pallet};
use frame_support::{
	dispatch::Weight,
	traits::{Get, GetStorageVersion, OnRuntimeUpgrade},
};
use sp_std::marker::PhantomData;

pub struct LookupReverseIndexMigration<T>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for LookupReverseIndexMigration<T> {
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		assert!(Pallet::<T>::on_chain_storage_version() < Pallet::<T>::current_storage_version());
		assert_eq!(ConnectedAccounts::<T>::iter().count(), 0);

		log::info!(
			"👥  DID lookup pallet to {:?} passes PRE migrate checks ✅",
			Pallet::<T>::current_storage_version()
		);

		Ok(())
	}

	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		// Account for the new storage version written below.
		let initial_weight = T::DbWeight::get().writes(1);

		// Origin was disabled, so there cannot be any existing links. But we check just
		// to be sure.
		let total_weight: Weight =
			ConnectedDids::<T>::iter().fold(initial_weight, |total_weight, (account, record)| {
				ConnectedAccounts::<T>::insert(record.did, account, ());
				// One read for the `ConnectedDids` entry, one write for the new
				// `ConnectedAccounts` entry.
				total_weight.saturating_add(T::DbWeight::get().reads_writes(1, 1))
			});

		Pallet::<T>::current_storage_version().put::<Pallet<T>>();

		log::info!(
			"👥  completed DID lookup pallet migration to {:?} ✅",
			Pallet::<T>::current_storage_version()
		);

		total_weight
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		assert_eq!(
			Pallet::<T>::on_chain_storage_version(),
			Pallet::<T>::current_storage_version()
		);

		// Verify DID -> Account integrity.
		ConnectedDids::<T>::iter().for_each(|(account, record)| {
			assert!(ConnectedAccounts::<T>::contains_key(record.did, account));
		});
		// Verify Account -> DID integrity.
		ConnectedAccounts::<T>::iter().for_each(|(did, account, _)| {
			let entry = ConnectedDids::<T>::get(account).expect("Should find a record for the given account.");
			assert_eq!(entry.did, did);
		});

		log::info!(
			"👥  DID lookup pallet to {:?} passes POST migrate checks ✅",
			Pallet::<T>::current_storage_version()
		);

		Ok(())
	}
}
