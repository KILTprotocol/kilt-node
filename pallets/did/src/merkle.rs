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

use sp_trie::{generate_trie_proof, LayoutV1, MemoryDB, TrieDBMutBuilder, TrieHash, TrieMut};

use crate::{did_details::DidDetails, Config, Did, DidIdentifierOf, Error};

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
		.try_for_each(|(_, k)| -> Result<(), Error<T>> {
			trie_builder
				.insert(b"enc-{i}", &k.encode())
				.map_err(|_| Error::<T>::InternalError)?;
			Ok(())
		})?;
	did_details
		.public_keys
		.iter()
		.enumerate()
		.try_for_each(|(_, (id, k))| -> Result<(), Error<T>> {
			trie_builder
				.insert(b"pub-{i}", &(id, k).encode())
				.map_err(|_| Error::<T>::InternalError)?;
			Ok(())
		})?;
	trie_builder.commit();
	Ok(trie_builder.root().to_owned())
}

// Could be made a runtime API and called by users, or implemented directly in
// the clients.
pub(crate) fn generate_merkle_proof<T: Config>(
	did_identifier: &DidIdentifierOf<T>,
) -> Result<(TrieHash<LayoutV1<T::Hashing>>, Vec<Vec<u8>>), Error<T>> {
	let did_details = Did::<T>::get(did_identifier).ok_or(Error::<T>::DidNotPresent)?;
	let mut db = MemoryDB::default();
	let merkle_root = generate_did_merkle_root(did_identifier, &did_details, Some(&mut db))?;
	let merkle_proof = generate_trie_proof::<LayoutV1<T::Hashing>, _, _, _>(&db, merkle_root, &[b"auth"]).unwrap();
	Ok((merkle_root, merkle_proof))
}

// sp_trie::verify_trie_proof::<LayoutV1<Blake2Hasher>, _, _, _>(
// 			&root,
// 			&merkle_proof,
// 			&[(
// 				b"auth",
// 				Some(sp_core::ed25519::Public::did_details.authentication_key.encode()),
// 			)],
