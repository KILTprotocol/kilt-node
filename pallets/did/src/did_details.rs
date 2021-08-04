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

use codec::{Decode, Encode, WrapperTypeEncode};
use sp_core::{ecdsa, ed25519, sr25519};
use sp_runtime::traits::Verify;
use sp_std::{
	collections::{btree_map::BTreeMap, btree_set::BTreeSet},
	convert::TryInto,
};

use crate::*;

/// Types of verification keys a DID can control.
#[derive(Clone, Decode, Debug, Encode, Eq, Ord, PartialEq, PartialOrd)]
pub enum DidVerificationKey {
	/// An Ed25519 public key.
	Ed25519(ed25519::Public),
	/// A Sr25519 public key.
	Sr25519(sr25519::Public),
	/// An ECDSA public key.
	Ecdsa(ecdsa::Public),
}

impl DidVerificationKey {
	/// Verify a DID signature using one of the DID keys.
	pub fn verify_signature(&self, payload: &Payload, signature: &DidSignature) -> Result<(), SignatureError> {
		match (self, signature) {
			(DidVerificationKey::Ed25519(public_key), DidSignature::Ed25519(sig)) => {
				ensure!(sig.verify(payload, public_key), SignatureError::InvalidSignature);
				Ok(())
			}
			// Follows same process as above, but using a Sr25519 instead
			(DidVerificationKey::Sr25519(public_key), DidSignature::Sr25519(sig)) => {
				ensure!(sig.verify(payload, public_key), SignatureError::InvalidSignature);
				Ok(())
			}
			(DidVerificationKey::Ecdsa(public_key), DidSignature::Ecdsa(sig)) => {
				ensure!(sig.verify(payload, public_key), SignatureError::InvalidSignature);
				Ok(())
			}
			_ => Err(SignatureError::InvalidSignatureFormat),
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

impl From<ecdsa::Public> for DidVerificationKey {
	fn from(key: ecdsa::Public) -> Self {
		DidVerificationKey::Ecdsa(key)
	}
}

/// Types of encryption keys a DID can control.
#[derive(Clone, Copy, Decode, Debug, Encode, Eq, Ord, PartialEq, PartialOrd)]
pub enum DidEncryptionKey {
	/// An X25519 public key.
	X25519([u8; 32]),
}

/// A general public key under the control of the DID.
#[derive(Clone, Decode, Debug, Encode, Eq, Ord, PartialEq, PartialOrd)]
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
#[derive(Clone, Copy, Debug, Decode, Encode, PartialEq, Eq)]
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
	/// An Ecdsa signature.
	Ecdsa(ecdsa::Signature),
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

impl From<ecdsa::Signature> for DidSignature {
	fn from(sig: ecdsa::Signature) -> Self {
		DidSignature::Ecdsa(sig)
	}
}

pub trait DidVerifiableIdentifier {
	/// Allows a verifiable identifier to verify a signature it produces and
	/// return the public key
	/// associated with the identifier.
	fn verify_and_recover_signature(
		&self,
		payload: &Payload,
		signature: &DidSignature,
	) -> Result<DidVerificationKey, SignatureError>;
}

// TODO: did not manage to implement this trait in the kilt_primitives crate due
// to some circular dependency problems.
impl DidVerifiableIdentifier for kilt_primitives::DidIdentifier {
	fn verify_and_recover_signature(
		&self,
		payload: &Payload,
		signature: &DidSignature,
	) -> Result<DidVerificationKey, SignatureError> {
		// So far, either the raw Ed25519/Sr25519 public key or the Blake2-256 hashed
		// ECDSA public key.
		let raw_public_key: &[u8; 32] = self.as_ref();
		match *signature {
			DidSignature::Ed25519(_) => {
				// from_raw simply converts a byte array into a public key with no particular
				// validations
				let ed25519_did_key = DidVerificationKey::Ed25519(ed25519::Public::from_raw(*raw_public_key));
				ed25519_did_key
					.verify_signature(payload, signature)
					.map(|_| ed25519_did_key)
			}
			DidSignature::Sr25519(_) => {
				let sr25519_did_key = DidVerificationKey::Sr25519(sr25519::Public::from_raw(*raw_public_key));
				sr25519_did_key
					.verify_signature(payload, signature)
					.map(|_| sr25519_did_key)
			}
			DidSignature::Ecdsa(ref signature) => {
				let ecdsa_signature: [u8; 65] = signature
					.encode()
					.try_into()
					.map_err(|_| SignatureError::InvalidSignature)?;
				// ECDSA uses blake2-256 hashing algorihtm for signatures, so we hash the given
				// message to recover the public key.
				let hashed_message = sp_io::hashing::blake2_256(payload);
				let recovered_pk: [u8; 33] =
					sp_io::crypto::secp256k1_ecdsa_recover_compressed(&ecdsa_signature, &hashed_message)
						.map_err(|_| SignatureError::InvalidSignature)?;
				let hashed_recovered_pk = sp_io::hashing::blake2_256(&recovered_pk);
				// The hashed recovered public key must be equal to the AccountId32 value, which
				// is the hashed key.
				ensure!(&hashed_recovered_pk == raw_public_key, SignatureError::InvalidSignature);
				// Safe to reconstruct the public key using the recovered value from
				// secp256k1_ecdsa_recover_compressed
				Ok(DidVerificationKey::from(ecdsa::Public(recovered_pk)))
			}
		}
	}
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
	pub block_number: BlockNumberOf<T>,
}

/// The details associated to a DID identity.
#[derive(Clone, Debug, Decode, Encode, PartialEq)]
pub struct DidDetails<T: Config> {
	/// The ID of the authentication key, used to authenticate DID-related
	/// operations.
	authentication_key: KeyIdOf<T>,
	/// The set of the key agreement key IDs, which can be used to encrypt
	/// data addressed to the DID subject.
	key_agreement_keys: BTreeSet<KeyIdOf<T>>,
	/// \[OPTIONAL\] The ID of the delegation key, used to verify the
	/// signatures of the delegations created by the DID subject.
	delegation_key: Option<KeyIdOf<T>>,
	/// \[OPTIONAL\] The ID of the attestation key, used to verify the
	/// signatures of the attestations created by the DID subject.
	attestation_key: Option<KeyIdOf<T>>,
	/// The map of public keys, with the key label as
	/// the key map and the tuple (key, addition_block_number) as the map
	/// value.
	/// The map includes all the keys under the control of the DID subject,
	/// including the ones currently used for authentication, key agreement,
	/// attestation, and delegation. Other than those, the map also contains
	/// the old attestation keys that have been rotated, i.e., they cannot
	/// be used to create new attestations but can still be used to verify
	/// previously issued attestations.
	public_keys: BTreeMap<KeyIdOf<T>, DidPublicKeyDetails<T>>,
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
	pub fn new(authentication_key: DidVerificationKey, block_number: BlockNumberOf<T>) -> Self {
		let mut public_keys: BTreeMap<KeyIdOf<T>, DidPublicKeyDetails<T>> = BTreeMap::new();
		let authentication_key_id = utils::calculate_key_id::<T>(&authentication_key.clone().into());
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

	// Creates a new DID entry from [DidUpdateDetails] and a given authentication
	// key.
	pub fn from_creation_details(
		details: DidCreationDetails<T>,
		new_auth_key: DidVerificationKey,
	) -> Result<Self, InputError> {
		ensure!(
			details.new_key_agreement_keys.len()
				<= <<T as Config>::MaxNewKeyAgreementKeys>::get().saturated_into::<usize>(),
			InputError::MaxKeyAgreementKeysLimitExceeded
		);

		if let Some(ref endpoint_url) = details.new_endpoint_url {
			ensure!(
				endpoint_url.len() <= T::MaxUrlLength::get().saturated_into::<usize>(),
				InputError::MaxUrlLengthExceeded
			);
		}

		let current_block_number = <frame_system::Pallet<T>>::block_number();

		// Creates a new DID with the given authentication key.
		let mut new_did_details = DidDetails::new(new_auth_key, current_block_number);

		new_did_details.add_key_agreement_keys(details.new_key_agreement_keys, current_block_number);

		if let Some(attesation_key) = details.new_attestation_key {
			new_did_details.update_attestation_key(attesation_key, current_block_number);
		}

		if let Some(delegation_key) = details.new_delegation_key {
			new_did_details.update_delegation_key(delegation_key, current_block_number);
		}

		new_did_details.endpoint_url = details.new_endpoint_url;

		Ok(new_did_details)
	}

	// Updates a DID entry by applying the changes in the [DidUpdateDetails].
	//
	// The operation fails with a [DidError] if the update details instructs to
	// delete a verification key that is not associated with the DID.
	pub fn apply_update_details(&mut self, update_details: DidUpdateDetails<T>) -> Result<(), DidError> {
		ensure!(
			update_details.new_key_agreement_keys.len()
				<= <<T as Config>::MaxNewKeyAgreementKeys>::get().saturated_into::<usize>(),
			DidError::InputError(InputError::MaxKeyAgreementKeysLimitExceeded)
		);

		ensure!(
			update_details.public_keys_to_remove.len()
				<= <<T as Config>::MaxVerificationKeysToRevoke>::get().saturated_into::<usize>(),
			DidError::InputError(InputError::MaxVerificationKeysToRemoveLimitExceeded)
		);

		if let Some(ref endpoint_url) = update_details.new_endpoint_url {
			ensure!(
				endpoint_url.len() <= T::MaxUrlLength::get().saturated_into::<usize>(),
				DidError::InputError(InputError::MaxUrlLengthExceeded)
			);
		}

		let current_block_number = <frame_system::Pallet<T>>::block_number();

		// Remove specified public keys.
		self.remove_public_keys(&update_details.public_keys_to_remove)
			.map_err(DidError::StorageError)?;

		// Update the authentication key, if needed.
		if let Some(new_authentication_key) = update_details.new_authentication_key {
			self.update_authentication_key(new_authentication_key, current_block_number);
		}

		// Add any new key agreement keys.
		self.add_key_agreement_keys(update_details.new_key_agreement_keys, current_block_number);

		// Update/remove the attestation key, if needed.
		match update_details.attestation_key_update {
			DidVerificationKeyUpdateAction::Delete => {
				self.delete_attestation_key();
			}
			DidVerificationKeyUpdateAction::Change(new_attestation_key) => {
				self.update_attestation_key(new_attestation_key, current_block_number);
			}
			// Nothing happens.
			DidVerificationKeyUpdateAction::Ignore => {}
		}

		// Update/remove the delegation key, if needed.
		match update_details.delegation_key_update {
			DidVerificationKeyUpdateAction::Delete => {
				self.delete_delegation_key();
			}
			DidVerificationKeyUpdateAction::Change(new_delegation_key) => {
				self.update_delegation_key(new_delegation_key, current_block_number);
			}
			// Nothing happens.
			DidVerificationKeyUpdateAction::Ignore => {}
		}

		// Update URL, if needed.
		if let Some(new_endpoint_url) = update_details.new_endpoint_url {
			self.endpoint_url = Some(new_endpoint_url);
		}

		Ok(())
	}

	/// Update the DID authentication key.
	///
	/// The old key is deleted from the set of verification keys if it is
	/// not used in any other part of the DID. The new key is added to the
	/// set of verification keys.
	pub fn update_authentication_key(
		&mut self,
		new_authentication_key: DidVerificationKey,
		block_number: BlockNumberOf<T>,
	) {
		let old_authentication_key_id = self.authentication_key;
		let new_authentication_key_id = utils::calculate_key_id::<T>(&new_authentication_key.clone().into());
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
		block_number: BlockNumberOf<T>,
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
	pub fn update_attestation_key(&mut self, new_attestation_key: DidVerificationKey, block_number: BlockNumberOf<T>) {
		let new_attestation_key_id = utils::calculate_key_id::<T>(&new_attestation_key.clone().into());
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
	/// The old key is not removed from the set of verification keys, hence
	/// it can still be used to verify past attestations.
	pub fn delete_attestation_key(&mut self) {
		self.attestation_key = None;
	}

	/// Update the DID delegation key.
	///
	/// The old key is deleted from the set of verification keys if it is
	/// not used in any other part of the DID. The new key is added to the
	/// set of verification keys.
	pub fn update_delegation_key(&mut self, new_delegation_key: DidVerificationKey, block_number: BlockNumberOf<T>) {
		let old_delegation_key_id = self.delegation_key;
		let new_delegation_key_id = utils::calculate_key_id::<T>(&new_delegation_key.clone().into());
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
	/// Once deleted, it cannot be used to write new delegations anymore.
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
	/// - 2. the set of keys to delete contains key IDs that are not currently
	///   stored on chain
	fn remove_public_keys(&mut self, key_ids: &BTreeSet<KeyIdOf<T>>) -> Result<(), StorageError> {
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
	fn remove_key_if_unused(&mut self, key_id: &KeyIdOf<T>) {
		if self.authentication_key != *key_id
			&& self.attestation_key != Some(*key_id)
			&& self.delegation_key != Some(*key_id)
			&& !self.key_agreement_keys.contains(key_id)
		{
			self.public_keys.remove(key_id);
		}
	}

	pub fn get_authentication_key_id(&self) -> KeyIdOf<T> {
		self.authentication_key
	}

	pub fn get_key_agreement_keys_ids(&self) -> &BTreeSet<KeyIdOf<T>> {
		&self.key_agreement_keys
	}

	pub fn get_attestation_key_id(&self) -> &Option<KeyIdOf<T>> {
		&self.attestation_key
	}

	pub fn get_delegation_key_id(&self) -> &Option<KeyIdOf<T>> {
		&self.delegation_key
	}

	pub fn get_public_keys(&self) -> &BTreeMap<KeyIdOf<T>, DidPublicKeyDetails<T>> {
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
			Some(key)
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

/// The details of a new DID to create.
#[derive(Clone, Debug, Decode, Encode, PartialEq)]
pub struct DidCreationDetails<T: Config> {
	/// The DID identifier. It has to be unique.
	pub did: DidIdentifierOf<T>,
	/// The new key agreement keys.
	pub new_key_agreement_keys: BTreeSet<DidEncryptionKey>,
	/// \[OPTIONAL\] The new attestation key.
	pub new_attestation_key: Option<DidVerificationKey>,
	/// \[OPTIONAL\] The new delegation key.
	pub new_delegation_key: Option<DidVerificationKey>,
	/// \[OPTIONAL\] The URL containing the DID endpoints description.
	pub new_endpoint_url: Option<Url>,
}

/// The details to update a DID.
#[derive(Clone, Debug, Decode, Encode, PartialEq)]
pub struct DidUpdateDetails<T: Config> {
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
	pub public_keys_to_remove: BTreeSet<KeyIdOf<T>>,
	/// \[OPTIONAL\] The new endpoint URL.
	pub new_endpoint_url: Option<Url>,
}

/// Possible actions on a DID verification key within a
/// [DidUpdateOperation].
#[derive(Clone, Decode, Debug, Encode, Eq, Ord, PartialEq, PartialOrd)]
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

/// Trait for extrinsic DID-based authorization.
///
/// The trait allows
/// [DidAuthorizedCallOperations](DidAuthorizedCallOperation) wrapping an
/// extrinsic to specify what DID key to use to perform signature validation
/// over the byte-encoded operation. A result of None indicates that the
/// extrinsic does not support DID-based authorization.
pub trait DeriveDidCallAuthorizationVerificationKeyRelationship {
	/// The type of the verification key to be used to validate the
	/// wrapped extrinsic.
	fn derive_verification_key_relationship(&self) -> Option<DidVerificationKeyRelationship>;

	// Return a call to dispatch in order to test the pallet proxy feature.
	#[cfg(feature = "runtime-benchmarks")]
	fn get_call_for_did_call_benchmark() -> Self;
}

/// A DID operation that wraps other extrinsic calls, allowing those
/// extrinsic to have a DID origin and perform DID-based authorization upon
/// their invocation.
#[derive(Clone, Debug, Decode, Encode, PartialEq)]
pub struct DidAuthorizedCallOperation<T: Config> {
	/// The DID identifier.
	pub did: DidIdentifierOf<T>,
	/// The DID tx counter.
	pub tx_counter: u64,
	/// The extrinsic call to authorize with the DID.
	pub call: DidCallableOf<T>,
}

/// Wrapper around a [DidAuthorizedCallOperation].
///
/// It contains additional information about the type of DID key to used for
/// authorization.
#[derive(Clone, Debug, PartialEq)]
pub struct DidAuthorizedCallOperationWithVerificationRelationship<T: Config> {
	/// The wrapped [DidAuthorizedCallOperation].
	pub operation: DidAuthorizedCallOperation<T>,
	/// The type of DID key to use for authorization.
	pub verification_key_relationship: DidVerificationKeyRelationship,
}

impl<T: Config> core::ops::Deref for DidAuthorizedCallOperationWithVerificationRelationship<T> {
	type Target = DidAuthorizedCallOperation<T>;

	fn deref(&self) -> &Self::Target {
		&self.operation
	}
}

// Opaque implementation.
// [DidAuthorizedCallOperationWithVerificationRelationship] encodes to
// [DidAuthorizedCallOperation].
impl<T: Config> WrapperTypeEncode for DidAuthorizedCallOperationWithVerificationRelationship<T> {}
