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
use frame_support::{traits::ConstU32, DefaultNoBound, RuntimeDebug};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{BoundedVec, SaturatedConversion};
use sp_std::{fmt::Debug, marker::PhantomData, vec::Vec};
use sp_trie::{verify_trie_proof, LayoutV1};

#[derive(Encode, Decode, RuntimeDebug, Clone, Eq, PartialEq, Default, TypeInfo)]
pub struct DidMerkleProof<BlindedValues, Leaf> {
	pub blinded: BlindedValues,
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
pub struct DidKeyMerkleValue<BlockNumber, AccountId>(pub DidPublicKeyDetails<BlockNumber, AccountId>);

impl<BlockNumber, AccountId> From<DidPublicKeyDetails<BlockNumber, AccountId>>
	for DidKeyMerkleValue<BlockNumber, AccountId>
{
	fn from(value: DidPublicKeyDetails<BlockNumber, AccountId>) -> Self {
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
pub struct Web3NameMerkleValue<BlockNumber>(pub BlockNumber);

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
pub enum RevealedDidMerkleProofLeaf<KeyId, AccountId, BlockNumber, Web3Name, LinkedAccountId> {
	// The key and value for the leaves of a merkle proof that contain a reference
	// (by ID) to the key details, provided in a separate leaf.
	DidKey(DidKeyMerkleKey<KeyId>, DidKeyMerkleValue<BlockNumber, AccountId>),
	Web3Name(Web3NameMerkleKey<Web3Name>, Web3NameMerkleValue<BlockNumber>),
	LinkedAccount(LinkedAccountMerkleKey<LinkedAccountId>, LinkedAccountMerkleValue),
}

impl<KeyId, AccountId, BlockNumber, Web3Name, LinkedAccountId>
	RevealedDidMerkleProofLeaf<KeyId, AccountId, BlockNumber, Web3Name, LinkedAccountId>
where
	KeyId: Encode,
	Web3Name: Encode,
	LinkedAccountId: Encode,
{
	pub fn encoded_key(&self) -> Vec<u8> {
		match self {
			RevealedDidMerkleProofLeaf::DidKey(key, _) => key.encode(),
			RevealedDidMerkleProofLeaf::Web3Name(key, _) => key.encode(),
			RevealedDidMerkleProofLeaf::LinkedAccount(key, _) => key.encode(),
		}
	}
}

impl<KeyId, AccountId, BlockNumber, Web3Name, LinkedAccountId>
	RevealedDidMerkleProofLeaf<KeyId, AccountId, BlockNumber, Web3Name, LinkedAccountId>
where
	AccountId: Encode,
	BlockNumber: Encode,
{
	pub fn encoded_value(&self) -> Vec<u8> {
		match self {
			RevealedDidMerkleProofLeaf::DidKey(_, value) => value.encode(),
			RevealedDidMerkleProofLeaf::Web3Name(_, value) => value.encode(),
			RevealedDidMerkleProofLeaf::LinkedAccount(_, value) => value.encode(),
		}
	}
}

#[derive(Clone, Encode, Decode, PartialEq, MaxEncodedLen, Eq, PartialOrd, Ord, RuntimeDebug, TypeInfo)]
pub struct RevealedDidKey<KeyId, BlockNumber, AccountId> {
	pub id: KeyId,
	pub relationship: DidKeyRelationship,
	pub details: DidPublicKeyDetails<BlockNumber, AccountId>,
}

#[derive(Clone, Encode, Decode, PartialEq, MaxEncodedLen, Eq, PartialOrd, Ord, RuntimeDebug, TypeInfo)]
pub struct RevealedWeb3Name<Web3Name, BlockNumber> {
	pub web3_name: Web3Name,
	pub claimed_at: BlockNumber,
}

#[derive(Clone, Debug, PartialEq, Eq, TypeInfo, MaxEncodedLen, Encode, Decode, DefaultNoBound)]
pub struct RevealedDidMerkleProofLeaves<
	KeyId,
	AccountId,
	BlockNumber,
	Web3Name,
	LinkedAccountId,
	const MAX_REVEALED_KEYS_COUNT: u32,
	const MAX_REVEALED_ACCOUNTS_COUNT: u32,
> {
	pub did_keys: BoundedVec<RevealedDidKey<KeyId, BlockNumber, AccountId>, ConstU32<MAX_REVEALED_KEYS_COUNT>>,
	pub web3_name: Option<RevealedWeb3Name<Web3Name, BlockNumber>>,
	pub linked_accounts: BoundedVec<LinkedAccountId, ConstU32<MAX_REVEALED_ACCOUNTS_COUNT>>,
}

impl<
		KeyId,
		AccountId,
		BlockNumber,
		Web3Name,
		LinkedAccountId,
		const MAX_REVEALED_KEYS_COUNT: u32,
		const MAX_REVEALED_ACCOUNTS_COUNT: u32,
	> sp_std::borrow::Borrow<[RevealedDidKey<KeyId, BlockNumber, AccountId>]>
	for RevealedDidMerkleProofLeaves<
		KeyId,
		AccountId,
		BlockNumber,
		Web3Name,
		LinkedAccountId,
		MAX_REVEALED_KEYS_COUNT,
		MAX_REVEALED_ACCOUNTS_COUNT,
	>
{
	fn borrow(&self) -> &[RevealedDidKey<KeyId, BlockNumber, AccountId>] {
		self.did_keys.borrow()
	}
}

pub enum DidMerkleProofVerifierError {
	InvalidMerkleProof,
	TooManyRevealedKeys,
	TooManyRevealedAccounts,
}

impl From<DidMerkleProofVerifierError> for u8 {
	fn from(value: DidMerkleProofVerifierError) -> Self {
		match value {
			DidMerkleProofVerifierError::InvalidMerkleProof => 0,
			DidMerkleProofVerifierError::TooManyRevealedKeys => 1,
			DidMerkleProofVerifierError::TooManyRevealedAccounts => 2,
		}
	}
}

/// A type that verifies a Merkle proof that reveals some leaves representing
/// keys in a DID Document.
/// Can also be used on its own, without any DID signature verification.
pub(crate) struct DidMerkleProofVerifier<
	Hasher,
	KeyId,
	AccountId,
	BlockNumber,
	Web3Name,
	LinkedAccountId,
	const MAX_REVEALED_KEYS_COUNT: u32,
	const MAX_REVEALED_ACCOUNTS_COUNT: u32,
>(#[allow(clippy::type_complexity)] PhantomData<(Hasher, KeyId, AccountId, BlockNumber, Web3Name, LinkedAccountId)>);

impl<
		Hasher,
		KeyId,
		AccountId,
		BlockNumber,
		Web3Name,
		LinkedAccountId,
		const MAX_REVEALED_KEYS_COUNT: u32,
		const MAX_REVEALED_ACCOUNTS_COUNT: u32,
	>
	DidMerkleProofVerifier<
		Hasher,
		KeyId,
		AccountId,
		BlockNumber,
		Web3Name,
		LinkedAccountId,
		MAX_REVEALED_KEYS_COUNT,
		MAX_REVEALED_ACCOUNTS_COUNT,
	> where
	BlockNumber: Encode + Clone,
	Hasher: sp_core::Hasher,
	KeyId: Encode + Clone,
	AccountId: Encode + Clone,
	LinkedAccountId: Encode + Clone,
	Web3Name: Encode + Clone,
{
	#[allow(clippy::result_unit_err)]
	#[allow(clippy::type_complexity)]
	pub(crate) fn verify_dip_merkle_proof(
		identity_commitment: &Hasher::Out,
		proof: DidMerkleProof<
			Vec<Vec<u8>>,
			RevealedDidMerkleProofLeaf<KeyId, AccountId, BlockNumber, Web3Name, LinkedAccountId>,
		>,
	) -> Result<
		RevealedDidMerkleProofLeaves<
			KeyId,
			AccountId,
			BlockNumber,
			Web3Name,
			LinkedAccountId,
			MAX_REVEALED_KEYS_COUNT,
			MAX_REVEALED_ACCOUNTS_COUNT,
		>,
		DidMerkleProofVerifierError,
	> {
		// TODO: more efficient by removing cloning and/or collecting.
		// Did not find another way of mapping a Vec<(Vec<u8>, Vec<u8>)> to a
		// Vec<(Vec<u8>, Option<Vec<u8>>)>.
		let proof_leaves = proof
			.revealed
			.iter()
			.map(|leaf| (leaf.encoded_key(), Some(leaf.encoded_value())))
			.collect::<Vec<(Vec<u8>, Option<Vec<u8>>)>>();
		verify_trie_proof::<LayoutV1<Hasher>, _, _, _>(identity_commitment, &proof.blinded, &proof_leaves)
			.map_err(|_| DidMerkleProofVerifierError::InvalidMerkleProof)?;

		// At this point, we know the proof is valid. We just need to map the revealed
		// leaves to something the consumer can easily operate on.
		#[allow(clippy::type_complexity)]
		let (did_keys, web3_name, linked_accounts): (
			BoundedVec<RevealedDidKey<KeyId, BlockNumber, AccountId>, ConstU32<MAX_REVEALED_KEYS_COUNT>>,
			Option<RevealedWeb3Name<Web3Name, BlockNumber>>,
			BoundedVec<LinkedAccountId, ConstU32<MAX_REVEALED_ACCOUNTS_COUNT>>,
		) = proof.revealed.iter().try_fold(
			(
				BoundedVec::with_bounded_capacity(MAX_REVEALED_KEYS_COUNT.saturated_into()),
				None,
				BoundedVec::with_bounded_capacity(MAX_REVEALED_ACCOUNTS_COUNT.saturated_into()),
			),
			|(mut keys, web3_name, mut linked_accounts), leaf| match leaf {
				RevealedDidMerkleProofLeaf::DidKey(key_id, key_value) => {
					keys.try_push(RevealedDidKey {
						// TODO: Avoid cloning if possible
						id: key_id.0.clone(),
						relationship: key_id.1,
						details: key_value.0.clone(),
					})
					.map_err(|_| DidMerkleProofVerifierError::TooManyRevealedKeys)?;
					Ok::<_, DidMerkleProofVerifierError>((keys, web3_name, linked_accounts))
				}
				// TODO: Avoid cloning if possible
				RevealedDidMerkleProofLeaf::Web3Name(revealed_web3_name, details) => Ok((
					keys,
					Some(RevealedWeb3Name {
						web3_name: revealed_web3_name.0.clone(),
						claimed_at: details.0.clone(),
					}),
					linked_accounts,
				)),
				RevealedDidMerkleProofLeaf::LinkedAccount(account_id, _) => {
					linked_accounts
						.try_push(account_id.0.clone())
						.map_err(|_| DidMerkleProofVerifierError::TooManyRevealedAccounts)?;
					Ok::<_, DidMerkleProofVerifierError>((keys, web3_name, linked_accounts))
				}
			},
		)?;

		Ok(RevealedDidMerkleProofLeaves {
			did_keys,
			web3_name,
			linked_accounts,
		})
	}
}
