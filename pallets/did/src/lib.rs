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

#[cfg(any(feature = "mock", test))]
pub mod mock;

#[cfg(any(feature = "runtime-benchmarks", test))]
pub mod benchmarking;

pub mod default_weights;

mod utils;

pub use default_weights::WeightInfo;

use codec::{Decode, Encode};

use core::ops::Sub;

use frame_support::{ensure, storage::types::StorageMap, Parameter};
use frame_system::{self, ensure_signed};
use sp_core::{ed25519, sr25519};
use sp_runtime::traits::Verify;
use sp_std::{collections::btree_set::BTreeSet, convert::TryFrom, fmt::Debug, prelude::Clone, str, vec::Vec};

pub use pallet::*;

/// The expected URI scheme for HTTP endpoints.
pub const HTTP_URI_SCHEME: &str = "http://";
/// The expected URI scheme for HTTPS endpoints.
pub const HTTPS_URI_SCHEME: &str = "https://";
/// The expected URI scheme for FTP endpoints.
pub const FTP_URI_SCHEME: &str = "ftp://";
/// The expected URI scheme for FTPS endpoints.
pub const FTPS_URI_SCHEME: &str = "ftps://";
/// The expected URI scheme for IPFS endpoints.
pub const IPFS_URI_SCHEME: &str = "ipfs://";

/// Reference to a payload of data of variable size.
pub type Payload = [u8];

/// Trait representing a public key under the control of a DID subject.
pub trait DidPublicKey {
	/// Returns the key method description as in the [DID specification](https://w3c.github.io/did-spec-registries/#verification-method-types).
	fn get_did_key_description(&self) -> &str;
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
	fn get_did_key_description(&self) -> &str {
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
	X25519([u8; 32]),
}

impl DidPublicKey for PublicEncryptionKey {
	fn get_did_key_description(&self) -> &str {
		// https://w3c.github.io/did-spec-registries/#x25519keyagreementkey2019
		"X25519KeyAgreementKey2019"
	}
}

/// All the errors that can be generated when validating a DID operation.
#[derive(Debug, Eq, PartialEq)]
pub enum DidError {
	StorageError(StorageError),
	SignatureError(SignatureError),
	UrlError(UrlError),
}

/// Error involving the pallet's storage.
#[derive(Debug, Eq, PartialEq)]
pub enum StorageError {
	/// The DID being created is already present on chain.
	DidAlreadyPresent,
	/// The expected DID cannot be found on chain.
	DidNotPresent,
	/// The given DID does not contain the right key to verify the signature of
	/// a DID operation.
	DidKeyNotPresent(DidVerificationKeyType),
	/// One or more verification keys referenced are not stored in the set of
	/// verification keys.
	VerificationKeysNotPresent(Vec<PublicVerificationKey>),
	/// The maximum supported value for the DID tx counter has been reached.
	/// No more operations with the DID are allowed.
	MaxTxCounterValue,
}

/// Error generated when validating a DID operation.
#[derive(Debug, Eq, PartialEq)]
pub enum SignatureError {
	/// The signature is not in the expected format the verification key
	/// expects.
	InvalidSignatureFormat,
	/// The signature is invalid for the payload and the verification key
	/// provided.
	InvalidSignature,
	/// The operation nonce is not equal to the current nonce + 1.
	InvalidNonce,
}

/// Error generated when validating a byte-encoded endpoint URL.
#[derive(Debug, Eq, PartialEq)]
pub enum UrlError {
	/// The URL specified is not ASCII-encoded.
	InvalidUrlEncoding,
	/// The URL specified is not properly formatted.
	InvalidUrlScheme,
}

/// A trait describing an operation that requires DID authentication.
pub trait DidOperation<T: Config>: Encode {
	/// Returns the type of the verification key to be used to validate the
	/// operation.
	fn get_verification_key_type(&self) -> DidVerificationKeyType;
	/// Returns the DID identifier of the subject.
	fn get_did(&self) -> &T::DidIdentifier;
	/// Returns the operation tx counter, used to protect against replay
	/// attacks.
	fn get_tx_counter(&self) -> u64;
}

/// A DID creation request. It contains the following values:
/// * The DID identifier being created (only Substrate addresses are allowed in
///   this version of the pallet)
/// * The new authentication key to use
/// * The new encryption key to use
/// * The optional attestation key to use
/// * The optional delegation key to use
/// * The optional endpoint URL pointing to the DID service endpoints
#[derive(Clone, Decode, Encode, PartialEq)]
pub struct DidCreationOperation<T: Config> {
	did: T::DidIdentifier,
	new_auth_key: PublicVerificationKey,
	new_key_agreement_key: PublicEncryptionKey,
	new_attestation_key: Option<PublicVerificationKey>,
	new_delegation_key: Option<PublicVerificationKey>,
	new_endpoint_url: Option<Url>,
}

impl<T: Config> DidOperation<T> for DidCreationOperation<T> {
	fn get_verification_key_type(&self) -> DidVerificationKeyType {
		DidVerificationKeyType::Authentication
	}

	fn get_did(&self) -> &T::DidIdentifier {
		&self.did
	}

	// Irrelevant for creation operations.
	fn get_tx_counter(&self) -> u64 {
		0u64
	}
}

// Required to use a struct as an extrinsic parameter, and since Config does not
// implement Debug, the derive macro does not work.
impl<T: Config> Debug for DidCreationOperation<T> {
	fn fmt(&self, f: &mut sp_std::fmt::Formatter<'_>) -> sp_std::fmt::Result {
		f.debug_tuple("DidCreationOperation")
			.field(&self.did)
			.field(&self.new_auth_key)
			.field(&self.new_key_agreement_key)
			.field(&self.new_attestation_key)
			.field(&self.new_delegation_key)
			.field(&self.new_endpoint_url)
			.finish()
	}
}

/// A DID update request. It contains the following values:
/// * The DID identifier being updated
/// * The optional new authentication key to use
/// * The optional new encryption key to use
/// * The optional new attestation key to use
/// * The optional new delegation key to use
/// * The optional set of old attestation keys to remove
/// * The optional new endpoint URL pointing to the DID service endpoints
/// * A counter used to protect against replay attacks
#[derive(Clone, Decode, Encode, PartialEq)]
pub struct DidUpdateOperation<T: Config> {
	did: T::DidIdentifier,
	new_auth_key: Option<PublicVerificationKey>,
	new_key_agreement_key: Option<PublicEncryptionKey>,
	new_attestation_key: Option<PublicVerificationKey>,
	new_delegation_key: Option<PublicVerificationKey>,
	verification_keys_to_remove: Option<BTreeSet<PublicVerificationKey>>,
	new_endpoint_url: Option<Url>,
	tx_counter: u64,
}

impl<T: Config> DidOperation<T> for DidUpdateOperation<T> {
	fn get_verification_key_type(&self) -> DidVerificationKeyType {
		DidVerificationKeyType::Authentication
	}

	fn get_did(&self) -> &T::DidIdentifier {
		&self.did
	}

	fn get_tx_counter(&self) -> u64 {
		self.tx_counter
	}
}

// Required to use a struct as an extrinsic parameter, and since Config does not
// implement Debug, the derive macro does not work.
impl<T: Config> Debug for DidUpdateOperation<T> {
	fn fmt(&self, f: &mut sp_std::fmt::Formatter<'_>) -> sp_std::fmt::Result {
		f.debug_tuple("DidUpdateOperation")
			.field(&self.did)
			.field(&self.new_auth_key)
			.field(&self.new_key_agreement_key)
			.field(&self.new_attestation_key)
			.field(&self.new_delegation_key)
			.field(&self.verification_keys_to_remove)
			.field(&self.new_endpoint_url)
			.field(&self.tx_counter)
			.finish()
	}
}

/// A DID deletion request. It contains the following values:
/// * The DID identifier being deleted
/// * A counter used to protect against replay attacks
#[derive(Clone, Decode, Encode, PartialEq)]
pub struct DidDeletionOperation<T: Config> {
	did: T::DidIdentifier,
	tx_counter: u64,
}

impl<T: Config> DidOperation<T> for DidDeletionOperation<T> {
	fn get_verification_key_type(&self) -> DidVerificationKeyType {
		DidVerificationKeyType::Authentication
	}

	fn get_did(&self) -> &T::DidIdentifier {
		&self.did
	}

	fn get_tx_counter(&self) -> u64 {
		self.tx_counter
	}
}

// Required to use a struct as an extrinsic parameter, and since Config does not
// implement Debug, the derive macro does not work.
impl<T: Config> Debug for DidDeletionOperation<T> {
	fn fmt(&self, f: &mut sp_std::fmt::Formatter<'_>) -> sp_std::fmt::Result {
		f.debug_tuple("DidDeletionOperation")
			.field(&self.did)
			.field(&self.tx_counter)
			.finish()
	}
}

/// A web URL starting with either http:// or https://
/// and containing only ASCII URL-encoded characters.
#[derive(Clone, Decode, Debug, Encode, PartialEq)]
pub struct HttpUrl {
	payload: Vec<u8>,
}

impl TryFrom<&[u8]> for HttpUrl {
	type Error = UrlError;

	// It fails if the byte sequence does not result in an ASCII-encoded string or
	// if the resulting string contains characters that are not allowed in a URL.
	fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
		let str_url = str::from_utf8(value).map_err(|_| UrlError::InvalidUrlEncoding)?;

		ensure!(
			str_url.starts_with(HTTP_URI_SCHEME) || str_url.starts_with(HTTPS_URI_SCHEME),
			UrlError::InvalidUrlScheme
		);

		ensure!(utils::is_valid_ascii_url(&str_url), UrlError::InvalidUrlEncoding);

		Ok(HttpUrl {
			payload: value.to_vec(),
		})
	}
}

/// An FTP URL starting with ftp:// or ftps://
/// and containing only ASCII URL-encoded characters.
#[derive(Clone, Decode, Debug, Encode, PartialEq)]
pub struct FtpUrl {
	payload: Vec<u8>,
}

impl TryFrom<&[u8]> for FtpUrl {
	type Error = UrlError;

	// It fails if the byte sequence does not result in an ASCII-encoded string or
	// if the resulting string contains characters that are not allowed in a URL.
	fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
		let str_url = str::from_utf8(value).map_err(|_| UrlError::InvalidUrlEncoding)?;

		ensure!(
			str_url.starts_with(FTP_URI_SCHEME) || str_url.starts_with(FTPS_URI_SCHEME),
			UrlError::InvalidUrlScheme
		);

		ensure!(utils::is_valid_ascii_url(&str_url), UrlError::InvalidUrlEncoding);

		Ok(FtpUrl {
			payload: value.to_vec(),
		})
	}
}

/// An IPFS URL starting with ipfs://. Both CIDs v0 and v1 supported.
#[derive(Clone, Decode, Debug, Encode, PartialEq)]
pub struct IpfsUrl {
	payload: Vec<u8>,
}

impl TryFrom<&[u8]> for IpfsUrl {
	type Error = UrlError;

	// It fails if the URL is not ASCII-encoded or does not start with the expected
	// URL scheme.
	fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
		let str_url = str::from_utf8(value).map_err(|_| UrlError::InvalidUrlEncoding)?;

		ensure!(str_url.starts_with(IPFS_URI_SCHEME), UrlError::InvalidUrlScheme);

		// Remove the characters of the URL scheme
		let slice_to_verify = str_url
			.get(IPFS_URI_SCHEME.len()..)
			.expect("The minimum length was ensured with starts_with.");

		// Verify the rest are either only base58 or only base32 characters (according
		// to the IPFS specification, respectively versions 0 and 1).
		ensure!(
			utils::is_base_32(slice_to_verify) || utils::is_base_58(slice_to_verify),
			UrlError::InvalidUrlEncoding
		);

		Ok(IpfsUrl {
			payload: value.to_vec(),
		})
	}
}

/// Enum representing one of the supported URLs.
#[derive(Clone, Decode, Debug, Encode, PartialEq)]
pub enum Url {
	Http(HttpUrl),
	Ftp(FtpUrl),
	Ipfs(IpfsUrl),
}

impl From<HttpUrl> for Url {
	fn from(url: HttpUrl) -> Self {
		Self::Http(url)
	}
}

impl From<FtpUrl> for Url {
	fn from(url: FtpUrl) -> Self {
		Self::Ftp(url)
	}
}

impl From<IpfsUrl> for Url {
	fn from(url: IpfsUrl) -> Self {
		Self::Ipfs(url)
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
///   each DID operation execution
#[derive(Clone, Debug, Decode, Encode, PartialEq)]
pub struct DidDetails {
	pub auth_key: PublicVerificationKey,
	pub key_agreement_key: PublicEncryptionKey,
	pub delegation_key: Option<PublicVerificationKey>,
	pub attestation_key: Option<PublicVerificationKey>,
	pub verification_keys: BTreeSet<PublicVerificationKey>,
	pub endpoint_url: Option<Url>,
	last_tx_counter: u64,
}

impl DidDetails {
	pub fn increase_tx_counter(&mut self) -> Result<(), StorageError> {
		self.last_tx_counter = self
			.last_tx_counter
			.checked_add(1)
			.ok_or(StorageError::MaxTxCounterValue)?;
		Ok(())
	}

	pub fn get_tx_counter_value(&self) -> u64 {
		self.last_tx_counter
	}

	#[cfg(any(feature = "mock", test))]
	pub fn set_tx_counter(&mut self, value: u64) {
		self.last_tx_counter = value;
	}
}

impl<T: Config> From<DidCreationOperation<T>> for DidDetails {
	fn from(op: DidCreationOperation<T>) -> Self {
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

/// Generates a new DID entry starting from the current one stored in the
/// storage and by applying the changes in the DidUpdateOperation. The operation
/// fails with a DidError if the update operation instructs to delete a
/// verification key that is not associated with the DID.
///
/// Please note that this method does not perform any checks regarding
/// the validity of the DidUpdateOperation signature nor whether the nonce
/// provided is valid.
impl<T: Config> TryFrom<(DidDetails, DidUpdateOperation<T>)> for DidDetails {
	type Error = DidError;

	fn try_from((old_details, update_operation): (DidDetails, DidUpdateOperation<T>)) -> Result<Self, Self::Error> {
		// Old attestation key is used later in the process, so it's saved here.
		let old_attestation_key = old_details.attestation_key;
		// Copy old state into new, and apply changes in operation to new state.
		let mut new_details = old_details;

		if let Some(verification_keys_to_remove) = update_operation.verification_keys_to_remove.as_ref() {
			// Verify that the set of keys to delete - the set of keys stored is empty
			// (otherwise keys to delete contains some keys not stored on chain -> notify
			// about them to the caller)
			let keys_not_present = verification_keys_to_remove.sub(&new_details.verification_keys);
			ensure!(
				keys_not_present.is_empty(),
				DidError::StorageError(StorageError::VerificationKeysNotPresent(
					keys_not_present.iter().copied().collect()
				))
			);
		};

		// Updates keys, endpoint and tx counter.
		if let Some(new_auth_key) = update_operation.new_auth_key {
			new_details.auth_key = new_auth_key;
		}
		if let Some(new_enc_key) = update_operation.new_key_agreement_key {
			new_details.key_agreement_key = new_enc_key;
		}
		if let Some(new_attestation_key) = update_operation.new_attestation_key {
			if let Some(old_attestation_key) = old_attestation_key {
				new_details.verification_keys.insert(old_attestation_key);
			}
			new_details.attestation_key = Some(new_attestation_key);
		}
		if let Some(new_delegation_key) = update_operation.new_delegation_key {
			new_details.delegation_key = Some(new_delegation_key);
		}
		if let Some(new_endpoint_url) = update_operation.new_endpoint_url {
			new_details.endpoint_url = Some(new_endpoint_url);
		}
		if let Some(verification_keys_to_remove) = update_operation.verification_keys_to_remove.as_ref() {
			new_details.verification_keys = new_details.verification_keys.sub(verification_keys_to_remove);
		}
		new_details.last_tx_counter = update_operation.tx_counter;

		Ok(new_details)
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
		DidCreated(T::AccountId, T::DidIdentifier),
		DidUpdated(T::AccountId, T::DidIdentifier),
		DidDeleted(T::AccountId, T::DidIdentifier),
	}

	#[pallet::error]
	pub enum Error<T> {
		InvalidSignatureFormat,
		InvalidSignature,
		DidAlreadyPresent,
		DidNotPresent,
		VerificationKeysNotPresent,
		InvalidNonce,
		InvalidUrlEncoding,
		InvalidUrlScheme,
		MaxTxCounterValue,
	}

	impl<T> From<DidError> for Error<T> {
		fn from(error: DidError) -> Self {
			match error {
				DidError::StorageError(storage_error) => Self::from(storage_error),
				DidError::SignatureError(operation_error) => Self::from(operation_error),
				DidError::UrlError(url_error) => Self::from(url_error),
			}
		}
	}

	impl<T> From<StorageError> for Error<T> {
		fn from(error: StorageError) -> Self {
			match error {
				StorageError::DidNotPresent => Self::DidNotPresent,
				StorageError::DidAlreadyPresent => Self::DidAlreadyPresent,
				StorageError::DidKeyNotPresent(_) | StorageError::VerificationKeysNotPresent(_) => {
					Self::VerificationKeysNotPresent
				}
				StorageError::MaxTxCounterValue => Self::MaxTxCounterValue,
			}
		}
	}

	impl<T> From<SignatureError> for Error<T> {
		fn from(error: SignatureError) -> Self {
			match error {
				SignatureError::InvalidSignature => Self::InvalidSignature,
				SignatureError::InvalidSignatureFormat => Self::InvalidSignatureFormat,
				SignatureError::InvalidNonce => Self::InvalidNonce,
			}
		}
	}

	impl<T> From<UrlError> for Error<T> {
		fn from(error: UrlError) -> Self {
			match error {
				UrlError::InvalidUrlEncoding => Self::InvalidUrlEncoding,
				UrlError::InvalidUrlScheme => Self::InvalidUrlScheme,
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Stores a new DID on chain, after verifying the signature associated
		/// with the creation operation. The parameters are:
		/// * origin: the Substrate account submitting the transaction (which
		///   can be different from the DID subject)
		/// * operation: a DidCreationOperation which contains the details of
		///   the new DID
		/// * signature: a signature over DidCreationOperation that must be
		///   signed with the authentication key associated with the new DID
		#[pallet::weight(T::WeightInfo::submit_did_create_operation())]
		pub fn submit_did_create_operation(
			origin: OriginFor<T>,
			operation: DidCreationOperation<T>,
			signature: DidSignature,
		) -> DispatchResultWithPostInfo {
			// origin of the transaction needs to be a signed sender account
			let sender = ensure_signed(origin)?;

			// There has to be no other DID with the same identifier already saved on chain,
			// otherwise generate a DidAlreadyPresent error.
			ensure!(
				!<Did<T>>::contains_key(operation.get_did()),
				<Error<T>>::DidAlreadyPresent
			);

			// Create a new DID entry from the details provided in the create operation.
			let did_entry = DidDetails::from(operation.clone());

			Self::verify_payload_signature_with_did_key_type(
				&operation.encode(),
				&signature,
				&did_entry,
				operation.get_verification_key_type(),
			)
			.map_err(<Error<T>>::from)?;

			let did_identifier = operation.get_did();
			log::debug!("Creating DID {:?}", did_identifier);
			<Did<T>>::insert(did_identifier, did_entry);

			Self::deposit_event(Event::DidCreated(sender, did_identifier.clone()));
			Ok(().into())
		}

		/// Updates the information associated with a DID on chain, after
		/// verifying the signature associated with the operation. The
		/// parameters are:
		/// * origin: the Substrate account submitting the transaction (which
		///   can be different from the DID subject)
		/// * operation: a DidUpdateOperation which contains the new details of
		///   the given DID
		/// * signature: a signature over the operation that must be signed with
		///   the authentication key associated with the new DID. In case the
		///   authentication key is being updated, the key used to verify is the
		///   old one that is getting updated.
		#[pallet::weight(T::WeightInfo::submit_did_update_operation())]
		pub fn submit_did_update_operation(
			origin: OriginFor<T>,
			operation: DidUpdateOperation<T>,
			signature: DidSignature,
		) -> DispatchResultWithPostInfo {
			// origin of the transaction needs to be a signed sender account
			let sender = ensure_signed(origin)?;

			// Saved here as it is consumed later when generating the new DidDetails object.
			let did_identifier = operation.get_did().clone();

			// If specified DID does not exist, generate a DidNotPresent error.
			let did_details = <Did<T>>::get(&did_identifier).ok_or(<Error<T>>::DidNotPresent)?;

			// Verify the signature and the nonce of the update operation.
			Self::verify_operation_validity_for_did(&operation, &signature, &did_details).map_err(<Error<T>>::from)?;

			// Generate a new DidDetails object by applying the changes in the update
			// operation to the old object (and consuming both).
			let new_did_details = DidDetails::try_from((did_details, operation)).map_err(<Error<T>>::from)?;

			log::debug!("Updating DID {:?}", did_identifier);
			<Did<T>>::insert(&did_identifier, new_did_details);

			Self::deposit_event(Event::DidUpdated(sender, did_identifier));
			Ok(().into())
		}

		/// Deletes all the information associated with a DID on chain, after
		/// verifying the signature associated with the operation. The
		/// parameters are:
		/// * origin: the Substrate account submitting the transaction (which
		///   can be different from the DID subject)
		/// * operation: a DidDeletionOperation which includes the DID to
		///   deactivate
		/// * signature: a signature over the operation that must be signed with
		///   the authentication key associated with the new DID.
		#[pallet::weight(T::WeightInfo::submit_did_delete_operation())]
		pub fn submit_did_delete_operation(
			origin: OriginFor<T>,
			operation: DidDeletionOperation<T>,
			signature: DidSignature,
		) -> DispatchResultWithPostInfo {
			// origin of the transaction needs to be a signed sender account
			let sender = ensure_signed(origin)?;

			let did_identifier = operation.get_did();

			// If specified DID does not exist, generate a DidNotPresent error.
			let did_details = <Did<T>>::get(&did_identifier).ok_or(<Error<T>>::DidNotPresent)?;

			// Verify the signature and the nonce of the delete operation.
			Self::verify_operation_validity_for_did(&operation, &signature, &did_details).map_err(<Error<T>>::from)?;

			log::debug!("Deleting DID {:?}", did_identifier);
			<Did<T>>::remove(&did_identifier);

			Self::deposit_event(Event::DidDeleted(sender, did_identifier.clone()));
			Ok(().into())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Verify the validity (i.e., nonce and signature) of a generic
	/// DidOperation and, if valid, update the DID state with the latest nonce.
	/// The parameters are:
	/// * operation: the reference to the operation which validity is to be
	///   verified
	/// * signature: a reference to the signature
	/// * did: the identifier to verify the operation signature for
	pub fn verify_operation_validity_and_increase_did_nonce<O: DidOperation<T>>(
		operation: &O,
		signature: &DidSignature,
	) -> Result<(), DidError> {
		let mut did_details =
			<Did<T>>::get(&operation.get_did()).ok_or(DidError::StorageError(StorageError::DidNotPresent))?;

		Self::verify_operation_validity_for_did(operation, &signature, &did_details)?;

		// Update tx counter in DID details and save to DID pallet
		did_details
			.increase_tx_counter()
			.expect("Increasing DID tx counter should be a safe operation.");
		<Did<T>>::insert(&operation.get_did(), did_details);

		Ok(())
	}

	// Internally verifies operation nonce and operation signature
	fn verify_operation_validity_for_did<O: DidOperation<T>>(
		operation: &O,
		signature: &DidSignature,
		did_details: &DidDetails,
	) -> Result<(), DidError> {
		Self::verify_operation_counter_for_did(operation, did_details)?;
		Self::verify_payload_signature_with_did_key_type(
			&operation.encode(),
			signature,
			did_details,
			operation.get_verification_key_type(),
		)
	}

	// Perform operation nonce validation.
	fn verify_operation_counter_for_did<O: DidOperation<T>>(
		operation: &O,
		did_details: &DidDetails,
	) -> Result<(), DidError> {
		// Verify that the DID has not reached the maximum tx counter value
		ensure!(
			did_details.last_tx_counter < u64::MAX,
			DidError::StorageError(StorageError::MaxTxCounterValue)
		);

		// Verify that the operation counter is equal to the stored one + 1.
		let expected_nonce_value = did_details
			.last_tx_counter
			.checked_add(1)
			.expect("Checked against overflow in previous step.");
		ensure!(
			operation.get_tx_counter() == expected_nonce_value,
			DidError::SignatureError(SignatureError::InvalidNonce)
		);

		Ok(())
	}

	// Verifies a generic payload signature
	pub fn verify_payload_signature_with_did_key_type(
		payload: &Payload,
		signature: &DidSignature,
		did_details: &DidDetails,
		key_type: DidVerificationKeyType,
	) -> Result<(), DidError> {
		// Retrieves the needed verification key from the DID details, or generate a
		// VerificationkeyNotPresent error if there is no key of the type required.
		let verification_key = did_details
			.get_verification_key_for_key_type(key_type.clone())
			.ok_or(DidError::StorageError(StorageError::DidKeyNotPresent(key_type)))?;

		// Verifies that the signature matches the expected format, otherwise generate
		// an InvalidSignatureFormat error.
		let is_signature_valid = verification_key
			.verify_signature(&payload, &signature)
			.map_err(|_| DidError::SignatureError(SignatureError::InvalidSignatureFormat))?;

		ensure!(
			is_signature_valid,
			DidError::SignatureError(SignatureError::InvalidSignature)
		);

		Ok(())
	}
}
