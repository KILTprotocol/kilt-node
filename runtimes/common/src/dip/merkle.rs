// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2023 BOTLabs GmbH

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
use did::{DidVerificationKeyRelationship, KeyIdOf};
use frame_support::RuntimeDebug;
use frame_system::pallet_prelude::BlockNumberFor;
use kilt_dip_primitives::merkle::{DidKeyMerkleKey, DidKeyMerkleValue, DidMerkleProof};
use pallet_did_lookup::linkable_account::LinkableAccountId;
use pallet_dip_provider::{
	traits::{IdentityCommitmentGenerator, IdentityProvider},
	IdentityCommitmentVersion, IdentityOf,
};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::{borrow::ToOwned, marker::PhantomData, vec::Vec};
use sp_trie::{generate_trie_proof, LayoutV1, MemoryDB, TrieDBMutBuilder, TrieHash, TrieMut};

use kilt_dip_primitives::merkle::{DidKeyRelationship, RevealedDidMerkleProofLeaf};

use crate::dip::did::LinkedDidInfoOf;

pub type BlindedValue = Vec<u8>;
/// Type of the Merkle proof revealing parts of the DIP identity of a given DID
/// subject.
pub type DidMerkleProofOf<T> = DidMerkleProof<
	Vec<BlindedValue>,
	RevealedDidMerkleProofLeaf<
		KeyIdOf<T>,
		<T as frame_system::Config>::AccountId,
		BlockNumberFor<T>,
		<T as pallet_web3_names::Config>::Web3Name,
		LinkableAccountId,
	>,
>;

/// Type of a complete DIP Merkle proof.
#[derive(Encode, Decode, RuntimeDebug, PartialEq, Eq, TypeInfo)]
pub struct CompleteMerkleProof<Root, Proof> {
	/// The Merkle root.
	pub root: Root,
	/// The Merkle proof revealing parts of the commitment that verify against
	/// the provided root.
	pub proof: Proof,
}

#[derive(Clone, RuntimeDebug, Encode, Decode, TypeInfo, PartialEq)]
pub enum DidMerkleProofError {
	UnsupportedVersion,
	KeyNotFound,
	LinkedAccountNotFound,
	Web3NameNotFound,
	Internal,
}

impl From<DidMerkleProofError> for u16 {
	fn from(value: DidMerkleProofError) -> Self {
		match value {
			DidMerkleProofError::UnsupportedVersion => 0,
			DidMerkleProofError::KeyNotFound => 1,
			DidMerkleProofError::LinkedAccountNotFound => 2,
			DidMerkleProofError::Web3NameNotFound => 3,
			DidMerkleProofError::Internal => u16::MAX,
		}
	}
}

pub mod v0 {
	use super::*;

	/// Type of a Merkle leaf revealed as part of a DIP Merkle proof.
	type ProofLeafOf<T> = RevealedDidMerkleProofLeaf<
		KeyIdOf<T>,
		<T as frame_system::Config>::AccountId,
		BlockNumberFor<T>,
		<T as pallet_web3_names::Config>::Web3Name,
		LinkableAccountId,
	>;

	/// Given the provided DID info, it calculates the Merkle commitment (root)
	/// using the provided in-memory DB.
	pub(super) fn calculate_root_with_db<Runtime, const MAX_LINKED_ACCOUNT: u32>(
		identity: &LinkedDidInfoOf<Runtime, MAX_LINKED_ACCOUNT>,
		db: &mut MemoryDB<Runtime::Hashing>,
	) -> Result<Runtime::Hash, DidMerkleProofError>
	where
		Runtime: did::Config + pallet_did_lookup::Config + pallet_web3_names::Config,
	{
		let LinkedDidInfoOf {
			did_details,
			web3_name_details,
			linked_accounts,
		} = identity;
		let mut trie = TrieHash::<LayoutV1<Runtime::Hashing>>::default();
		let mut trie_builder = TrieDBMutBuilder::<LayoutV1<Runtime::Hashing>>::new(db, &mut trie).build();

		// Authentication key
		let auth_key_details = did_details
			.public_keys
			.get(&did_details.authentication_key)
			.ok_or_else(|| {
				log::error!("Authentication key should be part of the public keys.");
				DidMerkleProofError::Internal
			})?;
		let auth_leaf = ProofLeafOf::<Runtime>::DidKey(
			DidKeyMerkleKey(
				did_details.authentication_key,
				DidVerificationKeyRelationship::Authentication.into(),
			),
			DidKeyMerkleValue(auth_key_details.clone()),
		);
		trie_builder
			.insert(auth_leaf.encoded_key().as_slice(), auth_leaf.encoded_value().as_slice())
			.map_err(|_| {
				log::error!(
					"Failed to insert authentication key in the trie builder. Authentication leaf: {:#?}",
					auth_leaf
				);
				DidMerkleProofError::Internal
			})?;
		// Attestation key, if present
		if let Some(att_key_id) = did_details.attestation_key {
			let att_key_details = did_details.public_keys.get(&att_key_id).ok_or_else(|| {
				log::error!("Attestation key should be part of the public keys.");
				DidMerkleProofError::Internal
			})?;
			let att_leaf = ProofLeafOf::<Runtime>::DidKey(
				(att_key_id, DidVerificationKeyRelationship::AssertionMethod.into()).into(),
				att_key_details.clone().into(),
			);
			trie_builder
				.insert(att_leaf.encoded_key().as_slice(), att_leaf.encoded_value().as_slice())
				.map_err(|_| {
					log::error!(
						"Failed to insert attestation key in the trie builder. Attestation leaf: {:#?}",
						att_leaf
					);
					DidMerkleProofError::Internal
				})?;
		};
		// Delegation key, if present
		if let Some(del_key_id) = did_details.delegation_key {
			let del_key_details = did_details.public_keys.get(&del_key_id).ok_or_else(|| {
				log::error!("Delegation key should be part of the public keys.");
				DidMerkleProofError::Internal
			})?;
			let del_leaf = ProofLeafOf::<Runtime>::DidKey(
				(del_key_id, DidVerificationKeyRelationship::CapabilityDelegation.into()).into(),
				del_key_details.clone().into(),
			);
			trie_builder
				.insert(del_leaf.encoded_key().as_slice(), del_leaf.encoded_value().as_slice())
				.map_err(|_| {
					log::error!(
						"Failed to insert delegation key in the trie builder. Delegation leaf: {:#?}",
						del_leaf
					);
					DidMerkleProofError::Internal
				})?;
		};
		// Key agreement keys
		did_details
			.key_agreement_keys
			.iter()
			.try_for_each(|id| -> Result<(), DidMerkleProofError> {
				let key_agreement_details = did_details.public_keys.get(id).ok_or_else(|| {
					log::error!("Key agreement key should be part of the public keys.");
					DidMerkleProofError::Internal
				})?;
				let enc_leaf = ProofLeafOf::<Runtime>::DidKey(
					(*id, DidKeyRelationship::Encryption).into(),
					key_agreement_details.clone().into(),
				);
				trie_builder
					.insert(enc_leaf.encoded_key().as_slice(), enc_leaf.encoded_value().as_slice())
					.map_err(|_| {
						log::error!(
							"Failed to insert key agreement key in the trie builder. Key agreement leaf: {:#?}",
							enc_leaf
						);
						DidMerkleProofError::Internal
					})?;
				Ok(())
			})?;

		// Linked accounts
		linked_accounts
			.iter()
			.try_for_each(|linked_account| -> Result<(), DidMerkleProofError> {
				let linked_account_leaf =
					ProofLeafOf::<Runtime>::LinkedAccount(linked_account.clone().into(), ().into());
				trie_builder
					.insert(
						linked_account_leaf.encoded_key().as_slice(),
						linked_account_leaf.encoded_value().as_slice(),
					)
					.map_err(|_| {
						log::error!(
							"Failed to insert linked account in the trie builder. Linked account leaf: {:#?}",
							linked_account_leaf
						);
						DidMerkleProofError::Internal
					})?;
				Ok(())
			})?;

		// Web3name, if present
		if let Some(web3name_details) = web3_name_details {
			let web3_name_leaf = ProofLeafOf::<Runtime>::Web3Name(
				web3name_details.web3_name.clone().into(),
				web3name_details.claimed_at.into(),
			);
			trie_builder
				.insert(
					web3_name_leaf.encoded_key().as_slice(),
					web3_name_leaf.encoded_value().as_slice(),
				)
				.map_err(|_| {
					log::error!(
						"Failed to insert web3name in the trie builder. Web3name leaf: {:#?}",
						web3_name_leaf
					);
					DidMerkleProofError::Internal
				})?;
		}

		trie_builder.commit();
		Ok(trie_builder.root().to_owned())
	}

	/// Given the provided DID info, and a set of DID key IDs, account IDs and a
	/// web3name, generates a Merkle proof that reveals only the provided
	/// identity components. The function fails if no key or account with the
	/// specified ID can be found, or if a web3name is requested to be revealed
	/// in the proof but is not present in the provided identity details.
	pub(super) fn generate_proof<'a, Runtime, K, A, const MAX_LINKED_ACCOUNT: u32>(
		identity: &LinkedDidInfoOf<Runtime, MAX_LINKED_ACCOUNT>,
		key_ids: K,
		should_include_web3_name: bool,
		account_ids: A,
	) -> Result<CompleteMerkleProof<Runtime::Hash, DidMerkleProofOf<Runtime>>, DidMerkleProofError>
	where
		Runtime: did::Config + pallet_did_lookup::Config + pallet_web3_names::Config,
		K: Iterator<Item = &'a KeyIdOf<Runtime>>,
		A: Iterator<Item = &'a LinkableAccountId>,
	{
		let LinkedDidInfoOf {
			did_details,
			web3_name_details,
			linked_accounts,
		} = identity;

		let mut db = MemoryDB::default();
		let root = calculate_root_with_db(identity, &mut db)?;

		let mut leaves = key_ids
			.map(|key_id| -> Result<_, DidMerkleProofError> {
				let key_details = did_details
					.public_keys
					.get(key_id)
					.ok_or(DidMerkleProofError::KeyNotFound)?;
				// Create the merkle leaf key depending on the relationship of the key to the
				// DID document.
				let did_key_merkle_key: DidKeyMerkleKey<KeyIdOf<Runtime>> = if *key_id == did_details.authentication_key
				{
					Ok((*key_id, DidVerificationKeyRelationship::Authentication.into()).into())
				} else if Some(*key_id) == did_details.attestation_key {
					Ok((*key_id, DidVerificationKeyRelationship::AssertionMethod.into()).into())
				} else if Some(*key_id) == did_details.delegation_key {
					Ok((*key_id, DidVerificationKeyRelationship::CapabilityDelegation.into()).into())
				} else if did_details.key_agreement_keys.contains(key_id) {
					Ok((*key_id, DidKeyRelationship::Encryption).into())
				} else {
					log::error!("Unknown key ID {:#?} retrieved from DID details.", key_id);
					Err(DidMerkleProofError::Internal)
				}?;
				Ok(RevealedDidMerkleProofLeaf::DidKey(
					did_key_merkle_key,
					key_details.clone().into(),
				))
			})
			.chain(account_ids.map(|account_id| -> Result<_, DidMerkleProofError> {
				if linked_accounts.contains(account_id) {
					Ok(RevealedDidMerkleProofLeaf::LinkedAccount(
						account_id.clone().into(),
						().into(),
					))
				} else {
					Err(DidMerkleProofError::LinkedAccountNotFound)
				}
			}))
			.collect::<Result<Vec<_>, _>>()?;

		match (should_include_web3_name, web3_name_details) {
			// If web3name should be included and it exists, add to the leaves to be revealed...
			(true, Some(web3name_details)) => {
				leaves.push(RevealedDidMerkleProofLeaf::Web3Name(
					web3name_details.web3_name.clone().into(),
					web3name_details.claimed_at.into(),
				));
			}
			// ...else if web3name should be included and it DOES NOT exist, return an error...
			(true, None) => return Err(DidMerkleProofError::Web3NameNotFound),
			// ...else (if web3name should NOT be included), skip.
			(false, _) => {}
		};

		let encoded_keys: Vec<Vec<u8>> = leaves.iter().map(|l| l.encoded_key()).collect();
		let proof =
			generate_trie_proof::<LayoutV1<Runtime::Hashing>, _, _, _>(&db, root, &encoded_keys).map_err(|_| {
				log::error!(
					"Failed to generate a merkle proof for the encoded keys: {:#?}",
					encoded_keys
				);
				DidMerkleProofError::Internal
			})?;
		Ok(CompleteMerkleProof {
			root,
			proof: DidMerkleProofOf::<Runtime> {
				blinded: proof,
				revealed: leaves.into_iter().collect(),
			},
		})
	}

	/// Given the provided DID info, generates a Merkle commitment (root).
	pub(super) fn generate_commitment<Runtime, const MAX_LINKED_ACCOUNT: u32>(
		identity: &IdentityOf<Runtime>,
	) -> Result<Runtime::Hash, DidMerkleProofError>
	where
		Runtime: did::Config + pallet_did_lookup::Config + pallet_web3_names::Config + pallet_dip_provider::Config,
		Runtime::IdentityProvider: IdentityProvider<Runtime, Success = LinkedDidInfoOf<Runtime, MAX_LINKED_ACCOUNT>>,
	{
		let mut db = MemoryDB::default();
		calculate_root_with_db(identity, &mut db)
	}
}

/// Type implementing the [`IdentityCommitmentGenerator`] and generating a
/// Merkle root of the provided identity details, according to the description
/// provided in the [README.md](./README.md),
pub struct DidMerkleRootGenerator<T>(PhantomData<T>);

impl<Runtime, const MAX_LINKED_ACCOUNT: u32> IdentityCommitmentGenerator<Runtime> for DidMerkleRootGenerator<Runtime>
where
	Runtime: did::Config + pallet_did_lookup::Config + pallet_web3_names::Config + pallet_dip_provider::Config,
	Runtime::IdentityProvider: IdentityProvider<Runtime, Success = LinkedDidInfoOf<Runtime, MAX_LINKED_ACCOUNT>>,
{
	type Error = DidMerkleProofError;
	type Output = Runtime::Hash;

	fn generate_commitment(
		_identifier: &Runtime::Identifier,
		identity: &IdentityOf<Runtime>,
		version: IdentityCommitmentVersion,
	) -> Result<Self::Output, Self::Error> {
		match version {
			0 => v0::generate_commitment::<Runtime, MAX_LINKED_ACCOUNT>(identity),
			_ => Err(DidMerkleProofError::UnsupportedVersion),
		}
	}
}

impl<Runtime> DidMerkleRootGenerator<Runtime>
where
	Runtime: did::Config + pallet_did_lookup::Config + pallet_web3_names::Config,
{
	pub fn generate_proof<'a, K, A, const MAX_LINKED_ACCOUNT: u32>(
		identity: &LinkedDidInfoOf<Runtime, MAX_LINKED_ACCOUNT>,
		version: IdentityCommitmentVersion,
		key_ids: K,
		should_include_web3_name: bool,
		account_ids: A,
	) -> Result<CompleteMerkleProof<Runtime::Hash, DidMerkleProofOf<Runtime>>, DidMerkleProofError>
	where
		K: Iterator<Item = &'a KeyIdOf<Runtime>>,
		A: Iterator<Item = &'a LinkableAccountId>,
	{
		match version {
			0 => v0::generate_proof(identity, key_ids, should_include_web3_name, account_ids),
			_ => Err(DidMerkleProofError::UnsupportedVersion),
		}
	}
}
