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
use sp_runtime::traits::{CheckedSub, Get};
use sp_std::{marker::PhantomData, vec::Vec};

use crate::{
	did::RevealedDidKeysSignatureAndCallVerifierError,
	merkle::{DidMerkleProofVerifierError, RevealedDidMerkleProofLeaf, RevealedDidMerkleProofLeaves},
	state_proofs::{parachain::DipIdentityCommitmentProofVerifierError, relay_chain::ParachainHeadProofVerifierError},
	traits::{self, Bump, DidSignatureVerifierContext, DipCallOriginFilter},
	utils::OutputOf,
};

#[derive(Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo, Clone)]
#[non_exhaustive]
pub enum VersionedSiblingParachainDipStateProof<
	RelayBlockHeight,
	DipMerkleProofBlindedValues,
	DipMerkleProofRevealedLeaf,
	LocalBlockNumber,
> {
	V0(
		v0::SiblingParachainDipStateProof<
			RelayBlockHeight,
			DipMerkleProofBlindedValues,
			DipMerkleProofRevealedLeaf,
			LocalBlockNumber,
		>,
	),
}

pub enum DipSiblingProviderStateProofVerifierError<
	ParachainHeadMerkleProofVerificationError,
	IdentityCommitmentMerkleProofVerificationError,
	DipProofVerificationError,
	DidSignatureVerificationError,
> {
	UnsupportedVersion,
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
		DipSiblingProviderStateProofVerifierError<
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
		value: DipSiblingProviderStateProofVerifierError<
			ParachainHeadMerkleProofVerificationError,
			IdentityCommitmentMerkleProofVerificationError,
			DipProofVerificationError,
			DidSignatureVerificationError,
		>,
	) -> Self {
		match value {
			DipSiblingProviderStateProofVerifierError::UnsupportedVersion => 0,
			DipSiblingProviderStateProofVerifierError::ParachainHeadMerkleProof(error) => {
				u8::MAX as u16 + error.into() as u16
			}
			DipSiblingProviderStateProofVerifierError::IdentityCommitmentMerkleProof(error) => {
				u8::MAX as u16 * 2 + error.into() as u16
			}
			DipSiblingProviderStateProofVerifierError::DipProof(error) => u8::MAX as u16 * 3 + error.into() as u16,
			DipSiblingProviderStateProofVerifierError::DidSignature(error) => u8::MAX as u16 * 4 + error.into() as u16,
		}
	}
}

pub struct VersionedDipSiblingProviderStateProofVerifier<
	RelayChainStateInfo,
	SiblingProviderParachainId,
	SiblingProviderStateInfo,
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
		RelayChainStateInfo,
		SiblingProviderParachainId,
		SiblingProviderStateInfo,
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
		Runtime,
		RelayChainStateInfo,
		SiblingProviderParachainId,
		SiblingProviderStateInfo,
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
	> IdentityProofVerifier<Runtime>
	for VersionedDipSiblingProviderStateProofVerifier<
		RelayChainStateInfo,
		SiblingProviderParachainId,
		SiblingProviderStateInfo,
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
	Runtime: pallet_dip_consumer::Config,
	<Runtime as pallet_dip_consumer::Config>::RuntimeCall: Encode,
	TxSubmitter: Encode,

	RelayChainStateInfo: traits::RelayChainStorageInfo + traits::RelayChainStateInfo,
	OutputOf<RelayChainStateInfo::Hasher>: Ord,
	RelayChainStateInfo::BlockNumber: Copy + Into<U256> + TryFrom<U256> + HasCompact,
	RelayChainStateInfo::Key: AsRef<[u8]>,

	SiblingProviderParachainId: Get<RelayChainStateInfo::ParaId>,

	SiblingProviderStateInfo:
		traits::ProviderParachainStateInfo<Identifier = Runtime::Identifier, Commitment = ProviderDipMerkleHasher::Out>,
	OutputOf<SiblingProviderStateInfo::Hasher>: Ord + From<OutputOf<RelayChainStateInfo::Hasher>>,
	SiblingProviderStateInfo::BlockNumber: Encode + Clone,
	SiblingProviderStateInfo::Commitment: Decode,
	SiblingProviderStateInfo::Key: AsRef<[u8]>,

	LocalContextProvider: DidSignatureVerifierContext,
	LocalContextProvider::BlockNumber: Encode + CheckedSub + From<u16> + PartialOrd,
	LocalContextProvider::Hash: Encode,
	LocalContextProvider::SignedExtra: Encode,
	LocalDidDetails: Bump + Default + Encode,
	LocalDidCallVerifier: DipCallOriginFilter<
		<Runtime as pallet_dip_consumer::Config>::RuntimeCall,
		OriginInfo = (DidVerificationKey<ProviderAccountId>, DidVerificationKeyRelationship),
	>,

	ProviderDipMerkleHasher: sp_core::Hasher,
	ProviderDidKeyId: Encode + Clone + Into<ProviderDipMerkleHasher::Out>,
	ProviderAccountId: Encode + Clone,
	ProviderLinkedAccountId: Encode + Clone,
	ProviderWeb3Name: Encode + Clone,
{
	type Error = DipSiblingProviderStateProofVerifierError<
		ParachainHeadProofVerifierError,
		DipIdentityCommitmentProofVerifierError,
		DidMerkleProofVerifierError,
		RevealedDidKeysSignatureAndCallVerifierError,
	>;
	type Proof = VersionedSiblingParachainDipStateProof<
		RelayChainStateInfo::BlockNumber,
		Vec<Vec<u8>>,
		RevealedDidMerkleProofLeaf<
			ProviderDidKeyId,
			ProviderAccountId,
			SiblingProviderStateInfo::BlockNumber,
			ProviderWeb3Name,
			ProviderLinkedAccountId,
		>,
		LocalContextProvider::BlockNumber,
	>;
	type VerificationResult = RevealedDidMerkleProofLeaves<
		ProviderDidKeyId,
		ProviderAccountId,
		SiblingProviderStateInfo::BlockNumber,
		ProviderWeb3Name,
		ProviderLinkedAccountId,
		MAX_REVEALED_KEYS_COUNT,
		MAX_REVEALED_ACCOUNTS_COUNT,
	>;

	fn verify_proof_for_call_against_details(
		call: &<Runtime as pallet_dip_consumer::Config>::RuntimeCall,
		subject: &Runtime::Identifier,
		submitter: &Runtime::AccountId,
		identity_details: &mut Option<Runtime::LocalIdentityInfo>,
		proof: Self::Proof,
	) -> Result<Self::VerificationResult, Self::Error> {
		match proof {
			VersionedSiblingParachainDipStateProof::V0(v0_proof) => {
				v0::DipSiblingProviderStateProofVerifier::<
					RelayChainStateInfo,
					SiblingProviderParachainId,
					SiblingProviderStateInfo,
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
				>::verify_proof_for_call_against_details(call, subject, submitter, identity_details, v0_proof)
			}
		}
	}
}

pub mod latest {
	pub use super::v0::SiblingParachainDipStateProof;
}

mod v0 {

	use super::*;

	use frame_support::Parameter;
	use sp_std::borrow::Borrow;

	use crate::{
		did::{RevealedDidKeysAndSignature, RevealedDidKeysSignatureAndCallVerifier},
		export::common::v0::{DipMerkleProofAndDidSignature, ParachainRootStateProof},
		merkle::DidMerkleProofVerifier,
		state_proofs::{parachain::DipIdentityCommitmentProofVerifier, relay_chain::ParachainHeadProofVerifier},
		traits::ProviderParachainStateInfo,
	};

	#[derive(Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo, Clone)]
	pub struct SiblingParachainDipStateProof<
		RelayBlockHeight,
		DipMerkleProofBlindedValues,
		DipMerkleProofRevealedLeaf,
		LocalBlockNumber,
	> {
		para_state_root: ParachainRootStateProof<RelayBlockHeight>,
		dip_identity_commitment: Vec<Vec<u8>>,
		did: DipMerkleProofAndDidSignature<DipMerkleProofBlindedValues, DipMerkleProofRevealedLeaf, LocalBlockNumber>,
	}

	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
	pub struct DipSiblingProviderStateProofVerifier<
		RelayChainStateInfo,
		SiblingProviderParachainId,
		SiblingProviderStateInfo,
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
			RelayChainStateInfo,
			SiblingProviderParachainId,
			SiblingProviderStateInfo,
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
			Runtime,
			RelayChainStateInfo,
			SiblingProviderParachainId,
			SiblingProviderStateInfo,
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
		> IdentityProofVerifier<Runtime>
		for DipSiblingProviderStateProofVerifier<
			RelayChainStateInfo,
			SiblingProviderParachainId,
			SiblingProviderStateInfo,
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
		Runtime: pallet_dip_consumer::Config,
		<Runtime as pallet_dip_consumer::Config>::RuntimeCall: Encode,
		Runtime::LocalIdentityInfo: Default,
		TxSubmitter: Encode,

		RelayChainStateInfo: traits::RelayChainStorageInfo + traits::RelayChainStateInfo,
		OutputOf<RelayChainStateInfo::Hasher>: Ord,
		RelayChainStateInfo::BlockNumber: Copy + Into<U256> + TryFrom<U256> + HasCompact + Parameter,
		RelayChainStateInfo::Key: AsRef<[u8]>,

		SiblingProviderParachainId: Get<RelayChainStateInfo::ParaId>,

		SiblingProviderStateInfo: traits::ProviderParachainStateInfo<
			Identifier = Runtime::Identifier,
			Commitment = ProviderDipMerkleHasher::Out,
		>,
		OutputOf<SiblingProviderStateInfo::Hasher>: Ord + From<OutputOf<RelayChainStateInfo::Hasher>>,
		SiblingProviderStateInfo::BlockNumber: Parameter,
		SiblingProviderStateInfo::Commitment: Decode,
		SiblingProviderStateInfo::Key: AsRef<[u8]>,

		LocalContextProvider: DidSignatureVerifierContext,
		LocalContextProvider::BlockNumber: Parameter + CheckedSub + From<u16> + PartialOrd,
		LocalContextProvider::Hash: Encode,
		LocalContextProvider::SignedExtra: Encode,
		LocalDidDetails: Bump + Default + Encode,
		LocalDidCallVerifier: DipCallOriginFilter<
			<Runtime as pallet_dip_consumer::Config>::RuntimeCall,
			OriginInfo = (DidVerificationKey<ProviderAccountId>, DidVerificationKeyRelationship),
		>,

		ProviderDipMerkleHasher: sp_core::Hasher,
		ProviderDidKeyId: Parameter + Into<ProviderDipMerkleHasher::Out>,
		ProviderAccountId: Parameter,
		ProviderLinkedAccountId: Parameter,
		ProviderWeb3Name: Parameter,
	{
		type Error = DipSiblingProviderStateProofVerifierError<
			ParachainHeadProofVerifierError,
			DipIdentityCommitmentProofVerifierError,
			DidMerkleProofVerifierError,
			RevealedDidKeysSignatureAndCallVerifierError,
		>;
		type Proof = SiblingParachainDipStateProof<
			RelayChainStateInfo::BlockNumber,
			Vec<Vec<u8>>,
			RevealedDidMerkleProofLeaf<
				ProviderDidKeyId,
				ProviderAccountId,
				SiblingProviderStateInfo::BlockNumber,
				ProviderWeb3Name,
				ProviderLinkedAccountId,
			>,
			LocalContextProvider::BlockNumber,
		>;
		type VerificationResult = RevealedDidMerkleProofLeaves<
			ProviderDidKeyId,
			ProviderAccountId,
			SiblingProviderStateInfo::BlockNumber,
			ProviderWeb3Name,
			ProviderLinkedAccountId,
			MAX_REVEALED_KEYS_COUNT,
			MAX_REVEALED_ACCOUNTS_COUNT,
		>;

		fn verify_proof_for_call_against_details(
			call: &<Runtime as pallet_dip_consumer::Config>::RuntimeCall,
			subject: &Runtime::Identifier,
			submitter: &Runtime::AccountId,
			identity_details: &mut Option<Runtime::LocalIdentityInfo>,
			proof: Self::Proof,
		) -> Result<Self::VerificationResult, Self::Error> {
			// 1. Verify relay chain proof.
			let provider_parachain_header =
				ParachainHeadProofVerifier::<RelayChainStateInfo>::verify_proof_for_parachain(
					&SiblingProviderParachainId::get(),
					&proof.para_state_root.relay_block_height,
					proof.para_state_root.proof,
				)
				.map_err(DipSiblingProviderStateProofVerifierError::ParachainHeadMerkleProof)?;

			// 2. Verify parachain state proof.
			let subject_identity_commitment =
				DipIdentityCommitmentProofVerifier::<SiblingProviderStateInfo>::verify_proof_for_identifier(
					subject,
					provider_parachain_header.state_root.into(),
					proof.dip_identity_commitment,
				)
				.map_err(DipSiblingProviderStateProofVerifierError::IdentityCommitmentMerkleProof)?;

			// 3. Verify DIP merkle proof.
			let proof_leaves: RevealedDidMerkleProofLeaves<
				ProviderDidKeyId,
				ProviderAccountId,
				<SiblingProviderStateInfo as ProviderParachainStateInfo>::BlockNumber,
				ProviderWeb3Name,
				ProviderLinkedAccountId,
				MAX_REVEALED_KEYS_COUNT,
				MAX_REVEALED_ACCOUNTS_COUNT,
			> = DidMerkleProofVerifier::<
				ProviderDipMerkleHasher,
				_,
				_,
				_,
				_,
				_,
				MAX_REVEALED_KEYS_COUNT,
				MAX_REVEALED_ACCOUNTS_COUNT,
			>::verify_dip_merkle_proof(&subject_identity_commitment, proof.did.leaves)
			.map_err(DipSiblingProviderStateProofVerifierError::DipProof)?;

			// 4. Verify DID signature.
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
				.map_err(DipSiblingProviderStateProofVerifierError::DidSignature)?;

			Ok(proof_leaves)
		}
	}
}
