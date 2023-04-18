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
	pallet_prelude::ValueQuery,
	storage_alias,
	traits::{Get, GetStorageVersion, OnRuntimeUpgrade, StorageVersion},
};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::AccountId32;
use sp_std::marker::PhantomData;

use crate::{linkable_account::LinkableAccountId, Config, Pallet};

/// A unified log target for did-lookup-migration operations.
pub const LOG_TARGET: &str = "runtime::pallet-did-lookup::migrations";

#[derive(Clone, Debug, Eq, PartialEq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum MixedStorageKey {
	V1(AccountId32),
	V2(LinkableAccountId),
}

#[derive(Clone, Debug, Eq, PartialEq, Encode, Decode, MaxEncodedLen, TypeInfo, Default)]
pub enum MigrationState {
	/// The migration was successful.
	Done,

	/// The storage has still the old layout, the migration wasn't started yet
	#[default]
	PreUpgrade,

	/// The upgrade is in progress and did migrate all storage up to the
	/// `MixedStorageKey`.
	Upgrading(MixedStorageKey),
}

impl MigrationState {
	pub fn is_done(&self) -> bool {
		matches!(self, MigrationState::Done)
	}

	pub fn is_in_progress(&self) -> bool {
		!matches!(self, MigrationState::Done)
	}
}

#[storage_alias]
type MigrationStateStore<T: Config> = StorageValue<Pallet<T>, MigrationState, ValueQuery>;

pub struct CleanupMigration<T>(PhantomData<T>);

impl<T: crate::pallet::Config> OnRuntimeUpgrade for CleanupMigration<T> {
	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		if Pallet::<T>::on_chain_storage_version() == StorageVersion::new(3) {
			log::info!("🔎 DidLookup: Initiating migration");
			MigrationStateStore::<T>::kill();
			Pallet::<T>::current_storage_version().put::<Pallet<T>>();

			T::DbWeight::get().reads_writes(1, 2)
		} else {
			// wrong storage version
			log::info!(
				target: LOG_TARGET,
				"Migration did not execute. This probably should be removed"
			);
			<T as frame_system::Config>::DbWeight::get().reads_writes(1, 0)
		}
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<sp_std::vec::Vec<u8>, &'static str> {
		use sp_std::vec;

		assert_eq!(
			Pallet::<T>::on_chain_storage_version(),
			StorageVersion::new(3),
			"On-chain storage version should be 3 before the migration"
		);
		assert!(MigrationStateStore::<T>::exists(), "Migration state should exist");

		log::info!(target: LOG_TARGET, "🔎 DidLookup: Pre migration checks successful");

		Ok(vec![])
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_pre_state: sp_std::vec::Vec<u8>) -> Result<(), &'static str> {
		assert_eq!(
			Pallet::<T>::on_chain_storage_version(),
			StorageVersion::new(4),
			"On-chain storage version should be updated"
		);
		assert!(!MigrationStateStore::<T>::exists(), "Migration state should be deleted");

		log::info!(target: LOG_TARGET, "🔎 DidLookup: Post migration checks successful");

		Ok(())
	}
}
