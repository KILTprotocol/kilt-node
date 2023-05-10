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

use did::{
	did_details::{DidEncryptionKey, DidPublicKey, DidPublicKeyDetails},
	DidVerificationKeyRelationship,
};
use frame_support::{traits::ConstU32, RuntimeDebug};
use pallet_dip_consumer::{identity::IdentityDetails, traits::IdentityProofVerifier};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::BoundedVec;
use sp_std::{collections::btree_map::BTreeMap, fmt::Debug, marker::PhantomData, vec::Vec};
use sp_trie::{verify_trie_proof, LayoutV1};

pub type BlindedValue = Vec<u8>;

#[derive(Encode, Decode, RuntimeDebug, Clone, Eq, PartialEq, TypeInfo, Default)]
pub struct MerkleProof<BlindedValue, Leaf> {
	pub blinded: BlindedValue,
	// TODO: Probably replace with a different data structure for better lookup capabilities
	pub revealed: Vec<Leaf>,
}

#[derive(Clone, Copy, RuntimeDebug, Encode, Decode, PartialEq, Eq, TypeInfo, PartialOrd, Ord, MaxEncodedLen)]
pub enum DidKeyRelationship {
	Encryption,
	Verification(DidVerificationKeyRelationship),
}

impl From<DidVerificationKeyRelationship> for DidKeyRelationship {
	fn from(value: DidVerificationKeyRelationship) -> Self {
		Self::Verification(value)
	}
}

impl TryFrom<DidKeyRelationship> for DidVerificationKeyRelationship {
	// TODO: Error handling
	type Error = ();

	fn try_from(value: DidKeyRelationship) -> Result<Self, Self::Error> {
		if let DidKeyRelationship::Verification(rel) = value {
			Ok(rel)
		} else {
			Err(())
		}
	}
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, PartialOrd, Ord, RuntimeDebug, TypeInfo)]
pub struct KeyReferenceKey<KeyId>(pub KeyId, pub DidKeyRelationship);
#[derive(Clone, Encode, Decode, PartialEq, Eq, PartialOrd, Ord, RuntimeDebug, TypeInfo)]
pub struct KeyReferenceValue;
#[derive(Clone, Encode, Decode, PartialEq, Eq, PartialOrd, Ord, RuntimeDebug, TypeInfo)]
pub struct KeyDetailsKey<KeyId>(pub KeyId);
#[derive(Clone, Encode, Decode, PartialEq, Eq, PartialOrd, Ord, RuntimeDebug, TypeInfo)]
pub struct KeyDetailsValue<BlockNumber>(pub DidPublicKeyDetails<BlockNumber>);

#[derive(Clone, Encode, Decode, PartialEq, Eq, PartialOrd, Ord, RuntimeDebug, TypeInfo)]
pub enum ProofLeaf<KeyId, BlockNumber> {
	// The key and value for the leaves of a merkle proof that contain a reference
	// (by ID) to the key details, provided in a separate leaf.
	KeyReference(KeyReferenceKey<KeyId>, KeyReferenceValue),
	// The key and value for the leaves of a merkle proof that contain the actual
	// details of a DID public key. The key is the ID of the key, and the value is its details, including creation
	// block number.
	KeyDetails(KeyDetailsKey<KeyId>, KeyDetailsValue<BlockNumber>),
}

impl<KeyId, BlockNumber> ProofLeaf<KeyId, BlockNumber>
where
	KeyId: Encode,
	BlockNumber: Encode,
{
	pub fn encoded_key(&self) -> Vec<u8> {
		match self {
			ProofLeaf::KeyReference(key, _) => key.encode(),
			ProofLeaf::KeyDetails(key, _) => key.encode(),
		}
	}

	pub fn encoded_value(&self) -> Vec<u8> {
		match self {
			ProofLeaf::KeyReference(_, value) => value.encode(),
			ProofLeaf::KeyDetails(_, value) => value.encode(),
		}
	}
}

// TODO: Avoid repetition of the same key if it appears multiple times, e.g., by
// having a vector of `DidKeyRelationship` instead.
#[derive(Clone, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen, Encode, Decode)]
pub struct ProofEntry<BlockNumber> {
	pub key: DidPublicKeyDetails<BlockNumber>,
	pub relationship: DidKeyRelationship,
}

#[cfg(feature = "runtime-benchmarks")]
impl<BlockNumber> Default for ProofEntry<BlockNumber>
where
	BlockNumber: Default,
{
	fn default() -> Self {
		Self {
			key: DidPublicKeyDetails {
				key: DidPublicKey::PublicEncryptionKey(DidEncryptionKey::X25519([0u8; 32])),
				block_number: BlockNumber::default(),
			},
			relationship: DidVerificationKeyRelationship::Authentication.into(),
		}
	}
}

// Contains the list of revealed public keys after a given merkle proof has been
// correctly verified.
#[derive(Clone, Debug, PartialEq, Eq, TypeInfo, MaxEncodedLen, Encode, Decode, Default)]
pub struct VerificationResult<BlockNumber, const MAX_REVEALED_LEAVES_COUNT: u32>(
	pub BoundedVec<ProofEntry<BlockNumber>, ConstU32<MAX_REVEALED_LEAVES_COUNT>>,
);

impl<BlockNumber, const MAX_REVEALED_LEAVES_COUNT: u32> TryFrom<Vec<ProofEntry<BlockNumber>>>
	for VerificationResult<BlockNumber, MAX_REVEALED_LEAVES_COUNT>
{
	// TODO: Better error handling
	type Error = ();

	fn try_from(value: Vec<ProofEntry<BlockNumber>>) -> Result<Self, Self::Error> {
		let bounded_inner = value.try_into().map_err(|_| ())?;
		Ok(Self(bounded_inner))
	}
}

impl<BlockNumber, const MAX_REVEALED_LEAVES_COUNT: u32> AsRef<[ProofEntry<BlockNumber>]>
	for VerificationResult<BlockNumber, MAX_REVEALED_LEAVES_COUNT>
{
	fn as_ref(&self) -> &[ProofEntry<BlockNumber>] {
		self.0.as_ref()
	}
}

/// A type that verifies a Merkle proof that reveals some leaves representing
/// keys in a DID Document.
pub struct DidMerkleProofVerifier<Hasher, AccountId, KeyId, BlockNumber, Details, MaxRevealedLeavesCount>(
	PhantomData<(Hasher, AccountId, KeyId, BlockNumber, Details, MaxRevealedLeavesCount)>,
);

impl<Call, Subject, Hasher, AccountId, KeyId, BlockNumber, Details, const MAX_REVEALED_LEAVES_COUNT: u32>
	IdentityProofVerifier<Call, Subject>
	for DidMerkleProofVerifier<Hasher, AccountId, KeyId, BlockNumber, Details, ConstU32<MAX_REVEALED_LEAVES_COUNT>>
where
	// TODO: Remove `Debug` bound
	BlockNumber: Encode + Clone + Debug,
	Hasher: sp_core::Hasher,
	KeyId: Encode + Clone + Ord + Into<Hasher::Out>,
{
	// TODO: Proper error handling
	type Error = ();
	type Proof = MerkleProof<Vec<Vec<u8>>, ProofLeaf<KeyId, BlockNumber>>;
	type IdentityDetails = IdentityDetails<KeyId, Details>;
	type Submitter = AccountId;
	type VerificationResult = VerificationResult<BlockNumber, MAX_REVEALED_LEAVES_COUNT>;

	fn verify_proof_for_call_against_entry(
		_call: &Call,
		_subject: &Subject,
		_submitter: &Self::Submitter,
		proof_entry: &mut Self::IdentityDetails,
		proof: &Self::Proof,
	) -> Result<Self::VerificationResult, Self::Error> {
		// TODO: more efficient by removing cloning and/or collecting.
		// Did not find another way of mapping a Vec<(Vec<u8>, Vec<u8>)> to a
		// Vec<(Vec<u8>, Option<Vec<u8>>)>.
		let proof_leaves = proof
			.revealed
			.iter()
			.map(|leaf| (leaf.encoded_key(), Some(leaf.encoded_value())))
			.collect::<Vec<(Vec<u8>, Option<Vec<u8>>)>>();
		verify_trie_proof::<LayoutV1<Hasher>, _, _, _>(
			&proof_entry.digest.clone().into(),
			&proof.blinded,
			&proof_leaves,
		)
		.map_err(|_| ())?;

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
			.iter()
			.filter_map(|leaf| {
				if let ProofLeaf::KeyReference(KeyReferenceKey(key_id, key_relationship), _) = leaf {
					// TODO: Better error handling.
					let key_details = public_keys
						.get(key_id)
						.expect("Key ID should be present in the map of revealed public keys.");
					Some(ProofEntry {
						key: key_details.clone(),
						relationship: *key_relationship,
					})
				} else {
					None
				}
			})
			.collect();
		keys.try_into()
	}
}
