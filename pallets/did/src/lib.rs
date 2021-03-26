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

#[cfg(test)]
mod test_utils;
/// Test module for attestations
#[cfg(test)]
mod tests;

#[cfg(any(feature = "runtime-benchmarks", test))]
pub mod benchmarking;

pub mod default_weights;
pub use default_weights::WeightInfo;

use codec::{Decode, Encode};

use codec::EncodeLike;
use frame_support::{ensure, storage::types::StorageMap};
use frame_system::{self, ensure_signed};
use sp_core::{ed25519, sr25519};
use sp_runtime::traits::Verify;
use sp_std::{
	collections::btree_set::BTreeSet, convert::TryFrom, fmt::Debug, prelude::Clone, vec::Vec,
};

pub use pallet::*;

/// Reference to a payload of data of variable size.
pub type PayloadReference<'a> = &'a [u8];

/// Type of a signature (variable size as different signature schemes are supported).
pub type SignatureEncoding = Vec<u8>;

/// Reference to a signature of variable size.
pub type SignatureReference<'a> = &'a [u8];

/// Type for an encoded URL.
pub type UrlEncoding = Vec<u8>;

/// Trait representing a public key under the control of a DID subject.
pub trait DIDPublicKey {
	/// Returns the key method description as in the [DID specification](https://w3c.github.io/did-spec-registries/#verification-method-types).
	fn get_did_key_description(&self) -> &'static str;
}

/// An enum describing the different verification methods a verification key can fulfil, according to the [DID specification](https://w3c.github.io/did-spec-registries/#verification-relationships).
#[derive(Clone, Debug, Decode, Encode, PartialEq, Eq)]
pub enum DIDVerificationKeyType {
	Authentication,
	CapabilityDelegation,
	CapabilityInvocation, // Not used for now, but added for potential future use
	AssertionMethod,
}

/// An enum describing the different verification methods an encryption key can fulfil, according to the [DID specification](https://w3c.github.io/did-spec-registries/#verification-relationships).
#[derive(Clone, Debug, Decode, Encode, PartialEq)]
pub enum DIDEncryptionKeyType {
	KeyAgreement,
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
	fn verify_signature(
		&self,
		payload: PayloadReference,
		signature: SignatureReference,
	) -> Result<bool, SignatureError> {
		// Discard all invalid signatures by comparing them with the expected length of the specific verification key.
		ensure!(
			signature.len() == self.get_expected_signature_size(),
			SignatureError::InvalidSignatureFormat
		);

		match self {
			PublicVerificationKey::Ed25519(raw_key) => {
				// Try to re-create a Signature value or throw an error if raw value is invalid.
				let ed25519_sig = ed25519::Signature::try_from(signature)
					.map_err(|_| SignatureError::InvalidSignatureFormat)?;
				// Re-create a Public value from the raw value of the key.
				let signer = ed25519::Public(*raw_key);
				// Returns the result of the signature verification.
				Ok(ed25519_sig.verify(payload[..].as_ref(), &signer))
			}
			// Follows same process as above, but using a Sr25519 instead.
			PublicVerificationKey::Sr25519(raw_key) => {
				let sr25519_sig = sr25519::Signature::try_from(signature)
					.map_err(|_| SignatureError::InvalidSignatureFormat)?;
				let signer = sr25519::Public(*raw_key);
				Ok(sr25519_sig.verify(payload.as_ref(), &signer))
			}
		}
	}

	/// Returns the expected signature length (of bytes) for the given verification key type.
	fn get_expected_signature_size(&self) -> usize {
		match self {
			PublicVerificationKey::Ed25519(_) | PublicVerificationKey::Sr25519(_) => 64,
		}
	}
}

impl DIDPublicKey for PublicVerificationKey {
	fn get_did_key_description(&self) -> &'static str {
		match self {
			PublicVerificationKey::Ed25519(_) => "Ed25519VerificationKey2018", // https://w3c.github.io/did-spec-registries/#ed25519verificationkey2018
			PublicVerificationKey::Sr25519(_) => "Sr25519VerificationKey2020", // Not yet defined in the DID specification.
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
		"X25519KeyAgreementKey2019" // https://w3c.github.io/did-spec-registries/#x25519keyagreementkey2019
	}
}

/// All the errors that can be generated when evaluating a DID operation.
#[derive(Debug, Eq, PartialEq)]
pub enum DIDError {
	StorageError(StorageError),
	SignatureError(SignatureError),
}

// Used internally to handle storage errors.
#[derive(Debug, Eq, PartialEq)]
pub enum StorageError {
	/// The DID being created is already present on chain.
	DIDAlreadyPresent,
	/// The expected DID cannot be found on chain.
	DIDNotPresent,
	/// The given DID does not contain the right key to verify the signature of a DID operation.
	VerificationkeyNotPresent(DIDVerificationKeyType),
}

// Used internally to handle signature errors.
#[derive(Debug, Eq, PartialEq)]
pub enum SignatureError {
	/// The signature is not in the expected format the verification key expects.
	InvalidSignatureFormat,
	/// The signature is invalid for the payload and the verification key provided.
	InvalidSignature,
}

/// A trait describing an operation that requires DID authentication.
pub trait DIDOperation<DIDIdentifier>: Encode
where
	DIDIdentifier: Encode + Decode + Clone + Debug + Eq + PartialEq + EncodeLike,
{
	/// Returns the type of the verification key to be used to validate the operation.
	fn get_verification_key_type(&self) -> DIDVerificationKeyType;
	/// Returns the DID identifier of the subject.
	fn get_did(&self) -> &DIDIdentifier;
}

/// A DID creation request. It contains the following values:
/// - the DID identifier being created (only Substrate addresses are allowed in this version of the pallet);
/// - the new authentication key to use;
/// - the new encryption key to use;
/// - the optional attestation key to use;
/// - the optional delegation key to use;
/// - the optional endpoint URL pointing to the DID service endpoints.
#[derive(Clone, Decode, Debug, Encode, PartialEq)]
pub struct DIDCreationOperation<DIDIdentifier>
where
	DIDIdentifier: Encode + Decode + Clone + Debug + Eq + PartialEq + EncodeLike,
{
	did: DIDIdentifier,
	new_auth_key: PublicVerificationKey,
	new_key_agreement_key: PublicEncryptionKey,
	new_attestation_key: Option<PublicVerificationKey>,
	new_delegation_key: Option<PublicVerificationKey>,
	new_endpoint_url: Option<UrlEncoding>,
}

impl<DIDIdentifier> DIDOperation<DIDIdentifier> for DIDCreationOperation<DIDIdentifier>
where
	DIDIdentifier: Encode + Decode + Clone + Debug + Eq + PartialEq + EncodeLike,
{
	fn get_verification_key_type(&self) -> DIDVerificationKeyType {
		DIDVerificationKeyType::Authentication
	}

	fn get_did(&self) -> &DIDIdentifier {
		&self.did
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
	endpoint_url: Option<UrlEncoding>,
	last_tx_counter: u64,
}

impl<DIDIdentifier> From<&DIDCreationOperation<DIDIdentifier>> for DIDDetails
where
	DIDIdentifier: Encode + Decode + Clone + Debug + Eq + PartialEq + EncodeLike,
{
	fn from(op: &DIDCreationOperation<DIDIdentifier>) -> Self {
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
	fn get_verification_key_for_key_type(
		&self,
		key_type: DIDVerificationKeyType,
	) -> Option<&PublicVerificationKey> {
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

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		pallet_prelude::*,
		traits::{Hooks, IsType},
	};
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type WeightInfo: WeightInfo;
		type DIDIdentifier: Encode + Decode + Clone + Debug + Eq + PartialEq + EncodeLike;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::storage]
	#[pallet::getter(fn get_did)]
	pub type Did<T> = StorageMap<_, Blake2_128Concat, <T as Config>::DIDIdentifier, DIDDetails>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		DidCreated(<T as frame_system::Config>::AccountId, T::DIDIdentifier),
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidSignatureFormat,
		InvalidSignature,
		DIDAlreadyPresent,
		VerificationKeyNotPresent,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Stores a new DID on chain, after verifying the signature associated with the creation operation. The parameters are:
		/// - origin: the Substrate account submitting the transaction (which can be different from the DID subject);
		/// - did_creation_operation: a [DIDCreationOperation](DIDCreationOperation) which contains the details of the new DID;
		/// - signature: a signature over [DIDCreationOperation](DIDCreationOperation) that must be signed with the authentication key associated with the new DID.
		#[pallet::weight(<T as Config>::WeightInfo::submit_did_create_operation())]
		pub fn submit_did_create_operation(
			origin: OriginFor<T>,
			did_creation_operation: DIDCreationOperation<T::DIDIdentifier>,
			signature: SignatureEncoding,
		) -> DispatchResultWithPostInfo {
			// origin of the transaction needs to be a signed sender account
			let sender = ensure_signed(origin)?;

			// There has to be no other DID with the same identifier already saved on chain, otherwise generate a DIDNotPresent error.
			ensure!(
				!<Did<T>>::contains_key(did_creation_operation.get_did()),
				<Error<T>>::DIDAlreadyPresent
			);

			// Create a new DID entry from the details provided in the create operation.
			let did_entry = DIDDetails::from(&did_creation_operation);

			// Retrieve the authentication key of the new DID, otherwise generate a VerificationKeyNotPresent error if it is not specified
			// (should never happen as the DIDCreateOperation requires the authentication key to be present).
			let signature_verification_key = did_entry
				.get_verification_key_for_key_type(DIDVerificationKeyType::Authentication)
				.ok_or(<Error<T>>::VerificationKeyNotPresent)?;

			// Re-create a Signature object from the authentication key retrieved, or generate a InvalidSignatureFormat error otherwise.
			let is_signature_valid = signature_verification_key
				.verify_signature(&did_creation_operation.encode(), &signature)
				.map_err(|_| <Error<T>>::InvalidSignatureFormat)?;

			// Verify the validity of the signature, or generate an InvalidSignature error otherwise.
			ensure!(is_signature_valid, <Error<T>>::InvalidSignature);

			let did_identifier = &did_creation_operation.get_did().clone();
			log::debug!("Creating DID {:?}", did_identifier);
			<Did<T>>::insert(did_identifier, did_entry);

			Self::deposit_event(Event::DidCreated(sender, did_identifier.clone()));
			//TODO: Return the real weight used
			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Verify the signature of a generic [DIDOperation](DIDOperation), and returns either Ok or a [DIDError](DIDError).
	/// The paremeters are:
	/// - op: a reference to the DID operation;
	/// - signature: a reference to the signature;
	pub fn verify_did_operation_signature<O: DIDOperation<T::DIDIdentifier>>(
		op: &O,
		signature: SignatureReference,
	) -> Result<(), DIDError> {
		// Switch to a slice
		// Try to retrieve from the storage the details of the given DID.
		let did_entry: Option<DIDDetails> = <Did<T>>::get(op.get_did());

		// If there is no DID stored, generate a DIDNotPresent error.
		ensure!(
			did_entry.is_some(),
			DIDError::StorageError(StorageError::DIDNotPresent)
		);

		// Force unwrap the DID details, as we are sure it is not None.
		let did_entry = did_entry.unwrap();

		// Retrieves the needed verification key from the DID details, or generate a VerificationkeyNotPresent error if there is no key of the type required.
		let verification_key = did_entry
			.get_verification_key_for_key_type(op.get_verification_key_type())
			.ok_or_else(|| {
				DIDError::StorageError(StorageError::VerificationkeyNotPresent(
					op.get_verification_key_type(),
				))
			})?;

		// Verifies that the signature matches the expected format, otherwise generate an InvalidSignatureFormat error.
		let is_signature_valid = verification_key
			.verify_signature(&op.encode(), signature)
			.map_err(|_| DIDError::SignatureError(SignatureError::InvalidSignatureFormat))?;

		ensure!(
			is_signature_valid,
			DIDError::SignatureError(SignatureError::InvalidSignature)
		);

		Ok(())
	}
}
