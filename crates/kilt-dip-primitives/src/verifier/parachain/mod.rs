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
use frame_system::pallet_prelude::BlockNumberFor;
use pallet_did_lookup::linkable_account::LinkableAccountId;
use pallet_dip_consumer::{traits::IdentityProofVerifier, RuntimeCallOf};
use pallet_dip_provider::traits::IdentityCommitmentGenerator;
use pallet_web3_names::Web3NameOf;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::{fmt::Debug, marker::PhantomData, vec::Vec};

use crate::{
	merkle_proofs::v0::RevealedDidKey,
	traits::{DipCallOriginFilter, GetWithArg, GetWithoutArg, Incrementable},
	utils::OutputOf,
	DipOriginInfo, ParachainDipDidProof,
};

pub mod v0;

mod error;
pub use error::*;

/// A KILT-specific DIP identity proof for a sibling consumer that supports
/// versioning.
///
/// For more info, refer to the version-specific proofs.
#[derive(Encode, Decode, PartialEq, Eq, Debug, TypeInfo, Clone)]
pub enum VersionedDipParachainStateProof<
	RelayBlockNumber,
	KiltDidKeyId,
	KiltAccountId,
	KiltBlockNumber,
	KiltWeb3Name,
	KiltLinkableAccountId,
	ConsumerBlockNumber,
> {
	V0(
		ParachainDipDidProof<
			RelayBlockNumber,
			KiltDidKeyId,
			KiltAccountId,
			KiltBlockNumber,
			KiltWeb3Name,
			KiltLinkableAccountId,
			ConsumerBlockNumber,
		>,
	),
}

impl<
		RelayBlockNumber,
		KiltDidKeyId,
		KiltAccountId,
		KiltBlockNumber,
		KiltWeb3Name,
		KiltLinkableAccountId,
		ConsumerBlockNumber,
	>
	From<
		ParachainDipDidProof<
			RelayBlockNumber,
			KiltDidKeyId,
			KiltAccountId,
			KiltBlockNumber,
			KiltWeb3Name,
			KiltLinkableAccountId,
			ConsumerBlockNumber,
		>,
	>
	for VersionedDipParachainStateProof<
		RelayBlockNumber,
		KiltDidKeyId,
		KiltAccountId,
		KiltBlockNumber,
		KiltWeb3Name,
		KiltLinkableAccountId,
		ConsumerBlockNumber,
	>
{
	fn from(
		value: ParachainDipDidProof<
			RelayBlockNumber,
			KiltDidKeyId,
			KiltAccountId,
			KiltBlockNumber,
			KiltWeb3Name,
			KiltLinkableAccountId,
			ConsumerBlockNumber,
		>,
	) -> Self {
		Self::V0(value)
	}
}

pub const DEFAULT_MAX_PROVIDER_HEAD_PROOF_LEAVE_COUNT: u32 = 128;
pub const DEFAULT_MAX_PROVIDER_HEAD_PROOF_LEAVE_SIZE: u32 = 1024;
pub const DEFAULT_MAX_DIP_COMMITMENT_PROOF_LEAVE_COUNT: u32 = 128;
pub const DEFAULT_MAX_DIP_COMMITMENT_PROOF_LEAVE_SIZE: u32 = 1024;
pub const DEFAULT_MAX_DID_MERKLE_PROOF_LEAVE_COUNT: u32 = 128;
pub const DEFAULT_MAX_DID_MERKLE_PROOF_LEAVE_SIZE: u32 = 1024;
pub const DEFAULT_MAX_DID_MERKLE_LEAVES_REVEALED: u32 = 128;

/// Versioned proof verifier. For version-specific description, refer to each
/// verifier's documentation.
pub struct KiltVersionedParachainVerifier<
	RelaychainRuntime,
	RelaychainStateRootStore,
	const KILT_PARA_ID: u32,
	KiltRuntime,
	DidCallVerifier,
	SignedExtra = (),
	const MAX_PROVIDER_HEAD_PROOF_LEAVE_COUNT: u32 = DEFAULT_MAX_PROVIDER_HEAD_PROOF_LEAVE_COUNT,
	const MAX_PROVIDER_HEAD_PROOF_LEAVE_SIZE: u32 = DEFAULT_MAX_PROVIDER_HEAD_PROOF_LEAVE_SIZE,
	const MAX_DIP_COMMITMENT_PROOF_LEAVE_COUNT: u32 = DEFAULT_MAX_DIP_COMMITMENT_PROOF_LEAVE_COUNT,
	const MAX_DIP_COMMITMENT_PROOF_LEAVE_SIZE: u32 = DEFAULT_MAX_DIP_COMMITMENT_PROOF_LEAVE_SIZE,
	const MAX_DID_MERKLE_PROOF_LEAVE_COUNT: u32 = DEFAULT_MAX_DID_MERKLE_PROOF_LEAVE_COUNT,
	const MAX_DID_MERKLE_PROOF_LEAVE_SIZE: u32 = DEFAULT_MAX_DID_MERKLE_PROOF_LEAVE_SIZE,
	const MAX_DID_MERKLE_LEAVES_REVEALED: u32 = DEFAULT_MAX_DID_MERKLE_LEAVES_REVEALED,
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
	for KiltVersionedParachainVerifier<
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
	type Proof = VersionedDipParachainStateProof<
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
