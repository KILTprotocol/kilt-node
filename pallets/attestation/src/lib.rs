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

//! # Attestation Pallet
//!
//! Provides means of adding KILT attestations on chain and revoking them.
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
//! - `add` - Create a new attestation for a given claim which is based on a
//!   CType. The attester can optionally provide a reference to an existing
//!   delegation that will be saved along with the attestation itself in the
//!   form of an attested delegation.
//! - `revoke` - Revoke an existing attestation for a given claim. The revoker
//!   must be either the creator of the attestation being revoked or an entity
//!   that in the delegation tree is an ancestor of the attester, i.e., it was
//!   either the delegator of the attester or an ancestor thereof.
//!
//! ## Assumptions
//!
//! - The claim which shall be attested is based on a CType and signed by the
//!   claimer.
//! - The Verifier trusts the Attester. Otherwise, the attestation is worthless
//!   for the Verifier

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

pub mod attestations;
pub mod default_weights;

#[cfg(any(feature = "mock", test))]
pub mod mock;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

#[cfg(test)]
mod tests;

pub use crate::{attestations::*, default_weights::WeightInfo, pallet::*};

use frame_support::traits::Get;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{pallet_prelude::*, BoundedVec};
	use frame_system::pallet_prelude::*;

	/// Type of a claim hash.
	pub type ClaimHashOf<T> = <T as frame_system::Config>::Hash;

	/// Type of an attestation CType hash.
	pub type CtypeHashOf<T> = ctype::CtypeHashOf<T>;

	/// Type of an attester identifier.
	pub type AttesterOf<T> = delegation::DelegatorIdOf<T>;

	/// Type of a delegation identifier.
	pub type DelegationNodeIdOf<T> = delegation::DelegationNodeIdOf<T>;

	#[pallet::config]
	pub trait Config: frame_system::Config + ctype::Config + delegation::Config {
		type EnsureOrigin: EnsureOrigin<Success = AttesterOf<Self>, <Self as frame_system::Config>::Origin>;
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type WeightInfo: WeightInfo;

		/// The maximum number of delegated attestations which can be made by
		/// the same delegation.
		#[pallet::constant]
		type MaxDelegatedAttestations: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	/// Attestations stored on chain.
	///
	/// It maps from a claim hash to the full attestation.
	#[pallet::storage]
	#[pallet::getter(fn attestations)]
	pub type Attestations<T> = StorageMap<_, Blake2_128Concat, ClaimHashOf<T>, AttestationDetails<T>>;

	/// Delegated attestations stored on chain.
	///
	/// It maps from a delegation ID to a vector of claim hashes.
	#[pallet::storage]
	#[pallet::getter(fn delegated_attestations)]
	pub type DelegatedAttestations<T> = StorageMap<
		_,
		Blake2_128Concat,
		DelegationNodeIdOf<T>,
		BoundedVec<ClaimHashOf<T>, <T as Config>::MaxDelegatedAttestations>,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new attestation has been created.
		/// \[attester ID, claim hash, CType hash, (optional) delegation ID\]
		AttestationCreated(
			AttesterOf<T>,
			ClaimHashOf<T>,
			CtypeHashOf<T>,
			Option<DelegationNodeIdOf<T>>,
		),
		/// An attestation has been revoked.
		/// \[revoker ID, claim hash\]
		AttestationRevoked(AttesterOf<T>, ClaimHashOf<T>),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// There is already an attestation with the same claim hash stored on
		/// chain.
		AlreadyAttested,
		/// The attestation has already been revoked.
		AlreadyRevoked,
		/// No attestation on chain matching the claim hash.
		AttestationNotFound,
		/// The attestation CType does not match the CType specified in the
		/// delegation hierarchy root.
		CTypeMismatch,
		/// The delegation node does not include the permission to create new
		/// attestations. Only when the revoker is not the original attester.
		DelegationUnauthorizedToAttest,
		/// The delegation node has already been revoked.
		/// Only when the revoker is not the original attester.
		DelegationRevoked,
		/// The delegation node owner is different than the attester.
		/// Only when the revoker is not the original attester.
		NotDelegatedToAttester,
		/// The delegation node is not under the control of the revoker, or it
		/// is but it has been revoked. Only when the revoker is not the
		/// original attester.
		UnauthorizedRevocation,
		/// The maximum number of delegated attestations has already been
		/// reached for the corresponding delegation id such that another one
		/// cannot be added.
		MaxDelegatedAttestationsExceeded,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new attestation.
		///
		/// The attester can optionally provide a reference to an existing
		/// delegation that will be saved along with the attestation itself in
		/// the form of an attested delegation.
		///
		/// The referenced CType hash must already be present on chain.
		///
		/// If an optional delegation id is provided, the dispatch origin must
		/// be the owner of the delegation. Otherwise, it could be any
		/// `DelegationEntityId`.
		///
		/// Emits `AttestationCreated`.
		///
		/// # <weight>
		/// Weight: O(1)
		/// - Reads: [Origin Account], Ctype, Attestations
		/// - Reads if delegation id is provided: Delegations, Roots,
		///   DelegatedAttestations
		/// - Writes: Attestations, (DelegatedAttestations)
		/// # </weight>
		#[pallet::weight(<T as pallet::Config>::WeightInfo::add())]
		pub fn add(
			origin: OriginFor<T>,
			claim_hash: ClaimHashOf<T>,
			ctype_hash: CtypeHashOf<T>,
			delegation_id: Option<DelegationNodeIdOf<T>>,
		) -> DispatchResult {
			let attester = <T as Config>::EnsureOrigin::ensure_origin(origin)?;

			ensure!(
				<ctype::Ctypes<T>>::contains_key(&ctype_hash),
				ctype::Error::<T>::CTypeNotFound
			);

			ensure!(
				!<Attestations<T>>::contains_key(&claim_hash),
				Error::<T>::AlreadyAttested
			);

			// Check for validity of the delegation node if specified.
			if let Some(delegation_id) = delegation_id {
				let delegation = <delegation::DelegationNodes<T>>::get(delegation_id)
					.ok_or(delegation::Error::<T>::DelegationNotFound)?;

				ensure!(!delegation.details.revoked, Error::<T>::DelegationRevoked);

				ensure!(delegation.details.owner == attester, Error::<T>::NotDelegatedToAttester);

				ensure!(
					(delegation.details.permissions & delegation::Permissions::ATTEST)
						== delegation::Permissions::ATTEST,
					Error::<T>::DelegationUnauthorizedToAttest
				);

				// Check if the CType of the delegation is matching the CType of the attestation
				let root = <delegation::DelegationHierarchies<T>>::get(delegation.hierarchy_root_id)
					.ok_or(delegation::Error::<T>::HierarchyNotFound)?;
				ensure!(root.ctype_hash == ctype_hash, Error::<T>::CTypeMismatch);

				// If the attestation is based on a delegation, store separately
				let mut delegated_attestations = <DelegatedAttestations<T>>::get(delegation_id).unwrap_or_default();
				delegated_attestations
					.try_push(claim_hash)
					.map_err(|_| Error::<T>::MaxDelegatedAttestationsExceeded)?;
				<DelegatedAttestations<T>>::insert(delegation_id, delegated_attestations);
			}

			log::debug!("insert Attestation");
			<Attestations<T>>::insert(
				&claim_hash,
				AttestationDetails {
					ctype_hash,
					attester: attester.clone(),
					delegation_id,
					revoked: false,
				},
			);

			Self::deposit_event(Event::AttestationCreated(
				attester,
				claim_hash,
				ctype_hash,
				delegation_id,
			));

			Ok(())
		}

		/// Revoke an existing attestation.
		///
		/// The revoker must be either the creator of the attestation being
		/// revoked or an entity that in the delegation tree is an ancestor of
		/// the attester, i.e., it was either the delegator of the attester or
		/// an ancestor thereof.
		///
		/// Emits `AttestationRevoked`.
		///
		/// # <weight>
		/// Weight: O(P) where P is the number of steps required to verify that
		/// the dispatch Origin controls the delegation entitled to revoke the
		/// attestation. It is bounded by `max_parent_checks`.
		/// - Reads: [Origin Account], Attestations, delegation::Roots
		/// - Reads per delegation step P: delegation::Delegations
		/// - Writes: Attestations, DelegatedAttestations
		/// # </weight>
		#[pallet::weight(<T as pallet::Config>::WeightInfo::revoke(*max_parent_checks))]
		pub fn revoke(
			origin: OriginFor<T>,
			claim_hash: ClaimHashOf<T>,
			max_parent_checks: u32,
		) -> DispatchResultWithPostInfo {
			let revoker = <T as Config>::EnsureOrigin::ensure_origin(origin)?;

			let attestation = <Attestations<T>>::get(&claim_hash).ok_or(Error::<T>::AttestationNotFound)?;

			ensure!(!attestation.revoked, Error::<T>::AlreadyRevoked);

			// Check the delegation tree if the sender of the revocation operation is not
			// the original attester
			let revocations = if attestation.attester != revoker {
				let delegation_id = attestation.delegation_id.ok_or(Error::<T>::UnauthorizedRevocation)?;
				ensure!(
					max_parent_checks <= T::MaxParentChecks::get(),
					delegation::Error::<T>::MaxParentChecksTooLarge
				);
				// Check whether the sender of the revocation controls the delegation node
				// specified, and that its status has not been revoked
				let (is_delegating, revocations) =
					<delegation::Pallet<T>>::is_delegating(&revoker, &delegation_id, max_parent_checks)?;
				ensure!(is_delegating, Error::<T>::UnauthorizedRevocation);
				revocations
			} else {
				0u32
			};

			log::debug!("revoking Attestation");
			<Attestations<T>>::insert(
				&claim_hash,
				AttestationDetails {
					revoked: true,
					..attestation
				},
			);

			Self::deposit_event(Event::AttestationRevoked(revoker, claim_hash));

			Ok(Some(<T as pallet::Config>::WeightInfo::revoke(revocations)).into())
		}
	}
}
