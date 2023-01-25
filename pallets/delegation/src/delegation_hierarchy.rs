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

use crate::{AccountIdOf, BalanceOf, Config, DelegationNodeIdOf, DelegatorIdOf, Error};
use bitflags::bitflags;
use codec::{Decode, Encode, MaxEncodedLen};
use ctype::CtypeHashOf;
use frame_support::{dispatch::DispatchResult, storage::bounded_btree_set::BoundedBTreeSet};
use kilt_support::deposit::Deposit;
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
#[derive(Clone, Encode, Decode, PartialEq, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct DelegationNode<T: Config> {
	/// The ID of the delegation hierarchy the node is part of.
	pub hierarchy_root_id: DelegationNodeIdOf<T>,
	/// The ID of the parent. For all but root nodes this is not None.
	pub parent: Option<DelegationNodeIdOf<T>>,
	/// The set of IDs of all the children nodes.
	pub children: BoundedBTreeSet<DelegationNodeIdOf<T>, T::MaxChildren>,
	/// The additional information attached to the delegation node.
	pub details: DelegationDetails<T>,
	/// The deposit that was taken to incentivise fair use of the on chain
	/// storage.
	pub deposit: Deposit<AccountIdOf<T>, BalanceOf<T>>,
}

impl<T: Config> DelegationNode<T> {
	/// Creates a new delegation root node with the given ID and delegation
	/// details.
	pub fn new_root_node(
		id: DelegationNodeIdOf<T>,
		details: DelegationDetails<T>,
		deposit_owner: AccountIdOf<T>,
		deposit_amount: BalanceOf<T>,
	) -> Self {
		Self {
			hierarchy_root_id: id,
			parent: None,
			children: BoundedBTreeSet::<DelegationNodeIdOf<T>, T::MaxChildren>::new(),
			details,
			deposit: Deposit::<AccountIdOf<T>, BalanceOf<T>> {
				owner: deposit_owner,
				amount: deposit_amount,
			},
		}
	}

	/// Creates a new delegation node under the given hierarchy ID, with the
	/// given parent and delegation details.
	pub fn new_node(
		hierarchy_root_id: DelegationNodeIdOf<T>,
		parent: DelegationNodeIdOf<T>,
		details: DelegationDetails<T>,
		deposit_owner: AccountIdOf<T>,
		deposit_amount: BalanceOf<T>,
	) -> Self {
		Self {
			hierarchy_root_id,
			parent: Some(parent),
			children: BoundedBTreeSet::<DelegationNodeIdOf<T>, T::MaxChildren>::new(),
			details,
			deposit: Deposit::<AccountIdOf<T>, BalanceOf<T>> {
				owner: deposit_owner,
				amount: deposit_amount,
			},
		}
	}

	/// Adds a node by its ID to the current node's children.
	pub fn try_add_child(&mut self, child_id: DelegationNodeIdOf<T>) -> DispatchResult {
		self.children
			.try_insert(child_id)
			.map_err(|_| Error::<T>::MaxChildrenExceeded)?;
		Ok(())
	}
}

/// Delegation information attached to delegation nodes.
#[derive(Clone, Debug, Encode, Decode, Eq, PartialEq, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]
pub struct DelegationDetails<T: Config> {
	/// The owner of the delegation (and its node).
	pub owner: DelegatorIdOf<T>,
	/// Status indicating whether the delegation has been revoked (true) or not
	/// (false).
	pub revoked: bool,
	/// The set of permissions associated with the delegation.
	pub permissions: Permissions,
}

impl<T: Config> DelegationDetails<T> {
	/// Creates new delegation details including the given owner.
	///
	/// The default revocation status is false and all permissions are granted
	/// by default.
	pub fn default_with_owner(owner: DelegatorIdOf<T>) -> Self {
		Self {
			owner,
			permissions: Permissions::all(),
			revoked: false,
		}
	}
}

/// The details associated with a delegation hierarchy.
#[derive(Clone, Debug, Encode, Decode, Eq, PartialEq, Ord, PartialOrd, TypeInfo, MaxEncodedLen)]
#[scale_info(skip_type_params(T))]
#[codec(mel_bound())]

pub struct DelegationHierarchyDetails<T: Config> {
	/// The authorised CTYPE hash that attesters can attest using this
	/// delegation hierarchy.
	pub ctype_hash: CtypeHashOf<T>,
}
