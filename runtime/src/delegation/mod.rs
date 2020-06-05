// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019  BOTLabs GmbH

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

//! Delegation: Handles delegations on chain,
//! creating and revoking root nodes of delegation hierarchies,
//! adding and revoking delegation nodes based on root nodes.

/// Test module for delegations
#[cfg(test)]
mod tests;

use sp_std::{
	prelude::{Clone, Eq, PartialEq, Vec},
	result,
};

use codec::{Decode, Encode};
use core::default::Default;
use support::{
	debug, decl_event, decl_module, decl_storage, dispatch::DispatchResult, Parameter, StorageMap,
};

use super::{ctype, error};
use sp_runtime::{
	codec::Codec,
	traits::{CheckEqual, Hash, IdentifyAccount, MaybeDisplay, Member, SimpleBitOps, Verify},
	verify_encoded_lazy,
};
use system::{self, ensure_signed};

bitflags! {
	/// Bitflags for permissions
	#[derive(Encode, Decode)]
	pub struct Permissions: u32 {
		/// Bit flag for attestation permission
		const ATTEST = 0b0000_0001;
		/// Bit flag for delegation permission
		const DELEGATE = 0b0000_0010;
	}
}

/// Implementation for permissions
impl Permissions {
	/// Encode permission bitflags into u8 array
	fn as_u8(self) -> [u8; 4] {
		let x: u32 = self.bits;
		let b1: u8 = ((x >> 24) & 0xff) as u8;
		let b2: u8 = ((x >> 16) & 0xff) as u8;
		let b3: u8 = ((x >> 8) & 0xff) as u8;
		let b4: u8 = (x & 0xff) as u8;
		[b4, b3, b2, b1]
	}
}

/// Implement Default trait for permissions
impl Default for Permissions {
	/// Default permissions to the attest permission
	fn default() -> Self {
		Permissions::ATTEST
	}
}

/// The delegation trait
pub trait Trait: ctype::Trait + system::Trait + error::Trait {
	/// Delegation specific event type
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

	/// Signature of a delegation
	type Signature: Verify<Signer = Self::Signer> + Member + Codec + Default;

	/// Signer of a delegation
	// type Signer: From<Self::AccountId> + IdentifyAccount<AccountId = Self::AccountId>> + Member + Codec;
	type Signer: IdentifyAccount<AccountId = Self::AccountId> + Member + Codec;

	/// Delegation node id type
	type DelegationNodeId: Parameter
		+ Member
		+ Codec
		+ MaybeDisplay
		+ SimpleBitOps
		+ Default
		+ Copy
		+ CheckEqual
		+ sp_std::hash::Hash
		+ AsRef<[u8]>
		+ AsMut<[u8]>;
}

decl_event!(
	/// Events for delegations
	pub enum Event<T> where <T as system::Trait>::Hash, <T as system::Trait>::AccountId,
			<T as Trait>::DelegationNodeId {
		/// A new root has been created
		RootCreated(AccountId, DelegationNodeId, Hash),
		/// A root has been revoked
		RootRevoked(AccountId, DelegationNodeId),
		/// A new delegation has been created
		DelegationCreated(AccountId, DelegationNodeId, DelegationNodeId, Option<DelegationNodeId>,
				AccountId, Permissions),
		/// A delegation has been revoked
		DelegationRevoked(AccountId, DelegationNodeId),
	}
);

decl_module! {
	/// The delegation runtime module
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		/// Deposit events
		fn deposit_event() = default;

		/// Creates a delegation hierarchy root on chain, where
		/// origin - the origin of the transaction
		/// root_id - unique identifier of the root node
		/// ctype_hash - hash of the CTYPE the hierarchy is created for
		#[weight = 1]
		pub fn create_root(origin, root_id: T::DelegationNodeId, ctype_hash: T::Hash) -> DispatchResult {
			// origin of the transaction needs to be a signed sender account
			let sender = ensure_signed(origin)?;
			// check if a root with the given id already exists
			if <Root<T>>::contains_key(root_id) {
				return Self::error(Self::ERROR_ROOT_ALREADY_EXISTS);
			}
			// check if CTYPE exists
			if !<ctype::CTYPEs<T>>::contains_key(ctype_hash) {
				return Self::error(<ctype::Module<T>>::ERROR_CTYPE_NOT_FOUND);
			}

			// add root node to storage
			debug::print!("insert Delegation Root");
			<Root<T>>::insert(root_id, (ctype_hash, sender.clone(), false));
			// deposit event that the root node has been created
			Self::deposit_event(RawEvent::RootCreated(sender, root_id, ctype_hash));
			Ok(())
		}

		/// Adds a delegation node on chain, where
		/// origin - the origin of the transaction
		/// delegation_id - unique identifier of the delegation node to be added
		/// root_id - id of the hierarchy root node
		/// parent_id - optional identifier of a parent node this delegation node is created under
		/// delegate - the delegate account
		/// permission - the permissions delegated
		/// delegate_signature - the signature of the delegate to ensure it's done under his permission
		#[weight = 1]
		pub fn add_delegation(
			origin,
			delegation_id: T::DelegationNodeId,
			root_id: T::DelegationNodeId,
			parent_id: Option<T::DelegationNodeId>,
			delegate: T::AccountId,
			permissions: Permissions,
			delegate_signature: T::Signature
		) -> DispatchResult {
			// origin of the transaction needs to be a signed sender account
			let sender = ensure_signed(origin)?;
			// check if a delegation node with the given identifier already exists
			if <Delegations<T>>::contains_key(delegation_id) {
				return Self::error(Self::ERROR_DELEGATION_ALREADY_EXISTS);
			}
			// calculate the hash root and check if the signature matches
			let hash_root = Self::calculate_hash(delegation_id, root_id, parent_id, permissions);
			if !verify_encoded_lazy(&delegate_signature, &&hash_root, &delegate) {
				return Self::error(Self::ERROR_BAD_DELEGATION_SIGNATURE);
			}

			// check if root exists
			let root = <error::Module<T>>::ok_or_deposit_err(
				<Root<T>>::get(root_id),
				Self::ERROR_ROOT_NOT_FOUND
			)?;
			// check if this delegation has a parent
			if let Some(p) = parent_id {
				// check if the parent exists
				let parent = <error::Module<T>>::ok_or_deposit_err(
					<Delegations<T>>::get(p),
					Self::ERROR_PARENT_NOT_FOUND
				)?;
				// check if the parent's delegate is the sender of this transaction and has permission to delegate
				if !parent.2.eq(&sender) {
					return Self::error(Self::ERROR_NOT_OWNER_OF_PARENT);
				} else if (parent.3 & Permissions::DELEGATE) != Permissions::DELEGATE {
					return Self::error(Self::ERROR_NOT_AUTHORIZED_TO_DELEGATE);
				} else {
					// insert delegation
					debug::print!("insert Delegation with parent");
					<Delegations<T>>::insert(delegation_id, (root_id,
							Some(p), delegate.clone(), permissions, false));
					// add child to tree structure
					Self::add_child(delegation_id, p);
				}
			} else {
				// check if the sender of this transaction is the creator of the root node (as no parent is given)
				if !root.1.eq(&sender) {
					return Self::error(Self::ERROR_NOT_OWNER_OF_ROOT);
				}
				// inser delegation
				debug::print!("insert Delegation without parent");
				<Delegations<T>>::insert(delegation_id, (root_id,
						Option::<T::DelegationNodeId>::None, delegate.clone(), permissions, false));
				// add child to tree structure
				Self::add_child(delegation_id, root_id);
			}
			// deposit event that the delegation node has been added
			Self::deposit_event(RawEvent::DelegationCreated(sender, delegation_id,
					root_id, parent_id, delegate, permissions));
			Ok(())
		}

		/// Revoke the root and therefore a complete hierarchy, where
		/// origin - the origin of the transaction
		/// root_id - id of the hierarchy root node
		#[weight = 1]
		pub fn revoke_root(origin, root_id: T::DelegationNodeId) -> DispatchResult {
			// origin of the transaction needs to be a signed sender account
			let sender = ensure_signed(origin)?;
			// check if root node exists
			let mut r = <error::Module<T>>::ok_or_deposit_err(
				<Root<T>>::get(root_id),
				Self::ERROR_ROOT_NOT_FOUND
			)?;
			// check if root node has been created by the sender of this transaction
			if !r.1.eq(&sender) {
				return Self::error(Self::ERROR_NOT_PERMITTED_TO_REVOKE);
			}
			if !r.2 {
				// store revoked root node
				r.2 = true;
				<Root<T>>::insert(root_id, r);
				// recursively revoke all children
				Self::revoke_children(&root_id, &sender)?;
			}
			// deposit event that the root node has been revoked
			Self::deposit_event(RawEvent::RootRevoked(sender, root_id));
			Ok(())
		}

		/// Revoke a delegation node and all its children, where
		/// origin - the origin of the transaction
		/// delegation_id - id of the delegation node
		#[weight = 1]
		pub fn revoke_delegation(origin, delegation_id: T::DelegationNodeId) -> DispatchResult {
			// origin of the transaction needs to be a signed sender account
			let sender = ensure_signed(origin)?;
			// check if delegation node exists
			if !<Delegations<T>>::contains_key(delegation_id) {
				return Self::error(Self::ERROR_DELEGATION_NOT_FOUND)
			}
			// check if the sender of this transaction is permitted by being the
			// owner of the delegation or of one of its parents
			if !Self::is_delegating(&sender, &delegation_id)? {
				return Self::error(Self::ERROR_NOT_PERMITTED_TO_REVOKE)
			}
			// revoke the delegation and recursively all of its children
			Self::revoke(&delegation_id, &sender)?;
			Ok(())
		}
	}
}

/// Implementation of further module constants and functions for delegations
impl<T: Trait> Module<T> {
	/// Error types for errors in delegation module
	pub const ERROR_BASE: u16 = 3000;
	pub const ERROR_ROOT_ALREADY_EXISTS: error::ErrorType =
		(Self::ERROR_BASE + 1, "root already exist");
	pub const ERROR_NOT_PERMITTED_TO_REVOKE: error::ErrorType =
		(Self::ERROR_BASE + 2, "not permitted to revoke");
	pub const ERROR_DELEGATION_NOT_FOUND: error::ErrorType =
		(Self::ERROR_BASE + 3, "delegation not found");
	pub const ERROR_DELEGATION_ALREADY_EXISTS: error::ErrorType =
		(Self::ERROR_BASE + 4, "delegation already exist");
	pub const ERROR_BAD_DELEGATION_SIGNATURE: error::ErrorType =
		(Self::ERROR_BASE + 5, "bad delegate signature");
	pub const ERROR_NOT_OWNER_OF_PARENT: error::ErrorType =
		(Self::ERROR_BASE + 6, "not owner of parent");
	pub const ERROR_NOT_AUTHORIZED_TO_DELEGATE: error::ErrorType =
		(Self::ERROR_BASE + 7, "not authorized to delegate");
	pub const ERROR_PARENT_NOT_FOUND: error::ErrorType = (Self::ERROR_BASE + 8, "parent not found");
	pub const ERROR_NOT_OWNER_OF_ROOT: error::ErrorType =
		(Self::ERROR_BASE + 9, "not owner of root");
	pub const ERROR_ROOT_NOT_FOUND: error::ErrorType = (Self::ERROR_BASE + 10, "root not found");

	/// Create an error using the error module
	pub fn error(error_type: error::ErrorType) -> DispatchResult {
		<error::Module<T>>::error(error_type)
	}

	/// Calculates the hash of all values of a delegation transaction
	pub fn calculate_hash(
		delegation_id: T::DelegationNodeId,
		root_id: T::DelegationNodeId,
		parent_id: Option<T::DelegationNodeId>,
		permissions: Permissions,
	) -> T::Hash {
		// add all values to an u8 vector
		let mut hashed_values: Vec<u8> = delegation_id.as_ref().to_vec();
		hashed_values.extend_from_slice(root_id.as_ref());
		if let Some(p) = parent_id {
			hashed_values.extend_from_slice(p.as_ref())
		}
		hashed_values.extend_from_slice(permissions.as_u8().as_ref());
		// hash vector
		T::Hashing::hash(&hashed_values)
	}

	/// Check if an account is the owner of the delegation or any delegation up the hierarchy (including the root)
	pub fn is_delegating(
		account: &T::AccountId,
		delegation: &T::DelegationNodeId,
	) -> result::Result<bool, &'static str> {
		// check if delegation exists
		let d = <error::Module<T>>::ok_or_deposit_err(
			<Delegations<T>>::get(delegation),
			Self::ERROR_DELEGATION_NOT_FOUND,
		)?;
		// check if the account is the owner of the delegation
		if d.2.eq(account) {
			Ok(true)
		} else if let Some(p) = d.1 {
			// recurse upwards in hierarchy
			Self::is_delegating(account, &p)
		} else {
			// return whether the account is owner of the root
			let r = <error::Module<T>>::ok_or_deposit_err(
				<Root<T>>::get(d.0),
				Self::ERROR_ROOT_NOT_FOUND,
			)?;
			Ok(r.1.eq(account))
		}
	}

	/// Revoke a delegation an all of its children
	fn revoke(delegation: &T::DelegationNodeId, sender: &T::AccountId) -> DispatchResult {
		// retrieve delegation node from storage
		let mut d = <error::Module<T>>::ok_or_deposit_err(
			<Delegations<T>>::get(*delegation),
			Self::ERROR_DELEGATION_NOT_FOUND,
		)?;
		// check if already revoked
		if !d.4 {
			// set revoked flag and store delegation node
			d.4 = true;
			<Delegations<T>>::insert(*delegation, d);
			// deposit event that the delegation has been revoked
			Self::deposit_event(RawEvent::DelegationRevoked(sender.clone(), *delegation));
			// revoke all children recursively
			Self::revoke_children(delegation, sender)?;
		}
		Ok(())
	}

	/// Revoke all children of a delegation
	fn revoke_children(delegation: &T::DelegationNodeId, sender: &T::AccountId) -> DispatchResult {
		// check if there's a child vector in the storage
		if <Children<T>>::contains_key(delegation) {
			// iterate child vector and revoke all nodes
			let children = <Children<T>>::get(delegation);
			for child in children {
				Self::revoke(&child, sender)?;
			}
		}
		Ok(())
	}

	/// Add a child node into the delegation hierarchy
	fn add_child(child: T::DelegationNodeId, parent: T::DelegationNodeId) {
		// get the children vector
		let mut children = <Children<T>>::get(parent);
		// add child element
		children.push(child);
		// store vector with new child
		<Children<T>>::insert(parent, children);
	}
}

decl_storage! {
	trait Store for Module<T: Trait> as Delegation {
		// Root: root-id => (ctype-hash, account, revoked)?
		pub Root get(fn root):map hasher(opaque_blake2_256) T::DelegationNodeId => Option<(T::Hash,T::AccountId,bool)>;
		// Delegations: delegation-id => (root-id, parent-id?, account, permissions, revoked)?
		pub Delegations get(fn delegation):map hasher(opaque_blake2_256) T::DelegationNodeId => Option<(T::DelegationNodeId,Option<T::DelegationNodeId>,T::AccountId,Permissions,bool)>;
		// Children: root-or-delegation-id => [delegation-id]
		pub Children get(fn children):map hasher(opaque_blake2_256) T::DelegationNodeId => Vec<T::DelegationNodeId>;
	}
}
