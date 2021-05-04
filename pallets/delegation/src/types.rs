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
pub use bitflags::*;
pub use codec::{Decode, Encode};

use crate::Config;

/// Type of a delegation node identifier.
pub type DelegationNodeId<T> = <T as Config>::DelegationNodeId;

/// Type of a delegation node's owner.
pub type DelegatorId<T> = did::DidIdentifier<T>;

/// The type of a CTYPE hash.
pub type CtypeHash<T> = ctype::CtypeHash<T>;

/// Type of a signature over the delegation details.
pub type DelegationSignature = did::DidSignature;

bitflags! {
	/// Bitflags for permissions.
	#[derive(Encode, Decode)]
	pub struct Permissions: u32 {
		/// Permission to write attestations on chain.
		const ATTEST = 0b0000_0001;
		/// Permission to write delegations on chain.
		const DELEGATE = 0b0000_0010;
	}
}

impl Permissions {
	/// Encode permission bitflags into u8 array.
	pub fn as_u8(self) -> [u8; 4] {
		let x: u32 = self.bits;
		let b1: u8 = ((x >> 24) & 0xff) as u8;
		let b2: u8 = ((x >> 16) & 0xff) as u8;
		let b3: u8 = ((x >> 8) & 0xff) as u8;
		let b4: u8 = (x & 0xff) as u8;
		[b4, b3, b2, b1]
	}
}

impl Default for Permissions {
	/// Default permissions to the attest permission.
	fn default() -> Self {
		Permissions::ATTEST
	}
}

/// A node representing a delegation hierarchy root.
#[derive(Clone, Debug, Encode, Decode, PartialEq)]
pub struct DelegationRoot<T: Config> {
	/// The hash of the CTYPE that delegated attesters can attest.
	pub ctype_hash: CtypeHash<T>,
	/// The DID of the root owner.
	pub owner: DelegatorId<T>,
	/// The flag indicating whether the root has been revoked or not.
	pub revoked: bool,
}

impl<T: Config> DelegationRoot<T> {
	pub fn new(ctype_hash: CtypeHash<T>, owner: DelegatorId<T>) -> Self {
		DelegationRoot {
			ctype_hash,
			owner,
			revoked: false,
		}
	}
}

/// A node representing a node in the deleagation hierarchy.
#[derive(Clone, Debug, Encode, Decode, PartialEq)]
pub struct DelegationNode<T: Config> {
	/// The ID of the delegation hierarchy root.
	pub root_id: DelegationNodeId<T>,
	/// \[OPTIONAL\] The ID of the parent node. If None, the node is
	/// considered a child of the root node.
	pub parent: Option<DelegationNodeId<T>>,
	/// The DID of the owner of the delegation node, i.e., the delegate.
	pub owner: DelegatorId<T>,
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
	/// * owner: the DID of the owner of the new delegation, i.e., the new
	///   delegate
	/// * permissions: the permission flags for the operations the delegate is
	///   allowed to perform
	pub fn new_root_child(root_id: DelegationNodeId<T>, owner: DelegatorId<T>, permissions: Permissions) -> Self {
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
	/// * owner: the DID of the owner of the new delegation, i.e., the new
	///   delegate
	/// * permissions: the permission flags for the operations the delegate is
	///   allowed to perform
	pub fn new_node_child(
		root_id: DelegationNodeId<T>,
		parent: DelegationNodeId<T>,
		owner: DelegatorId<T>,
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
