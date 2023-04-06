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

use did::{did_details::DidDetails, DidVerificationKeyRelationship, KeyIdOf};
use dip_support::latest::Proof;
use frame_support::RuntimeDebug;
use pallet_dip_sender::traits::{IdentityProofGenerator, IdentityProvider};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::{borrow::ToOwned, collections::btree_set::BTreeSet, marker::PhantomData, vec::Vec};
use sp_trie::{generate_trie_proof, LayoutV1, MemoryDB, TrieDBMutBuilder, TrieHash, TrieMut};

use crate::dip::{KeyDetailsKey, KeyDetailsValue, KeyReferenceKey, KeyReferenceValue, KeyRelationship, ProofLeaf};

pub type BlindedValue = Vec<u8>;

pub type DidMerkleProof<T> = Proof<Vec<BlindedValue>, ProofLeaf<KeyIdOf<T>, <T as frame_system::Config>::BlockNumber>>;

#[derive(Encode, Decode, RuntimeDebug, PartialEq, Eq, TypeInfo)]
pub struct CompleteMerkleProof<Root, Proof> {
	pub root: Root,
	pub proof: Proof,
}

pub struct DidMerkleRootGenerator<T>(PhantomData<T>);

impl<T> DidMerkleRootGenerator<T>
where
	T: did::Config,
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
	fn calculate_root_with_db(identity: &DidDetails<T>, db: &mut MemoryDB<T::Hashing>) -> Result<T::Hash, ()> {
		let mut trie = TrieHash::<LayoutV1<T::Hashing>>::default();
		let mut trie_builder = TrieDBMutBuilder::<LayoutV1<T::Hashing>>::new(db, &mut trie).build();

		// Authentication key
		let auth_leaf = ProofLeaf::<_, T::BlockNumber>::KeyReference(
			KeyReferenceKey(
				identity.authentication_key,
				DidVerificationKeyRelationship::Authentication.into(),
			),
			KeyReferenceValue,
		);
		trie_builder
			.insert(auth_leaf.encoded_key().as_slice(), auth_leaf.encoded_value().as_slice())
			.map_err(|_| ())?;
		// Attestation key, if present
		if let Some(att_key_id) = identity.attestation_key {
			let att_leaf = ProofLeaf::<_, T::BlockNumber>::KeyReference(
				KeyReferenceKey(att_key_id, DidVerificationKeyRelationship::AssertionMethod.into()),
				KeyReferenceValue,
			);
			trie_builder
				.insert(att_leaf.encoded_key().as_slice(), att_leaf.encoded_value().as_slice())
				.map_err(|_| ())?;
		};
		// Delegation key, if present
		if let Some(del_key_id) = identity.delegation_key {
			let del_leaf = ProofLeaf::<_, T::BlockNumber>::KeyReference(
				KeyReferenceKey(del_key_id, DidVerificationKeyRelationship::CapabilityDelegation.into()),
				KeyReferenceValue,
			);
			trie_builder
				.insert(del_leaf.encoded_key().as_slice(), del_leaf.encoded_value().as_slice())
				.map_err(|_| ())?;
		};
		// Key agreement keys
		identity
			.key_agreement_keys
			.iter()
			.try_for_each(|id| -> Result<(), ()> {
				let enc_leaf = ProofLeaf::<_, T::BlockNumber>::KeyReference(
					KeyReferenceKey(*id, KeyRelationship::Encryption),
					KeyReferenceValue,
				);
				trie_builder
					.insert(enc_leaf.encoded_key().as_slice(), enc_leaf.encoded_value().as_slice())
					.map_err(|_| ())?;
				Ok(())
			})?;
		// Public keys
		identity
			.public_keys
			.iter()
			.try_for_each(|(id, key_details)| -> Result<(), ()> {
				let key_leaf = ProofLeaf::KeyDetails(KeyDetailsKey(*id), KeyDetailsValue(key_details.clone()));
				trie_builder
					.insert(key_leaf.encoded_key().as_slice(), key_leaf.encoded_value().as_slice())
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
		identity: &DidDetails<T>,
		mut key_ids: K,
	) -> Result<CompleteMerkleProof<T::Hash, DidMerkleProof<T>>, ()>
	where
		K: Iterator<Item = &'a KeyIdOf<T>>,
	{
		let mut db = MemoryDB::default();
		let root = Self::calculate_root_with_db(identity, &mut db)?;

		#[allow(clippy::type_complexity)]
		let leaves: BTreeSet<ProofLeaf<KeyIdOf<T>, T::BlockNumber>> =
			key_ids.try_fold(BTreeSet::new(), |mut set, key_id| -> Result<_, ()> {
				let key_details = identity.public_keys.get(key_id).ok_or(())?;
				// Adds a key reference leaf for each relationship the key ID is part of.
				if *key_id == identity.authentication_key {
					set.insert(ProofLeaf::KeyReference(
						KeyReferenceKey(*key_id, DidVerificationKeyRelationship::Authentication.into()),
						KeyReferenceValue,
					));
				}
				if Some(*key_id) == identity.attestation_key {
					set.insert(ProofLeaf::KeyReference(
						KeyReferenceKey(*key_id, DidVerificationKeyRelationship::AssertionMethod.into()),
						KeyReferenceValue,
					));
				}
				if Some(*key_id) == identity.delegation_key {
					set.insert(ProofLeaf::KeyReference(
						KeyReferenceKey(*key_id, DidVerificationKeyRelationship::CapabilityDelegation.into()),
						KeyReferenceValue,
					));
				}
				if identity.key_agreement_keys.contains(key_id) {
					set.insert(ProofLeaf::KeyReference(
						KeyReferenceKey(*key_id, KeyRelationship::Encryption),
						KeyReferenceValue,
					));
				};
				// Then adds the actual key details to the merkle proof.
				// If the same key is specified twice, the old key is simply replaced with a new
				// key of the same value.
				let key_details_leaf =
					ProofLeaf::KeyDetails(KeyDetailsKey(*key_id), KeyDetailsValue(key_details.clone()));
				if !set.contains(&key_details_leaf) {
					set.insert(key_details_leaf);
				}
				Ok(set)
			})?;
		let encoded_keys: Vec<Vec<u8>> = leaves.iter().map(|l| l.encoded_key()).collect();
		let proof = generate_trie_proof::<LayoutV1<T::Hashing>, _, _, _>(&db, root, &encoded_keys).map_err(|_| ())?;
		Ok(CompleteMerkleProof {
			root,
			proof: DidMerkleProof::<T> {
				blinded: proof,
				revealed: leaves.into_iter().collect::<Vec<_>>(),
			},
		})
	}
}

impl<T> IdentityProofGenerator<T::DidIdentifier, DidDetails<T>> for DidMerkleRootGenerator<T>
where
	T: did::Config,
{
	// TODO: Proper error handling
	type Error = ();
	type Output = T::Hash;

	fn generate_commitment(_identifier: &T::DidIdentifier, identity: &DidDetails<T>) -> Result<T::Hash, Self::Error> {
		let mut db = MemoryDB::default();
		Self::calculate_root_with_db(identity, &mut db)
	}
}

pub struct DidIdentityProvider<T>(PhantomData<T>);

impl<T> IdentityProvider<T::DidIdentifier, DidDetails<T>, ()> for DidIdentityProvider<T>
where
	T: did::Config,
{
	// TODO: Proper error handling
	type Error = ();

	fn retrieve(identifier: &T::DidIdentifier) -> Result<Option<(DidDetails<T>, ())>, Self::Error> {
		match (
			did::Pallet::<T>::get_did(identifier),
			did::Pallet::<T>::get_deleted_did(identifier),
		) {
			(Some(details), _) => Ok(Some((details, ()))),
			(_, Some(_)) => Ok(None),
			_ => Err(()),
		}
	}
}
