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

use did::{did_details::DidPublicKeyDetails, DidVerificationKeyRelationship};
use frame_support::{traits::ConstU32, RuntimeDebug};
use pallet_dip_consumer::{identity::IdentityDetails, traits::IdentityProofVerifier};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{BoundedVec, SaturatedConversion};
use sp_std::{fmt::Debug, marker::PhantomData, vec::Vec};
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
pub struct DidKeyMerkleKey<KeyId>(pub KeyId, pub DidKeyRelationship);

impl<KeyId> From<(KeyId, DidKeyRelationship)> for DidKeyMerkleKey<KeyId> {
	fn from(value: (KeyId, DidKeyRelationship)) -> Self {
		Self(value.0, value.1)
	}
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, PartialOrd, Ord, RuntimeDebug, TypeInfo)]
pub struct DidKeyMerkleValue<BlockNumber>(pub DidPublicKeyDetails<BlockNumber>);

impl<BlockNumber> From<DidPublicKeyDetails<BlockNumber>> for DidKeyMerkleValue<BlockNumber> {
	fn from(value: DidPublicKeyDetails<BlockNumber>) -> Self {
		Self(value)
	}
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, PartialOrd, Ord, RuntimeDebug, TypeInfo)]
pub struct Web3NameMerkleKey<Web3Name>(pub Web3Name);

impl<Web3Name> From<Web3Name> for Web3NameMerkleKey<Web3Name> {
	fn from(value: Web3Name) -> Self {
		Self(value)
	}
}
#[derive(Clone, Encode, Decode, PartialEq, Eq, PartialOrd, Ord, RuntimeDebug, TypeInfo)]
pub struct Web3NameMerkleValue<BlockNumber>(BlockNumber);

impl<BlockNumber> From<BlockNumber> for Web3NameMerkleValue<BlockNumber> {
	fn from(value: BlockNumber) -> Self {
		Self(value)
	}
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, PartialOrd, Ord, RuntimeDebug, TypeInfo)]
pub struct LinkedAccountMerkleKey<AccountId>(pub AccountId);

impl<AccountId> From<AccountId> for LinkedAccountMerkleKey<AccountId> {
	fn from(value: AccountId) -> Self {
		Self(value)
	}
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, PartialOrd, Ord, RuntimeDebug, TypeInfo)]
pub struct LinkedAccountMerkleValue;

impl From<()> for LinkedAccountMerkleValue {
	fn from(_value: ()) -> Self {
		Self
	}
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, PartialOrd, Ord, RuntimeDebug, TypeInfo)]
pub enum ProofLeaf<KeyId, BlockNumber, Web3Name, LinkedAccountId> {
	// The key and value for the leaves of a merkle proof that contain a reference
	// (by ID) to the key details, provided in a separate leaf.
	DidKey(DidKeyMerkleKey<KeyId>, DidKeyMerkleValue<BlockNumber>),
	Web3Name(Web3NameMerkleKey<Web3Name>, Web3NameMerkleValue<BlockNumber>),
	LinkedAccount(LinkedAccountMerkleKey<LinkedAccountId>, LinkedAccountMerkleValue),
}

impl<KeyId, BlockNumber, Web3Name, LinkedAccountId> ProofLeaf<KeyId, BlockNumber, Web3Name, LinkedAccountId>
where
	KeyId: Encode,
	Web3Name: Encode,
	LinkedAccountId: Encode,
{
	pub fn encoded_key(&self) -> Vec<u8> {
		match self {
			ProofLeaf::DidKey(key, _) => key.encode(),
			ProofLeaf::Web3Name(key, _) => key.encode(),
			ProofLeaf::LinkedAccount(key, _) => key.encode(),
		}
	}
}

impl<KeyId, BlockNumber, Web3Name, LinkedAccountId> ProofLeaf<KeyId, BlockNumber, Web3Name, LinkedAccountId>
where
	BlockNumber: Encode,
{
	pub fn encoded_value(&self) -> Vec<u8> {
		match self {
			ProofLeaf::DidKey(_, value) => value.encode(),
			ProofLeaf::Web3Name(_, value) => value.encode(),
			ProofLeaf::LinkedAccount(_, value) => value.encode(),
		}
	}
}

#[derive(Clone, Encode, Decode, PartialEq, MaxEncodedLen, Eq, PartialOrd, Ord, RuntimeDebug, TypeInfo)]
pub struct RevealedDidKey<KeyId, BlockNumber> {
	pub id: KeyId,
	pub relationship: DidKeyRelationship,
	pub details: DidPublicKeyDetails<BlockNumber>,
}

#[derive(Clone, Encode, Decode, PartialEq, MaxEncodedLen, Eq, PartialOrd, Ord, RuntimeDebug, TypeInfo)]
pub struct RevealedWeb3Name<Web3Name, BlockNumber> {
	pub web3_name: Web3Name,
	pub claimed_at: BlockNumber,
}

#[derive(Clone, Debug, PartialEq, Eq, TypeInfo, MaxEncodedLen, Encode, Decode, Default)]
pub struct VerificationResult<
	KeyId,
	BlockNumber,
	Web3Name,
	LinkedAccountId,
	const MAX_REVEALED_KEYS_COUNT: u32,
	const MAX_REVEALED_ACCOUNTS_COUNT: u32,
> {
	pub did_keys: BoundedVec<RevealedDidKey<KeyId, BlockNumber>, ConstU32<MAX_REVEALED_KEYS_COUNT>>,
	pub web3_name: Option<RevealedWeb3Name<Web3Name, BlockNumber>>,
	pub linked_accounts: BoundedVec<LinkedAccountId, ConstU32<MAX_REVEALED_ACCOUNTS_COUNT>>,
}

impl<
		KeyId,
		BlockNumber,
		Web3Name,
		LinkedAccountId,
		const MAX_REVEALED_KEYS_COUNT: u32,
		const MAX_REVEALED_ACCOUNTS_COUNT: u32,
	> AsRef<[RevealedDidKey<KeyId, BlockNumber>]>
	for VerificationResult<
		KeyId,
		BlockNumber,
		Web3Name,
		LinkedAccountId,
		MAX_REVEALED_KEYS_COUNT,
		MAX_REVEALED_ACCOUNTS_COUNT,
	>
{
	fn as_ref(&self) -> &[RevealedDidKey<KeyId, BlockNumber>] {
		self.did_keys.as_ref()
	}
}

/// A type that verifies a Merkle proof that reveals some leaves representing
/// keys in a DID Document.
/// Can also be used on its own, without any DID signature verification.
pub struct DidMerkleProofVerifier<
	Hasher,
	AccountId,
	KeyId,
	BlockNumber,
	Details,
	Web3Name,
	LinkedAccountId,
	const MAX_REVEALED_KEYS_COUNT: u32,
	const MAX_REVEALED_ACCOUNTS_COUNT: u32,
>(
	#[allow(clippy::type_complexity)]
	PhantomData<(
		Hasher,
		AccountId,
		KeyId,
		BlockNumber,
		Details,
		Web3Name,
		LinkedAccountId,
		ConstU32<MAX_REVEALED_KEYS_COUNT>,
		ConstU32<MAX_REVEALED_ACCOUNTS_COUNT>,
	)>,
);

impl<
		Call,
		Subject,
		Hasher,
		AccountId,
		KeyId,
		BlockNumber,
		Details,
		Web3Name,
		LinkedAccountId,
		const MAX_REVEALED_KEYS_COUNT: u32,
		const MAX_REVEALED_ACCOUNTS_COUNT: u32,
	> IdentityProofVerifier<Call, Subject>
	for DidMerkleProofVerifier<
		Hasher,
		AccountId,
		KeyId,
		BlockNumber,
		Details,
		Web3Name,
		LinkedAccountId,
		MAX_REVEALED_KEYS_COUNT,
		MAX_REVEALED_ACCOUNTS_COUNT,
	> where
	// TODO: Remove `Debug` bound
	BlockNumber: Encode + Clone + Debug,
	Hasher: sp_core::Hasher,
	KeyId: Encode + Clone + Ord + Into<Hasher::Out>,
	LinkedAccountId: Encode + Clone,
	Web3Name: Encode + Clone,
{
	// TODO: Proper error handling
	type Error = ();
	type Proof = MerkleProof<Vec<Vec<u8>>, ProofLeaf<KeyId, BlockNumber, Web3Name, LinkedAccountId>>;
	type IdentityDetails = IdentityDetails<KeyId, Details>;
	type Submitter = AccountId;
	type VerificationResult = VerificationResult<
		KeyId,
		BlockNumber,
		Web3Name,
		LinkedAccountId,
		MAX_REVEALED_KEYS_COUNT,
		MAX_REVEALED_ACCOUNTS_COUNT,
	>;

	fn verify_proof_for_call_against_details(
		_call: &Call,
		_subject: &Subject,
		_submitter: &Self::Submitter,
		identity_details: &mut Self::IdentityDetails,
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
			&identity_details.digest.clone().into(),
			&proof.blinded,
			&proof_leaves,
		)
		.map_err(|_| ())?;

		// At this point, we know the proof is valid. We just need to map the revealed
		// leaves to something the consumer can easily operate on.
		#[allow(clippy::type_complexity)]
		let (did_keys, web3_name, linked_accounts): (
			BoundedVec<RevealedDidKey<KeyId, BlockNumber>, ConstU32<MAX_REVEALED_KEYS_COUNT>>,
			Option<RevealedWeb3Name<Web3Name, BlockNumber>>,
			BoundedVec<LinkedAccountId, ConstU32<MAX_REVEALED_ACCOUNTS_COUNT>>,
		) = proof.revealed.iter().try_fold(
			(
				BoundedVec::with_bounded_capacity(MAX_REVEALED_KEYS_COUNT.saturated_into()),
				None,
				BoundedVec::with_bounded_capacity(MAX_REVEALED_ACCOUNTS_COUNT.saturated_into()),
			),
			|(mut keys, web3_name, mut linked_accounts), leaf| match leaf {
				ProofLeaf::DidKey(key_id, key_value) => {
					keys.try_push(RevealedDidKey {
						// TODO: Avoid cloning if possible
						id: key_id.0.clone(),
						relationship: key_id.1,
						details: key_value.0.clone(),
					})
					.map_err(|_| ())?;
					Ok::<_, ()>((keys, web3_name, linked_accounts))
				}
				// TODO: Avoid cloning if possible
				ProofLeaf::Web3Name(revealed_web3_name, details) => Ok((
					keys,
					Some(RevealedWeb3Name {
						web3_name: revealed_web3_name.0.clone(),
						claimed_at: details.0.clone(),
					}),
					linked_accounts,
				)),
				ProofLeaf::LinkedAccount(account_id, _) => {
					linked_accounts.try_push(account_id.0.clone()).map_err(|_| ())?;
					Ok::<_, ()>((keys, web3_name, linked_accounts))
				}
			},
		)?;

		Ok(VerificationResult {
			did_keys,
			web3_name,
			linked_accounts,
		})
	}
}
