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

use did::KeyIdOf;
use frame_system::pallet_prelude::BlockNumberFor;
use pallet_did_lookup::linkable_account::LinkableAccountId;
use pallet_dip_consumer::{traits::IdentityProofVerifier, RuntimeCallOf};
use pallet_dip_provider::IdentityCommitmentOf;
use pallet_web3_names::Web3NameOf;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_core::{RuntimeDebug, U256};
use sp_runtime::traits::Hash;
use sp_std::marker::PhantomData;

use crate::{
	merkle::v0::RevealedDidKey,
	traits::{DipCallOriginFilter, GetWithArg, GetWithoutArg, Incrementable},
	utils::OutputOf,
	DipVerifiedInfo, Error,
};

/// A KILT-specific DIP identity proof for a parent consumer that supports
/// versioning.
///
/// For more info, refer to the version-specific proofs.
#[derive(Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo, Clone)]
pub enum VersionedRelaychainStateProof<
	ConsumerBlockNumber: Copy + Into<U256> + TryFrom<U256>,
	ConsumerBlockHasher: Hash,
	KiltDidKeyId,
	KiltAccountId,
	KiltBlockNumber,
	KiltWeb3Name,
	KiltLinkableAccountId,
> {
	V0(
		crate::merkle::v0::RelayDipDidProof<
			ConsumerBlockNumber,
			ConsumerBlockHasher,
			KiltDidKeyId,
			KiltAccountId,
			KiltBlockNumber,
			KiltWeb3Name,
			KiltLinkableAccountId,
		>,
	),
}

pub enum DipRelaychainStateProofVerifierError<DidOriginError> {
	UnsupportedVersion,
	ProofVerification(Error),
	DidOriginError(DidOriginError),
	Internal,
}

impl<DidOriginError> From<DipRelaychainStateProofVerifierError<DidOriginError>> for u16
where
	DidOriginError: Into<u8>,
{
	fn from(value: DipRelaychainStateProofVerifierError<DidOriginError>) -> Self {
		match value {
			DipRelaychainStateProofVerifierError::UnsupportedVersion => 1,
			DipRelaychainStateProofVerifierError::ProofVerification(error) => u8::MAX as u16 + u8::from(error) as u16,
			DipRelaychainStateProofVerifierError::DidOriginError(error) => u8::MAX as u16 * 2 + error.into() as u16,
			DipRelaychainStateProofVerifierError::Internal => u16::MAX,
		}
	}
}

/// Proof verifier configured given a specific KILT runtime implementation.
///
/// A specialization of the
/// [`GenericVersionedRelaychainVerifier`] type, with
/// configurations derived from the provided KILT runtime.
///
/// The generic types are the following:
/// * `KiltRuntime`: A KILT runtime definition.
/// * `KiltParachainId`: The ID of the specific KILT parachain instance.
/// * `RelayChainInfo`: The type providing information about the consumer
///   (relay)chain.
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
/// It specializes the [`GenericVersionedRelaychainVerifier`]
/// type by using the following types for its generics:
/// * `RelayChainInfo`: The provided `RelayChainInfo`.
/// * `ChildProviderParachainId`: The provided `KiltParachainId`.
/// * `ChildProviderStateInfo`: The
///   [`ProviderParachainStateInfoViaProviderPallet`] type configured with the
///   provided `KiltRuntime`.
/// * `ProviderDipMerkleHasher`: The provided `KiltDipMerkleHasher`.
/// * `ProviderDidKeyId`: The [`KeyIdOf`] type configured with the provided
///   `KiltRuntime`.
/// * `KiltAccountId`: The `KiltRuntime::AccountId` type.
/// * `KiltWeb3Name`: The `KiltRuntime::Web3Name` type.
/// * `KiltLinkableAccountId`: The [`LinkableAccountId`] type.
/// * `MAX_REVEALED_KEYS_COUNT`: The provided `MAX_REVEALED_KEYS_COUNT`.
/// * `MAX_REVEALED_ACCOUNTS_COUNT`: The provided `MAX_REVEALED_ACCOUNTS_COUNT`.
/// * `LocalContextProvider`: The [`FrameSystemDidSignatureContext`] type
///   configured with the provided `KiltRuntime` and
///   `MAX_DID_SIGNATURE_DURATION`.
/// * `LocalDidCallVerifier`: The provided `LocalDidCallVerifier`.
pub struct KiltVersionedRelaychainVerifier<
	ConsumerBlockHashStore,
	const KILT_PARA_ID: u32,
	KiltRuntime,
	DidCallVerifier,
	SignedExtra = (),
	const MAX_LEAVES_REVEALED: u32 = 50,
>(#[allow(clippy::type_complexity)] PhantomData<(ConsumerBlockHashStore, KiltRuntime, DidCallVerifier, SignedExtra)>);

impl<
		ConsumerRuntime,
		ConsumerBlockHashStore,
		const KILT_PARA_ID: u32,
		KiltRuntime,
		DidCallVerifier,
		SignedExtra,
		const MAX_LEAVES_REVEALED: u32,
	> IdentityProofVerifier<ConsumerRuntime>
	for KiltVersionedRelaychainVerifier<
		ConsumerBlockHashStore,
		KILT_PARA_ID,
		KiltRuntime,
		DidCallVerifier,
		SignedExtra,
		MAX_LEAVES_REVEALED,
	> where
	ConsumerRuntime: pallet_dip_consumer::Config<Identifier = KiltRuntime::Identifier>,
	ConsumerRuntime::LocalIdentityInfo: Incrementable + Default,
	BlockNumberFor<ConsumerRuntime>: Into<U256> + TryFrom<U256>,
	ConsumerBlockHashStore:
		GetWithArg<BlockNumberFor<ConsumerRuntime>, Result = Option<OutputOf<ConsumerRuntime::Hashing>>>,
	KiltRuntime: pallet_dip_provider::Config + did::Config + pallet_web3_names::Config + pallet_did_lookup::Config,
	IdentityCommitmentOf<KiltRuntime>: Into<KiltRuntime::Hash>,
	SignedExtra: GetWithoutArg,
	SignedExtra::Result: Encode,
	DidCallVerifier: DipCallOriginFilter<
		RuntimeCallOf<ConsumerRuntime>,
		OriginInfo = RevealedDidKey<KeyIdOf<KiltRuntime>, BlockNumberFor<KiltRuntime>, KiltRuntime::AccountId>,
	>,
	DidCallVerifier::Error: Into<u8>,
{
	type Error = DipRelaychainStateProofVerifierError<DidCallVerifier::Error>;
	type Proof = VersionedRelaychainStateProof<
		BlockNumberFor<ConsumerRuntime>,
		ConsumerRuntime::Hashing,
		KeyIdOf<KiltRuntime>,
		KiltRuntime::AccountId,
		BlockNumberFor<KiltRuntime>,
		Web3NameOf<KiltRuntime>,
		LinkableAccountId,
	>;
	type VerificationResult = DipVerifiedInfo<
		KeyIdOf<KiltRuntime>,
		KiltRuntime::AccountId,
		BlockNumberFor<KiltRuntime>,
		Web3NameOf<KiltRuntime>,
		LinkableAccountId,
	>;

	fn verify_proof_for_call_against_details(
		call: &RuntimeCallOf<ConsumerRuntime>,
		subject: &ConsumerRuntime::Identifier,
		submitter: &ConsumerRuntime::AccountId,
		identity_details: &mut Option<ConsumerRuntime::LocalIdentityInfo>,
		proof: Self::Proof,
	) -> Result<Self::VerificationResult, Self::Error> {
		match proof {
			VersionedRelaychainStateProof::V0(v0_proof) => <v0::RelaychainVerifier<
				ConsumerBlockHashStore,
				KILT_PARA_ID,
				KiltRuntime,
				DidCallVerifier,
				SignedExtra,
				MAX_LEAVES_REVEALED,
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

pub mod v0 {
	use super::*;

	use frame_system::pallet_prelude::HeaderFor;
	use sp_runtime::traits::Zero;

	use crate::RelayDipDidProof;

	/// Generic proof verifier for KILT-specific DIP identity proofs coming from
	/// a child provider running one of the available KILT runtimes.
	/// The proof verification step is performed on every request, and this
	/// specific verifier has no knowledge of caching or storing state about the
	/// subject. It only takes the provided
	/// `ConsumerRuntime::LocalIdentityInfo` and increases it if the proof is
	/// successfully verified, to prevent replay attacks. If additional logic is
	/// to be stored under the `ConsumerRuntime::LocalIdentityInfo` entry, a
	/// different verifier or a wrapper around this verifier must be built.
	///
	/// It expects the DIP proof to be a
	/// [`VersionedRelaychainStateProof`], and returns
	/// [`RevealedDidMerkleProofLeaves`] if the proof is successfully verified.
	/// This information is then made availabe as an origin to the downstream
	/// call dispatched.
	///
	/// The verifier performs the following steps:
	/// 1. Verifies the state proof about the state root of the relaychain block
	///    at the provided height. The state root is retrieved from the provided
	///    relaychain header, which is checked to be the header of a
	///    previously-finalized relaychain block.
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
	/// * `RelayChainInfo`: The type providing information about the consumer
	///   (relay)chain.
	/// * `ChildProviderParachainId`: The parachain ID of the provider KILT
	///   child parachain.
	/// * `ChildProviderStateInfo`: The type providing storage and state
	///   information about the provider KILT child parachain.
	/// * `ProviderDipMerkleHasher`: The hashing algorithm used by the KILT
	///   parachain for the generation of the DIP identity commitment.
	/// * `ProviderDidKeyId`: The runtime type of a DID key ID as defined by the
	///   KILT child parachain.
	/// * `KiltAccountId`: The runtime type of an account ID as defined by the
	///   KILT child parachain.
	/// * `KiltWeb3Name`: The runtime type of a web3name as defined by the KILT
	///   child parachain.
	/// * `KiltLinkableAccountId`: The runtime type of a linked account ID as
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
	pub struct RelaychainVerifier<
		ConsumerBlockHashStore,
		const KILT_PARA_ID: u32,
		KiltRuntime,
		DidCallVerifier,
		SignedExtra,
		const MAX_LEAVES_REVEALED: u32,
	>(
		#[allow(clippy::type_complexity)]
		PhantomData<(ConsumerBlockHashStore, KiltRuntime, DidCallVerifier, SignedExtra)>,
	);

	impl<
			ConsumerRuntime,
			ConsumerBlockHashStore,
			const KILT_PARA_ID: u32,
			KiltRuntime,
			DidCallVerifier,
			SignedExtra,
			const MAX_LEAVES_REVEALED: u32,
		> IdentityProofVerifier<ConsumerRuntime>
		for RelaychainVerifier<
			ConsumerBlockHashStore,
			KILT_PARA_ID,
			KiltRuntime,
			DidCallVerifier,
			SignedExtra,
			MAX_LEAVES_REVEALED,
		> where
		ConsumerRuntime: pallet_dip_consumer::Config<Identifier = KiltRuntime::Identifier>,
		ConsumerRuntime::LocalIdentityInfo: Incrementable + Default,
		BlockNumberFor<ConsumerRuntime>: Into<U256> + TryFrom<U256>,
		ConsumerBlockHashStore:
			GetWithArg<BlockNumberFor<ConsumerRuntime>, Result = Option<OutputOf<ConsumerRuntime::Hashing>>>,
		KiltRuntime: pallet_dip_provider::Config + did::Config + pallet_web3_names::Config + pallet_did_lookup::Config,
		IdentityCommitmentOf<KiltRuntime>: Into<KiltRuntime::Hash>,
		SignedExtra: GetWithoutArg,
		SignedExtra::Result: Encode,
		DidCallVerifier: DipCallOriginFilter<
			RuntimeCallOf<ConsumerRuntime>,
			OriginInfo = RevealedDidKey<KeyIdOf<KiltRuntime>, BlockNumberFor<KiltRuntime>, KiltRuntime::AccountId>,
		>,
		DidCallVerifier::Error: Into<u8>,
	{
		type Error = DipRelaychainStateProofVerifierError<DidCallVerifier::Error>;
		type Proof = RelayDipDidProof<
			BlockNumberFor<ConsumerRuntime>,
			ConsumerRuntime::Hashing,
			KeyIdOf<KiltRuntime>,
			KiltRuntime::AccountId,
			BlockNumberFor<KiltRuntime>,
			Web3NameOf<KiltRuntime>,
			LinkableAccountId,
		>;
		type VerificationResult = DipVerifiedInfo<
			KeyIdOf<KiltRuntime>,
			KiltRuntime::AccountId,
			BlockNumberFor<KiltRuntime>,
			Web3NameOf<KiltRuntime>,
			LinkableAccountId,
		>;

		fn verify_proof_for_call_against_details(
			call: &RuntimeCallOf<ConsumerRuntime>,
			subject: &ConsumerRuntime::Identifier,
			submitter: &ConsumerRuntime::AccountId,
			identity_details: &mut Option<ConsumerRuntime::LocalIdentityInfo>,
			proof: Self::Proof,
		) -> Result<Self::VerificationResult, Self::Error> {
			// 1. Verify provided relaychain header.
			let proof_without_header = proof
				.verify_relay_header::<ConsumerBlockHashStore>()
				.map_err(DipRelaychainStateProofVerifierError::ProofVerification)?;

			// 2. Verify parachain state is finalized by relay chain and fresh.
			let proof_without_relaychain = proof_without_header
				.verify_provider_head_proof::<HeaderFor<KiltRuntime>>(KILT_PARA_ID)
				.map_err(DipRelaychainStateProofVerifierError::ProofVerification)?;

			// 3. Verify commitment is included in provider parachain state.
			let proof_without_parachain = proof_without_relaychain
				.verify_dip_commitment_proof_for_subject::<KiltRuntime::Hashing, KiltRuntime, _>(subject)
				.map_err(DipRelaychainStateProofVerifierError::ProofVerification)?;

			// 4. Verify DIP Merkle proof.
			let proof_without_dip_merkle = proof_without_parachain
				.verify_dip_proof::<KiltRuntime::Hashing>()
				.map_err(DipRelaychainStateProofVerifierError::ProofVerification)?;

			// 5. Verify call is signed by one of the DID keys revealed in the proof
			let current_block_number = frame_system::Pallet::<ConsumerRuntime>::block_number();
			let consumer_genesis_hash =
				frame_system::Pallet::<ConsumerRuntime>::block_hash(BlockNumberFor::<ConsumerRuntime>::zero());
			let signed_extra = SignedExtra::get();
			let encoded_payload = (call, &identity_details, submitter, consumer_genesis_hash, signed_extra).encode();
			let revealed_did_info = proof_without_dip_merkle
				.verify_signature_time(&current_block_number)
				.and_then(|p| p.retrieve_signing_leaf_for_payload(&encoded_payload[..]))
				.map_err(DipRelaychainStateProofVerifierError::ProofVerification)?;

			// 6. Verify the signing key fulfills the requirements
			let signing_key = revealed_did_info
				.get_signing_leaf()
				.map_err(DipRelaychainStateProofVerifierError::ProofVerification)?;
			DidCallVerifier::check_call_origin_info(call, signing_key)
				.map_err(DipRelaychainStateProofVerifierError::DidOriginError)?;

			// 7. Increment the local details
			if let Some(details) = identity_details {
				details.increment();
			} else {
				*identity_details = Some(Default::default());
			};

			Ok(revealed_did_info)
		}
	}
}
