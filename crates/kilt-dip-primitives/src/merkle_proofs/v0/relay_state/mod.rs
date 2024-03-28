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

use parity_scale_codec::{Codec, Decode, Encode};
use scale_info::TypeInfo;
use sp_core::U256;
use sp_runtime::{
	generic::Header,
	traits::{AtLeast32BitUnsigned, Hash, Header as HeaderT, MaybeDisplay, Member},
};

use crate::{
	merkle_proofs::v0::{
		input_common::{DidMerkleProof, DipCommitmentStateProof, ProviderHeadStateProof, TimeBoundDidSignature},
		provider_state::ParachainDipDidProof,
	},
	traits::GetWithArg,
	utils::OutputOf,
	DipDidProofWithVerifiedStateRoot, Error,
};

#[cfg(test)]
mod tests;

/// A DIP proof submitted to a relaychain consumer.
///
/// The generic types indicate the following:
/// * `RelayBlockNumber`: The `BlockNumber` definition of the relaychain.
/// * `RelayHasher`: The hashing algorithm used by the relaychain.
/// * `KiltDidKeyId`: The DID key ID type configured by the KILT chain.
/// * `KiltAccountId`: The `AccountId` type configured by the KILT chain.
/// * `KiltBlockNumber`: The `BlockNumber` type configured by the KILT chain.
/// * `KiltWeb3Name`: The web3name type configured by the KILT chain.
/// * `KiltLinkableAccountId`: The linkable account ID type configured by the
///   KILT chain.
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo)]
pub struct RelayDipDidProof<
	RelayBlockNumber: Copy + Into<U256> + TryFrom<U256>,
	RelayHasher: Hash,
	KiltDidKeyId,
	KiltAccountId,
	KiltBlockNumber,
	KiltWeb3Name,
	KiltLinkableAccountId,
> {
	/// The relaychain header for the relaychain block specified in the
	/// `provider_head_proof`.
	pub(crate) relay_header: Header<RelayBlockNumber, RelayHasher>,
	/// The state proof for the given parachain head.
	pub(crate) provider_head_proof: ProviderHeadStateProof<RelayBlockNumber>,
	/// The raw state proof for the DIP commitment of the given subject.
	pub(crate) dip_commitment_proof: DipCommitmentStateProof,
	/// The Merkle proof of the subject's DID details.
	pub(crate) dip_proof:
		DidMerkleProof<KiltDidKeyId, KiltAccountId, KiltBlockNumber, KiltWeb3Name, KiltLinkableAccountId>,
	/// The cross-chain DID signature.
	pub(crate) signature: TimeBoundDidSignature<RelayBlockNumber>,
}

impl<
		RelayBlockNumber: Member + sp_std::hash::Hash + Copy + MaybeDisplay + AtLeast32BitUnsigned + Codec + Into<U256> + TryFrom<U256>,
		RelayHasher: Hash,
		KiltDidKeyId,
		KiltAccountId,
		KiltBlockNumber,
		KiltWeb3Name,
		KiltLinkableAccountId,
	>
	RelayDipDidProof<
		RelayBlockNumber,
		RelayHasher,
		KiltDidKeyId,
		KiltAccountId,
		KiltBlockNumber,
		KiltWeb3Name,
		KiltLinkableAccountId,
	>
{
	/// Verifies the relaychain part of the state proof using the provided block
	/// hash.
	#[allow(clippy::type_complexity)]
	pub fn verify_relay_header_with_block_hash(
		self,
		block_hash: &OutputOf<RelayHasher>,
	) -> Result<
		RelayDipDidProofWithVerifiedRelayStateRoot<
			OutputOf<RelayHasher>,
			RelayBlockNumber,
			KiltDidKeyId,
			KiltAccountId,
			KiltBlockNumber,
			KiltWeb3Name,
			KiltLinkableAccountId,
		>,
		Error,
	> {
		if block_hash != &self.relay_header.hash() {
			return Err(Error::InvalidRelayHeader);
		}

		Ok(RelayDipDidProofWithVerifiedRelayStateRoot {
			relay_state_root: self.relay_header.state_root,
			provider_head_proof: self.provider_head_proof,
			dip_commitment_proof: self.dip_commitment_proof,
			dip_proof: self.dip_proof,
			signature: self.signature,
		})
	}

	/// Verifies the relaychain part of the state proof using the block hash
	/// returned by the provided implementation.
	///
	/// The generic types indicate the following:
	/// * `RelayHashStore`: The type that returns a relaychain block hash given
	///   a relaychain block number.
	#[allow(clippy::type_complexity)]
	pub fn verify_relay_header<RelayHashStore>(
		self,
	) -> Result<
		RelayDipDidProofWithVerifiedRelayStateRoot<
			OutputOf<RelayHasher>,
			RelayBlockNumber,
			KiltDidKeyId,
			KiltAccountId,
			KiltBlockNumber,
			KiltWeb3Name,
			KiltLinkableAccountId,
		>,
		Error,
	>
	where
		RelayHashStore: GetWithArg<RelayBlockNumber, Result = Option<OutputOf<RelayHasher>>>,
	{
		let relay_block_hash = RelayHashStore::get(&self.relay_header.number).ok_or(Error::RelayBlockNotFound)?;
		self.verify_relay_header_with_block_hash(&relay_block_hash)
	}
}

/// A DIP proof submitted to a relaychain consumer that has had the proof header
/// verified against a given block hash.
///
/// The generic types indicate the following:
/// * `StateRoot`: The type of the state root used by the relaychain.
/// * `RelayBlockNumber`: The `BlockNumber` definition of the relaychain.
/// * `KiltDidKeyId`: The DID key ID type configured by the KILT chain.
/// * `KiltAccountId`: The `AccountId` type configured by the KILT chain.
/// * `KiltBlockNumber`: The `BlockNumber` type configured by the KILT chain.
/// * `KiltWeb3Name`: The web3name type configured by the KILT chain.
/// * `KiltLinkableAccountId`: The linkable account ID type configured by the
///   KILT chain.
#[derive(Debug, PartialEq, Eq)]
pub struct RelayDipDidProofWithVerifiedRelayStateRoot<
	StateRoot,
	RelayBlockNumber,
	KiltDidKeyId,
	KiltAccountId,
	KiltBlockNumber,
	KiltWeb3Name,
	KiltLinkableAccountId,
> {
	/// The verified state root for the relaychain at the block specified in the
	/// proof.
	pub(crate) relay_state_root: StateRoot,
	/// The state proof for the given parachain head.
	pub(crate) provider_head_proof: ProviderHeadStateProof<RelayBlockNumber>,
	/// The raw state proof for the DIP commitment of the given subject.
	pub(crate) dip_commitment_proof: DipCommitmentStateProof,
	/// The Merkle proof of the subject's DID details.
	pub(crate) dip_proof:
		DidMerkleProof<KiltDidKeyId, KiltAccountId, KiltBlockNumber, KiltWeb3Name, KiltLinkableAccountId>,
	/// The cross-chain DID signature.
	pub(crate) signature: TimeBoundDidSignature<RelayBlockNumber>,
}

impl<
		StateRoot,
		RelayBlockNumber,
		KiltDidKeyId,
		KiltAccountId,
		KiltBlockNumber,
		KiltWeb3Name,
		KiltLinkableAccountId,
	>
	RelayDipDidProofWithVerifiedRelayStateRoot<
		StateRoot,
		RelayBlockNumber,
		KiltDidKeyId,
		KiltAccountId,
		KiltBlockNumber,
		KiltWeb3Name,
		KiltLinkableAccountId,
	>
{
	/// Verifies the head data of the state proof for the provider with the
	/// given para ID.
	///
	/// The generic types indicate the following:
	/// * `RelayHasher`: The head data hashing algorithm used by the relaychain.
	/// * `ProviderHeader`: The type of the parachain header to be revealed in
	///   the state proof.
	#[allow(clippy::type_complexity)]
	pub fn verify_provider_head_proof<RelayHasher, ProviderHeader>(
		self,
		provider_para_id: u32,
	) -> Result<
		DipDidProofWithVerifiedStateRoot<
			OutputOf<RelayHasher>,
			KiltDidKeyId,
			KiltAccountId,
			KiltBlockNumber,
			KiltWeb3Name,
			KiltLinkableAccountId,
			RelayBlockNumber,
		>,
		Error,
	>
	where
		RelayHasher: Hash<Output = StateRoot>,
		ProviderHeader: Decode + HeaderT<Hash = OutputOf<RelayHasher>, Number = KiltBlockNumber>,
	{
		let parachain_dip_proof = ParachainDipDidProof {
			provider_head_proof: self.provider_head_proof,
			dip_commitment_proof: self.dip_commitment_proof,
			dip_proof: self.dip_proof,
			signature: self.signature,
		};

		parachain_dip_proof.verify_provider_head_proof_with_state_root::<RelayHasher, ProviderHeader>(
			provider_para_id,
			&self.relay_state_root,
		)
	}
}
