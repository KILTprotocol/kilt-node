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

// TODO: Crate documentation

#![cfg_attr(not(feature = "std"), no_std)]

use parity_scale_codec::{Decode, Encode, HasCompact};
use scale_info::TypeInfo;
use sp_core::{Get, RuntimeDebug, U256};
use sp_runtime::traits::CheckedSub;
use sp_std::{borrow::Borrow, marker::PhantomData, vec::Vec};

use ::did::{did_details::DidVerificationKey, DidVerificationKeyRelationship};
use pallet_dip_consumer::traits::IdentityProofVerifier;

use crate::{
	did::{RevealedDidKeysAndSignature, RevealedDidKeysSignatureAndCallVerifier, TimeBoundDidSignature},
	merkle::{DidMerkleProof, DidMerkleProofVerifier, RevealedDidMerkleProofLeaf, RevealedDidMerkleProofLeaves},
	state_proofs::{parachain::DipIdentityCommitmentProofVerifier, relay_chain::SiblingParachainHeadProofVerifier},
	traits::{Bump, DidSignatureVerifierContext, DipCallOriginFilter},
	utils::OutputOf,
};

pub mod did;
pub mod merkle;
pub mod state_proofs;
pub mod traits;
pub mod utils;

pub use state_proofs::{
	parachain::KiltDipCommitmentsForDipProviderPallet, relay_chain::RococoStateRootsViaRelayStorePallet,
};

#[derive(Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo, Clone)]
pub struct SiblingParachainDipStateProof<
	RelayBlockHeight,
	DipMerkleProofBlindedValues,
	DipMerkleProofRevealedLeaf,
	DipProviderBlockNumber,
> {
	para_state_root: SiblingParachainRootStateProof<RelayBlockHeight>,
	dip_identity_commitment: Vec<Vec<u8>>,
	did: DipMerkleProofAndDidSignature<DipMerkleProofBlindedValues, DipMerkleProofRevealedLeaf, DipProviderBlockNumber>,
}

#[derive(Encode, Decode, PartialEq, Eq, PartialOrd, Ord, RuntimeDebug, TypeInfo, Clone)]
pub struct SiblingParachainRootStateProof<RelayBlockHeight> {
	relay_block_height: RelayBlockHeight,
	proof: Vec<Vec<u8>>,
}

#[derive(Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo, Clone)]
pub struct DipMerkleProofAndDidSignature<BlindedValues, Leaf, BlockNumber> {
	leaves: DidMerkleProof<BlindedValues, Leaf>,
	signature: TimeBoundDidSignature<BlockNumber>,
}

pub struct DipSiblingProviderStateProofVerifier<
	RelayChainStateInfo,
	SiblingProviderParachainId,
	SiblingProviderStateInfo,
	TxSubmitter,
	ProviderDipMerkleHasher,
	ProviderDidKeyId,
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
		RelayChainStateInfo,
		SiblingProviderParachainId,
		SiblingProviderStateInfo,
		TxSubmitter,
		ProviderDipMerkleHasher,
		ProviderDidKeyId,
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
		RelayChainStateInfo,
		SiblingProviderParachainId,
		SiblingProviderStateInfo,
		TxSubmitter,
		ProviderDipMerkleHasher,
		ProviderDidKeyId,
		ProviderWeb3Name,
		ProviderLinkedAccountId,
		const MAX_REVEALED_KEYS_COUNT: u32,
		const MAX_REVEALED_ACCOUNTS_COUNT: u32,
		LocalDidDetails,
		LocalContextProvider,
		LocalDidCallVerifier,
	> IdentityProofVerifier<Call, Subject>
	for DipSiblingProviderStateProofVerifier<
		RelayChainStateInfo,
		SiblingProviderParachainId,
		SiblingProviderStateInfo,
		TxSubmitter,
		ProviderDipMerkleHasher,
		ProviderDidKeyId,
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

	RelayChainStateInfo: traits::RelayChainStateInfo,
	RelayChainStateInfo::Hasher: 'static,
	OutputOf<RelayChainStateInfo::Hasher>: Ord,
	RelayChainStateInfo::BlockNumber: Copy + Into<U256> + TryFrom<U256> + HasCompact,
	RelayChainStateInfo::Key: AsRef<[u8]>,

	SiblingProviderParachainId: Get<RelayChainStateInfo::ParaId>,

	SiblingProviderStateInfo:
		traits::ProviderParachainStateInfo<Identifier = Subject, Commitment = ProviderDipMerkleHasher::Out>,
	SiblingProviderStateInfo::Hasher: 'static,
	OutputOf<SiblingProviderStateInfo::Hasher>: Ord + From<OutputOf<RelayChainStateInfo::Hasher>>,
	SiblingProviderStateInfo::BlockNumber: Encode + Clone,
	SiblingProviderStateInfo::Commitment: Decode,
	SiblingProviderStateInfo::Key: AsRef<[u8]>,

	LocalContextProvider: DidSignatureVerifierContext,
	LocalContextProvider::BlockNumber: Encode + CheckedSub + From<u16> + PartialOrd,
	LocalContextProvider::Hash: Encode,
	LocalContextProvider::SignedExtra: Encode,
	LocalDidDetails: Bump + Default + Encode,
	LocalDidCallVerifier: DipCallOriginFilter<Call, OriginInfo = (DidVerificationKey, DidVerificationKeyRelationship)>,

	ProviderDipMerkleHasher: sp_core::Hasher,
	ProviderDidKeyId: Encode + Clone + Into<ProviderDipMerkleHasher::Out>,
	ProviderLinkedAccountId: Encode + Clone,
	ProviderWeb3Name: Encode + Clone,
{
	type Error = ();
	type IdentityDetails = LocalDidDetails;
	type Proof = SiblingParachainDipStateProof<
		RelayChainStateInfo::BlockNumber,
		Vec<Vec<u8>>,
		RevealedDidMerkleProofLeaf<
			ProviderDidKeyId,
			SiblingProviderStateInfo::BlockNumber,
			ProviderWeb3Name,
			ProviderLinkedAccountId,
		>,
		LocalContextProvider::BlockNumber,
	>;
	type Submitter = TxSubmitter;
	type VerificationResult = RevealedDidMerkleProofLeaves<
		ProviderDidKeyId,
		SiblingProviderStateInfo::BlockNumber,
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
		// 1. Verify relay chain proof.
		let provider_parachain_header =
			SiblingParachainHeadProofVerifier::<RelayChainStateInfo>::verify_proof_for_parachain(
				&SiblingProviderParachainId::get(),
				&proof.para_state_root.relay_block_height,
				proof.para_state_root.proof,
			)?;

		// 2. Verify parachain state proof.
		let subject_identity_commitment =
			DipIdentityCommitmentProofVerifier::<SiblingProviderStateInfo>::verify_proof_for_identifier(
				subject,
				provider_parachain_header.state_root.into(),
				proof.dip_identity_commitment,
			)?;

		// 3. Verify DIP merkle proof.
		let proof_leaves = DidMerkleProofVerifier::<
			ProviderDipMerkleHasher,
			_,
			_,
			_,
			_,
			MAX_REVEALED_KEYS_COUNT,
			MAX_REVEALED_ACCOUNTS_COUNT,
		>::verify_dip_merkle_proof(&subject_identity_commitment, proof.did.leaves)?;

		// 4. Verify DID signature.
		RevealedDidKeysSignatureAndCallVerifier::<
			_,
			_,
			_,
			_,
			LocalContextProvider,
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
		)?;

		Ok(proof_leaves)
	}
}
