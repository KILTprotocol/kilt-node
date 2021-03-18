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

//! DID: Handles decentralized identifiers on chain,
//! adding and removing DIDs.
#![cfg_attr(not(feature = "std"), no_std)]

/// Test module for attestations
#[cfg(test)]
mod tests;

#[cfg(any(feature = "runtime-benchmarks", test))]
pub mod benchmarking;

pub mod default_weights;
pub mod migration;

pub use default_weights::WeightInfo;

use frame_support::{codec::{Encode, Decode}, Parameter, StorageMap, decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure};
use frame_system::{self, ensure_signed};
use sp_runtime::traits::Verify;
use sp_std::{prelude::*, convert::TryFrom, collections::btree_set::BTreeSet};
use sp_core::{ed25519, sr25519};
use x25519_dalek;

/// The DID trait
pub trait Config: frame_system::Config {
	/// DID specific event type
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;

	/// Weight information for extrinsics in this pallet.
	type WeightInfo: WeightInfo;
}

pub trait DIDPublicKey: Encode {
	fn get_did_key_description(&self) -> &'static str;
}

pub trait DIDPublicSigningKey: DIDPublicKey {
	fn verify_signature(&self, payload: &Payload, signature: &Signature) -> SignatureVerificationResult;
	fn get_expected_signature_size(&self) -> usize;		// Number of bytes for valid signatures
}

pub type Payload = Vec<u8>;
pub type Signature = Vec<u8>;
pub type KeyValue = Vec<u8>;
pub type DIDIdentifier = Vec<u8>;
pub type KeyIdentifier = Vec<u8>;
pub type URL = Vec<u8>;
pub type SignatureVerificationResult = Result<bool, SigningKeyError>;

// Ed25519 key
impl DIDPublicKey for ed25519::Public {
    fn get_did_key_description(&self) -> &'static str {
        "Ed25519VerificationKey2018"				// From https://w3c.github.io/did-spec-registries/#ed25519verificationkey2018
    }
}

impl DIDPublicSigningKey for ed25519::Public {
    fn verify_signature(&self, payload: &Payload, signature: &Signature) -> SignatureVerificationResult {
		ensure!(signature.len() == self.get_expected_signature_size(), SigningKeyError::InvalidSignatureFormat);
		let signature = signature as &[u8];
		let ed25519_sig = ed25519::Signature::try_from(
			signature
		).map_err(|_| SigningKeyError::InvalidSignatureFormat)?;
		
		Ok(ed25519_sig.verify(payload as &[u8], self))
    }

    fn get_expected_signature_size(&self) -> usize {
        64
    }
}

// SR25519 key
impl DIDPublicKey for sr25519::Public {
    fn get_did_key_description(&self) -> &'static str {
        "Sr25519VerificationKey2020"			// Not official yet. Proposed by Dock https://github.com/w3c-ccg/security-vocab/issues/32
    }
}

impl DIDPublicSigningKey for sr25519::Public {
    fn verify_signature(&self, payload: &Payload, signature: &Signature) -> SignatureVerificationResult {
		ensure!(signature.len() == self.get_expected_signature_size(), SigningKeyError::InvalidSignatureFormat);
		let signature = signature as &[u8];
        let sr25519_sig = sr25519::Signature::try_from(
			signature
		).map_err(|_| SigningKeyError::InvalidSignatureFormat)?;
		
		Ok(sr25519_sig.verify(payload as &[u8], self))
    }

	fn get_expected_signature_size(&self) -> usize {
		64
	}
}

// X25519 key
pub trait DIDPublicEncryptionKey: DIDPublicKey {}

impl DIDPublicEncryptionKey for x25519_dalek::PublicKey {}

impl DIDPublicKey for x25519_dalek::PublicKey {
    fn get_did_key_description(&self) -> &'static str {
        "X25519KeyAgreementKey2019"				// https://w3c.github.io/did-spec-registries/#x25519keyagreementkey2019
    }
}

#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct DIDDetails {
	auth_key: Box<dyn DIDPublicSigningKey>,
	key_agreement_key: Box<dyn DIDPublicEncryptionKey>,
	delegation_key: Option<Box<dyn DIDPublicSigningKey>>,
	attestation_key: Option<Box<dyn DIDPublicSigningKey>>,
	verification_keys: Option<BTreeSet<Box<dyn DIDPublicSigningKey>>>,
	endpoint_url: Option<URL>,
	last_tx_counter: u64
}

pub enum DIDUpdate {
	ActiveKeyDelete(DIDKeyType),
	EndpointUpdate(URL),
	KeyUpdate(DIDKeyType, Box<dyn DIDPublicKey>),
	VerificationKeyDelete(KeyIdentifier),
}

pub enum DIDKeyType {
	Authentication,
	CapabilityDelegation,
	CapabilityInvocation,           // For future use
	AssertionMethod,
	KeyAgreement,
}

pub trait DIDOperation: Encode {
	fn get_verification_key_type(&self) -> DIDKeyType;
	fn get_signature(&self) -> Signature;
	fn get_did(&self) -> DIDIdentifier;
}

#[derive(Encode, Decode, Default, Clone, PartialEq)]
pub struct DIDUpdateOperation {
	did: DIDIdentifier,
	operations: Vec<DIDUpdate>,
	signature: Signature,
	txCounter: u64,
}

impl DIDOperation for DIDUpdateOperation {
    fn get_verification_key_type(&self) -> DIDKeyType {
        DIDKeyType::Authentication
    }

    fn get_signature(&self) -> Signature {
        self.signature
    }

    fn get_did(&self) -> DIDIdentifier {
        self.did
    }
}

pub enum SigningKeyError {
	InvalidSignatureFormat
}

decl_error! {
	pub enum Error for Module<T: Config> {
		SigningKeyError
	}
}

decl_event!(
	/// Events for DIDs
	pub enum Event<T> where <T as frame_system::Config>::AccountId {
		/// A did has been created
		DidCreated(AccountId),
		/// A did has been removed
		DidRemoved(AccountId),
	}
);

decl_module! {
	/// The DID runtime module
	pub struct Module<T: Config> for enum Call where origin: T::Origin {

		/// Deposit events
		fn deposit_event() = default;

		/// Adds a DID on chain, where
		/// origin - the origin of the transaction
		/// sign_key - public signing key of the DID
		/// box_key - public boxing key of the DID
		/// doc_ref - optional reference to the DID document storage
		#[weight = <T as Config>::WeightInfo::add()]
		pub fn submit_did_update_operation(origin, did_operation: DIDUpdateOperation) -> DispatchResult {
			// origin of the transaction needs to be a signed sender account
			let sender = ensure_signed(origin)?;
			Ok(())
		}
	}
}

// impl<T: Config> Module<T> {

// 	pub fn verify_did_update_operation_signature(op: DIDUpdateOperation) -> SignatureVerificationResult {
// 		let did_entry = <DIDs<T>>::get(op.did);
// 		if (!did_entry.is_some()) {
// 			Ok(true)
// 		} else {
// 			Self::verify_did_operation_signature(op)?
// 			let last_tx_counter = did_entry.last_tx_counter;
// 			if (last_tx_counter >= op.txCounter) {
// 				Error(InvalidNonce)
// 			} else {
// 				Ok(true)
// 			}
// 		}
// 	}

// 	pub fn verify_did_operation_signature<O: DIDOperation>(op: O) -> SignatureVerificationResult {
// 		let did_verification_key = Self::retrieve_did_key_for_type(op.get_did(), op.get_verification_key_type());
// 		op.get_signature().is_some() && did_verification_key.is_some() && did_verification_key.verify_signature(op.encode().as_ref(), op.get_signature())
// 	}

// 	fn retrieve_did_key_for_type(did: DIDIdentifier, key_type: DIDKeyType) -> Option<DIDKeyEntry> {
// 		let did_entry = <DIDs<T>>::get(did);
// 		if (!did_entry.is_some()) {
// 			None
// 		}

// 		let key_identifier: Option<KeyIdentifier> = match key_type {
// 			DIDKeyType::AssertionMethod => did_entry.attestation_key_id,
// 			DIDKeyType::Authentication => did_entry.auth_key_id,
// 			DIDKeyType::CapabilityDelegation => did_entry.delegation_key_id,
// 			DIDKeyType::KeyAgreement => did_entry.key_agreement_key_id,
// 			_ => None
// 		};
// 		if (!key_identifier.is_some()) {
// 			None
// 		}

// 		<KeyDetails<T>>::get(key_identifier);
// 	}
// }
 

decl_storage! {
	trait Store for Module<T: Config> as DID {
		DIDs get(fn dids):map hasher(opaque_blake2_256) DIDIdentifier => Option<DIDDetails>;
	}
}