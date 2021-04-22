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

#[cfg(test)]
mod tests;

#[cfg(any(feature = "mock", test))]
pub mod mock;

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
		/// Allows an entity to write attestations on chain.
		const ATTEST = 0b0000_0001;
		/// Allows an entity to write delegations on chain.
		const DELEGATE = 0b0000_0010;
	}
}

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

impl Default for Permissions {
	/// Default permissions to the attest permission
	fn default() -> Self {
		Permissions::ATTEST
	}
}

/// A node containing information about a delegation root.
///
/// It contains the following information:
/// * ctype_hash: the credential CTYPE that can be issued with this delegation
/// * owner: the owner of the delegation root, which has the capability to
///   remove all of its children delegations
/// * revoked: boolean flag indicating whether the delegation is revoked or not
#[derive(Clone, Debug, Encode, Decode, PartialEq)]
pub struct DelegationRoot<T: Config> {
	pub ctype_hash: T::Hash,
	pub owner: T::DidIdentifier,
	pub revoked: bool,
}

impl<T: Config> DelegationRoot<T> {
	fn new(ctype_hash: T::Hash, owner: T::DidIdentifier) -> Self {
		DelegationRoot {
			ctype_hash,
			owner,
			revoked: false,
		}
	}
}

/// A node containing information about a delegation node.
///
/// It contains the following information:
/// * root_id: the root node ID of which this node is a descendent of
/// * parent: an optional parent of this node. If None, this node is a direct
///   descendent of the root node
/// * owner: the owner of the delegation node, which has the capability to
///   remove all of its children delegations
/// * permissions: indicates what kinds of actions the delegate can perform
/// * revoked: boolean flag indicating whether the delegation is revoked or not
#[derive(Clone, Debug, Encode, Decode, PartialEq)]
pub struct DelegationNode<T: Config> {
	pub root_id: T::DelegationNodeId,
	pub parent: Option<T::DelegationNodeId>,
	pub owner: T::DidIdentifier,
	pub permissions: Permissions,
	pub revoked: bool,
}

impl<T: Config> DelegationNode<T> {
	/// Create a new delegation node that is a direct descendent of the given
	/// root.
	///
	/// * root_id: the unique ID of the root node this node will be a child of
	/// * owner: the owner of the delegation
	/// * permissions: the delegation permissions
	pub fn new_root_child(root_id: T::DelegationNodeId, owner: T::DidIdentifier, permissions: Permissions) -> Self {
		DelegationNode {
			root_id,
			owner,
			permissions,
			revoked: false,
			parent: None,
		}
	}

	/// Creates a new delegation node that is a direct descendent of the given
	/// node.
	///
	/// * root_id: the unique ID of the root node this node will be a child of
	/// * parent - the unique ID of the parent node this node will be a child of
	/// * owner: the owner of the delegation
	/// * permissions: the delegation permissions
	pub fn new_node_child(
		root_id: T::DelegationNodeId,
		parent: T::DelegationNodeId,
		owner: T::DidIdentifier,
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

/// An operation that contains instructions to create a new delegation root.
///
/// It contains the following information:
/// * caller_did: the DID of the new root owner
/// * root_id: the unique ID of the root node this node will be a child of
/// * ctype_hash: hash of the CTYPE the hierarchy is created for
/// * tx_counter: a DID counter used to mitigate replay attacks
#[derive(Clone, Decode, Encode, PartialEq)]
pub struct DelegationRootCreationOperation<T: Config> {
	caller_did: T::DidIdentifier,
	root_id: T::DelegationNodeId,
	ctype_hash: T::Hash,
	tx_counter: u64,
}

impl<T: Config> DidOperation<T> for DelegationRootCreationOperation<T> {
	fn get_verification_key_type(&self) -> did::DidVerificationKeyType {
		did::DidVerificationKeyType::CapabilityDelegation
	}

	fn get_did(&self) -> &T::DidIdentifier {
		&self.caller_did
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
			.field(&self.caller_did)
			.field(&self.root_id)
			.field(&self.ctype_hash)
			.field(&self.tx_counter)
			.finish()
	}
}

/// An operation that contains instructions to create a new delegation node.
///
/// It contains the following information:
/// * caller_did: the DID of the new root owner
/// * delegation_id: unique ID of the delegation node to be added
/// * root_id: ID of the hierarchy root node
/// * parent_id: optional identifier of a parent node this delegation node is
///   created under
/// * delegate_did: the DID of the delegate entity
/// * permissions: the permissions delegated
/// * delegate_signature: the signature of the delegate to ensure it's done
///   under his permission
/// * tx_counter: a DID counter used to mitigate replay attacks
#[derive(Clone, Decode, Encode, PartialEq)]
pub struct DelegationCreationOperation<T: Config> {
	caller_did: T::DidIdentifier,
	delegation_id: T::DelegationNodeId,
	root_id: T::DelegationNodeId,
	parent_id: Option<T::DelegationNodeId>,
	delegate_did: T::DidIdentifier,
	permissions: Permissions,
	delegate_signature: did::DidSignature,
	tx_counter: u64,
}

impl<T: Config> DidOperation<T> for DelegationCreationOperation<T> {
	fn get_verification_key_type(&self) -> did::DidVerificationKeyType {
		did::DidVerificationKeyType::CapabilityDelegation
	}

	fn get_did(&self) -> &T::DidIdentifier {
		&self.caller_did
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
			.field(&self.caller_did)
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

/// An operation that contains instructions to revoke a new delegation root.
///
/// It contains the following information:
/// * caller_did: the DID of the root owner
/// * root_id: ID of the hierarchy root node
/// * max_children: max number of children of the root which can be revoked with
///   this call
/// * tx_counter: a DID counter used to mitigate replay attacks
#[derive(Clone, Decode, Encode, PartialEq)]
pub struct DelegationRootRevocationOperation<T: Config> {
	caller_did: T::DidIdentifier,
	root_id: T::DelegationNodeId,
	max_children: u32,
	tx_counter: u64,
}

impl<T: Config> DidOperation<T> for DelegationRootRevocationOperation<T> {
	fn get_verification_key_type(&self) -> did::DidVerificationKeyType {
		did::DidVerificationKeyType::CapabilityDelegation
	}

	fn get_did(&self) -> &T::DidIdentifier {
		&self.caller_did
	}

	fn get_tx_counter(&self) -> u64 {
		self.tx_counter
	}
}

// Required to use a struct as an extrinsic parameter, and since Config does not
// implement Debug, the derive macro does not work.
impl<T: Config> Debug for DelegationRootRevocationOperation<T> {
	fn fmt(&self, f: &mut sp_std::fmt::Formatter<'_>) -> sp_std::fmt::Result {
		f.debug_tuple("DelegationRootRevocationOperation")
			.field(&self.caller_did)
			.field(&self.root_id)
			.field(&self.max_children)
			.field(&self.tx_counter)
			.finish()
	}
}

/// An operation that contains instructions to revoke a new delegation node.
///
/// It contains the following information:
/// * caller_did: the DID of the root owner
/// * delegation_id: ID of the delegation node
/// * max_parent_checks: max number of parent checks of the delegation node
///   supported in this call to verify that the caller of this operation is
///   allowed to revoke the specified node
/// * max_revocations: max number of children of the delegation node which can
///   be revoked with this call
/// * tx_counter: a DID counter used to mitigate replay attacks
#[derive(Clone, Decode, Encode, PartialEq)]
pub struct DelegationRevocationOperation<T: Config> {
	caller_did: T::DidIdentifier,
	delegation_id: T::DelegationNodeId,
	max_parent_checks: u32,
	max_revocations: u32,
	tx_counter: u64,
}

impl<T: Config> DidOperation<T> for DelegationRevocationOperation<T> {
	fn get_verification_key_type(&self) -> did::DidVerificationKeyType {
		did::DidVerificationKeyType::CapabilityDelegation
	}

	fn get_did(&self) -> &T::DidIdentifier {
		&self.caller_did
	}

	fn get_tx_counter(&self) -> u64 {
		self.tx_counter
	}
}

// Required to use a struct as an extrinsic parameter, and since Config does not
// implement Debug, the derive macro does not work.
impl<T: Config> Debug for DelegationRevocationOperation<T> {
	fn fmt(&self, f: &mut sp_std::fmt::Formatter<'_>) -> sp_std::fmt::Result {
		f.debug_tuple("DelegationRevocationOperation")
			.field(&self.caller_did)
			.field(&self.delegation_id)
			.field(&self.max_parent_checks)
			.field(&self.max_revocations)
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
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type WeightInfo: WeightInfo;

		/// Delegation node ID type
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

	// root node ID -> root node
	#[pallet::storage]
	#[pallet::getter(fn roots)]
	pub type Roots<T> = StorageMap<_, Blake2_128Concat, <T as Config>::DelegationNodeId, DelegationRoot<T>>;

	// node ID -> node
	#[pallet::storage]
	#[pallet::getter(fn delegations)]
	pub type Delegations<T> = StorageMap<_, Blake2_128Concat, <T as Config>::DelegationNodeId, DelegationNode<T>>;

	// node ID -> vec<node id>
	#[pallet::storage]
	#[pallet::getter(fn children)]
	pub type Children<T> =
		StorageMap<_, Blake2_128Concat, <T as Config>::DelegationNodeId, Vec<<T as Config>::DelegationNodeId>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new root has been created
		RootCreated(T::DidIdentifier, T::DelegationNodeId, T::Hash),
		/// A root has been revoked
		RootRevoked(T::DidIdentifier, T::DelegationNodeId),
		/// A new delegation has been created
		DelegationCreated(
			T::DidIdentifier,
			T::DelegationNodeId,
			T::DelegationNodeId,
			Option<T::DelegationNodeId>,
			T::DidIdentifier,
			Permissions,
		),
		/// A delegation has been revoked
		DelegationRevoked(T::DidIdentifier, T::DelegationNodeId),
	}

	#[pallet::error]
	pub enum Error<T> {
		DelegationAlreadyExists,
		InvalidDelegateSignature,
		DelegationNotFound,
		DelegateNotFound,
		RootAlreadyExists,
		RootNotFound,
		MaxSearchDepthReached,
		NotOwnerOfParentDelegation,
		NotOwnerOfRootDelegation,
		ParentDelegationNotFound,
		UnauthorizedRevocation,
		UnauthorizedDelegation,
		ExceededRevocationBounds,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Submits a new DelegationRootCreationOperation operation.
		///
		/// origin: the origin of the transaction
		/// operation: the DelegationRootCreationOperation operation
		/// signature: the signature over the byte-encoded operation
		#[pallet::weight(<T as Config>::WeightInfo::submit_delegation_root_creation_operation())]
		pub fn submit_delegation_root_creation_operation(
			origin: OriginFor<T>,
			operation: DelegationRootCreationOperation<T>,
			signature: did::DidSignature,
		) -> DispatchResultWithPostInfo {
			// Origin of the transaction needs to be a signed sender account
			ensure_signed(origin)?;

			// Check if DID exists, if counter is valid, if signature is valid, and increase
			// DID tx counter
			did::pallet::Pallet::verify_operation_validity_and_increase_did_nonce(&operation, &signature)
				.map_err(<did::Error<T>>::from)?;

			// Check if a root with the given id already exists
			ensure!(
				!<Roots<T>>::contains_key(&operation.root_id),
				Error::<T>::RootAlreadyExists
			);

			// Check if CTYPE exists
			ensure!(
				<ctype::Ctypes<T>>::contains_key(&operation.ctype_hash),
				<ctype::Error<T>>::CTypeNotFound
			);

			// Add root node to storage
			log::debug!("insert Delegation Root");
			<Roots<T>>::insert(
				&operation.root_id,
				DelegationRoot::new(operation.ctype_hash, operation.caller_did.clone()),
			);

			// Deposit event that the root node has been created
			Self::deposit_event(Event::RootCreated(
				operation.caller_did,
				operation.root_id,
				operation.ctype_hash,
			));

			Ok(().into())
		}

		/// Submits a new DelegationCreationOperation operation.
		///
		/// origin: the origin of the transaction
		/// operation: the DelegationCreationOperation operation
		/// signature: the signature over the byte-encoded operation
		#[pallet::weight(<T as Config>::WeightInfo::submit_delegation_creation_operation())]
		pub fn submit_delegation_creation_operation(
			origin: OriginFor<T>,
			operation: DelegationCreationOperation<T>,
			signature: did::DidSignature,
		) -> DispatchResultWithPostInfo {
			// Origin of the transaction needs to be a signed sender account
			ensure_signed(origin)?;

			// Check if DID exists, if counter is valid, if signature is valid, and increase
			// DID tx counter
			did::pallet::Pallet::verify_operation_validity_and_increase_did_nonce(&operation, &signature)
				.map_err(<did::Error<T>>::from)?;

			// Retrieve delegate DID details for signature verification
			let delegate_did_details =
				<did::Did<T>>::get(&operation.delegate_did).ok_or(Error::<T>::DelegateNotFound)?;

			// Calculate the hash root
			let hash_root = Self::calculate_hash(
				&operation.delegation_id,
				&operation.root_id,
				&operation.parent_id,
				&operation.permissions,
			);

			// Verify that the hash root has been signed with the delegate's authentication
			// key
			did::pallet::Pallet::<T>::verify_payload_signature_with_did_key_type(
				hash_root.as_ref(),
				&operation.delegate_signature,
				&delegate_did_details,
				did::DidVerificationKeyType::Authentication,
			)
			.map_err(|err| match err {
				// Should never happen as a DID has always a valid authentication key and UrlErrors are never thrown
				// here.
				did::DidError::StorageError(_) | did::DidError::UrlError(_) => Error::<T>::DelegateNotFound,
				did::DidError::SignatureError(_) => Error::<T>::InvalidDelegateSignature,
			})?;

			// Check if a delegation node with the given identifier already exists
			ensure!(
				!<Delegations<T>>::contains_key(&operation.delegation_id),
				Error::<T>::DelegationAlreadyExists
			);

			// Check if root exists
			let root = <Roots<T>>::get(&operation.root_id).ok_or(Error::<T>::RootNotFound)?;

			// Computes the delegation parent. Either the given parent or the root node.
			let parent_id = if let Some(parent_id) = operation.parent_id {
				// Check if the parent exists
				let parent_node = <Delegations<T>>::get(&parent_id).ok_or(Error::<T>::ParentDelegationNotFound)?;

				// Check if the parent's delegate is the creator of this delegation node and has
				// permission to delegate
				ensure!(
					parent_node.owner.eq(&operation.caller_did),
					Error::<T>::NotOwnerOfParentDelegation
				);

				// Check if the parent has permission to delegate
				ensure!(
					(parent_node.permissions & Permissions::DELEGATE) == Permissions::DELEGATE,
					Error::<T>::UnauthorizedDelegation
				);

				// Insert delegation
				log::debug!("insert Delegation with parent");
				<Delegations<T>>::insert(
					&operation.delegation_id,
					DelegationNode::<T>::new_node_child(
						operation.root_id,
						parent_id,
						operation.delegate_did.clone(),
						operation.permissions,
					),
				);
				// Return parent_id as the result of this if branch
				parent_id
			} else {
				// Check if the creator of this delegation node is the creator of the root node
				// (as no parent is given)
				ensure!(
					root.owner.eq(&operation.caller_did),
					Error::<T>::NotOwnerOfRootDelegation
				);

				// Insert delegation
				log::debug!("insert Delegation without parent");
				<Delegations<T>>::insert(
					&operation.delegation_id,
					DelegationNode::<T>::new_root_child(
						operation.root_id,
						operation.delegate_did.clone(),
						operation.permissions,
					),
				);

				// Return node_id as the result of this if branch
				operation.root_id
			};

			Self::add_child(operation.delegation_id, parent_id);

			// Deposit event that the delegation node has been added
			Self::deposit_event(Event::DelegationCreated(
				operation.caller_did,
				operation.delegation_id,
				operation.root_id,
				operation.parent_id,
				operation.delegate_did,
				operation.permissions,
			));

			Ok(().into())
		}

		/// Submits a new DelegationRootRevocationOperation operation.
		///
		/// origin: the origin of the transaction
		/// operation: the DelegationRootRevocationOperation operation
		/// signature: the signature over the byte-encoded operation
		#[pallet::weight(<T as Config>::WeightInfo::submit_delegation_root_revocation_operation(operation.max_children))]
		pub fn submit_delegation_root_revocation_operation(
			origin: OriginFor<T>,
			operation: DelegationRootRevocationOperation<T>,
			signature: did::DidSignature,
		) -> DispatchResultWithPostInfo {
			// Origin of the transaction needs to be a signed sender account
			ensure_signed(origin)?;

			// Check if DID exists, if counter is valid, if signature is valid, and increase
			// DID tx counter
			did::pallet::Pallet::verify_operation_validity_and_increase_did_nonce(&operation, &signature)
				.map_err(<did::Error<T>>::from)?;

			// Check if root node exists
			let mut root = <Roots<T>>::get(&operation.root_id).ok_or(Error::<T>::RootNotFound)?;

			// Check if root node has been created by the sender of this transaction
			ensure!(root.owner.eq(&operation.caller_did), Error::<T>::UnauthorizedRevocation);

			let consumed_weight: Weight = if !root.revoked {
				// Recursively revoke all children
				let (remaining_revocations, post_weight) =
					Self::revoke_children(&operation.root_id, &operation.caller_did, operation.max_children)?;

				if remaining_revocations > 0 {
					// Store revoked root node
					root.revoked = true;
					<Roots<T>>::insert(&operation.root_id, root);
				}
				post_weight + T::DbWeight::get().writes(1)
			} else {
				0
			};

			// Deposit event that the root node has been revoked
			Self::deposit_event(Event::RootRevoked(operation.caller_did, operation.root_id));
			// Post call weight correction
			Ok(Some(consumed_weight + T::DbWeight::get().reads(1)).into())
		}

		/// Submits a new DelegationRevocationOperation operation.
		///
		/// origin: the origin of the transaction
		/// operation: the DelegationRevocationOperation operation
		/// signature: the signature over the byte-encoded operation
		#[pallet::weight(<T as Config>::WeightInfo::revoke_delegation_leaf(operation.max_parent_checks + 1).max(<T as Config>::WeightInfo::submit_delegation_revocation_operation(operation.max_parent_checks + 1)))]
		pub fn submit_delegation_revocation_operation(
			origin: OriginFor<T>,
			operation: DelegationRevocationOperation<T>,
			signature: did::DidSignature,
		) -> DispatchResultWithPostInfo {
			// Origin of the transaction needs to be a signed sender account
			ensure_signed(origin)?;

			// Check if DID exists, if counter is valid, if signature is valid, and increase
			// DID tx counter
			did::pallet::Pallet::verify_operation_validity_and_increase_did_nonce(&operation, &signature)
				.map_err(<did::Error<T>>::from)?;

			// Check if a delegation node with the given identifier already exists
			ensure!(
				<Delegations<T>>::contains_key(&operation.delegation_id),
				Error::<T>::DelegationNotFound
			);

			// Check if the sender of this transaction is permitted by being the
			// owner of the delegation or of one of its parents
			// 1 lookup performed for current node + 1 for every parent that is traversed
			// If the value is already the max given, do not perform +1.
			let max_parent_checks = operation
				.max_parent_checks
				.saturating_add(1);
			ensure!(
				Self::is_delegating(&operation.caller_did, &operation.delegation_id, max_parent_checks)?,
				Error::<T>::UnauthorizedRevocation
			);

			// Revoke the delegation and recursively all of its children
			let (_, consumed_weight) = Self::revoke(
				&operation.delegation_id,
				&operation.caller_did,
				operation.max_revocations,
			)?;

			// Add worst case reads from `is_delegating`
			//TODO: Return proper weight consumption.
			Ok(Some(consumed_weight + T::DbWeight::get().reads((operation.max_parent_checks.saturating_add(2)).into())).into())
		}
	}
}

impl<T: Config> Pallet<T> {
	// Calculate the hash of all values of a delegation transaction
	fn calculate_hash(
		delegation_id: &T::DelegationNodeId,
		root_id: &T::DelegationNodeId,
		parent_id: &Option<T::DelegationNodeId>,
		permissions: &Permissions,
	) -> T::Hash {
		// Add all values to an u8 vector
		let mut hashed_values: Vec<u8> = delegation_id.as_ref().to_vec();
		hashed_values.extend_from_slice(root_id.as_ref());
		if let Some(parent) = parent_id {
			hashed_values.extend_from_slice(parent.as_ref())
		}
		hashed_values.extend_from_slice(permissions.as_u8().as_ref());
		// Hash the resulting vector
		T::Hashing::hash(&hashed_values)
	}

	// Check if an account is the owner of the delegation or any delegation up
	// the hierarchy (including the root), up to `max_lookups` nodes.
	pub fn is_delegating(
		account: &T::DidIdentifier,
		delegation: &T::DelegationNodeId,
		max_lookups: u32,
	) -> Result<bool, DispatchError> {
		// Check for recursion anchor
		ensure!(max_lookups > 0, Error::<T>::MaxSearchDepthReached);

		// Check if delegation exists
		let delegation_node = <Delegations<T>>::get(delegation).ok_or(Error::<T>::DelegationNotFound)?;

		// Check if the given account is the owner of the delegation and that the
		// delegation has not been removed Else: since at the moment there might be
		// another node up the hierarchy, we keep searching for a valid one.
		// It should be changed in the future to stop when a matching node is found,
		// after we ensure there is only one delegation
		if delegation_node.owner.eq(account) && !delegation_node.revoked {
			Ok(true)
		} else if let Some(parent) = delegation_node.parent {
			// This case should never happen as we check in the beginning that max_lookups >
			// 0
			let remaining_lookups = max_lookups.checked_sub(1).ok_or(Error::<T>::MaxSearchDepthReached)?;
			// Recursively check upwards in hierarchy
			Self::is_delegating(account, &parent, remaining_lookups)
		} else {
			// Return whether the given account is the owner of the root and the root has
			// not been revoked
			let root = <Roots<T>>::get(delegation_node.root_id).ok_or(Error::<T>::RootNotFound)?;
			Ok(root.owner.eq(account) && !root.revoked)
		}
	}

	/// Revoke a delegation and all of its children recursively
	fn revoke(
		delegation: &T::DelegationNodeId,
		sender: &T::DidIdentifier,
		max_revocations: u32,
	) -> Result<(u32, Weight), DispatchError> {
		let mut revocations: u32 = 0;
		let mut consumed_weight: Weight = 0;
		// Retrieve delegation node from storage
		let mut delegation_node = <Delegations<T>>::get(*delegation).ok_or(Error::<T>::DelegationNotFound)?;
		consumed_weight += T::DbWeight::get().reads(1);

		// Check if already revoked
		if !delegation_node.revoked {
			// First revoke all children recursively
			let remaining_revocations = max_revocations
				.checked_sub(1)
				.ok_or(Error::<T>::ExceededRevocationBounds)?;
			Self::revoke_children(delegation, sender, remaining_revocations).map(|(r, w)| {
				revocations += r;
				consumed_weight += w;
			})?;

			// If we run out of revocation gas, we only revoke children. The tree will be
			// changed but is still valid.
			ensure!(revocations < max_revocations, Error::<T>::ExceededRevocationBounds);

			// Set revoked flag and store delegation node
			delegation_node.revoked = true;
			<Delegations<T>>::insert(*delegation, delegation_node);
			consumed_weight += T::DbWeight::get().writes(1);
			// Deposit event that the delegation has been revoked
			Self::deposit_event(Event::DelegationRevoked(sender.clone(), *delegation));
			revocations += 1;
		}
		Ok((revocations, consumed_weight))
	}

	/// Revoke all children of a delegation
	fn revoke_children(
		delegation: &T::DelegationNodeId,
		sender: &T::DidIdentifier,
		max_revocations: u32,
	) -> Result<(u32, Weight), DispatchError> {
		let mut revocations: u32 = 0;
		let mut consumed_weight: Weight = 0;
		// Check if there's a child vector in the storage
		if let Some(children) = <Children<T>>::get(delegation) {
			consumed_weight += T::DbWeight::get().reads(1);

			// Iterate child vector and revoke all nodes
			for child in children {
				let remaining_revocations = max_revocations
					.checked_sub(revocations)
					.ok_or(Error::<T>::ExceededRevocationBounds)?;

				// Check whether we ran out of gas
				ensure!(remaining_revocations > 0, Error::<T>::ExceededRevocationBounds);

				Self::revoke(&child, sender, remaining_revocations).map(|(r, w)| {
					revocations += r;
					consumed_weight += w;
				})?;
			}
		}
		Ok((revocations, consumed_weight + T::DbWeight::get().reads(1)))
	}

	// Add a child node into the delegation hierarchy
	fn add_child(child: T::DelegationNodeId, parent: T::DelegationNodeId) {
		// Get the children vector or initialize an empty one if none
		let mut children = <Children<T>>::get(parent).unwrap_or_default();
		children.push(child);
		<Children<T>>::insert(parent, children);
	}
}
