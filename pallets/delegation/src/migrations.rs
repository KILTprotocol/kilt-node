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

use sp_std::marker::PhantomData;

use codec::{Decode, Encode};
use sp_std::collections::btree_map::BTreeMap;

use crate::*;

pub trait VersionMigratorTrait<Config: frame_system::Config> {
	#[cfg(any(feature = "try-runtime", test))]
	fn pre_migrate(&self) -> Result<(), &str>;
	fn migrate(&self) -> Weight;
	#[cfg(any(feature = "try-runtime", test))]
	fn post_migrate(&self) -> Result<(), &str>;
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Encode, Eq, Decode, PartialEq)]
pub enum DelegationStorageVersion {
	v1, v2
}

impl Default for DelegationStorageVersion {
    fn default() -> Self {
        Self::v2
    }
}

impl<T: Config> VersionMigratorTrait<T> for DelegationStorageVersion {

	#[cfg(any(feature = "try-runtime", test))]
	fn pre_migrate(&self) -> Result<(), &str> {
		match *self {
			Self::v1 => v1::pre_migrate::<T>(),
			Self::v2 => Ok(())
		}
	}

    fn migrate(&self) -> Weight {
		match *self {
			Self::v1 => v1::migrate::<T>(),
			Self::v2 => 0u64
		}
    }

	#[cfg(any(feature = "try-runtime", test))]
	fn post_migrate(&self) -> Result<(), &str> {
		match *self {
			Self::v1 => v1::post_migrate::<T>(),
			Self::v2 => Ok(())
		}
	}
}

mod v1 {
	use super::*;

	#[cfg(any(feature = "try-runtime", test))]
	pub(crate) fn pre_migrate<T: Config>() -> Result<(), &'static str> {
		ensure!(
			StorageVersion::<T>::get() == DelegationStorageVersion::v1,
			"Current deployed version is not v1."
		);
		log::info!("Version storage migrating from v1 to v2");
		Ok(())
	}

	pub(crate) fn migrate<T: Config>() -> Weight {
		log::info!("v1 -> v2 delegation storage migrator started!");
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

		StorageVersion::<T>::set(DelegationStorageVersion::v2);
		// Adds a write from StorageVersion::set() weight
		total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
		log::debug!("Total weight consumed: {}", total_weight);
		log::info!("v1 -> v2 delegation storage migrator finished!");
		total_weight
	}

	#[cfg(any(feature = "try-runtime", test))]
	pub(crate) fn post_migrate<T: Config>() -> Result<(), &'static str> {
		ensure!(
			StorageVersion::<T>::get() == DelegationStorageVersion::v2,
			"The version after deployment is not 2 as expected."
		);
		for (node_id, node) in DelegationNodes::<T>::iter() {
			if let Some(parent_id) = node.parent {
				let parent_node = DelegationNodes::<T>::get(parent_id).expect("Parent node should be in the storage.");
				ensure!(
					parent_node.children.contains(&node_id),
					"Parent-child wrong"
				);
			}
		}
		log::info!("Version storage migrated from v1 to v2");
		Ok(())
	}

	#[cfg(test)]
	mod tests {
		use super::*;

		use sp_core::Pair;

		use mock::Test as TestRuntime;

		#[test]
		fn ok_no_delegations() {
			let mut ext = mock::ExtBuilder::default().with_storage_version(DelegationStorageVersion::v1).build(None);
			ext.execute_with(|| {
				assert!(
					pre_migrate::<TestRuntime>().is_ok(),
					"Pre-migration for v1 should not fail."
				);

				migrate::<TestRuntime>();

				assert!(
					post_migrate::<TestRuntime>().is_ok(),
					"Post-migration for v1 should not fail."
				);
			});
		}

		#[test]
		fn ok_only_root() {
			let mut ext = mock::ExtBuilder::default().with_storage_version(DelegationStorageVersion::v1).build(None);
			ext.execute_with(|| {
				let alice = mock::get_ed25519_account(mock::get_alice_ed25519().public());
				let old_root_id = mock::get_delegation_id(true);
				let old_root_node =
					crate::deprecated::v0::DelegationRoot::<TestRuntime>::new(ctype::mock::get_ctype_hash(true), alice);
				Roots::insert(old_root_id, old_root_node.clone());

				assert!(
					pre_migrate::<TestRuntime>().is_ok(),
					"Pre-migration for v1 should not fail."
				);

				migrate::<TestRuntime>();

				assert!(
					post_migrate::<TestRuntime>().is_ok(),
					"Post-migration for v1 should not fail."
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
		fn ok_root_two_children() {
			let mut ext = mock::ExtBuilder::default().with_storage_version(DelegationStorageVersion::v1).build(None);
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
					pre_migrate::<TestRuntime>().is_ok(),
					"Pre-migration for v1 should not fail."
				);

				migrate::<TestRuntime>();

				assert!(
					post_migrate::<TestRuntime>().is_ok(),
					"Post-migration for v1 should not fail."
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
	}
}

pub struct DelegationStorageMigrator<T>(PhantomData<T>);

impl<T: Config> DelegationStorageMigrator<T> {

	fn get_next_storage_version(current: DelegationStorageVersion) -> Option<DelegationStorageVersion> {
		match current {
			DelegationStorageVersion::v1 => Some(DelegationStorageVersion::v2),
			DelegationStorageVersion::v2 => None
		}
	}

	#[cfg(any(feature = "try-runtime", test))]
	pub(crate) fn pre_migrate() -> Result<(), &'static str> {
		ensure!(
			StorageVersion::<T>::get() != DelegationStorageVersion::default(),
			"Already the latest (default) storage version."
		);

		Ok(())
	}

	pub(crate) fn migrate() -> Weight {
		let mut current_version: Option<DelegationStorageVersion> = Some(StorageVersion::<T>::get());
		let mut total_weight = T::DbWeight::get().reads(1);

		while let Some(ver) = current_version {
			#[cfg(feature = "try-runtime")]
			if let Err(err) =  <DelegationStorageVersion as VersionMigratorTrait<T>>::pre_migrate(&ver) {
				panic!("{:?}", err);
			}
			let consumed_weight = <DelegationStorageVersion as VersionMigratorTrait<T>>::migrate(&ver);
			total_weight = total_weight.saturating_add(consumed_weight);
			#[cfg(feature = "try-runtime")]
			if let Err(err) = <DelegationStorageVersion as VersionMigratorTrait<T>>::post_migrate(&ver) {
				panic!("{:?}", err);
			}
			current_version = Self::get_next_storage_version(ver);
		}

		total_weight
	}

	#[cfg(any(feature = "try-runtime", test))]
	pub(crate) fn post_migrate() -> Result<(), &'static str> {
		ensure!(
			StorageVersion::<T>::get() == DelegationStorageVersion::default(),
			"Not updated to the latest (default) version."
		);

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	use mock::Test as TestRuntime;

	#[test]
	fn ok_v1_migration() {
		let mut ext = mock::ExtBuilder::default().with_storage_version(DelegationStorageVersion::v1).build(None);
		ext.execute_with(|| {
			assert!(
				DelegationStorageMigrator::<TestRuntime>::pre_migrate().is_ok(),
				"Storage pre-migrate from v1 should not fail."
			);

			DelegationStorageMigrator::<TestRuntime>::migrate();

			assert!(
				DelegationStorageMigrator::<TestRuntime>::post_migrate().is_ok(),
				"Storage post-migrate from v1 should not fail."
			);
		});
	}
}
