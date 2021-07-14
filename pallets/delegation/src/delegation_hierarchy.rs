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

pub use v1::*;

use bitflags::bitflags;
use codec::{Decode, Encode};
use ctype::CtypeHashOf;
use sp_std::collections::btree_set::BTreeSet;

use crate::*;

bitflags! {
	/// Bitflags for permissions.
	///
	/// Permission bits can be combined to express multiple permissions.
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
	fn default() -> Self {
		Permissions::ATTEST
	}
}

pub(crate) mod v0 {
	use super::*;

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
		/// The identifier of the owner of the delegation node, i.e., the delegate.
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
		/// * owner: the identifier of the owner of the new delegation, i.e., the
		///   new delegate
		/// * permissions: the permission flags for the operations the delegate is
		///   allowed to perform
		pub fn new_root_child(root_id: DelegationNodeIdOf<T>, owner: DelegatorIdOf<T>, permissions: Permissions) -> Self {
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
		/// * owner: the identifier of the owner of the new delegation, i.e., the
		///   new delegate
		/// * permissions: the permission flags for the operations the delegate is
		///   allowed to perform
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

pub(crate) mod v1 {
	use super::*;

	#[derive(Clone, Debug, Encode, Decode, PartialEq)]
	pub struct DelegationNode<T: Config> {
		pub hierarchy_root_id: DelegationNodeIdOf<T>,
		pub parent: Option<DelegationNodeIdOf<T>>,
		pub children: BTreeSet<DelegationNodeIdOf<T>>,
		pub details: DelegationDetails<T>,
	}

	impl<T: Config> DelegationNode<T> {
		pub fn new_root_node(id: DelegationNodeIdOf<T>, details: DelegationDetails<T>) -> Self {
			Self {
				hierarchy_root_id: id,
				parent: None,
				children: BTreeSet::new(),
				details,
			}
		}

		pub fn new_node(
			hierarchy_root_id: DelegationNodeIdOf<T>,
			parent: DelegationNodeIdOf<T>,
			details: DelegationDetails<T>,
		) -> Self {
			let mut new_node = Self::new_root_node(hierarchy_root_id, details);
			new_node.parent = Some(parent);

			new_node
		}

		pub fn add_child(&mut self, child_id: DelegationNodeIdOf<T>) {
			self.children.insert(child_id);
		}
	}

	#[derive(Clone, Debug, Encode, Decode, PartialEq)]
	pub struct DelegationDetails<T: Config> {
		pub owner: DelegatorIdOf<T>,
		pub revoked: bool,
		pub permissions: Permissions,
	}

	impl<T: Config> DelegationDetails<T> {
		pub fn default_with_owner(owner: DelegatorIdOf<T>) -> Self {
			Self {
				owner,
				permissions: Permissions::all(),
				revoked: false,
			}
		}
	}

	#[derive(Clone, Debug, Encode, Decode, Eq, PartialEq, Ord, PartialOrd)]
	pub struct DelegationHierarchyInfo<T: Config> {
		pub ctype_hash: CtypeHashOf<T>,
	}
}

/// The result that the delegation pallet expects from the implementer of the
/// delegate's signature verification operation.
pub type SignatureVerificationResult = Result<(), SignatureVerificationError>;

/// Types of errors the signature verification is expected to generate.
pub enum SignatureVerificationError {
	/// The delegate's information is not present on chain.
	SignerInformationNotPresent,
	/// The signature over the delegation information is invalid.
	SignatureInvalid,
}

/// Trait to implement to provide to the delegation pallet signature
/// verification over a delegation details.
pub trait VerifyDelegateSignature {
	/// The type of the delegate identifier.
	type DelegateId;
	/// The type of the encoded delegation details.
	type Payload;
	/// The type of the signature generated.
	type Signature;

	/// Verifies that the signature matches the payload and has been generated
	/// by the delegate.
	fn verify(
		delegate: &Self::DelegateId,
		payload: &Self::Payload,
		signature: &Self::Signature,
	) -> SignatureVerificationResult;
}
