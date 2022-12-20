// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2022 BOTLabs GmbH

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

use sp_trie::{generate_trie_proof, verify_trie_proof, LayoutV1, MemoryDB, TrieDBMutBuilder, TrieHash, TrieMut};

use crate::{
	did_details::{DidDetails, DidPublicKeyDetails},
	Config, Did, DidIdentifierOf, Error, KeyIdOf,
};

// To be called from the runtime before sending the result over with XCM.
pub(crate) fn generate_did_merkle_root<T: Config>(
	did_identifier: &DidIdentifierOf<T>,
	did_details: &DidDetails<T>,
	db: Option<&mut MemoryDB<T::Hashing>>,
) -> Result<TrieHash<LayoutV1<T::Hashing>>, Error<T>> {
	let mut default_memory_db = MemoryDB::default();
	let db = db.unwrap_or(&mut default_memory_db);
	let mut trie = TrieHash::<LayoutV1<T::Hashing>>::default();
	let mut trie_builder = TrieDBMutBuilder::<LayoutV1<T::Hashing>>::new(db, &mut trie).build();

	// Should never happen
	trie_builder
		.insert(b"did", &did_identifier.encode())
		.map_err(|_| Error::<T>::InternalError)?;
	// Should never happen
	trie_builder
		.insert(b"auth", &did_details.authentication_key.encode())
		.map_err(|_| Error::<T>::InternalError)?;
	if let Some(att_key) = did_details.attestation_key {
		trie_builder
			.insert(b"att", &att_key.encode())
			.map_err(|_| Error::<T>::InternalError)?;
	}
	if let Some(del_key) = did_details.delegation_key {
		trie_builder
			.insert(b"del", &del_key.encode())
			.map_err(|_| Error::<T>::InternalError)?;
	}
	did_details
		.key_agreement_keys
		.iter()
		.enumerate()
		.try_for_each(|(i, k)| -> Result<(), Error<T>> {
			trie_builder
				.insert(&[b"enc-".as_slice(), i.to_be_bytes().as_slice()].concat(), &k.encode())
				.map_err(|_| Error::<T>::InternalError)?;
			Ok(())
		})?;
	did_details
		.public_keys
		.iter()
		.enumerate()
		.try_for_each(|(i, (id, k))| -> Result<(), Error<T>> {
			trie_builder
				.insert(
					&[b"pub-".as_slice(), i.to_be_bytes().as_slice()].concat(),
					&(id, k).encode(),
				)
				.map_err(|_| Error::<T>::InternalError)?;
			Ok(())
		})?;
	trie_builder.commit();
	Ok(trie_builder.root().to_owned())
}

pub(crate) type MerkleRoot<Hash> = TrieHash<LayoutV1<Hash>>;
pub(crate) type MerkleProof = Vec<Vec<u8>>;
pub(crate) type MerkleRootAndProof<Hash> = (MerkleRoot<Hash>, MerkleProof);

// TODO: Allow to specify multiple keys (e.g., proving that a key is an
// authentication key requires to prove the value of the authentication field
// and the value of the key itself from the public keys) Could be made a runtime

// API and called by users, or implemented directly in the clients.
pub(crate) fn generate_merkle_proof<T: Config>(
	did_identifier: &DidIdentifierOf<T>,
	key_id: KeyIdOf<T>,
) -> Result<MerkleRootAndProof<T::Hashing>, Error<T>> {
	let did_details = Did::<T>::get(did_identifier).ok_or(Error::<T>::DidNotPresent)?;
	let mut db = MemoryDB::default();
	let merkle_root = generate_did_merkle_root(did_identifier, &did_details, Some(&mut db))?;
	let proof_key: Vec<u8> = if key_id == did_details.authentication_key {
		Ok(b"auth".to_vec())
	} else if did_details.attestation_key == Some(key_id) {
		Ok(b"att".to_vec())
	} else if did_details.delegation_key == Some(key_id) {
		Ok(b"del".to_vec())
	} else if let Some((i, _)) = did_details
		.key_agreement_keys
		.into_iter()
		.enumerate()
		.find(|(_, enc_key_id)| *enc_key_id == key_id)
	{
		Ok([b"enc-".as_slice(), i.to_be_bytes().as_slice()].concat())
	} else if let Some((i, _)) = did_details
		.public_keys
		.into_iter()
		.enumerate()
		.find(|(_, (public_key_id, _))| *public_key_id == key_id)
	{
		Ok([b"pub-".as_slice(), i.to_be_bytes().as_slice()].concat())
	} else {
		Err(Error::<T>::VerificationKeyNotPresent)
	}?;
	let merkle_proof = generate_trie_proof::<LayoutV1<T::Hashing>, _, _, _>(&db, merkle_root, &[proof_key]).unwrap();
	Ok((merkle_root, merkle_proof))
}

pub(crate) fn verify_merkle_proof<T: Config>(
	merkle_root: MerkleRoot<T::Hashing>,
	merkle_proof: MerkleProof,
	merkle_key: &[u8],
	did_key: (KeyIdOf<T>, DidPublicKeyDetails<T::BlockNumber>),
) -> bool {
	verify_trie_proof::<LayoutV1<T::Hashing>, _, _, _>(
		&merkle_root,
		&merkle_proof,
		&[(&merkle_key, Some(did_key.encode()))],
	)
	.is_ok()
}
