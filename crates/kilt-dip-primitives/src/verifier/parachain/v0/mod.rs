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
use pallet_dip_provider::traits::IdentityCommitmentGenerator;
use pallet_web3_names::Web3NameOf;
use parity_scale_codec::Encode;
use sp_runtime::{traits::Zero, SaturatedConversion};
use sp_std::{fmt::Debug, marker::PhantomData, vec::Vec};

use crate::{
	merkle_proofs::v0::ParachainDipDidProof,
	traits::{DipCallOriginFilter, GetWithArg, GetWithoutArg, Incrementable},
	utils::OutputOf,
	verifier::errors::DipProofComponentTooLargeError,
	DipOriginInfo, DipParachainStateProofVerifierError, RevealedDidKey,
};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

const LOG_TARGET: &str = "dip::consumer::ParachainVerifierV0";

/// Proof verifier configured given a specific KILT runtime implementation.
///
/// The generic types
/// indicate the following:
/// * `RelaychainRuntime`: The relaychain runtime definition.
/// * `RelaychainStateRootStore`: A type providing state roots for relaychain
///   blocks.
/// * `KILT_PARA_ID`: The ID of the specific KILT parachain instance.
/// * `KiltRuntime`: A KILT runtime definition.
/// * `DidCallVerifier`: Logic to map `RuntimeCall`s to a specific DID key
///   relationship. This information is used once the Merkle proof is verified,
///   to filter only the revealed keys that match the provided relationship.
/// * `SignedExtra`: Any additional information that must be signed by the DID
///   subject in the cross-chain operation.
/// * `MAX_PROVIDER_HEAD_PROOF_LEAVE_COUNT`: The maximum number of leaves that
///   can be revealed as part of the parachain head storage proof.
/// * `MAX_PROVIDER_HEAD_PROOF_LEAVE_SIZE`: The maximum size of each leaf
///   revealed as part of the parachain head storage proof.
/// * `MAX_DIP_COMMITMENT_PROOF_LEAVE_COUNT`: The maximum number of leaves that
///   can be revealed as part of the DIP commitment storage proof.
/// * `MAX_DIP_COMMITMENT_PROOF_LEAVE_SIZE`: The maximum size of each leaf
///   revealed as part of the DIP commitment storage proof.
/// * `MAX_DID_MERKLE_PROOF_LEAVE_COUNT`: The maximum number of *blinded* leaves
///   that can be revealed as part of the DID Merkle proof.
/// * `MAX_DID_MERKLE_PROOF_LEAVE_SIZE`: The maximum size of each *blinded* leaf
///   revealed as part of the DID Merkle proof.
/// * `MAX_DID_MERKLE_LEAVES_REVEALED`: The maximum number of leaves that can be
///   revealed as part of the DID Merkle proof.
pub struct ParachainVerifier<
	RelaychainRuntime,
	RelaychainStateRootStore,
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
	PhantomData<(
		RelaychainRuntime,
		RelaychainStateRootStore,
		KiltRuntime,
		DidCallVerifier,
		SignedExtra,
	)>,
);

impl<
		ConsumerRuntime,
		RelaychainRuntime,
		RelaychainStateRootStore,
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
	for ParachainVerifier<
		RelaychainRuntime,
		RelaychainStateRootStore,
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
	RelaychainRuntime: frame_system::Config,
	RelaychainStateRootStore:
		GetWithArg<BlockNumberFor<RelaychainRuntime>, Result = Option<OutputOf<RelaychainRuntime::Hashing>>>,
	KiltRuntime: frame_system::Config<Hash = RelaychainRuntime::Hash>
		+ pallet_dip_provider::Config
		+ did::Config
		+ pallet_web3_names::Config
		+ pallet_did_lookup::Config,
	KiltRuntime::IdentityCommitmentGenerator:
		IdentityCommitmentGenerator<KiltRuntime, Output = RelaychainRuntime::Hash>,
	SignedExtra: GetWithoutArg,
	SignedExtra::Result: Encode + Debug,
	DidCallVerifier: DipCallOriginFilter<
		RuntimeCallOf<ConsumerRuntime>,
		OriginInfo = Vec<RevealedDidKey<KeyIdOf<KiltRuntime>, BlockNumberFor<KiltRuntime>, KiltRuntime::AccountId>>,
	>,
	DidCallVerifier::Error: Into<u8> + Debug,
{
	type Error = DipParachainStateProofVerifierError<DidCallVerifier::Error>;
	type Proof = ParachainDipDidProof<
		BlockNumberFor<RelaychainRuntime>,
		KeyIdOf<KiltRuntime>,
		KiltRuntime::AccountId,
		BlockNumberFor<KiltRuntime>,
		Web3NameOf<KiltRuntime>,
		LinkableAccountId,
		BlockNumberFor<ConsumerRuntime>,
	>;

	type VerificationResult = DipOriginInfo<
		KeyIdOf<KiltRuntime>,
		KiltRuntime::AccountId,
		BlockNumberFor<KiltRuntime>,
		Web3NameOf<KiltRuntime>,
		LinkableAccountId,
		MAX_DID_MERKLE_LEAVES_REVEALED,
	>;

	fn verify_proof_for_call_against_details(
		call: &RuntimeCallOf<ConsumerRuntime>,
		subject: &<ConsumerRuntime as pallet_dip_consumer::Config>::Identifier,
		submitter: &<ConsumerRuntime>::AccountId,
		identity_details: &mut Option<<ConsumerRuntime as pallet_dip_consumer::Config>::LocalIdentityInfo>,
		proof: Self::Proof,
	) -> Result<Self::VerificationResult, Self::Error> {
		// 1. Verify parachain state is finalized by relay chain and fresh.
		if proof.provider_head_proof.proof.len() > MAX_PROVIDER_HEAD_PROOF_LEAVE_COUNT.saturated_into() {
			let inner_error = DipProofComponentTooLargeError::ParachainHeadProofTooManyLeaves;
			log::info!(
				target: LOG_TARGET,
				"Failed to verify DIP proof with error {:#?}",
				inner_error
			);
			return Err(DipParachainStateProofVerifierError::ProofComponentTooLarge(
				inner_error as u8,
			));
		}

		if proof
			.provider_head_proof
			.proof
			.iter()
			.any(|l| l.len() > MAX_PROVIDER_HEAD_PROOF_LEAVE_SIZE.saturated_into())
		{
			let inner_error = DipProofComponentTooLargeError::ParachainHeadProofLeafTooLarge;
			log::info!(
				target: LOG_TARGET,
				"Failed to verify DIP proof with error {:#?}",
				inner_error
			);
			return Err(DipParachainStateProofVerifierError::ProofComponentTooLarge(
				inner_error as u8,
			));
		}

		let proof_without_relaychain = proof
			.verify_provider_head_proof::<RelaychainRuntime::Hashing, RelaychainStateRootStore, HeaderFor<KiltRuntime>>(
				KILT_PARA_ID,
			)
			.map_err(|e| {
				log::info!(target: LOG_TARGET, "Failed to verify DIP proof with error {:#?}", e);
				DipParachainStateProofVerifierError::ProofVerification(e)
			})?;
		log::info!(
			target: LOG_TARGET,
			"Verified parachain state root: {:#?}",
			proof_without_relaychain.state_root
		);

		// 2. Verify commitment is included in provider parachain state.
		if proof_without_relaychain.dip_commitment_proof.0.len() > MAX_DIP_COMMITMENT_PROOF_LEAVE_COUNT.saturated_into()
		{
			let inner_error = DipProofComponentTooLargeError::DipCommitmentProofTooManyLeaves;
			log::info!(
				target: LOG_TARGET,
				"Failed to verify DIP proof with error {:#?}",
				inner_error
			);
			return Err(DipParachainStateProofVerifierError::ProofComponentTooLarge(
				inner_error as u8,
			));
		}

		if proof_without_relaychain
			.dip_commitment_proof
			.0
			.iter()
			.any(|l| l.len() > MAX_DIP_COMMITMENT_PROOF_LEAVE_SIZE.saturated_into())
		{
			let inner_error = DipProofComponentTooLargeError::DipCommitmentProofLeafTooLarge;
			log::info!(
				target: LOG_TARGET,
				"Failed to verify DIP proof with error {:#?}",
				inner_error
			);
			return Err(DipParachainStateProofVerifierError::ProofComponentTooLarge(
				inner_error as u8,
			));
		}

		let proof_without_parachain = proof_without_relaychain
			.verify_dip_commitment_proof_for_subject::<KiltRuntime::Hashing, KiltRuntime>(subject)
			.map_err(DipParachainStateProofVerifierError::ProofVerification)?;
		log::info!(
			target: LOG_TARGET,
			"Verified subject DIP commitment: {:#?}",
			proof_without_parachain.dip_commitment
		);

		// 3. Verify DIP Merkle proof.
		if proof_without_parachain.dip_proof.blinded.len() > MAX_DID_MERKLE_PROOF_LEAVE_COUNT.saturated_into() {
			let inner_error = DipProofComponentTooLargeError::DipProofTooManyLeaves;
			log::info!(
				target: LOG_TARGET,
				"Failed to verify DIP proof with error {:#?}",
				inner_error
			);
			return Err(DipParachainStateProofVerifierError::ProofComponentTooLarge(
				inner_error as u8,
			));
		}

		if proof_without_parachain
			.dip_proof
			.blinded
			.iter()
			.any(|l| l.len() > MAX_DID_MERKLE_PROOF_LEAVE_SIZE.saturated_into())
		{
			let inner_error = DipProofComponentTooLargeError::DipProofLeafTooLarge;
			log::info!(
				target: LOG_TARGET,
				"Failed to verify DIP proof with error {:#?}",
				inner_error
			);
			return Err(DipParachainStateProofVerifierError::ProofComponentTooLarge(
				inner_error as u8,
			));
		}

		let proof_without_dip_merkle = proof_without_parachain
			.verify_dip_proof::<KiltRuntime::Hashing, MAX_DID_MERKLE_LEAVES_REVEALED>()
			.map_err(|e| {
				log::info!(target: LOG_TARGET, "Failed to verify DIP proof with error {:#?}", e);
				DipParachainStateProofVerifierError::ProofVerification(e)
			})?;
		log::info!(
			target: LOG_TARGET,
			"Verified DID Merkle leaves: {:#?}",
			proof_without_dip_merkle.revealed_leaves
		);

		// 4. Verify call is signed by one of the DID keys revealed in the proof
		let current_block_number = frame_system::Pallet::<ConsumerRuntime>::block_number();
		let consumer_genesis_hash =
			frame_system::Pallet::<ConsumerRuntime>::block_hash(BlockNumberFor::<ConsumerRuntime>::zero());
		let signed_extra = SignedExtra::get();
		log::trace!(target: LOG_TARGET, "Additional components for signature verification: current block number = {:#?}, genesis hash = {:#?}, signed extra = {:#?}", current_block_number, consumer_genesis_hash, signed_extra);
		let encoded_payload = (
			call,
			&identity_details,
			submitter,
			proof_without_dip_merkle.signature.valid_until,
			consumer_genesis_hash,
			signed_extra,
		)
			.encode();
		log::trace!(target: LOG_TARGET, "Encoded final payload: {:#?}", encoded_payload);

		let revealed_did_info = proof_without_dip_merkle
			.verify_signature_time(&current_block_number)
			.and_then(|p| p.retrieve_signing_leaves_for_payload(&encoded_payload[..]))
			.map_err(|e| {
				log::info!(target: LOG_TARGET, "Failed to verify DIP proof with error {:#?}", e);
				DipParachainStateProofVerifierError::ProofVerification(e)
			})?;

		// 5. Verify the signing key fulfills the requirements
		let signing_keys = revealed_did_info.get_signing_leaves().map_err(|e| {
			log::info!(target: LOG_TARGET, "Failed to verify DIP proof with error {:#?}", e);
			DipParachainStateProofVerifierError::ProofVerification(e)
		})?;
		DidCallVerifier::check_call_origin_info(call, &signing_keys.cloned().collect::<Vec<_>>()).map_err(|e| {
			log::info!(target: LOG_TARGET, "Failed to verify DIP proof with error {:#?}", e);
			DipParachainStateProofVerifierError::DidOriginError(e)
		})?;

		// 6. Increment the local details
		if let Some(details) = identity_details {
			details.increment();
		} else {
			let default_details = Default::default();
			log::trace!(
				target: LOG_TARGET,
				"No details present for subject {:#?}. Setting default ones: {:#?}.",
				subject,
				default_details
			);
			*identity_details = Some(default_details);
		};

		Ok(revealed_did_info)
	}
}
