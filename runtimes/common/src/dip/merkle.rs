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
use kilt_dip_support::merkle::{DidKeyMerkleKey, DidKeyMerkleValue, DidMerkleProof};
use pallet_did_lookup::linkable_account::LinkableAccountId;
use pallet_dip_provider::{traits::IdentityCommitmentGenerator, IdentityCommitmentVersion};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::{borrow::ToOwned, marker::PhantomData, vec::Vec};
use sp_trie::{generate_trie_proof, LayoutV1, MemoryDB, TrieDBMutBuilder, TrieHash, TrieMut};

use kilt_dip_support::merkle::{DidKeyRelationship, RevealedDidMerkleProofLeaf};

use crate::{dip::did::LinkedDidInfoOf, DidIdentifier};

pub type BlindedValue = Vec<u8>;
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

#[derive(Encode, Decode, RuntimeDebug, PartialEq, Eq, TypeInfo)]
pub struct CompleteMerkleProof<Root, Proof> {
	pub root: Root,
	pub proof: Proof,
}

#[derive(Clone, RuntimeDebug, Encode, Decode, TypeInfo, PartialEq)]
pub enum DidMerkleProofError {
	UnsupportedVersion,
	DidNotFound,
	KeyNotFound,
	LinkedAccountNotFound,
	Web3NameNotFound,
	Internal,
}

impl From<DidMerkleProofError> for u16 {
	fn from(value: DidMerkleProofError) -> Self {
		match value {
			DidMerkleProofError::UnsupportedVersion => 0,
			DidMerkleProofError::DidNotFound => 1,
			DidMerkleProofError::KeyNotFound => 2,
			DidMerkleProofError::LinkedAccountNotFound => 3,
			DidMerkleProofError::Web3NameNotFound => 4,
			DidMerkleProofError::Internal => u16::MAX,
		}
	}
}

pub mod v0 {
	use super::*;

	type ProofLeafOf<T> = RevealedDidMerkleProofLeaf<
		KeyIdOf<T>,
		<T as frame_system::Config>::AccountId,
		BlockNumberFor<T>,
		<T as pallet_web3_names::Config>::Web3Name,
		LinkableAccountId,
	>;

	pub(super) fn calculate_root_with_db<Runtime>(
		identity: &LinkedDidInfoOf<Runtime>,
		db: &mut MemoryDB<Runtime::Hashing>,
	) -> Result<Runtime::Hash, DidMerkleProofError>
	where
		Runtime: did::Config + pallet_did_lookup::Config + pallet_web3_names::Config,
	{
		// Fails if the DID details do not exist.
		let (Some(did_details), web3_name, linked_accounts) = (&identity.a, &identity.b, &identity.c) else {
			return Err(DidMerkleProofError::DidNotFound);
		};
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
		if let Some(linked_accounts) = linked_accounts {
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
		}

		// Web3name, if present
		if let Some(web3name_details) = web3_name {
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

	pub(super) fn generate_proof<'a, Runtime, K, A>(
		identity: &LinkedDidInfoOf<Runtime>,
		key_ids: K,
		should_include_web3_name: bool,
		account_ids: A,
	) -> Result<CompleteMerkleProof<Runtime::Hash, DidMerkleProofOf<Runtime>>, DidMerkleProofError>
	where
		Runtime: did::Config + pallet_did_lookup::Config + pallet_web3_names::Config,
		K: Iterator<Item = &'a KeyIdOf<Runtime>>,
		A: Iterator<Item = &'a LinkableAccountId>,
	{
		// Fails if the DID details do not exist.
		let (Some(did_details), linked_web3_name, linked_accounts) = (&identity.a, &identity.b, &identity.c) else {
			return Err(DidMerkleProofError::DidNotFound);
		};

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
				let Some(linked_accounts) = linked_accounts else {
					// Directly LinkedAccountNotFound since there's no linked accounts to check
					// against.
					return Err(DidMerkleProofError::LinkedAccountNotFound);
				};
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

		match (should_include_web3_name, linked_web3_name) {
			// If web3name should be included and it exists...
			(true, Some(web3name_details)) => {
				leaves.push(RevealedDidMerkleProofLeaf::Web3Name(
					web3name_details.web3_name.clone().into(),
					web3name_details.claimed_at.into(),
				));
				Ok(())
			}
			// ...else if web3name should be included and it DOES NOT exist...
			(true, None) => Err(DidMerkleProofError::Web3NameNotFound),
			// ...else (if web3name should NOT be included).
			(false, _) => Ok(()),
		}?;

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

	pub(super) fn generate_commitment<Runtime>(
		identity: &LinkedDidInfoOf<Runtime>,
	) -> Result<Runtime::Hash, DidMerkleProofError>
	where
		Runtime: did::Config + pallet_did_lookup::Config + pallet_web3_names::Config,
	{
		let mut db = MemoryDB::default();
		calculate_root_with_db(identity, &mut db)
	}
}

pub struct DidMerkleRootGenerator<T>(PhantomData<T>);

impl<Runtime> IdentityCommitmentGenerator<DidIdentifier, LinkedDidInfoOf<Runtime>> for DidMerkleRootGenerator<Runtime>
where
	Runtime: did::Config + pallet_did_lookup::Config + pallet_web3_names::Config,
{
	type Error = DidMerkleProofError;
	type Output = Runtime::Hash;

	fn generate_commitment(
		_identifier: &DidIdentifier,
		identity: &LinkedDidInfoOf<Runtime>,
		version: IdentityCommitmentVersion,
	) -> Result<Runtime::Hash, Self::Error> {
		match version {
			0 => v0::generate_commitment::<Runtime>(identity),
			_ => Err(DidMerkleProofError::UnsupportedVersion),
		}
	}
}

impl<Runtime> DidMerkleRootGenerator<Runtime>
where
	Runtime: did::Config + pallet_did_lookup::Config + pallet_web3_names::Config,
{
	pub fn generate_proof<'a, K, A>(
		identity: &LinkedDidInfoOf<Runtime>,
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
