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

	use codec::Encode;
	use did::did_details::{DidDetails, DidPublicKeyDetails};
	use pallet_dip_sender::traits::{IdentityProofGenerator, IdentityProvider};

	pub(crate) const AUTH_KEY_LEAF: &[u8] = b"auth";
	pub(crate) const ATT_KEY_LEAF: &[u8] = b"att";
	pub(crate) const DEL_KEY_LEAF: &[u8] = b"del";
	pub(crate) const ENC_KEY_PREFIX: &[u8] = b"enc-";
	pub(crate) const PUB_KEY_PREFIX: &[u8] = b"pub-";

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
			trie_builder
				.insert(AUTH_KEY_LEAF, identity.authentication_key.encode().as_slice())
				.map_err(|_| ())?;
			// Attestation key
			if let Some(ref att_key) = identity.attestation_key {
				trie_builder
					.insert(ATT_KEY_LEAF, att_key.encode().as_slice())
					.map_err(|_| ())?;
			};
			// Delegation key
			if let Some(ref del_key) = identity.delegation_key {
				trie_builder
					.insert(DEL_KEY_LEAF, del_key.encode().as_slice())
					.map_err(|_| ())?;
			};
			// Key agreement keys
			identity
				.key_agreement_keys
				.iter()
				.enumerate()
				.try_for_each(|(i, id)| -> Result<(), ()> {
					// Key leaf = "enc-<index>"
					let key_leaf = [ENC_KEY_PREFIX, i.to_be_bytes().as_slice()].concat();
					trie_builder.insert(&key_leaf, id.encode().as_slice()).map_err(|_| ())?;
					Ok(())
				})?;
			// Public keys
			identity.public_keys.iter().enumerate().try_for_each(
				|(i, (id, DidPublicKeyDetails { key: public_key, .. }))| -> Result<(), ()> {
					// Key leaf = "pub-<index>"
					let key_leaf = [PUB_KEY_PREFIX, i.to_be_bytes().as_slice()].concat();
					trie_builder
						.insert(&key_leaf, (id, public_key).encode().as_slice())
						.map_err(|_| ())?;
					Ok(())
				},
			)?;
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

	use did::{did_details::DidPublicKey, DidVerificationKeyRelationship};
	use dip_support::VersionedIdentityProof;
	use pallet_dip_receiver::traits::IdentityProofVerifier;
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
		type LeafValue = Option<Vec<u8>>;
		type ProofDigest = T::Hash;
		type VerificationResult = VerificationResult;

		fn verify_proof_against_digest(
			proof: VersionedIdentityProof<Self::BlindedValue, Self::LeafKey, Self::LeafValue>,
			digest: Self::ProofDigest,
		) -> Result<Self::VerificationResult, Self::Error> {
			use crate::dip::sender::{ATT_KEY_LEAF, AUTH_KEY_LEAF, DEL_KEY_LEAF, ENC_KEY_PREFIX, PUB_KEY_PREFIX};
			use dip_support::v1;
			use sp_trie::verify_trie_proof;

			let proof: v1::Proof<_, _, _> = proof.try_into()?;
			verify_trie_proof::<LayoutV1<T::Hashing>, _, _, _>(&digest, &proof.blinded, &proof.revealed)
				.map_err(|_| ())?;
			let mut revealed_leafs: Vec<ProofEntry> = vec![];

			let revealed_leafs: Vec<ProofEntry> = proof.revealed.iter().map(|(key, value)| match (key) {
				_ => ProofEntry {
					key: value.try_into(),
					verification_relationship: DidVerificationKeyRelationship::Authentication,
				},
			});
		}
	}
}
