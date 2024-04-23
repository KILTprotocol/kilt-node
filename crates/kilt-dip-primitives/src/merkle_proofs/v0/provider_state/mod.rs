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

use frame_support::ensure;
use pallet_dip_provider::IdentityCommitmentOf;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{Hash, Header as HeaderT},
	BoundedVec, SaturatedConversion,
};
use sp_std::{fmt::Debug, vec::Vec};
use sp_trie::{verify_trie_proof, LayoutV1};

use crate::{
	merkle_proofs::v0::{
		dip_subject_state::DipRevealedDetailsAndUnverifiedDidSignature,
		input_common::{DidMerkleProof, DipCommitmentStateProof, ProviderHeadStateProof, TimeBoundDidSignature},
	},
	state_proofs::{verify_storage_value_proof, verify_storage_value_proof_with_decoder},
	traits::GetWithArg,
	utils::{
		calculate_dip_identity_commitment_storage_key_for_runtime, calculate_parachain_head_storage_key, OutputOf,
	},
	Error,
};

#[cfg(test)]
mod tests;

/// A DIP proof submitted to a parachain consumer.
///
/// The generic types indicate the following:
/// * `RelayBlockNumber`: The `BlockNumber` definition of the relaychain.
/// * `KiltDidKeyId`: The DID key ID type configured by the KILT chain.
/// * `KiltAccountId`: The `AccountId` type configured by the KILT chain.
/// * `KiltBlockNumber`: The `BlockNumber` type configured by the KILT chain.
/// * `KiltWeb3Name`: The web3name type configured by the KILT chain.
/// * `KiltLinkableAccountId`: The linkable account ID type configured by the
///   KILT chain.
/// * `ConsumerBlockNumber`: The `BlockNumber` definition of the consumer
///   parachain.
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq, TypeInfo)]
pub struct ParachainDipDidProof<
	RelayBlockNumber,
	KiltDidKeyId,
	KiltAccountId,
	KiltBlockNumber,
	KiltWeb3Name,
	KiltLinkableAccountId,
	ConsumerBlockNumber,
> {
	/// The state proof for the given parachain head.
	pub(crate) provider_head_proof: ProviderHeadStateProof<RelayBlockNumber>,
	/// The raw state proof for the DIP commitment of the given subject.
	pub(crate) dip_commitment_proof: DipCommitmentStateProof,
	/// The Merkle proof of the subject's DID details.
	pub(crate) dip_proof:
		DidMerkleProof<KiltDidKeyId, KiltAccountId, KiltBlockNumber, KiltWeb3Name, KiltLinkableAccountId>,
	/// The cross-chain DID signature.
	pub(crate) signature: TimeBoundDidSignature<ConsumerBlockNumber>,
}

impl<
		RelayBlockNumber,
		KiltDidKeyId,
		KiltAccountId,
		KiltBlockNumber,
		KiltWeb3Name,
		KiltLinkableAccountId,
		ConsumerBlockNumber,
	>
	ParachainDipDidProof<
		RelayBlockNumber,
		KiltDidKeyId,
		KiltAccountId,
		KiltBlockNumber,
		KiltWeb3Name,
		KiltLinkableAccountId,
		ConsumerBlockNumber,
	>
{
	pub fn new(
		provider_head_proof: ProviderHeadStateProof<RelayBlockNumber>,
		dip_commitment_proof: DipCommitmentStateProof,
		dip_proof: DidMerkleProof<KiltDidKeyId, KiltAccountId, KiltBlockNumber, KiltWeb3Name, KiltLinkableAccountId>,
		signature: TimeBoundDidSignature<ConsumerBlockNumber>,
	) -> Self {
		Self {
			dip_commitment_proof,
			dip_proof,
			provider_head_proof,
			signature,
		}
	}

	pub fn provider_head_proof(&self) -> &ProviderHeadStateProof<RelayBlockNumber> {
		&self.provider_head_proof
	}

	pub fn dip_commitment_proof(&self) -> &DipCommitmentStateProof {
		&self.dip_commitment_proof
	}

	pub fn dip_proof(
		&self,
	) -> &DidMerkleProof<KiltDidKeyId, KiltAccountId, KiltBlockNumber, KiltWeb3Name, KiltLinkableAccountId> {
		&self.dip_proof
	}

	pub fn signature(&self) -> &TimeBoundDidSignature<ConsumerBlockNumber> {
		&self.signature
	}
}

impl<
		RelayBlockNumber,
		KiltDidKeyId,
		KiltAccountId,
		KiltBlockNumber,
		KiltWeb3Name,
		KiltLinkableAccountId,
		ConsumerBlockNumber,
	>
	ParachainDipDidProof<
		RelayBlockNumber,
		KiltDidKeyId,
		KiltAccountId,
		KiltBlockNumber,
		KiltWeb3Name,
		KiltLinkableAccountId,
		ConsumerBlockNumber,
	>
{
	/// Verifies the head data of the state proof for the provider with the
	/// given para ID and relaychain state root.
	///
	/// The generic types indicate the following:
	/// * `RelayHasher`: The head data hashing algorithm used by the relaychain.
	/// * `ProviderHeader`: The type of the parachain header to be revealed in
	///   the state proof.
	#[allow(clippy::type_complexity)]
	pub fn verify_provider_head_proof_with_state_root<RelayHasher, ProviderHeader>(
		self,
		provider_para_id: u32,
		relay_state_root: &OutputOf<RelayHasher>,
	) -> Result<
		DipDidProofWithVerifiedStateRoot<
			OutputOf<RelayHasher>,
			KiltDidKeyId,
			KiltAccountId,
			KiltBlockNumber,
			KiltWeb3Name,
			KiltLinkableAccountId,
			ConsumerBlockNumber,
		>,
		Error,
	>
	where
		RelayHasher: Hash,
		ProviderHeader: Decode + HeaderT<Hash = OutputOf<RelayHasher>, Number = KiltBlockNumber>,
	{
		let provider_head_storage_key = calculate_parachain_head_storage_key(provider_para_id);
		log::trace!(target: "dip::consumer::ParachainDipDidProofV0", "Calculated storage key for para ID {:#?} = {:#?}", provider_para_id, provider_head_storage_key);
		// TODO: Figure out why RPC call returns 2 bytes in front which we don't need
		//This could be the reason (and the solution): https://substrate.stackexchange.com/a/1891/1795
		let provider_header = verify_storage_value_proof_with_decoder::<_, RelayHasher, ProviderHeader>(
			&provider_head_storage_key,
			*relay_state_root,
			self.provider_head_proof.proof,
			|input| {
				if input.len() < 2 {
					return None;
				}
				let mut trimmed_input = &input[2..];
				ProviderHeader::decode(&mut trimmed_input).ok()
			},
		)
		.map_err(Error::ParaHeadMerkleProof)?;
		Ok(DipDidProofWithVerifiedStateRoot {
			state_root: *provider_header.state_root(),
			dip_commitment_proof: self.dip_commitment_proof,
			dip_proof: self.dip_proof,
			signature: self.signature,
		})
	}

	/// Verifies the head data of the state proof for the provider with the
	/// given para ID using the state root returned by the provided
	/// implementation.
	///
	/// The generic types indicate the following:
	/// * `RelayHasher`: The hashing algorithm used on the relaychain to
	///   generate the parachains head data.
	/// * `StateRootStore`: The type that returns a relaychain state root given
	///   a relaychain block number.
	/// * `ProviderHeader`: The type of the parachain header to be revealed in
	///   the state proof.
	#[allow(clippy::type_complexity)]
	pub fn verify_provider_head_proof<RelayHasher, StateRootStore, ProviderHeader>(
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
			ConsumerBlockNumber,
		>,
		Error,
	>
	where
		RelayHasher: Hash,
		StateRootStore: GetWithArg<RelayBlockNumber, Result = Option<OutputOf<RelayHasher>>>,
		ProviderHeader: Decode + HeaderT<Hash = OutputOf<RelayHasher>, Number = KiltBlockNumber>,
	{
		let relay_state_root =
			StateRootStore::get(&self.provider_head_proof.relay_block_number).ok_or(Error::RelayStateRootNotFound)?;
		self.verify_provider_head_proof_with_state_root::<RelayHasher, ProviderHeader>(
			provider_para_id,
			&relay_state_root,
		)
	}
}

/// A DIP proof that has had the proof header and the relaychain state verified
/// for the provided relaychain block number.
///
/// The generic types indicate the following:
/// * `StateRoot`: The type of the relaychain state root.
/// * `KiltDidKeyId`: The DID key ID type configured by the KILT chain.
/// * `KiltAccountId`: The `AccountId` type configured by the KILT chain.
/// * `KiltBlockNumber`: The `BlockNumber` type configured by the KILT chain.
/// * `KiltWeb3Name`: The web3name type configured by the KILT chain.
/// * `KiltLinkableAccountId`: The linkable account ID type configured by the
///   KILT chain.
/// * `ConsumerBlockNumber`: The `BlockNumber` definition of the consumer
///   parachain.
#[derive(Debug, PartialEq, Eq)]
pub struct DipDidProofWithVerifiedStateRoot<
	StateRoot,
	KiltDidKeyId,
	KiltAccountId,
	KiltBlockNumber,
	KiltWeb3Name,
	KiltLinkableAccountId,
	ConsumerBlockNumber,
> {
	/// The provider state root for the block specified in the DIP proof.
	pub(crate) state_root: StateRoot,
	/// The raw state proof for the DIP commitment of the given subject.
	pub(crate) dip_commitment_proof: DipCommitmentStateProof,
	/// The Merkle proof of the subject's DID details.
	pub(crate) dip_proof:
		DidMerkleProof<KiltDidKeyId, KiltAccountId, KiltBlockNumber, KiltWeb3Name, KiltLinkableAccountId>,
	/// The cross-chain DID signature.
	pub(crate) signature: TimeBoundDidSignature<ConsumerBlockNumber>,
}

impl<
		StateRoot,
		KiltDidKeyId,
		KiltAccountId,
		KiltBlockNumber,
		KiltWeb3Name,
		KiltLinkableAccountId,
		ConsumerBlockNumber,
	>
	DipDidProofWithVerifiedStateRoot<
		StateRoot,
		KiltDidKeyId,
		KiltAccountId,
		KiltBlockNumber,
		KiltWeb3Name,
		KiltLinkableAccountId,
		ConsumerBlockNumber,
	>
{
	/// Verifies the DIP commitment part of the state proof for the subject with
	/// the given identifier.
	///
	/// The generic types indicate the following:
	/// * `ParachainHasher`: The hashing algorithm used to hash storage on the
	///   parachain.
	/// * `ProviderRuntime`: The provider runtime definition.
	#[allow(clippy::type_complexity)]
	pub fn verify_dip_commitment_proof_for_subject<ParachainHasher, ProviderRuntime>(
		self,
		subject: &ProviderRuntime::Identifier,
	) -> Result<
		DipDidProofWithVerifiedSubjectCommitment<
			IdentityCommitmentOf<ProviderRuntime>,
			KiltDidKeyId,
			KiltAccountId,
			KiltBlockNumber,
			KiltWeb3Name,
			KiltLinkableAccountId,
			ConsumerBlockNumber,
		>,
		Error,
	>
	where
		StateRoot: Ord,
		ParachainHasher: Hash<Output = StateRoot>,
		ProviderRuntime: pallet_dip_provider::Config,
	{
		let dip_commitment_storage_key =
			calculate_dip_identity_commitment_storage_key_for_runtime::<ProviderRuntime>(subject, 0);
		log::trace!(target: "dip::consumer::DipDidProofWithVerifiedStateRootV0", "Calculated storage key for subject {:#?} = {:#?}", subject, dip_commitment_storage_key);
		let dip_commitment = verify_storage_value_proof::<_, ParachainHasher, IdentityCommitmentOf<ProviderRuntime>>(
			&dip_commitment_storage_key,
			self.state_root,
			self.dip_commitment_proof.0,
		)
		.map_err(Error::DipCommitmentMerkleProof)?;
		Ok(DipDidProofWithVerifiedSubjectCommitment {
			dip_commitment,
			dip_proof: self.dip_proof,
			signature: self.signature,
		})
	}
}

/// A DIP proof that has had the relaychain state and the DIP commitment
/// verified for the provided relaychain block number.
///
/// The generic types indicate the following:
/// * `Commitment`: The DIP identity commitment type configured by the KILT
///   chain.
/// * `KiltDidKeyId`: The DID key ID type configured by the KILT chain.
/// * `KiltAccountId`: The `AccountId` type configured by the KILT chain.
/// * `KiltBlockNumber`: The `BlockNumber` type configured by the KILT chain.
/// * `KiltWeb3Name`: The web3name type configured by the KILT chain.
/// * `KiltLinkableAccountId`: The linkable account ID type configured by the
///   KILT chain.
/// * `ConsumerBlockNumber`: The `BlockNumber` definition of the consumer
///   parachain.
#[derive(Debug, PartialEq, Eq)]
pub struct DipDidProofWithVerifiedSubjectCommitment<
	Commitment,
	KiltDidKeyId,
	KiltAccountId,
	KiltBlockNumber,
	KiltWeb3Name,
	KiltLinkableAccountId,
	ConsumerBlockNumber,
> {
	/// The verified DIP identity commitment.
	pub(crate) dip_commitment: Commitment,
	/// The Merkle proof of the subject's DID details.
	pub(crate) dip_proof:
		DidMerkleProof<KiltDidKeyId, KiltAccountId, KiltBlockNumber, KiltWeb3Name, KiltLinkableAccountId>,
	/// The cross-chain DID signature.
	pub(crate) signature: TimeBoundDidSignature<ConsumerBlockNumber>,
}

impl<
		Commitment,
		KiltDidKeyId,
		KiltAccountId,
		KiltBlockNumber,
		KiltWeb3Name,
		KiltLinkableAccountId,
		ConsumerBlockNumber,
	>
	DipDidProofWithVerifiedSubjectCommitment<
		Commitment,
		KiltDidKeyId,
		KiltAccountId,
		KiltBlockNumber,
		KiltWeb3Name,
		KiltLinkableAccountId,
		ConsumerBlockNumber,
	>
{
	pub fn new(
		dip_commitment: Commitment,
		dip_proof: DidMerkleProof<KiltDidKeyId, KiltAccountId, KiltBlockNumber, KiltWeb3Name, KiltLinkableAccountId>,
		signature: TimeBoundDidSignature<ConsumerBlockNumber>,
	) -> Self {
		Self {
			dip_commitment,
			dip_proof,
			signature,
		}
	}
}

impl<
		Commitment,
		KiltDidKeyId,
		KiltAccountId,
		KiltBlockNumber,
		KiltWeb3Name,
		KiltLinkableAccountId,
		ConsumerBlockNumber,
	>
	DipDidProofWithVerifiedSubjectCommitment<
		Commitment,
		KiltDidKeyId,
		KiltAccountId,
		KiltBlockNumber,
		KiltWeb3Name,
		KiltLinkableAccountId,
		ConsumerBlockNumber,
	> where
	KiltDidKeyId: Encode,
	KiltAccountId: Encode,
	KiltBlockNumber: Encode,
	KiltWeb3Name: Encode,
	KiltLinkableAccountId: Encode,
{
	/// Verifies the Merkle proof of the subject's DID details.
	///
	/// The generic types indicate the following:
	/// * `DidMerkleHasher`: The hashing algorithm used to merkleize the DID
	///   details.
	/// * `MAX_REVEALED_LEAVES_COUNT`: The maximum number of leaves revealable
	///   in the proof.
	pub fn verify_dip_proof<DidMerkleHasher, const MAX_REVEALED_LEAVES_COUNT: u32>(
		self,
	) -> Result<
		DipRevealedDetailsAndUnverifiedDidSignature<
			KiltDidKeyId,
			KiltAccountId,
			KiltBlockNumber,
			KiltWeb3Name,
			KiltLinkableAccountId,
			ConsumerBlockNumber,
			MAX_REVEALED_LEAVES_COUNT,
		>,
		Error,
	>
	where
		DidMerkleHasher: Hash<Output = Commitment>,
	{
		ensure!(
			self.dip_proof.revealed.len() <= MAX_REVEALED_LEAVES_COUNT.saturated_into(),
			Error::TooManyLeavesRevealed
		);

		let proof_leaves_key_value_pairs = self
			.dip_proof
			.revealed
			.iter()
			.map(|revealed_leaf| (revealed_leaf.encoded_key(), Some(revealed_leaf.encoded_value())))
			.collect::<Vec<_>>();
		verify_trie_proof::<LayoutV1<DidMerkleHasher>, _, _, _>(
			&self.dip_commitment,
			self.dip_proof.blinded.as_slice(),
			proof_leaves_key_value_pairs.as_slice(),
		)
		// Can't log since the result returned by `verify_trie_proof` implements `Debug` only with `std`.
		.map_err(|_| Error::InvalidDidMerkleProof)?;

		let revealed_leaves = BoundedVec::try_from(self.dip_proof.revealed).map_err(|_| {
			log::error!(target: "dip::consumer::DipDidProofWithVerifiedSubjectCommitmentV0", "Failed to construct BoundedVec<u8, {MAX_REVEALED_LEAVES_COUNT}>.");
			Error::Internal
		})?;

		Ok(DipRevealedDetailsAndUnverifiedDidSignature {
			revealed_leaves,
			signature: self.signature,
		})
	}
}
