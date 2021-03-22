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
// pub mod migration;				// Temporary disabled

use codec::{Decode, Encode};
pub use default_weights::WeightInfo;

use frame_support::{StorageMap, decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure};
use frame_system::{self, ensure_signed};
use sp_runtime::traits::Verify;
use sp_std::{prelude::{Clone}, convert::TryFrom, collections::{btree_set::BTreeSet}, vec::Vec};
use sp_core::{ed25519, sr25519};

pub type Payload = Vec<u8>;
pub type Signature = Vec<u8>;
pub type KeyValue = Vec<u8>;
pub type DIDIdentifier = Vec<u8>;
pub type KeyIdentifier = Vec<u8>;
pub type URL = Vec<u8>;
pub type SignatureVerificationResult = Result<bool, SignatureError>;
pub type DIDSignatureVerificationResult = Result<bool, DIDError>;

/// The DID trait
pub trait Config: frame_system::Config {
	/// DID specific event type
	type Event: From<Event> + Into<<Self as frame_system::Config>::Event>;

	/// Weight information for extrinsics in this pallet.
	type WeightInfo: WeightInfo;
}

pub trait DIDPublicKey {
	fn get_did_key_description(&self) -> &'static str;
}

#[derive(Clone, Decode, Debug, Encode, Eq, Ord, PartialEq, PartialOrd)]
pub enum PublicVerificationKey {
	Ed25519([u8; 32]),
	Sr25519([u8; 32]),
}

impl PublicVerificationKey {
	fn verify_signature(&self, payload: &Payload, signature: &Signature) -> SignatureVerificationResult {
		ensure!(signature.len() == self.get_expected_signature_size(), SignatureError::InvalidSignatureFormat);
		let signature = signature as &[u8];
		// Did not find a way to return a Signature object from the match and then call directly verify on that...
		match self {
			PublicVerificationKey::Ed25519(raw_key) => {
				let ed25519_sig = ed25519::Signature::try_from(
					signature
				).map_err(|_| SignatureError::InvalidSignatureFormat)?;
				let signer = ed25519::Public(raw_key.clone());
				Ok(ed25519_sig.verify(payload as &[u8], &signer))
			}
			PublicVerificationKey::Sr25519(raw_key) => {
				let sr25519_sig = sr25519::Signature::try_from(
					signature
				).map_err(|_| SignatureError::InvalidSignatureFormat)?;
				let signer = sr25519::Public(raw_key.clone());
				Ok(sr25519_sig.verify(payload as &[u8], &signer))
			}
		}
    }

    fn get_expected_signature_size(&self) -> usize {
        match self {
			PublicVerificationKey::Ed25519(_) | PublicVerificationKey::Sr25519(_) => {
				64
			}
		}
    }
}

impl Default for PublicVerificationKey {
    fn default() -> Self {
        PublicVerificationKey::Sr25519([0; 32])
    }
}

impl DIDPublicKey for PublicVerificationKey {
    fn get_did_key_description(&self) -> &'static str {
		match self {
			PublicVerificationKey::Ed25519(_) => "Ed25519VerificationKey2018",
			PublicVerificationKey::Sr25519(_) => "Sr25519VerificationKey2020"
		}
    }
}

#[derive(Clone, Decode, Debug, Encode, Eq, Ord, PartialEq, PartialOrd)]
pub enum PublicEncryptionKey {
	X55519([u8; 32]),
}

impl DIDPublicKey for PublicEncryptionKey {
    fn get_did_key_description(&self) -> &'static str {
        "X25519KeyAgreementKey2019"				// https://w3c.github.io/did-spec-registries/#x25519keyagreementkey2019
    }
}

impl Default for PublicEncryptionKey {
    fn default() -> Self {
        PublicEncryptionKey::X55519([0; 32])
    }
}

#[derive(Clone, Decode, Encode, PartialEq)]
pub struct DIDDetails {
	auth_key: PublicVerificationKey,
	key_agreement_key: PublicEncryptionKey,
	delegation_key: Option<PublicVerificationKey>,
	attestation_key: Option<PublicVerificationKey>,
	verification_keys: BTreeSet<PublicVerificationKey>,
	endpoint_url: Option<URL>,
	last_tx_counter: u64
}

impl<'a> From<&'a DIDCreationOperation> for DIDDetails {
    fn from(op: &DIDCreationOperation) -> Self {
		DIDDetails {
			auth_key: op.new_auth_key.clone(),
			key_agreement_key: op.new_key_agreement_key.clone(),
			delegation_key: op.new_delegation_key.clone(),
			attestation_key: op.new_attestation_key.clone(),
			verification_keys: BTreeSet::new(),
			endpoint_url: op.new_endpoint_url.clone(),
			last_tx_counter: 0,
		}
    }
}

impl DIDDetails {
	fn get_verification_key_for_key_type(&self, key_type: DIDVerificationKeyType) -> Option<&PublicVerificationKey> {
		match key_type {
			DIDVerificationKeyType::AssertionMethod => Option::from(self.attestation_key.as_ref()),
			DIDVerificationKeyType::Authentication => Option::from(&self.auth_key),
			DIDVerificationKeyType::CapabilityDelegation => Option::from(self.delegation_key.as_ref()),
			_ => None,
		}
	}

	// fn get_kencryption_key_for_key_type(&self, key_type: DIDEncryptionKeyType) -> Option<&PublicEncryptionKey> {
	// 	match key_type {
	// 		DIDEncryptionKeyType::KeyAgreement => Option::from(&self.key_agreement_key),
	// 	}
	// }
}

#[derive(Clone, Decode, Debug, Encode, PartialEq)]
pub struct DIDCreationOperation {
	did: DIDIdentifier,
	new_auth_key: PublicVerificationKey,
	new_key_agreement_key: PublicEncryptionKey,
	new_attestation_key: Option<PublicVerificationKey>,
	new_delegation_key: Option<PublicVerificationKey>,
	new_endpoint_url: Option<URL>,
}

impl DIDOperation for DIDCreationOperation {
    fn get_verification_key_type(&self) -> DIDVerificationKeyType {
        DIDVerificationKeyType::Authentication
    }

    fn get_did(&self) -> &DIDIdentifier {
        self.did.as_ref()
    }
}

#[derive(Clone, Debug, Decode, Encode, PartialEq)]
pub enum DIDVerificationKeyType {
	Authentication,
	CapabilityDelegation,
	CapabilityInvocation,           // For future use
	AssertionMethod,
}

#[derive(Clone, Debug, Decode, Encode, PartialEq)]
pub enum DIDEncryptionKeyType {
	KeyAgreement,
}

pub trait DIDOperation: Encode {
	fn get_verification_key_type(&self) -> DIDVerificationKeyType;
	fn get_did(&self) -> &DIDIdentifier;
}

pub enum DIDError {
	SignatureError(SignatureError),
	StorageError(StorageError),
	DIDFormatError(DIDFormatError),
}

pub enum SignatureError {
	InvalidSignatureFormat,
	InvalidSignature,
}

pub enum StorageError {
	DIDAlreadyPresent,
	DIDNotPresent,
	VerificationkeyNotPresent,
}

pub enum DIDFormatError {
	AuhenticationKeyNotPresent,
}

decl_error! {
	// TODO: How to add the DIDError enum to here and use within the extrinsics?
	pub enum Error for Module<T: Config> {
		InvalidSignatureFormat,
		InvalidSignature,
		StorageError,
		DIDAlreadyPresent,
		DIDNotPresent,
		VerificationkeyNotPresent,
		AuhenticationKeyNotPresent,
	}
}

decl_event!(
	/// Events for DIDs
	pub enum Event {
		/// A did has been created
		DidCreated(DIDIdentifier),
		/// A did has been updated
		DidUpdated(DIDIdentifier),
		/// A did has been removed
		DidRemoved(DIDIdentifier),
	}
);

decl_module! {
	/// The DID runtime module
	pub struct Module<T: Config> for enum Call where origin: T::Origin {

		/// Deposit events
		fn deposit_event() = default;

		type Error = Error<T>;

		#[weight = <T as Config>::WeightInfo::add()]
		pub fn submit_did_create_operation(origin, did_creation_operation: DIDCreationOperation, signature: Signature) -> DispatchResult {
			// origin of the transaction needs to be a signed sender account
			ensure_signed(origin)?;

			ensure!(DIDs::contains_key(did_creation_operation.get_did()), <Error<T>>::DIDNotPresent);

			let did_entry = DIDDetails::from(&did_creation_operation);

			let signature_verification_key = did_entry.get_verification_key_for_key_type(DIDVerificationKeyType::Authentication).ok_or(<Error<T>>::AuhenticationKeyNotPresent)?;

			let is_signature_valid = signature_verification_key.verify_signature(&did_creation_operation.encode(), &signature).map_err(|_| <Error<T>>::InvalidSignatureFormat)?;

			ensure!(is_signature_valid, <Error<T>>::InvalidSignature);

			Ok(())	// Emit relevant event
		}
	}
}

impl<T: Config> Module<T> {
	pub fn verify_did_operation_signature<O: DIDOperation>(op: &O, signature: &Signature, fail_if_absent: bool) -> DIDSignatureVerificationResult {
		let did_entry: Option<DIDDetails> = DIDs::get(op.get_did());

		ensure!(did_entry.is_some() || !fail_if_absent, DIDError::StorageError(StorageError::DIDNotPresent));

		if let Some(did_entry) = did_entry {
			let verification_key = did_entry.get_verification_key_for_key_type(op.get_verification_key_type()).ok_or(DIDError::StorageError(StorageError::VerificationkeyNotPresent))?;
			let is_signature_valid = verification_key.verify_signature(&op.encode(), signature).map_err(|err| DIDError::SignatureError(err))?;
			return Ok(is_signature_valid);
		}
		Ok(true)		// If no DID entry is present and signature verification should not fail (fail_if_absent is false), return true.
	}
}
 

decl_storage! {
	trait Store for Module<T: Config> as DID {
		DIDs get(fn dids):map hasher(opaque_blake2_256) DIDIdentifier => Option<DIDDetails>;
	}
}