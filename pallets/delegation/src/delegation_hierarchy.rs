// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2023 BOTLabs GmbH

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

use bitflags::bitflags;
use frame_support::{storage::bounded_btree_set::BoundedBTreeSet, traits::Get};
use kilt_support::deposit::Deposit;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

bitflags! {
	/// Bitflags for permissions.
	///
	/// Permission bits can be combined to express multiple permissions.
	#[derive(Encode, Decode, TypeInfo, MaxEncodedLen)]
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

/// A node in a delegation hierarchy.
///
/// For quicker lookups of the hierarchy details, all nodes maintain a direct
/// link to the hierarchy root node. Furthermore, all nodes have a parent except
/// the root nodes, which point to themselves for the hierarchy root node link.
#[derive(Clone, Debug, Encode, Decode, Eq, PartialEq, MaxEncodedLen, TypeInfo)]
pub struct DelegationNode<DelegationNodeId, MaxChildren: Get<u32>, DelegationDetails, AccountId, Balance> {
	/// The ID of the delegation hierarchy the node is part of.
	pub hierarchy_root_id: DelegationNodeId,
	/// The ID of the parent. For all but root nodes this is not None.
	pub parent: Option<DelegationNodeId>,
	/// The set of IDs of all the children nodes.
	pub children: BoundedBTreeSet<DelegationNodeId, MaxChildren>,
	/// The additional information attached to the delegation node.
	pub details: DelegationDetails,
	/// The deposit that was taken to incentivise fair use of the on chain
	/// storage.
	pub deposit: Deposit<AccountId, Balance>,
}

impl<DelegationNodeId: Ord, MaxChildren: Get<u32>, DelegationDetails, AccountId, Balance>
	DelegationNode<DelegationNodeId, MaxChildren, DelegationDetails, AccountId, Balance>
{
	/// Creates a new delegation root node with the given ID and delegation
	/// details.
	pub fn new_root_node(
		id: DelegationNodeId,
		details: DelegationDetails,
		deposit_owner: AccountId,
		deposit_amount: Balance,
	) -> Self {
		Self {
			hierarchy_root_id: id,
			parent: None,
			children: BoundedBTreeSet::<DelegationNodeId, MaxChildren>::new(),
			details,
			deposit: Deposit::<AccountId, Balance> {
				owner: deposit_owner,
				amount: deposit_amount,
			},
		}
	}

	/// Creates a new delegation node under the given hierarchy ID, with the
	/// given parent and delegation details.
	pub fn new_node(
		hierarchy_root_id: DelegationNodeId,
		parent: DelegationNodeId,
		details: DelegationDetails,
		deposit_owner: AccountId,
		deposit_amount: Balance,
	) -> Self {
		Self {
			hierarchy_root_id,
			parent: Some(parent),
			children: BoundedBTreeSet::<DelegationNodeId, MaxChildren>::new(),
			details,
			deposit: Deposit::<AccountId, Balance> {
				owner: deposit_owner,
				amount: deposit_amount,
			},
		}
	}

	/// Adds a node by its ID to the current node's children.
	pub fn try_add_child(&mut self, child_id: DelegationNodeId) -> Result<(), DelegationNodeId> {
		self.children.try_insert(child_id)?;
		Ok(())
	}
}

/// Delegation information attached to delegation nodes.
#[derive(Clone, Debug, Encode, Decode, Eq, PartialEq, TypeInfo, MaxEncodedLen)]
pub struct DelegationDetails<DelegatorId> {
	/// The owner of the delegation (and its node).
	pub owner: DelegatorId,
	/// Status indicating whether the delegation has been revoked (true) or not
	/// (false).
	pub revoked: bool,
	/// The set of permissions associated with the delegation.
	pub permissions: Permissions,
}

impl<DelegatorId> DelegationDetails<DelegatorId> {
	/// Creates new delegation details including the given owner.
	///
	/// The default revocation status is false and all permissions are granted
	/// by default.
	pub fn default_with_owner(owner: DelegatorId) -> Self {
		Self {
			owner,
			permissions: Permissions::all(),
			revoked: false,
		}
	}
}

/// The details associated with a delegation hierarchy.
#[derive(Clone, Debug, Encode, Decode, Eq, PartialEq, Ord, PartialOrd, TypeInfo, MaxEncodedLen)]
pub struct DelegationHierarchyDetails<CtypeHash> {
	/// The authorised CTYPE hash that attesters can attest using this
	/// delegation hierarchy.
	pub ctype_hash: CtypeHash,
}
