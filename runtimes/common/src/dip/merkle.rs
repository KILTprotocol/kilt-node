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
use pallet_dip_provider::traits::IdentityProofGenerator;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::{borrow::ToOwned, collections::btree_set::BTreeSet, marker::PhantomData, vec::Vec};
use sp_trie::{generate_trie_proof, LayoutV1, MemoryDB, TrieDBMutBuilder, TrieHash, TrieMut};

use kilt_dip_support::merkle::{DidKeyRelationship, ProofLeaf};

use crate::{dip::did::LinkedDidInfoOf, DidIdentifier};

pub type BlindedValue = Vec<u8>;
pub type DidMerkleProofOf<T> = MerkleProof<
	Vec<BlindedValue>,
	ProofLeaf<KeyIdOf<T>, <T as frame_system::Config>::BlockNumber, <T as pallet_web3_names::Config>::Web3Name>,
>;

#[derive(Encode, Decode, RuntimeDebug, PartialEq, Eq, TypeInfo)]
pub struct CompleteMerkleProof<Root, Proof> {
	pub root: Root,
	pub proof: Proof,
}

pub struct DidMerkleRootGenerator<T>(PhantomData<T>);

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
		let (Some(did_details), _web3_name, _linked_accounts) = (&identity.a, &identity.b, &identity.c) else { return Err(()) };
		let mut trie = TrieHash::<LayoutV1<T::Hashing>>::default();
		let mut trie_builder = TrieDBMutBuilder::<LayoutV1<T::Hashing>>::new(db, &mut trie).build();

		// Authentication key
		// TODO: No panic
		let auth_key_details = did_details
			.public_keys
			.get(&did_details.authentication_key)
			.expect("Authentication key should be part of the public keys.");
		let auth_leaf = ProofLeaf::<_, _, T::Web3Name>::DidKey(
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
			let att_leaf = ProofLeaf::<_, _, T::Web3Name>::DidKey(
				DidKeyMerkleKey(att_key_id, DidVerificationKeyRelationship::AssertionMethod.into()),
				DidKeyMerkleValue(att_key_details.clone()),
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
			let del_leaf = ProofLeaf::<_, _, T::Web3Name>::DidKey(
				DidKeyMerkleKey(del_key_id, DidVerificationKeyRelationship::CapabilityDelegation.into()),
				DidKeyMerkleValue(del_key_details.clone()),
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
				let enc_leaf = ProofLeaf::<_, _, T::Web3Name>::DidKey(
					DidKeyMerkleKey(*id, DidKeyRelationship::Encryption),
					DidKeyMerkleValue(key_agreement_details.clone()),
				);
				trie_builder
					.insert(enc_leaf.encoded_key().as_slice(), enc_leaf.encoded_value().as_slice())
					.map_err(|_| ())?;
				Ok(())
			})?;
		trie_builder.commit();
		Ok(trie_builder.root().to_owned())
	}

	// TODO: Better error handling
	// Only used for testing and as part of the features exposed by the runtime API
	// of the provider. Given the provided `DidDetails` and a list of key IDs, it
	// generates a merkle proof which only reveals the details of the provided key
	// IDs.
	#[allow(clippy::result_unit_err)]
	pub fn generate_proof<'a, K>(
		identity: &LinkedDidInfoOf<T>,
		mut key_ids: K,
	) -> Result<CompleteMerkleProof<T::Hash, DidMerkleProofOf<T>>, ()>
	where
		K: Iterator<Item = &'a KeyIdOf<T>>,
	{
		let (Some(did_details), _web3_name, _linked_accounts) = (&identity.a, &identity.b, &identity.c) else { return Err(()) };

		let mut db = MemoryDB::default();
		let root = Self::calculate_root_with_db(identity, &mut db)?;

		#[allow(clippy::type_complexity)]
		let leaves: BTreeSet<ProofLeaf<KeyIdOf<T>, T::BlockNumber, T::Web3Name>> =
			key_ids.try_fold(BTreeSet::new(), |mut set, key_id| -> Result<_, ()> {
				let key_details = did_details.public_keys.get(key_id).ok_or(())?;
				// Create the merkle leaf key depending on the relationship of the key to the
				// DID document.
				let did_key_merkle_key = if *key_id == did_details.authentication_key {
					Ok(DidKeyMerkleKey(
						*key_id,
						DidVerificationKeyRelationship::Authentication.into(),
					))
				} else if Some(*key_id) == did_details.attestation_key {
					Ok(DidKeyMerkleKey(
						*key_id,
						DidVerificationKeyRelationship::AssertionMethod.into(),
					))
				} else if Some(*key_id) == did_details.delegation_key {
					Ok(DidKeyMerkleKey(
						*key_id,
						DidVerificationKeyRelationship::CapabilityDelegation.into(),
					))
				} else if did_details.key_agreement_keys.contains(key_id) {
					Ok(DidKeyMerkleKey(*key_id, DidKeyRelationship::Encryption))
				} else {
					Err(())
				}?;
				// Then adds the actual key details to the merkle leaf.
				let did_key_merkle_value = DidKeyMerkleValue(key_details.clone());
				let did_merkle_merkle_leaf = ProofLeaf::DidKey(did_key_merkle_key, did_key_merkle_value);
				if !set.contains(&did_merkle_merkle_leaf) {
					set.insert(did_merkle_merkle_leaf);
				}
				Ok(set)
			})?;
		let encoded_keys: Vec<Vec<u8>> = leaves.iter().map(|l| l.encoded_key()).collect();
		let proof = generate_trie_proof::<LayoutV1<T::Hashing>, _, _, _>(&db, root, &encoded_keys).map_err(|_| ())?;
		Ok(CompleteMerkleProof {
			root,
			proof: DidMerkleProofOf::<T> {
				blinded: proof,
				revealed: leaves.into_iter().collect::<Vec<_>>(),
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
