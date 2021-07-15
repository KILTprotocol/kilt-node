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

use sp_std::{boxed::Box, collections::btree_map::BTreeMap, vec};

use crate::*;

// Contains the latest version that can be upgraded.
// For instance, if v1 of the storage is the last one available, only v0 would
// be upgradeable to v1, so the value of LATEST_UPGRADEABLE_VERSION would be 0.
// This needs to be bumped (by 1) every time a new storage migration should take
// place. Along with this, a new version migrator must be defined and added to
// the `migrations` vector or the `StorageMigrator`.
const LATEST_UPGRADEABLE_VERSION: u16 = 0;

// The trait that each version migrator must implement.
trait VersionMigrator<T: Config> {
	#[cfg(feature = "try-runtime")]
	fn pre_migrate(&self) -> Result<(), DelegationMigrationError>;
	fn migrate(&self) -> Weight;
	#[cfg(feature = "try-runtime")]
	fn post_migrate(&self) -> Result<(), DelegationMigrationError>;
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum DelegationMigrationError {
	AlreadyLatest,
	MigrationResultInconsistent,
}

// The "manager" of the pallet's storage migrations. It contains a vector of
// version migrations each of which corresponds to a new runtime upgrade to run,
// sequentially. It interacts with the `LastUpgradeVersion` of the pallet's
// storage.
//
// When a new runtime upgrade needs to be added, a new component implementing
// the `VersionMigrator` trait must be defined and added at the end of the
// `migrations` vector, so that it is always executed after all
// previous migrations have taken place.
pub(crate) struct StorageMigrator<T: Config> {
	// The vector of version migrators.
	migrations: Vec<Box<dyn VersionMigrator<T>>>,
}

impl<T: Config> StorageMigrator<T> {
	pub(crate) fn new() -> Self {
		Self {
			migrations: vec![Box::new(v0::V0Migrator {})],
		}
	}

	// Checks whether there is at least one storage migration to perform.
	#[cfg(any(feature = "try-runtime", test))]
	#[allow(clippy::absurd_extreme_comparisons)]
	pub(crate) fn pre_migration(&self) -> Result<(), DelegationMigrationError> {
		ensure!(
			LastUpgradeVersion::<T>::get() <= migrations::LATEST_UPGRADEABLE_VERSION,
			DelegationMigrationError::AlreadyLatest
		);

		Ok(())
	}

	// It retrieves the latest version deployed on chain, and sequentially applies
	// any runtime migration until the state is updated to the latest version.
	// It provides a test feature that panics whenever something goes wrong for one
	// of the version migrators, but in production it is just assumed that all the
	// migrations will go through.
	pub(crate) fn migrate(&self) -> Weight {
		let mut total_weight_used: Weight = 0;
		let current_version = LastUpgradeVersion::<T>::get();
		// Applies all the needed migration from the current version up to
		// LATEST_UPGRADEABLE_VERSION (which is always the latest possible version - 1).
		for version in current_version..=LATEST_UPGRADEABLE_VERSION {
			// It is assumed that the are exactly LATEST_UPGRADEABLE_VERSION+1 migrators, so
			// as long as the pre-condition is met, the following is assumed to never panic.
			let version_migrator: &dyn VersionMigrator<T> = self.migrations[version as usize].as_ref();
			// Test pre-conditions (only in testing) for each version migrator.
			#[cfg(feature = "try-runtime")]
			if let Err(err) = version_migrator.pre_migrate() {
				panic!("{:?}", err);
			}
			// Apply the migration and keep adding the weight.
			total_weight_used = total_weight_used.saturating_add(version_migrator.migrate());
			// Test post-conditions (only in testing) for each version migrator.
			#[cfg(feature = "try-runtime")]
			if let Err(err) = version_migrator.post_migrate() {
				panic!("{:?}", err);
			}
		}
		// Set a version number that is not upgradeable anymore until a new version is
		// available, which means that the new version will be
		// LATEST_UPGRADEABLE_VERSION + 1.
		LastUpgradeVersion::<T>::set(LATEST_UPGRADEABLE_VERSION.saturating_add(1));

		// Add a DB read and write for the LastUpgradeVersion storage update
		total_weight_used.saturating_add(T::DbWeight::get().reads_writes(1, 1))
	}

	// Checks whether the upgrade as a whole went through by verifying that the
	// latest version set it not upgradeable anymore.
	#[cfg(any(feature = "try-runtime", test))]
	pub(crate) fn post_migration(&self) -> Result<(), DelegationMigrationError> {
		ensure!(
			LastUpgradeVersion::<T>::get() == migrations::LATEST_UPGRADEABLE_VERSION.saturating_add(1),
			DelegationMigrationError::MigrationResultInconsistent
		);

		Ok(())
	}
}

mod v0 {
	use super::*;
	// Migrator for the first actual pallet's migration. It migrates the Roots,
	// Delegations, and Children storage entries to a simpler HierarchyInfos,
	// DelegationNodes, while maintaining the original information an all
	// parent-child links.
	pub struct V0Migrator();

	impl<T: Config> VersionMigrator<T> for V0Migrator {
		#[cfg(feature = "try-runtime")]
		fn pre_migrate(&self) -> Result<(), DelegationMigrationError> {
			ensure!(
				LastUpgradeVersion::<T>::get() == 0,
				DelegationMigrationError::AlreadyLatest
			);
			log::info!("Version storage migrating from v0 to v1");
			Ok(())
		}

		fn migrate(&self) -> Weight {
			log::info!("v0 -> v1 delegation storage migrator started!");
			let mut total_weight = 0u64;

			// Before being stored, the nodes are saved in a map so that after we go over
			// all the nodes and the parent-child relationship in the storage, we can update
			// the `parent` link of each node accordingly. Otherwise, it would be possible
			// that a node does not exist when fetched from the Children storage entry.
			let mut new_nodes: BTreeMap<DelegationNodeIdOf<T>, DelegationNode<T>> = BTreeMap::new();

			// First iterate over the delegation roots and translate them to hierarchies.
			for (old_root_id, old_root_node) in Roots::<T>::drain() {
				let new_hierarchy_info = DelegationHierarchyInfo::<T> {
					ctype_hash: old_root_node.ctype_hash,
				};
				let new_root_details = DelegationDetails::<T> {
					owner: old_root_node.owner,
					// Old roots did not have any permissions. So now we give them all permissions.
					permissions: Permissions::all(),
					revoked: old_root_node.revoked,
				};
				// In here, we already check for potential children of root nodes and ONLY
				// update the children information. The parent information will be updated
				// later, when we know we have seen all the children already.
				let mut new_root_node = DelegationNode::new_root_node(old_root_id, new_root_details);
				if let Some(root_children_ids) = Children::<T>::take(old_root_id) {
					new_root_node.children = root_children_ids.iter().copied().collect();
				}
				// Add Children::take() weight
				total_weight = total_weight.saturating_add(T::DbWeight::get().reads(1));

				DelegationHierarchies::insert(old_root_id, new_hierarchy_info);
				// Adds a read from Roots::drain() and a write from
				// DelegationHierarchies::insert() weights
				total_weight = total_weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
				new_nodes.insert(old_root_id, new_root_node);
			}

			// Then iterate over the regular delegation nodes.
			for (old_node_id, old_node) in Delegations::<T>::drain() {
				let new_node_details = DelegationDetails::<T> {
					owner: old_node.owner,
					permissions: old_node.permissions,
					revoked: old_node.revoked,
				};
				// In the old version, a parent None indicated the node is a child of the root.
				let new_node_parent_id = old_node.parent.unwrap_or(old_node.root_id);
				let mut new_node =
					DelegationNode::<T>::new_node(old_node.root_id, new_node_parent_id, new_node_details);
				if let Some(children_ids) = Children::<T>::take(old_node_id) {
					new_node.children = children_ids.iter().copied().collect();
				}
				// Add Children::take() weight
				total_weight = total_weight.saturating_add(T::DbWeight::get().reads(1));
				// Adds a read from Roots::drain() weight
				total_weight = total_weight.saturating_add(T::DbWeight::get().reads(1));
				new_nodes.insert(old_node_id, new_node);
			}

			// By now, all the children should have been correctly added to the nodes.
			// We now need to modify all the nodes that are children by adding a reference
			// to their parents.
			for (new_node_id, new_node) in new_nodes.clone().into_iter() {
				// FIXME: new_node.children.iter().cloned() might be possibly changed to
				// iter_mut.
				for child_id in new_node.children.iter().cloned() {
					new_nodes
						.entry(child_id)
						.and_modify(|node| node.parent = Some(new_node_id));
				}
				// We can then finally insert the new delegation node in the storage.
				DelegationNodes::<T>::insert(new_node_id, new_node);
				// Adds a write from DelegationNodes::insert() weight
				total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
			}

			LastUpgradeVersion::<T>::set(1);
			// Adds a write from LastUpgradeVersion::set() weight
			total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
			log::debug!("Total weight consumed: {}", total_weight);
			log::info!("v0 -> v1 delegation storage migrator finished!");
			total_weight
		}

		#[cfg(feature = "try-runtime")]
		fn post_migrate(&self) -> Result<(), DelegationMigrationError> {
			ensure!(
				LastUpgradeVersion::<T>::get() == 1,
				DelegationMigrationError::MigrationResultInconsistent
			);
			log::info!("Version storage migrated from v0 to v1");
			Ok(())
		}
	}

	#[cfg(test)]
	mod tests {
		use super::*;

		use mock::Test as TestRuntime;
		use sp_core::Pair;

		fn get_storage_migrator() -> StorageMigrator<TestRuntime> {
			StorageMigrator::<TestRuntime>::new()
		}

		fn init_logger() {
			let _ = env_logger::builder().is_test(true).try_init();
		}

		#[test]
		fn ok_no_delegations() {
			let migrator = get_storage_migrator();
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
		fn ok_only_root() {
			init_logger();
			let migrator = get_storage_migrator();
			let mut ext = mock::ExtBuilder::default().build(None);
			ext.execute_with(|| {
				let alice = mock::get_ed25519_account(mock::get_alice_ed25519().public());
				let old_root_id = mock::get_delegation_id(true);
				let old_root_node =
					crate::deprecated::v0::DelegationRoot::<TestRuntime>::new(ctype::mock::get_ctype_hash(true), alice);
				Roots::insert(old_root_id, old_root_node.clone());

				assert!(
					migrator.pre_migration().is_ok(),
					"Pre-migration for v0 should not fail."
				);

				migrator.migrate();

				assert!(
					migrator.post_migration().is_ok(),
					"Post-migration for v0 should not fail."
				);

				assert_eq!(Roots::<TestRuntime>::iter_values().count(), 0);
				assert_eq!(Delegations::<TestRuntime>::iter_values().count(), 0);
				assert_eq!(Children::<TestRuntime>::iter_values().count(), 0);

				let new_stored_hierarchy = DelegationHierarchies::<TestRuntime>::get(old_root_id)
					.expect("New delegation hierarchy should exist in the storage.");
				assert_eq!(new_stored_hierarchy.ctype_hash, old_root_node.ctype_hash);
				let new_stored_root = DelegationNodes::<TestRuntime>::get(old_root_id)
					.expect("New delegation root should exist in the storage.");
				assert_eq!(new_stored_root.hierarchy_root_id, old_root_id);
				assert!(new_stored_root.parent.is_none());
				assert!(new_stored_root.children.is_empty());
				assert_eq!(new_stored_root.details.owner, old_root_node.owner);
				assert_eq!(new_stored_root.details.revoked, old_root_node.revoked);
			});
		}

		#[test]
		fn ok_three_level_hierarchy() {
			init_logger();
			let migrator = get_storage_migrator();
			let mut ext = mock::ExtBuilder::default().build(None);
			ext.execute_with(|| {
				let alice = mock::get_ed25519_account(mock::get_alice_ed25519().public());
				let bob = mock::get_sr25519_account(mock::get_bob_sr25519().public());
				let old_root_id = mock::get_delegation_id(true);
				let old_root_node =
					crate::deprecated::v0::DelegationRoot::<TestRuntime>::new(ctype::mock::get_ctype_hash(true), alice.clone());
				let old_parent_id = mock::get_delegation_id(false);
				let old_parent_node =
					crate::deprecated::v0::DelegationNode::<TestRuntime>::new_root_child(old_root_id, alice, Permissions::all());
				let old_node_id = mock::get_delegation_id_2(true);
				let old_node = crate::deprecated::v0::DelegationNode::<TestRuntime>::new_node_child(
					old_root_id,
					old_parent_id,
					bob,
					Permissions::ATTEST,
				);
				Roots::insert(old_root_id, old_root_node.clone());
				Delegations::insert(old_parent_id, old_parent_node.clone());
				Delegations::insert(old_node_id, old_node.clone());
				Children::<TestRuntime>::insert(old_root_id, vec![old_parent_id]);
				Children::<TestRuntime>::insert(old_parent_id, vec![old_node_id]);

				assert!(
					migrator.pre_migration().is_ok(),
					"Pre-migration for v0 should not fail."
				);

				migrator.migrate();

				assert!(
					migrator.post_migration().is_ok(),
					"Post-migration for v0 should not fail."
				);

				assert_eq!(Roots::<TestRuntime>::iter_values().count(), 0);
				assert_eq!(Delegations::<TestRuntime>::iter_values().count(), 0);
				assert_eq!(Children::<TestRuntime>::iter_values().count(), 0);

				let new_stored_hierarchy = DelegationHierarchies::<TestRuntime>::get(old_root_id)
					.expect("New delegation hierarchy should exist in the storage.");
				assert_eq!(new_stored_hierarchy.ctype_hash, old_root_node.ctype_hash);
				let new_stored_root = DelegationNodes::<TestRuntime>::get(old_root_id)
					.expect("New delegation root should exist in the storage.");
				assert_eq!(new_stored_root.hierarchy_root_id, old_root_id);
				assert!(new_stored_root.parent.is_none());
				assert_eq!(new_stored_root.children.len(), 1);
				assert!(new_stored_root.children.contains(&old_parent_id));
				assert_eq!(new_stored_root.details.owner, old_root_node.owner);
				assert_eq!(new_stored_root.details.revoked, old_root_node.revoked);

				let new_stored_parent = DelegationNodes::<TestRuntime>::get(old_parent_id)
					.expect("New delegation parent should exist in the storage.");
				assert_eq!(new_stored_parent.hierarchy_root_id, old_root_id);
				assert_eq!(new_stored_parent.parent, Some(old_root_id));
				assert_eq!(new_stored_parent.children.len(), 1);
				assert!(new_stored_parent.children.contains(&old_node_id));
				assert_eq!(new_stored_parent.details.owner, old_parent_node.owner);
				assert_eq!(new_stored_parent.details.revoked, old_parent_node.revoked);

				let new_stored_node = DelegationNodes::<TestRuntime>::get(old_node_id)
					.expect("New delegation node should exist in the storage.");
				assert_eq!(new_stored_node.hierarchy_root_id, old_root_id);
				assert_eq!(new_stored_node.parent, Some(old_parent_id));
				assert!(new_stored_node.children.is_empty());
				assert_eq!(new_stored_node.details.owner, old_node.owner);
				assert_eq!(new_stored_node.details.revoked, old_node.revoked);
			});
		}

		#[test]
		fn ok_root_two_children() {
			init_logger();
			let migrator = get_storage_migrator();
			let mut ext = mock::ExtBuilder::default().build(None);
			ext.execute_with(|| {
				let alice = mock::get_ed25519_account(mock::get_alice_ed25519().public());
				let bob = mock::get_sr25519_account(mock::get_bob_sr25519().public());
				let old_root_id = mock::get_delegation_id(true);
				let old_root_node =
					crate::deprecated::v0::DelegationRoot::<TestRuntime>::new(ctype::mock::get_ctype_hash(true), alice.clone());
				let old_node_id_1 = mock::get_delegation_id(false);
				let old_node_1 =
					crate::deprecated::v0::DelegationNode::<TestRuntime>::new_root_child(old_root_id, alice, Permissions::DELEGATE);
				let old_node_id_2 = mock::get_delegation_id_2(true);
				let old_node_2 =
					crate::deprecated::v0::DelegationNode::<TestRuntime>::new_root_child(old_root_id, bob, Permissions::ATTEST);
				Roots::insert(old_root_id, old_root_node.clone());
				Delegations::insert(old_node_id_1, old_node_1.clone());
				Delegations::insert(old_node_id_2, old_node_2.clone());
				Children::<TestRuntime>::insert(old_root_id, vec![old_node_id_1, old_node_id_2]);

				assert!(
					migrator.pre_migration().is_ok(),
					"Pre-migration for v0 should not fail."
				);

				migrator.migrate();

				assert!(
					migrator.post_migration().is_ok(),
					"Post-migration for v0 should not fail."
				);

				assert_eq!(Roots::<TestRuntime>::iter_values().count(), 0);
				assert_eq!(Delegations::<TestRuntime>::iter_values().count(), 0);
				assert_eq!(Children::<TestRuntime>::iter_values().count(), 0);

				let new_stored_hierarchy = DelegationHierarchies::<TestRuntime>::get(old_root_id)
					.expect("New delegation hierarchy should exist in the storage.");
				assert_eq!(new_stored_hierarchy.ctype_hash, old_root_node.ctype_hash);
				let new_stored_root = DelegationNodes::<TestRuntime>::get(old_root_id)
					.expect("New delegation root should exist in the storage.");
				assert_eq!(new_stored_root.hierarchy_root_id, old_root_id);
				assert!(new_stored_root.parent.is_none());
				assert_eq!(new_stored_root.children.len(), 2);
				assert!(new_stored_root.children.contains(&old_node_id_1));
				assert!(new_stored_root.children.contains(&old_node_id_2));
				assert_eq!(new_stored_root.details.owner, old_root_node.owner);
				assert_eq!(new_stored_root.details.revoked, old_root_node.revoked);

				let new_stored_node_1 = DelegationNodes::<TestRuntime>::get(old_node_id_1)
					.expect("New delegation 1 should exist in the storage.");
				assert_eq!(new_stored_node_1.hierarchy_root_id, old_root_id);
				assert_eq!(new_stored_node_1.parent, Some(old_root_id));
				assert!(new_stored_node_1.children.is_empty());
				assert_eq!(new_stored_node_1.details.owner, old_node_1.owner);
				assert_eq!(new_stored_node_1.details.revoked, old_node_1.revoked);

				let new_stored_node_2 = DelegationNodes::<TestRuntime>::get(old_node_id_2)
					.expect("New delegation 2 should exist in the storage.");
				assert_eq!(new_stored_node_2.hierarchy_root_id, old_root_id);
				assert_eq!(new_stored_node_2.parent, Some(old_root_id));
				assert!(new_stored_node_2.children.is_empty());
				assert_eq!(new_stored_node_2.details.owner, old_node_2.owner);
				assert_eq!(new_stored_node_2.details.revoked, old_node_2.revoked);
			});
		}

		#[test]
		fn err_already_max_migrator() {
			let migrator = StorageMigrator::<TestRuntime>::new();
			let mut ext = mock::ExtBuilder::default().build(None);
			ext.execute_with(|| {
				LastUpgradeVersion::<TestRuntime>::set(1);
				assert!(migrator.pre_migration().is_err(), "Pre-migration for v0 should fail.");
			});
		}

		#[test]
		fn err_more_than_max_migrator() {
			let migrator = StorageMigrator::<TestRuntime>::new();
			let mut ext = mock::ExtBuilder::default().build(None);
			ext.execute_with(|| {
				LastUpgradeVersion::<TestRuntime>::set(u16::MAX);
				assert!(migrator.pre_migration().is_err(), "Pre-migration for v0 should fail.");
			});
		}
	}
}
