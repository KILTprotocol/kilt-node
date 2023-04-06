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

use did::did_details::DidPublicKeyDetails;
use dip_support::{v1, VersionedIdentityProof};
use frame_support::RuntimeDebug;
use pallet_dip_receiver::traits::IdentityProofVerifier;
use parity_scale_codec::Encode;
use sp_std::{collections::btree_map::BTreeMap, marker::PhantomData, vec::Vec};
use sp_trie::{verify_trie_proof, LayoutV1};

use crate::dip::{sender, KeyDetailsKey, KeyDetailsValue, KeyReferenceKey, KeyRelationship, ProofLeaf};

// TODO: Avoid repetition of the same key if it appears multiple times, e.g., by
// having a vector of `KeyRelationship` instead.
#[derive(RuntimeDebug, PartialEq, Eq)]
pub struct ProofEntry<BlockNumber> {
	pub key: DidPublicKeyDetails<BlockNumber>,
	pub relationship: KeyRelationship,
}

// Contains the list of revealed public keys after a given merkle proof has been
// correctly verified.
#[derive(RuntimeDebug, PartialEq, Eq)]
pub struct VerificationResult<BlockNumber>(pub Vec<ProofEntry<BlockNumber>>);

impl<BlockNumber> From<Vec<ProofEntry<BlockNumber>>> for VerificationResult<BlockNumber> {
	fn from(value: Vec<ProofEntry<BlockNumber>>) -> Self {
		Self(value)
	}
}

pub struct DidMerkleProofVerifier<KeyId, BlockNumber, Hasher>(PhantomData<(KeyId, BlockNumber, Hasher)>);

impl<KeyId, BlockNumber, Hasher> IdentityProofVerifier for DidMerkleProofVerifier<KeyId, BlockNumber, Hasher>
where
	KeyId: Encode + Clone + Ord,
	BlockNumber: Encode + Clone + Ord,
	Hasher: sp_core::Hasher,
{
	type BlindedValue = Vec<sender::BlindedValue>;
	// TODO: Proper error handling
	type Error = ();
	type ProofDigest = <Hasher as sp_core::Hasher>::Out;
	type ProofLeaf = ProofLeaf<KeyId, BlockNumber>;
	type VerificationResult = VerificationResult<BlockNumber>;

	fn verify_proof_against_digest(
		proof: VersionedIdentityProof<Self::BlindedValue, Self::ProofLeaf>,
		digest: Self::ProofDigest,
	) -> Result<Self::VerificationResult, Self::Error> {
		let proof: v1::Proof<_, _> = proof.try_into()?;
		// TODO: more efficient by removing cloning and/or collecting.
		// Did not find another way of mapping a Vec<(Vec<u8>, Vec<u8>)> to a
		// Vec<(Vec<u8>, Option<Vec<u8>>)>.
		let proof_leaves = proof
			.revealed
			.iter()
			.map(|leaf| (leaf.encoded_key(), Some(leaf.encoded_value())))
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
			.filter_map(|leaf| {
				if let ProofLeaf::KeyDetails(KeyDetailsKey(key_id), KeyDetailsValue(key_details)) = leaf {
					Some((key_id, key_details))
				} else {
					None
				}
			})
			.collect();
		// Create a list of the revealed keys by consuming the provided key reference
		// leaves, and looking up the full details from the just-built `public_keys`
		// map.
		let keys: Vec<ProofEntry<BlockNumber>> = proof
			.revealed
			.into_iter()
			.filter_map(|leaf| {
				if let ProofLeaf::KeyReference(KeyReferenceKey(key_id, key_relationship), _) = leaf {
					// TODO: Better error handling.
					let key_details = public_keys
						.get(&key_id)
						.expect("Key ID should be present in the map of revealed public keys.");
					Some(ProofEntry {
						key: key_details.clone(),
						relationship: key_relationship,
					})
				} else {
					None
				}
			})
			.collect();
		Ok(keys.into())
	}
}
