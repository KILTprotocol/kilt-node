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

use crate::{
	deprecated::{
		v1::storage::{Children, Delegations, Roots},
		v2::{storage::DelegationNodes, DelegationNode as DelegationNodeV2},
	},
	migrations::DelegationStorageVersion,
	Config, DelegationDetails, DelegationHierarchies, DelegationHierarchyDetails, DelegationNodeIdOf, Permissions,
	StorageVersion, Weight,
};
use frame_support::{
	storage::bounded_btree_set::BoundedBTreeSet, traits::Get, IterableStorageMap, StorageMap, StoragePrefixedMap,
};
use sp_runtime::traits::Zero;
use sp_std::{
	collections::{btree_map::BTreeMap, btree_set::BTreeSet},
	convert::TryFrom,
};

/// Checks whether the deployed storage version is v1. If not, it won't try
/// migrate any data.
///
/// Since we have the default storage version to this one, it can happen
/// that new nodes will still try to perform runtime migrations. This is not
/// a problem as at the end of the day there will not be anything in the old
/// storage entries to migrate from. Hence, the "pseudo-"migration will
/// simply result in the update of the storage deployed version.
#[cfg(feature = "try-runtime")]
pub(crate) fn pre_migrate<T: Config>() -> Result<(), &'static str> {
	frame_support::ensure!(
		StorageVersion::<T>::get() == DelegationStorageVersion::V1,
		"Current deployed version is not v1."
	);

	log::info!("Version storage migrating from v1 to v2");
	Ok(())
}

/// It migrates the old storage entries to the new ones.
///
/// Specifically, for each entry in Roots, a new entry in
/// DelegationHierarchies + a new node in DelegationNodes is created.
/// Furthermore, nodes in Delegations are migrated to the new structure and
/// stored under DelegationNodes, with any children from the Children
/// storage entry added to the nodes under the children set.
pub(crate) fn migrate<T: Config>() -> Weight {
	log::info!("v1 -> v2 delegation storage migrator started!");
	let mut total_weight = Weight::zero();

	// Before being stored, the nodes are saved in a map so that after we go over
	// all the nodes and the parent-child relationship in the storage, we can update
	// the `parent` link of each node accordingly. Otherwise, we would need to save
	// the node in the storage, and then retrieve it again to update the parent
	// link.
	let mut new_nodes: BTreeMap<DelegationNodeIdOf<T>, DelegationNodeV2<T>> = BTreeMap::new();

	// First iterate over the delegation roots and translate them to hierarchies.
	total_weight = total_weight.saturating_add(migrate_roots::<T>(&mut new_nodes));

	// Then iterate over the regular delegation nodes.
	total_weight = total_weight.saturating_add(migrate_nodes::<T>(&mut new_nodes, total_weight));

	// By now, all the children should have been correctly added to the nodes.
	// We now need to modify all the nodes that are children by adding a reference
	// to their parents.
	total_weight = total_weight.saturating_add(finalize_children_nodes::<T>(&mut new_nodes, total_weight));

	StorageVersion::<T>::set(DelegationStorageVersion::V2);
	// Adds a write from StorageVersion::set() weight.
	total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));
	log::debug!("Total weight consumed: {}", total_weight);
	log::info!("v1 -> v2 delegation storage migrator finished!");

	total_weight
}

fn migrate_roots<T: Config>(new_nodes: &mut BTreeMap<DelegationNodeIdOf<T>, DelegationNodeV2<T>>) -> Weight {
	let total_weight = Roots::<T>::iter().fold(Weight::zero(), |mut total_weight, (old_root_id, old_root_node)| {
		let new_hierarchy_details = DelegationHierarchyDetails::<T> {
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
		let mut new_root_node = DelegationNodeV2::new_root_node(old_root_id, new_root_details);
		if let Some(root_children_ids) = Children::<T>::take(old_root_id) {
			let children_set: BTreeSet<DelegationNodeIdOf<T>> = root_children_ids.iter().copied().collect();
			new_root_node.children = BoundedBTreeSet::try_from(children_set).expect("Should not exceed MaxChildren");
		}
		// Add Children::take() weight
		total_weight = total_weight.saturating_add(T::DbWeight::get().reads(1));

		DelegationHierarchies::insert(old_root_id, new_hierarchy_details);
		// Adds a read from Roots::drain() and a write from
		// DelegationHierarchies::insert() weights
		total_weight = total_weight.saturating_add(T::DbWeight::get().reads_writes(1, 1));
		// Add the node to the temporary map of nodes to be added at the end.
		new_nodes.insert(old_root_id, new_root_node);

		total_weight
	});

	// If runtime testing, makes sure that the old number of roots is reflected in
	// the new number of nodes and hierarchies migrated.
	#[cfg(feature = "try-runtime")]
	{
		assert_eq!(
			Roots::<T>::iter().count(),
			DelegationHierarchies::<T>::iter().count(),
			"The # of old roots does not match the # of new delegation hierarchies."
		);

		assert_eq!(
			Roots::<T>::iter().count(),
			new_nodes.iter().count(),
			"The # of old roots does not match the current # of new delegation nodes."
		);

		log::info!("{} root(s) migrated.", Roots::<T>::iter().count());
	}

	// Removes the whole Roots storage.
	frame_support::migration::remove_storage_prefix(Roots::<T>::module_prefix(), Roots::<T>::storage_prefix(), b"");

	total_weight
}

fn migrate_nodes<T: Config>(
	new_nodes: &mut BTreeMap<DelegationNodeIdOf<T>, DelegationNodeV2<T>>,
	initial_weight: Weight,
) -> Weight {
	let total_weight = Delegations::<T>::iter().fold(initial_weight, |mut total_weight, (old_node_id, old_node)| {
		let new_node_details = DelegationDetails::<T> {
			owner: old_node.owner,
			permissions: old_node.permissions,
			revoked: old_node.revoked,
		};
		// In the old version, a parent None indicated the node is a child of the root.
		let new_node_parent_id = old_node.parent.unwrap_or(old_node.root_id);
		let mut new_node = DelegationNodeV2::new_node(old_node.root_id, new_node_parent_id, new_node_details);
		if let Some(children_ids) = Children::<T>::take(old_node_id) {
			let children_set: BTreeSet<DelegationNodeIdOf<T>> = children_ids.iter().copied().collect();
			new_node.children = BoundedBTreeSet::try_from(children_set).expect("Should not exceed MaxChildren");
		}
		// Add Children::take() weight
		total_weight = total_weight.saturating_add(T::DbWeight::get().reads(1));
		// Adds a read from Roots::drain() weight
		total_weight = total_weight.saturating_add(T::DbWeight::get().reads(1));
		new_nodes.insert(old_node_id, new_node);

		total_weight
	});

	// If runtime testing, makes sure that the old number of delegations is
	// reflected in the new number of nodes that will be added to the storage.
	#[cfg(feature = "try-runtime")]
	{
		assert_eq!(
			Delegations::<T>::iter().count(),
			new_nodes.iter().count().saturating_sub(DelegationHierarchies::<T>::iter().count()),
			"The # of old delegation nodes does not match the # of new delegation nodes (calculate as the total # of nodes - the # of delegation hierarchies)."
		);

		log::info!("{} regular node(s) migrated.", Delegations::<T>::iter().count());
	}

	// Removes the whole Delegations and Children storages.
	frame_support::migration::remove_storage_prefix(
		Delegations::<T>::module_prefix(),
		Delegations::<T>::storage_prefix(),
		b"",
	);
	frame_support::migration::remove_storage_prefix(
		Children::<T>::module_prefix(),
		Children::<T>::storage_prefix(),
		b"",
	);

	total_weight
}

fn finalize_children_nodes<T: Config>(
	new_nodes: &mut BTreeMap<DelegationNodeIdOf<T>, DelegationNodeV2<T>>,
	initial_weight: Weight,
) -> Weight {
	new_nodes
		.clone()
		.into_iter()
		.fold(initial_weight, |mut total_weight, (new_node_id, new_node)| {
			// Iterate over the children of every node and update their parent link.
			new_node.children.iter().for_each(|child_id| {
				new_nodes
					.entry(*child_id)
					.and_modify(|node| node.parent = Some(new_node_id));
			});
			// We can then finally insert the new delegation node in the storage as it won't
			// be updated anymore during the migration.
			DelegationNodes::<T>::insert(new_node_id, new_node);
			// Adds a write  from DelegationNodes::insert() weight
			total_weight = total_weight.saturating_add(T::DbWeight::get().writes(1));

			total_weight
		})
}

/// Checks whether the deployed storage version is v2 and whether any
/// parent-child link has gone missing.
#[cfg(feature = "try-runtime")]
pub(crate) fn post_migrate<T: Config>() -> Result<(), &'static str> {
	frame_support::ensure!(
		StorageVersion::<T>::get() == DelegationStorageVersion::V2,
		"The version after deployment is not 2 as expected."
	);
	frame_support::ensure!(
		verify_parent_children_integrity::<T>(),
		"Some parent-child relationship has been broken in the migration."
	);
	log::info!("Version storage migrated from v1 to v2");
	Ok(())
}

// Verifies that for any node that has a parent, the parent includes that node
// in its children.
#[cfg(feature = "try-runtime")]
fn verify_parent_children_integrity<T: Config>() -> bool {
	// If all's good and false is returned, returns true.
	!DelegationNodes::<T>::iter().any(|(node_id, node)| {
		if let Some(parent_id) = node.parent {
			if let Some(parent_node) = DelegationNodes::<T>::get(parent_id) {
				// True if the children set does not contain the parent ID
				return !parent_node.children.contains(&node_id);
			} else {
				// If the parent node cannot be found, it is definitely an error, so return
				// true.
				return true;
			}
		}
		// If all's good we keep returning false.
		false
	})
}

// Tests for the v1 storage migrator.
#[cfg(test)]
mod tests {
	use sp_core::Pair;

	use super::*;
	use crate::{
		deprecated::v1::{DelegationNode as DelegationNodeV1, DelegationRoot},
		mock,
		mock::Test as TestRuntime,
	};

	#[test]
	fn fail_version_higher() {
		mock::ExtBuilder::default()
			.with_storage_version(DelegationStorageVersion::V2)
			.build()
			.execute_with(|| {
				#[cfg(feature = "try-runtime")]
				assert!(
					pre_migrate::<TestRuntime>().is_err(),
					"Pre-migration for v1 should fail."
				);
			});
	}

	#[test]
	fn ok_no_delegations() {
		mock::ExtBuilder::default()
			.with_storage_version(DelegationStorageVersion::V1)
			.build()
			.execute_with(|| {
				#[cfg(feature = "try-runtime")]
				assert!(
					pre_migrate::<TestRuntime>().is_ok(),
					"Pre-migration for v1 should not fail."
				);

				migrate::<TestRuntime>();

				#[cfg(feature = "try-runtime")]
				assert!(
					post_migrate::<TestRuntime>().is_ok(),
					"Post-migration for v1 should not fail."
				);
			});
	}

	#[test]
	fn ok_only_root() {
		mock::ExtBuilder::default()
			.with_storage_version(DelegationStorageVersion::V1)
			.build()
			.execute_with(|| {
				let alice = mock::get_ed25519_account(mock::get_alice_ed25519().public());
				let old_root_id = mock::get_delegation_id(true);
				let old_root_node =
					DelegationRoot::<TestRuntime>::new(ctype::mock::get_ctype_hash::<TestRuntime>(true), alice);
				Roots::insert(old_root_id, old_root_node.clone());

				#[cfg(feature = "try-runtime")]
				assert!(
					pre_migrate::<TestRuntime>().is_ok(),
					"Pre-migration for v1 should not fail."
				);

				migrate::<TestRuntime>();

				#[cfg(feature = "try-runtime")]
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
		mock::ExtBuilder::default()
			.with_storage_version(DelegationStorageVersion::V1)
			.build()
			.execute_with(|| {
				let alice = mock::get_ed25519_account(mock::get_alice_ed25519().public());
				let bob = mock::get_sr25519_account(mock::get_bob_sr25519().public());
				let old_root_id = mock::get_delegation_id(true);
				let old_root_node =
					DelegationRoot::<TestRuntime>::new(ctype::mock::get_ctype_hash::<TestRuntime>(true), alice.clone());
				let old_node_id_1 = mock::get_delegation_id(false);
				let old_node_1 =
					DelegationNodeV1::<TestRuntime>::new_root_child(old_root_id, alice, Permissions::DELEGATE);
				let old_node_id_2 = mock::get_delegation_id_2(true);
				let old_node_2 = DelegationNodeV1::<TestRuntime>::new_root_child(old_root_id, bob, Permissions::ATTEST);
				Roots::insert(old_root_id, old_root_node.clone());
				Delegations::insert(old_node_id_1, old_node_1.clone());
				Delegations::insert(old_node_id_2, old_node_2.clone());
				Children::<TestRuntime>::insert(old_root_id, vec![old_node_id_1, old_node_id_2]);

				#[cfg(feature = "try-runtime")]
				assert!(
					pre_migrate::<TestRuntime>().is_ok(),
					"Pre-migration for v1 should not fail."
				);

				migrate::<TestRuntime>();

				#[cfg(feature = "try-runtime")]
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

	#[test]
	fn ok_three_level_hierarchy() {
		mock::ExtBuilder::default()
			.with_storage_version(DelegationStorageVersion::V1)
			.build()
			.execute_with(|| {
				let alice = mock::get_ed25519_account(mock::get_alice_ed25519().public());
				let bob = mock::get_sr25519_account(mock::get_bob_sr25519().public());
				let old_root_id = mock::get_delegation_id(true);
				let old_root_node =
					DelegationRoot::<TestRuntime>::new(ctype::mock::get_ctype_hash::<TestRuntime>(true), alice.clone());
				let old_parent_id = mock::get_delegation_id(false);
				let old_parent_node =
					DelegationNodeV1::<TestRuntime>::new_root_child(old_root_id, alice, Permissions::all());
				let old_node_id = mock::get_delegation_id_2(true);
				let old_node = DelegationNodeV1::<TestRuntime>::new_node_child(
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

				#[cfg(feature = "try-runtime")]
				assert!(
					pre_migrate::<TestRuntime>().is_ok(),
					"Pre-migration for v1 should not fail."
				);

				migrate::<TestRuntime>();

				#[cfg(feature = "try-runtime")]
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
}
