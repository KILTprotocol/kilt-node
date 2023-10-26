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

use did::{did_details::DidVerificationKey, DidVerificationKeyRelationship};
use pallet_dip_consumer::traits::IdentityProofVerifier;
use parity_scale_codec::{Decode, Encode, HasCompact};
use scale_info::TypeInfo;
use sp_core::{RuntimeDebug, U256};
use sp_runtime::traits::{CheckedSub, Get, Hash};
use sp_std::marker::PhantomData;

#[derive(Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo, Clone)]
pub enum VersionedChildParachainDipStateProof<
	ParentBlockHeight: Copy + Into<U256> + TryFrom<U256>,
	ParentBlockHasher: Hash,
	DipMerkleProofBlindedValues,
	DipMerkleProofRevealedLeaf,
> {
	V1(
		v1::ChildParachainDipStateProof<
			ParentBlockHeight,
			ParentBlockHasher,
			DipMerkleProofBlindedValues,
			DipMerkleProofRevealedLeaf,
		>,
	),
}

pub enum DipChildProviderStateProofVerifierError<
	ParachainHeadMerkleProofVerificationError,
	IdentityCommitmentMerkleProofVerificationError,
	DipProofVerificationError,
	DidSignatureVerificationError,
> {
	InvalidBlockHeight,
	InvalidBlockHash,
	ParachainHeadMerkleProof(ParachainHeadMerkleProofVerificationError),
	IdentityCommitmentMerkleProof(IdentityCommitmentMerkleProofVerificationError),
	DipProof(DipProofVerificationError),
	DidSignature(DidSignatureVerificationError),
}

impl<
		ParachainHeadMerkleProofVerificationError,
		IdentityCommitmentMerkleProofVerificationError,
		DipProofVerificationError,
		DidSignatureVerificationError,
	>
	From<
		DipChildProviderStateProofVerifierError<
			ParachainHeadMerkleProofVerificationError,
			IdentityCommitmentMerkleProofVerificationError,
			DipProofVerificationError,
			DidSignatureVerificationError,
		>,
	> for u16
where
	ParachainHeadMerkleProofVerificationError: Into<u8>,
	IdentityCommitmentMerkleProofVerificationError: Into<u8>,
	DipProofVerificationError: Into<u8>,
	DidSignatureVerificationError: Into<u8>,
{
	fn from(
		value: DipChildProviderStateProofVerifierError<
			ParachainHeadMerkleProofVerificationError,
			IdentityCommitmentMerkleProofVerificationError,
			DipProofVerificationError,
			DidSignatureVerificationError,
		>,
	) -> Self {
		match value {
			DipChildProviderStateProofVerifierError::InvalidBlockHeight => 0,
			DipChildProviderStateProofVerifierError::InvalidBlockHash => 1,
			DipChildProviderStateProofVerifierError::ParachainHeadMerkleProof(error) => {
				u8::MAX as u16 + error.into() as u16
			}
			DipChildProviderStateProofVerifierError::IdentityCommitmentMerkleProof(error) => {
				u8::MAX as u16 * 2 + error.into() as u16
			}
			DipChildProviderStateProofVerifierError::DipProof(error) => u8::MAX as u16 * 3 + error.into() as u16,
			DipChildProviderStateProofVerifierError::DidSignature(error) => u8::MAX as u16 * 4 + error.into() as u16,
		}
	}
}

mod v1 {
	use super::*;

	use parity_scale_codec::Codec;
	use sp_runtime::{
		generic::Header,
		traits::{AtLeast32BitUnsigned, Hash, MaybeDisplay, Member, SimpleBitOps},
	};
	use sp_std::borrow::Borrow;

	use crate::{
		did::{
			RevealedDidKeysAndSignature, RevealedDidKeysSignatureAndCallVerifier,
			RevealedDidKeysSignatureAndCallVerifierError,
		},
		export::common::v1::{DipMerkleProofAndDidSignature, ParachainRootStateProof},
		merkle::{
			DidMerkleProofVerifier, DidMerkleProofVerifierError, RevealedDidMerkleProofLeaf,
			RevealedDidMerkleProofLeaves,
		},
		state_proofs::{
			parachain::{DipIdentityCommitmentProofVerifier, DipIdentityCommitmentProofVerifierError},
			relay_chain::{ParachainHeadProofVerifier, ParachainHeadProofVerifierError},
		},
		traits::{
			Bump, DidSignatureVerifierContext, DipCallOriginFilter, HistoricalBlockRegistry,
			ProviderParachainStateInfo, RelayChainStorageInfo,
		},
		utils::OutputOf,
	};

	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
	pub struct ChildParachainDipStateProof<
		ParentBlockHeight: Copy + Into<U256> + TryFrom<U256>,
		ParentBlockHasher: Hash,
		DipMerkleProofBlindedValues,
		DipMerkleProofRevealedLeaf,
	> {
		para_state_root: ParachainRootStateProof<ParentBlockHeight>,
		relay_header: Header<ParentBlockHeight, ParentBlockHasher>,
		dip_identity_commitment: Vec<Vec<u8>>,
		did: DipMerkleProofAndDidSignature<DipMerkleProofBlindedValues, DipMerkleProofRevealedLeaf, ParentBlockHeight>,
	}

	pub struct DipChildProviderStateProofVerifier<
		RelayChainInfo,
		ChildProviderParachainId,
		ChildProviderStateInfo,
		TxSubmitter,
		ProviderDipMerkleHasher,
		ProviderDidKeyId,
		ProviderAccountId,
		ProviderWeb3Name,
		ProviderLinkedAccountId,
		const MAX_REVEALED_KEYS_COUNT: u32,
		const MAX_REVEALED_ACCOUNTS_COUNT: u32,
		LocalDidDetails,
		LocalContextProvider,
		LocalDidCallVerifier,
	>(
		#[allow(clippy::type_complexity)]
		PhantomData<(
			RelayChainInfo,
			ChildProviderParachainId,
			ChildProviderStateInfo,
			TxSubmitter,
			ProviderDipMerkleHasher,
			ProviderDidKeyId,
			ProviderAccountId,
			ProviderWeb3Name,
			ProviderLinkedAccountId,
			LocalDidDetails,
			LocalContextProvider,
			LocalDidCallVerifier,
		)>,
	);

	impl<
			Call,
			Subject,
			RelayChainInfo,
			ChildProviderParachainId,
			ChildProviderStateInfo,
			TxSubmitter,
			ProviderDipMerkleHasher,
			ProviderDidKeyId,
			ProviderAccountId,
			ProviderWeb3Name,
			ProviderLinkedAccountId,
			const MAX_REVEALED_KEYS_COUNT: u32,
			const MAX_REVEALED_ACCOUNTS_COUNT: u32,
			LocalDidDetails,
			LocalContextProvider,
			LocalDidCallVerifier,
		> IdentityProofVerifier<Call, Subject>
		for DipChildProviderStateProofVerifier<
			RelayChainInfo,
			ChildProviderParachainId,
			ChildProviderStateInfo,
			TxSubmitter,
			ProviderDipMerkleHasher,
			ProviderDidKeyId,
			ProviderAccountId,
			ProviderWeb3Name,
			ProviderLinkedAccountId,
			MAX_REVEALED_KEYS_COUNT,
			MAX_REVEALED_ACCOUNTS_COUNT,
			LocalDidDetails,
			LocalContextProvider,
			LocalDidCallVerifier,
		> where
		Call: Encode,
		TxSubmitter: Encode,

		RelayChainInfo: RelayChainStorageInfo
			+ HistoricalBlockRegistry<
				BlockNumber = <RelayChainInfo as RelayChainStorageInfo>::BlockNumber,
				Hasher = <RelayChainInfo as RelayChainStorageInfo>::Hasher,
			>,
		OutputOf<<RelayChainInfo as RelayChainStorageInfo>::Hasher>:
			Ord + Default + sp_std::hash::Hash + Copy + Member + MaybeDisplay + SimpleBitOps + Codec,
		<RelayChainInfo as RelayChainStorageInfo>::BlockNumber: Copy
			+ Into<U256>
			+ TryFrom<U256>
			+ HasCompact
			+ Member
			+ sp_std::hash::Hash
			+ MaybeDisplay
			+ AtLeast32BitUnsigned
			+ Codec,
		RelayChainInfo::Key: AsRef<[u8]>,

		ChildProviderParachainId: Get<RelayChainInfo::ParaId>,

		ChildProviderStateInfo:
			ProviderParachainStateInfo<Identifier = Subject, Commitment = ProviderDipMerkleHasher::Out>,
		OutputOf<ChildProviderStateInfo::Hasher>:
			Ord + From<OutputOf<<RelayChainInfo as RelayChainStorageInfo>::Hasher>>,
		ChildProviderStateInfo::BlockNumber: Encode + Clone,
		ChildProviderStateInfo::Commitment: Decode,
		ChildProviderStateInfo::Key: AsRef<[u8]>,

		LocalContextProvider:
			DidSignatureVerifierContext<BlockNumber = <RelayChainInfo as RelayChainStorageInfo>::BlockNumber>,
		LocalContextProvider::BlockNumber: CheckedSub + From<u16>,
		LocalContextProvider::Hash: Encode,
		LocalContextProvider::SignedExtra: Encode,
		LocalDidDetails: Bump + Default + Encode,
		LocalDidCallVerifier: DipCallOriginFilter<
			Call,
			OriginInfo = (DidVerificationKey<ProviderAccountId>, DidVerificationKeyRelationship),
		>,

		ProviderDipMerkleHasher: sp_core::Hasher,
		ProviderDidKeyId: Encode + Clone + Into<ProviderDipMerkleHasher::Out>,
		ProviderAccountId: Encode + Clone,
		ProviderLinkedAccountId: Encode + Clone,
		ProviderWeb3Name: Encode + Clone,
	{
		type Error = DipChildProviderStateProofVerifierError<
			ParachainHeadProofVerifierError,
			DipIdentityCommitmentProofVerifierError,
			DidMerkleProofVerifierError,
			RevealedDidKeysSignatureAndCallVerifierError,
		>;
		type IdentityDetails = LocalDidDetails;
		type Proof = ChildParachainDipStateProof<
			<RelayChainInfo as RelayChainStorageInfo>::BlockNumber,
			<RelayChainInfo as RelayChainStorageInfo>::Hasher,
			Vec<Vec<u8>>,
			RevealedDidMerkleProofLeaf<
				ProviderDidKeyId,
				ProviderAccountId,
				ChildProviderStateInfo::BlockNumber,
				ProviderWeb3Name,
				ProviderLinkedAccountId,
			>,
		>;
		type Submitter = TxSubmitter;
		type VerificationResult = RevealedDidMerkleProofLeaves<
			ProviderDidKeyId,
			ProviderAccountId,
			ChildProviderStateInfo::BlockNumber,
			ProviderWeb3Name,
			ProviderLinkedAccountId,
			MAX_REVEALED_KEYS_COUNT,
			MAX_REVEALED_ACCOUNTS_COUNT,
		>;

		fn verify_proof_for_call_against_details(
			call: &Call,
			subject: &Subject,
			submitter: &Self::Submitter,
			identity_details: &mut Option<Self::IdentityDetails>,
			proof: Self::Proof,
		) -> Result<Self::VerificationResult, Self::Error> {
			// 1. Retrieve block hash from provider at the proof height
			let block_hash_at_height = RelayChainInfo::block_hash_for(&proof.para_state_root.relay_block_height)
				.ok_or(DipChildProviderStateProofVerifierError::InvalidBlockHeight)?;

			// 1.1 Verify that the provided header hashes to the same block has retrieved
			if block_hash_at_height != proof.relay_header.hash() {
				return Err(DipChildProviderStateProofVerifierError::InvalidBlockHash);
			}
			// 1.2 If so, extract the state root from the header
			let state_root_at_height = proof.relay_header.state_root;

			// FIXME: Compilation error
			// 2. Verify relay chain proof
			let provider_parachain_header =
				ParachainHeadProofVerifier::<RelayChainInfo>::verify_proof_for_parachain_with_root(
					&ChildProviderParachainId::get(),
					&state_root_at_height,
					proof.para_state_root.proof,
				)
				.map_err(DipChildProviderStateProofVerifierError::ParachainHeadMerkleProof)?;

			// 3. Verify parachain state proof.
			let subject_identity_commitment =
				DipIdentityCommitmentProofVerifier::<ChildProviderStateInfo>::verify_proof_for_identifier(
					subject,
					provider_parachain_header.state_root.into(),
					proof.dip_identity_commitment,
				)
				.map_err(DipChildProviderStateProofVerifierError::IdentityCommitmentMerkleProof)?;

			// 4. Verify DIP merkle proof.
			let proof_leaves = DidMerkleProofVerifier::<
				ProviderDipMerkleHasher,
				_,
				_,
				_,
				_,
				_,
				MAX_REVEALED_KEYS_COUNT,
				MAX_REVEALED_ACCOUNTS_COUNT,
			>::verify_dip_merkle_proof(&subject_identity_commitment, proof.did.leaves)
			.map_err(DipChildProviderStateProofVerifierError::DipProof)?;

			// 5. Verify DID signature.
			RevealedDidKeysSignatureAndCallVerifier::<
					_,
					_,
					_,
					_,
					LocalContextProvider,
					_,
					_,
					_,
					LocalDidCallVerifier,
				>::verify_did_signature_for_call(
					call,
					submitter,
					identity_details,
					RevealedDidKeysAndSignature {
						merkle_leaves: proof_leaves.borrow(),
						did_signature: proof.did.signature,
					},
				)
				.map_err(DipChildProviderStateProofVerifierError::DidSignature)?;
			Ok(proof_leaves)
		}
	}
}
