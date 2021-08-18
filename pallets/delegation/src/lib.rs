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

//! # Delegation Pallet
//!
//! Provides means of adding KILT delegations on chain and revoking them. Each
//! delegation is based on a specific CType. The most basic delegation is just a
//! root node to which you can add further delegations by
//! appending them to the root node resulting in a tree structure.
//!
//! - [`Config`]
//! - [`Call`]
//! - [`Pallet`]
//!
//! ### Terminology
//!
//! - **Claimer:**: A user which claims properties about themselves in the
//!   format of a CType. This could be a person which claims to have a valid
//!   driver's license.
//!
//! - **Attester:**: An entity which checks a user's claim and approves its
//!   validity. This could be a Citizens Registration Office which issues
//!   drivers licenses.
//!
//! - **Verifier:**: An entity which wants to check a user's claim by checking
//!   the provided attestation.
//!
//! - **CType:**: CTypes are claim types. In everyday language, they are
//!   standardised structures for credentials. For example, a company may need a
//!   standard identification credential to identify workers that includes their
//!   full name, date of birth, access level and id number. Each of these are
//!   referred to as an attribute of a credential.
//!
//! - **Attestation:**: An approved or revoked user's claim in the format of a
//!   CType.
//!
//! - **Delegation:**: An attestation which is not issued by the attester
//!   directly but via a (chain of) delegations which entitle the delegated
//!   attester. This could be an employe of a company which is authorized to
//!   sign documents for their superiors.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//! - `create_root` - Create a new root delegation based on a specific CType.
//! - `add_delegation` - Add a new delegation node to an existing delegation
//!   node acting as the root for the newly added node.
//! - `revoke_root` - Revoke a delegation root which implicitly revokes the
//!   entire delegation tree.
//! - `revoke_delegation` - Revoke a delegation node and its sub delegations.
//!
//! ## Assumptions
//!
//! - The maximum depth of a delegation tree is bounded by `MaxParentChecks`.
//!   This is not enforced when adding new delegations. However, you can only
//!   revoke up to `MaxParentChecks` many sub-delegations.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

pub mod default_weights;
pub mod delegation_hierarchy;
pub mod migrations;

#[cfg(any(feature = "mock", test))]
pub mod mock;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

#[cfg(test)]
mod tests;

mod deprecated;

pub use crate::{default_weights::WeightInfo, delegation_hierarchy::*, pallet::*};

use frame_support::{dispatch::DispatchResult, ensure, pallet_prelude::Weight, traits::Get};
use sp_runtime::{traits::Hash, DispatchError};
use sp_std::vec::Vec;

use migrations::*;

#[frame_support::pallet]
pub mod pallet {

	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	/// Type of a delegation node identifier.
	pub type DelegationNodeIdOf<T> = <T as Config>::DelegationNodeId;

	/// Type of a delegator or a delegate.
	pub type DelegatorIdOf<T> = <T as Config>::DelegationEntityId;

	/// The type of a CType hash.
	pub type CtypeHashOf<T> = ctype::CtypeHashOf<T>;

	/// Type of a signature verification operation over the delegation details.
	pub type DelegationSignatureVerificationOf<T> = <T as Config>::DelegationSignatureVerification;

	/// Type of the signature that the delegate generates over the delegation
	/// information.
	pub type DelegateSignatureTypeOf<T> = <DelegationSignatureVerificationOf<T> as VerifyDelegateSignature>::Signature;

	#[pallet::config]
	pub trait Config: frame_system::Config + ctype::Config {
		type Signature: Parameter;
		type DelegationSignatureVerification: VerifyDelegateSignature<
			DelegateId = Self::DelegationEntityId,
			Payload = Vec<u8>,
			Signature = Self::Signature,
		>;
		type DelegationEntityId: Parameter;
		type DelegationNodeId: Parameter + Copy + AsRef<[u8]> + Eq + PartialEq + Ord + PartialOrd;
		type EnsureOrigin: EnsureOrigin<Success = DelegatorIdOf<Self>, <Self as frame_system::Config>::Origin>;
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		#[pallet::constant]
		type MaxSignatureByteLength: Get<u16>;

		/// Maximum number of revocations.
		#[pallet::constant]
		type MaxRevocations: Get<u32>;

		/// Maximum number of upwards traversals of the delegation tree from a
		/// node to the root and thus the depth of the delegation tree.
		#[pallet::constant]
		type MaxParentChecks: Get<u32>;

		/// Maximum number of all children for a delegation node. For a binary
		/// tree, this should be twice the maximum depth of the tree, i.e.
		/// `2 ^ MaxParentChecks`.
		#[pallet::constant]
		type MaxChildren: Get<u32> + Clone;
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<(), &'static str> {
			migrations::DelegationStorageMigrator::<T>::pre_migrate()
		}

		fn on_runtime_upgrade() -> Weight {
			migrations::DelegationStorageMigrator::<T>::migrate()
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade() -> Result<(), &'static str> {
			migrations::DelegationStorageMigrator::<T>::post_migrate()
		}
	}

	/// Contains the latest storage version deployed.
	#[pallet::storage]
	#[pallet::getter(fn last_version_migration_used)]
	pub(crate) type StorageVersion<T> = StorageValue<_, DelegationStorageVersion, ValueQuery>;

	/// Delegation nodes stored on chain.
	///
	/// It maps from a node ID to the node details.
	#[pallet::storage]
	#[pallet::getter(fn delegation_nodes)]
	pub type DelegationNodes<T> = StorageMap<_, Blake2_128Concat, DelegationNodeIdOf<T>, DelegationNode<T>>;

	/// Delegation hierarchies stored on chain.
	///
	/// It maps for a (root) node ID to the hierarchy details.
	#[pallet::storage]
	#[pallet::getter(fn delegation_hierarchies)]
	pub type DelegationHierarchies<T> =
		StorageMap<_, Blake2_128Concat, DelegationNodeIdOf<T>, DelegationHierarchyDetails<T>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new hierarchy has been created.
		/// \[creator ID, root node ID, CTYPE hash\]
		HierarchyCreated(DelegatorIdOf<T>, DelegationNodeIdOf<T>, CtypeHashOf<T>),
		/// A hierarchy has been revoked.
		/// \[revoker ID, root node ID\]
		HierarchyRevoked(DelegatorIdOf<T>, DelegationNodeIdOf<T>),
		/// A new delegation has been created.
		/// \[creator ID, root node ID, delegation node ID, parent node ID,
		/// delegate ID, permissions\]
		DelegationCreated(
			DelegatorIdOf<T>,
			DelegationNodeIdOf<T>,
			DelegationNodeIdOf<T>,
			DelegationNodeIdOf<T>,
			DelegatorIdOf<T>,
			Permissions,
		),
		/// A delegation has been revoked.
		/// \[revoker ID, delegation node ID\]
		DelegationRevoked(DelegatorIdOf<T>, DelegationNodeIdOf<T>),
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
		/// No delegate with the given ID stored on chain.
		DelegateNotFound,
		/// There is already a hierarchy with the same ID stored on chain.
		HierarchyAlreadyExists,
		/// No hierarchy with the given ID stored on chain.
		HierarchyNotFound,
		/// Max number of nodes checked without verifying the given condition.
		MaxSearchDepthReached,
		/// Max number of nodes checked without verifying the given condition.
		NotOwnerOfParentDelegation,
		/// The delegation creator is not allowed to write the delegation
		/// because he is not the owner of the delegation root node.
		NotOwnerOfDelegationHierarchy,
		/// No parent delegation with the given ID stored on chain.
		ParentDelegationNotFound,
		/// The parent delegation has previously been revoked.
		ParentDelegationRevoked,
		/// The delegation revoker is not allowed to revoke the delegation.
		UnauthorizedRevocation,
		/// The delegation creator is not allowed to create the delegation.
		UnauthorizedDelegation,
		/// Max number of delegation nodes revocation has been reached for the
		/// operation.
		ExceededRevocationBounds,
		/// The max number of revocation exceeds the limit for the pallet.
		MaxRevocationsTooLarge,
		/// The max number of parent checks exceeds the limit for the pallet.
		MaxParentChecksTooLarge,
		/// An error that is not supposed to take place, yet it happened.
		InternalError,
		/// The max number of all children has been reached for the
		/// corresponding delegation node.
		MaxChildrenExceeded,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new delegation root associated with a given CType hash.
		///
		/// The new root will allow a new trust hierarchy to be created by
		/// adding children delegations to the root.
		///
		/// There must be no delegation with the same ID stored on chain, while
		/// there must be already a CType with the given hash stored in the
		/// CType pallet.
		///
		/// The dispatch origin must be `DelegationEntityId`.
		///
		/// Emits `RootCreated`.
		///
		/// # <weight>
		/// Weight: O(1)
		/// - Reads: [Origin Account], Roots, CTypes
		/// - Writes: Roots
		/// # </weight>
		#[pallet::weight(<T as Config>::WeightInfo::create_hierarchy())]
		pub fn create_hierarchy(
			origin: OriginFor<T>,
			root_node_id: DelegationNodeIdOf<T>,
			ctype_hash: CtypeHashOf<T>,
		) -> DispatchResult {
			let creator = <T as Config>::EnsureOrigin::ensure_origin(origin)?;

			ensure!(
				!<DelegationHierarchies<T>>::contains_key(&root_node_id),
				Error::<T>::HierarchyAlreadyExists
			);

			ensure!(
				<ctype::Ctypes<T>>::contains_key(&ctype_hash),
				<ctype::Error<T>>::CTypeNotFound
			);

			log::debug!("insert Delegation Root");

			Self::create_and_store_new_hierarchy(
				root_node_id,
				DelegationHierarchyDetails::<T> { ctype_hash },
				creator.clone(),
			);

			Self::deposit_event(Event::HierarchyCreated(creator, root_node_id, ctype_hash));

			Ok(())
		}

		/// Create a new delegation node.
		///
		/// The new delegation node represents a new trust hierarchy that
		/// considers the new node as its root. The owner of this node has full
		/// control over any of its direct and indirect descendants.
		///
		/// For the creation to succeed, the delegatee must provide a valid
		/// signature over the (blake256) hash of the creation operation details
		/// which include (in order) delegation id, root node id, parent id, and
		/// permissions of the new node.
		///
		/// There must be no delegation with the same id stored on chain.
		/// Furthermore, the referenced root and parent nodes must already be
		/// present on chain and contain the valid permissions and revocation
		/// status (i.e., not revoked).
		///
		/// The dispatch origin must be `DelegationEntityId`.
		///
		/// Emits `DelegationCreated`.
		///
		/// # <weight>
		/// Weight: O(1)
		/// - Reads: [Origin Account], Roots, Delegations
		/// - Writes: Delegations
		/// # </weight>
		#[pallet::weight(<T as Config>::WeightInfo::add_delegation())]
		pub fn add_delegation(
			origin: OriginFor<T>,
			delegation_id: DelegationNodeIdOf<T>,
			parent_id: DelegationNodeIdOf<T>,
			delegate: DelegatorIdOf<T>,
			permissions: Permissions,
			delegate_signature: DelegateSignatureTypeOf<T>,
		) -> DispatchResult {
			let delegator = <T as Config>::EnsureOrigin::ensure_origin(origin)?;

			ensure!(
				!<DelegationNodes<T>>::contains_key(&delegation_id),
				Error::<T>::DelegationAlreadyExists
			);

			let parent_node = <DelegationNodes<T>>::get(&parent_id).ok_or(Error::<T>::ParentDelegationNotFound)?;
			let hierarchy_root_id = parent_node.hierarchy_root_id;

			// Calculate the hash root
			let hash_root =
				Self::calculate_delegation_creation_hash(&delegation_id, &hierarchy_root_id, &parent_id, &permissions);

			// Verify that the hash root signature is correct.
			DelegationSignatureVerificationOf::<T>::verify(&delegate, &hash_root.encode(), &delegate_signature)
				.map_err(|err| match err {
					SignatureVerificationError::SignerInformationNotPresent => Error::<T>::DelegateNotFound,
					SignatureVerificationError::SignatureInvalid => Error::<T>::InvalidDelegateSignature,
				})?;

			// Check if the parent's delegate is the creator of this delegation node...
			ensure!(
				parent_node.details.owner == delegator,
				Error::<T>::NotOwnerOfParentDelegation
			);
			// ... and that the node has not been revoked...
			ensure!(!parent_node.details.revoked, Error::<T>::ParentDelegationRevoked);
			// ... and that has permission to delegate
			ensure!(
				(parent_node.details.permissions & Permissions::DELEGATE) == Permissions::DELEGATE,
				Error::<T>::UnauthorizedDelegation
			);

			Self::store_delegation_under_parent(
				delegation_id,
				DelegationNode::new_node(
					hierarchy_root_id,
					parent_id,
					DelegationDetails {
						owner: delegate.clone(),
						permissions,
						revoked: false,
					},
				),
				parent_id,
				parent_node,
			)?;

			Self::deposit_event(Event::DelegationCreated(
				delegator,
				hierarchy_root_id,
				delegation_id,
				parent_id,
				delegate,
				permissions,
			));

			Ok(())
		}

		/// Revoke a delegation node (potentially a root node) and all its
		/// children.
		///
		/// Revoking a delegation node results in the trust hierarchy starting
		/// from the given node being revoked. Nevertheless, revocation starts
		/// from the leave nodes upwards, so if the operation ends prematurely
		/// because it runs out of gas, the delegation state would be consistent
		/// as no child would "survive" its parent. As a consequence, if the
		/// given node is revoked, the trust hierarchy with the node as root is
		/// to be considered revoked.
		///
		/// The dispatch origin must be `DelegationEntityId`.
		///
		/// Emits C * `DelegationRevoked`.
		///
		/// # <weight>
		/// Weight: O(C) where C is the number of children of the delegation
		/// node which is bounded by `max_children`.
		/// - Reads: [Origin Account], Roots, C * Delegations, C * Children.
		/// - Writes: Roots, C * Delegations
		/// # </weight>
		#[pallet::weight(
			<T as Config>::WeightInfo::revoke_delegation_root_child(*max_revocations, *max_parent_checks)
				.max(<T as Config>::WeightInfo::revoke_delegation_leaf(*max_revocations, *max_parent_checks)))]
		pub fn revoke_delegation(
			origin: OriginFor<T>,
			delegation_id: DelegationNodeIdOf<T>,
			max_parent_checks: u32,
			max_revocations: u32,
		) -> DispatchResultWithPostInfo {
			let invoker = <T as Config>::EnsureOrigin::ensure_origin(origin)?;

			ensure!(
				<DelegationNodes<T>>::contains_key(&delegation_id),
				Error::<T>::DelegationNotFound
			);

			ensure!(
				max_parent_checks <= T::MaxParentChecks::get(),
				Error::<T>::MaxParentChecksTooLarge
			);

			ensure!(
				max_revocations <= T::MaxRevocations::get(),
				Error::<T>::MaxRevocationsTooLarge
			);

			let (authorized, parent_checks) = Self::is_delegating(&invoker, &delegation_id, max_parent_checks)?;
			ensure!(authorized, Error::<T>::UnauthorizedRevocation);

			// Revoke the delegation and recursively all of its children (add 1 to
			// max_revocations to account for the node itself)
			let (revocation_checks, _) = Self::revoke(&delegation_id, &invoker, max_revocations.saturating_add(1))?;

			// If the revoked node is a root node, emit also a HierarchyRevoked event.
			if DelegationHierarchies::<T>::contains_key(&delegation_id) {
				Self::deposit_event(Event::HierarchyRevoked(invoker, delegation_id));
			}

			// Add worst case reads from `is_delegating`
			Ok(Some(
				<T as Config>::WeightInfo::revoke_delegation_root_child(revocation_checks, parent_checks).max(
					<T as Config>::WeightInfo::revoke_delegation_leaf(revocation_checks, parent_checks),
				),
			)
			.into())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Calculate the hash of all values of a delegation creation transaction.
	///
	/// # <weight>
	/// Weight: O(1)
	/// # </weight>
	fn calculate_delegation_creation_hash(
		delegation_id: &DelegationNodeIdOf<T>,
		root_id: &DelegationNodeIdOf<T>,
		parent_id: &DelegationNodeIdOf<T>,
		permissions: &Permissions,
	) -> T::Hash {
		// Add all values to an u8 vector.
		let mut hashed_values: Vec<u8> = delegation_id.as_ref().to_vec();
		hashed_values.extend_from_slice(root_id.as_ref());
		hashed_values.extend_from_slice(parent_id.as_ref());
		hashed_values.extend_from_slice(permissions.as_u8().as_ref());
		// Hash the resulting vector
		T::Hashing::hash(&hashed_values)
	}

	// Creates a new root node with the given details and store the new hierarchy in
	// the hierarchies storage and the new root node in the nodes storage.
	fn create_and_store_new_hierarchy(
		root_id: DelegationNodeIdOf<T>,
		hierarchy_details: DelegationHierarchyDetails<T>,
		hierarchy_owner: DelegatorIdOf<T>,
	) {
		let root_node = DelegationNode::new_root_node(root_id, DelegationDetails::default_with_owner(hierarchy_owner));
		<DelegationNodes<T>>::insert(root_id, root_node);
		<DelegationHierarchies<T>>::insert(root_id, hierarchy_details);
	}

	// Adds the given node to the storage and updates the parent node to include the
	// given node as child.
	//
	// This function assumes that the parent node is already stored on the chain. If
	// not, the behaviour of the system is undefined.
	fn store_delegation_under_parent(
		delegation_id: DelegationNodeIdOf<T>,
		delegation_node: DelegationNode<T>,
		parent_id: DelegationNodeIdOf<T>,
		mut parent_node: DelegationNode<T>,
	) -> DispatchResult {
		<DelegationNodes<T>>::insert(delegation_id, delegation_node);
		// Add the new node as a child of that node
		parent_node.try_add_child(delegation_id)?;
		<DelegationNodes<T>>::insert(parent_id, parent_node);
		Ok(())
	}

	/// Check if an identity is the owner of the given delegation node or any
	/// node up the hierarchy, and if the delegation has not been yet revoked.
	///
	/// It checks whether the conditions are required for the given node,
	/// otherwise it goes up up to `max_parent_checks` nodes, including the root
	/// node, to check whether the given identity is a valid delegator of the
	/// given delegation.
	///
	/// # <weight>
	/// Weight: O(P) where P is the number of steps required to verify that
	/// the dispatch Origin controls the delegation entitled to revoke the
	/// attestation. It is bounded by `max_parent_checks`.
	/// - Reads: Roots, P * Delegations
	/// # </weight>
	pub fn is_delegating(
		identity: &DelegatorIdOf<T>,
		delegation: &DelegationNodeIdOf<T>,
		max_parent_checks: u32,
	) -> Result<(bool, u32), DispatchError> {
		let delegation_node = <DelegationNodes<T>>::get(delegation).ok_or(Error::<T>::DelegationNotFound)?;

		// Check if the given account is the owner of the delegation and that the
		// delegation has not been removed
		if &delegation_node.details.owner == identity {
			Ok((!delegation_node.details.revoked, 0u32))
		} else if let Some(parent) = delegation_node.parent {
			// Only decrease (and perhaps fail) remaining_lookups if there are more parents
			// to visit
			let remaining_lookups = max_parent_checks
				.checked_sub(1)
				.ok_or(Error::<T>::MaxSearchDepthReached)?;

			// Recursively check upwards in hierarchy
			Self::is_delegating(identity, &parent, remaining_lookups)
		} else {
			// Return false and return max_parent_checks as no other check is performed
			Ok((false, max_parent_checks))
		}
	}

	/// Revokes all children of a delegation.
	/// Returns the number of revoked delegations and the consumed weight.
	///
	/// # <weight>
	/// Weight: O(C) where C is the number of children of the delegation node
	/// which is bounded by `max_children`.
	/// - Reads: C * Children, C * Delegations
	/// - Writes: C * Delegations
	/// # </weight>
	fn revoke_children(
		delegation: &DelegationNodeIdOf<T>,
		sender: &DelegatorIdOf<T>,
		max_revocations: u32,
	) -> Result<(u32, Weight), DispatchError> {
		let mut revocations: u32 = 0;
		let mut consumed_weight: Weight = 0;
		if let Some(delegation_node) = <DelegationNodes<T>>::get(delegation) {
			// Iterate children and revoke all nodes
			for child in delegation_node.children.iter() {
				let remaining_revocations = max_revocations
					.checked_sub(revocations)
					.ok_or(Error::<T>::ExceededRevocationBounds)?;

				// Check whether we ran out of gas
				ensure!(remaining_revocations > 0, Error::<T>::ExceededRevocationBounds);

				Self::revoke(child, sender, remaining_revocations).map(|(r, w)| {
					revocations = revocations.saturating_add(r);
					consumed_weight = consumed_weight.saturating_add(w);
				})?;
			}
		}
		Ok((revocations, consumed_weight.saturating_add(T::DbWeight::get().reads(1))))
	}

	/// Revoke a delegation and all of its children recursively.
	///
	/// Emits DelegationRevoked for each revoked node.
	///
	/// # <weight>
	/// Weight: O(C) where C is the number of children of the root which is
	/// bounded by `max_children`.
	/// - Reads: C * Delegations
	/// - Writes: C * Delegations
	/// # </weight>
	fn revoke(
		delegation: &DelegationNodeIdOf<T>,
		sender: &DelegatorIdOf<T>,
		max_revocations: u32,
	) -> Result<(u32, Weight), DispatchError> {
		let mut revocations: u32 = 0;
		let mut consumed_weight: Weight = 0;
		// Retrieve delegation node from storage
		let mut delegation_node = <DelegationNodes<T>>::get(*delegation).ok_or(Error::<T>::DelegationNotFound)?;
		consumed_weight = consumed_weight.saturating_add(T::DbWeight::get().reads(1));

		// Check if already revoked
		if !delegation_node.details.revoked {
			// First revoke all children recursively
			let remaining_revocations = max_revocations
				.checked_sub(1)
				.ok_or(Error::<T>::ExceededRevocationBounds)?;
			Self::revoke_children(delegation, sender, remaining_revocations).map(|(r, w)| {
				revocations = revocations.saturating_add(r);
				consumed_weight = consumed_weight.saturating_add(w);
			})?;

			// If we run out of revocation gas, we only revoke children. The tree will be
			// changed but is still valid.
			ensure!(revocations < max_revocations, Error::<T>::ExceededRevocationBounds);

			// Set revoked flag and store delegation node
			delegation_node.details.revoked = true;
			<DelegationNodes<T>>::insert(*delegation, delegation_node);
			consumed_weight = consumed_weight.saturating_add(T::DbWeight::get().writes(1));
			// Deposit event that the delegation has been revoked
			Self::deposit_event(Event::DelegationRevoked(sender.clone(), *delegation));
			revocations = revocations.saturating_add(1);
		}
		Ok((revocations, consumed_weight))
	}
}
