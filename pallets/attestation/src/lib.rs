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
use frame_support::{
	debug, decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
	StorageMap,
};
use frame_system::{self, ensure_signed};
use sp_std::prelude::{Clone, PartialEq, Vec};

/// The attestation trait
pub trait Config: frame_system::Config + delegation::Config {
	/// Attestation specific event type
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;

	/// Weight information for extrinsics in this pallet.
	type WeightInfo: WeightInfo;
}

decl_event!(
	/// Events for attestations
	pub enum Event<T> where <T as frame_system::Config>::AccountId, <T as frame_system::Config>::Hash,
			<T as delegation::Config>::DelegationNodeId {
		/// An attestation has been added
		AttestationCreated(AccountId, Hash, Hash, Option<DelegationNodeId>),
		/// An attestation has been revoked
		AttestationRevoked(AccountId, Hash),
	}
);

// The pallet's errors
decl_error! {
	pub enum Error for Module<T: Config> {
		AlreadyAttested,
		AlreadyRevoked,
		AttestationNotFound,
		CTypeMismatch,
		DelegationUnauthorizedToAttest,
		DelegationRevoked,
		NotDelegatedToAttester,
		UnauthorizedRevocation,
	}
}

decl_module! {
	/// The attestation runtime module
	pub struct Module<T: Config> for enum Call where origin: T::Origin {
		/// Deposit events
		fn deposit_event() = default;

		// Initializing errors
		// this includes information about your errors in the node's metadata.
		// it is needed only if you are using errors in your pallet
		type Error = Error<T>;

		/// Adds an attestation on chain, where
		/// origin - the origin of the transaction
		/// claim_hash - hash of the attested claim
		/// ctype_hash - hash of the CTYPE of the claim
		/// delegation_id - optional id that refers to a delegation this attestation is based on
		#[weight = <T as Config>::WeightInfo::add()]
		pub fn add(origin, claim_hash: T::Hash, ctype_hash: T::Hash, delegation_id: Option<T::DelegationNodeId>) -> DispatchResult {
			// origin of the transaction needs to be a signed sender account
			let sender = ensure_signed(origin)?;

			// check if the CTYPE exists
			ensure!(<ctype::CTYPEs<T>>::contains_key(ctype_hash), ctype::Error::<T>::NotFound);

			// check if attestation already exists
			ensure!(!<Attestations<T>>::contains_key(claim_hash), Error::<T>::AlreadyAttested);

			if let Some(d) = delegation_id {
				// check if delegation exists
				let delegation = <delegation::Delegations<T>>::get(d).ok_or(delegation::Error::<T>::DelegationNotFound)?;
				// check whether delegation has been revoked already
				ensure!(!delegation.revoked, Error::<T>::DelegationRevoked);

				// check whether the owner of the delegation is not the sender of this transaction
				ensure!(delegation.owner.eq(&sender), Error::<T>::NotDelegatedToAttester);

				// check whether the delegation is not set up for attesting claims
				ensure!((delegation.permissions & Permissions::ATTEST) == Permissions::ATTEST, Error::<T>::DelegationUnauthorizedToAttest);

				// check if CTYPE of the delegation is matching the CTYPE of the attestation
				let root = <delegation::Root<T>>::get(delegation.root_id).ok_or(delegation::Error::<T>::RootNotFound)?;
				ensure!(root.ctype_hash.eq(&ctype_hash), Error::<T>::CTypeMismatch);
			}

			// insert attestation
			debug::print!("insert Attestation");
			<Attestations<T>>::insert(claim_hash, Attestation {ctype_hash, attester: sender.clone(), delegation_id, revoked: false});

			if let Some(d) = delegation_id {
				// if attestation is based on a delegation, store separately
				let mut delegated_attestations = <DelegatedAttestations<T>>::get(d);
				delegated_attestations.push(claim_hash);
				<DelegatedAttestations<T>>::insert(d, delegated_attestations);
			}

			// deposit event that attestation has beed added
			Self::deposit_event(RawEvent::AttestationCreated(sender, claim_hash, ctype_hash, delegation_id));
			Ok(())
		}

		/// Revokes an attestation on chain, where
		/// origin - the origin of the transaction
		/// claim_hash - hash of the attested claim
		/// max_depth - max number of parent checks of the delegation node supported in this call until finding the owner
		#[weight = <T as Config>::WeightInfo::revoke(*max_depth)]
		pub fn revoke(origin, claim_hash: T::Hash, max_depth: u32) -> DispatchResult {
			// origin of the transaction needs to be a signed sender account
			let sender = ensure_signed(origin)?;

			// lookup attestation & check if the attestation exists
			let Attestation {ctype_hash, attester, delegation_id, revoked, ..} = <Attestations<T>>::get(claim_hash).ok_or(Error::<T>::AttestationNotFound)?;

			// check if the attestation has already been revoked
			ensure!(!revoked, Error::<T>::AlreadyRevoked);

			// check delegation tree if the sender of the revocation transaction is not the attester
			if !attester.eq(&sender) {
				// check whether the attestation includes a delegation
				let del_id = delegation_id.ok_or(Error::<T>::UnauthorizedRevocation)?;
				// check whether the sender of the revocation is not a parent in the delegation hierarchy
				ensure!(<delegation::Module<T>>::is_delegating(&sender, &del_id, max_depth)?, Error::<T>::UnauthorizedRevocation);
			}

			debug::print!("revoking Attestation");
			<Attestations<T>>::insert(claim_hash, Attestation {
				ctype_hash,
				attester,
				delegation_id,
				revoked: true
			});
			Self::deposit_event(RawEvent::AttestationRevoked(sender, claim_hash));

			Ok(())
		}
	}
}

#[derive(Debug, Encode, Decode, PartialEq)]
pub struct Attestation<T: Config> {
	// hash of the CTYPE used for this attestation
	ctype_hash: T::Hash,
	// the account which executed the attestation
	attester: T::AccountId,
	// id of the delegation node (if existent)
	delegation_id: Option<T::DelegationNodeId>,
	// revocation status
	revoked: bool,
}

decl_storage! {
	trait Store for Module<T: Config> as Attestation {
		/// Attestations: claim-hash -> (ctype-hash, attester-account, delegation-id?, revoked)?
		Attestations get(fn attestations): map hasher(opaque_blake2_256) T::Hash => Option<Attestation<T>>;
		/// DelegatedAttestations: delegation-id -> [claim-hash]
		DelegatedAttestations get(fn delegated_attestations): map hasher(opaque_blake2_256) T::DelegationNodeId => Vec<T::Hash>;
	}
}
