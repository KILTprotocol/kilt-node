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

#[cfg(test)]
mod tests;

#[cfg(any(feature = "mock", test))]
pub mod mock;

pub mod default_weights;
pub use default_weights::WeightInfo;

use codec::{Decode, Encode};
use delegation::Permissions;
use frame_support::{
	ensure,
	traits::{Hooks, IsType},
};
use frame_system::{self, ensure_signed};
use sp_std::{
	fmt::Debug,
	prelude::{Clone, PartialEq, Vec},
};

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	/// An on-chain attestation written by an authorised attester.
	#[derive(Clone, Debug, Encode, Decode, PartialEq)]
	pub struct Attestation<T: Config> {
		/// The hash of the CTYPE used for this attestation.
		pub ctype_hash: T::Hash,
		/// The DID of the attester.
		pub attester: T::DidIdentifier,
		/// \[OPTIONAL\] The ID of the delegation node used to authorize the
		/// attester.
		pub delegation_id: Option<T::DelegationNodeId>,
		/// The flag indicating whether the attestation has been revoked or not.
		pub revoked: bool,
	}

	/// An operation to create a new attestation.
	///
	/// The struct implements the DidOperation trait, and as such it must
	/// contain information about the caller's DID, the type of DID key
	/// required to verify the operation signature, and the tx counter to
	/// protect against replay attacks.
	#[derive(Clone, Decode, Encode, PartialEq)]
	pub struct AttestationCreationOperation<T: Config> {
		/// The DID of the attester.
		pub attester_did: T::DidIdentifier,
		/// The hash of the claim to attest. It has to be unique.
		pub claim_hash: T::Hash,
		/// The hash of the CTYPE used for this attestation.
		pub ctype_hash: T::Hash,
		/// \[OPTIONAL\] The ID of the delegation node used to authorise the
		/// attester.
		pub delegation_id: Option<T::DelegationNodeId>,
		/// The DID tx counter.
		pub tx_counter: u64,
	}

	impl<T: Config> did::DidOperation<T> for AttestationCreationOperation<T> {
		fn get_verification_key_type(&self) -> did::DidVerificationKeyRelationship {
			did::DidVerificationKeyRelationship::AssertionMethod
		}

		fn get_did(&self) -> &T::DidIdentifier {
			&self.attester_did
		}

		fn get_tx_counter(&self) -> u64 {
			self.tx_counter
		}
	}

	// Required to use a struct as an extrinsic parameter, and since Config does not
	// implement Debug, the derive macro does not work.
	impl<T: Config> Debug for AttestationCreationOperation<T> {
		fn fmt(&self, f: &mut sp_std::fmt::Formatter<'_>) -> sp_std::fmt::Result {
			f.debug_tuple("AttestationCreationOperation")
				.field(&self.attester_did)
				.field(&self.claim_hash)
				.field(&self.ctype_hash)
				.field(&self.delegation_id)
				.field(&self.tx_counter)
				.finish()
		}
	}

	/// An operation to revoke an existing attestation.
	///
	/// The struct implements the DidOperation trait, and as such it must
	/// contain information about the caller's DID, the type of DID key
	/// required to verify the operation signature, and the tx counter to
	/// protect against replay attacks.
	#[derive(Clone, Decode, Encode, PartialEq)]
	pub struct AttestationRevocationOperation<T: Config> {
		/// The DID of the revoker.
		pub revoker_did: T::DidIdentifier,
		/// The hash of the claim to revoke.
		pub claim_hash: T::Hash,
		/// For delegated attestations, the number of nodes to check up in the
		/// trust hierarchy (including the root node but excluding the given
		/// node) to verify whether the caller is authorised to revoke the
		/// specified attestation.
		pub max_parent_checks: u32,
		/// The DID tx counter.
		pub tx_counter: u64,
	}

	impl<T: Config> did::DidOperation<T> for AttestationRevocationOperation<T> {
		fn get_verification_key_type(&self) -> did::DidVerificationKeyRelationship {
			did::DidVerificationKeyRelationship::AssertionMethod
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
	impl<T: Config> Debug for AttestationRevocationOperation<T> {
		fn fmt(&self, f: &mut sp_std::fmt::Formatter<'_>) -> sp_std::fmt::Result {
			f.debug_tuple("AttestationRevocationOperation")
				.field(&self.revoker_did)
				.field(&self.claim_hash)
				.field(&self.max_parent_checks)
				.field(&self.tx_counter)
				.finish()
		}
	}

	#[pallet::config]
	pub trait Config: frame_system::Config + did::Config + delegation::Config {
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
	pub type Attestations<T> = StorageMap<_, Blake2_128Concat, <T as frame_system::Config>::Hash, Attestation<T>>;

	/// Delegated attestations stored on chain.
	///
	/// It maps from a delegation ID to a vector of claim hashes.
	#[pallet::storage]
	#[pallet::getter(fn delegated_attestations)]
	pub type DelegatedAttestations<T> = StorageMap<
		_,
		Blake2_128Concat,
		<T as delegation::Config>::DelegationNodeId,
		Vec<<T as frame_system::Config>::Hash>,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new attestation has been created.
		/// \[attester DID, claim hash, CTYPE hash, delegation ID\]
		AttestationCreated(T::DidIdentifier, T::Hash, T::Hash, Option<T::DelegationNodeId>),
		/// An attestation has been revoked.
		/// \[revoker DID, claim hash\]
		AttestationRevoked(T::DidIdentifier, T::Hash),
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
		/// Submit a new AttestationCreationOperation operation.
		///
		/// * origin: the origin of the transaction
		/// * operation: the AttestationCreationOperation operation
		/// * signature: the signature over the byte-encoded operation
		#[pallet::weight(<T as Config>::WeightInfo::submit_attestation_creation_operation())]
		pub fn submit_attestation_creation_operation(
			origin: OriginFor<T>,
			operation: AttestationCreationOperation<T>,
			signature: did::DidSignature,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;

			// Check if DID exists, if counter is valid, if signature is valid, and increase
			// DID tx counter
			did::pallet::Pallet::verify_operation_validity_and_increase_did_nonce(&operation, &signature)
				.map_err(<did::Error<T>>::from)?;

			ensure!(
				<ctype::Ctypes<T>>::contains_key(&operation.ctype_hash),
				ctype::Error::<T>::CTypeNotFound
			);

			ensure!(
				!<Attestations<T>>::contains_key(&operation.claim_hash),
				Error::<T>::AlreadyAttested
			);

			// Check for validity of the delegation node if specified.
			if let Some(delegation_id) = operation.delegation_id {
				let delegation = <delegation::Delegations<T>>::get(delegation_id)
					.ok_or(delegation::Error::<T>::DelegationNotFound)?;

				ensure!(!delegation.revoked, Error::<T>::DelegationRevoked);

				ensure!(
					delegation.owner == operation.attester_did,
					Error::<T>::NotDelegatedToAttester
				);

				ensure!(
					(delegation.permissions & Permissions::ATTEST) == Permissions::ATTEST,
					Error::<T>::DelegationUnauthorizedToAttest
				);

				// Check if the CTYPE of the delegation is matching the CTYPE of the attestation
				let root =
					<delegation::Roots<T>>::get(delegation.root_id).ok_or(delegation::Error::<T>::RootNotFound)?;
				ensure!(root.ctype_hash == operation.ctype_hash, Error::<T>::CTypeMismatch);

				// If the attestation is based on a delegation, store separately
				let mut delegated_attestations = <DelegatedAttestations<T>>::get(delegation_id).unwrap_or_default();
				delegated_attestations.push(operation.claim_hash);
				<DelegatedAttestations<T>>::insert(delegation_id, delegated_attestations);
			}

			log::debug!("insert Attestation");
			<Attestations<T>>::insert(
				&operation.claim_hash,
				Attestation {
					ctype_hash: operation.ctype_hash,
					attester: operation.attester_did.clone(),
					delegation_id: operation.delegation_id,
					revoked: false,
				},
			);

			Self::deposit_event(Event::AttestationCreated(
				operation.attester_did,
				operation.claim_hash,
				operation.ctype_hash,
				operation.delegation_id,
			));

			Ok(None.into())
		}

		/// Submit a new AttestationRevocationOperation operation.
		///
		/// * origin: the origin of the transaction
		/// * operation: the AttestationRevocationOperation operation
		/// * signature: the signature over the byte-encoded operation
		#[pallet::weight(<T as Config>::WeightInfo::submit_attestation_revocation_operation(operation.max_parent_checks))]
		pub fn submit_attestation_revocation_operation(
			origin: OriginFor<T>,
			operation: AttestationRevocationOperation<T>,
			signature: did::DidSignature,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;

			// Check if DID exists, if counter is valid, if signature is valid, and increase
			// DID tx counter
			did::pallet::Pallet::verify_operation_validity_and_increase_did_nonce(&operation, &signature)
				.map_err(<did::Error<T>>::from)?;

			let attestation = <Attestations<T>>::get(&operation.claim_hash).ok_or(Error::<T>::AttestationNotFound)?;

			ensure!(!attestation.revoked, Error::<T>::AlreadyRevoked);

			// Check the delegation tree if the sender of the revocation operation is not
			// the original attester
			if attestation.attester != operation.revoker_did {
				let delegation_id = attestation.delegation_id.ok_or(Error::<T>::UnauthorizedRevocation)?;
				// Check whether the sender of the revocation controls the delegation node
				// specified, and that its status has not been revoked
				ensure!(
					<delegation::Pallet<T>>::is_delegating(
						&operation.revoker_did,
						&delegation_id,
						operation.max_parent_checks
					)?,
					Error::<T>::UnauthorizedRevocation
				);
			}

			log::debug!("revoking Attestation");
			<Attestations<T>>::insert(
				&operation.claim_hash,
				Attestation {
					revoked: true,
					..attestation
				},
			);

			Self::deposit_event(Event::AttestationRevoked(operation.revoker_did, operation.claim_hash));

			//TODO: Return actual weight used, which should be returned by
			// delegation::is_actively_delegating
			Ok(None.into())
		}
	}
}
