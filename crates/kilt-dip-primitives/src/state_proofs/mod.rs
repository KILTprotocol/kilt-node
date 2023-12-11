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

//! Module to deal with cross-chain state proofs.

/// Parachain-related state proof logic.
pub(crate) mod parachain;
/// Relaychain-related state proof logic.
pub(crate) mod relaychain;

// Ported from https://github.com/paritytech/substrate/blob/b27c470eaff379f512d1dec052aff5d551ed3b03/primitives/state-machine/src/lib.rs#L1076
// Needs to be replaced with its runtime-friendly version when available, or be
// kept up-to-date with upstream.
mod substrate_no_std_port {
	use hash_db::EMPTY_PREFIX;
	use parity_scale_codec::Codec;
	use sp_core::Hasher;
	use sp_state_machine::{Backend, TrieBackend, TrieBackendBuilder};
	use sp_std::{collections::btree_map::BTreeMap, vec::Vec};
	use sp_trie::{HashDBT, MemoryDB, StorageProof};

	pub(super) fn read_proof_check<H, I>(
		root: H::Out,
		proof: StorageProof,
		keys: I,
	) -> Result<BTreeMap<Vec<u8>, Option<Vec<u8>>>, ()>
	where
		H: Hasher,
		H::Out: Ord + Codec,
		I: IntoIterator,
		I::Item: AsRef<[u8]>,
	{
		let proving_backend = create_proof_check_backend::<H>(root, proof)?;
		let mut result = BTreeMap::new();
		for key in keys.into_iter() {
			let value = read_proof_check_on_proving_backend(&proving_backend, key.as_ref())?;
			result.insert(key.as_ref().to_vec(), value);
		}
		Ok(result)
	}

	fn read_proof_check_on_proving_backend<H>(
		proving_backend: &TrieBackend<MemoryDB<H>, H>,
		key: &[u8],
	) -> Result<Option<Vec<u8>>, ()>
	where
		H: Hasher,
		H::Out: Ord + Codec,
	{
		proving_backend.storage(key).map_err(|_| ())
	}

	fn create_proof_check_backend<H>(root: H::Out, proof: StorageProof) -> Result<TrieBackend<MemoryDB<H>, H>, ()>
	where
		H: Hasher,
		H::Out: Codec,
	{
		let db = proof.into_memory_db();

		if db.contains(&root, EMPTY_PREFIX) {
			Ok(TrieBackendBuilder::new(db, root).build())
		} else {
			Err(())
		}
	}
}
