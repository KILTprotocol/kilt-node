// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2024 BOTLabs GmbH

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
use kilt_dip_primitives::DidMerkleProof;
use pallet_did_lookup::linkable_account::LinkableAccountId;
use pallet_dip_provider::{
	traits::{IdentityCommitmentGenerator, IdentityProvider},
	IdentityCommitmentVersion, IdentityOf,
};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::{borrow::ToOwned, marker::PhantomData, vec::Vec};
use sp_trie::{generate_trie_proof, LayoutV1, MemoryDB, TrieDBMutBuilder, TrieHash, TrieMut};

use crate::dip::did::LinkedDidInfoOf;

/// Type of the Merkle proof revealing parts of the DIP identity of a given DID
/// subject.
pub type DidMerkleProofOf<T> = DidMerkleProof<
	KeyIdOf<T>,
	<T as frame_system::Config>::AccountId,
	BlockNumberFor<T>,
	<T as pallet_web3_names::Config>::Web3Name,
	LinkableAccountId,
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
	TooManyLeaves,
	Internal,
}

impl From<DidMerkleProofError> for u16 {
	fn from(value: DidMerkleProofError) -> Self {
		match value {
			// DO NOT USE 0
			// Errors of different sub-parts are separated by a `u8::MAX`.
			// A value of 0 would make it confusing whether it's the previous sub-part error (u8::MAX)
			// or the new sub-part error (u8::MAX + 0).
			DidMerkleProofError::UnsupportedVersion => 1,
			DidMerkleProofError::KeyNotFound => 2,
			DidMerkleProofError::LinkedAccountNotFound => 3,
			DidMerkleProofError::Web3NameNotFound => 4,
			DidMerkleProofError::TooManyLeaves => 5,
			DidMerkleProofError::Internal => u16::MAX,
		}
	}
}

pub mod v0 {
	use did::did_details::DidDetails;
	use kilt_dip_primitives::{
		DidKeyRelationship, RevealedAccountId, RevealedDidKey, RevealedDidMerkleProofLeaf, RevealedWeb3Name,
	};
	use pallet_web3_names::Web3NameOf;
	use sp_std::vec;

	use crate::dip::did::Web3OwnershipOf;

	use super::*;

	fn get_auth_leaves<Runtime>(
		did_details: &DidDetails<Runtime>,
	) -> Result<
		impl Iterator<Item = RevealedDidKey<KeyIdOf<Runtime>, BlockNumberFor<Runtime>, Runtime::AccountId>>,
		DidMerkleProofError,
	>
	where
		Runtime: did::Config,
	{
		let auth_key_details = did_details
			.public_keys
			.get(&did_details.authentication_key)
			.ok_or_else(|| {
				log::error!("Authentication key should be part of the public keys.");
				DidMerkleProofError::Internal
			})?;
		Ok([RevealedDidKey {
			id: did_details.authentication_key,
			relationship: DidVerificationKeyRelationship::Authentication.into(),
			details: auth_key_details.clone(),
		}]
		.into_iter())
	}

	fn get_att_leaves<Runtime>(
		did_details: &DidDetails<Runtime>,
	) -> Result<
		impl Iterator<Item = RevealedDidKey<KeyIdOf<Runtime>, BlockNumberFor<Runtime>, Runtime::AccountId>>,
		DidMerkleProofError,
	>
	where
		Runtime: did::Config,
	{
		let Some(att_key_id) = did_details.attestation_key else {
			return Ok(vec![].into_iter());
		};
		let att_key_details = did_details.public_keys.get(&att_key_id).ok_or_else(|| {
			log::error!("Attestation key should be part of the public keys.");
			DidMerkleProofError::Internal
		})?;
		Ok(vec![RevealedDidKey {
			id: att_key_id,
			relationship: DidVerificationKeyRelationship::AssertionMethod.into(),
			details: att_key_details.clone(),
		}]
		.into_iter())
	}

	fn get_del_leaves<Runtime>(
		did_details: &DidDetails<Runtime>,
	) -> Result<
		impl Iterator<Item = RevealedDidKey<KeyIdOf<Runtime>, BlockNumberFor<Runtime>, Runtime::AccountId>>,
		DidMerkleProofError,
	>
	where
		Runtime: did::Config,
	{
		let Some(del_key_id) = did_details.delegation_key else {
			return Ok(vec![].into_iter());
		};
		let del_key_details = did_details.public_keys.get(&del_key_id).ok_or_else(|| {
			log::error!("Delegation key should be part of the public keys.");
			DidMerkleProofError::Internal
		})?;
		Ok(vec![RevealedDidKey {
			id: del_key_id,
			relationship: DidVerificationKeyRelationship::CapabilityDelegation.into(),
			details: del_key_details.clone(),
		}]
		.into_iter())
	}

	fn get_enc_leaves<Runtime>(
		did_details: &DidDetails<Runtime>,
	) -> Result<
		impl Iterator<Item = RevealedDidKey<KeyIdOf<Runtime>, BlockNumberFor<Runtime>, Runtime::AccountId>>,
		DidMerkleProofError,
	>
	where
		Runtime: did::Config,
	{
		let keys = did_details
			.key_agreement_keys
			.iter()
			.map(|id| {
				let key_agreement_details = did_details.public_keys.get(id).ok_or_else(|| {
					log::error!("Key agreement key should be part of the public keys.");
					DidMerkleProofError::Internal
				})?;
				Ok(RevealedDidKey {
					id: *id,
					relationship: DidKeyRelationship::Encryption,
					details: key_agreement_details.clone(),
				})
			})
			.collect::<Result<Vec<_>, _>>()?;
		Ok(keys.into_iter())
	}

	fn get_linked_account_leaves(
		linked_accounts: &[LinkableAccountId],
	) -> impl Iterator<Item = RevealedAccountId<LinkableAccountId>> + '_ {
		linked_accounts.iter().cloned().map(RevealedAccountId)
	}

	fn get_web3name_leaf<Runtime>(
		web3name_details: &Web3OwnershipOf<Runtime>,
	) -> RevealedWeb3Name<Web3NameOf<Runtime>, BlockNumberFor<Runtime>>
	where
		Runtime: pallet_web3_names::Config,
	{
		RevealedWeb3Name {
			web3_name: web3name_details.web3_name.clone(),
			claimed_at: web3name_details.claimed_at,
		}
	}

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

		// Authentication key.
		let auth_leaves = get_auth_leaves(did_details)?;
		// Attestation key, if present.
		let att_leaves = get_att_leaves(did_details)?;
		// Delegation key, if present.
		let del_leaves = get_del_leaves(did_details)?;
		// Key agreement keys.
		let enc_leaves = get_enc_leaves(did_details)?;
		// Linked accounts.
		let linked_accounts = get_linked_account_leaves(linked_accounts);
		// Web3name.
		let web3_name = web3_name_details.as_ref().map(get_web3name_leaf::<Runtime>);

		// Add all leaves to the proof builder.
		let keys = auth_leaves
			.chain(att_leaves)
			.chain(del_leaves)
			.chain(enc_leaves)
			.map(RevealedDidMerkleProofLeaf::from);
		let linked_accounts = linked_accounts.map(RevealedDidMerkleProofLeaf::from);
		let web3_names = web3_name
			.map(|n| vec![n])
			.unwrap_or_default()
			.into_iter()
			.map(RevealedDidMerkleProofLeaf::from);

		keys.chain(linked_accounts).chain(web3_names).try_for_each(|leaf| {
			trie_builder
				.insert(leaf.encoded_key().as_slice(), leaf.encoded_value().as_slice())
				.map_err(|_| {
					log::error!("Failed to insert leaf in the trie builder. Leaf: {:#?}", leaf);
					DidMerkleProofError::Internal
				})?;
			Ok(())
		})?;

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
				let key_relationship = if *key_id == did_details.authentication_key {
					Ok(DidVerificationKeyRelationship::Authentication.into())
				} else if Some(*key_id) == did_details.attestation_key {
					Ok(DidVerificationKeyRelationship::AssertionMethod.into())
				} else if Some(*key_id) == did_details.delegation_key {
					Ok(DidVerificationKeyRelationship::CapabilityDelegation.into())
				} else if did_details.key_agreement_keys.contains(key_id) {
					Ok(DidKeyRelationship::Encryption)
				} else {
					log::error!("Unknown key ID {:#?} retrieved from DID details.", key_id);
					Err(DidMerkleProofError::Internal)
				}?;
				Ok(RevealedDidMerkleProofLeaf::from(RevealedDidKey {
					id: *key_id,
					relationship: key_relationship,
					details: key_details.clone(),
				}))
			})
			.chain(account_ids.map(|account_id| -> Result<_, DidMerkleProofError> {
				if linked_accounts.contains(account_id) {
					Ok(RevealedDidMerkleProofLeaf::from(RevealedAccountId(account_id.clone())))
				} else {
					Err(DidMerkleProofError::LinkedAccountNotFound)
				}
			}))
			.collect::<Result<Vec<_>, _>>()?;

		match (should_include_web3_name, web3_name_details) {
			// If web3name should be included and it exists, add to the leaves to be revealed...
			(true, Some(web3name_details)) => {
				leaves.push(RevealedDidMerkleProofLeaf::from(RevealedWeb3Name {
					web3_name: web3name_details.web3_name.clone(),
					claimed_at: web3name_details.claimed_at,
				}));
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
			proof: DidMerkleProofOf::<Runtime>::new(proof.into_iter().into(), leaves),
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
