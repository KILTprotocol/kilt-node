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

use sp_std::{boxed::Box, vec};

use crate::*;

// Contains the latest version that can be upgraded.
// For instance, if v1 of the storage is the last one available, only v0 would be
// upgradeable to v1, so the value of LATEST_UPGRADEABLE_VERSION would be 0.
const LATEST_UPGRADEABLE_VERSION: u16 = 0;

trait VersionMigrator<T: Config> {
	#[cfg(feature = "try-runtime")]
	fn pre_migrate(&self) -> Result<(), &'static str>;
	fn migrate(&self) -> Weight;
	#[cfg(feature = "try-runtime")]
	fn post_migrate(&self) -> Result<(), &'static str>;
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum DelegationMigrationError {
	AlreadyLatest,
	MigrationResultInconsistent,
}

pub(crate) struct StorageMigrator<T: Config> {
	migrations: Vec<Box<dyn VersionMigrator<T>>>,
}

impl<T: Config> StorageMigrator<T> {
	pub(crate) fn new() -> Self {
		Self {
			migrations: vec![Box::new(V0Migrator{})]
		}
	}

	#[cfg(any(feature = "try-runtime", test))]
	pub(crate) fn pre_migration(&self) -> Result<(), DelegationMigrationError> {
		ensure!(
			LastUpgradeVersion::<T>::get() <= migrations::LATEST_UPGRADEABLE_VERSION,
			DelegationMigrationError::AlreadyLatest
		);

		Ok(())
	}

	pub(crate) fn migrate(&self) -> Weight {
		let mut total_weight_used: Weight = 0;
		let current_version = LastUpgradeVersion::<T>::get();
		for version in current_version..=LATEST_UPGRADEABLE_VERSION {
			let version_migrator: &dyn VersionMigrator<T> = self.migrations[version as usize].as_ref();
			// Test pre-conditions for each migrated version
			#[cfg(feature = "try-runtime")]
			if let Err(err) = version_migrator.pre_migrate() {
				assert!(false, "{}", err);
			}
			total_weight_used = total_weight_used.saturating_add(version_migrator.migrate());
			// Test post-conditions for each migrated version
			#[cfg(feature = "try-runtime")]
			if let Err(err) = version_migrator.post_migrate() {
				assert!(false, "{}", err);
			}
		}
		// Set a version number that is not upgradeable anymore until a new version is available
		LastUpgradeVersion::<T>::set(LATEST_UPGRADEABLE_VERSION.saturating_add(1));

		// Add a DB read and write for the LastUpgradeVersion storage update
		total_weight_used.saturating_add(T::DbWeight::get().reads_writes(1, 1))
	}

	#[cfg(any(feature = "try-runtime", test))]
	pub(crate) fn post_migration(&self) -> Result<(), DelegationMigrationError> {
		ensure!(
			LastUpgradeVersion::<T>::get() == migrations::LATEST_UPGRADEABLE_VERSION.saturating_add(1),
			DelegationMigrationError::MigrationResultInconsistent
		);

		Ok(())
	}
}

struct V0Migrator();

impl<T: Config> VersionMigrator<T> for V0Migrator {
	#[cfg(feature = "try-runtime")]
	fn pre_migrate(&self) -> Result<(), &'static str> {
		assert!(
			LastUpgradeVersion::<T>::get() == 0,
			"Version not equal to 0 before v0 -> v1 migration."
		);
		log::debug!("Version storage migrating from v0 to v1");
		Ok(())
	}

	fn migrate(&self) -> Weight {
		log::debug!("v0 -> v1 delegation storage migrator started!");
		let total_weight = 0u64;
		LastUpgradeVersion::<T>::set(1);
		log::debug!("v0 -> v1 delegation storage migrator finished!");
		total_weight
	}

	#[cfg(feature = "try-runtime")]
	fn post_migrate(&self) -> Result<(), &'static str> {
		assert!(
			LastUpgradeVersion::<T>::get() == 0,
			"Version not equal to 1 after v0 -> v1 migration."
		);
		log::debug!("Version storage migrated from v0 to v1");
		Ok(())
	}
}

#[test]
fn ok_migrator_v0() {
	let migrator = StorageMigrator::<mock::Test>::new();
	let mut ext = mock::ExtBuilder::default().build(None);
	ext.execute_with(|| {
		assert!(
			migrator.pre_migration().is_ok(),
			"Pre-migration for v0 should not fail."
		);
		migrator.migrate();
		assert!(
			migrator.post_migration().is_ok(),
			"Post-migration for v0 should not fail."
		);
	});
}

#[test]
fn already_max_migrator_v0() {
	let migrator = StorageMigrator::<mock::Test>::new();
	let mut ext = mock::ExtBuilder::default().build(None);
	ext.execute_with(|| {
		LastUpgradeVersion::<mock::Test>::set(1);
		assert!(
			migrator.pre_migration().is_err(),
			"Pre-migration for v0 should fail."
		);
	});
}

#[test]
fn more_than_max_migrator_v0() {
	let migrator = StorageMigrator::<mock::Test>::new();
	let mut ext = mock::ExtBuilder::default().build(None);
	ext.execute_with(|| {
		LastUpgradeVersion::<mock::Test>::set(u16::MAX);
		assert!(
			migrator.pre_migration().is_err(),
			"Pre-migration for v0 should fail."
		);
	});
}
