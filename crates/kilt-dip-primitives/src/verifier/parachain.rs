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
use sp_core::RuntimeDebug;
use sp_std::marker::PhantomData;

use crate::{
	merkle::v0::RevealedDidKey,
	traits::{DipCallOriginFilter, GetWithArg, GetWithoutArg, Incrementable},
	utils::OutputOf,
	DipSignatureVerifiedInfo, Error,
};

/// A KILT-specific DIP identity proof for a sibling consumer that supports
/// versioning.
///
/// For more info, refer to the version-specific proofs.
#[derive(Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo, Clone)]
pub enum VersionedDipParachainStateProof<
	RelayBlockNumber,
	KiltDidKeyId,
	KiltAccountId,
	KiltWeb3Name,
	KiltBlockNumber,
	KiltLinkableAccountId,
	ConsumerBlockNumber,
> {
	V0(
		crate::merkle::v0::ParachainDipDidProof<
			RelayBlockNumber,
			KiltDidKeyId,
			KiltAccountId,
			KiltWeb3Name,
			KiltBlockNumber,
			KiltLinkableAccountId,
			ConsumerBlockNumber,
		>,
	),
}

pub enum DipParachainStateProofVerifierError<DidOriginError> {
	UnsupportedVersion,
	ProofVerification(Error),
	DidOriginError(DidOriginError),
	Internal,
}

impl<DidOriginError> From<DipParachainStateProofVerifierError<DidOriginError>> for u16
where
	DidOriginError: Into<u8>,
{
	fn from(value: DipParachainStateProofVerifierError<DidOriginError>) -> Self {
		match value {
			DipParachainStateProofVerifierError::UnsupportedVersion => 1,
			DipParachainStateProofVerifierError::ProofVerification(error) => u8::MAX as u16 + u8::from(error) as u16,
			DipParachainStateProofVerifierError::DidOriginError(error) => u8::MAX as u16 * 2 + error.into() as u16,
			DipParachainStateProofVerifierError::Internal => u16::MAX,
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
/// * `KiltAccountId`: The `KiltRuntime::AccountId` type.
/// * `KiltWeb3Name`: The `KiltRuntime::Web3Name` type.
/// * `KiltLinkableAccountId`: The [`LinkableAccountId`] type.
/// * `MAX_REVEALED_KEYS_COUNT`: The provided `MAX_REVEALED_KEYS_COUNT`.
/// * `MAX_REVEALED_ACCOUNTS_COUNT`: The provided `MAX_REVEALED_ACCOUNTS_COUNT`.
/// * `LocalContextProvider`: The [`FrameSystemDidSignatureContext`] type
///   configured with the provided `KiltRuntime` and
///   `MAX_DID_SIGNATURE_DURATION`.
/// * `LocalDidCallVerifier`: The provided `LocalDidCallVerifier`.
pub struct KiltVersionedParachainVerifier<
	RelaychainRuntime,
	RelaychainStateRootStore,
	const KILT_PARA_ID: u32,
	KiltRuntime,
	DidCallVerifier,
	SignedExtra = (),
	const MAX_LEAVES_REVEALED: u32 = 50,
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
		const MAX_LEAVES_REVEALED: u32,
	> IdentityProofVerifier<ConsumerRuntime>
	for KiltVersionedParachainVerifier<
		RelaychainRuntime,
		RelaychainStateRootStore,
		KILT_PARA_ID,
		KiltRuntime,
		DidCallVerifier,
		SignedExtra,
		MAX_LEAVES_REVEALED,
	> where
	ConsumerRuntime: pallet_dip_consumer::Config<Identifier = KiltRuntime::Identifier>,
	ConsumerRuntime::LocalIdentityInfo: Incrementable + Default,
	RelaychainRuntime: frame_system::Config,
	RelaychainStateRootStore:
		GetWithArg<BlockNumberFor<RelaychainRuntime>, Result = Option<OutputOf<RelaychainRuntime::Hashing>>>,
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
	type Error = DipParachainStateProofVerifierError<DidCallVerifier::Error>;
	type Proof = VersionedDipParachainStateProof<
		BlockNumberFor<RelaychainRuntime>,
		KeyIdOf<KiltRuntime>,
		KiltRuntime::AccountId,
		BlockNumberFor<KiltRuntime>,
		Web3NameOf<KiltRuntime>,
		LinkableAccountId,
		BlockNumberFor<ConsumerRuntime>,
	>;
	type VerificationResult = DipSignatureVerifiedInfo<
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
			VersionedDipParachainStateProof::V0(v0_proof) => <v0::ParachainVerifier<
				RelaychainRuntime,
				RelaychainStateRootStore,
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

	use crate::merkle::v0::{DipSignatureVerifiedInfo, ParachainDipDidProof};

	pub struct ParachainVerifier<
		RelaychainRuntime,
		RelaychainStateRootStore,
		const KILT_PARA_ID: u32,
		KiltRuntime,
		DidCallVerifier,
		SignedExtra,
		const MAX_LEAVES_REVEALED: u32,
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
			const MAX_LEAVES_REVEALED: u32,
		> IdentityProofVerifier<ConsumerRuntime>
		for ParachainVerifier<
			RelaychainRuntime,
			RelaychainStateRootStore,
			KILT_PARA_ID,
			KiltRuntime,
			DidCallVerifier,
			SignedExtra,
			MAX_LEAVES_REVEALED,
		> where
		ConsumerRuntime: pallet_dip_consumer::Config<Identifier = KiltRuntime::Identifier>,
		ConsumerRuntime::LocalIdentityInfo: Incrementable + Default,
		RelaychainRuntime: frame_system::Config,
		RelaychainStateRootStore:
			GetWithArg<BlockNumberFor<RelaychainRuntime>, Result = Option<OutputOf<RelaychainRuntime::Hashing>>>,
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

		type VerificationResult = DipSignatureVerifiedInfo<
			KeyIdOf<KiltRuntime>,
			KiltRuntime::AccountId,
			BlockNumberFor<KiltRuntime>,
			Web3NameOf<KiltRuntime>,
			LinkableAccountId,
		>;

		fn verify_proof_for_call_against_details(
			call: &RuntimeCallOf<ConsumerRuntime>,
			subject: &<ConsumerRuntime as pallet_dip_consumer::Config>::Identifier,
			submitter: &<ConsumerRuntime>::AccountId,
			identity_details: &mut Option<<ConsumerRuntime as pallet_dip_consumer::Config>::LocalIdentityInfo>,
			proof: Self::Proof,
		) -> Result<Self::VerificationResult, Self::Error> {
			// 1. Verify parachain state is finalized by relay chain and fresh.
			let proof_without_relaychain = proof
				.verify_provider_head_proof::<RelaychainRuntime::Hashing, RelaychainStateRootStore, HeaderFor<KiltRuntime>>(
					KILT_PARA_ID,
				)
				.map_err(DipParachainStateProofVerifierError::ProofVerification)?;

			// 2. Verify commitment is included in provider parachain state.
			let proof_without_parachain = proof_without_relaychain
				.verify_dip_commitment_proof_for_subject::<KiltRuntime::Hashing, KiltRuntime, _>(subject)
				.map_err(DipParachainStateProofVerifierError::ProofVerification)?;

			// 3. Verify DIP Merkle proof.
			let proof_without_dip_merkle = proof_without_parachain
				.verify_dip_proof::<KiltRuntime::Hashing>()
				.map_err(DipParachainStateProofVerifierError::ProofVerification)?;

			// 4. Verify call is signed by one of the DID keys revealed in the proof
			let current_block_number = frame_system::Pallet::<ConsumerRuntime>::block_number();
			let consumer_genesis_hash =
				frame_system::Pallet::<ConsumerRuntime>::block_hash(BlockNumberFor::<ConsumerRuntime>::zero());
			let signed_extra = SignedExtra::get();
			let encoded_payload = (call, &identity_details, submitter, consumer_genesis_hash, signed_extra).encode();
			let revealed_did_info = proof_without_dip_merkle
				.verify_signature_time(&current_block_number)
				.and_then(|p| p.retrieve_signing_leaf_for_payload(&encoded_payload[..]))
				.map_err(DipParachainStateProofVerifierError::ProofVerification)?;

			// 5. Verify the signing key fulfills the requirements
			let signing_key = revealed_did_info.get_signing_leaf();
			DidCallVerifier::check_call_origin_info(call, signing_key)
				.map_err(DipParachainStateProofVerifierError::DidOriginError)?;

			// 6. Increment the local details
			if let Some(details) = identity_details {
				details.increment();
			} else {
				*identity_details = Some(Default::default());
			};

			Ok(revealed_did_info)
		}
	}
}
