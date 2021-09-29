// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2021 BOTLabs GmbH

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

use codec::{Decode, Encode};
use kilt_traits::VersionMigratorTrait;
use sp_std::marker::PhantomData;

use crate::*;

mod v1;

/// Storage version of the delegation pallet.
#[derive(Copy, Clone, Encode, Eq, Decode, Ord, PartialEq, PartialOrd)]
pub enum DelegationStorageVersion {
	V1,
	V2,
}

#[cfg(feature = "try-runtime")]
impl DelegationStorageVersion {
	/// The latest storage version.
	fn latest() -> Self {
		Self::V2
	}
}

// All nodes will default to this, which is not bad, as in case the "real"
// version is a later one (i.e. the node has been started with already the
// latest version), the migration will simply do nothing as there's nothing in
// the old storage entries to migrate from.
//
// It might get updated in the future when we know that no node is running this
// old version anymore.
impl Default for DelegationStorageVersion {
	fn default() -> Self {
		Self::V1
	}
}

impl<T: Config> VersionMigratorTrait<T> for DelegationStorageVersion {
	// It runs the right pre_migrate logic depending on the current storage version.
	#[cfg(feature = "try-runtime")]
	fn pre_migrate(&self) -> Result<(), &str> {
		match *self {
			Self::V1 => v1::pre_migrate::<T>(),
			Self::V2 => Ok(()),
		}
	}

	// It runs the right migration logic depending on the current storage version.
	fn migrate(&self) -> Weight {
		match *self {
			Self::V1 => v1::migrate::<T>(),
			Self::V2 => 0u64,
		}
	}

	// It runs the right post_migrate logic depending on the current storage
	// version.
	#[cfg(feature = "try-runtime")]
	fn post_migrate(&self) -> Result<(), &str> {
		match *self {
			Self::V1 => v1::post_migrate::<T>(),
			Self::V2 => Ok(()),
		}
	}
}

/// The delegation pallet's storage migrator, which handles all version
/// migrations in a sequential fashion.
///
/// If a node has missed on more than one upgrade, the migrator will apply the
/// needed migrations one after the other. Otherwise, if no migration is needed,
/// the migrator will simply not do anything.
pub struct DelegationStorageMigrator<T>(PhantomData<T>);

impl<T: Config> DelegationStorageMigrator<T> {
	// Contains the migration sequence logic.
	fn get_next_storage_version(current: DelegationStorageVersion) -> Option<DelegationStorageVersion> {
		// If the version current deployed is at least v1, there is no more migrations
		// to run (other than the one from v1).
		match current {
			DelegationStorageVersion::V1 => None,
			DelegationStorageVersion::V2 => None,
		}
	}

	/// Checks whether the latest storage version deployed is lower than the
	/// latest possible.
	#[cfg(feature = "try-runtime")]
	pub(crate) fn pre_migrate() -> Result<(), &'static str> {
		// Don't need to check for any other pre_migrate, as in try-runtime it is also
		// called in the migrate() function. Same applies for post_migrate checks for
		// each version migrator.

		Ok(())
	}

	/// Applies all the needed migrations from the currently deployed version to
	/// the latest possible, one after the other.
	///
	/// It returns the total weight consumed by ALL the migrations applied.
	pub(crate) fn migrate() -> Weight {
		let mut current_version: Option<DelegationStorageVersion> = Some(StorageVersion::<T>::get());
		// Weight for StorageVersion::get().
		let mut total_weight = T::DbWeight::get().reads(1);

		while let Some(ver) = current_version {
			// If any of the needed migrations pre-checks fail, the whole chain panics
			// (during tests).
			#[cfg(feature = "try-runtime")]
			if let Err(err) = <DelegationStorageVersion as VersionMigratorTrait<T>>::pre_migrate(&ver) {
				panic!("{:?}", err);
			}
			let consumed_weight = <DelegationStorageVersion as VersionMigratorTrait<T>>::migrate(&ver);
			total_weight = total_weight.saturating_add(consumed_weight);
			// If any of the needed migrations post-checks fail, the whole chain panics
			// (during tests).
			#[cfg(feature = "try-runtime")]
			if let Err(err) = <DelegationStorageVersion as VersionMigratorTrait<T>>::post_migrate(&ver) {
				panic!("{:?}", err);
			}
			// If more migrations should be applied, current_version will not be None.
			current_version = Self::get_next_storage_version(ver);
		}

		total_weight
	}

	/// Checks whether the storage version after all the needed migrations match
	/// the latest one.
	#[cfg(feature = "try-runtime")]
	pub(crate) fn post_migrate() -> Result<(), &'static str> {
		ensure!(
			StorageVersion::<T>::get() == DelegationStorageVersion::latest(),
			"Not updated to the latest version."
		);

		Ok(())
	}
}

// Tests for the entire storage migrator.
#[cfg(test)]
mod tests {
	use super::*;

	use crate::mock::Test as TestRuntime;

	#[test]
	fn ok_from_v1_migration() {
		let mut ext = mock::ExtBuilder::default()
			.with_storage_version(DelegationStorageVersion::V1)
			.build(None);
		ext.execute_with(|| {
			#[cfg(feature = "try-runtime")]
			assert!(
				DelegationStorageMigrator::<TestRuntime>::pre_migrate().is_ok(),
				"Storage pre-migrate from v1 should not fail."
			);

			DelegationStorageMigrator::<TestRuntime>::migrate();

			#[cfg(feature = "try-runtime")]
			assert!(
				DelegationStorageMigrator::<TestRuntime>::post_migrate().is_ok(),
				"Storage post-migrate from v1 should not fail."
			);
		});
	}

	#[test]
	fn ok_from_default_migration() {
		let mut ext = mock::ExtBuilder::default().build(None);
		ext.execute_with(|| {
			#[cfg(feature = "try-runtime")]
			assert!(
				DelegationStorageMigrator::<TestRuntime>::pre_migrate().is_ok(),
				"Storage pre-migrate from default version should not fail."
			);

			DelegationStorageMigrator::<TestRuntime>::migrate();

			#[cfg(feature = "try-runtime")]
			assert!(
				DelegationStorageMigrator::<TestRuntime>::post_migrate().is_ok(),
				"Storage post-migrate from default version should not fail."
			);
		});
	}
}
