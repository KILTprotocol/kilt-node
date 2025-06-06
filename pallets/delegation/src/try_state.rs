// KILT Blockchain – <https://kilt.io>
// Copyright (C) 2025, KILT Foundation

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

// If you feel like getting in touch with us, you can do so at <hello@kilt.io>

use frame_support::ensure;
use kilt_support::test_utils::log_and_return_error_message;
use scale_info::prelude::format;
use sp_runtime::TryRuntimeError;

use crate::{Config, DelegationHierarchies, DelegationNodeOf, DelegationNodes};

pub(crate) fn do_try_state<T: Config>() -> Result<(), TryRuntimeError> {
	DelegationNodes::<T>::iter().try_for_each(
		|(delegation_node_id, delegation_details)| -> Result<(), TryRuntimeError> {
			let hierarchy_id = delegation_details.hierarchy_root_id;

			// check if node is in part of a delegation hierarchy.
			ensure!(
				DelegationHierarchies::<T>::contains_key(hierarchy_id),
				log_and_return_error_message(format!("Delegation hierarchy {:?} not found", hierarchy_id))
			);

			// Delegation hierarchy should be a tree.
			check_no_cycles::<T>(&delegation_details, &delegation_node_id)?;

			// if a node is revoked, the subtree should be revoked as well.
			check_subtree_is_revoked::<T>(delegation_details, &delegation_node_id)
		},
	)
}

fn get_merged_subtree<T: Config>(node: DelegationNodeOf<T>) -> sp_std::vec::Vec<DelegationNodeOf<T>> {
	let mut nodes_to_explore = sp_std::vec::Vec::from([node]);
	let mut children = sp_std::vec::Vec::new();
	while let Some(current_node) = nodes_to_explore.pop() {
		let child_nodes = current_node.children.iter().filter_map(DelegationNodes::<T>::get);
		nodes_to_explore.extend(child_nodes.clone());
		children.extend(child_nodes);
	}
	children
}

fn check_subtree_is_revoked<T: Config>(
	delegation_details: DelegationNodeOf<T>,
	delegation_node_id: &<T as Config>::DelegationNodeId,
) -> Result<(), TryRuntimeError> {
	if delegation_details.details.revoked {
		let is_subtree_revoked = get_merged_subtree::<T>(delegation_details)
			.iter()
			.map(|child: &DelegationNodeOf<T>| child.details.revoked)
			.all(|x| x);
		ensure!(
			is_subtree_revoked,
			log_and_return_error_message(format!(
				"Revoked delegation node {:?} has an unrevoked subtree.",
				delegation_node_id
			))
		);
	}
	Ok(())
}

fn check_no_cycles<T: Config>(
	delegation_details: &DelegationNodeOf<T>,
	delegation_node_id: &<T as Config>::DelegationNodeId,
) -> Result<(), TryRuntimeError> {
	let parent_count = DelegationNodes::<T>::iter_values()
		.filter(|delegation_node: &DelegationNodeOf<T>| delegation_node.children.contains(delegation_node_id))
		.count();

	if delegation_details.parent.is_some() {
		// If node is a leaf or intermediate, check if it occurs only once. Otherwise we
		// have cycles.
		ensure!(
			parent_count == 1,
			log_and_return_error_message(format!(
				"Delegation with cycles detected. Node {:?} in hierarchy {:?} has two or more parents.",
				delegation_node_id, delegation_details.hierarchy_root_id
			))
		);
	} else {
		// if parent is None, check that the root is not the children
		// from another node.
		ensure!(
			parent_count == 0,
			log_and_return_error_message(format!(
				"Root node {:?} is child from other delegation nodes",
				delegation_node_id
			))
		);
	}
	Ok(())
}
