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

#[cfg(any(feature = "runtime-benchmarks", test))]
pub mod benchmarking;
/// Test module for attestations
#[cfg(test)]
mod tests;

pub mod migration;

pub mod default_weights;
pub use default_weights::WeightInfo;

use codec::{Decode, Encode};
use delegation::Permissions;
use frame_support::ensure;
use frame_system::{self, ensure_signed};
use sp_std::prelude::{Clone, PartialEq, Vec};
use sp_std::fmt::Debug;

pub use pallet::*;

#[derive(Debug, Encode, Decode, PartialEq)]
pub struct Attestation<T: Config> {
	// Hash of the CTYPE used for this attestation
	ctype_hash: T::Hash,
	// The DID of the attestation creator
	attester: T::DidIdentifier,
	// ID of the delegation node (if existent)
	delegation_id: Option<T::DelegationNodeId>,
	// Revocation status
	revoked: bool,
}

/// An operation to create a new attestation.
/// The struct implements the DidOperation trait, and as such it must contain
/// information about the creator's DID, the type of DID key required to
/// verify the operation signature, and the tx counter to protect against replay
/// attacks. The struct has the following fields:
/// * creator_did: the DID of the attestation creator
/// * claim_hash: the hash of the attested claim
/// * ctype_hash: the hash of the attested claim CTYPE
/// * delegation_id: optional ID that refers to a delegation this attestation is based on
/// * tx_counter: the DID tx counter to mitigate replay attacks
#[derive(Clone, Decode, Encode, PartialEq)]
pub struct AttestationCreationOperation<T: Config> {
	caller_did: T::DidIdentifier,
	claim_hash: T::Hash,
	ctype_hash: T::Hash,
	delegation_id: Option<T::DelegationNodeId>,
	tx_counter: u64,
}

impl<T: Config> did::DidOperation<T> for AttestationCreationOperation<T> {
	fn get_verification_key_type(&self) -> did::DidVerificationKeyType {
		did::DidVerificationKeyType::AssertionMethod
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
impl<T: Config> Debug for AttestationCreationOperation<T> {
	fn fmt(&self, f: &mut sp_std::fmt::Formatter<'_>) -> sp_std::fmt::Result {
		f.debug_tuple("AttestationCreationOperation")
			.field(&self.caller_did)
			.field(&self.claim_hash)
			.field(&self.ctype_hash)
			.field(&self.delegation_id)
			.field(&self.tx_counter)
			.finish()
	}
}

/// An operation to revoke an existing attestation.
/// The struct implements the DidOperation trait, and as such it must contain
/// information about the creator's DID, the type of DID key required to
/// verify the operation signature, and the tx counter to protect against replay
/// attacks. The struct has the following fields:
/// * caller_did: the DID of the attestation creator
/// * claim_hash: the hash of the claim to revoke
/// * max_depth: max number of parent checks of the delegation node supported in this call to verify that the caller of this operation is allowed to revoke the specified node
/// * tx_counter: the DID tx counter to mitigate replay attacks
#[derive(Clone, Decode, Encode, PartialEq)]
pub struct AttestationRevocationOperation<T: Config> {
	caller_did: T::DidIdentifier,
	claim_hash: T::Hash,
	max_depth: u32,
	tx_counter: u64,
}

impl<T: Config> did::DidOperation<T> for AttestationRevocationOperation<T> {
	fn get_verification_key_type(&self) -> did::DidVerificationKeyType {
		did::DidVerificationKeyType::AssertionMethod
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
impl<T: Config> Debug for AttestationRevocationOperation<T> {
	fn fmt(&self, f: &mut sp_std::fmt::Formatter<'_>) -> sp_std::fmt::Result {
		f.debug_tuple("AttestationRevocationOperation")
			.field(&self.caller_did)
			.field(&self.claim_hash)
			.field(&self.max_depth)
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
	pub trait Config: frame_system::Config + did::Config + delegation::Config {
		/// Attestation specific event type
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	/// Attestations: claim-hash -> (ctype-hash, attester-account, delegation-id?, revoked)?
	#[pallet::storage]
	#[pallet::getter(fn attestations)]
	pub type Attestations<T> =
		StorageMap<_, Blake2_128Concat, <T as frame_system::Config>::Hash, Attestation<T>>;

	/// DelegatedAttestations: delegation-id -> [claim-hash]
	#[pallet::storage]
	#[pallet::getter(fn delegated_attestations)]
	pub type DelegatedAttestations<T> =
		StorageMap<_, Blake2_128Concat, <T as delegation::Config>::DelegationNodeId, Vec<<T as frame_system::Config>::Hash>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// An attestation has been added
		AttestationCreated(T::DidIdentifier, T::Hash, T::Hash, Option<T::DelegationNodeId>),
		/// An attestation has been revoked
		AttestationRevoked(T::DidIdentifier, T::Hash),
	}

	#[pallet::error]
	pub enum Error<T> {
		AlreadyAttested,
		AlreadyRevoked,
		AttestationNotFound,
		CTypeMismatch,
		DelegationUnauthorizedToAttest,
		DelegationRevoked,
		NotDelegatedToAttester,
		UnauthorizedRevocation,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(<T as Config>::WeightInfo::add())]
		pub fn submit_attestation_creation_operation(
			origin: OriginFor<T>,
			operation: AttestationCreationOperation<T>,
			signature: did::DidSignature
		) -> DispatchResultWithPostInfo {
			// Origin of the transaction needs to be a signed sender account
			ensure_signed(origin)?;

			let mut did_details = <did::Did<T>>::get(&operation.caller_did).ok_or(<did::Error<T>>::DidNotPresent)?;

			// Verify both tx counter and signature validity
			did::pallet::Pallet::verify_operation_validity_for_did(&operation, &signature, &did_details)
				.map_err(<did::Error<T>>::from)?;

			// Update tx counter in DID details and save to DID pallet
			did_details
				.increase_tx_counter()
				.expect("Increasing DID tx counter should be a safe operation.");
			<did::Did<T>>::insert(&operation.caller_did, did_details);

			// Check if the CTYPE exists
			ensure!(<ctype::Ctypes<T>>::contains_key(&operation.ctype_hash), ctype::Error::<T>::CTypeNotFound);

			// Check if attestation already exists
			ensure!(!<Attestations<T>>::contains_key(&operation.claim_hash), Error::<T>::AlreadyAttested);

			if let Some(delegation_id) = operation.delegation_id {
				// Check if delegation exists
				let delegation = <delegation::Delegations<T>>::get(delegation_id).ok_or(delegation::Error::<T>::DelegationNotFound)?;
				// Check whether delegation has been revoked already
				ensure!(!delegation.revoked, Error::<T>::DelegationRevoked);

				// Check whether the owner of the delegation is the sender of this transaction
				ensure!(delegation.owner.eq(&operation.caller_did), Error::<T>::NotDelegatedToAttester);

				// Check whether the delegation is not set up for attesting claims
				ensure!((delegation.permissions & Permissions::ATTEST) == Permissions::ATTEST, Error::<T>::DelegationUnauthorizedToAttest);

				// Check if CTYPE of the delegation is matching the CTYPE of the attestation
				let root = <delegation::Roots<T>>::get(delegation.root_id).ok_or(delegation::Error::<T>::RootNotFound)?;
				ensure!(root.ctype_hash.eq(&operation.ctype_hash), Error::<T>::CTypeMismatch);
			}

			// Insert attestation
			log::debug!("insert Attestation");
			<Attestations<T>>::insert(&operation.claim_hash, Attestation {ctype_hash: operation.ctype_hash, attester: operation.caller_did.clone(), delegation_id: operation.delegation_id, revoked: false});

			if let Some(delegation_id) = operation.delegation_id {
				// If attestation is based on a delegation, store separately
				let mut delegated_attestations = <DelegatedAttestations<T>>::get(delegation_id).unwrap_or_default();
				delegated_attestations.push(operation.claim_hash);
				<DelegatedAttestations<T>>::insert(delegation_id, delegated_attestations);
			}

			// Deposit event that attestation has beed added
			Self::deposit_event(Event::AttestationCreated(operation.caller_did, operation.claim_hash, operation.ctype_hash, operation.delegation_id));
			Ok(().into())
		}

		#[pallet::weight(<T as Config>::WeightInfo::revoke(operation.max_depth))]
		pub fn submit_attestation_revocation_operation(
			origin: OriginFor<T>,
			operation: AttestationRevocationOperation<T>,
			signature: did::DidSignature
		) -> DispatchResultWithPostInfo {
			// Origin of the transaction needs to be a signed sender account
			ensure_signed(origin)?;

			let mut did_details = <did::Did<T>>::get(&operation.caller_did).ok_or(<did::Error<T>>::DidNotPresent)?;

			// Verify both tx counter and signature validity
			did::pallet::Pallet::verify_operation_validity_for_did(&operation, &signature, &did_details)
				.map_err(<did::Error<T>>::from)?;

			// Update tx counter in DID details and save to DID pallet
			did_details
				.increase_tx_counter()
				.expect("Increasing DID tx counter should be a safe operation.");
			<did::Did<T>>::insert(&operation.caller_did, did_details);

			// Lookup attestation & check if the attestation exists
			let attestation = <Attestations<T>>::get(&operation.claim_hash).ok_or(Error::<T>::AttestationNotFound)?;

			// Check if the attestation has already been revoked
			ensure!(!attestation.revoked, Error::<T>::AlreadyRevoked);

			// Check delegation tree if the sender of the revocation transaction is not the attester
			if !attestation.attester.eq(&operation.caller_did) {
				// Check whether the attestation includes a delegation
				let delegation_id = attestation.delegation_id.ok_or(Error::<T>::UnauthorizedRevocation)?;
				// Check whether the sender of the revocation is not a parent in the delegation hierarchy
				ensure!(<delegation::Pallet<T>>::is_delegating(&operation.caller_did, &delegation_id, operation.max_depth)?, Error::<T>::UnauthorizedRevocation);
			}

			log::debug!("revoking Attestation");
			<Attestations<T>>::insert(&operation.claim_hash, Attestation {
				revoked: true,
				..attestation
			});

			Self::deposit_event(Event::AttestationRevoked(operation.caller_did, operation.claim_hash));

			Ok(().into())
		}
	}
}
