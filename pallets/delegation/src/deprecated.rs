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

pub(crate) mod v0 {
	use codec::{Decode, Encode};

	use crate::*;

	/// A node representing a delegation hierarchy root.
	#[derive(Clone, Debug, Encode, Decode, PartialEq)]
	pub struct DelegationRoot<T: Config> {
		/// The hash of the CTYPE that delegated attesters within this trust
		/// hierarchy can attest.
		pub ctype_hash: CtypeHashOf<T>,
		/// The identifier of the root owner.
		pub owner: DelegatorIdOf<T>,
		/// The flag indicating whether the root has been revoked or not.
		pub revoked: bool,
	}

	impl<T: Config> DelegationRoot<T> {
		pub fn new(ctype_hash: CtypeHashOf<T>, owner: DelegatorIdOf<T>) -> Self {
			DelegationRoot {
				ctype_hash,
				owner,
				revoked: false,
			}
		}
	}

	/// A node representing a node in the delegation hierarchy.
	#[derive(Clone, Debug, Encode, Decode, PartialEq)]
	pub struct DelegationNode<T: Config> {
		/// The ID of the delegation hierarchy root.
		pub root_id: DelegationNodeIdOf<T>,
		/// \[OPTIONAL\] The ID of the parent node. If None, the node is
		/// considered a direct child of the root node.
		pub parent: Option<DelegationNodeIdOf<T>>,
		/// The identifier of the owner of the delegation node, i.e., the
		/// delegate.
		pub owner: DelegatorIdOf<T>,
		/// The permission flags for the operations the delegate is allowed to
		/// perform.
		pub permissions: Permissions,
		/// The flag indicating whether the delegation has been revoked or not.
		pub revoked: bool,
	}

	impl<T: Config> DelegationNode<T> {
		/// Create a new delegation node that is a direct descendent of the
		/// given root.
		///
		/// * root_id: the root node ID this node will be a child of
		/// * owner: the identifier of the owner of the new delegation, i.e.,
		///   the new delegate
		/// * permissions: the permission flags for the operations the delegate
		///   is allowed to perform
		pub fn new_root_child(
			root_id: DelegationNodeIdOf<T>,
			owner: DelegatorIdOf<T>,
			permissions: Permissions,
		) -> Self {
			DelegationNode {
				root_id,
				owner,
				permissions,
				revoked: false,
				parent: None,
			}
		}

		/// Creates a new delegation node that is a direct descendent of the
		/// given node.
		///
		/// * root_id: the root node ID this node will be a child of
		/// * parent - the parent node ID this node will be a child of
		/// * owner: the identifier of the owner of the new delegation, i.e.,
		///   the new delegate
		/// * permissions: the permission flags for the operations the delegate
		///   is allowed to perform
		pub fn new_node_child(
			root_id: DelegationNodeIdOf<T>,
			parent: DelegationNodeIdOf<T>,
			owner: DelegatorIdOf<T>,
			permissions: Permissions,
		) -> Self {
			DelegationNode {
				root_id,
				parent: Some(parent),
				owner,
				permissions,
				revoked: false,
			}
		}
	}
}
