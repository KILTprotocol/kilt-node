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

pub mod default_weights;
pub mod delegation_hierarchy;
pub mod migrations;

#[cfg(any(feature = "mock", test))]
pub mod mock;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

#[cfg(test)]
mod tests;

pub use crate::{default_weights::WeightInfo, delegation_hierarchy::*, pallet::*};

use frame_support::{ensure, pallet_prelude::Weight, traits::Get};
use sp_runtime::{traits::Hash, DispatchError};
use sp_std::vec::Vec;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	/// Type of a delegation node identifier.
	pub type DelegationNodeIdOf<T> = <T as Config>::DelegationNodeId;

	/// Type of a delegator or a delegate.
	pub type DelegatorIdOf<T> = <T as Config>::DelegationEntityId;

	/// The type of a CTYPE hash.
	pub type CtypeHashOf<T> = ctype::CtypeHashOf<T>;

	/// Type of a signature verification operation over the delegation details.
	pub type DelegationSignatureVerificationOf<T> = <T as Config>::DelegationSignatureVerification;

	/// Type of the signature that the delegate generates over the delegation
	/// information.
	pub type DelegateSignatureTypeOf<T> = <DelegationSignatureVerificationOf<T> as VerifyDelegateSignature>::Signature;

	#[pallet::config]
	pub trait Config: frame_system::Config + ctype::Config {
		type DelegationSignatureVerification: VerifyDelegateSignature<
			DelegateId = Self::DelegationEntityId,
			Payload = Vec<u8>,
			Signature = Vec<u8>,
		>;
		type DelegationEntityId: Parameter;
		type DelegationNodeId: Parameter + Copy + AsRef<[u8]> + Eq + PartialEq + Ord + PartialOrd;
		type EnsureOrigin: EnsureOrigin<Success = DelegatorIdOf<Self>, <Self as frame_system::Config>::Origin>;
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		#[pallet::constant]
		type MaxSignatureByteLength: Get<u16>;
		#[pallet::constant]
		type MaxRevocations: Get<u32>;
		#[pallet::constant]
		type MaxParentChecks: Get<u32>;
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {

		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<(), &'static str> {
			let last_upgrade_version = LastUpgradeVersion::<T>::get();
			if let Ok(migrator) = migrations::StorageMigrator::<T>::try_new(last_upgrade_version) {
				return migrator.pre_migrate();
			} else {
				return Err("No migrations to apply.");
			}
		}

		fn on_runtime_upgrade() -> Weight {
			let last_upgrade_version = LastUpgradeVersion::<T>::get();
			if let Ok(migrator) = migrations::StorageMigrator::<T>::try_new(last_upgrade_version) {
				migrator.migrate()
			} else {
				0u64
			}
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade() -> Result<(), &'static str> {
			let last_upgrade_version = LastUpgradeVersion::<T>::get();
			if let Ok(migrator) = migrations::StorageMigrator::<T>::try_new(last_upgrade_version) {
				return migrator.post_migrate();
			} else {
				return Err("No migrations applied.");
			}
		}
	}

	/// Contains the version of the latest runtime upgrade performed.
	#[pallet::storage]
	#[pallet::getter(fn last_upgrade_version)]
	pub type LastUpgradeVersion<T> = StorageValue<_, u16, ValueQuery>;

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
		StorageMap<_, Blake2_128Concat, DelegationNodeIdOf<T>, DelegationHierarchyInfo<T>>;

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
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new delegation root.
		///
		/// The new root will allow a new trust hierarchy to be created by
		/// adding children delegations to the root.
		///
		/// * origin: the identifier of the delegation creator
		/// * root_id: the ID of the root node. It has to be unique
		/// * ctype_hash: the CTYPE hash that delegates can use for attestations
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

			Self::create_and_store_new_hierarchy(
				root_node_id,
				DelegationHierarchyInfo::<T> { ctype_hash },
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
		/// * origin: the identifier of the delegation creator
		/// * delegation_id: the ID of the new delegation node. It has to be
		///   unique
		/// * root_id: the ID of the delegation hierarchy root to add this
		///   delegation to
		/// * parent_id: \[OPTIONAL\] The ID of the parent node to verify that
		///   the creator is allowed to create a new delegation. If None, the
		///   verification is performed against the provided root node
		/// * delegate: the identifier of the delegate
		/// * permissions: the permission flags for the operations the delegate
		///   is allowed to perform
		/// * delegate_signature: the delegate's signature over the new
		///   delegation ID, root ID, parent ID, and permission flags
		#[pallet::weight(<T as Config>::WeightInfo::add_delegation())]
		pub fn add_delegation(
			origin: OriginFor<T>,
			delegation_id: DelegationNodeIdOf<T>,
			root_node_id: DelegationNodeIdOf<T>,
			parent_id: DelegationNodeIdOf<T>,
			delegate: DelegatorIdOf<T>,
			permissions: Permissions,
			delegate_signature: DelegateSignatureTypeOf<T>,
		) -> DispatchResult {
			let delegator = <T as Config>::EnsureOrigin::ensure_origin(origin)?;

			// Calculate the hash root
			let hash_root =
				Self::calculate_delegation_hash_root(&delegation_id, &root_node_id, &parent_id, &permissions);

			// Verify that the hash root signature is correct.
			DelegationSignatureVerificationOf::<T>::verify(&delegate, &hash_root.encode(), &delegate_signature)
				.map_err(|err| match err {
					SignatureVerificationError::SignerInformationNotPresent => Error::<T>::DelegateNotFound,
					SignatureVerificationError::SignatureInvalid => Error::<T>::InvalidDelegateSignature,
				})?;

			ensure!(
				!<DelegationNodes<T>>::contains_key(&delegation_id),
				Error::<T>::DelegationAlreadyExists
			);

			let parent_node = <DelegationNodes<T>>::get(&parent_id).ok_or(Error::<T>::ParentDelegationNotFound)?;

			// Check if the parent's delegate is the creator of this delegation node...
			ensure!(
				parent_node.details.owner == delegator,
				Error::<T>::NotOwnerOfParentDelegation
			);
			// ... and has permission to delegate
			ensure!(
				(parent_node.details.permissions & Permissions::DELEGATE) == Permissions::DELEGATE,
				Error::<T>::UnauthorizedDelegation
			);

			Self::store_delegation_under_parent(
				delegation_id,
				DelegationNode::new_node(
					root_node_id,
					parent_id,
					DelegationDetails {
						owner: delegate.clone(),
						permissions,
						revoked: false,
					},
				),
				parent_id,
				parent_node,
			);

			Self::deposit_event(Event::DelegationCreated(
				delegator,
				root_node_id,
				delegation_id,
				parent_id,
				delegate,
				permissions,
			));

			Ok(())
		}

		/// Revoke a delegation root.
		///
		/// Revoking a delegation root results in the whole trust hierarchy
		/// being revoked. Nevertheless, revocation starts from the leave nodes
		/// upwards, so if the operation ends prematurely because it runs out of
		/// gas, the delegation state would be consisent as no child would
		/// "survive" its parent. As a consequence, if the root node is revoked,
		/// the whole trust hierarchy is to be considered revoked.
		///
		/// * origin: the identifier of the revoker
		/// * root_id: the ID of the delegation root to revoke
		/// * max_children: the maximum number of nodes descending from the root
		///   to revoke as a consequence of the root revocation
		#[pallet::weight(<T as Config>::WeightInfo::revoke_hierarchy(*max_children))]
		pub fn revoke_hierarchy(
			origin: OriginFor<T>,
			root_node_id: DelegationNodeIdOf<T>,
			max_children: u32,
		) -> DispatchResultWithPostInfo {
			let invoker = <T as Config>::EnsureOrigin::ensure_origin(origin)?;

			let hierarchy_root_node = <DelegationNodes<T>>::get(&root_node_id).ok_or(Error::<T>::HierarchyNotFound)?;

			ensure!(
				hierarchy_root_node.details.owner == invoker,
				Error::<T>::UnauthorizedRevocation
			);

			ensure!(
				max_children <= T::MaxRevocations::get(),
				Error::<T>::MaxRevocationsTooLarge
			);

			let consumed_weight: Weight = if !hierarchy_root_node.details.revoked {
				// Recursively revoke all children
				let (_, post_weight) = Self::revoke_children(&root_node_id, &invoker, max_children)?;

				// If we didn't return an ExceededRevocationBounds error, we can revoke the root
				// too.
				Self::revoke_and_store_hierarchy_root(root_node_id, hierarchy_root_node);
				// We don't cancel the delegation_hierarchy from storage.

				post_weight.saturating_add(T::DbWeight::get().writes(1))
			} else {
				0
			};

			Self::deposit_event(Event::HierarchyRevoked(invoker, root_node_id));

			Ok(Some(consumed_weight.saturating_add(T::DbWeight::get().reads(1))).into())
		}

		/// Revoke a delegation node and all its children.
		///
		/// Revoking a delegation node results in the trust hierarchy starting
		/// from the given node being revoked. Nevertheless, revocation starts
		/// from the leave nodes upwards, so if the operation ends prematurely
		/// because it runs out of gas, the delegation state would be consisent
		/// as no child would "survive" its parent. As a consequence, if the
		/// given node is revoked, the trust hierarchy with the node as root is
		/// to be considered revoked.
		///
		/// * origin: the identifier of the revoker
		/// * delegation_id: the ID of the delegation root to revoke
		/// * max_parent_checks: in case the revoker is not the owner of the
		///   specified node, the number of parent nodes to check to verify that
		///   the revoker is authorised to perform the revokation. The
		///   evaluation terminates when a valid node is reached, when the whole
		///   hierarchy including the root node has been checked, or when the
		///   max number of parents is reached
		/// * max_revocations: the maximum number of nodes descending from this
		///   one to revoke as a consequence of this node revocation
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

			let (authorized, parent_checks) = Self::is_delegating(&invoker, &delegation_id, max_parent_checks)?;
			ensure!(authorized, Error::<T>::UnauthorizedRevocation);

			ensure!(
				max_revocations <= T::MaxRevocations::get(),
				Error::<T>::MaxRevocationsTooLarge
			);

			// Revoke the delegation and recursively all of its children
			let (revocation_checks, _) = Self::revoke(&delegation_id, &invoker, max_revocations)?;

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
	// Calculate the hash of all values of a delegation transaction
	fn calculate_delegation_hash_root(
		delegation_id: &DelegationNodeIdOf<T>,
		root_id: &DelegationNodeIdOf<T>,
		parent_id: &DelegationNodeIdOf<T>,
		permissions: &Permissions,
	) -> T::Hash {
		// Add all values to an u8 vector
		let mut hashed_values: Vec<u8> = delegation_id.as_ref().to_vec();
		hashed_values.extend_from_slice(root_id.as_ref());
		hashed_values.extend_from_slice(parent_id.as_ref());
		hashed_values.extend_from_slice(permissions.as_u8().as_ref());
		// Hash the resulting vector
		T::Hashing::hash(&hashed_values)
	}

	fn create_and_store_new_hierarchy(
		root_id: DelegationNodeIdOf<T>,
		hierarchy_info: DelegationHierarchyInfo<T>,
		hierarchy_owner: DelegatorIdOf<T>,
	) {
		let root_node = DelegationNode::new_root_node(root_id, DelegationDetails::default_with_owner(hierarchy_owner));
		<DelegationNodes<T>>::insert(root_id, root_node);
		<DelegationHierarchies<T>>::insert(root_id, hierarchy_info);
	}

	fn store_delegation_under_parent(
		delegation_id: DelegationNodeIdOf<T>,
		delegation_node: DelegationNode<T>,
		parent_id: DelegationNodeIdOf<T>,
		mut parent_node: DelegationNode<T>,
	) {
		<DelegationNodes<T>>::insert(delegation_id, delegation_node);
		// Add the new node as a child of that node
		parent_node.add_child(delegation_id);
		<DelegationNodes<T>>::insert(parent_id, parent_node);
	}

	fn revoke_and_store_hierarchy_root(root_id: DelegationNodeIdOf<T>, mut root_node: DelegationNode<T>) {
		root_node.details.revoked = true;
		<DelegationNodes<T>>::insert(root_id, root_node);
	}

	/// Check if an identity is the owner of the given delegation node or any
	/// node up the hierarchy, and if the delegation has not been yet revoked.
	///
	/// It checks whether the conditions are required for the given node,
	/// otherwise it goes up up to `max_parent_checks` nodes, including the root
	/// node, to check whether the given identity is a valid delegator of the
	/// given delegation.
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
				// Safe because remaining lookups is at most max_parent_checks
				Ok((false, max_parent_checks - remaining_lookups))
			}
		}
	}

	/// Revokes all children of a delegation.
	/// Returns the number of revoked delegations and the consumed weight.
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

	// Revoke a delegation and all of its children recursively.
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
