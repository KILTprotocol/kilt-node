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
use parity_scale_codec::{Codec, Decode, Encode, HasCompact};
use scale_info::TypeInfo;
use sp_core::{RuntimeDebug, U256};
use sp_runtime::traits::{AtLeast32BitUnsigned, Get, Hash, MaybeDisplay, Member, SimpleBitOps};
use sp_std::{marker::PhantomData, vec::Vec};

use crate::{
	did::RevealedDidKeysSignatureAndCallVerifierError,
	merkle::{DidMerkleProofVerifierError, RevealedDidMerkleProofLeaf, RevealedDidMerkleProofLeaves},
	state_proofs::{parachain::DipIdentityCommitmentProofVerifierError, relay_chain::ParachainHeadProofVerifierError},
	traits::{
		Bump, DidSignatureVerifierContext, DipCallOriginFilter, HistoricalBlockRegistry, ProviderParachainStateInfo,
		RelayChainStorageInfo,
	},
	utils::OutputOf,
	FrameSystemDidSignatureContext, ProviderParachainStateInfoViaProviderPallet,
};

#[derive(Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo, Clone)]
#[non_exhaustive]
pub enum VersionedChildParachainDipStateProof<
	ParentBlockHeight: Copy + Into<U256> + TryFrom<U256>,
	ParentBlockHasher: Hash,
	DipMerkleProofBlindedValues,
	DipMerkleProofRevealedLeaf,
> {
	V0(
		v0::ChildParachainDipStateProof<
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
	UnsupportedVersion,
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
			DipChildProviderStateProofVerifierError::UnsupportedVersion => 0,
			DipChildProviderStateProofVerifierError::InvalidBlockHeight => 1,
			DipChildProviderStateProofVerifierError::InvalidBlockHash => 2,
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

pub struct VersionedSiblingKiltProviderVerifier<
	KiltRuntime,
	ConsumerRuntime,
	RelayChainInfo,
	KiltDipMerkleHasher,
	LocalDidCallVerifier,
	const MAX_REVEALED_KEYS_COUNT: u32,
	const MAX_REVEALED_ACCOUNTS_COUNT: u32,
	const MAX_DID_SIGNATURE_DURATION: u16,
>(
	PhantomData<(
		KiltRuntime,
		ConsumerRuntime,
		RelayChainInfo,
		KiltDipMerkleHasher,
		LocalDidCallVerifier,
	)>,
);

impl<
		KiltRuntime,
		ConsumerRuntime,
		RelayChainInfo,
		KiltDipMerkleHasher,
		LocalDidCallVerifier,
		const MAX_REVEALED_KEYS_COUNT: u32,
		const MAX_REVEALED_ACCOUNTS_COUNT: u32,
		const MAX_DID_SIGNATURE_DURATION: u16,
	> IdentityProofVerifier<ConsumerRuntime>
	for VersionedSiblingKiltProviderVerifier<
		KiltRuntime,
		ConsumerRuntime,
		RelayChainInfo,
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
	OutputOf<KiltRuntime::Hashing>: Ord + From<OutputOf<<RelayChainInfo as RelayChainStorageInfo>::Hasher>>,
	KeyIdOf<KiltRuntime>: Into<KiltDipMerkleHasher::Out>,
	KiltDipMerkleHasher: sp_core::Hasher<Out = IdentityCommitmentOf<KiltRuntime>>,
	ConsumerRuntime: pallet_dip_consumer::Config,
	ConsumerRuntime::LocalIdentityInfo: Bump + Default + Encode,
	RelayChainInfo: RelayChainStorageInfo<BlockNumber = BlockNumberFor<ConsumerRuntime>>
		+ HistoricalBlockRegistry<
			BlockNumber = <RelayChainInfo as RelayChainStorageInfo>::BlockNumber,
			Hasher = <RelayChainInfo as RelayChainStorageInfo>::Hasher,
		>,
	RelayChainInfo::ParaId: From<ParaId>,
	OutputOf<<RelayChainInfo as RelayChainStorageInfo>::Hasher>:
		Ord + Default + sp_std::hash::Hash + Copy + Member + MaybeDisplay + SimpleBitOps + Codec,
	<RelayChainInfo as RelayChainStorageInfo>::Hasher: Parameter + 'static,
	<RelayChainInfo as RelayChainStorageInfo>::BlockNumber: Copy
		+ Into<U256>
		+ TryFrom<U256>
		+ HasCompact
		+ Member
		+ sp_std::hash::Hash
		+ MaybeDisplay
		+ AtLeast32BitUnsigned
		+ Codec
		+ Parameter
		+ 'static,
	RelayChainInfo::Key: AsRef<[u8]>,
	LocalDidCallVerifier: DipCallOriginFilter<
		RuntimeCallOf<ConsumerRuntime>,
		OriginInfo = (
			DidVerificationKey<KiltRuntime::AccountId>,
			DidVerificationKeyRelationship,
		),
	>,
{
	type Error = DipChildProviderStateProofVerifierError<
		ParachainHeadProofVerifierError,
		DipIdentityCommitmentProofVerifierError,
		DidMerkleProofVerifierError,
		RevealedDidKeysSignatureAndCallVerifierError,
	>;
	type Proof = VersionedChildParachainDipStateProof<
		<RelayChainInfo as RelayChainStorageInfo>::BlockNumber,
		<RelayChainInfo as RelayChainStorageInfo>::Hasher,
		Vec<Vec<u8>>,
		RevealedDidMerkleProofLeaf<
			KeyIdOf<KiltRuntime>,
			KiltRuntime::AccountId,
			BlockNumberFor<KiltRuntime>,
			KiltRuntime::Web3Name,
			LinkableAccountId,
		>,
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
			VersionedChildParachainDipStateProof::V0(v0_proof) => <v0::DipChildProviderStateProofVerifier<
				RelayChainInfo,
				KiltParachainId<KiltRuntime, RelayChainInfo::ParaId>,
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

pub struct VersionedDipChildProviderStateProofVerifier<
	RelayChainInfo,
	ChildProviderParachainId,
	ChildProviderStateInfo,
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
		RelayChainInfo,
		ChildProviderParachainId,
		ChildProviderStateInfo,
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
		RelayChainInfo,
		ChildProviderParachainId,
		ChildProviderStateInfo,
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
	for VersionedDipChildProviderStateProofVerifier<
		RelayChainInfo,
		ChildProviderParachainId,
		ChildProviderStateInfo,
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

	RelayChainInfo: RelayChainStorageInfo<BlockNumber = BlockNumberFor<ConsumerRuntime>>
		+ HistoricalBlockRegistry<
			BlockNumber = <RelayChainInfo as RelayChainStorageInfo>::BlockNumber,
			Hasher = <RelayChainInfo as RelayChainStorageInfo>::Hasher,
		>,
	OutputOf<<RelayChainInfo as RelayChainStorageInfo>::Hasher>:
		Ord + Default + sp_std::hash::Hash + Copy + Member + MaybeDisplay + SimpleBitOps + Codec,
	<RelayChainInfo as RelayChainStorageInfo>::Hasher: Parameter + 'static,
	<RelayChainInfo as RelayChainStorageInfo>::BlockNumber: Copy
		+ Into<U256>
		+ TryFrom<U256>
		+ HasCompact
		+ Member
		+ sp_std::hash::Hash
		+ MaybeDisplay
		+ AtLeast32BitUnsigned
		+ Codec
		+ Parameter
		+ 'static,
	RelayChainInfo::Key: AsRef<[u8]>,

	ChildProviderParachainId: Get<RelayChainInfo::ParaId>,

	ChildProviderStateInfo:
		ProviderParachainStateInfo<Identifier = ConsumerRuntime::Identifier, Commitment = ProviderDipMerkleHasher::Out>,
	OutputOf<ChildProviderStateInfo::Hasher>: Ord + From<OutputOf<<RelayChainInfo as RelayChainStorageInfo>::Hasher>>,
	ChildProviderStateInfo::BlockNumber: Parameter + 'static,
	ChildProviderStateInfo::Commitment: Decode,
	ChildProviderStateInfo::Key: AsRef<[u8]>,

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
	type Error = DipChildProviderStateProofVerifierError<
		ParachainHeadProofVerifierError,
		DipIdentityCommitmentProofVerifierError,
		DidMerkleProofVerifierError,
		RevealedDidKeysSignatureAndCallVerifierError,
	>;
	type Proof = VersionedChildParachainDipStateProof<
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
		call: &RuntimeCallOf<ConsumerRuntime>,
		subject: &ConsumerRuntime::Identifier,
		submitter: &ConsumerRuntime::AccountId,
		identity_details: &mut Option<ConsumerRuntime::LocalIdentityInfo>,
		proof: Self::Proof,
	) -> Result<Self::VerificationResult, Self::Error> {
		match proof {
			VersionedChildParachainDipStateProof::V0(v0_proof) => <v0::DipChildProviderStateProofVerifier<
				RelayChainInfo,
				ChildProviderParachainId,
				ChildProviderStateInfo,
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
	pub use super::v0::ChildParachainDipStateProof;
}

mod v0 {
	use super::*;

	use parity_scale_codec::Codec;
	use sp_runtime::{
		generic::Header,
		traits::{AtLeast32BitUnsigned, Hash, MaybeDisplay, Member, SimpleBitOps},
	};
	use sp_std::{borrow::Borrow, vec::Vec};

	use crate::{
		did::{
			RevealedDidKeysAndSignature, RevealedDidKeysSignatureAndCallVerifier,
			RevealedDidKeysSignatureAndCallVerifierError,
		},
		export::common::v0::{DipMerkleProofAndDidSignature, ParachainRootStateProof},
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
			RelayChainInfo,
			ChildProviderParachainId,
			ChildProviderStateInfo,
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
			RelayChainInfo,
			ChildProviderParachainId,
			ChildProviderStateInfo,
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
		for DipChildProviderStateProofVerifier<
			RelayChainInfo,
			ChildProviderParachainId,
			ChildProviderStateInfo,
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

		RelayChainInfo: RelayChainStorageInfo<BlockNumber = BlockNumberFor<ConsumerRuntime>>
			+ HistoricalBlockRegistry<
				BlockNumber = <RelayChainInfo as RelayChainStorageInfo>::BlockNumber,
				Hasher = <RelayChainInfo as RelayChainStorageInfo>::Hasher,
			>,
		<RelayChainInfo as RelayChainStorageInfo>::Hasher: Parameter + 'static,
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
			+ Codec
			+ Parameter
			+ 'static,
		RelayChainInfo::Key: AsRef<[u8]>,

		ChildProviderParachainId: Get<RelayChainInfo::ParaId>,

		ChildProviderStateInfo: ProviderParachainStateInfo<
			Identifier = ConsumerRuntime::Identifier,
			Commitment = ProviderDipMerkleHasher::Out,
		>,
		OutputOf<ChildProviderStateInfo::Hasher>:
			Ord + From<OutputOf<<RelayChainInfo as RelayChainStorageInfo>::Hasher>>,
		ChildProviderStateInfo::BlockNumber: Parameter + 'static,
		ChildProviderStateInfo::Commitment: Decode,
		ChildProviderStateInfo::Key: AsRef<[u8]>,

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
		type Error = DipChildProviderStateProofVerifierError<
			ParachainHeadProofVerifierError,
			DipIdentityCommitmentProofVerifierError,
			DidMerkleProofVerifierError,
			RevealedDidKeysSignatureAndCallVerifierError,
		>;
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
			call: &RuntimeCallOf<ConsumerRuntime>,
			subject: &ConsumerRuntime::Identifier,
			submitter: &ConsumerRuntime::AccountId,
			identity_details: &mut Option<ConsumerRuntime::LocalIdentityInfo>,
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
