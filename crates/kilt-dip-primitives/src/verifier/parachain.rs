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
use sp_std::marker::PhantomData;

use crate::{
	did::RevealedDidKeysSignatureAndCallVerifierError,
	merkle::{DidMerkleProofVerifierError, RevealedDidMerkleProofLeaf, RevealedDidMerkleProofLeaves},
	state_proofs::{parachain::DipIdentityCommitmentProofVerifierError, relaychain::ParachainHeadProofVerifierError},
	traits::{self, DidSignatureVerifierContext, DipCallOriginFilter, Incrementable},
	utils::OutputOf,
	BoundedBlindedValue, FrameSystemDidSignatureContext, ProviderParachainStateInfoViaProviderPallet,
};

/// A KILT-specific DIP identity proof for a sibling consumer that supports
/// versioning.
///
/// For more info, refer to the version-specific proofs.
#[derive(Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo, Clone)]
pub enum VersionedDipParachainStateProof<
	RelayBlockHeight,
	DipMerkleProofBlindedValues,
	DipMerkleProofRevealedLeaf,
	LocalBlockNumber,
> {
	V0(
		v0::ParachainDipStateProof<
			RelayBlockHeight,
			DipMerkleProofBlindedValues,
			DipMerkleProofRevealedLeaf,
			LocalBlockNumber,
		>,
	),
}

#[cfg(feature = "runtime-benchmarks")]
impl<RelayBlockHeight, DipMerkleProofBlindedValues, DipMerkleProofRevealedLeaf, LocalBlockNumber, Context>
	kilt_support::traits::GetWorstCase<Context>
	for VersionedDipParachainStateProof<
		RelayBlockHeight,
		DipMerkleProofBlindedValues,
		DipMerkleProofRevealedLeaf,
		LocalBlockNumber,
	> where
	RelayBlockHeight: Default,
	DipMerkleProofBlindedValues: kilt_support::traits::GetWorstCase<Context>,
	DipMerkleProofRevealedLeaf: Default + Clone,
	LocalBlockNumber: Default,
	Context: Clone,
{
	fn worst_case(context: Context) -> Self {
		Self::V0(v0::ParachainDipStateProof::worst_case(context))
	}
}

pub enum DipParachainStateProofVerifierError<
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
		DipParachainStateProofVerifierError<
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
		value: DipParachainStateProofVerifierError<
			ParachainHeadMerkleProofVerificationError,
			IdentityCommitmentMerkleProofVerificationError,
			DipProofVerificationError,
			DidSignatureVerificationError,
		>,
	) -> Self {
		match value {
			DipParachainStateProofVerifierError::UnsupportedVersion => 0,
			DipParachainStateProofVerifierError::ParachainHeadMerkleProof(error) => {
				u8::MAX as u16 + error.into() as u16
			}
			DipParachainStateProofVerifierError::IdentityCommitmentMerkleProof(error) => {
				u8::MAX as u16 * 2 + error.into() as u16
			}
			DipParachainStateProofVerifierError::DipProof(error) => u8::MAX as u16 * 3 + error.into() as u16,
			DipParachainStateProofVerifierError::DidSignature(error) => u8::MAX as u16 * 4 + error.into() as u16,
		}
	}
}

/// Proof verifier configured given a specific KILT runtime implementation.
///
/// It is a specialization of the
/// [`GenericVersionedParachainVerifier`] type, with
/// configurations derived from the provided KILT runtime.
///
/// The generic types
/// indicate the following:
/// * `KiltRuntime`: A KILT runtime definition.
/// * `KiltParachainId`: The ID of the specific KILT parachain instance.
/// * `RelayChainInfo`: The type providing information about the relaychain.
/// * `KiltDipMerkleHasher`: The hashing algorithm used by the KILT parachain
///   for the generation of the DIP identity commitment.
/// * `LocalDidCallVerifier`: Logic to map `RuntimeCall`s to a specific DID key
///   relationship. This information is used once the Merkle proof is verified,
///   to filter only the revealed keys that match the provided relationship.
/// * `MAX_REVEALED_KEYS_COUNT`: **OPTIONAL** Max number of DID keys that the
///   verifier will accept revealed as part of the DIP identity proof. It
///   defaults to **10**.
/// * `MAX_REVEALED_ACCOUNTS_COUNT`: **OPTIONAL** Max number of linked accounts
///   that the verifier will accept revealed as part of the DIP identity proof.
///   It defaults to **10**.
/// * `MAX_DID_SIGNATURE_DURATION`: **OPTIONAL** Max number of blocks a
///   cross-chain DID signature is considered fresh. It defaults to **50**.
///
/// It specializes the [`GenericVersionedParachainVerifier`]
/// type by using the following types for its generics:
/// * `RelayChainInfo`: The provided `RelayChainInfo`.
/// * `ChildProviderParachainId`: The provided `KiltParachainId`.
/// * `ChildProviderStateInfo`: The
///   [`ProviderParachainStateInfoViaProviderPallet`] type configured with the
///   provided `KiltRuntime`.
/// * `ProviderDipMerkleHasher`: The provided `KiltDipMerkleHasher`.
/// * `ProviderDidKeyId`: The [`KeyIdOf`] type configured with the provided
///   `KiltRuntime`.
/// * `ProviderAccountId`: The `KiltRuntime::AccountId` type.
/// * `ProviderWeb3Name`: The `KiltRuntime::Web3Name` type.
/// * `ProviderLinkedAccountId`: The [`LinkableAccountId`] type.
/// * `MAX_REVEALED_KEYS_COUNT`: The provided `MAX_REVEALED_KEYS_COUNT`.
/// * `MAX_REVEALED_ACCOUNTS_COUNT`: The provided `MAX_REVEALED_ACCOUNTS_COUNT`.
/// * `LocalContextProvider`: The [`FrameSystemDidSignatureContext`] type
///   configured with the provided `KiltRuntime` and
///   `MAX_DID_SIGNATURE_DURATION`.
/// * `LocalDidCallVerifier`: The provided `LocalDidCallVerifier`.
pub struct KiltVersionedParachainVerifier<
	KiltRuntime,
	KiltParachainId,
	RelayChainStateInfo,
	KiltDipMerkleHasher,
	LocalDidCallVerifier,
	const MAX_REVEALED_KEYS_COUNT: u32 = 10,
	const MAX_REVEALED_ACCOUNTS_COUNT: u32 = 10,
	const MAX_DID_SIGNATURE_DURATION: u16 = 50,
>(
	PhantomData<(
		KiltRuntime,
		KiltParachainId,
		RelayChainStateInfo,
		KiltDipMerkleHasher,
		LocalDidCallVerifier,
	)>,
);

impl<
		ConsumerRuntime,
		KiltRuntime,
		KiltParachainId,
		RelayChainStateInfo,
		KiltDipMerkleHasher,
		LocalDidCallVerifier,
		const MAX_REVEALED_KEYS_COUNT: u32,
		const MAX_REVEALED_ACCOUNTS_COUNT: u32,
		const MAX_DID_SIGNATURE_DURATION: u16,
	> IdentityProofVerifier<ConsumerRuntime>
	for KiltVersionedParachainVerifier<
		KiltRuntime,
		KiltParachainId,
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
	KiltParachainId: Get<RelayChainStateInfo::ParaId>,
	OutputOf<KiltRuntime::Hashing>: Ord + From<OutputOf<RelayChainStateInfo::Hasher>>,
	KeyIdOf<KiltRuntime>: Into<KiltDipMerkleHasher::Out>,
	KiltDipMerkleHasher: sp_core::Hasher<Out = IdentityCommitmentOf<KiltRuntime>>,
	ConsumerRuntime: pallet_dip_consumer::Config,
	ConsumerRuntime::LocalIdentityInfo: Incrementable + Default + Encode,
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
	type Error = DipParachainStateProofVerifierError<
		ParachainHeadProofVerifierError,
		DipIdentityCommitmentProofVerifierError,
		DidMerkleProofVerifierError,
		RevealedDidKeysSignatureAndCallVerifierError,
	>;
	type Proof = VersionedDipParachainStateProof<
		RelayChainStateInfo::BlockNumber,
		BoundedBlindedValue<u8>,
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
		<GenericVersionedParachainVerifier<
			RelayChainStateInfo,
			KiltParachainId,
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
			proof,
		)
	}
}

/// Generic proof verifier for KILT-specific DIP identity proofs of different
/// versions coming from a sibling provider running one of the available KILT
/// runtimes.
///
/// It expects the DIP proof to be a [`VersionedDipParachainStateProof`],
/// and returns [`RevealedDidMerkleProofLeaves`] if the proof is successfully
/// verified.
///
/// For more info, refer to the version-specific proof identifiers.
pub struct GenericVersionedParachainVerifier<
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
	for GenericVersionedParachainVerifier<
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
	ConsumerRuntime::LocalIdentityInfo: Incrementable + Default,

	RelayChainStateInfo: traits::RelayChainStorageInfo + traits::RelayChainStateInfo,
	OutputOf<RelayChainStateInfo::Hasher>: Ord,
	RelayChainStateInfo::BlockNumber: Parameter + 'static + Copy + Into<U256> + TryFrom<U256> + HasCompact,
	RelayChainStateInfo::Key: AsRef<[u8]>,

	SiblingProviderParachainId: Get<RelayChainStateInfo::ParaId>,

	SiblingProviderStateInfo: traits::ProviderParachainStorageInfo<
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
	type Error = DipParachainStateProofVerifierError<
		ParachainHeadProofVerifierError,
		DipIdentityCommitmentProofVerifierError,
		DidMerkleProofVerifierError,
		RevealedDidKeysSignatureAndCallVerifierError,
	>;
	type Proof = VersionedDipParachainStateProof<
		RelayChainStateInfo::BlockNumber,
		BoundedBlindedValue<u8>,
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
			VersionedDipParachainStateProof::V0(v0_proof) => <v0::ParachainVerifier<
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
	pub use super::v0::ParachainDipStateProof;
}

pub mod v0 {
	use super::*;

	use frame_support::Parameter;
	use sp_std::borrow::Borrow;

	use crate::{
		did::{verify_did_signature_for_call, RevealedDidKeysAndSignature},
		merkle::verify_dip_merkle_proof,
		state_proofs::{parachain::DipIdentityCommitmentProofVerifier, relaychain::ParachainHeadProofVerifier},
		traits::ProviderParachainStorageInfo,
		verifier::common::v0::{DipMerkleProofAndDidSignature, ParachainRootStateProof},
	};

	/// The expected format of a cross-chain DIP identity proof when the
	/// identity information is bridged from a provider that is a sibling
	/// of the chain where the information is consumed (i.e., consumer
	/// chain).
	#[derive(Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo, Clone)]
	pub struct ParachainDipStateProof<
		RelayBlockHeight,
		DipMerkleProofBlindedValues,
		DipMerkleProofRevealedLeaf,
		LocalBlockNumber,
	> {
		/// The state proof for the given parachain head.
		pub(crate) para_state_root: ParachainRootStateProof<RelayBlockHeight>,
		/// The raw state proof for the DIP commitment of the given subject.
		pub(crate) dip_identity_commitment: BoundedBlindedValue<u8>,
		/// The cross-chain DID signature.
		pub(crate) did:
			DipMerkleProofAndDidSignature<DipMerkleProofBlindedValues, DipMerkleProofRevealedLeaf, LocalBlockNumber>,
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl<RelayBlockHeight, DipMerkleProofBlindedValues, DipMerkleProofRevealedLeaf, LocalBlockNumber, Context>
		kilt_support::traits::GetWorstCase<Context>
		for ParachainDipStateProof<RelayBlockHeight, DipMerkleProofBlindedValues, DipMerkleProofRevealedLeaf, LocalBlockNumber>
	where
		DipMerkleProofBlindedValues: kilt_support::traits::GetWorstCase<Context>,
		DipMerkleProofRevealedLeaf: Default + Clone,
		RelayBlockHeight: Default,
		LocalBlockNumber: Default,
		Context: Clone,
	{
		fn worst_case(context: Context) -> Self {
			Self {
				para_state_root: ParachainRootStateProof::worst_case(context.clone()),
				dip_identity_commitment: BoundedBlindedValue::worst_case(context.clone()),
				did: DipMerkleProofAndDidSignature::worst_case(context),
			}
		}
	}

	/// Generic proof verifier for KILT-specific DIP identity proofs coming from
	/// a sibling provider running one of the available KILT runtimes.
	///
	/// The proof verification step is performed on every request, and this
	/// specific verifier has no knowledge of caching or storing state about the
	/// subject. It only takes the provided
	/// `ConsumerRuntime::LocalIdentityInfo` and increases it if the proof is
	/// successfully verified, to prevent replay attacks. If additional logic is
	/// to be stored under the `ConsumerRuntime::LocalIdentityInfo` entry, a
	/// different verifier or a wrapper around this verifier must be built.
	///
	/// It expects the DIP proof to be a
	/// [`VersionedDipParachainStateProof`], and returns
	/// [`RevealedDidMerkleProofLeaves`] if the proof is successfully verified.
	/// This information is then made availabe as an origin to the downstream
	/// call dispatched.
	///
	/// The verifier performs the following steps:
	/// 1. Verifies the state proof about the state root of the relaychain block
	///    at the provided height. The state root is provided by the
	///    `RelayChainInfo` type.
	/// 2. Verifies the state proof about the DIP commitment value on the
	///    provider parachain at the block finalized at the given relaychain
	///    block, using the relay state root validated in the previous step.
	/// 3. Verifies the DIP Merkle proof revealing parts of the subject's DID
	///    Document against the retrieved DIP commitment validated in the
	///    previous step.
	/// 4. Verifies the cross-chain DID signature over the payload composed by
	///    the SCALE-encoded tuple of `(C, D, S, B, G, E)`, with:
	///    * `C`: The `RuntimeCall` to dispatch after performing DIP
	///      verification.
	///    * `D`: The local details associated to the DID subject as stored in
	///      the [`pallet_dip_consumer`] `IdentityEntries` storage map.
	///    * `S`: The tx submitter's address.
	///    * `B`: The block number of the consumer chain provided in the
	///      cross-chain DID signature.
	///    * `G`: The genesis hash of the consumer chain.
	///    * `E`: Any additional information provided by the
	///      `LocalContextProvider` implementation.
	/// The generic types
	/// indicate the following:
	/// * `RelayChainInfo`: The type providing information about the relaychain.
	/// * `SiblingProviderParachainId`: The parachain ID of the provider KILT
	///   sibling parachain.
	/// * `SiblingProviderStateInfo`: The type providing storage and state
	///   information about the provider KILT sibling parachain.
	/// * `ProviderDipMerkleHasher`: The hashing algorithm used by the KILT
	///   parachain for the generation of the DIP identity commitment.
	/// * `ProviderDidKeyId`: The runtime type of a DID key ID as defined by the
	///   KILT child parachain.
	/// * `ProviderAccountId`: The runtime type of an account ID as defined by
	///   the KILT child parachain.
	/// * `ProviderWeb3Name`: The runtime type of a web3name as defined by the
	///   KILT child parachain.
	/// * `ProviderLinkedAccountId`: The runtime type of a linked account ID as
	///   defined by the KILT child parachain.
	/// * `MAX_REVEALED_KEYS_COUNT`: Max number of DID keys that the verifier
	///   will accept revealed as part of the DIP identity proof.
	/// * `MAX_REVEALED_ACCOUNTS_COUNT`: Max number of linked accounts that the
	///   verifier will accept revealed as part of the DIP identity proof.
	/// * `LocalContextProvider`: The type providing context of the consumer
	///   chain (e.g., current block number) for the sake of cross-chain DID
	///   signature verification.
	/// * `LocalDidCallVerifier`: Logic to map `RuntimeCall`s to a specific DID
	///   key relationship. This information is used once the Merkle proof is
	///   verified, to filter only the revealed keys that match the provided
	///   relationship.
	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
	pub struct ParachainVerifier<
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
		for ParachainVerifier<
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
		ConsumerRuntime::LocalIdentityInfo: Incrementable + Default,

		RelayChainStateInfo: traits::RelayChainStorageInfo + traits::RelayChainStateInfo,
		OutputOf<RelayChainStateInfo::Hasher>: Ord,
		RelayChainStateInfo::BlockNumber: Parameter + 'static + Copy + Into<U256> + TryFrom<U256> + HasCompact,
		RelayChainStateInfo::Key: AsRef<[u8]>,

		SiblingProviderParachainId: Get<RelayChainStateInfo::ParaId>,

		SiblingProviderStateInfo: traits::ProviderParachainStorageInfo<
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
		type Error = DipParachainStateProofVerifierError<
			ParachainHeadProofVerifierError,
			DipIdentityCommitmentProofVerifierError,
			DidMerkleProofVerifierError,
			RevealedDidKeysSignatureAndCallVerifierError,
		>;
		type Proof = ParachainDipStateProof<
			RelayChainStateInfo::BlockNumber,
			BoundedBlindedValue<u8>,
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
			// 1. Verify parachain state is finalized by relay chain and fresh.
			let provider_parachain_header =
				ParachainHeadProofVerifier::<RelayChainStateInfo>::verify_proof_for_parachain(
					&SiblingProviderParachainId::get(),
					&proof.para_state_root.relay_block_height,
					proof.para_state_root.proof,
				)
				.map_err(DipParachainStateProofVerifierError::ParachainHeadMerkleProof)?;

			// 2. Verify commitment is included in provider parachain.
			let subject_identity_commitment =
				DipIdentityCommitmentProofVerifier::<SiblingProviderStateInfo>::verify_proof_for_identifier(
					subject,
					provider_parachain_header.state_root.into(),
					proof.dip_identity_commitment,
				)
				.map_err(DipParachainStateProofVerifierError::IdentityCommitmentMerkleProof)?;

			// 3. Verify DIP merkle proof.
			let proof_leaves: RevealedDidMerkleProofLeaves<
				ProviderDidKeyId,
				ProviderAccountId,
				<SiblingProviderStateInfo as ProviderParachainStorageInfo>::BlockNumber,
				ProviderWeb3Name,
				ProviderLinkedAccountId,
				MAX_REVEALED_KEYS_COUNT,
				MAX_REVEALED_ACCOUNTS_COUNT,
			> = verify_dip_merkle_proof::<
				ProviderDipMerkleHasher,
				_,
				_,
				_,
				_,
				_,
				MAX_REVEALED_KEYS_COUNT,
				MAX_REVEALED_ACCOUNTS_COUNT,
			>(&subject_identity_commitment, proof.did.leaves)
			.map_err(DipParachainStateProofVerifierError::DipProof)?;

			// 4. Verify call is signed by one of the DID keys revealed at step 3.
			verify_did_signature_for_call::<_, _, _, _, LocalContextProvider, _, _, _, LocalDidCallVerifier>(
				call,
				submitter,
				identity_details,
				RevealedDidKeysAndSignature {
					merkle_leaves: proof_leaves.borrow(),
					did_signature: proof.did.signature,
				},
			)
			.map_err(DipParachainStateProofVerifierError::DidSignature)?;

			Ok(proof_leaves)
		}
	}
}
