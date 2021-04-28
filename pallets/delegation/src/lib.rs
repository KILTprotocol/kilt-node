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
#![allow(clippy::unused_unit)]

#[cfg(test)]
mod tests;

#[cfg(any(feature = "mock", test))]
pub mod mock;

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

use bitflags::*;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

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
		pub ctype_hash: T::Hash,
		/// The DID of the root owner.
		pub owner: T::DidIdentifier,
		/// The flag indicating whether the root has been revoked or not.
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

	/// A node representing a node in the deleagation hierarchy.
	#[derive(Clone, Debug, Encode, Decode, PartialEq)]
	pub struct DelegationNode<T: Config> {
		/// The ID of the delegation hierarchy root.
		pub root_id: T::DelegationNodeId,
		/// \[OPTIONAL\] The ID of the parent node. If None, the node is
		/// considered a child of the root node.
		pub parent: Option<T::DelegationNodeId>,
		/// The DID of the owner of the delegation node, i.e., the delegate.
		pub owner: T::DidIdentifier,
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
		/// * permissions: the permission flags for the operations the delegate
		///   is allowed to perform
		pub fn new_root_child(root_id: T::DelegationNodeId, owner: T::DidIdentifier, permissions: Permissions) -> Self {
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
		/// * permissions: the permission flags for the operations the delegate
		///   is allowed to perform
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

	/// An operation to create a new delegation root.
	///
	/// The struct implements the DidOperation trait, and as such it must
	/// contain information about the caller's DID, the type of DID key
	/// required to verify the operation signature, and the tx counter to
	/// protect against replay attacks.
	#[derive(Clone, Decode, Encode, PartialEq)]
	pub struct DelegationRootCreationOperation<T: Config> {
		/// The DID of the root creator.
		pub creator_did: T::DidIdentifier,
		/// The ID of the root node. It has to be unique.
		pub root_id: T::DelegationNodeId,
		/// The CTYPE hash that delegates can use for attestations.
		pub ctype_hash: T::Hash,
		/// The DID tx counter.
		pub tx_counter: u64,
	}

	impl<T: Config> DidOperation<T> for DelegationRootCreationOperation<T> {
		fn get_verification_key_type(&self) -> did::DidVerificationKeyRelationship {
			did::DidVerificationKeyRelationship::CapabilityDelegation
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

	/// An operation to create a new delegation node.
	///
	/// The struct implements the DidOperation trait, and as such it must
	/// contain information about the caller's DID, the type of DID key
	/// required to verify the operation signature, and the tx counter to
	/// protect against replay attacks.
	#[derive(Clone, Decode, Encode, PartialEq)]
	pub struct DelegationCreationOperation<T: Config> {
		/// The DID of the node creator.
		pub creator_did: T::DidIdentifier,
		/// The ID of the new delegation node. It has to be unique.
		pub delegation_id: T::DelegationNodeId,
		/// The ID of the delegation hierarchy root to add this delegation to.
		pub root_id: T::DelegationNodeId,
		/// \[OPTIONAL\] The ID of the parent node to verify that the creator is
		/// allowed to create a new delegation. If None, the verification is
		/// performed against the provided root node.
		pub parent_id: Option<T::DelegationNodeId>,
		/// The DID of the delegate.
		pub delegate_did: T::DidIdentifier,
		/// The permission flags for the operations the delegate is allowed to
		/// perform
		pub permissions: Permissions,
		/// The delegate's signature over the new delegation ID, root ID, parent
		/// ID, and permission flags.
		pub delegate_signature: did::DidSignature,
		/// The DID tx counter.
		pub tx_counter: u64,
	}

	impl<T: Config> DidOperation<T> for DelegationCreationOperation<T> {
		fn get_verification_key_type(&self) -> did::DidVerificationKeyRelationship {
			did::DidVerificationKeyRelationship::CapabilityDelegation
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

	/// An operation to revoke a delegation root.
	///
	/// The struct implements the DidOperation trait, and as such it must
	/// contain information about the caller's DID, the type of DID key
	/// required to verify the operation signature, and the tx counter to
	/// protect against replay attacks.
	#[derive(Clone, Decode, Encode, PartialEq)]
	pub struct DelegationRootRevocationOperation<T: Config> {
		/// The DID of the revoker.
		pub revoker_did: T::DidIdentifier,
		/// The ID of the delegation root to revoke.
		pub root_id: T::DelegationNodeId,
		/// The maximum number of nodes descending from the root to revoke as a
		/// consequence of the root revocation. The revocation starts from the
		/// leaves, so in case of values that are not large enough, the nodes at
		/// the bottom of the hierarchy might get revoked while the ones higher
		/// up, including the root node, might not.
		pub max_children: u32,
		/// The DID tx counter.
		pub tx_counter: u64,
	}

	impl<T: Config> DidOperation<T> for DelegationRootRevocationOperation<T> {
		fn get_verification_key_type(&self) -> did::DidVerificationKeyRelationship {
			did::DidVerificationKeyRelationship::CapabilityDelegation
		}

		fn get_did(&self) -> &T::DidIdentifier {
			&self.revoker_did
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
				.field(&self.revoker_did)
				.field(&self.root_id)
				.field(&self.max_children)
				.field(&self.tx_counter)
				.finish()
		}
	}

	/// An operation to revoke a new delegation node.
	///
	/// The struct implements the DidOperation trait, and as such it must
	/// contain information about the caller's DID, the type of DID key
	/// required to verify the operation signature, and the tx counter to
	/// protect against replay attacks.
	#[derive(Clone, Decode, Encode, PartialEq)]
	pub struct DelegationRevocationOperation<T: Config> {
		/// The DID of the revoker.
		pub revoker_did: T::DidIdentifier,
		/// The ID of the delegation root to revoke.
		pub delegation_id: T::DelegationNodeId,
		/// In case the revoker is not the owner of the specified node, the
		/// number of parent nodes to check to verify that the revoker is
		/// authorised to perform the revokation. The evaluation terminates when
		/// a valid node is reached, when the whole hierarchy including the root
		/// node has been checked, or when the max number of parents is reached.
		pub max_parent_checks: u32,
		/// The maximum number of nodes descending from this one to revoke as a
		/// consequence of this node revocation. The revocation starts from the
		/// leaves, so in case of values that are not large enough, the nodes at
		/// the bottom of the hierarchy might get revoked while the ones higher
		/// up, including this node, might not.
		pub max_revocations: u32,
		/// The DID tx counter.
		pub tx_counter: u64,
	}

	impl<T: Config> DidOperation<T> for DelegationRevocationOperation<T> {
		fn get_verification_key_type(&self) -> did::DidVerificationKeyRelationship {
			did::DidVerificationKeyRelationship::CapabilityDelegation
		}

		fn get_did(&self) -> &T::DidIdentifier {
			&self.revoker_did
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
				.field(&self.revoker_did)
				.field(&self.delegation_id)
				.field(&self.max_parent_checks)
				.field(&self.max_revocations)
				.field(&self.tx_counter)
				.finish()
		}
	}

	#[pallet::config]
	pub trait Config: frame_system::Config + ctype::Config + did::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type WeightInfo: WeightInfo;
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

	/// Delegation root nodes stored on chain.
	///
	/// It maps from a root node ID to the full root node.
	#[pallet::storage]
	#[pallet::getter(fn roots)]
	pub type Roots<T> = StorageMap<_, Blake2_128Concat, <T as Config>::DelegationNodeId, DelegationRoot<T>>;

	/// Delegation nodes stored on chain.
	///
	/// It maps from a node ID to the full delegation node.
	#[pallet::storage]
	#[pallet::getter(fn delegations)]
	pub type Delegations<T> = StorageMap<_, Blake2_128Concat, <T as Config>::DelegationNodeId, DelegationNode<T>>;

	/// Children delegation nodes.
	///
	/// It maps from a delegation node ID, including the root node, to the list
	/// of children nodes, sorted by time of creation.
	#[pallet::storage]
	#[pallet::getter(fn children)]
	pub type Children<T> =
		StorageMap<_, Blake2_128Concat, <T as Config>::DelegationNodeId, Vec<<T as Config>::DelegationNodeId>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new root has been created.
		/// \[creator DID, root node ID, CTYPE hash\]
		RootCreated(T::DidIdentifier, T::DelegationNodeId, T::Hash),
		/// A root has been revoked.
		/// \[revoker DID, root node ID\]
		RootRevoked(T::DidIdentifier, T::DelegationNodeId),
		/// A new delegation has been created.
		/// \[creator DID, root node ID, delegation node ID, parent node ID,
		/// delegate DID, permissions\]
		DelegationCreated(
			T::DidIdentifier,
			T::DelegationNodeId,
			T::DelegationNodeId,
			Option<T::DelegationNodeId>,
			T::DidIdentifier,
			Permissions,
		),
		/// A delegation has been revoked.
		/// \[revoker DID, delegation node ID\]
		DelegationRevoked(T::DidIdentifier, T::DelegationNodeId),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// There is already a delegation node with the same ID stored on chain.
		DelegationAlreadyExists,
		/// The delegate's signature for the delegation creation operation is
		/// invalid.
		InvalidDelegateSignature,
		/// No delegation with the given ID stored on chain.
		DelegationNotFound,
		/// No delegate with the given DID stored on chain.
		DelegateNotFound,
		/// There is already a root node with the same ID stored on chain.
		RootAlreadyExists,
		/// No root delegation with the given ID stored on chain.
		RootNotFound,
		/// Max number of nodes checked without verifying the given condition.
		MaxSearchDepthReached,
		/// Max number of nodes checked without verifying the given condition.
		NotOwnerOfParentDelegation,
		/// The delegation creator is not allowed to write the delegation
		/// because he is not the owner of the delegation root node.
		NotOwnerOfRootDelegation,
		/// No parent delegation with the given ID stored on chain.
		ParentDelegationNotFound,
		/// The delegation revoker is not allowed to revoke the delegation.
		UnauthorizedRevocation,
		/// The delegation creator is not allowed to create the delegation.
		UnauthorizedDelegation,
		/// Max number of delegation nodes revocation has been reached for the
		/// operation.
		ExceededRevocationBounds,
		/// An error that is not supposed to take place, yet it happened.
		InternalError,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Submits a new DelegationRootCreationOperation operation.
		///
		/// * origin: the origin of the transaction
		/// * operation: the DelegationRootCreationOperation operation
		/// * signature: the signature over the byte-encoded operation
		#[pallet::weight(<T as Config>::WeightInfo::submit_delegation_root_creation_operation())]
		pub fn submit_delegation_root_creation_operation(
			origin: OriginFor<T>,
			operation: DelegationRootCreationOperation<T>,
			signature: did::DidSignature,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;

			// Check if DID exists, if counter is valid, if signature is valid, and increase
			// DID tx counter
			did::pallet::Pallet::verify_operation_validity_and_increase_did_nonce(&operation, &signature)
				.map_err(<did::Error<T>>::from)?;

			ensure!(
				!<Roots<T>>::contains_key(&operation.root_id),
				Error::<T>::RootAlreadyExists
			);

			ensure!(
				<ctype::Ctypes<T>>::contains_key(&operation.ctype_hash),
				<ctype::Error<T>>::CTypeNotFound
			);

			log::debug!("insert Delegation Root");
			<Roots<T>>::insert(
				&operation.root_id,
				DelegationRoot::new(operation.ctype_hash, operation.creator_did.clone()),
			);

			Self::deposit_event(Event::RootCreated(
				operation.creator_did,
				operation.root_id,
				operation.ctype_hash,
			));

			Ok(None.into())
		}

		/// Submits a new DelegationCreationOperation operation.
		///
		/// * origin: the origin of the transaction
		/// * operation: the DelegationCreationOperation operation
		/// * signature: the signature over the byte-encoded operation
		#[pallet::weight(<T as Config>::WeightInfo::submit_delegation_creation_operation())]
		pub fn submit_delegation_creation_operation(
			origin: OriginFor<T>,
			operation: DelegationCreationOperation<T>,
			signature: did::DidSignature,
		) -> DispatchResultWithPostInfo {
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
				did::DidVerificationKeyRelationship::Authentication,
			)
			.map_err(|err| match err {
				// Should never happen as a DID has always a valid authentication key and UrlErrors are never thrown
				// here.
				did::DidError::StorageError(_) | did::DidError::UrlError(_) => Error::<T>::DelegateNotFound,
				did::DidError::SignatureError(_) => Error::<T>::InvalidDelegateSignature,
				// Should never happen as we are not checking the delegate's DID tx counter.
				did::DidError::InternalError => Error::<T>::InternalError,
			})?;

			ensure!(
				!<Delegations<T>>::contains_key(&operation.delegation_id),
				Error::<T>::DelegationAlreadyExists
			);

			let root = <Roots<T>>::get(&operation.root_id).ok_or(Error::<T>::RootNotFound)?;

			// Computes the delegation parent. Either the given parent (if allowed) or the
			// root node.
			let parent_id = if let Some(parent_id) = operation.parent_id {
				let parent_node = <Delegations<T>>::get(&parent_id).ok_or(Error::<T>::ParentDelegationNotFound)?;

				// Check if the parent's delegate is the creator of this delegation node...
				ensure!(
					parent_node.owner == operation.creator_did,
					Error::<T>::NotOwnerOfParentDelegation
				);
				// ... and has permission to delegate
				ensure!(
					(parent_node.permissions & Permissions::DELEGATE) == Permissions::DELEGATE,
					Error::<T>::UnauthorizedDelegation
				);

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
					root.owner == operation.creator_did,
					Error::<T>::NotOwnerOfRootDelegation
				);

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

			// Regardless of the node returned as parent, add the new node as a child of
			// that node
			Self::add_child(operation.delegation_id, parent_id);

			Self::deposit_event(Event::DelegationCreated(
				operation.creator_did,
				operation.delegation_id,
				operation.root_id,
				operation.parent_id,
				operation.delegate_did,
				operation.permissions,
			));

			Ok(None.into())
		}

		/// Submits a new DelegationRootRevocationOperation operation.
		///
		/// * origin: the origin of the transaction
		/// * operation: the DelegationRootRevocationOperation operation
		/// * signature: the signature over the byte-encoded operation
		#[pallet::weight(<T as Config>::WeightInfo::submit_delegation_root_revocation_operation(operation.max_children))]
		pub fn submit_delegation_root_revocation_operation(
			origin: OriginFor<T>,
			operation: DelegationRootRevocationOperation<T>,
			signature: did::DidSignature,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;

			// Check if DID exists, if counter is valid, if signature is valid, and increase
			// DID tx counter
			did::pallet::Pallet::verify_operation_validity_and_increase_did_nonce(&operation, &signature)
				.map_err(<did::Error<T>>::from)?;

			let mut root = <Roots<T>>::get(&operation.root_id).ok_or(Error::<T>::RootNotFound)?;

			ensure!(root.owner == operation.revoker_did, Error::<T>::UnauthorizedRevocation);

			let consumed_weight: Weight = if !root.revoked {
				// Recursively revoke all children
				let (remaining_revocations, post_weight) =
					Self::revoke_children(&operation.root_id, &operation.revoker_did, operation.max_children)?;

				// If gas left, store revoked root node
				if remaining_revocations > 0 {
					root.revoked = true;
					<Roots<T>>::insert(&operation.root_id, root);
				}
				post_weight.saturating_add(T::DbWeight::get().writes(1))
			} else {
				0
			};

			Self::deposit_event(Event::RootRevoked(operation.revoker_did, operation.root_id));

			Ok(Some(consumed_weight.saturating_add(T::DbWeight::get().reads(1))).into())
		}

		/// Submits a new DelegationRevocationOperation operation.
		///
		/// * origin: the origin of the transaction
		/// * operation: the DelegationRevocationOperation operation
		/// * signature: the signature over the byte-encoded operation
		#[pallet::weight(<T as Config>::WeightInfo::revoke_delegation_leaf(operation.max_parent_checks.saturating_add(1)).max(<T as Config>::WeightInfo::submit_delegation_revocation_operation(operation.max_parent_checks.saturating_add(1))))]
		pub fn submit_delegation_revocation_operation(
			origin: OriginFor<T>,
			operation: DelegationRevocationOperation<T>,
			signature: did::DidSignature,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;

			// Check if DID exists, if counter is valid, if signature is valid, and increase
			// DID tx counter
			did::pallet::Pallet::verify_operation_validity_and_increase_did_nonce(&operation, &signature)
				.map_err(<did::Error<T>>::from)?;

			ensure!(
				<Delegations<T>>::contains_key(&operation.delegation_id),
				Error::<T>::DelegationNotFound
			);

			ensure!(
				Self::is_delegating(
					&operation.revoker_did,
					&operation.delegation_id,
					operation.max_parent_checks
				)?,
				Error::<T>::UnauthorizedRevocation
			);

			// Revoke the delegation and recursively all of its children
			let (_, consumed_weight) = Self::revoke(
				&operation.delegation_id,
				&operation.revoker_did,
				operation.max_revocations,
			)?;

			// Add worst case reads from `is_delegating`
			//TODO: Return proper weight consumption.
			Ok(Some(
				consumed_weight
					.saturating_add(T::DbWeight::get().reads((operation.max_parent_checks.saturating_add(2)) as u64)),
			)
			.into())
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

	/// Check if an identity is the owner of the given delegation node or any
	/// node up the hierarchy, and if the delegation has not been yet revoked.
	///
	/// It checks whether the conditions are required for the given node,
	/// otherwise it goes up up to `max_parent_checks` nodes, including the root
	/// node, to check whether the given identity is a valid delegator of the
	/// given delegation.
	pub fn is_delegating(
		identity: &T::DidIdentifier,
		delegation: &T::DelegationNodeId,
		max_parent_checks: u32,
	) -> Result<bool, DispatchError> {
		let delegation_node = <Delegations<T>>::get(delegation).ok_or(Error::<T>::DelegationNotFound)?;

		// Check if the given account is the owner of the delegation and that the
		// delegation has not been removed
		if delegation_node.owner.eq(identity) {
			Ok(!delegation_node.revoked)
		} else {
			// Counter is decreased regardless of whether we are checking the parent node
			// next of the root node, as the root node is as a matter of fact the top node's
			// parent.
			let remaining_lookups = max_parent_checks
				.checked_sub(1)
				.ok_or(Error::<T>::MaxSearchDepthReached)?;

			if let Some(parent) = delegation_node.parent {
				// Recursively check upwards in hierarchy
				Self::is_delegating(identity, &parent, remaining_lookups)
			} else {
				// Return whether the given account is the owner of the root and the root has
				// not been revoked
				let root = <Roots<T>>::get(delegation_node.root_id).ok_or(Error::<T>::RootNotFound)?;
				Ok(root.owner.eq(identity) && !root.revoked)
			}
		}
	}

	// Revoke a delegation and all of its children recursively.
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

	// Revoke all children of a delegation
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
		Ok((revocations, consumed_weight.saturating_add(T::DbWeight::get().reads(1))))
	}

	// Add a child node into the delegation hierarchy
	fn add_child(child: T::DelegationNodeId, parent: T::DelegationNodeId) {
		// Get the children vector or initialize an empty one if none
		let mut children = <Children<T>>::get(parent).unwrap_or_default();
		children.push(child);
		<Children<T>>::insert(parent, children);
	}
}
