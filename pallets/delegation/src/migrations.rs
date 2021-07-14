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

pub const LATEST_UPGRADEABLE_VERSION: u16 = 0;

trait VersionMigrator<T: Config> {
	fn pre_migrate(&self, current_version: u16) -> Result<u16, &'static str>;
	fn migrate(&self) -> Weight;
	fn post_migrate(&self) -> Result<(), &'static str>;
}

pub struct StorageMigrator<T: Config> {
	version: u16,
	migrations: Vec<Box<dyn VersionMigrator<T>>>,
}

impl<T: Config> StorageMigrator<T> {
	pub(crate) fn try_new(from_version: u16) -> Result<Self, &'static str> {
		ensure!(
			from_version <= LATEST_UPGRADEABLE_VERSION,
			"Version to migrate from cannot be higher than the latest upgradeable version."
		);

		Ok(Self {
			version: from_version,
			migrations: vec![Box::new(V0Migrator{})]
		})
	}

	// Calls the pre-migrate of the first migration to apply.
	#[cfg(feature = "try-runtime")]
	pub(crate) fn pre_migrate(&self) -> Result<(), &'static str> {
		self.migrations[self.version as usize].as_ref().pre_migrate(self.version).map(|_| ())
	}

	pub(crate) fn migrate(& self) -> Weight {
		let mut total_weight_used: Weight = 0;

		for version in self.version..=LATEST_UPGRADEABLE_VERSION {
			let version_migrator: &dyn VersionMigrator<T> = self.migrations[version as usize].as_ref();
			#[cfg(feature = "try-runtime")]
			if let Err(err) = version_migrator.pre_migrate(version) {
				assert!(false, "{}", err);
			}
			total_weight_used = total_weight_used.saturating_add(version_migrator.migrate());
			#[cfg(feature = "try-runtime")]
			if let Err(err) = version_migrator.post_migrate() {
				assert!(false, "{}", err);
			}
		}

		total_weight_used
	}

	// Calls the post-migrate of the last migration applied.
	#[cfg(feature = "try-runtime")]
	pub(crate) fn post_migrate(&self) -> Result<(), &'static str> {
		self.migrations[LATEST_UPGRADEABLE_VERSION as usize].as_ref().post_migrate()
	}
}

struct V0Migrator();

impl<T: Config> VersionMigrator<T> for V0Migrator {
	fn pre_migrate(&self, current_version: u16) -> Result<u16, &'static str> {
		assert!(
			current_version == 0,
			"Current version not equal to 0"
		);
		log::debug!("Migrating version storage from v0 to v1");
		Ok(1)
	}

	fn migrate(&self) -> Weight {
		log::debug!("V0 delegation storage migrator started!");
		let total_weight = 0u64;
		log::debug!("V0 delegation storage migrator finished!");
		total_weight
	}

	fn post_migrate(&self) -> Result<(), &'static str> {
		log::debug!("Version storage migrated from v0 to v1");
		Ok(())
	}
}

#[test]
fn test_migrator_v0() {
	let migrator = StorageMigrator::<mock::Test>::try_new(0).expect("Initializing storage migrator with version 0 should not fail.");
	migrator.migrate();
}

#[test]
fn test_migrator_v1() {
	assert!(
		StorageMigrator::<mock::Test>::try_new(1).is_err(),
		"Initializing storage migrator with at least version 1 should fail."
	);
}

#[test]
fn test_migrator_v2() {
	assert!(
		StorageMigrator::<mock::Test>::try_new(2).is_err(),
		"Initializing storage migrator with at least version 2 should fail."
	);
}
