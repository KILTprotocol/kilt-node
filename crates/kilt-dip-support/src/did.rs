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

use did::{
	did_details::{DidPublicKey, DidPublicKeyDetails, DidVerificationKey},
	DidSignature, DidVerificationKeyRelationship,
};
use pallet_dip_consumer::traits::IdentityProofVerifier;
use parity_scale_codec::Encode;
use sp_core::{ConstU32, Get};
use sp_std::marker::PhantomData;

use crate::{
	merkle::{ProofEntry, VerificationResult},
	traits::{Bump, DidDipOriginFilter},
};

pub struct DidSignatureVerifier<BlockNumber, Digest, Details, AccountId, SignedExtraProvider, S, L>(
	PhantomData<(BlockNumber, Digest, Details, AccountId, SignedExtraProvider, S, L)>,
);

impl<Call, Subject, BlockNumber, Digest, Details, AccountId, SignedExtraProvider, S, const L: u32>
	IdentityProofVerifier<Call, Subject>
	for DidSignatureVerifier<BlockNumber, Digest, Details, AccountId, SignedExtraProvider, S, ConstU32<L>>
where
	AccountId: Encode,
	BlockNumber: Encode,
	Call: Encode,
	Digest: Encode,
	Details: Bump + Encode,
	SignedExtraProvider: Get<S>,
	S: Encode,
{
	// TODO: Error handling
	type Error = ();
	type Proof = (VerificationResult<BlockNumber, L>, DidSignature);
	type ProofEntry = pallet_dip_consumer::proof::ProofEntry<Digest, Details>;
	type Submitter = AccountId;
	type VerificationResult = (DidVerificationKey, DidVerificationKeyRelationship);

	fn verify_proof_for_call_against_entry(
		call: &Call,
		_subject: &Subject,
		submitter: &Self::Submitter,
		proof_entry: &mut Self::ProofEntry,
		proof: &Self::Proof,
	) -> Result<Self::VerificationResult, Self::Error> {
		let encoded_payload = (call, proof_entry.details(), submitter, SignedExtraProvider::get()).encode();
		let mut proof_verification_keys = proof.0 .0.iter().filter_map(
			|ProofEntry {
			     key: DidPublicKeyDetails { key, .. },
			     relationship,
			 }| {
				if let DidPublicKey::PublicVerificationKey(k) = key {
					Some((
						k,
						DidVerificationKeyRelationship::try_from(*relationship).expect("Should never fail."),
					))
				} else {
					None
				}
			},
		);
		let valid_signing_key = proof_verification_keys
			.find(|(verification_key, _)| verification_key.verify_signature(&encoded_payload, &proof.1).is_ok());
		if let Some((key, relationship)) = valid_signing_key {
			proof_entry.details.bump();
			Ok((key.clone(), relationship))
		} else {
			Err(())
		}
	}
}

// Verifies a DID signature over the call details AND verifies whether the call
// could be dispatched with the provided signature.
pub struct DidSignatureAndCallVerifier<DidSignatureVerifier, CallVerifier>(
	PhantomData<(DidSignatureVerifier, CallVerifier)>,
);

impl<Call, Subject, DidSignatureVerifier, CallVerifier> IdentityProofVerifier<Call, Subject>
	for DidSignatureAndCallVerifier<DidSignatureVerifier, CallVerifier>
where
	DidSignatureVerifier: IdentityProofVerifier<Call, Subject>,
	CallVerifier: DidDipOriginFilter<
		Call,
		OriginInfo = <DidSignatureVerifier as IdentityProofVerifier<Call, Subject>>::VerificationResult,
	>,
{
	// FIXME: Better error handling
	type Error = ();
	type Proof = <DidSignatureVerifier as IdentityProofVerifier<Call, Subject>>::Proof;
	type ProofEntry = <DidSignatureVerifier as IdentityProofVerifier<Call, Subject>>::ProofEntry;
	type Submitter = <DidSignatureVerifier as IdentityProofVerifier<Call, Subject>>::Submitter;
	type VerificationResult = <DidSignatureVerifier as IdentityProofVerifier<Call, Subject>>::VerificationResult;

	fn verify_proof_for_call_against_entry(
		call: &Call,
		subject: &Subject,
		submitter: &Self::Submitter,
		proof_entry: &mut Self::ProofEntry,
		proof: &Self::Proof,
	) -> Result<Self::VerificationResult, Self::Error> {
		let did_signing_key =
			DidSignatureVerifier::verify_proof_for_call_against_entry(call, subject, submitter, proof_entry, proof)
				.map_err(|_| ())?;
		CallVerifier::check_call_origin_info(call, &did_signing_key).map_err(|_| ())?;
		Ok(did_signing_key)
	}
}

pub struct MerkleProofAndDidSignatureVerifier<MerkleProofVerifier, DidSignatureVerifier>(
	PhantomData<(MerkleProofVerifier, DidSignatureVerifier)>,
);

impl<Call, Subject, MerkleProofVerifier, DidSignatureVerifier> IdentityProofVerifier<Call, Subject>
	for MerkleProofAndDidSignatureVerifier<MerkleProofVerifier, DidSignatureVerifier>
where
	MerkleProofVerifier: IdentityProofVerifier<Call, Subject>,
	// TODO: get rid of this if possible
	MerkleProofVerifier::VerificationResult: Clone,
	DidSignatureVerifier: IdentityProofVerifier<
		Call,
		Subject,
		Proof = (
			<MerkleProofVerifier as IdentityProofVerifier<Call, Subject>>::VerificationResult,
			DidSignature,
		),
		ProofEntry = <MerkleProofVerifier as IdentityProofVerifier<Call, Subject>>::ProofEntry,
		Submitter = <MerkleProofVerifier as IdentityProofVerifier<Call, Subject>>::Submitter,
	>,
{
	// FIXME: Better error handling
	type Error = ();
	// FIXME: Better type declaration
	type Proof = (
		<MerkleProofVerifier as IdentityProofVerifier<Call, Subject>>::Proof,
		DidSignature,
	);
	type ProofEntry = <DidSignatureVerifier as IdentityProofVerifier<Call, Subject>>::ProofEntry;
	type Submitter = <MerkleProofVerifier as IdentityProofVerifier<Call, Subject>>::Submitter;
	type VerificationResult = <MerkleProofVerifier as IdentityProofVerifier<Call, Subject>>::VerificationResult;

	fn verify_proof_for_call_against_entry(
		call: &Call,
		subject: &Subject,
		submitter: &Self::Submitter,
		proof_entry: &mut Self::ProofEntry,
		proof: &Self::Proof,
	) -> Result<Self::VerificationResult, Self::Error> {
		let merkle_proof_verification =
			MerkleProofVerifier::verify_proof_for_call_against_entry(call, subject, submitter, proof_entry, &proof.0)
				.map_err(|_| ())?;
		DidSignatureVerifier::verify_proof_for_call_against_entry(
			call,
			subject,
			submitter,
			proof_entry,
			// FIXME: Remove `clone()` requirement
			&(merkle_proof_verification.clone(), proof.1.clone()),
		)
		.map_err(|_| ())?;
		Ok(merkle_proof_verification)
	}
}
