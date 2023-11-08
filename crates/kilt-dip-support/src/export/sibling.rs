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

use cumulus_primitives_core::ParaId;
use did::{did_details::DidVerificationKey, DidVerificationKeyRelationship, KeyIdOf};
use frame_support::Parameter;
use frame_system::pallet_prelude::BlockNumberFor;
use pallet_did_lookup::linkable_account::LinkableAccountId;
use pallet_dip_consumer::{traits::IdentityProofVerifier, RuntimeCallOf};
use pallet_dip_provider::IdentityCommitmentOf;
use parity_scale_codec::{Decode, Encode, HasCompact};
use scale_info::TypeInfo;
use sp_core::{RuntimeDebug, U256};
use sp_runtime::traits::Get;
use sp_std::{marker::PhantomData, vec::Vec};

use crate::{
	did::RevealedDidKeysSignatureAndCallVerifierError,
	merkle::{DidMerkleProofVerifierError, RevealedDidMerkleProofLeaf, RevealedDidMerkleProofLeaves},
	state_proofs::{parachain::DipIdentityCommitmentProofVerifierError, relay_chain::ParachainHeadProofVerifierError},
	traits::{self, Bump, DidSignatureVerifierContext, DipCallOriginFilter},
	utils::OutputOf,
	FrameSystemDidSignatureContext, ProviderParachainStateInfoViaProviderPallet,
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

struct KiltParachainId<Runtime, Id>(PhantomData<(Runtime, Id)>);

impl<Runtime, Id> Get<Id> for KiltParachainId<Runtime, Id>
where
	Runtime: parachain_info::Config,
	Id: From<ParaId>,
{
	fn get() -> Id {
		parachain_info::Pallet::<Runtime>::parachain_id().into()
	}
}

// Implements the same `IdentityProvider` trait, but it is internally configured
// by receiving the runtime definitions of both the provider and the receiver.
pub struct VersionedSiblingKiltProviderVerifier<
	KiltRuntime,
	ConsumerRuntime,
	RelayChainStateInfo,
	KiltDipMerkleHasher,
	LocalDidCallVerifier,
	const MAX_REVEALED_KEYS_COUNT: u32,
	const MAX_REVEALED_ACCOUNTS_COUNT: u32,
	const MAX_DID_SIGNATURE_DURATION: u16,
>(
	PhantomData<(
		KiltRuntime,
		ConsumerRuntime,
		RelayChainStateInfo,
		KiltDipMerkleHasher,
		LocalDidCallVerifier,
	)>,
);

impl<
		KiltRuntime,
		ConsumerRuntime,
		RelayChainStateInfo,
		KiltDipMerkleHasher,
		LocalDidCallVerifier,
		const MAX_REVEALED_KEYS_COUNT: u32,
		const MAX_REVEALED_ACCOUNTS_COUNT: u32,
		const MAX_DID_SIGNATURE_DURATION: u16,
	> IdentityProofVerifier<ConsumerRuntime>
	for VersionedSiblingKiltProviderVerifier<
		KiltRuntime,
		ConsumerRuntime,
		RelayChainStateInfo,
		KiltDipMerkleHasher,
		LocalDidCallVerifier,
		MAX_REVEALED_KEYS_COUNT,
		MAX_REVEALED_ACCOUNTS_COUNT,
		MAX_DID_SIGNATURE_DURATION,
	> where
	KiltRuntime: did::Config
		+ pallet_web3_names::Config
		+ pallet_did_lookup::Config
		+ parachain_info::Config
		+ pallet_dip_provider::Config<Identifier = ConsumerRuntime::Identifier>,
	OutputOf<KiltRuntime::Hashing>: Ord + From<OutputOf<RelayChainStateInfo::Hasher>>,
	KeyIdOf<KiltRuntime>: Into<KiltDipMerkleHasher::Out>,
	KiltDipMerkleHasher: sp_core::Hasher<Out = IdentityCommitmentOf<KiltRuntime>>,
	ConsumerRuntime: pallet_dip_consumer::Config,
	ConsumerRuntime::LocalIdentityInfo: Bump + Default + Encode,
	RelayChainStateInfo: traits::RelayChainStorageInfo + traits::RelayChainStateInfo,
	RelayChainStateInfo::ParaId: From<ParaId>,
	RelayChainStateInfo::BlockNumber: Parameter + 'static + Copy + Into<U256> + TryFrom<U256> + HasCompact,
	RelayChainStateInfo::Key: AsRef<[u8]>,
	LocalDidCallVerifier: DipCallOriginFilter<
		RuntimeCallOf<ConsumerRuntime>,
		OriginInfo = (
			DidVerificationKey<KiltRuntime::AccountId>,
			DidVerificationKeyRelationship,
		),
	>,
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
			KeyIdOf<KiltRuntime>,
			KiltRuntime::AccountId,
			BlockNumberFor<KiltRuntime>,
			KiltRuntime::Web3Name,
			LinkableAccountId,
		>,
		BlockNumberFor<ConsumerRuntime>,
	>;
	type VerificationResult = RevealedDidMerkleProofLeaves<
		KeyIdOf<KiltRuntime>,
		KiltRuntime::AccountId,
		BlockNumberFor<KiltRuntime>,
		KiltRuntime::Web3Name,
		LinkableAccountId,
		MAX_REVEALED_KEYS_COUNT,
		MAX_REVEALED_ACCOUNTS_COUNT,
	>;

	fn verify_proof_for_call_against_details(
		call: &RuntimeCallOf<ConsumerRuntime>,
		subject: &ConsumerRuntime::Identifier,
		submitter: &ConsumerRuntime::AccountId,
		identity_details: &mut Option<ConsumerRuntime::LocalIdentityInfo>,
		proof: Self::Proof,
	) -> Result<Self::VerificationResult, Self::Error> {
		match proof {
			VersionedSiblingParachainDipStateProof::V0(v0_proof) => <v0::DipSiblingProviderStateProofVerifier<
				RelayChainStateInfo,
				KiltParachainId<KiltRuntime, RelayChainStateInfo::ParaId>,
				ProviderParachainStateInfoViaProviderPallet<KiltRuntime>,
				KiltDipMerkleHasher,
				KeyIdOf<KiltRuntime>,
				KiltRuntime::AccountId,
				KiltRuntime::Web3Name,
				LinkableAccountId,
				MAX_REVEALED_KEYS_COUNT,
				MAX_REVEALED_ACCOUNTS_COUNT,
				FrameSystemDidSignatureContext<ConsumerRuntime, MAX_DID_SIGNATURE_DURATION>,
				LocalDidCallVerifier,
			> as IdentityProofVerifier<ConsumerRuntime>>::verify_proof_for_call_against_details(
				call,
				subject,
				submitter,
				identity_details,
				v0_proof,
			),
		}
	}
}

// More generic version compared to `VersionedSiblingKiltProviderVerifier`, to
// be used in cases in which it is not possible or not desirable to depend on
// the whole provider runtime definition. Hence, required types must be filled
// in manually.
pub struct GenericVersionedDipSiblingProviderStateProofVerifier<
	RelayChainStateInfo,
	SiblingProviderParachainId,
	SiblingProviderStateInfo,
	ProviderDipMerkleHasher,
	ProviderDidKeyId,
	ProviderAccountId,
	ProviderWeb3Name,
	ProviderLinkedAccountId,
	const MAX_REVEALED_KEYS_COUNT: u32,
	const MAX_REVEALED_ACCOUNTS_COUNT: u32,
	LocalContextProvider,
	LocalDidCallVerifier,
>(
	#[allow(clippy::type_complexity)]
	PhantomData<(
		RelayChainStateInfo,
		SiblingProviderParachainId,
		SiblingProviderStateInfo,
		ProviderDipMerkleHasher,
		ProviderDidKeyId,
		ProviderAccountId,
		ProviderWeb3Name,
		ProviderLinkedAccountId,
		LocalContextProvider,
		LocalDidCallVerifier,
	)>,
);

impl<
		ConsumerRuntime,
		RelayChainStateInfo,
		SiblingProviderParachainId,
		SiblingProviderStateInfo,
		ProviderDipMerkleHasher,
		ProviderDidKeyId,
		ProviderAccountId,
		ProviderWeb3Name,
		ProviderLinkedAccountId,
		const MAX_REVEALED_KEYS_COUNT: u32,
		const MAX_REVEALED_ACCOUNTS_COUNT: u32,
		LocalContextProvider,
		LocalDidCallVerifier,
	> IdentityProofVerifier<ConsumerRuntime>
	for GenericVersionedDipSiblingProviderStateProofVerifier<
		RelayChainStateInfo,
		SiblingProviderParachainId,
		SiblingProviderStateInfo,
		ProviderDipMerkleHasher,
		ProviderDidKeyId,
		ProviderAccountId,
		ProviderWeb3Name,
		ProviderLinkedAccountId,
		MAX_REVEALED_KEYS_COUNT,
		MAX_REVEALED_ACCOUNTS_COUNT,
		LocalContextProvider,
		LocalDidCallVerifier,
	> where
	ConsumerRuntime: pallet_dip_consumer::Config,
	ConsumerRuntime::LocalIdentityInfo: Bump + Default,

	RelayChainStateInfo: traits::RelayChainStorageInfo + traits::RelayChainStateInfo,
	OutputOf<RelayChainStateInfo::Hasher>: Ord,
	RelayChainStateInfo::BlockNumber: Parameter + 'static + Copy + Into<U256> + TryFrom<U256> + HasCompact,
	RelayChainStateInfo::Key: AsRef<[u8]>,

	SiblingProviderParachainId: Get<RelayChainStateInfo::ParaId>,

	SiblingProviderStateInfo: traits::ProviderParachainStateInfo<
		Identifier = ConsumerRuntime::Identifier,
		Commitment = ProviderDipMerkleHasher::Out,
	>,
	OutputOf<SiblingProviderStateInfo::Hasher>: Ord + From<OutputOf<RelayChainStateInfo::Hasher>>,
	SiblingProviderStateInfo::BlockNumber: Parameter + 'static,
	SiblingProviderStateInfo::Commitment: Decode,
	SiblingProviderStateInfo::Key: AsRef<[u8]>,

	LocalContextProvider:
		DidSignatureVerifierContext<BlockNumber = BlockNumberFor<ConsumerRuntime>, Hash = ConsumerRuntime::Hash>,
	LocalContextProvider::SignedExtra: Encode,
	LocalDidCallVerifier: DipCallOriginFilter<
		RuntimeCallOf<ConsumerRuntime>,
		OriginInfo = (DidVerificationKey<ProviderAccountId>, DidVerificationKeyRelationship),
	>,

	ProviderDipMerkleHasher: sp_core::Hasher,
	ProviderDidKeyId: Parameter + 'static + Into<ProviderDipMerkleHasher::Out>,
	ProviderAccountId: Parameter + 'static,
	ProviderLinkedAccountId: Parameter + 'static,
	ProviderWeb3Name: Parameter + 'static,
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
		BlockNumberFor<ConsumerRuntime>,
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
		call: &RuntimeCallOf<ConsumerRuntime>,
		subject: &ConsumerRuntime::Identifier,
		submitter: &ConsumerRuntime::AccountId,
		identity_details: &mut Option<ConsumerRuntime::LocalIdentityInfo>,
		proof: Self::Proof,
	) -> Result<Self::VerificationResult, Self::Error> {
		match proof {
			VersionedSiblingParachainDipStateProof::V0(v0_proof) => <v0::DipSiblingProviderStateProofVerifier<
				RelayChainStateInfo,
				SiblingProviderParachainId,
				SiblingProviderStateInfo,
				ProviderDipMerkleHasher,
				ProviderDidKeyId,
				ProviderAccountId,
				ProviderWeb3Name,
				ProviderLinkedAccountId,
				MAX_REVEALED_KEYS_COUNT,
				MAX_REVEALED_ACCOUNTS_COUNT,
				LocalContextProvider,
				LocalDidCallVerifier,
			> as IdentityProofVerifier<ConsumerRuntime>>::verify_proof_for_call_against_details(
				call,
				subject,
				submitter,
				identity_details,
				v0_proof,
			),
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
		ProviderDipMerkleHasher,
		ProviderDidKeyId,
		ProviderAccountId,
		ProviderWeb3Name,
		ProviderLinkedAccountId,
		const MAX_REVEALED_KEYS_COUNT: u32,
		const MAX_REVEALED_ACCOUNTS_COUNT: u32,
		LocalContextProvider,
		LocalDidCallVerifier,
	>(
		#[allow(clippy::type_complexity)]
		PhantomData<(
			RelayChainStateInfo,
			SiblingProviderParachainId,
			SiblingProviderStateInfo,
			ProviderDipMerkleHasher,
			ProviderDidKeyId,
			ProviderAccountId,
			ProviderWeb3Name,
			ProviderLinkedAccountId,
			LocalContextProvider,
			LocalDidCallVerifier,
		)>,
	);

	impl<
			ConsumerRuntime,
			RelayChainStateInfo,
			SiblingProviderParachainId,
			SiblingProviderStateInfo,
			ProviderDipMerkleHasher,
			ProviderDidKeyId,
			ProviderAccountId,
			ProviderWeb3Name,
			ProviderLinkedAccountId,
			const MAX_REVEALED_KEYS_COUNT: u32,
			const MAX_REVEALED_ACCOUNTS_COUNT: u32,
			LocalContextProvider,
			LocalDidCallVerifier,
		> IdentityProofVerifier<ConsumerRuntime>
		for DipSiblingProviderStateProofVerifier<
			RelayChainStateInfo,
			SiblingProviderParachainId,
			SiblingProviderStateInfo,
			ProviderDipMerkleHasher,
			ProviderDidKeyId,
			ProviderAccountId,
			ProviderWeb3Name,
			ProviderLinkedAccountId,
			MAX_REVEALED_KEYS_COUNT,
			MAX_REVEALED_ACCOUNTS_COUNT,
			LocalContextProvider,
			LocalDidCallVerifier,
		> where
		ConsumerRuntime: pallet_dip_consumer::Config,
		ConsumerRuntime::LocalIdentityInfo: Bump + Default,

		RelayChainStateInfo: traits::RelayChainStorageInfo + traits::RelayChainStateInfo,
		OutputOf<RelayChainStateInfo::Hasher>: Ord,
		RelayChainStateInfo::BlockNumber: Parameter + 'static + Copy + Into<U256> + TryFrom<U256> + HasCompact,
		RelayChainStateInfo::Key: AsRef<[u8]>,

		SiblingProviderParachainId: Get<RelayChainStateInfo::ParaId>,

		SiblingProviderStateInfo: traits::ProviderParachainStateInfo<
			Identifier = ConsumerRuntime::Identifier,
			Commitment = ProviderDipMerkleHasher::Out,
		>,
		OutputOf<SiblingProviderStateInfo::Hasher>: Ord + From<OutputOf<RelayChainStateInfo::Hasher>>,
		SiblingProviderStateInfo::BlockNumber: Parameter + 'static,
		SiblingProviderStateInfo::Commitment: Decode,
		SiblingProviderStateInfo::Key: AsRef<[u8]>,

		LocalContextProvider:
			DidSignatureVerifierContext<BlockNumber = BlockNumberFor<ConsumerRuntime>, Hash = ConsumerRuntime::Hash>,
		LocalContextProvider::SignedExtra: Encode,
		LocalDidCallVerifier: DipCallOriginFilter<
			RuntimeCallOf<ConsumerRuntime>,
			OriginInfo = (DidVerificationKey<ProviderAccountId>, DidVerificationKeyRelationship),
		>,

		ProviderDipMerkleHasher: sp_core::Hasher,
		ProviderDidKeyId: Parameter + 'static + Into<ProviderDipMerkleHasher::Out>,
		ProviderAccountId: Parameter + 'static,
		ProviderLinkedAccountId: Parameter + 'static,
		ProviderWeb3Name: Parameter + 'static,
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
			BlockNumberFor<ConsumerRuntime>,
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
			call: &RuntimeCallOf<ConsumerRuntime>,
			subject: &ConsumerRuntime::Identifier,
			submitter: &ConsumerRuntime::AccountId,
			identity_details: &mut Option<ConsumerRuntime::LocalIdentityInfo>,
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
