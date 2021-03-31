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

#[cfg(test)]
mod mock;

#[cfg(any(feature = "runtime-benchmarks", test))]
pub mod benchmarking;

pub mod default_weights;
pub use default_weights::WeightInfo;

use codec::{Decode, Encode};

use frame_support::{ensure, storage::types::StorageMap, Parameter};
use frame_system::{self, ensure_signed};
use sp_core::{ed25519, sr25519};
use sp_runtime::traits::Verify;
use sp_std::{collections::btree_set::BTreeSet, fmt::Debug, prelude::Clone, vec::Vec};

pub use pallet::*;

/// Reference to a payload of data of variable size.
pub type Payload = [u8];

/// Type for an encoded URL.
pub type UrlEncoding = Vec<u8>;

/// Trait representing a public key under the control of a DID subject.
pub trait DidPublicKey {
	/// Returns the key method description as in the [DID specification](https://w3c.github.io/did-spec-registries/#verification-method-types).
	fn get_did_key_description(&self) -> &'static str;
}

/// An enum describing the different verification methods a verification key can
/// fulfil, according to the [DID specification](https://w3c.github.io/did-spec-registries/#verification-relationships).
#[derive(Clone, Debug, Decode, Encode, PartialEq, Eq)]
pub enum DidVerificationKeyType {
	Authentication,
	CapabilityDelegation,
	// Not used for now, but added for potential future use.
	CapabilityInvocation,
	AssertionMethod,
}

/// An enum describing the different verification methods an encryption key can
/// fulfil, according to the [DID specification](https://w3c.github.io/did-spec-registries/#verification-relationships).
#[derive(Clone, Debug, Decode, Encode, PartialEq)]
pub enum DidEncryptionKeyType {
	KeyAgreement,
}

/// Enum representing the types of verification keys a DID can control.
#[derive(Clone, Copy, Decode, Debug, Encode, Eq, Ord, PartialEq, PartialOrd)]
pub enum PublicVerificationKey {
	/// An Ed25519 public key.
	Ed25519(ed25519::Public),
	/// A Sr25519 public key.
	Sr25519(sr25519::Public),
}

impl From<ed25519::Public> for PublicVerificationKey {
	fn from(key: ed25519::Public) -> Self {
		PublicVerificationKey::Ed25519(key)
	}
}

impl From<sr25519::Public> for PublicVerificationKey {
	fn from(key: sr25519::Public) -> Self {
		PublicVerificationKey::Sr25519(key)
	}
}

impl PublicVerificationKey {
	/// Given a payload and a signature, the specific public verification key
	/// will return either a SignatureError if the signature
	/// is not properly formed, or a boolean indicating the result of the
	/// verification.
	fn verify_signature(&self, payload: &Payload, signature: &DidSignature) -> Result<bool, SignatureError> {
		match self {
			PublicVerificationKey::Ed25519(public_key) => {
				// Try to re-create a Signature value or throw an error if raw value is invalid.
				if let DidSignature::Ed25519(sig) = signature {
					Ok(sig.verify(payload, &public_key))
				} else {
					Err(SignatureError::InvalidSignatureFormat)
				}
			}
			// Follows same process as above, but using a Sr25519 instead.
			PublicVerificationKey::Sr25519(public_key) => {
				if let DidSignature::Sr25519(sig) = signature {
					Ok(sig.verify(payload, &public_key))
				} else {
					Err(SignatureError::InvalidSignatureFormat)
				}
			}
		}
	}
}

impl DidPublicKey for PublicVerificationKey {
	fn get_did_key_description(&self) -> &'static str {
		match self {
			// https://w3c.github.io/did-spec-registries/#ed25519verificationkey2018
			PublicVerificationKey::Ed25519(_) => "Ed25519VerificationKey2018",
			// Not yet defined in the DID specification.
			PublicVerificationKey::Sr25519(_) => "Sr25519VerificationKey2020",
		}
	}
}

/// Enum representing the types of signatures supported by this pallet.
#[derive(Clone, Decode, Debug, Encode, Eq, PartialEq)]
pub enum DidSignature {
	/// A Ed25519 signature
	Ed25519(ed25519::Signature),
	/// A Sr25519 signature
	Sr25519(sr25519::Signature),
}

impl From<ed25519::Signature> for DidSignature {
	fn from(sig: ed25519::Signature) -> Self {
		DidSignature::Ed25519(sig)
	}
}

impl From<sr25519::Signature> for DidSignature {
	fn from(sig: sr25519::Signature) -> Self {
		DidSignature::Sr25519(sig)
	}
}

/// Enum representing the types of encryption keys a DID can control.
#[derive(Clone, Copy, Decode, Debug, Encode, Eq, Ord, PartialEq, PartialOrd)]
pub enum PublicEncryptionKey {
	/// An X25519 public key.
	X55519([u8; 32]),
}

impl DidPublicKey for PublicEncryptionKey {
	fn get_did_key_description(&self) -> &'static str {
		// https://w3c.github.io/did-spec-registries/#x25519keyagreementkey2019
		"X25519KeyAgreementKey2019"
	}
}

/// All the errors that can be generated when evaluating a DID operation.
#[derive(Debug, Eq, PartialEq)]
pub enum DidError {
	StorageError(StorageError),
	SignatureError(SignatureError),
}

// Used internally to handle storage errors.
#[derive(Debug, Eq, PartialEq)]
pub enum StorageError {
	/// The DID being created is already present on chain.
	DidAlreadyPresent,
	/// The expected DID cannot be found on chain.
	DidNotPresent,
	/// The given DID does not contain the right key to verify the signature of
	/// a DID operation.
	DidKeyNotPresent(DidVerificationKeyType),
}

// Used internally to handle signature errors.
#[derive(Debug, Eq, PartialEq)]
pub enum SignatureError {
	/// The signature is not in the expected format the verification key
	/// expects.
	InvalidSignatureFormat,
	/// The signature is invalid for the payload and the verification key
	/// provided.
	InvalidSignature,
}

/// A trait describing an operation that requires DID authentication.
pub trait DidOperation<DidIdentifier>: Encode {
	/// Returns the type of the verification key to be used to validate the
	/// operation.
	fn get_verification_key_type(&self) -> DidVerificationKeyType;
	/// Returns the DID identifier of the subject.
	fn get_did(&self) -> &DidIdentifier;
}

/// A DID creation request. It contains the following values:
/// * The DID identifier being created (only Substrate addresses are allowed in
///   this version of the pallet)
/// * The new authentication key to use
/// * The new encryption key to use
/// * The optional attestation key to use
/// * The optional delegation key to use
/// * The optional endpoint URL pointing to the DID service endpoints
#[derive(Clone, Decode, Debug, Encode, PartialEq)]
pub struct DidCreationOperation<DidIdentifier>
where
	DidIdentifier: Parameter + Encode + Decode + Debug,
{
	did: DidIdentifier,
	new_auth_key: PublicVerificationKey,
	new_key_agreement_key: PublicEncryptionKey,
	new_attestation_key: Option<PublicVerificationKey>,
	new_delegation_key: Option<PublicVerificationKey>,
	new_endpoint_url: Option<UrlEncoding>,
}

impl<DidIdentifier> DidOperation<DidIdentifier> for DidCreationOperation<DidIdentifier>
where
	DidIdentifier: Parameter + Encode + Decode + Debug,
{
	fn get_verification_key_type(&self) -> DidVerificationKeyType {
		DidVerificationKeyType::Authentication
	}

	fn get_did(&self) -> &DidIdentifier {
		&self.did
	}
}

/// The details associated to a DID identity. Specifically:
/// * The authentication key, used to authenticate DID-related operations
/// * The key agreement key, used to encrypt data addressed to the DID subject
/// * An optional delegation key, used by the DID subject to sign delegation
///   nodes before writing them on chain
/// * An optional attestation key, used by the DID subject to sign attestations
///   before writing them on chain
/// * An optional URL pointing to the service endpoints the DID subject publicly
///   exposes
/// * A counter used to avoid replay attacks, which is checked and updated upon
///   each DID-related operation
#[derive(Clone, Decode, Encode, PartialEq)]
pub struct DidDetails {
	auth_key: PublicVerificationKey,
	key_agreement_key: PublicEncryptionKey,
	delegation_key: Option<PublicVerificationKey>,
	attestation_key: Option<PublicVerificationKey>,
	verification_keys: BTreeSet<PublicVerificationKey>,
	endpoint_url: Option<UrlEncoding>,
	last_tx_counter: u64,
}

impl<DidIdentifier> From<DidCreationOperation<DidIdentifier>> for DidDetails
where
	DidIdentifier: Parameter + Encode + Decode + Debug,
{
	fn from(op: DidCreationOperation<DidIdentifier>) -> Self {
		DidDetails {
			auth_key: op.new_auth_key,
			key_agreement_key: op.new_key_agreement_key,
			delegation_key: op.new_delegation_key,
			attestation_key: op.new_attestation_key,
			verification_keys: BTreeSet::new(),
			endpoint_url: op.new_endpoint_url,
			last_tx_counter: 0,
		}
	}
}

impl DidDetails {
	/// Returns a reference to a specific verification key given the type of the
	/// key needed.
	fn get_verification_key_for_key_type(&self, key_type: DidVerificationKeyType) -> Option<&PublicVerificationKey> {
		match key_type {
			DidVerificationKeyType::AssertionMethod => self.attestation_key.as_ref(),
			DidVerificationKeyType::Authentication => Option::from(&self.auth_key),
			DidVerificationKeyType::CapabilityDelegation => self.delegation_key.as_ref(),
			_ => None,
		}
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
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type WeightInfo: WeightInfo;
		type DidIdentifier: Parameter + Encode + Decode + Debug;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::storage]
	#[pallet::getter(fn get_did)]
	pub type Did<T> = StorageMap<_, Blake2_128Concat, <T as Config>::DidIdentifier, DidDetails>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		DidCreated(<T as frame_system::Config>::AccountId, T::DidIdentifier),
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidSignatureFormat,
		InvalidSignature,
		DidAlreadyPresent,
		DidNotPresent,
		VerificationKeyNotPresent,
	}

	impl<T> From<DidError> for Error<T> {
		fn from(error: DidError) -> Self {
			match error {
				DidError::SignatureError(signature_error) => Self::from(signature_error),
				DidError::StorageError(storage_error) => Self::from(storage_error),
			}
		}
	}

	impl<T> From<SignatureError> for Error<T> {
		fn from(error: SignatureError) -> Self {
			match error {
				SignatureError::InvalidSignature => Self::InvalidSignature,
				SignatureError::InvalidSignatureFormat => Self::InvalidSignatureFormat,
			}
		}
	}

	impl<T> From<StorageError> for Error<T> {
		fn from(error: StorageError) -> Self {
			match error {
				StorageError::DidNotPresent => Self::DidNotPresent,
				StorageError::DidAlreadyPresent => Self::DidAlreadyPresent,
				StorageError::DidKeyNotPresent(_) => Self::VerificationKeyNotPresent,
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Stores a new DID on chain, after verifying the signature associated
		/// with the creation operation. The parameters are:
		/// * origin: the Substrate account submitting the transaction (which
		///   can be different from the DID subject)
		/// * did_creation_operation: a DidCreationOperation which contains the
		///   details of the new DID
		/// * signature: a signature over DidCreationOperation that must be
		///   signed with the authentication key associated with the new DID
		#[pallet::weight(<T as Config>::WeightInfo::submit_did_create_operation())]
		pub fn submit_did_create_operation(
			origin: OriginFor<T>,
			did_creation_operation: DidCreationOperation<T::DidIdentifier>,
			signature: DidSignature,
		) -> DispatchResultWithPostInfo {
			// origin of the transaction needs to be a signed sender account
			let sender = ensure_signed(origin)?;

			// There has to be no other DID with the same identifier already saved on chain,
			// otherwise generate a DidAlreadyPresent error.
			ensure!(
				!<Did<T>>::contains_key(did_creation_operation.get_did()),
				<Error<T>>::DidAlreadyPresent
			);

			// Create a new DID entry from the details provided in the create operation.
			let did_entry = DidDetails::from(did_creation_operation.clone());

			// Retrieve the authentication key of the new DID, otherwise generate a
			// VerificationKeyNotPresent error if it is not specified (should never happen
			// as the DIDCreateOperation requires the authentication key to be present).
			let signature_verification_key = did_entry
				.get_verification_key_for_key_type(DidVerificationKeyType::Authentication)
				.ok_or(<Error<T>>::VerificationKeyNotPresent)?;

			// Re-create a Signature object from the authentication key retrieved, or
			// generate a InvalidSignatureFormat error otherwise.
			let is_signature_valid = signature_verification_key
				.verify_signature(&did_creation_operation.encode(), &signature)
				.map_err(<Error<T>>::from)?;

			// Verify the validity of the signature, or generate an InvalidSignature error
			// otherwise.
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
	/// Verify the signature of a generic DidOperation, and
	/// returns either Ok or a DidError. The paremeters are:
	/// * op: a reference to the DID operation
	/// * signature: a reference to the signature
	pub fn verify_did_operation_signature<O: DidOperation<T::DidIdentifier>>(
		op: &O,
		signature: &DidSignature,
	) -> Result<(), DidError> {
		// Try to retrieve from the storage the details of the given DID. If there is no
		// DID stored, generate a DidNotPresent error.
		let did_entry: DidDetails =
			<Did<T>>::get(op.get_did()).ok_or(DidError::StorageError(StorageError::DidNotPresent))?;

		// Retrieves the needed verification key from the DID details, or generate a
		// VerificationkeyNotPresent error if there is no key of the type required.
		let verification_key = did_entry
			.get_verification_key_for_key_type(op.get_verification_key_type())
			.ok_or_else(|| DidError::StorageError(StorageError::DidKeyNotPresent(op.get_verification_key_type())))?;

		// Verifies that the signature matches the expected format, otherwise generate
		// an InvalidSignatureFormat error.
		let is_signature_valid = verification_key
			.verify_signature(&op.encode(), &signature)
			.map_err(|_| DidError::SignatureError(SignatureError::InvalidSignatureFormat))?;

		ensure!(
			is_signature_valid,
			DidError::SignatureError(SignatureError::InvalidSignature)
		);

		Ok(())
	}
}
