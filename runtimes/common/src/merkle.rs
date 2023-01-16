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

use codec::Encode;
use frame_support::weights::Weight;
use sp_std::{borrow::ToOwned, vec::Vec};
use sp_trie::{generate_trie_proof, verify_trie_proof, LayoutV1, MemoryDB, TrieDBMutBuilder, TrieHash, TrieMut};

use did::{
	did_details::{DidDetails, DidPublicKeyDetails},
	traits::DidDocumentHasher,
	DidIdentifierOf, KeyIdOf,
};

pub struct DidMerkleRootHasher<DidIdentifier, DidDetails, Output>(
	sp_std::marker::PhantomData<(DidIdentifier, DidDetails, Output)>,
);

impl<T: did::Config> DidDocumentHasher<DidIdentifierOf<T>, DidDetails<T>, <T as frame_system::Config>::Hash>
	for DidMerkleRootHasher<DidIdentifierOf<T>, DidDetails<T>, <T as frame_system::Config>::Hash>
{
	// TODO: Change to a reasonable value based on benchmarks.
	const MAX_WEIGHT: frame_support::weights::Weight = Weight::zero();

	fn calculate_root(
		did: &DidIdentifierOf<T>,
		details: &DidDetails<T>,
	) -> Result<(<T as frame_system::Config>::Hash, Weight), sp_runtime::DispatchError> {
		let mut db = MemoryDB::default();
		Self::calculate_root_with_db(did, details, &mut db)
	}
}

impl<T: did::Config> DidMerkleRootHasher<DidIdentifierOf<T>, DidDetails<T>, <T as frame_system::Config>::Hash> {
	fn calculate_root_with_db(
		did: &DidIdentifierOf<T>,
		details: &DidDetails<T>,
		db: &mut MemoryDB<<T as frame_system::Config>::Hashing>,
	) -> Result<(<T as frame_system::Config>::Hash, Weight), sp_runtime::DispatchError> {
		let mut trie = TrieHash::<LayoutV1<T::Hashing>>::default();
		let mut trie_builder = TrieDBMutBuilder::<LayoutV1<T::Hashing>>::new(db, &mut trie).build();

		// Should never happen
		trie_builder
			.insert(b"did", &did.encode())
			.map_err(|_| did::Error::<T>::InternalError)?;
		// Should never happen
		trie_builder
			.insert(b"auth", &details.authentication_key.encode())
			.map_err(|_| did::Error::<T>::InternalError)?;
		if let Some(att_key) = details.attestation_key {
			trie_builder
				.insert(b"att", &att_key.encode())
				.map_err(|_| did::Error::<T>::InternalError)?;
		}
		if let Some(del_key) = details.delegation_key {
			trie_builder
				.insert(b"del", &del_key.encode())
				.map_err(|_| did::Error::<T>::InternalError)?;
		}
		details
			.key_agreement_keys
			.iter()
			.enumerate()
			.try_for_each(|(i, k)| -> Result<(), did::Error<T>> {
				trie_builder
					.insert(&[b"enc-".as_slice(), i.to_be_bytes().as_slice()].concat(), &k.encode())
					.map_err(|_| did::Error::<T>::InternalError)?;
				Ok(())
			})?;
		details
			.public_keys
			.iter()
			.enumerate()
			.try_for_each(|(i, (id, k))| -> Result<(), did::Error<T>> {
				trie_builder
					.insert(
						&[b"pub-".as_slice(), i.to_be_bytes().as_slice()].concat(),
						&(id, k).encode(),
					)
					.map_err(|_| did::Error::<T>::InternalError)?;
				Ok(())
			})?;
		trie_builder.commit();
		// TODO: Benchmark weight
		Ok((trie_builder.root().to_owned(), Weight::zero()))
	}

	pub fn generate_proof(
		did_identifier: &DidIdentifierOf<T>,
		details: &DidDetails<T>,
		key_id: KeyIdOf<T>,
	) -> Result<(<T as frame_system::Config>::Hash, Vec<Vec<u8>>), sp_runtime::DispatchError> {
		let mut db = MemoryDB::default();
		let (merkle_root, _) = Self::calculate_root_with_db(did_identifier, details, &mut db)?;
		let proof_key: Vec<u8> = if key_id == details.authentication_key {
			Ok(b"auth".to_vec())
		} else if details.attestation_key == Some(key_id) {
			Ok(b"att".to_vec())
		} else if details.delegation_key == Some(key_id) {
			Ok(b"del".to_vec())
		} else if let Some((i, _)) = details
			.key_agreement_keys
			.iter()
			.enumerate()
			.find(|(_, enc_key_id)| **enc_key_id == key_id)
		{
			Ok([b"enc-".as_slice(), i.to_be_bytes().as_slice()].concat())
		} else if let Some((i, _)) = details
			.public_keys
			.iter()
			.enumerate()
			.find(|(_, (public_key_id, _))| **public_key_id == key_id)
		{
			Ok([b"pub-".as_slice(), i.to_be_bytes().as_slice()].concat())
		} else {
			Err(did::Error::<T>::VerificationKeyNotPresent)
		}?;
		let merkle_proof =
			generate_trie_proof::<LayoutV1<T::Hashing>, _, _, _>(&db, merkle_root, &[proof_key]).unwrap();
		Ok((merkle_root, merkle_proof))
	}

	pub fn verify_proof(
		merkle_root: <T as frame_system::Config>::Hash,
		proof: Vec<Vec<u8>>,
		merkle_key: &[u8],
		did_key: (KeyIdOf<T>, DidPublicKeyDetails<T::BlockNumber>),
	) -> bool {
		verify_trie_proof::<LayoutV1<T::Hashing>, _, _, _>(
			&merkle_root,
			&proof,
			&[(&merkle_key, Some(did_key.encode()))],
		)
		.is_ok()
	}
}

// TODO: This will have to be moved into a separate crate for other projects to
// import it.
pub mod xcm {
	pub use v1 as latest;

	pub mod v1 {
		use codec::{Decode, Encode};
		use did::traits::DidRootStateAction;

		use crate::{DidIdentifier, Hash};

		#[derive(Encode, Decode)]
		pub struct MerkleRootXcmMessage(DidRootStateAction<DidIdentifier, Hash>);
	}
}
