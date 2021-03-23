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

use codec::{Decode, Encode};
pub use default_weights::WeightInfo;

use frame_support::{StorageMap, decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure};
use frame_system::{self, ensure_signed};
use sp_runtime::traits::Verify;
use sp_std::{prelude::{Clone}, convert::TryFrom, collections::{btree_set::BTreeSet}, vec::Vec};
use sp_core::{ed25519, sr25519};

/// Type of a payload of data (to verify signatures against).
pub type Payload = Vec<u8>;

/// Reference to a payload of data of variable size.
pub type PayloadReference<'a> = &'a [u8];

/// Type of a signature (variable size as different signature schemes are supported).
pub type Signature = Vec<u8>;

/// Reference to a signature of variable size.
pub type SignatureReference<'a> = &'a [u8];

/// Type for a DID identifier.
pub type DIDIdentifier = Vec<u8>;

/// Type for a URL.
pub type Url = Vec<u8>;

pub trait Config: frame_system::Config {
	/// DID specific event type.
	type Event: From<Event> + Into<<Self as frame_system::Config>::Event>;

	/// Weight information for extrinsics in this pallet.
	type WeightInfo: WeightInfo;
}

/// Trait representing a public key under the control of a DID subject.
pub trait DIDPublicKey {
	/// Returns the key method description as in the [DID specification](https://w3c.github.io/did-spec-registries/#verification-method-types).
	fn get_did_key_description(&self) -> &'static str;
}

/// Enum representing the types of verification keys a DID can control.
#[derive(Clone, Decode, Debug, Encode, Eq, Ord, PartialEq, PartialOrd)]
pub enum PublicVerificationKey {
	/// An Ed25519 public key.
	Ed25519([u8; 32]),
	/// A Sr25519 public key.
	Sr25519([u8; 32]),
}

impl PublicVerificationKey {
	/// Given a payload and a signature, the specific public verification key will return either
	/// a [SignatureError](SignatureError) if the signature is not properly formed, or a boolean indicating
	/// the result of the verification.
	fn verify_signature(&self, payload: PayloadReference, signature: SignatureReference) -> Result<bool, SignatureError> {
		// Discard all invalid signatures by comparing them with the expected length of the specific verification key.
		ensure!(signature.len() == self.get_expected_signature_size(), SignatureError::InvalidSignatureFormat);

		match self {
			PublicVerificationKey::Ed25519(raw_key) => {
				// Try to re-create a Signature value or throw an error if raw value is invalid.
				let ed25519_sig = ed25519::Signature::try_from(
					signature
				).map_err(|_| SignatureError::InvalidSignatureFormat)?;
				// Re-create a Public value from the raw value of the key.
				let signer = ed25519::Public(*raw_key);
				// Returns the result of the signature verification.
				Ok(ed25519_sig.verify(payload[..].as_ref(), &signer))
			}
			// Follows same process as above, but using a Sr25519 instead.
			PublicVerificationKey::Sr25519(raw_key) => {
				let sr25519_sig = sr25519::Signature::try_from(
					signature
				).map_err(|_| SignatureError::InvalidSignatureFormat)?;
				let signer = sr25519::Public(*raw_key);
				Ok(sr25519_sig.verify(payload.as_ref(), &signer))
			}
		}
    }

	/// Returns the expected signature length (of bytes) for the given verification key type.
    fn get_expected_signature_size(&self) -> usize {
        match self {
			PublicVerificationKey::Ed25519(_) | PublicVerificationKey::Sr25519(_) => {
				64
			}
		}
    }
}

impl DIDPublicKey for PublicVerificationKey {
    fn get_did_key_description(&self) -> &'static str {
		match self {
			PublicVerificationKey::Ed25519(_) => "Ed25519VerificationKey2018",		// https://w3c.github.io/did-spec-registries/#ed25519verificationkey2018
			PublicVerificationKey::Sr25519(_) => "Sr25519VerificationKey2020"		// Not yet defined in the DID specification.
		}
    }
}

/// Enum representing the types of encryption keys a DID can control.
#[derive(Clone, Decode, Debug, Encode, Eq, Ord, PartialEq, PartialOrd)]
pub enum PublicEncryptionKey {
	/// An X25519 public key.
	X55519([u8; 32]),
}

impl DIDPublicKey for PublicEncryptionKey {
    fn get_did_key_description(&self) -> &'static str {
		"X25519KeyAgreementKey2019"				// https://w3c.github.io/did-spec-registries/#x25519keyagreementkey2019
    }
}

/// The details associated to a DID identity. Specifically:
/// - the authentication key, used to authenticate DID-related operations;
/// - the key agreement key, used to encrypt data addressed to the DID subject;
/// - an optional delegation key, used by the DID subject to sign delegation nodes before writing them on chain;
/// - an optional attestation key, used by the DID subject to sign attestations before writing them on chain;
/// - an optional URL pointing to the service endpoints the DID subject publicly exposes;
/// - a counter used to avoid replay attacks, which is checked and updated upon each DID-related operation.
#[derive(Clone, Decode, Encode, PartialEq)]
pub struct DIDDetails {
	auth_key: PublicVerificationKey,
	key_agreement_key: PublicEncryptionKey,
	delegation_key: Option<PublicVerificationKey>,
	attestation_key: Option<PublicVerificationKey>,
	verification_keys: BTreeSet<PublicVerificationKey>,
	endpoint_url: Option<Url>,
	last_tx_counter: u64
}

impl From<&DIDCreationOperation> for DIDDetails {
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
	/// Returns a reference to a specific verification key given the type of the key needed.
	fn get_verification_key_for_key_type(&self, key_type: DIDVerificationKeyType) -> Option<&PublicVerificationKey> {
		match key_type {
			DIDVerificationKeyType::AssertionMethod => self.attestation_key.as_ref(),
			DIDVerificationKeyType::Authentication => Option::from(&self.auth_key),
			DIDVerificationKeyType::CapabilityDelegation => self.delegation_key.as_ref(),
			_ => None,
		}
	}

	// fn get_kencryption_key_for_key_type(&self, key_type: DIDEncryptionKeyType) -> Option<&PublicEncryptionKey> {
	// 	match key_type {
	// 		DIDEncryptionKeyType::KeyAgreement => Option::from(&self.key_agreement_key),
	// 	}
	// }
}

/// A trait describing an operation that requires DID authentication.
pub trait DIDOperation: Encode {
	/// Returns the type of the verification key to be used to validate the operation.
	fn get_verification_key_type(&self) -> DIDVerificationKeyType;
	/// Returns the DID identifier of the subject.
	fn get_did(&self) -> &DIDIdentifier;
}

/// An enum describing the different verification methods a verification key can fulfil, according to the [DID specification](https://w3c.github.io/did-spec-registries/#verification-relationships).
#[derive(Clone, Debug, Decode, Encode, PartialEq)]
pub enum DIDVerificationKeyType {
	Authentication,
	CapabilityDelegation,
	CapabilityInvocation,           // Not used for now, but added for potential future use
	AssertionMethod,
}

/// An enum describing the different verification methods an encryption key can fulfil, according to the [DID specification](https://w3c.github.io/did-spec-registries/#verification-relationships).
#[derive(Clone, Debug, Decode, Encode, PartialEq)]
pub enum DIDEncryptionKeyType {
	KeyAgreement,
}

/// A DID creation request. It contains the following values:
/// - the DID identifier being created;
/// - the new authentication key to use;
/// - the new encryption key to use;
/// - the optional attestation key to use;
/// - the optional delegation key to use;
/// - the optional endpoint URL pointing to the DID service endpoints.
#[derive(Clone, Decode, Debug, Encode, PartialEq)]
pub struct DIDCreationOperation {
	did: DIDIdentifier,
	new_auth_key: PublicVerificationKey,
	new_key_agreement_key: PublicEncryptionKey,
	new_attestation_key: Option<PublicVerificationKey>,
	new_delegation_key: Option<PublicVerificationKey>,
	new_endpoint_url: Option<Url>,
}

impl DIDOperation for DIDCreationOperation {
    fn get_verification_key_type(&self) -> DIDVerificationKeyType {
        DIDVerificationKeyType::Authentication
    }

    fn get_did(&self) -> &DIDIdentifier {
        self.did.as_ref()
    }
}

/// All the errors that can be generated when evaluating a DID operation.
pub enum DIDError {
	StorageError(StorageError),
	SignatureError(SignatureError),
}

pub enum StorageError {
	/// The DID being created is already present on chain.
	DIDAlreadyPresent,
	/// The expected DID cannot be found on chain.
	DIDNotPresent,
	/// The given DID does not contain the right key to verify the signature of a DID operation.
	VerificationkeyNotPresent(DIDVerificationKeyType),
}

// Used internally to handle signature errors.
pub enum SignatureError {
	/// The signature is not in the expected format the verification key expects.
	InvalidSignatureFormat,
	/// The signature is invalid for the payload and the verification key provided.
	InvalidSignature,
}

decl_error! {
	pub enum Error for Module<T: Config> {
		InvalidSignatureFormat,
		InvalidSignature,
		DIDNotPresent,
		VerificationKeyNotPresent,
	}
}

decl_event!(
	/// Events for DIDs
	pub enum Event {
		/// Event generated after a succesfull creation of a new DID.
		DidCreated(DIDIdentifier),
	}
);

decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {

		/// Deposit events.
		fn deposit_event() = default;

		// Initialize errors.
		type Error = Error<T>;

		/// Stores a new DID on chain, after verifying the signature associated with the creation operation. The parameters are:
		/// - origin: the Substrate account submitting the transaction (which can be different from the DID subject);
		/// - did_creation_operation: a [DIDCreationOperation](DIDCreationOperation) which contains the details of the new DID;
		/// - signature: a signature over [DIDCreationOperation](DIDCreationOperation) that must be signed with the authentication key associated with the new DID.
		#[weight = <T as Config>::WeightInfo::add()]
		pub fn submit_did_create_operation(origin, did_creation_operation: DIDCreationOperation, signature: Signature) -> DispatchResult {
			// origin of the transaction needs to be a signed sender account
			ensure_signed(origin)?;

			// There has to be no other DID with the same identifier already saved on chain, otherwise generate a DIDNotPresent error.
			ensure!(DIDs::contains_key(did_creation_operation.get_did()), <Error<T>>::DIDNotPresent);

			// Create a new DID entry from the details provided in the create operation.
			let did_entry = DIDDetails::from(&did_creation_operation);

			// Retrieve the authentication key of the new DID, otherwise generate a VerificationKeyNotPresent error if it is not specified
			// (should never happen as the DIDCreateOperation requires the authentication key to be present).
			let signature_verification_key = did_entry.get_verification_key_for_key_type(DIDVerificationKeyType::Authentication).ok_or(<Error<T>>::VerificationKeyNotPresent)?;

			// Re-create a Signature object from the authentication key retrieved, or generate a InvalidSignatureFormat error otherwise.
			let is_signature_valid = signature_verification_key.verify_signature(&did_creation_operation.encode(), &signature).map_err(|_| <Error<T>>::InvalidSignatureFormat)?;

			// Verify the validity of the signature, or generate an InvalidSignature error otherwise.
			ensure!(is_signature_valid, <Error<T>>::InvalidSignature);

			let did_identifier = &did_creation_operation.get_did().clone();
			log::debug!("Creating DID {:?}", did_identifier);
			DIDs::insert(did_identifier, did_entry);

			Self::deposit_event(Event::DidCreated(did_identifier.to_vec()));
			Ok(())
		}
	}
}

impl<T: Config> Module<T> {
	/// Verify the signature of a generic [DIDOperation](DIDOperation), and returns either Ok or a [DIDError](DIDError).
	/// The paremeters are:
	/// - op: a reference to the DID operation;
	/// - signature: a reference to the signature;
	pub fn verify_did_operation_signature<O: DIDOperation>(op: &O, signature: SignatureReference) -> Result<bool, DIDError> {			// Switch to a slice
		// Try to retrieve from the storage the details of the given DID.
		let did_entry: Option<DIDDetails> = DIDs::get(op.get_did());

		// If there is no DID stored, generate a DIDNotPresent error.
		ensure!(did_entry.is_some(), DIDError::StorageError(StorageError::DIDNotPresent));

		// Force unwrap the DID details, as we are sure it is not None.
		let did_entry = did_entry.unwrap();

		// Retrieves the needed verification key from the DID details, or generate a VerificationkeyNotPresent error if there is no key of the type required.
		let verification_key = did_entry.get_verification_key_for_key_type(op.get_verification_key_type()).ok_or_else(|| DIDError::StorageError(StorageError::VerificationkeyNotPresent(op.get_verification_key_type())))?;

		// Verifies that the signature matches the expected format, otherwise generate an InvalidSignatureFormat error.
		let is_signature_valid = verification_key.verify_signature(&op.encode(), signature).map_err(|_| DIDError::SignatureError(SignatureError::InvalidSignatureFormat))?;

		// Return the result of the signature verification.
		Ok(is_signature_valid)
	}
}


decl_storage! {
	trait Store for Module<T: Config> as DID {
		DIDs get(fn dids):map hasher(opaque_blake2_256) DIDIdentifier => Option<DIDDetails>;
	}
}
