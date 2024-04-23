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
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::ConstU32;
use sp_runtime::{BoundedVec, SaturatedConversion};
use sp_std::{fmt::Debug, vec::Vec};

use crate::Error;

/// Information, available as an origin, after the whole DIP proof has been
/// verified.
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub struct DipOriginInfo<
	KiltDidKeyId,
	KiltAccountId,
	KiltBlockNumber,
	KiltWeb3Name,
	KiltLinkableAccountId,
	const MAX_REVEALED_LEAVES_COUNT: u32,
> {
	/// The parts of the subject's DID details revealed in the DIP proof.
	pub(crate) revealed_leaves: BoundedVec<
		RevealedDidMerkleProofLeaf<KiltDidKeyId, KiltAccountId, KiltBlockNumber, KiltWeb3Name, KiltLinkableAccountId>,
		ConstU32<MAX_REVEALED_LEAVES_COUNT>,
	>,
	/// The index of the signing leaves from the vector above.
	pub(crate) signing_leaves_indices: BoundedVec<u32, ConstU32<MAX_REVEALED_LEAVES_COUNT>>,
}

impl<
		KiltDidKeyId,
		KiltAccountId,
		KiltBlockNumber,
		KiltWeb3Name,
		KiltLinkableAccountId,
		const MAX_REVEALED_LEAVES_COUNT: u32,
	>
	DipOriginInfo<
		KiltDidKeyId,
		KiltAccountId,
		KiltBlockNumber,
		KiltWeb3Name,
		KiltLinkableAccountId,
		MAX_REVEALED_LEAVES_COUNT,
	>
{
	/// Returns an iterator over the revealed DID leaves.
	pub fn iter_leaves(
		&self,
	) -> impl Iterator<
		Item = &RevealedDidMerkleProofLeaf<
			KiltDidKeyId,
			KiltAccountId,
			KiltBlockNumber,
			KiltWeb3Name,
			KiltLinkableAccountId,
		>,
	> {
		self.revealed_leaves.iter()
	}

	/// Returns an owned iterator over the revealed DID leaves.
	pub fn into_iter_leaves(
		self,
	) -> impl Iterator<
		Item = RevealedDidMerkleProofLeaf<
			KiltDidKeyId,
			KiltAccountId,
			KiltBlockNumber,
			KiltWeb3Name,
			KiltLinkableAccountId,
		>,
	> {
		self.revealed_leaves.into_iter()
	}

	/// Returns a reference to the leaves that signed the cross-chain operation.
	/// This operation should never fail, so the only error it returns is an
	/// `Error::Internal` which, anyway, should never happen.
	pub fn get_signing_leaves(
		&self,
	) -> Result<impl Iterator<Item = &RevealedDidKey<KiltDidKeyId, KiltBlockNumber, KiltAccountId>>, Error>
	where
		KiltDidKeyId: Debug,
		KiltBlockNumber: Debug,
		KiltAccountId: Debug,
		KiltWeb3Name: Debug,
		KiltLinkableAccountId: Debug,
	{
		const LOG_TARGET: &str = "dip::consumer::DipOriginInfoV0";
		let leaves = self
			.signing_leaves_indices
			.iter()
			.map(|index| {
				let leaf = self.revealed_leaves.get(usize::saturated_from(*index)).ok_or_else(|| {
					log::error!(
						target: LOG_TARGET,
						"Failed to retrieve the signing leaf at index {:#?}.",
						index
					);
					Error::Internal
				})?;
				let RevealedDidMerkleProofLeaf::DidKey(did_key) = leaf else {
					log::error!(target: LOG_TARGET, "Failed to convert the signing leaf {:#?} to a DID Key leaf.", leaf);
					return Err(Error::Internal);
				};
				Ok(did_key)
			})
			.collect::<Result<Vec<_>, _>>()?;
		Ok(leaves.into_iter())
	}
}

#[cfg(feature = "runtime-benchmarks")]
impl<
		KiltDidKeyId,
		KiltAccountId,
		KiltBlockNumber,
		KiltWeb3Name,
		KiltLinkableAccountId,
		const MAX_REVEALED_LEAVES_COUNT: u32,
	> Default
	for DipOriginInfo<
		KiltDidKeyId,
		KiltAccountId,
		KiltBlockNumber,
		KiltWeb3Name,
		KiltLinkableAccountId,
		MAX_REVEALED_LEAVES_COUNT,
	> where
	KiltDidKeyId: crate::traits::BenchmarkDefault,
	KiltBlockNumber: crate::traits::BenchmarkDefault,
{
	fn default() -> Self {
		let default_keys = sp_std::vec![RevealedDidKey {
			id: KiltDidKeyId::default(),
			details: DidPublicKeyDetails {
				key: did::did_details::DidVerificationKey::Ed25519(sp_core::ed25519::Public::from_raw([0u8; 32]))
					.into(),
				block_number: KiltBlockNumber::default()
			},
			relationship: DidVerificationKeyRelationship::Authentication.into()
		}
		.into()];
		let bounded_keys = default_keys
			.try_into()
			// To avoid requiring types to implement `Debug`.
			.map_err(|_| "Should not fail to convert single element to a BoundedVec.")
			.unwrap();
		Self {
			revealed_leaves: bounded_keys,
			signing_leaves_indices: Default::default(),
		}
	}
}

/// Relationship of a key to a DID Document.
#[derive(Clone, Copy, Debug, Encode, Decode, PartialEq, Eq, PartialOrd, Ord, TypeInfo, MaxEncodedLen)]
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

/// All possible Merkle leaf types that can be revealed as part of a DIP
/// identity Merkle proof.
#[derive(Clone, Copy, Debug, Encode, Decode, PartialEq, Eq, TypeInfo, MaxEncodedLen)]
pub enum RevealedDidMerkleProofLeaf<KeyId, AccountId, BlockNumber, Web3Name, LinkedAccountId> {
	DidKey(RevealedDidKey<KeyId, BlockNumber, AccountId>),
	Web3Name(RevealedWeb3Name<Web3Name, BlockNumber>),
	LinkedAccount(RevealedAccountId<LinkedAccountId>),
}

impl<KeyId, AccountId, BlockNumber, Web3Name, LinkedAccountId> From<RevealedDidKey<KeyId, BlockNumber, AccountId>>
	for RevealedDidMerkleProofLeaf<KeyId, AccountId, BlockNumber, Web3Name, LinkedAccountId>
{
	fn from(value: RevealedDidKey<KeyId, BlockNumber, AccountId>) -> Self {
		Self::DidKey(value)
	}
}

impl<KeyId, AccountId, BlockNumber, Web3Name, LinkedAccountId> From<RevealedWeb3Name<Web3Name, BlockNumber>>
	for RevealedDidMerkleProofLeaf<KeyId, AccountId, BlockNumber, Web3Name, LinkedAccountId>
{
	fn from(value: RevealedWeb3Name<Web3Name, BlockNumber>) -> Self {
		Self::Web3Name(value)
	}
}

impl<KeyId, AccountId, BlockNumber, Web3Name, LinkedAccountId> From<RevealedAccountId<LinkedAccountId>>
	for RevealedDidMerkleProofLeaf<KeyId, AccountId, BlockNumber, Web3Name, LinkedAccountId>
{
	fn from(value: RevealedAccountId<LinkedAccountId>) -> Self {
		Self::LinkedAccount(value)
	}
}

#[cfg(any(test, feature = "runtime-benchmarks"))]
impl<KeyId, AccountId, BlockNumber, Web3Name, LinkedAccountId> Default
	for RevealedDidMerkleProofLeaf<KeyId, AccountId, BlockNumber, Web3Name, LinkedAccountId>
where
	KeyId: Default,
	BlockNumber: Default,
{
	fn default() -> Self {
		RevealedDidKey {
			id: KeyId::default(),
			relationship: DidVerificationKeyRelationship::Authentication.into(),
			details: DidPublicKeyDetails {
				key: did::did_details::DidVerificationKey::Ed25519(sp_core::ed25519::Public::from_raw([0u8; 32]))
					.into(),
				block_number: BlockNumber::default(),
			},
		}
		.into()
	}
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
			RevealedDidMerkleProofLeaf::DidKey(RevealedDidKey { id, relationship, .. }) => (id, relationship).encode(),
			RevealedDidMerkleProofLeaf::Web3Name(RevealedWeb3Name { web3_name, .. }) => web3_name.encode(),
			RevealedDidMerkleProofLeaf::LinkedAccount(RevealedAccountId(account_id)) => account_id.encode(),
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
			RevealedDidMerkleProofLeaf::DidKey(RevealedDidKey { details, .. }) => details.encode(),
			RevealedDidMerkleProofLeaf::Web3Name(RevealedWeb3Name { claimed_at, .. }) => claimed_at.encode(),
			RevealedDidMerkleProofLeaf::LinkedAccount(_) => ().encode(),
		}
	}
}

/// The details of a DID key after it has been successfully verified in a Merkle
/// proof.
#[derive(Clone, Copy, Debug, Encode, Decode, PartialEq, Eq, PartialOrd, Ord, TypeInfo, MaxEncodedLen)]
pub struct RevealedDidKey<KeyId, BlockNumber, AccountId> {
	/// The key ID, according to the provider's definition.
	pub id: KeyId,
	/// The key relationship to the subject's DID Document.
	pub relationship: DidKeyRelationship,
	/// The details of the DID Key, including its creation block number on the
	/// provider chain.
	pub details: DidPublicKeyDetails<BlockNumber, AccountId>,
}

/// The details of a web3name after it has been successfully verified in a
/// Merkle proof.
#[derive(Clone, Copy, Debug, Encode, Decode, PartialEq, Eq, PartialOrd, Ord, TypeInfo, MaxEncodedLen)]
pub struct RevealedWeb3Name<Web3Name, BlockNumber> {
	/// The web3name.
	pub web3_name: Web3Name,
	/// The block number on the provider chain in which it was linked to the DID
	/// subject.
	pub claimed_at: BlockNumber,
}

/// The details of an account after it has been successfully verified in a
/// Merkle proof.
#[derive(Clone, Copy, Debug, Encode, Decode, PartialEq, Eq, PartialOrd, Ord, TypeInfo, MaxEncodedLen)]
pub struct RevealedAccountId<AccountId>(pub AccountId);
