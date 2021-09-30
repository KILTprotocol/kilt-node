use kilt_traits::VersionMigratorTrait;

use sp_runtime::traits::Zero;

pub struct StorageMigrator<VersionMigrator, T>(sp_std::marker::PhantomData<VersionMigrator>, sp_std::marker::PhantomData<T>);

impl<VersionMigrator, T> StorageMigrator<VersionMigrator, T> where VersionMigrator: VersionMigratorTrait<T> {
	#[cfg(feature = "try-runtime")]
	pub fn pre_migrate(migrator: VersionMigrator) -> Result<(), &'static str> {
		migrator.pre_migrate()
	}
	pub fn migrate(migrator: VersionMigrator) -> frame_support::weights::Weight {
		let mut current_version = Some(migrator);
		let mut total_weight = frame_support::weights::Weight::zero();

		while let Some(ver) = current_version {
			// If any of the needed migrations pre-checks fail, the whole chain panics
			// (during tests).
			#[cfg(feature = "try-runtime")]
			if let Err(err) = ver.pre_migrate() {
				panic!("{:?}", err);
			}
			let consumed_weight = ver.migrate();
			total_weight = total_weight.saturating_add(consumed_weight);
			// If any of the needed migrations post-checks fail, the whole chain panics
			// (during tests).
			#[cfg(feature = "try-runtime")]
			if let Err(err) = ver.post_migrate() {
				panic!("{:?}", err);
			}
			// If more migrations should be applied, current_version will not be None.
			current_version = ver.next_version();
		}
		total_weight
	}
	#[cfg(feature = "try-runtime")]
	pub fn post_migrate(migrator: VersionMigrator) -> Result<(), &'static str> {
		migrator.post_migrate()
	}
}
