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

use parity_scale_codec::{Decode, HasCompact};
use sp_core::{Get, U256};
use sp_runtime::traits::Hash;
use sp_std::marker::PhantomData;

use pallet_dip_consumer::{identity::IdentityDetails, traits::IdentityProofVerifier};

use crate::did::MerkleLeavesAndDidSignature;

pub mod did;
pub mod merkle;
pub mod state_proofs;
pub mod traits;

#[derive(Clone)]
pub struct DipSiblingParachainStateProof<InnerProof> {
	para_root_proof: Vec<Vec<u8>>,
	dip_commitment_proof: Vec<Vec<u8>>,
	dip_proof: InnerProof,
}

pub struct StateProofDipVerifier<
	ProviderParaId,
	RelayInfoProvider,
	ProviderParaInfoProvider,
	DipVerifier,
	LocalIdentityInfo,
>(
	PhantomData<(
		ProviderParaId,
		RelayInfoProvider,
		ProviderParaInfoProvider,
		DipVerifier,
		LocalIdentityInfo,
	)>,
);

impl<Call, Subject, ProviderParaId, RelayInfoProvider, ProviderParaInfoProvider, DipVerifier, LocalIdentityInfo>
	IdentityProofVerifier<Call, Subject>
	for StateProofDipVerifier<ProviderParaId, RelayInfoProvider, ProviderParaInfoProvider, DipVerifier, LocalIdentityInfo>
where
	ProviderParaId: Get<RelayInfoProvider::ParaId>,

	RelayInfoProvider: state_proofs::relay_chain::RelayChainStateInfoProvider,
	RelayInfoProvider::Hasher: 'static,
	<RelayInfoProvider::Hasher as Hash>::Output: Ord,
	RelayInfoProvider::BlockNumber: Copy + Into<U256> + TryFrom<U256> + HasCompact,
	RelayInfoProvider::Key: AsRef<[u8]>,

	ProviderParaInfoProvider: state_proofs::parachain::ParachainStateInfoProvider<Identifier = Subject>,
	ProviderParaInfoProvider::Hasher: 'static,
	<ProviderParaInfoProvider::Hasher as Hash>::Output: Ord + PartialEq<<RelayInfoProvider::Hasher as Hash>::Output>,
	ProviderParaInfoProvider::Commitment: Decode,
	ProviderParaInfoProvider::Key: AsRef<[u8]>,

	DipVerifier: IdentityProofVerifier<
		Call,
		Subject,
		IdentityDetails = IdentityDetails<ProviderParaInfoProvider::Commitment, Option<LocalIdentityInfo>>,
	>,

	LocalIdentityInfo: Clone,
{
	type Error = ();
	type IdentityDetails = LocalIdentityInfo;
	type Proof = DipSiblingParachainStateProof<DipVerifier::Proof>;
	type Submitter = DipVerifier::Submitter;
	type VerificationResult = DipVerifier::VerificationResult;

	fn verify_proof_for_call_against_details(
		call: &Call,
		subject: &Subject,
		submitter: &Self::Submitter,
		identity_details: &mut Option<Self::IdentityDetails>,
		proof: &Self::Proof,
	) -> Result<Self::VerificationResult, Self::Error> {
		let provider_parachain_header =
			state_proofs::relay_chain::ParachainHeadProofVerifier::<RelayInfoProvider>::verify_proof_for_parachain(
				&ProviderParaId::get(),
				proof.para_root_proof.clone(),
			)?;
		let provider_parachain_root_state = provider_parachain_header.state_root;
		debug_assert!(
			ProviderParaInfoProvider::state_root() == provider_parachain_root_state,
			"Provided parachain state root and calculated parachain state root do not match."
		);
		let subject_identity_commitment = state_proofs::parachain::DipCommitmentValueProofVerifier::<
			ProviderParaInfoProvider,
		>::verify_proof_for_identifier(subject, proof.dip_commitment_proof.clone())?;
		let mapped_identity_details = IdentityDetails {
			digest: subject_identity_commitment,
			details: identity_details.clone(),
		};
		DipVerifier::verify_proof_for_call_against_details(
			call,
			subject,
			submitter,
			&mut Some(mapped_identity_details),
			&proof.dip_proof,
		)
		.map_err(|_| ())
	}
}

/// A type that chains a Merkle proof verification with a DID signature
/// verification. The required input of this type is a tuple (A, B) where A is
/// /// the type of input required by the `MerkleProofVerifier` and B is a
/// `DidSignature`.
/// The successful output of this type is the output type of the
/// `MerkleProofVerifier`, meaning that DID signature verification happens
/// internally and does not transform the result in any way.
pub struct MerkleProofAndDidSignatureVerifier<BlockNumber, MerkleProofVerifier, DidSignatureVerifier>(
	PhantomData<(BlockNumber, MerkleProofVerifier, DidSignatureVerifier)>,
);

impl<Call, Subject, BlockNumber, MerkleProofVerifier, DidSignatureVerifier> IdentityProofVerifier<Call, Subject>
	for MerkleProofAndDidSignatureVerifier<BlockNumber, MerkleProofVerifier, DidSignatureVerifier>
where
	BlockNumber: Clone,
	MerkleProofVerifier: IdentityProofVerifier<Call, Subject>,
	// TODO: get rid of this if possible
	MerkleProofVerifier::VerificationResult: Clone,
	DidSignatureVerifier: IdentityProofVerifier<
		Call,
		Subject,
		Proof = MerkleLeavesAndDidSignature<MerkleProofVerifier::VerificationResult, BlockNumber>,
		IdentityDetails = MerkleProofVerifier::IdentityDetails,
		Submitter = MerkleProofVerifier::Submitter,
	>,
{
	// FIXME: Better error handling
	type Error = ();
	// FIXME: Better type declaration
	type Proof = MerkleLeavesAndDidSignature<MerkleProofVerifier::Proof, BlockNumber>;
	type IdentityDetails = DidSignatureVerifier::IdentityDetails;
	type Submitter = MerkleProofVerifier::Submitter;
	type VerificationResult = MerkleProofVerifier::VerificationResult;

	fn verify_proof_for_call_against_details(
		call: &Call,
		subject: &Subject,
		submitter: &Self::Submitter,
		identity_details: &mut Option<Self::IdentityDetails>,
		proof: &Self::Proof,
	) -> Result<Self::VerificationResult, Self::Error> {
		let merkle_proof_verification = MerkleProofVerifier::verify_proof_for_call_against_details(
			call,
			subject,
			submitter,
			identity_details,
			&proof.merkle_leaves,
		)
		.map_err(|_| ())?;
		DidSignatureVerifier::verify_proof_for_call_against_details(
			call,
			subject,
			submitter,
			identity_details,
			// FIXME: Remove `clone()` requirement
			&MerkleLeavesAndDidSignature {
				merkle_leaves: merkle_proof_verification.clone(),
				did_signature: proof.did_signature.clone(),
			},
		)
		.map_err(|_| ())?;
		Ok(merkle_proof_verification)
	}
}
