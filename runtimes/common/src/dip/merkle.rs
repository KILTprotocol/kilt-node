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
use kilt_dip_support::merkle::{DidKeyMerkleKey, DidKeyMerkleValue, MerkleProof};
use pallet_did_lookup::linkable_account::LinkableAccountId;
use pallet_dip_provider::traits::IdentityProofGenerator;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::{borrow::ToOwned, marker::PhantomData, vec::Vec};
use sp_trie::{generate_trie_proof, LayoutV1, MemoryDB, TrieDBMutBuilder, TrieHash, TrieMut};

use kilt_dip_support::merkle::{DidKeyRelationship, ProofLeaf};

use crate::{dip::did::LinkedDidInfoOf, DidIdentifier};

pub type BlindedValue = Vec<u8>;
pub type DidMerkleProofOf<T> = MerkleProof<
	Vec<BlindedValue>,
	ProofLeaf<
		KeyIdOf<T>,
		<T as frame_system::Config>::BlockNumber,
		<T as pallet_web3_names::Config>::Web3Name,
		LinkableAccountId,
	>,
>;

#[derive(Encode, Decode, RuntimeDebug, PartialEq, Eq, TypeInfo)]
pub struct CompleteMerkleProof<Root, Proof> {
	pub root: Root,
	pub proof: Proof,
}

pub struct DidMerkleRootGenerator<T>(PhantomData<T>);

type ProofLeafOf<T> = ProofLeaf<
	KeyIdOf<T>,
	<T as frame_system::Config>::BlockNumber,
	<T as pallet_web3_names::Config>::Web3Name,
	LinkableAccountId,
>;

impl<T> DidMerkleRootGenerator<T>
where
	T: did::Config + pallet_did_lookup::Config + pallet_web3_names::Config,
{
	// Calls the function in the `sp_trie` crate to generate the merkle root given
	// the provided `DidDetails`.
	// Each key in the merkle tree is added in the following way:
	// - keys in the `public_keys` map are added by value in the merkle tree, with
	//   the leaf key being the key ID and the value being the key details
	// - keys everywhere else in the DidDetails are added by reference, with the
	//   leaf key being the encoding of the tuple (keyID, key relationship) and the
	//   value being hte empty tuple
	// A valid proof will contain a leaf with the key details for each reference
	// leaf, with multiple reference leaves potentially referring to the same
	// details leaf, as we already do with out `DidDetails` type.
	fn calculate_root_with_db(identity: &LinkedDidInfoOf<T>, db: &mut MemoryDB<T::Hashing>) -> Result<T::Hash, ()> {
		// Fails if the DID details do not exist.
		let (Some(did_details), web3_name, linked_accounts) = (&identity.a, &identity.b, &identity.c) else { return Err(()) };
		let mut trie = TrieHash::<LayoutV1<T::Hashing>>::default();
		let mut trie_builder = TrieDBMutBuilder::<LayoutV1<T::Hashing>>::new(db, &mut trie).build();

		// Authentication key
		// TODO: No panic
		let auth_key_details = did_details
			.public_keys
			.get(&did_details.authentication_key)
			.expect("Authentication key should be part of the public keys.");
		let auth_leaf = ProofLeafOf::<T>::DidKey(
			DidKeyMerkleKey(
				did_details.authentication_key,
				DidVerificationKeyRelationship::Authentication.into(),
			),
			DidKeyMerkleValue(auth_key_details.clone()),
		);
		trie_builder
			.insert(auth_leaf.encoded_key().as_slice(), auth_leaf.encoded_value().as_slice())
			.map_err(|_| ())?;
		// Attestation key, if present
		if let Some(att_key_id) = did_details.attestation_key {
			let att_key_details = did_details
				.public_keys
				.get(&att_key_id)
				.expect("Attestation key should be part of the public keys.");
			let att_leaf = ProofLeafOf::<T>::DidKey(
				(att_key_id, DidVerificationKeyRelationship::AssertionMethod.into()).into(),
				att_key_details.clone().into(),
			);
			trie_builder
				.insert(att_leaf.encoded_key().as_slice(), att_leaf.encoded_value().as_slice())
				.map_err(|_| ())?;
		};
		// Delegation key, if present
		if let Some(del_key_id) = did_details.delegation_key {
			let del_key_details = did_details
				.public_keys
				.get(&del_key_id)
				.expect("Delegation key should be part of the public keys.");
			let del_leaf = ProofLeafOf::<T>::DidKey(
				(del_key_id, DidVerificationKeyRelationship::CapabilityDelegation.into()).into(),
				del_key_details.clone().into(),
			);
			trie_builder
				.insert(del_leaf.encoded_key().as_slice(), del_leaf.encoded_value().as_slice())
				.map_err(|_| ())?;
		};
		// Key agreement keys
		did_details
			.key_agreement_keys
			.iter()
			.try_for_each(|id| -> Result<(), ()> {
				let key_agreement_details = did_details
					.public_keys
					.get(id)
					.expect("Key agreement key should be part of the public keys.");
				let enc_leaf = ProofLeafOf::<T>::DidKey(
					(*id, DidKeyRelationship::Encryption).into(),
					key_agreement_details.clone().into(),
				);
				trie_builder
					.insert(enc_leaf.encoded_key().as_slice(), enc_leaf.encoded_value().as_slice())
					.map_err(|_| ())?;
				Ok(())
			})?;

		// Linked accounts
		if let Some(linked_accounts) = linked_accounts {
			linked_accounts
				.iter()
				.try_for_each(|linked_account| -> Result<(), ()> {
					let linked_account_leaf = ProofLeafOf::<T>::LinkedAccount(linked_account.clone().into(), ().into());
					trie_builder
						.insert(
							linked_account_leaf.encoded_key().as_slice(),
							linked_account_leaf.encoded_value().as_slice(),
						)
						.map_err(|_| ())?;
					Ok(())
				})?;
		}

		// Web3name, if present
		if let Some(web3name_details) = web3_name {
			let web3_name_leaf = ProofLeafOf::<T>::Web3Name(
				web3name_details.web3_name.clone().into(),
				web3name_details.claimed_at.into(),
			);
			trie_builder
				.insert(
					web3_name_leaf.encoded_key().as_slice(),
					web3_name_leaf.encoded_value().as_slice(),
				)
				.map_err(|_| ())?;
		}

		trie_builder.commit();
		Ok(trie_builder.root().to_owned())
	}

	// TODO: Better error handling
	// Only used for testing and as part of the features exposed by the runtime API
	// of the provider. Given the provided `DidDetails` and a list of key IDs, it
	// generates a merkle proof which only reveals the details of the provided key
	// IDs.
	#[allow(clippy::result_unit_err)]
	pub fn generate_proof<'a, K, A>(
		identity: &LinkedDidInfoOf<T>,
		key_ids: K,
		should_include_web3_name: bool,
		account_ids: A,
	) -> Result<CompleteMerkleProof<T::Hash, DidMerkleProofOf<T>>, ()>
	where
		K: Iterator<Item = &'a KeyIdOf<T>>,
		A: Iterator<Item = &'a LinkableAccountId>,
	{
		// Fails if the DID details do not exist.
		let (Some(did_details), linked_web3_name, linked_accounts) = (&identity.a, &identity.b, &identity.c) else { return Err(()) };

		let mut db = MemoryDB::default();
		let root = Self::calculate_root_with_db(identity, &mut db)?;

		let mut leaves = key_ids
			.map(|key_id| -> Result<ProofLeafOf<T>, ()> {
				let key_details = did_details.public_keys.get(key_id).ok_or(())?;
				// Create the merkle leaf key depending on the relationship of the key to the
				// DID document.
				let did_key_merkle_key: DidKeyMerkleKey<KeyIdOf<T>> = if *key_id == did_details.authentication_key {
					Ok((*key_id, DidVerificationKeyRelationship::Authentication.into()).into())
				} else if Some(*key_id) == did_details.attestation_key {
					Ok((*key_id, DidVerificationKeyRelationship::AssertionMethod.into()).into())
				} else if Some(*key_id) == did_details.delegation_key {
					Ok((*key_id, DidVerificationKeyRelationship::CapabilityDelegation.into()).into())
				} else if did_details.key_agreement_keys.contains(key_id) {
					Ok((*key_id, DidKeyRelationship::Encryption).into())
				} else {
					Err(())
				}?;
				Ok(ProofLeaf::DidKey(did_key_merkle_key, key_details.clone().into()))
			})
			.chain(account_ids.map(|account_id| -> Result<ProofLeafOf<T>, ()> {
				let Some(linked_accounts) = linked_accounts else { return Err(()) };
				if linked_accounts.contains(account_id) {
					Ok(ProofLeaf::LinkedAccount(account_id.clone().into(), ().into()))
				} else {
					Err(())
				}
			}))
			.collect::<Result<Vec<_>, _>>()?;

		match (should_include_web3_name, linked_web3_name) {
			// If web3name should be included and it exists...
			(true, Some(web3name_details)) => {
				leaves.push(ProofLeaf::Web3Name(
					web3name_details.web3_name.clone().into(),
					web3name_details.claimed_at.into(),
				));
				Ok(())
			}
			// ...else if web3name should be included and it DOES NOT exist...
			(true, None) => Err(()),
			// ...else if web3name should NOT be included.
			(false, _) => Ok(()),
		}?;

		let encoded_keys: Vec<Vec<u8>> = leaves.iter().map(|l| l.encoded_key()).collect();
		let proof = generate_trie_proof::<LayoutV1<T::Hashing>, _, _, _>(&db, root, &encoded_keys).map_err(|_| ())?;
		Ok(CompleteMerkleProof {
			root,
			proof: DidMerkleProofOf::<T> {
				blinded: proof,
				revealed: leaves.into_iter().collect(),
			},
		})
	}
}

impl<T> IdentityProofGenerator<DidIdentifier, LinkedDidInfoOf<T>> for DidMerkleRootGenerator<T>
where
	T: did::Config + pallet_did_lookup::Config + pallet_web3_names::Config,
{
	// TODO: Proper error handling
	type Error = ();
	type Output = T::Hash;

	fn generate_commitment(_identifier: &DidIdentifier, identity: &LinkedDidInfoOf<T>) -> Result<T::Hash, Self::Error> {
		let mut db = MemoryDB::default();
		Self::calculate_root_with_db(identity, &mut db)
	}
}
