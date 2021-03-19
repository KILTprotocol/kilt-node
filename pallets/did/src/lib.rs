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

use codec::{Decode, Encode, WrapperTypeDecode};
pub use default_weights::WeightInfo;

use frame_support::{Parameter, StorageMap, decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure};
use frame_system::{self, ensure_signed};
use sp_runtime::traits::Verify;
use sp_std::{prelude::{Clone}, convert::TryFrom, collections::btree_set::BTreeSet};
use sp_core::{ed25519, sr25519};

/// The DID trait
pub trait Config: frame_system::Config {
	/// DID specific event type
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;

	/// Weight information for extrinsics in this pallet.
	type WeightInfo: WeightInfo;
}

pub trait DIDPublicKey {
	fn get_did_key_description(&self) -> &'static str;
}

#[derive(Encode, Decode, PartialEq, Clone, Ord, Eq, PartialOrd, Debug)]
pub enum PublicVerificationKey {
	Ed25519([u8; 32]),
	Sr25519([u8; 32]),
}

impl PublicVerificationKey {
	fn verify_signature(&self, payload: &Payload, signature: &Signature) -> SignatureVerificationResult {
		ensure!(signature.len() == self.get_expected_signature_size(), SigningKeyError::InvalidSignatureFormat);
		let signature = signature as &[u8];
		// Did not find a way to return a Signature object from the match and then call directly verify on that...
		match self {
			PublicVerificationKey::Ed25519(raw_key) => {
				let ed25519_sig = ed25519::Signature::try_from(
					signature
				).map_err(|_| SigningKeyError::InvalidSignatureFormat)?;
				let signer = ed25519::Public(raw_key.clone());
				Ok(ed25519_sig.verify(payload as &[u8], &signer))
			}
			PublicVerificationKey::Sr25519(raw_key) => {
				let sr25519_sig = sr25519::Signature::try_from(
					signature
				).map_err(|_| SigningKeyError::InvalidSignatureFormat)?;
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

pub type Payload = Vec<u8>;
pub type Signature = Vec<u8>;
pub type KeyValue = Vec<u8>;
pub type DIDIdentifier = Vec<u8>;
pub type KeyIdentifier = Vec<u8>;
pub type URL = Vec<u8>;
pub type SignatureVerificationResult = Result<bool, SigningKeyError>;

#[derive(Encode, Decode, PartialEq, Clone, Ord, Eq, PartialOrd, Debug)]
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

#[derive(Encode, Decode, Clone, PartialEq)]
pub struct DIDDetails {
	auth_key: PublicVerificationKey,
	key_agreement_key: PublicEncryptionKey,
	delegation_key: Option<PublicVerificationKey>,
	attestation_key: Option<PublicVerificationKey>,
	verification_keys: BTreeSet<PublicVerificationKey>,
	endpoint_url: Option<URL>,
	last_tx_counter: u64
}

#[derive(Debug, Encode, Decode, Clone, PartialEq)]
pub enum DIDUpdateAction {
	SetVerificationKey(DIDVerificationKeyType, PublicVerificationKey),
	SetEncryptionKey(DIDEncryptionKeyType, PublicEncryptionKey),
	RemoveSigningKey(DIDVerificationKeyType),
	RemoveEncryptionKey(DIDEncryptionKeyType),
	DeleteVerificationKey(KeyIdentifier),
	SetServiceEndpoint(URL),
	DeleteServiceEndpoint(),
}

#[derive(Debug, Encode, Decode, Clone, PartialEq)]
pub enum DIDVerificationKeyType {
	Authentication,
	CapabilityDelegation,
	CapabilityInvocation,           // For future use
	AssertionMethod,
}

#[derive(Debug, Encode, Decode, Clone, PartialEq)]
pub enum DIDEncryptionKeyType {
	KeyAgreement,
}

pub trait DIDOperation {
	fn get_verification_key_type(&self) -> DIDVerificationKeyType;
	fn get_signature(&self) -> &Signature;
	fn get_did(&self) -> &DIDIdentifier;
}

#[derive(Debug, Encode, Decode, Clone, PartialEq)]
pub struct DIDUpdateOperation {
	did: DIDIdentifier,
	operations: Vec<DIDUpdateAction>,
	signature: Signature,
	txCounter: u64,
}

impl DIDOperation for DIDUpdateOperation {
    fn get_verification_key_type(&self) -> DIDVerificationKeyType {
        DIDVerificationKeyType::Authentication
    }

    fn get_signature(&self) -> &Signature {
        self.signature.as_ref()
    }

    fn get_did(&self) -> &DIDIdentifier {
        self.did.as_ref()
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
			ensure_signed(origin)?;

			let did_entry: Option<DIDDetails> = <DIDs<T>>::get(did);

			if (did_entry.is_none()) {

			} else {

			}


			Ok(())
		}
	}
}

impl<T: Config> Module<T> {
	// If did_entry is None, it is a new DID and the signature must match the authentication key specified.
	pub fn verify_did_update_operation_signature(op: DIDUpdateOperation, did_entry: Option<DIDDetails>) -> SignatureVerificationResult {
		if (did_entry.is_some()) {
			Ok(true)
		} else {
			Self::verify_did_operation_signature(op)?
			let last_tx_counter = did_entry.last_tx_counter;
			if (last_tx_counter >= op.txCounter) {
				Error(InvalidNonce)
			} else {
				Ok(true)
			}
		}
	}

	pub fn verify_did_operation_signature<O: DIDOperation>(op: O) -> SignatureVerificationResult {
		let did_verification_key = Self::retrieve_did_key_for_type(op.get_did(), op.get_verification_key_type());
		op.get_signature().is_some() && did_verification_key.is_some() && did_verification_key.verify_signature(op.encode().as_ref(), op.get_signature())
	}

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
}
 

decl_storage! {
	trait Store for Module<T: Config> as DID {
		DIDs get(fn dids):map hasher(opaque_blake2_256) DIDIdentifier => Option<DIDDetails>;
	}
}