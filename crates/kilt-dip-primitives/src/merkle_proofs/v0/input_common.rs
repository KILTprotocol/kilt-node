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

use did::DidSignature;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::vec::Vec;

use crate::merkle_proofs::v0::output_common::RevealedDidMerkleProofLeaf;

/// The state proof for a parachain head.
///
/// The generic types indicate the following:
/// * `RelayBlockNumber`: The `BlockNumber` definition of the relaychain.
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo)]
#[cfg_attr(test, derive(Default))]
pub struct ProviderHeadStateProof<RelayBlockNumber> {
	pub(crate) relay_block_number: RelayBlockNumber,
	pub(crate) proof: Vec<Vec<u8>>,
}

impl<RelayBlockNumber> ProviderHeadStateProof<RelayBlockNumber> {
	pub fn new(relay_block_number: RelayBlockNumber, proof: Vec<Vec<u8>>) -> Self {
		Self {
			proof,
			relay_block_number,
		}
	}
}

/// The state proof for a DIP commitment.
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo)]
#[cfg_attr(test, derive(Default))]
pub struct DipCommitmentStateProof(pub(crate) Vec<Vec<u8>>);

impl DipCommitmentStateProof {
	pub fn new(proof: Vec<Vec<u8>>) -> Self {
		Self(proof)
	}
}

/// The Merkle proof for a KILT DID.
///
/// The generic types indicate the following:
/// * `ProviderDidKeyId`: The DID key ID type configured by the provider.
/// * `ProviderAccountId`: The `AccountId` type configured by the provider.
/// * `ProviderBlockNumber`: The `BlockNumber` type configured by the provider.
/// * `ProviderWeb3Name`: The web3name type configured by the provider.
/// * `ProviderLinkableAccountId`: The linkable account ID type configured by
///   the provider.
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo)]
pub struct DidMerkleProof<
	ProviderDidKeyId,
	ProviderAccountId,
	ProviderBlockNumber,
	ProviderWeb3Name,
	ProviderLinkableAccountId,
> {
	pub(crate) blinded: Vec<Vec<u8>>,
	pub(crate) revealed: Vec<
		RevealedDidMerkleProofLeaf<
			ProviderDidKeyId,
			ProviderAccountId,
			ProviderBlockNumber,
			ProviderWeb3Name,
			ProviderLinkableAccountId,
		>,
	>,
}

impl<ProviderDidKeyId, ProviderAccountId, ProviderBlockNumber, ProviderWeb3Name, ProviderLinkableAccountId>
	DidMerkleProof<ProviderDidKeyId, ProviderAccountId, ProviderBlockNumber, ProviderWeb3Name, ProviderLinkableAccountId>
{
	pub fn new(
		blinded: Vec<Vec<u8>>,
		revealed: Vec<
			RevealedDidMerkleProofLeaf<
				ProviderDidKeyId,
				ProviderAccountId,
				ProviderBlockNumber,
				ProviderWeb3Name,
				ProviderLinkableAccountId,
			>,
		>,
	) -> Self {
		Self { blinded, revealed }
	}

	pub fn revealed(
		&self,
	) -> &[RevealedDidMerkleProofLeaf<
		ProviderDidKeyId,
		ProviderAccountId,
		ProviderBlockNumber,
		ProviderWeb3Name,
		ProviderLinkableAccountId,
	>] {
		self.revealed.as_ref()
	}
}

#[cfg(test)]
impl<ProviderDidKeyId, ProviderAccountId, ProviderBlockNumber, ProviderWeb3Name, ProviderLinkableAccountId> Default
	for DidMerkleProof<
		ProviderDidKeyId,
		ProviderAccountId,
		ProviderBlockNumber,
		ProviderWeb3Name,
		ProviderLinkableAccountId,
	> where
	ProviderDidKeyId: Default,
	ProviderBlockNumber: Default,
{
	fn default() -> Self {
		Self {
			revealed: Default::default(),
			blinded: Default::default(),
		}
	}
}

/// A DID signature anchored to a specific block height.
///
/// The generic types indicate the following:
/// * `BlockNumber`: The `BlockNumber` definition of the chain consuming (i.e.,
///   validating) this signature.
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo)]
pub struct TimeBoundDidSignature<BlockNumber> {
	/// The signature.
	pub(crate) signature: DidSignature,
	/// The block number until the signature is to be considered valid.
	pub(crate) valid_until: BlockNumber,
}

impl<BlockNumber> TimeBoundDidSignature<BlockNumber> {
	pub fn new(signature: DidSignature, valid_until: BlockNumber) -> Self {
		Self { signature, valid_until }
	}
}

#[cfg(test)]
impl<BlockNumber> Default for TimeBoundDidSignature<BlockNumber>
where
	BlockNumber: Default,
{
	fn default() -> Self {
		Self {
			signature: DidSignature::Ed25519(sp_core::ed25519::Signature([0u8; 64])),
			valid_until: BlockNumber::default(),
		}
	}
}
