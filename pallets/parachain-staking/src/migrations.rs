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
#[cfg(feature = "try-runtime")]
use frame_support::ensure;
use frame_support::{dispatch::Weight, traits::Get};
use kilt_traits::VersionMigratorTrait;
use sp_runtime::{
	codec::{Decode, Encode},
	traits::Zero,
};
use sp_std::marker::PhantomData;

use crate::*;

mod v2;
mod v3;
mod v4;
mod v5;

// A value placed in storage that represents the current version of the Staking
// storage. This value is used by the `on_runtime_upgrade` logic to determine
// whether we run storage migration logic. This should match directly with the
// semantic versions of the Rust crate.
#[derive(Copy, Clone, Encode, Eq, Decode, Debug, Ord, PartialEq, PartialOrd)]
pub enum StakingStorageVersion {
	V1_0_0,
	V2_0_0, // New Reward calculation, MaxCollatorCandidateStake
	V3_0_0, // Update InflationConfig
	V4,     // Sort TopCandidates and parachain-stakings by amount
	V5,     // Remove SelectedCandidates, Count Candidates
}

#[cfg(feature = "try-runtime")]
impl StakingStorageVersion {
	/// The latest storage version.
	fn latest() -> Self {
		Self::V5
	}
}

// All nodes will default to this, which is not bad, as in case the "real"
// version is a later one (i.e. the node has been started with already the
// latest version), the migration will simply do nothing as there's nothing in
// the old storage entries to migrate from.
//
// It might get updated in the future when we know that no node is running this
// old version anymore.
impl Default for StakingStorageVersion {
	fn default() -> Self {
		Self::V5
	}
}

impl<T: Config> VersionMigratorTrait<T> for StakingStorageVersion {
	// It runs the right pre_migrate logic depending on the current storage version.
	#[cfg(feature = "try-runtime")]
	fn pre_migrate(&self) -> Result<(), &str> {
		match *self {
			Self::V1_0_0 => v2::pre_migrate::<T>(),
			Self::V2_0_0 => v3::pre_migrate::<T>(),
			Self::V3_0_0 => v4::pre_migrate::<T>(),
			Self::V4 => v5::pre_migrate::<T>(),
			Self::V5 => Ok(()),
		}
	}

	// It runs the right migration logic depending on the current storage version.
	fn migrate(&self) -> Weight {
		match *self {
			Self::V1_0_0 => v2::migrate::<T>(),
			Self::V2_0_0 => v3::migrate::<T>(),
			Self::V3_0_0 => v4::migrate::<T>(),
			Self::V4 => v5::migrate::<T>(),
			Self::V5 => Weight::zero(),
		}
	}

	// It runs the right post_migrate logic depending on the current storage
	// version.
	#[cfg(feature = "try-runtime")]
	fn post_migrate(&self) -> Result<(), &str> {
		match *self {
			Self::V1_0_0 => v2::post_migrate::<T>(),
			Self::V2_0_0 => v3::post_migrate::<T>(),
			Self::V3_0_0 => v4::post_migrate::<T>(),
			Self::V4 => v5::post_migrate::<T>(),
			Self::V5 => Ok(()),
		}
	}
}

/// The parachain-staking pallet's storage migrator, which handles all version
/// migrations in a sequential fashion.
///
/// If a node has missed on more than one upgrade, the migrator will apply the
/// needed migrations one after the other. Otherwise, if no migration is needed,
/// the migrator will simply not do anything.
pub struct StakingStorageMigrator<T>(PhantomData<T>);

impl<T: Config> StakingStorageMigrator<T> {
	// Contains the migration sequence logic.
	fn get_next_storage_version(current: StakingStorageVersion) -> Option<StakingStorageVersion> {
		match current {
			StakingStorageVersion::V1_0_0 => Some(StakingStorageVersion::V2_0_0),
			StakingStorageVersion::V2_0_0 => Some(StakingStorageVersion::V3_0_0),
			// Migration happens naturally, no need to point to the latest version
			StakingStorageVersion::V3_0_0 => Some(StakingStorageVersion::V4),
			StakingStorageVersion::V4 => Some(StakingStorageVersion::V5),
			StakingStorageVersion::V5 => None,
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
		let mut current_version: Option<StakingStorageVersion> = Some(StorageVersion::<T>::get());
		// Weight for StorageVersion::get().
		let mut total_weight = T::DbWeight::get().reads(1);

		while let Some(ver) = current_version {
			// If any of the needed migrations pre-checks fail, the whole chain panics
			// (during tests).
			#[cfg(feature = "try-runtime")]
			if let Err(err) = <StakingStorageVersion as VersionMigratorTrait<T>>::pre_migrate(&ver) {
				panic!("{:?}", err);
			}
			let consumed_weight = <StakingStorageVersion as VersionMigratorTrait<T>>::migrate(&ver);
			total_weight = total_weight.saturating_add(consumed_weight);
			// If any of the needed migrations post-checks fail, the whole chain panics
			// (during tests).
			#[cfg(feature = "try-runtime")]
			if let Err(err) = <StakingStorageVersion as VersionMigratorTrait<T>>::post_migrate(&ver) {
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
			StorageVersion::<T>::get() == StakingStorageVersion::latest(),
			"Not updated to the latest version."
		);

		Ok(())
	}
}
