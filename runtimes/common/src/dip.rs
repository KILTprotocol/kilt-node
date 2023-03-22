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

use codec::{Decode, Encode, MaxEncodedLen};
use did::{did_details::DidPublicKeyDetails, DidVerificationKeyRelationship};
use scale_info::TypeInfo;
use sp_std::{marker::PhantomData, vec::Vec};

#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo)]
pub enum KeyRelationship {
	Encryption,
	Verification(DidVerificationKeyRelationship),
}

impl From<DidVerificationKeyRelationship> for KeyRelationship {
	fn from(value: DidVerificationKeyRelationship) -> Self {
		Self::Verification(value)
	}
}

#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo)]
pub enum LeafKey<KeyId> {
	KeyId(KeyId),
	KeyRelationship(KeyRelationship),
}

#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo)]
pub enum LeafValue<KeyId, BlockNumber: MaxEncodedLen> {
	KeyId(KeyId),
	KeyDetails(DidPublicKeyDetails<BlockNumber>),
}

struct MerkleLeaf<KeyId, BlockNumber: MaxEncodedLen> {
	pub key: LeafKey<KeyId>,
	pub value: LeafValue<KeyId, BlockNumber>,
}

// TODO: Generalize Vec<u8> input to the minimum capabilities needed for this,
// so as to avoid unnecessary allocations and/or copies.
impl<KeyId, BlockNumber> TryFrom<(Vec<u8>, Vec<u8>)> for MerkleLeaf<KeyId, BlockNumber>
where
	KeyId: Decode,
	BlockNumber: Decode + MaxEncodedLen,
{
	// TODO: Proper error handling
	type Error = ();

	fn try_from((key, value): (Vec<u8>, Vec<u8>)) -> Result<Self, Self::Error> {
		let (key, value) = (&mut key.as_slice(), &mut value.as_slice());
		let decoded_key = LeafKey::decode(key).map_err(|_| ())?;
		let decoded_value = LeafValue::decode(value).map_err(|_| ())?;
		match (&decoded_key, &decoded_value) {
			(LeafKey::KeyId(_), LeafValue::KeyDetails(_)) => Ok(Self {
				key: decoded_key,
				value: decoded_value,
			}),
			(LeafKey::KeyRelationship(_), LeafValue::KeyId(_)) => Ok(Self {
				key: decoded_key,
				value: decoded_value,
			}),
			_ => Err(()),
		}
	}
}

pub mod sender {
	use super::*;

	use did::did_details::DidDetails;
	use pallet_dip_sender::traits::{IdentityProofGenerator, IdentityProvider};
	use sp_std::borrow::ToOwned;

	pub struct DidMerkleRootGenerator<T>(PhantomData<T>);

	impl<T> IdentityProofGenerator<T::DidIdentifier, DidDetails<T>> for DidMerkleRootGenerator<T>
	where
		T: did::Config,
	{
		// TODO: Proper error handling
		type Error = ();
		type Output = T::Hash;

		fn generate_commitment(
			_identifier: &T::DidIdentifier,
			identity: &DidDetails<T>,
		) -> Result<T::Hash, Self::Error> {
			use sp_trie::{LayoutV1, MemoryDB, TrieDBMutBuilder, TrieHash, TrieMut};

			let mut db = MemoryDB::default();
			let mut trie = TrieHash::<LayoutV1<T::Hashing>>::default();
			let mut trie_builder = TrieDBMutBuilder::<LayoutV1<T::Hashing>>::new(&mut db, &mut trie).build();

			// Authentication key
			let auth_key_leaf = MerkleLeaf {
				key: LeafKey::KeyRelationship(DidVerificationKeyRelationship::Authentication.into()),
				value: LeafValue::<_, T::BlockNumber>::KeyId(identity.authentication_key),
			};
			trie_builder
				.insert(
					auth_key_leaf.key.encode().as_slice(),
					auth_key_leaf.value.encode().as_slice(),
				)
				.map_err(|_| ())?;
			// Attestation key: (key relationship, key id)
			if let Some(att_key_id) = identity.attestation_key {
				let att_key_leaf = MerkleLeaf {
					key: LeafKey::KeyRelationship(DidVerificationKeyRelationship::AssertionMethod.into()),
					value: LeafValue::<_, T::BlockNumber>::KeyId(att_key_id),
				};
				trie_builder
					.insert(
						att_key_leaf.key.encode().as_slice(),
						att_key_leaf.value.encode().as_slice(),
					)
					.map_err(|_| ())?;
			};
			// Delegation key: (key relationship, key id)
			if let Some(del_key_id) = identity.delegation_key {
				let del_key_leaf = MerkleLeaf {
					key: LeafKey::KeyRelationship(DidVerificationKeyRelationship::CapabilityDelegation.into()),
					value: LeafValue::<_, T::BlockNumber>::KeyId(del_key_id),
				};
				trie_builder
					.insert(
						del_key_leaf.key.encode().as_slice(),
						del_key_leaf.value.encode().as_slice(),
					)
					.map_err(|_| ())?;
			};
			// Key agreement keys [(enc-<key_id>, key id)]
			identity
				.key_agreement_keys
				.iter()
				.try_for_each(|id| -> Result<(), ()> {
					let enc_key_leaf = MerkleLeaf {
						key: LeafKey::KeyRelationship(KeyRelationship::Encryption),
						value: LeafValue::<_, T::BlockNumber>::KeyId(id),
					};
					trie_builder
						.insert(
							enc_key_leaf.key.encode().as_slice(),
							enc_key_leaf.value.encode().as_slice(),
						)
						.map_err(|_| ())?;
					Ok(())
				})?;
			// Public keys: [(key id, public key)]
			identity
				.public_keys
				.iter()
				.try_for_each(|(id, key_details)| -> Result<(), ()> {
					let pub_key_leaf = MerkleLeaf {
						key: LeafKey::KeyId(id),
						value: LeafValue::KeyDetails(key_details.clone()),
					};
					trie_builder
						.insert(
							pub_key_leaf.key.encode().as_slice(),
							pub_key_leaf.value.encode().as_slice(),
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
	use super::*;

	use dip_support::VersionedIdentityProof;
	use pallet_dip_receiver::traits::IdentityProofVerifier;
	use sp_std::{collections::btree_map::BTreeMap, vec::Vec};
	use sp_trie::LayoutV1;

	pub struct ProofEntry<BlockNumber: MaxEncodedLen> {
		pub key: DidPublicKeyDetails<BlockNumber>,
		pub relationship: KeyRelationship,
	}

	pub struct VerificationResult<BlockNumber: MaxEncodedLen>(pub Vec<ProofEntry<BlockNumber>>);

	impl<BlockNumber> From<Vec<ProofEntry<BlockNumber>>> for VerificationResult<BlockNumber>
	where
		BlockNumber: MaxEncodedLen,
	{
		fn from(value: Vec<ProofEntry<BlockNumber>>) -> Self {
			Self(value)
		}
	}

	pub struct DidMerkleProofVerifier<KeyId, BlockNumber, Hasher>(PhantomData<(KeyId, BlockNumber, Hasher)>);

	impl<KeyId, BlockNumber, Hasher> IdentityProofVerifier for DidMerkleProofVerifier<KeyId, BlockNumber, Hasher>
	where
		KeyId: MaxEncodedLen + Clone + Ord,
		BlockNumber: MaxEncodedLen + Clone + Ord,
		Hasher: sp_core::Hasher,
	{
		type BlindedValue = Vec<Vec<u8>>;
		// TODO: Proper error handling
		type Error = ();
		type LeafKey = LeafKey<KeyId>;
		type LeafValue = LeafValue<KeyId, BlockNumber>;
		type ProofDigest = <Hasher as sp_core::Hasher>::Out;
		type VerificationResult = VerificationResult<BlockNumber>;

		fn verify_proof_against_digest(
			proof: VersionedIdentityProof<Self::BlindedValue, Self::LeafKey, Self::LeafValue>,
			digest: Self::ProofDigest,
		) -> Result<Self::VerificationResult, Self::Error> {
			use dip_support::v1;
			use sp_trie::verify_trie_proof;

			let proof: v1::Proof<_, _, _> = proof.try_into()?;
			// TODO: more efficient by removing cloning and/or collecting. Did not find
			// another way of mapping a Vec<(Vec<u8>, Vec<u8>)> to a Vec<(Vec<u8>,
			// Option<Vec<u8>>)>.
			let proof_leaves = proof
				.revealed
				.iter()
				.map(|(key, value)| (key.encode(), Some(value.encode())))
				.collect::<Vec<(Vec<u8>, Option<Vec<u8>>)>>();
			verify_trie_proof::<LayoutV1<Hasher>, _, _, _>(&digest, &proof.blinded, &proof_leaves).map_err(|_| ())?;

			// At this point, we know the proof is valid. We just need to map the revealed
			// leaves to something the consumer can easily operate on.

			// Create a map of the revealed public keys
			//TODO: Avoid cloning, and use a map of references for the lookup
			let public_keys: BTreeMap<KeyId, DidPublicKeyDetails<BlockNumber>> = proof
				.revealed
				.clone()
				.into_iter()
				.filter_map(|(key, value)| {
					if let (LeafKey::KeyId(key_id), LeafValue::KeyDetails(key_details)) = (key, value) {
						Some((key_id, key_details))
					} else {
						None
					}
				})
				.collect();
			// Create a list of the revealed verification keys
			let verification_keys: Vec<ProofEntry<BlockNumber>> = proof
				.revealed
				.into_iter()
				.filter_map(|(key, value)| {
					if let (LeafKey::KeyRelationship(key_rel), LeafValue::KeyId(key_id)) = (key, value) {
						// TODO: Better error handling.
						let key_details = public_keys
							.get(&key_id)
							.expect("Key ID should be present in the map of revealed public keys.");
						Some(ProofEntry {
							key: key_details.clone(),
							relationship: key_rel,
						})
					} else {
						None
					}
				})
				.collect();
			Ok(verification_keys.into())
		}
	}
}
