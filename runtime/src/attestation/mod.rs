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

/// Test module for attestations
#[cfg(test)]
mod tests;

use super::{ctype, delegation, error};
use rstd::{
	prelude::{Clone, PartialEq, Vec},
	result,
};
use support::{decl_event, decl_module, decl_storage, dispatch::Result, StorageMap};
use system::{self, ensure_signed};

/// The attestation trait
pub trait Trait: system::Trait + delegation::Trait + error::Trait {
	/// Attestation specific event type
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_event!(
	/// Events for attestations
	pub enum Event<T> where <T as system::Trait>::AccountId, <T as system::Trait>::Hash,
			<T as delegation::Trait>::DelegationNodeId {
		/// An attestation has been added
		AttestationCreated(AccountId, Hash, Hash, Option<DelegationNodeId>),
		/// An attestation has been revoked
		AttestationRevoked(AccountId, Hash),
	}
);

decl_module! {
	/// The attestation runtime module
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		/// Deposit events
		fn deposit_event<T>() = default;

		/// Adds an attestation on chain, where
		/// origin - the origin of the transaction
		/// claim_hash - hash of the attested claim
		/// ctype_hash - hash of the CTYPE of the claim
		/// delegation_id - optional id that refers to a delegation this attestation is based on
		pub fn add(origin, claim_hash: T::Hash, ctype_hash: T::Hash, delegation_id: Option<T::DelegationNodeId>) -> Result {
			// origin of the transaction needs to be a signed sender account
			let sender = ensure_signed(origin)?;
			// check if the CTYPE exists
			if !<ctype::CTYPEs<T>>::exists(ctype_hash) {
				return Self::error(<ctype::Module<T>>::ERROR_CTYPE_NOT_FOUND);
			}

			match delegation_id {
				// has a delegation
				Some(d) => {
					// check if delegation exists
					let delegation = <error::Module<T>>::ok_or_deposit_err(
						<delegation::Delegations<T>>::get(d.clone()),
						<delegation::Module<T>>::ERROR_DELEGATION_NOT_FOUND
					)?;
					if delegation.4 {
						// delegation has been revoked
						return Self::error(Self::ERROR_DELEGATION_REVOKED);
					} else if !delegation.2.eq(&sender) {
						// delegation is not made up for the sender of this transaction
						return Self::error(Self::ERROR_NOT_DELEGATED_TO_ATTESTER);
					} else if (delegation.3 & delegation::Permissions::ATTEST) != delegation::Permissions::ATTEST {
						// delegation is not set up for attesting claims
						return Self::error(Self::ERROR_DELEGATION_NOT_AUTHORIZED_TO_ATTEST);
					} else {
						// check if CTYPE of the delegation is matching the CTYPE of the attestation
						let root = <error::Module<T>>::ok_or_deposit_err(
							<delegation::Root<T>>::get(delegation.0.clone()),
							<delegation::Module<T>>::ERROR_ROOT_NOT_FOUND
						)?;
						if !root.0.eq(&ctype_hash) {
							return Self::error(Self::ERROR_CTYPE_OF_DELEGATION_NOT_MATCHING);
						}
					}
				},
				None => {}
			}

			// check if attestation already exists
			if <Attestations<T>>::exists(claim_hash.clone()) {
				return Self::error(Self::ERROR_ALREADY_ATTESTED);
			}

			// insert attestation
			::runtime_io::print("insert Attestation");
			<Attestations<T>>::insert(claim_hash.clone(), (ctype_hash.clone(), sender.clone(), delegation_id.clone(), false));

			match delegation_id {
				Some(d) => {
					// if attestation is based on a delegation this is stored in a separate map
					let mut delegated_attestations = <DelegatedAttestations<T>>::get(d);
					delegated_attestations.push(claim_hash.clone());
					<DelegatedAttestations<T>>::insert(d.clone(), delegated_attestations);
				},
				None => {}
			}

			// deposit event that attestation has beed added
			Self::deposit_event(RawEvent::AttestationCreated(sender.clone(), claim_hash.clone(),
					ctype_hash.clone(), delegation_id.clone()));
			Ok(())
		}

		/// Revokes an attestation on chain, where
		/// origin - the origin of the transaction
		/// claim_hash - hash of the attested claim
		pub fn revoke(origin, claim_hash: T::Hash) -> Result {
			// origin of the transaction needs to be a signed sender account
			let sender = ensure_signed(origin)?;

			// lookup attestation & check if the attestation exists
			let mut existing_attestation = <error::Module<T>>::ok_or_deposit_err(
				<Attestations<T>>::get(claim_hash.clone()),
				Self::ERROR_ATTESTATION_NOT_FOUND
			)?;
			// if the sender of the revocation transaction is not the attester, check delegation tree
			if !existing_attestation.1.eq(&sender) {
				match existing_attestation.2 {
					Some(d) => {
						if !Self::is_delegating(&sender, &d)? {
							// the sender of the revocation is not a parent in the delegation hierarchy
							return Self::error(Self::ERROR_NOT_PERMITTED_TO_REVOKE_ATTESTATION);
						}
					},
					None => {
						// the attestation has not been made up based on a delegation
						return Self::error(Self::ERROR_NOT_PERMITTED_TO_REVOKE_ATTESTATION);
					}
				}
			}

			// check if already revoked
			if existing_attestation.3 {
				return Self::error(Self::ERROR_ALREADY_REVOKED);
			}

			// revoke attestation
			::runtime_io::print("revoking Attestation");
			existing_attestation.3 = true;
			<Attestations<T>>::insert(claim_hash.clone(), existing_attestation.clone());

			// deposit event that the attestation has been revoked
			Self::deposit_event(RawEvent::AttestationRevoked(sender.clone(), claim_hash.clone()));
			Ok(())
		}
	}
}

/// Implementation of further module constants and functions for attestations
impl<T: Trait> Module<T> {
	/// Error types for errors in attestation module
	const ERROR_BASE: u16 = 2000;
	const ERROR_ALREADY_ATTESTED: error::ErrorType = (Self::ERROR_BASE + 1, "already attested");
	const ERROR_ALREADY_REVOKED: error::ErrorType = (Self::ERROR_BASE + 2, "already revoked");
	const ERROR_ATTESTATION_NOT_FOUND: error::ErrorType = (Self::ERROR_BASE + 3, "attestation not found");
	const ERROR_DELEGATION_REVOKED: error::ErrorType = (Self::ERROR_BASE + 4, "delegation revoked");
	const ERROR_NOT_DELEGATED_TO_ATTESTER: error::ErrorType = (Self::ERROR_BASE + 5, "not delegated to attester");
	const ERROR_DELEGATION_NOT_AUTHORIZED_TO_ATTEST: error::ErrorType = (Self::ERROR_BASE + 6, "delegation not authorized to attest");
	const ERROR_CTYPE_OF_DELEGATION_NOT_MATCHING: error::ErrorType = (Self::ERROR_BASE + 7, "CTYPE of delegation does not match");
	const ERROR_NOT_PERMITTED_TO_REVOKE_ATTESTATION: error::ErrorType = (Self::ERROR_BASE + 8, "not permitted to revoke attestation");

	/// Create an error using the error module
	pub fn error(error_type: error::ErrorType) -> Result {
		return <error::Module<T>>::error(error_type);
	}

	/// Check delegation hierarchy using the delegation module
	fn is_delegating(
		account: &T::AccountId,
		delegation: &T::DelegationNodeId,
	) -> result::Result<bool, &'static str> {
		<delegation::Module<T>>::is_delegating(account, delegation)
	}
}

decl_storage! {
	trait Store for Module<T: Trait> as Attestation {
		/// Attestations: claim-hash -> (ctype-hash, account, delegation-id?, revoked)?
		Attestations get(attestations): map T::Hash => Option<(T::Hash,T::AccountId,Option<T::DelegationNodeId>,bool)>;
		/// DelegatedAttestations: delegation-id -> [claim-hash]
		DelegatedAttestations get(delegated_attestations): map T::DelegationNodeId => Vec<T::Hash>;
	}
}
