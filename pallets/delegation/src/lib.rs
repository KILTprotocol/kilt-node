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

//! Delegation: Handles delegations on chain,
//! creating and revoking root nodes of delegation hierarchies,
//! adding and revoking delegation nodes based on root nodes.
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(any(feature = "runtime-benchmarks", test))]
pub mod benchmarking;
/// Test module for delegations
#[cfg(test)]
mod tests;

pub mod migration;

#[macro_use]
extern crate bitflags;

pub mod default_weights;
pub use default_weights::WeightInfo;

use codec::{Decode, Encode};
use core::default::Default;
use did::DidOperation;
use frame_support::{
	ensure,
	pallet_prelude::{DispatchError, DispatchResultWithPostInfo, Weight},
	traits::Get,
	Parameter,
};
use frame_system::{self, ensure_signed};
use sp_runtime::{
	codec::Codec,
	traits::{CheckEqual, Hash, MaybeDisplay, Member, SimpleBitOps},
};
use sp_std::{
	fmt::Debug,
	prelude::{Clone, Eq, PartialEq, Vec},
};

pub use pallet::*;

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

#[derive(Debug, Encode, Decode, PartialEq)]
pub struct DelegationNode<T: Config> {
	pub root_id: T::DelegationNodeId,
	pub parent: Option<T::DelegationNodeId>,
	pub owner: <T as did::Config>::DidIdentifier,
	pub permissions: Permissions,
	pub revoked: bool,
}

impl<T: Config> DelegationNode<T> {
	pub fn new_root(
		root_id: T::DelegationNodeId,
		owner: <T as did::Config>::DidIdentifier,
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

	/// new_child creates a new child node for the delegation tree.
	///
	/// root_id - the root of the delegation tree
	/// parent - the parent in the tree
	/// owner - the owner of the new child root. He will receive the delegated
	/// permissions permissions - the permissions that are delegated
	pub fn new_child(
		root_id: T::DelegationNodeId,
		parent: T::DelegationNodeId,
		owner: <T as did::Config>::DidIdentifier,
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

#[derive(Debug, Encode, Decode, PartialEq)]
pub struct DelegationRoot<T: Config> {
	pub ctype_hash: T::Hash,
	pub owner: <T as did::Config>::DidIdentifier,
	pub revoked: bool,
}

impl<T: Config> DelegationRoot<T> {
	fn new(ctype_hash: T::Hash, owner: <T as did::Config>::DidIdentifier) -> Self {
		DelegationRoot {
			ctype_hash,
			owner,
			revoked: false,
		}
	}
}

#[derive(Clone, Decode, Encode, PartialEq)]
pub struct DelegationRootCreationOperation<T: Config> {
	creator_did: <T as did::Config>::DidIdentifier,
	root_id: T::DelegationNodeId,
	ctype_hash: <T as frame_system::Config>::Hash,
	tx_counter: u64,
}

impl<T: Config> DidOperation<T> for DelegationRootCreationOperation<T> {
	fn get_verification_key_type(&self) -> did::DidVerificationKeyType {
		did::DidVerificationKeyType::CapabilityDelegation
	}

	fn get_did(&self) -> &T::DidIdentifier {
		&self.creator_did
	}

	fn get_tx_counter(&self) -> u64 {
		self.tx_counter
	}
}

// Required to use a struct as an extrinsic parameter, and since Config does not
// implement Debug, the derive macro does not work.
impl<T: Config> Debug for DelegationRootCreationOperation<T> {
	fn fmt(&self, f: &mut sp_std::fmt::Formatter<'_>) -> sp_std::fmt::Result {
		f.debug_tuple("DelegationRootCreationOperation")
			.field(&self.creator_did)
			.field(&self.root_id)
			.field(&self.ctype_hash)
			.field(&self.tx_counter)
			.finish()
	}
}

#[derive(Clone, Decode, Encode, PartialEq)]
pub struct DelegationCreationOperation<T: Config> {
	creator_did: <T as did::Config>::DidIdentifier,
	delegation_id: T::DelegationNodeId,
	root_id: T::DelegationNodeId,
	parent_id: Option<T::DelegationNodeId>,
	delegate_did: <T as did::Config>::DidIdentifier,
	permissions: Permissions,
	delegate_signature: did::DidSignature,
	tx_counter: u64,
}

impl<T: Config> DidOperation<T> for DelegationCreationOperation<T> {
	fn get_verification_key_type(&self) -> did::DidVerificationKeyType {
		did::DidVerificationKeyType::CapabilityDelegation
	}

	fn get_did(&self) -> &T::DidIdentifier {
		&self.creator_did
	}

	fn get_tx_counter(&self) -> u64 {
		self.tx_counter
	}
}

// Required to use a struct as an extrinsic parameter, and since Config does not
// implement Debug, the derive macro does not work.
impl<T: Config> Debug for DelegationCreationOperation<T> {
	fn fmt(&self, f: &mut sp_std::fmt::Formatter<'_>) -> sp_std::fmt::Result {
		f.debug_tuple("DelegationCreationOperation")
			.field(&self.creator_did)
			.field(&self.delegation_id)
			.field(&self.root_id)
			.field(&self.parent_id)
			.field(&self.delegate_did)
			.field(&self.permissions)
			.field(&self.delegate_signature)
			.field(&self.tx_counter)
			.finish()
	}
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		pallet_prelude::*,
		traits::{Hooks, IsType},
	};
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config + ctype::Config + did::Config {
		/// Delegation specific event type
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

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

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::storage]
	#[pallet::getter(fn root)]
	pub type Root<T> = StorageMap<_, Blake2_128Concat, <T as Config>::DelegationNodeId, DelegationRoot<T>>;

	#[pallet::storage]
	#[pallet::getter(fn delegation)]
	pub type Delegations<T> = StorageMap<_, Blake2_128Concat, <T as Config>::DelegationNodeId, DelegationNode<T>>;

	#[pallet::storage]
	#[pallet::getter(fn children)]
	pub type Children<T> =
		StorageMap<_, Blake2_128Concat, <T as Config>::DelegationNodeId, Vec<<T as Config>::DelegationNodeId>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new root has been created
		RootCreated(
			<T as did::Config>::DidIdentifier,
			T::DelegationNodeId,
			<T as frame_system::Config>::Hash,
		),
		/// A root has been revoked
		RootRevoked(<T as did::Config>::DidIdentifier, T::DelegationNodeId),
		/// A new delegation has been created
		DelegationCreated(
			<T as did::Config>::DidIdentifier,
			T::DelegationNodeId,
			T::DelegationNodeId,
			Option<T::DelegationNodeId>,
			<T as did::Config>::DidIdentifier,
			Permissions,
		),
		/// A delegation has been revoked
		DelegationRevoked(<T as did::Config>::DidIdentifier, T::DelegationNodeId),
	}

	#[pallet::error]
	pub enum Error<T> {
		AlreadyExists,
		BadSignature,
		DelegationNotFound,
		DelegateNotFound,
		RootAlreadyExists,
		RootNotFound,
		MaxSearchDepthReached,
		NotOwnerOfParent,
		NotOwnerOfRoot,
		ParentNotFound,
		UnauthorizedRevocation,
		UnauthorizedDelegation,
		ExceededRevocationBounds,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Creates a delegation hierarchy root on chain, where
		/// origin - the origin of the transaction
		/// delegation_root_creation_operation - the
		/// DelegationRootCreationOperation to execute operation_signature - a
		/// signature over the root delegation creation operation
		#[pallet::weight(<T as Config>::WeightInfo::submit_delegation_root_creation_operation())]
		pub fn submit_delegation_root_creation_operation(
			origin: OriginFor<T>,
			delegation_root_creation_operation: DelegationRootCreationOperation<T>,
			operation_signature: did::DidSignature,
		) -> DispatchResultWithPostInfo {
			// origin of the transaction needs to be a signed sender account
			ensure_signed(origin)?;

			let mut did_details = <did::Did<T>>::get(&delegation_root_creation_operation.creator_did)
				.ok_or(<did::Error<T>>::DidNotPresent)?;

			did::pallet::Pallet::verify_operation_validity_for_did(
				&delegation_root_creation_operation,
				&operation_signature,
				&did_details,
			)
			.map_err(<did::Error<T>>::from)?;

			// check if a root with the given id already exists
			ensure!(
				!<Root<T>>::contains_key(&delegation_root_creation_operation.root_id),
				Error::<T>::RootAlreadyExists
			);

			// add root node to storage
			log::debug!("insert Delegation Root");
			<Root<T>>::insert(
				&delegation_root_creation_operation.root_id,
				DelegationRoot::new(
					delegation_root_creation_operation.ctype_hash,
					delegation_root_creation_operation.creator_did.clone(),
				),
			);

			// Update tx counter in DID details and save to DID pallet
			did_details
				.increase_tx_counter()
				.expect("Increasing DID tx counter should be a safe operation.");
			<did::Did<T>>::insert(&delegation_root_creation_operation.creator_did, did_details);

			// deposit event that the root node has been created
			Self::deposit_event(Event::RootCreated(
				delegation_root_creation_operation.creator_did,
				delegation_root_creation_operation.root_id,
				delegation_root_creation_operation.ctype_hash,
			));

			Ok(().into())
		}

		/// Adds a delegation node on chain, where
		/// origin - the origin of the transaction
		/// delegation_creation_operation - the DelegationCreationOperation to
		/// execute operation_signature - a signature over the delegation
		/// creation operation
		#[pallet::weight(<T as Config>::WeightInfo::submit_delegation_creation_operation())]
		pub fn submit_delegation_creation_operation(
			origin: OriginFor<T>,
			delegation_creation_operation: DelegationCreationOperation<T>,
			operation_signature: did::DidSignature,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;

			let mut did_details =
				<did::Did<T>>::get(&delegation_creation_operation.creator_did).ok_or(<did::Error<T>>::DidNotPresent)?;

			did::pallet::Pallet::verify_operation_validity_for_did(
				&delegation_creation_operation,
				&operation_signature,
				&did_details,
			)
			.map_err(<did::Error<T>>::from)?;

			let delegate_did_details = <did::Did<T>>::get(&delegation_creation_operation.delegate_did)
				.ok_or(<did::Error<T>>::DidNotPresent)?;

			// Calculate the hash root
			let hash_root = Self::calculate_hash(
				&delegation_creation_operation.delegation_id,
				&delegation_creation_operation.root_id,
				&delegation_creation_operation.parent_id,
				&delegation_creation_operation.permissions,
			);

			// Verify that the hash root has been signed with the delegate's authentication
			// key
			did::pallet::Pallet::<T>::verify_payload_signature_with_did_key_type(
				hash_root.as_ref(),
				&delegation_creation_operation.delegate_signature,
				&delegate_did_details,
				did::DidVerificationKeyType::Authentication,
			)
			.map_err(|err| match err {
				did::DidError::StorageError(_) | did::DidError::UrlError(_) => Error::<T>::DelegateNotFound,
				did::DidError::SignatureError(_) => Error::<T>::BadSignature,
			})?;

			// check if a delegation node with the given identifier already exists
			ensure!(
				!<Delegations<T>>::contains_key(&delegation_creation_operation.delegation_id),
				Error::<T>::AlreadyExists
			);

			// check if root exists
			let root = <Root<T>>::get(&delegation_creation_operation.root_id).ok_or(Error::<T>::RootNotFound)?;

			// check if this delegation has a parent
			if let Some(parent_id) = delegation_creation_operation.parent_id {
				// check if the parent exists
				let parent_node = <Delegations<T>>::get(&parent_id).ok_or(Error::<T>::ParentNotFound)?;

				// check if the parent's delegate is the creator of this delegation node and has
				// permission to delegate
				ensure!(
					parent_node.owner.eq(&delegation_creation_operation.creator_did),
					Error::<T>::NotOwnerOfParent
				);

				// check if the parent has permission to delegate
				ensure!(
					(parent_node.permissions & Permissions::DELEGATE) == Permissions::DELEGATE,
					Error::<T>::UnauthorizedDelegation
				);

				// insert delegation
				log::debug!("insert Delegation with parent");
				<Delegations<T>>::insert(
					&delegation_creation_operation.delegation_id,
					DelegationNode::<T>::new_child(
						delegation_creation_operation.root_id,
						parent_id,
						delegation_creation_operation.creator_did.clone(),
						delegation_creation_operation.permissions,
					),
				);
				// add child to tree structure
				Self::add_child(delegation_creation_operation.delegation_id, parent_id);
			} else {
				// check if the creator of this delegation node is the creator of the root node
				// (as no parent is given)
				ensure!(
					root.owner.eq(&delegation_creation_operation.creator_did),
					Error::<T>::NotOwnerOfRoot
				);

				// insert delegation
				log::debug!("insert Delegation without parent");
				<Delegations<T>>::insert(
					&delegation_creation_operation.delegation_id,
					DelegationNode::<T>::new_root(
						delegation_creation_operation.root_id,
						delegation_creation_operation.delegate_did.clone(),
						delegation_creation_operation.permissions,
					),
				);

				// add child to tree structure
				Self::add_child(
					delegation_creation_operation.delegation_id,
					delegation_creation_operation.root_id,
				);
			}

			did_details
				.increase_tx_counter()
				.expect("Increasing DID tx counter should be a safe operation.");
			<did::Did<T>>::insert(&delegation_creation_operation.creator_did, did_details);

			// deposit event that the delegation node has been added
			Self::deposit_event(Event::DelegationCreated(
				delegation_creation_operation.creator_did,
				delegation_creation_operation.delegation_id,
				delegation_creation_operation.root_id,
				delegation_creation_operation.parent_id,
				delegation_creation_operation.delegate_did,
				delegation_creation_operation.permissions,
			));

			Ok(().into())
		}
	}
}

// // check if this delegation has a parent
// if let Some(parent_id) = parent_id {
// 	// check if the parent exists
// 	let parent_node =
// <Delegations<T>>::get(parent_id).ok_or(Error::<T>::ParentNotFound)?;

// 	// check if the parent's delegate is the sender of this transaction and has
// permission to delegate 	ensure!(parent_node.owner.eq(&sender),
// Error::<T>::NotOwnerOfParent);

// 	// check if the parent has permission to delegate
// 	ensure!((parent_node.permissions & Permissions::DELEGATE) ==
// Permissions::DELEGATE, Error::<T>::UnauthorizedDelegation);

// 	// insert delegation
// 	log::debug!("insert Delegation with parent");
// 	<Delegations<T>>::insert(delegation_id, DelegationNode::<T>::new_child(
// 		root_id,
// 		parent_id,
// 		delegate.clone(),
// 		permissions,
// 	));
// 	// add child to tree structure
// 	Self::add_child(delegation_id, parent_id);
// } else {
// 	// check if the sender of this transaction is the creator of the root node
// (as no parent is given) 	ensure!(root.owner.eq(&sender),
// Error::<T>::NotOwnerOfRoot);

// 	// inser delegation
// 	log::debug!("insert Delegation without parent");
// 	<Delegations<T>>::insert(delegation_id,
// DelegationNode::<T>::new_root(root_id, delegate.clone(), permissions));

// 	// add child to tree structure
// 	Self::add_child(delegation_id, root_id);
// }
// // deposit event that the delegation node has been added
// Self::deposit_event(RawEvent::DelegationCreated(sender, delegation_id,
// 		root_id, parent_id, delegate, permissions));
// Ok(())
// 		}

// 		/// Revoke the root and therefore a complete hierarchy, where
// 		/// origin - the origin of the transaction
// 		/// root_id - id of the hierarchy root node
// 		/// max_children - max number of children of the root which can be revoked
// with this call 		#[weight = <T as
// Config>::WeightInfo::revoke_root(*max_children)] 		pub fn revoke_root(origin,
// root_id: T::DelegationNodeId, max_children: u32) ->
// DispatchResultWithPostInfo { 			// origin of the transaction needs to be a
// signed sender account 			let sender = ensure_signed(origin)?;

// 			// check if root node exists
// 			let mut root = <Root<T>>::get(root_id).ok_or(Error::<T>::RootNotFound)?;

// 			// check if root node has been created by the sender of this transaction
// 			ensure!(root.owner.eq(&sender), Error::<T>::UnauthorizedRevocation);

// 			let consumed_weight: Weight = if !root.revoked {
// 				// recursively revoke all children
// 				let (remaining_revocations, post_weight) = Self::revoke_children(&root_id,
// &sender, max_children)?;

// 				if remaining_revocations > 0 {
// 					// store revoked root node
// 					root.revoked = true;
// 					<Root<T>>::insert(root_id, root);
// 				}
// 				post_weight + T::DbWeight::get().writes(1)
// 			} else {
// 				0
// 			};
// 			// deposit event that the root node has been revoked
// 			Self::deposit_event(RawEvent::RootRevoked(sender, root_id));
// 			// post call weight correction
// 			Ok(Some(consumed_weight + T::DbWeight::get().reads(1)).into())
// 		}

// 		/// Revoke a delegation node and all its children, where
// 		/// origin - the origin of the transaction
// 		/// delegation_id - id of the delegation node
// 		/// max_depth - max number of parent checks of the delegation node supported
// in this call until finding the owner 		/// max_revocations - max number of
// children of the delegation node which can be revoked with this call 		#[weight
// = <T as Config>::WeightInfo::revoke_delegation_leaf(*max_depth + 1).max(<T as
// Config>::WeightInfo::revoke_delegation_root_child(*max_depth + 1))]
// 		pub fn revoke_delegation(origin, delegation_id: T::DelegationNodeId,
// max_depth: u32, max_revocations: u32) -> DispatchResultWithPostInfo { 			// origin
// of the transaction needs to be a signed sender account 			let sender =
// ensure_signed(origin)?; 			// check if a delegation node with the given
// identifier already exists 			ensure!(<Delegations<T>>::
// contains_key(delegation_id), Error::<T>::DelegationNotFound);

// 			// check if the sender of this transaction is permitted by being the
// 			// owner of the delegation or of one of its parents
// 			// 1 lookup performed for current node + 1 for every parent that is traversed
// 			ensure!(Self::is_delegating(&sender, &delegation_id, max_depth + 1)?,
// Error::<T>::UnauthorizedRevocation);

// 			// revoke the delegation and recursively all of its children
// 			// post call weight correction
// 			let (_, consumed_weight) = Self::revoke(&delegation_id, &sender,
// max_revocations)?;

// 			// add worst case reads from `is_delegating`
// 			Ok(Some(consumed_weight + T::DbWeight::get().reads((2 +
// max_depth).into())).into()) 		}
// 	}
// }

impl<T: Config> Pallet<T> {
	/// Calculates the hash of all values of a delegation transaction
	pub fn calculate_hash(
		delegation_id: &T::DelegationNodeId,
		root_id: &T::DelegationNodeId,
		parent_id: &Option<T::DelegationNodeId>,
		permissions: &Permissions,
	) -> <T as frame_system::Config>::Hash {
		// add all values to an u8 vector
		let mut hashed_values: Vec<u8> = delegation_id.as_ref().to_vec();
		hashed_values.extend_from_slice(root_id.as_ref());
		if let Some(parent) = parent_id {
			hashed_values.extend_from_slice(parent.as_ref())
		}
		hashed_values.extend_from_slice(permissions.as_u8().as_ref());
		// hash vector
		T::Hashing::hash(&hashed_values)
	}

	/// Check if an account is the owner of the delegation or any delegation up
	/// the hierarchy (including the root)
	pub fn is_delegating(
		account: &<T as did::Config>::DidIdentifier,
		delegation: &T::DelegationNodeId,
		max_lookups: u32,
	) -> Result<bool, DispatchError> {
		// check for recursion anchor
		ensure!(max_lookups > 0, Error::<T>::MaxSearchDepthReached);

		// check if delegation exists
		let delegation_node = <Delegations<T>>::get(delegation).ok_or(Error::<T>::DelegationNotFound)?;

		// check if the given account is the owner of the delegation
		if delegation_node.owner.eq(account) {
			Ok(true)
		} else if let Some(parent) = delegation_node.parent {
			// recursively check upwards in hierarchy
			Self::is_delegating(account, &parent, max_lookups - 1)
		} else {
			// return whether the given account is the owner of the root
			let root = <Root<T>>::get(delegation_node.root_id).ok_or(Error::<T>::RootNotFound)?;
			Ok(root.owner.eq(account))
		}
	}

	/// Revoke a delegation and all of its children recursively
	fn revoke(
		delegation: &T::DelegationNodeId,
		sender: &<T as did::Config>::DidIdentifier,
		max_revocations: u32,
	) -> Result<(u32, Weight), DispatchError> {
		let mut revocations: u32 = 0;
		let mut consumed_weight: Weight = 0;
		// retrieve delegation node from storage
		let mut delegation_node = <Delegations<T>>::get(*delegation).ok_or(Error::<T>::DelegationNotFound)?;
		consumed_weight += T::DbWeight::get().reads(1);

		// check if already revoked
		if !delegation_node.revoked {
			// first revoke all children recursively
			Self::revoke_children(delegation, sender, max_revocations - 1).map(|(r, w)| {
				revocations += r;
				consumed_weight += w;
			})?;

			// if we run out of revocation gas, we only revoke children. The tree will be
			// changed but is still valid.
			if revocations < max_revocations {
				// set revoked flag and store delegation node
				delegation_node.revoked = true;
				<Delegations<T>>::insert(*delegation, delegation_node);
				consumed_weight += T::DbWeight::get().writes(1);
				// deposit event that the delegation has been revoked
				Self::deposit_event(Event::DelegationRevoked(sender.clone(), *delegation));
				revocations += 1;
			} else {
				return Err(Error::<T>::ExceededRevocationBounds.into());
			}
		}
		Ok((revocations, consumed_weight))
	}

	/// Revoke all children of a delegation
	fn revoke_children(
		delegation: &T::DelegationNodeId,
		sender: &<T as did::Config>::DidIdentifier,
		max_revocations: u32,
	) -> Result<(u32, Weight), DispatchError> {
		let mut revocations: u32 = 0;
		let mut consumed_weight: Weight = 0;
		// check if there's a child vector in the storage
		if <Children<T>>::contains_key(delegation) {
			// iterate child vector and revoke all nodes
			let children = <Children<T>>::get(delegation).unwrap();
			consumed_weight += T::DbWeight::get().reads(1);

			for child in children {
				let remaining_revocations = max_revocations.saturating_sub(revocations);
				if remaining_revocations > 0 {
					Self::revoke(&child, sender, remaining_revocations).map(|(r, w)| {
						revocations += r;
						consumed_weight += w;
					})?;
				} else {
					return Err(Error::<T>::ExceededRevocationBounds.into());
				}
			}
		}
		Ok((revocations, consumed_weight + T::DbWeight::get().reads(1)))
	}

	/// Add a child node into the delegation hierarchy
	fn add_child(child: T::DelegationNodeId, parent: T::DelegationNodeId) {
		// get the children vector
		let mut children = <Children<T>>::get(parent).unwrap();
		// add child element
		children.push(child);
		// store vector with new child
		<Children<T>>::insert(parent, children);
	}
}
