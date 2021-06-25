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

//! Attestation: Handles attestations on chain,
//! adding and revoking attestations.
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

use sp_std::vec::Vec;

pub use crate::{attestations::*, default_weights::WeightInfo, pallet::*};

use frame_support::traits::Get;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	/// Type of a claim hash.
	pub type ClaimHashOf<T> = <T as frame_system::Config>::Hash;

	/// Type of an attestation CTYPE hash.
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
	pub type DelegatedAttestations<T> = StorageMap<_, Blake2_128Concat, DelegationNodeIdOf<T>, Vec<ClaimHashOf<T>>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new attestation has been created.
		/// \[attester ID, claim hash, CTYPE hash, (optional) delegation ID\]
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
		/// The attestation CTYPE does not match the CTYPE specified in the
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
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Create a new attestation.
		///
		/// The attester can optionally provide a reference to an existing
		/// delegation that will be saved along with the attestation itself in
		/// the form of an attested delegation.
		///
		/// * origin: the identifier of the attester
		/// * claim_hash: the hash of the claim to attest. It has to be unique
		/// * ctype_hash: the hash of the CTYPE used for this attestation
		/// * delegation_id: \[OPTIONAL\] the ID of the delegation node used to
		///   authorise the attester
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
				let delegation = <delegation::Delegations<T>>::get(delegation_id)
					.ok_or(delegation::Error::<T>::DelegationNotFound)?;

				ensure!(!delegation.revoked, Error::<T>::DelegationRevoked);

				ensure!(delegation.owner == attester, Error::<T>::NotDelegatedToAttester);

				ensure!(
					(delegation.permissions & delegation::Permissions::ATTEST) == delegation::Permissions::ATTEST,
					Error::<T>::DelegationUnauthorizedToAttest
				);

				// Check if the CTYPE of the delegation is matching the CTYPE of the attestation
				let root =
					<delegation::Roots<T>>::get(delegation.root_id).ok_or(delegation::Error::<T>::RootNotFound)?;
				ensure!(root.ctype_hash == ctype_hash, Error::<T>::CTypeMismatch);

				// If the attestation is based on a delegation, store separately
				let mut delegated_attestations = <DelegatedAttestations<T>>::get(delegation_id).unwrap_or_default();
				delegated_attestations.push(claim_hash);
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
		/// * origin: the identifier of the revoker
		/// * claim_hash: the hash of the claim to revoke
		/// * max_parent_checks: for delegated attestations, the number of
		///   delegation nodes to check up in the trust hierarchy (including the
		///   root node but excluding the provided node) to verify whether the
		///   caller is an ancestor of the attestation attester and hence
		///   authorised to revoke the specified attestation.
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
