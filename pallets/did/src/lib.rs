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
#![allow(clippy::unused_unit)]

#[cfg(test)]
mod tests;

#[cfg(any(feature = "mock", test))]
pub mod mock;

pub mod default_weights;

mod utils;

pub use default_weights::WeightInfo;

use codec::{Decode, Encode};

use frame_support::{ensure, storage::types::StorageMap, Parameter};
use frame_system::{self, ensure_signed};
use sp_core::{ed25519, sr25519};
use sp_runtime::traits::Verify;
use sp_std::{
	collections::{btree_map::BTreeMap, btree_set::BTreeSet},
	convert::TryFrom,
	fmt::Debug,
	prelude::Clone,
	str,
	vec::Vec,
};

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

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

	/// Type for a DID key identifier.
	pub type KeyId<T> = <T as frame_system::Config>::Hash;

	/// Type for a block number.
	pub type BlockNumber<T> = <T as frame_system::Config>::BlockNumber;

	/// The string description of a DID public key.
	///
	/// The description must follow the [DID specification](https://w3c.github.io/did-spec-registries/#verification-method-types).
	pub trait DidPublicKeyDescription {
		fn get_did_key_description(&self) -> &str;
	}

	/// Types of verification keys a DID can control.
	#[derive(Clone, Copy, Decode, Debug, Encode, Eq, Ord, PartialEq, PartialOrd)]
	pub enum DidVerificationKey {
		/// An Ed25519 public key.
		Ed25519(ed25519::Public),
		/// A Sr25519 public key.
		Sr25519(sr25519::Public),
	}

	impl DidVerificationKey {
		/// Verify a DID signature using one of the DID keys.
		pub fn verify_signature(&self, payload: &Payload, signature: &DidSignature) -> Result<bool, SignatureError> {
			match self {
				DidVerificationKey::Ed25519(public_key) => {
					// Try to re-create a Signature value or throw an error if raw value is invalid
					if let DidSignature::Ed25519(sig) = signature {
						Ok(sig.verify(payload, &public_key))
					} else {
						Err(SignatureError::InvalidSignatureFormat)
					}
				}
				// Follows same process as above, but using a Sr25519 instead
				DidVerificationKey::Sr25519(public_key) => {
					if let DidSignature::Sr25519(sig) = signature {
						Ok(sig.verify(payload, &public_key))
					} else {
						Err(SignatureError::InvalidSignatureFormat)
					}
				}
			}
		}
	}

	impl DidPublicKeyDescription for DidVerificationKey {
		fn get_did_key_description(&self) -> &str {
			match self {
				// https://w3c.github.io/did-spec-registries/#ed25519verificationkey2018
				DidVerificationKey::Ed25519(_) => "Ed25519VerificationKey2018",
				// Not yet defined in the DID specification.
				DidVerificationKey::Sr25519(_) => "Sr25519VerificationKey2020",
			}
		}
	}

	impl From<ed25519::Public> for DidVerificationKey {
		fn from(key: ed25519::Public) -> Self {
			DidVerificationKey::Ed25519(key)
		}
	}

	impl From<sr25519::Public> for DidVerificationKey {
		fn from(key: sr25519::Public) -> Self {
			DidVerificationKey::Sr25519(key)
		}
	}

	/// Types of encryption keys a DID can control.
	#[derive(Clone, Copy, Decode, Debug, Encode, Eq, Ord, PartialEq, PartialOrd)]
	pub enum DidEncryptionKey {
		/// An X25519 public key.
		X25519([u8; 32]),
	}

	impl DidPublicKeyDescription for DidEncryptionKey {
		fn get_did_key_description(&self) -> &str {
			// https://w3c.github.io/did-spec-registries/#x25519keyagreementkey2019
			"X25519KeyAgreementKey2019"
		}
	}

	/// A general public key under the control of the DID.
	#[derive(Clone, Copy, Decode, Debug, Encode, Eq, Ord, PartialEq, PartialOrd)]
	pub enum DidPublicKey {
		/// A verification key, used to generate and verify signatures.
		PublicVerificationKey(DidVerificationKey),
		/// An encryption key, used to encrypt and decrypt payloads.
		PublicEncryptionKey(DidEncryptionKey),
	}

	impl From<DidVerificationKey> for DidPublicKey {
		fn from(verification_key: DidVerificationKey) -> Self {
			Self::PublicVerificationKey(verification_key)
		}
	}

	impl From<DidEncryptionKey> for DidPublicKey {
		fn from(encryption_key: DidEncryptionKey) -> Self {
			Self::PublicEncryptionKey(encryption_key)
		}
	}

	/// Verification methods a verification key can
	/// fulfil, according to the [DID specification](https://w3c.github.io/did-spec-registries/#verification-relationships).
	#[derive(Clone, Debug, Decode, Encode, PartialEq, Eq)]
	pub enum DidVerificationKeyRelationship {
		/// Key used to authenticate all the DID operations.
		Authentication,
		/// Key used to write and revoke delegations on chain.
		CapabilityDelegation,
		/// Not used for now.
		CapabilityInvocation,
		/// Key used to write and revoke attestations on chain.
		AssertionMethod,
	}

	/// Types of signatures supported by this pallet.
	#[derive(Clone, Decode, Debug, Encode, Eq, PartialEq)]
	pub enum DidSignature {
		/// A Ed25519 signature.
		Ed25519(ed25519::Signature),
		/// A Sr25519 signature.
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

	/// All the errors that can be generated when validating a DID operation.
	#[derive(Debug, Eq, PartialEq)]
	pub enum DidError {
		/// See [StorageError].
		StorageError(StorageError),
		/// See [SignatureError].
		SignatureError(SignatureError),
		/// See [UrlError].
		UrlError(UrlError),
		/// An error that is not supposed to take place, yet it happened.
		InternalError,
	}

	/// Error involving the pallet's storage.
	#[derive(Debug, Eq, PartialEq)]
	pub enum StorageError {
		/// The DID being created is already present on chain.
		DidAlreadyPresent,
		/// The expected DID cannot be found on chain.
		DidNotPresent,
		/// The given DID does not contain the right key to verify the signature
		/// of a DID operation.
		DidKeyNotPresent(DidVerificationKeyRelationship),
		/// At least one verification key referenced is not stored in the set
		/// of verification keys.
		VerificationKeyNotPresent,
		/// The user tries to delete a verification key that is currently being
		/// used to authorize operations, and this is not allowed.
		CurrentlyActiveKey,
		/// The maximum supported value for the DID tx counter has been reached.
		/// No more operations with the DID are allowed.
		MaxTxCounterValue,
	}

	/// Error generated when validating a DID operation.
	#[derive(Debug, Eq, PartialEq)]
	pub enum SignatureError {
		/// The signature is not in the format the verification key expects.
		InvalidSignatureFormat,
		/// The signature is invalid for the payload and the verification key
		/// provided.
		InvalidSignature,
		/// The operation nonce is not equal to the current DID nonce + 1.
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

	/// Details of a public key, which includes the key value and the
	/// block number at which it was set.
	///
	/// It is currently used to keep track of all the past and current
	/// attestation keys a DID might control.
	#[derive(Clone, Debug, Decode, Encode, PartialEq)]
	pub struct DidPublicKeyDetails<T: Config> {
		/// A public key the DID controls.
		pub key: DidPublicKey,
		/// The block number in which the verification key was added to the DID.
		pub block_number: BlockNumber<T>,
	}

	/// The details associated to a DID identity.
	#[derive(Clone, Debug, Decode, Encode, PartialEq)]
	pub struct DidDetails<T: Config> {
		/// The ID of the authentication key, used to authenticate DID-related
		/// operations.
		authentication_key: KeyId<T>,
		/// The set of the key agreement key IDs, which can be used to encrypt
		/// data addressed to the DID subject.
		key_agreement_keys: BTreeSet<KeyId<T>>,
		/// \[OPTIONAL\] The ID of the delegation key, used to verify the
		/// signatures of the delegations created by the DID subject.
		delegation_key: Option<KeyId<T>>,
		/// \[OPTIONAL\] The ID of the attestation key, used to verify the
		/// signatures of the attestations created by the DID subject.
		attestation_key: Option<KeyId<T>>,
		/// The map of public keys, with the key label as
		/// the key map and the tuple (key, addition_block_number) as the map
		/// value.
		/// The map includes all the keys under the control of the DID subject,
		/// including the ones currently used for authentication, key agreement,
		/// attestation, and delegation. Other than those, the map also contains
		/// the old attestation keys that have been rotated, i.e., they cannot
		/// be used to create new attestations but can still be used to verify
		/// previously issued attestations.
		public_keys: BTreeMap<KeyId<T>, DidPublicKeyDetails<T>>,
		/// \[OPTIONAL\] The URL pointing to the service endpoints the DID
		/// subject publicly exposes.
		pub endpoint_url: Option<Url>,
		/// The counter used to avoid replay attacks, which is checked and
		/// updated upon each DID operation involving with the subject as the
		/// creator.
		pub(crate) last_tx_counter: u64,
	}

	impl<T: Config> DidDetails<T> {
		/// Creates a new instance of DID details with the minimum information,
		/// i.e., an authentication key and the block creation time.
		///
		/// The tx counter is set by default to 0.
		pub fn new(authentication_key: DidVerificationKey, block_number: BlockNumber<T>) -> Self {
			let mut public_keys: BTreeMap<KeyId<T>, DidPublicKeyDetails<T>> = BTreeMap::new();
			let authentication_key_id = utils::calculate_key_id::<T>(&authentication_key.into());
			public_keys.insert(
				authentication_key_id,
				DidPublicKeyDetails {
					key: authentication_key.into(),
					block_number,
				},
			);
			Self {
				authentication_key: authentication_key_id,
				key_agreement_keys: BTreeSet::new(),
				attestation_key: None,
				delegation_key: None,
				endpoint_url: None,
				public_keys,
				last_tx_counter: 0u64,
			}
		}

		/// Update the DID authentication key.
		///
		/// The old key is deleted from the set of verification keys if it is
		/// not used in any other part of the DID. The new key is added to the
		/// set of verification keys.
		pub fn update_authentication_key(
			&mut self,
			new_authentication_key: DidVerificationKey,
			block_number: BlockNumber<T>,
		) {
			let old_authentication_key_id = self.authentication_key;
			let new_authentication_key_id = utils::calculate_key_id::<T>(&new_authentication_key.into());
			self.authentication_key = new_authentication_key_id;
			// Remove old key ID from public keys, if not used anymore.
			self.remove_key_if_unused(&old_authentication_key_id);
			// Add new key ID to public keys. If a key with the same ID is already present,
			// the result is simply that the block number is updated.
			self.public_keys.insert(
				new_authentication_key_id,
				DidPublicKeyDetails {
					key: new_authentication_key.into(),
					block_number,
				},
			);
		}

		/// Add new key agreement keys to the DID.
		///
		/// The new keys are added to the set of verification keys.
		pub fn add_key_agreement_keys(
			&mut self,
			new_key_agreement_keys: BTreeSet<DidEncryptionKey>,
			block_number: BlockNumber<T>,
		) {
			for new_key_agreement_key in new_key_agreement_keys {
				let new_key_agreement_id = utils::calculate_key_id::<T>(&new_key_agreement_key.into());
				self.public_keys.insert(
					new_key_agreement_id,
					DidPublicKeyDetails {
						key: new_key_agreement_key.into(),
						block_number,
					},
				);
				self.key_agreement_keys.insert(new_key_agreement_id);
			}
		}

		/// Update the DID attestation key.
		///
		/// The old key is not removed from the set of verification keys, hence
		/// it can still be used to verify past attestations.
		/// The new key is added to the set of verification keys.
		pub fn update_attestation_key(
			&mut self,
			new_attestation_key: DidVerificationKey,
			block_number: BlockNumber<T>,
		) {
			let new_attestation_key_id = utils::calculate_key_id::<T>(&new_attestation_key.into());
			self.attestation_key = Some(new_attestation_key_id);
			self.public_keys.insert(
				new_attestation_key_id,
				DidPublicKeyDetails {
					key: new_attestation_key.into(),
					block_number,
				},
			);
		}

		/// Delete the DID attestation key.
		///
		/// Once deleted, it cannot be used to write new attestations anymore.
		/// The key is also removed from the set of verification keys if it not
		/// used anywhere else in the DID.
		pub fn delete_attestation_key(&mut self) {
			if let Some(old_attestation_key_id) = self.attestation_key {
				self.attestation_key = None;
				self.remove_key_if_unused(&old_attestation_key_id);
			}
		}

		/// Update the DID delegation key.
		///
		/// The old key is deleted from the set of verification keys if it is
		/// not used in any other part of the DID. The new key is added to the
		/// set of verification keys.
		pub fn update_delegation_key(&mut self, new_delegation_key: DidVerificationKey, block_number: BlockNumber<T>) {
			let old_delegation_key_id = self.delegation_key;
			let new_delegation_key_id = utils::calculate_key_id::<T>(&new_delegation_key.into());
			self.delegation_key = Some(new_delegation_key_id);
			if let Some(old_delegation_key) = old_delegation_key_id {
				self.remove_key_if_unused(&old_delegation_key);
			}
			self.public_keys.insert(
				new_delegation_key_id,
				DidPublicKeyDetails {
					key: new_delegation_key.into(),
					block_number,
				},
			);
		}

		/// Delete the DID delegation key.
		///
		/// It cannot be used to write new delegations anymore.
		/// The key is also removed from the set of verification keys if it not
		/// used anywhere else in the DID.
		pub fn delete_delegation_key(&mut self) {
			if let Some(old_delegation_key_id) = self.delegation_key {
				self.delegation_key = None;
				self.remove_key_if_unused(&old_delegation_key_id);
			}
		}

		/// Deletes a public key from the set of public keys stored on chain.
		/// Additionally, if the public key to remove is among the key agreement
		/// keys, it also eliminates it from there.
		///
		/// When deleting a public key, the following conditions are verified:
		/// - 1. the set of keys to delete does not contain any of the currently
		///   active verification keys, i.e., authentication, attestation, and
		///   delegation key, i.e., only key agreement keys and past attestation
		///   keys can be deleted.
		/// - 2. the set of keys to delete contains key IDs that are not
		///   currently stored on chain
		fn remove_public_keys(&mut self, key_ids: &BTreeSet<KeyId<T>>) -> Result<(), StorageError> {
			// Consider the currently active authentication, attestation, and delegation key
			// as forbidden to delete. They can be deleted with the right operation for the
			// respective fields in the DidUpdateOperation.
			let mut forbidden_verification_key_ids = BTreeSet::new();
			forbidden_verification_key_ids.insert(self.authentication_key);
			if let Some(attestation_key_id) = self.attestation_key {
				forbidden_verification_key_ids.insert(attestation_key_id);
			}
			if let Some(delegation_key_id) = self.delegation_key {
				forbidden_verification_key_ids.insert(delegation_key_id);
			}

			for key_id in key_ids.iter() {
				// Check for condition 1.
				ensure!(
					!forbidden_verification_key_ids.contains(key_id),
					StorageError::CurrentlyActiveKey
				);
				// Check for condition 2.
				self.public_keys
					.remove(key_id)
					.ok_or(StorageError::VerificationKeyNotPresent)?;
				// Also remove from the set of key agreement keys, if present.
				self.key_agreement_keys.remove(key_id);
			}

			Ok(())
		}

		// Remove a key from the map of public keys if none of the other keys, i.e.,
		// authentication, key agreement, attestation, or delegation, is referencing it.
		fn remove_key_if_unused(&mut self, key_id: &KeyId<T>) {
			if self.authentication_key != *key_id
				&& self.attestation_key != Some(*key_id)
				&& self.delegation_key != Some(*key_id)
				&& !self.key_agreement_keys.contains(key_id)
			{
				self.public_keys.remove(key_id);
			}
		}

		pub fn get_authentication_key_id(&self) -> KeyId<T> {
			self.authentication_key
		}

		pub fn get_key_agreement_keys_ids(&self) -> &BTreeSet<KeyId<T>> {
			&self.key_agreement_keys
		}

		pub fn get_attestation_key_id(&self) -> &Option<KeyId<T>> {
			&self.attestation_key
		}

		pub fn get_delegation_key_id(&self) -> &Option<KeyId<T>> {
			&self.delegation_key
		}

		pub fn get_public_keys(&self) -> &BTreeMap<KeyId<T>, DidPublicKeyDetails<T>> {
			&self.public_keys
		}

		/// Returns a reference to a specific verification key given the type of
		/// the key needed.
		pub fn get_verification_key_for_key_type(
			&self,
			key_type: DidVerificationKeyRelationship,
		) -> Option<&DidVerificationKey> {
			let key_id = match key_type {
				DidVerificationKeyRelationship::AssertionMethod => self.attestation_key,
				DidVerificationKeyRelationship::Authentication => Some(self.authentication_key),
				DidVerificationKeyRelationship::CapabilityDelegation => self.delegation_key,
				_ => None,
			}?;
			let key_details = self.public_keys.get(&key_id)?;
			if let DidPublicKey::PublicVerificationKey(key) = &key_details.key {
				Some(&key)
			} else {
				// The case of something different than a verification key should never happen.
				None
			}
		}

		/// Increase the tx counter of the DID.
		pub fn increase_tx_counter(&mut self) -> Result<(), StorageError> {
			self.last_tx_counter = self
				.last_tx_counter
				.checked_add(1)
				.ok_or(StorageError::MaxTxCounterValue)?;
			Ok(())
		}

		/// Returns the last used tx counter for the DID.
		pub fn get_tx_counter_value(&self) -> u64 {
			self.last_tx_counter
		}

		/// Set the DID tx counter to an arbitrary value.
		#[cfg(any(feature = "mock", test))]
		pub fn set_tx_counter(&mut self, value: u64) {
			self.last_tx_counter = value;
		}
	}

	impl<T: Config> From<DidCreationOperation<T>> for DidDetails<T> {
		fn from(op: DidCreationOperation<T>) -> Self {
			let current_block_number = <frame_system::Pallet<T>>::block_number();

			// Creates a new DID with the given authentication key.
			let mut new_did_details = DidDetails::new(op.new_authentication_key, current_block_number);

			new_did_details.add_key_agreement_keys(op.new_key_agreement_keys, current_block_number);

			if let Some(attesation_key) = op.new_attestation_key {
				new_did_details.update_attestation_key(attesation_key, current_block_number);
			}

			if let Some(delegation_key) = op.new_delegation_key {
				new_did_details.update_delegation_key(delegation_key, current_block_number);
			}

			new_did_details.endpoint_url = op.new_endpoint_url;

			new_did_details
		}
	}

	// Generates a new DID entry starting from the current one stored in the
	// storage and by applying the changes in the [DidUpdateOperation].
	//
	// The operation fails with a [DidError] if the update operation instructs to
	// delete a verification key that is not associated with the DID.
	//
	// Please note that this method does not perform any checks regarding
	// the validity of the [DidUpdateOperation] signature nor whether the nonce
	// provided is valid.
	impl<T: Config> TryFrom<(DidDetails<T>, DidUpdateOperation<T>)> for DidDetails<T> {
		type Error = DidError;

		fn try_from(
			(old_details, update_operation): (DidDetails<T>, DidUpdateOperation<T>),
		) -> Result<Self, Self::Error> {
			let current_block_number = <frame_system::Pallet<T>>::block_number();

			let mut new_details = old_details;

			// Remove specified public keys.
			new_details
				.remove_public_keys(&update_operation.public_keys_to_remove)
				.map_err(DidError::StorageError)?;

			// Update the authentication key, if needed.
			if let Some(new_authentication_key) = update_operation.new_authentication_key {
				new_details.update_authentication_key(new_authentication_key, current_block_number);
			}

			// Add any new key agreement keys.
			new_details.add_key_agreement_keys(update_operation.new_key_agreement_keys, current_block_number);

			// Update/remove the attestation key, if needed.
			match update_operation.attestation_key_update {
				DidVerificationKeyUpdateAction::Delete => {
					new_details.delete_attestation_key();
				}
				DidVerificationKeyUpdateAction::Change(new_attestation_key) => {
					new_details.update_attestation_key(new_attestation_key, current_block_number);
				}
				// Nothing happens.
				DidVerificationKeyUpdateAction::Ignore => {}
			}

			// Update/remove the delegation key, if needed.
			match update_operation.delegation_key_update {
				DidVerificationKeyUpdateAction::Delete => {
					new_details.delete_delegation_key();
				}
				DidVerificationKeyUpdateAction::Change(new_delegation_key) => {
					new_details.update_delegation_key(new_delegation_key, current_block_number);
				}
				// Nothing happens.
				DidVerificationKeyUpdateAction::Ignore => {}
			}

			// Update URL, if needed.
			if let Some(new_endpoint_url) = update_operation.new_endpoint_url {
				new_details.endpoint_url = Some(new_endpoint_url);
			}

			// Update DID counter.
			new_details.last_tx_counter = update_operation.tx_counter;

			Ok(new_details)
		}
	}

	/// An operation that requires DID authentication.
	pub trait DidOperation<T: Config>: Encode {
		/// The type of the verification key to be used to validate the
		/// operation.
		fn get_verification_key_type(&self) -> DidVerificationKeyRelationship;
		/// The DID identifier of the subject.
		fn get_did(&self) -> &T::DidIdentifier;
		/// The operation tx counter, used to protect against replay attacks.
		fn get_tx_counter(&self) -> u64;
	}

	/// An operation to create a new DID.
	///
	/// The struct implements the [DidOperation] trait, and as such it must
	/// contain information about the caller's DID, the type of DID key
	/// required to verify the operation signature, and the tx counter to
	/// protect against replay attacks.
	#[derive(Clone, Debug, Decode, Encode, PartialEq)]
	pub struct DidCreationOperation<T: Config> {
		/// The DID identifier. It has to be unique.
		pub did: T::DidIdentifier,
		/// The new authentication key.
		pub new_authentication_key: DidVerificationKey,
		/// The new key agreement keys.
		pub new_key_agreement_keys: BTreeSet<DidEncryptionKey>,
		/// \[OPTIONAL\] The new attestation key.
		pub new_attestation_key: Option<DidVerificationKey>,
		/// \[OPTIONAL\] The new delegation key.
		pub new_delegation_key: Option<DidVerificationKey>,
		/// \[OPTIONAL\] The URL containing the DID endpoints description.
		pub new_endpoint_url: Option<Url>,
	}

	impl<T: Config> DidOperation<T> for DidCreationOperation<T> {
		fn get_verification_key_type(&self) -> DidVerificationKeyRelationship {
			DidVerificationKeyRelationship::Authentication
		}

		fn get_did(&self) -> &T::DidIdentifier {
			&self.did
		}

		// Irrelevant for creation operations.
		fn get_tx_counter(&self) -> u64 {
			0u64
		}
	}

	/// An operation to update a DID.
	///
	/// The struct implements the [DidOperation] trait, and as such it must
	/// contain information about the caller's DID, the type of DID key
	/// required to verify the operation signature, and the tx counter to
	/// protect against replay attacks.
	#[derive(Clone, Debug, Decode, Encode, PartialEq)]
	pub struct DidUpdateOperation<T: Config> {
		/// The DID identifier.
		pub did: T::DidIdentifier,
		/// \[OPTIONAL\] The new authentication key.
		pub new_authentication_key: Option<DidVerificationKey>,
		/// A new set of key agreement keys to add to the ones already stored.
		pub new_key_agreement_keys: BTreeSet<DidEncryptionKey>,
		/// \[OPTIONAL\] The attestation key update action.
		pub attestation_key_update: DidVerificationKeyUpdateAction,
		/// \[OPTIONAL\] The delegation key update action.
		pub delegation_key_update: DidVerificationKeyUpdateAction,
		/// The set of old attestation keys to remove, given their identifiers.
		/// If the operation also replaces the current attestation key, it will
		/// not be considered for removal in this operation, so it is not
		/// possible to specify it for removal in this set.
		pub public_keys_to_remove: BTreeSet<KeyId<T>>,
		/// \[OPTIONAL\] The new endpoint URL.
		pub new_endpoint_url: Option<Url>,
		/// The DID tx counter.
		pub tx_counter: u64,
	}

	impl<T: Config> DidOperation<T> for DidUpdateOperation<T> {
		fn get_verification_key_type(&self) -> DidVerificationKeyRelationship {
			DidVerificationKeyRelationship::Authentication
		}

		fn get_did(&self) -> &T::DidIdentifier {
			&self.did
		}

		fn get_tx_counter(&self) -> u64 {
			self.tx_counter
		}
	}

	/// Possible actions on a DID verification key within a
	/// [DidUpdateOperation].
	#[derive(Clone, Copy, Decode, Debug, Encode, Eq, Ord, PartialEq, PartialOrd)]
	pub enum DidVerificationKeyUpdateAction {
		/// Do not change the verification key.
		Ignore,
		/// Change the verification key to the new one provided.
		Change(DidVerificationKey),
		/// Delete the verification key.
		Delete,
	}

	// Return the ignore operation by default
	impl Default for DidVerificationKeyUpdateAction {
		fn default() -> Self {
			Self::Ignore
		}
	}

	/// An operation to delete a DID.
	///
	/// The struct implements the [DidOperation] trait, and as such it must
	/// contain information about the caller's DID, the type of DID key
	/// required to verify the operation signature, and the tx counter to
	/// protect against replay attacks.
	#[derive(Clone, Debug, Decode, Encode, PartialEq)]
	pub struct DidDeletionOperation<T: Config> {
		/// The DID identifier.
		pub did: T::DidIdentifier,
		/// The DID tx counter.
		pub tx_counter: u64,
	}

	impl<T: Config> DidOperation<T> for DidDeletionOperation<T> {
		fn get_verification_key_type(&self) -> DidVerificationKeyRelationship {
			DidVerificationKeyRelationship::Authentication
		}

		fn get_did(&self) -> &T::DidIdentifier {
			&self.did
		}

		fn get_tx_counter(&self) -> u64 {
			self.tx_counter
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

	/// Supported URLs.
	#[derive(Clone, Decode, Debug, Encode, PartialEq)]
	pub enum Url {
		/// See [HttpUrl].
		Http(HttpUrl),
		/// See [FtpUrl].
		Ftp(FtpUrl),
		/// See [IpfsUrl].
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

	#[pallet::config]
	pub trait Config: frame_system::Config + Debug {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type WeightInfo: WeightInfo;
		type DidIdentifier: Parameter + Encode + Decode + Debug;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	/// DIDs stored on chain.
	///
	/// It maps from a DID identifier to the DID details.
	#[pallet::storage]
	#[pallet::getter(fn get_did)]
	pub type Did<T> = StorageMap<_, Blake2_128Concat, <T as Config>::DidIdentifier, DidDetails<T>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new DID has been created.
		/// \[transaction signer, DID identifier\]
		DidCreated(T::AccountId, T::DidIdentifier),
		/// A DID has been updated.
		/// \[transaction signer, DID identifier\]
		DidUpdated(T::AccountId, T::DidIdentifier),
		/// A DID has been deleted.
		/// \[transaction signer, DID identifier\]
		DidDeleted(T::AccountId, T::DidIdentifier),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The DID operation signature is not in the format the verification
		/// key expects.
		InvalidSignatureFormat,
		/// The DID operation signature is invalid for the payload and the
		/// verification key provided.
		InvalidSignature,
		/// The DID with the given identifier is already present on chain.
		DidAlreadyPresent,
		/// No DID with the given identifier is present on chain.
		DidNotPresent,
		/// One or more verification keys referenced are not stored in the set
		/// of verification keys.
		VerificationKeyNotPresent,
		/// The DID operation nonce is not equal to the current DID nonce + 1.
		InvalidNonce,
		/// The URL specified is not ASCII-encoded.
		InvalidUrlEncoding,
		/// The URL specified is not properly formatted.
		InvalidUrlScheme,
		/// The maximum supported value for the DID tx counter has been reached.
		/// No more operations with the DID are allowed.
		MaxTxCounterValue,
		/// The user tries to delete a verification key that is currently being
		/// used as an authentication, delegation, or attestation key, and this
		/// is not allowed.
		CurrentlyActiveKey,
		/// An error that is not supposed to take place, yet it happened.
		InternalError,
	}

	impl<T> From<DidError> for Error<T> {
		fn from(error: DidError) -> Self {
			match error {
				DidError::StorageError(storage_error) => Self::from(storage_error),
				DidError::SignatureError(operation_error) => Self::from(operation_error),
				DidError::UrlError(url_error) => Self::from(url_error),
				DidError::InternalError => Self::InternalError,
			}
		}
	}

	impl<T> From<StorageError> for Error<T> {
		fn from(error: StorageError) -> Self {
			match error {
				StorageError::DidNotPresent => Self::DidNotPresent,
				StorageError::DidAlreadyPresent => Self::DidAlreadyPresent,
				StorageError::DidKeyNotPresent(_) | StorageError::VerificationKeyNotPresent => {
					Self::VerificationKeyNotPresent
				}
				StorageError::MaxTxCounterValue => Self::MaxTxCounterValue,
				StorageError::CurrentlyActiveKey => Self::CurrentlyActiveKey,
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
		/// with the creation operation.
		///
		/// * origin: the Substrate account submitting the transaction (which
		///   can be different from the DID subject)
		/// * operation: the [DidCreationOperation] which contains the details
		///   of the new DID
		/// * signature: the [signature](DidSignature) over the operation that
		///   must be signed with the authentication key provided in the
		///   operation
		#[pallet::weight(T::WeightInfo::submit_did_create_operation())]
		pub fn submit_did_create_operation(
			origin: OriginFor<T>,
			operation: DidCreationOperation<T>,
			signature: DidSignature,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			// There has to be no other DID with the same identifier already saved on chain,
			// otherwise generate a DidAlreadyPresent error.
			ensure!(
				!<Did<T>>::contains_key(operation.get_did()),
				<Error<T>>::DidAlreadyPresent
			);

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

			Ok(None.into())
		}

		/// Updates the information associated with a DID on chain, after
		/// verifying the signature associated with the operation.
		///
		/// * origin: the Substrate account submitting the transaction (which
		///   can be different from the DID subject)
		/// * operation: the [DidUpdateOperation] which contains the new details
		///   of the given DID
		/// * signature: the [signature](DidSignature) over the operation that
		///   must be signed with the authentication key associated with the new
		///   DID. Even in case the authentication key is being updated, the
		///   operation must still be signed with the old one being replaced.
		#[pallet::weight(T::WeightInfo::submit_did_update_operation())]
		pub fn submit_did_update_operation(
			origin: OriginFor<T>,
			operation: DidUpdateOperation<T>,
			signature: DidSignature,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			// Saved here as it is consumed later when generating the new DidDetails object.
			let did_identifier = operation.get_did().clone();

			let did_details = <Did<T>>::get(&did_identifier).ok_or(<Error<T>>::DidNotPresent)?;

			// Verify the signature and the nonce of the update operation.
			Self::verify_operation_validity_for_did(&operation, &signature, &did_details).map_err(<Error<T>>::from)?;

			// Generate a new DidDetails object by applying the changes in the update
			// operation to the old object (and consuming both).
			let new_did_details = DidDetails::try_from((did_details, operation)).map_err(<Error<T>>::from)?;

			log::debug!("Updating DID {:?}", did_identifier);
			<Did<T>>::insert(&did_identifier, new_did_details);

			Self::deposit_event(Event::DidUpdated(sender, did_identifier));

			Ok(None.into())
		}

		/// Deletes all the information associated with a DID on chain, after
		/// verifying the signature associated with the operation.
		///
		/// * origin: the Substrate account submitting the transaction (which
		///   can be different from the DID subject)
		/// * operation: the [DidDeletionOperation] which includes the DID to
		///   deactivate
		/// * signature: the [signature](DidSignature) over the operation that
		///   must be signed with the authentication key associated with the new
		///   DID.
		#[pallet::weight(T::WeightInfo::submit_did_delete_operation())]
		pub fn submit_did_delete_operation(
			origin: OriginFor<T>,
			operation: DidDeletionOperation<T>,
			signature: DidSignature,
		) -> DispatchResultWithPostInfo {
			let sender = ensure_signed(origin)?;

			let did_identifier = operation.get_did();

			let did_details = <Did<T>>::get(&did_identifier).ok_or(<Error<T>>::DidNotPresent)?;

			// Verify the signature and the nonce of the delete operation.
			Self::verify_operation_validity_for_did(&operation, &signature, &did_details).map_err(<Error<T>>::from)?;

			log::debug!("Deleting DID {:?}", did_identifier);
			<Did<T>>::remove(&did_identifier);

			Self::deposit_event(Event::DidDeleted(sender, did_identifier.clone()));

			Ok(None.into())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Verify the validity (i.e., nonce and signature) of a generic
	/// [DidOperation] and, if valid, update the DID state with the latest
	/// nonce.
	///
	/// * operation: the reference to the [DidOperation] which validity is to be
	///   verified
	/// * signature: a reference to the [signature](DidSignature)
	/// * did: the DID identifier to verify the operation signature for
	pub fn verify_operation_validity_and_increase_did_nonce<O: DidOperation<T>>(
		operation: &O,
		signature: &DidSignature,
	) -> Result<(), DidError> {
		let mut did_details =
			<Did<T>>::get(&operation.get_did()).ok_or(DidError::StorageError(StorageError::DidNotPresent))?;

		Self::verify_operation_validity_for_did(operation, &signature, &did_details)?;

		// Update tx counter in DID details and save to DID pallet
		did_details.increase_tx_counter().map_err(DidError::StorageError)?;
		<Did<T>>::insert(&operation.get_did(), did_details);

		Ok(())
	}

	// Internally verifies the validity of a DID operation nonce and signature.
	fn verify_operation_validity_for_did<O: DidOperation<T>>(
		operation: &O,
		signature: &DidSignature,
		did_details: &DidDetails<T>,
	) -> Result<(), DidError> {
		Self::verify_operation_counter_for_did(operation, did_details)?;
		Self::verify_payload_signature_with_did_key_type(
			&operation.encode(),
			signature,
			did_details,
			operation.get_verification_key_type(),
		)
	}

	// Verify the validity of a DID operation nonce.
	// To be valid, the nonce must be equal to the one currently stored + 1.
	// This is to avoid quickly "consuming" all the possible values for the counter,
	// as that would result in the DID being unusable, since we do not have yet any
	// mechanism in place to wrap the counter value around when the limit is
	// reached.
	fn verify_operation_counter_for_did<O: DidOperation<T>>(
		operation: &O,
		did_details: &DidDetails<T>,
	) -> Result<(), DidError> {
		// Verify that the DID has not reached the maximum tx counter value
		ensure!(
			did_details.get_tx_counter_value() < u64::MAX,
			DidError::StorageError(StorageError::MaxTxCounterValue)
		);

		// Verify that the operation counter is equal to the stored one + 1
		let expected_nonce_value = did_details
			.get_tx_counter_value()
			.checked_add(1)
			.ok_or(DidError::InternalError)?;
		ensure!(
			operation.get_tx_counter() == expected_nonce_value,
			DidError::SignatureError(SignatureError::InvalidNonce)
		);

		Ok(())
	}

	// Verify a generic payload signature using a given DID verification key type.
	pub fn verify_payload_signature_with_did_key_type(
		payload: &Payload,
		signature: &DidSignature,
		did_details: &DidDetails<T>,
		key_type: DidVerificationKeyRelationship,
	) -> Result<(), DidError> {
		// Retrieve the needed verification key from the DID details, or generate an
		// error if there is no key of the type required
		let verification_key = did_details
			.get_verification_key_for_key_type(key_type.clone())
			.ok_or(DidError::StorageError(StorageError::DidKeyNotPresent(key_type)))?;

		// Verify that the signature matches the expected format, otherwise generate
		// an error
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
