// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2024 BOTLabs GmbH

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
use frame_system::pallet_prelude::{BlockNumberFor, HeaderFor};
use pallet_did_lookup::linkable_account::LinkableAccountId;
use pallet_dip_consumer::{traits::IdentityProofVerifier, RuntimeCallOf};
use pallet_dip_provider::{traits::IdentityCommitmentGenerator, IdentityCommitmentOf};
use pallet_web3_names::Web3NameOf;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_core::{RuntimeDebug, U256};
use sp_runtime::traits::Hash;
use sp_std::marker::PhantomData;

use crate::{
	merkle::v0::RevealedDidKey,
	traits::{BenchmarkDefault, DipCallOriginFilter, GetWithArg, GetWithoutArg, Incrementable},
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
	ProofComponentTooLarge(u8),
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
			DipRelaychainStateProofVerifierError::ProofComponentTooLarge(component_id) => {
				u8::MAX as u16 + component_id as u16
			}
			DipRelaychainStateProofVerifierError::ProofVerification(error) => {
				u8::MAX as u16 * 2 + u8::from(error) as u16
			}
			DipRelaychainStateProofVerifierError::DidOriginError(error) => u8::MAX as u16 * 3 + error.into() as u16,
			DipRelaychainStateProofVerifierError::Internal => u16::MAX,
		}
	}
}

/// Versioned proof verifier. For version-specific description, refer to each
/// verifier's documentation.
pub struct KiltVersionedRelaychainVerifier<
	ConsumerBlockHashStore,
	const KILT_PARA_ID: u32,
	KiltRuntime,
	DidCallVerifier,
	SignedExtra = (),
	const MAX_PROVIDER_HEAD_PROOF_LEAVE_COUNT: u32 = 10,
	const MAX_PROVIDER_HEAD_PROOF_LEAVE_SIZE: u32 = 128,
	const MAX_DIP_COMMITMENT_PROOF_LEAVE_COUNT: u32 = 10,
	const MAX_DIP_COMMITMENT_PROOF_LEAVE_SIZE: u32 = 128,
	const MAX_DID_MERKLE_PROOF_LEAVE_COUNT: u32 = 10,
	const MAX_DID_MERKLE_PROOF_LEAVE_SIZE: u32 = 128,
	const MAX_DID_MERKLE_LEAVES_REVEALED: u32 = 10,
>(#[allow(clippy::type_complexity)] PhantomData<(ConsumerBlockHashStore, KiltRuntime, DidCallVerifier, SignedExtra)>);

impl<
		ConsumerRuntime,
		ConsumerBlockHashStore,
		const KILT_PARA_ID: u32,
		KiltRuntime,
		DidCallVerifier,
		SignedExtra,
		const MAX_PROVIDER_HEAD_PROOF_LEAVE_COUNT: u32,
		const MAX_PROVIDER_HEAD_PROOF_LEAVE_SIZE: u32,
		const MAX_DIP_COMMITMENT_PROOF_LEAVE_COUNT: u32,
		const MAX_DIP_COMMITMENT_PROOF_LEAVE_SIZE: u32,
		const MAX_DID_MERKLE_PROOF_LEAVE_COUNT: u32,
		const MAX_DID_MERKLE_PROOF_LEAVE_SIZE: u32,
		const MAX_DID_MERKLE_LEAVES_REVEALED: u32,
	> IdentityProofVerifier<ConsumerRuntime>
	for KiltVersionedRelaychainVerifier<
		ConsumerBlockHashStore,
		KILT_PARA_ID,
		KiltRuntime,
		DidCallVerifier,
		SignedExtra,
		MAX_PROVIDER_HEAD_PROOF_LEAVE_COUNT,
		MAX_PROVIDER_HEAD_PROOF_LEAVE_SIZE,
		MAX_DIP_COMMITMENT_PROOF_LEAVE_COUNT,
		MAX_DIP_COMMITMENT_PROOF_LEAVE_SIZE,
		MAX_DID_MERKLE_PROOF_LEAVE_COUNT,
		MAX_DID_MERKLE_PROOF_LEAVE_SIZE,
		MAX_DID_MERKLE_LEAVES_REVEALED,
	> where
	ConsumerRuntime: pallet_dip_consumer::Config<Identifier = KiltRuntime::Identifier>,
	ConsumerRuntime::LocalIdentityInfo: Incrementable + Default,
	BlockNumberFor<ConsumerRuntime>: Into<U256> + TryFrom<U256>,
	ConsumerBlockHashStore:
		GetWithArg<BlockNumberFor<ConsumerRuntime>, Result = Option<OutputOf<ConsumerRuntime::Hashing>>>,
	KiltRuntime: frame_system::Config<Hash = ConsumerRuntime::Hash>
		+ pallet_dip_provider::Config
		+ did::Config
		+ pallet_web3_names::Config
		+ pallet_did_lookup::Config,
	KiltRuntime::IdentityCommitmentGenerator: IdentityCommitmentGenerator<KiltRuntime, Output = ConsumerRuntime::Hash>,
	HeaderFor<KiltRuntime>: BenchmarkDefault,
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
		MAX_DID_MERKLE_LEAVES_REVEALED,
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
				MAX_PROVIDER_HEAD_PROOF_LEAVE_COUNT,
				MAX_PROVIDER_HEAD_PROOF_LEAVE_SIZE,
				MAX_DIP_COMMITMENT_PROOF_LEAVE_COUNT,
				MAX_DIP_COMMITMENT_PROOF_LEAVE_SIZE,
				MAX_DID_MERKLE_PROOF_LEAVE_COUNT,
				MAX_DID_MERKLE_PROOF_LEAVE_SIZE,
				MAX_DID_MERKLE_LEAVES_REVEALED,
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

	/// Proof verifier configured given a specific KILT runtime implementation.
	///
	/// The generic types are the following:
	///
	/// * `ConsumerBlockHashStore`: A type providing block hashes for the
	///   relaychain blocks.
	/// * `KILT_PARA_ID`: The ID of the specific KILT parachain instance.
	/// * `KiltRuntime`: A KILT runtime definition.
	/// * `DidCallVerifier`: Logic to map `RuntimeCall`s to a specific DID key
	///   relationship. This information is used once the Merkle proof is
	///   verified, to filter only the revealed keys that match the provided
	///   relationship.
	/// * `SignedExtra`: Any additional information that must be signed by the
	///   DID subject in the cross-chain operation.
	/// * `MAX_PROVIDER_HEAD_PROOF_LEAVE_COUNT`: The maximum number of leaves
	///   that can be revealed as part of the parachain head storage proof.
	/// * `MAX_PROVIDER_HEAD_PROOF_LEAVE_SIZE`: The maximum size of each leaf
	///   revealed as part of the parachain head storage proof.
	/// * `MAX_DIP_COMMITMENT_PROOF_LEAVE_COUNT`: The maximum number of leaves
	///   that can be revealed as part of the DIP commitment storage proof.
	/// * `MAX_DIP_COMMITMENT_PROOF_LEAVE_SIZE`: The maximum size of each leaf
	///   revealed as part of the DIP commitment storage proof.
	/// * `MAX_DID_MERKLE_PROOF_LEAVE_COUNT`: The maximum number of *blinded*
	///   leaves that can be revealed as part of the DID Merkle proof.
	/// * `MAX_DID_MERKLE_PROOF_LEAVE_SIZE`: The maximum size of each *blinded*
	///   leaf revealed as part of the DID Merkle proof.
	/// * `MAX_DID_MERKLE_PROOF_LEAVE_SIZE`: The maximum number of leaves that
	///   can be revealed as part of the DID Merkle proof.
	pub struct RelaychainVerifier<
		ConsumerBlockHashStore,
		const KILT_PARA_ID: u32,
		KiltRuntime,
		DidCallVerifier,
		SignedExtra,
		const MAX_PROVIDER_HEAD_PROOF_LEAVE_COUNT: u32,
		const MAX_PROVIDER_HEAD_PROOF_LEAVE_SIZE: u32,
		const MAX_DIP_COMMITMENT_PROOF_LEAVE_COUNT: u32,
		const MAX_DIP_COMMITMENT_PROOF_LEAVE_SIZE: u32,
		const MAX_DID_MERKLE_PROOF_LEAVE_COUNT: u32,
		const MAX_DID_MERKLE_PROOF_LEAVE_SIZE: u32,
		const MAX_DID_MERKLE_LEAVES_REVEALED: u32,
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
			const MAX_PROVIDER_HEAD_PROOF_LEAVE_COUNT: u32,
			const MAX_PROVIDER_HEAD_PROOF_LEAVE_SIZE: u32,
			const MAX_DIP_COMMITMENT_PROOF_LEAVE_COUNT: u32,
			const MAX_DIP_COMMITMENT_PROOF_LEAVE_SIZE: u32,
			const MAX_DID_MERKLE_PROOF_LEAVE_COUNT: u32,
			const MAX_DID_MERKLE_PROOF_LEAVE_SIZE: u32,
			const MAX_DID_MERKLE_LEAVES_REVEALED: u32,
		> IdentityProofVerifier<ConsumerRuntime>
		for RelaychainVerifier<
			ConsumerBlockHashStore,
			KILT_PARA_ID,
			KiltRuntime,
			DidCallVerifier,
			SignedExtra,
			MAX_PROVIDER_HEAD_PROOF_LEAVE_COUNT,
			MAX_PROVIDER_HEAD_PROOF_LEAVE_SIZE,
			MAX_DIP_COMMITMENT_PROOF_LEAVE_COUNT,
			MAX_DIP_COMMITMENT_PROOF_LEAVE_SIZE,
			MAX_DID_MERKLE_PROOF_LEAVE_COUNT,
			MAX_DID_MERKLE_PROOF_LEAVE_SIZE,
			MAX_DID_MERKLE_LEAVES_REVEALED,
		> where
		ConsumerRuntime: pallet_dip_consumer::Config<Identifier = KiltRuntime::Identifier>,
		ConsumerRuntime::LocalIdentityInfo: Incrementable + Default,
		BlockNumberFor<ConsumerRuntime>: Into<U256> + TryFrom<U256>,
		ConsumerBlockHashStore:
			GetWithArg<BlockNumberFor<ConsumerRuntime>, Result = Option<OutputOf<ConsumerRuntime::Hashing>>>,
		KiltRuntime: frame_system::Config<Hash = ConsumerRuntime::Hash>
			+ pallet_dip_provider::Config
			+ did::Config
			+ pallet_web3_names::Config
			+ pallet_did_lookup::Config,
		KiltRuntime::IdentityCommitmentGenerator:
			IdentityCommitmentGenerator<KiltRuntime, Output = ConsumerRuntime::Hash>,
		HeaderFor<KiltRuntime>: BenchmarkDefault,
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
			MAX_DID_MERKLE_LEAVES_REVEALED,
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
				.verify_provider_head_proof::<ConsumerRuntime::Hashing, HeaderFor<KiltRuntime>>(KILT_PARA_ID)
				.map_err(DipRelaychainStateProofVerifierError::ProofVerification)?;

			// 3. Verify commitment is included in provider parachain state.
			let proof_without_parachain = proof_without_relaychain
				.verify_dip_commitment_proof_for_subject::<KiltRuntime::Hashing, KiltRuntime>(subject)
				.map_err(DipRelaychainStateProofVerifierError::ProofVerification)?;

			// 4. Verify DIP Merkle proof.
			let proof_without_dip_merkle = proof_without_parachain
				.verify_dip_proof::<KiltRuntime::Hashing, MAX_DID_MERKLE_LEAVES_REVEALED>()
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
