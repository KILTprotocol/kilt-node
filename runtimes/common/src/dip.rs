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

use sp_std::marker::PhantomData;

pub mod sender {
	use super::*;

	use codec::{Decode, Encode};
	use did::{
		did_details::{DidDetails, DidPublicKeyDetails},
		DidVerificationKeyRelationship, KeyIdOf,
	};
	use pallet_dip_sender::traits::{IdentityProofGenerator, IdentityProvider};

	pub enum MerkleLeaf<T: did::Config> {
		VerificationKey(KeyIdOf<T>, DidVerificationKeyRelationship),
		EncryptionKey(KeyIdOf<T>),
		PublicKey(KeyIdOf<T>, DidPublicKeyDetails<T::BlockNumber>),
	}

	impl<T> MerkleLeaf<T>
	where
		T: did::Config,
	{
		pub fn key(&self) -> Vec<u8> {
			match self {
				MerkleLeaf::VerificationKey(_, relationship) => relationship.encode(),
				MerkleLeaf::EncryptionKey(key_id) => key_id.encode(),
				MerkleLeaf::PublicKey(key_id, _) => key_id.encode(),
			}
		}

		pub fn value(&self) -> Vec<u8> {
			match self {
				MerkleLeaf::VerificationKey(key_id, relationship) => key_id.encode(),
				MerkleLeaf::EncryptionKey(key_id) => key_id.encode(),
				MerkleLeaf::PublicKey(_, public_key) => public_key.encode(),
			}
		}
	}

	impl<T> TryFrom<(Vec<u8>, Vec<u8>)> for MerkleLeaf<T>
	where
		T: did::Config,
	{
		// TODO: Proper error handling
		type Error = ();

		fn try_from((key, value): (Vec<u8>, Vec<u8>)) -> Result<Self, Self::Error> {
			let (key, value) = (&mut key.as_slice(), &mut value.as_slice());
			if let (Ok(relationship), Ok(key_id)) =
				(DidVerificationKeyRelationship::decode(key), KeyIdOf::<T>::decode(value))
			{
				Ok(Self::VerificationKey(key_id, relationship))
			} else if let (Ok(key_id), Ok(_)) = (KeyIdOf::<T>::decode(key), KeyIdOf::<T>::decode(value)) {
				Ok(Self::EncryptionKey(key_id))
			} else if let (Ok(key_id), Ok(public_key)) = (KeyIdOf::<T>::decode(key), DidPublicKeyDetails::decode(value))
			{
				Ok(Self::PublicKey(key_id, public_key))
			} else {
				Err(())
			}
		}
	}

	pub struct DidMerkleRootGenerator<T>(PhantomData<T>);

	impl<T> IdentityProofGenerator<T::DidIdentifier, DidDetails<T>, T::Hash> for DidMerkleRootGenerator<T>
	where
		T: did::Config,
	{
		// TODO: Proper error handling
		type Error = ();

		fn generate_proof(_identifier: &T::DidIdentifier, identity: &DidDetails<T>) -> Result<T::Hash, Self::Error> {
			use sp_trie::{LayoutV1, MemoryDB, TrieDBMutBuilder, TrieHash, TrieMut};

			let mut db = MemoryDB::default();
			let mut trie = TrieHash::<LayoutV1<T::Hashing>>::default();
			let mut trie_builder = TrieDBMutBuilder::<LayoutV1<T::Hashing>>::new(&mut db, &mut trie).build();

			// Authentication key
			let auth_key_leaf = MerkleLeaf::<T>::VerificationKey(
				identity.authentication_key,
				DidVerificationKeyRelationship::Authentication,
			);
			trie_builder
				.insert(
					auth_key_leaf.key().encode().as_slice(),
					auth_key_leaf.value().encode().as_slice(),
				)
				.map_err(|_| ())?;
			// Attestation key: (key relationship, key id)
			if let Some(att_key_id) = identity.attestation_key {
				let att_key_leaf =
					MerkleLeaf::<T>::VerificationKey(att_key_id, DidVerificationKeyRelationship::AssertionMethod);
				trie_builder
					.insert(
						att_key_leaf.key().encode().as_slice(),
						att_key_leaf.value().encode().as_slice(),
					)
					.map_err(|_| ())?;
			};
			// Delegation key: (key relationship, key id)
			if let Some(del_key_id) = identity.delegation_key {
				let del_key_leaf =
					MerkleLeaf::<T>::VerificationKey(del_key_id, DidVerificationKeyRelationship::CapabilityDelegation);
				trie_builder
					.insert(
						del_key_leaf.key().encode().as_slice(),
						del_key_leaf.value().encode().as_slice(),
					)
					.map_err(|_| ())?;
			};
			// Key agreement keys [(enc-<key_id>, key id)]
			identity
				.key_agreement_keys
				.into_iter()
				.try_for_each(|id| -> Result<(), ()> {
					let enc_key_leaf = MerkleLeaf::<T>::EncryptionKey(id);
					trie_builder
						.insert(
							enc_key_leaf.key().encode().as_slice(),
							enc_key_leaf.value().encode().as_slice(),
						)
						.map_err(|_| ())?;
					Ok(())
				})?;
			// Public keys: [(key id, public key)]
			identity
				.public_keys
				.into_iter()
				.try_for_each(|(id, key_details)| -> Result<(), ()> {
					let pub_key_leaf = MerkleLeaf::<T>::PublicKey(id, key_details);
					trie_builder
						.insert(
							pub_key_leaf.key().encode().as_slice(),
							pub_key_leaf.value().encode().as_slice(),
						)
						.map_err(|_| ())?;
					Ok(())
				})?;
			trie_builder.commit();
			Ok(trie_builder.root().to_owned())
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
				(None, Some(_)) => Ok(None),
				_ => Err(()),
			}
		}
	}
}

pub mod receiver {
	use crate::dip::sender::MerkleLeaf;

	use super::*;

	use did::{did_details::DidPublicKey, DidVerificationKeyRelationship};
	use dip_support::VersionedIdentityProof;
	use pallet_dip_receiver::traits::IdentityProofVerifier;
	use sp_std::vec::Vec;
	use sp_trie::LayoutV1;

	pub struct ProofEntry {
		key: DidPublicKey,
		verification_relationship: DidVerificationKeyRelationship,
	}

	pub struct VerificationResult(Vec<ProofEntry>);

	pub struct DidMerkleProofVerifier<T>(PhantomData<T>);

	impl<T> IdentityProofVerifier for DidMerkleProofVerifier<T>
	where
		T: did::Config,
	{
		// TODO: Proper error handling
		type BlindedValue = Vec<Vec<u8>>;
		type Error = ();
		type LeafKey = Vec<u8>;
		type LeafValue = Vec<u8>;
		type ProofDigest = T::Hash;
		type VerificationResult = VerificationResult;

		fn verify_proof_against_digest(
			proof: VersionedIdentityProof<Self::BlindedValue, Self::LeafKey, Self::LeafValue>,
			digest: Self::ProofDigest,
		) -> Result<Self::VerificationResult, Self::Error> {
			use dip_support::v1;
			use sp_trie::verify_trie_proof;

			let proof: v1::Proof<_, _, _> = proof.try_into()?;
			verify_trie_proof::<LayoutV1<T::Hashing>, _, _, _>(
				&digest,
				&proof.blinded,
				proof.revealed.iter().map(|(key, value)| (key, Some(value))).collect(),
			)
			.map_err(|_| ())?;

			let revealed_leaves: Vec<MerkleLeaf<T>> = proof
				.revealed
				.iter()
				.map(|(key, value)| MerkleLeaf::try_from((*key, *value)).expect("Error."))
				.collect();
			// .map(|(key, value)| match key.as_ref() {
			// 	AUTH_KEY_LEAF => (
			// 		KeyIdOf::<T>::from(key.as_slice()),
			// 		DidVerificationKeyRelationship::Authentication,
			// 	),
			// 	ATT_KEY_LEAF => (*key,
			// DidVerificationKeyRelationship::AssertionMethod), 	DEL_KEY_LEAF =>
			// (*key, DidVerificationKeyRelationship::CapabilityDelegation), })
			// .collect();
		}
	}
}
