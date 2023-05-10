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
use pallet_dip_consumer::{identity::IdentityDetails, traits::IdentityProofVerifier};
use parity_scale_codec::Encode;
use sp_core::Get;
use sp_std::marker::PhantomData;

use crate::{
	merkle::ProofEntry,
	traits::{Bump, DidDipOriginFilter},
};

/// A type that verifies a DID signature over some DID keys revealed by a
/// previously-verified Merkle proof. It requires the `Details` type to
/// implement the `Bump` trait to avoid replay attacks. The basic verification
/// logic verifies that the signature has been generated over the encoded tuple
/// (call, identity details). Additional details can be added to the end of the
/// tuple by providing a `SignedExtraProvider`.
pub struct MerkleRevealedDidSignatureVerifier<
	BlockNumber,
	Digest,
	Details,
	AccountId,
	SignedExtraProvider,
	SignedExtra,
	MerkleProofEntries,
>(
	PhantomData<(
		BlockNumber,
		Digest,
		Details,
		AccountId,
		SignedExtraProvider,
		SignedExtra,
		MerkleProofEntries,
	)>,
);

impl<Call, Subject, BlockNumber, Digest, Details, AccountId, SignedExtraProvider, SignedExtra, MerkleProofEntries>
	IdentityProofVerifier<Call, Subject>
	for MerkleRevealedDidSignatureVerifier<
		BlockNumber,
		Digest,
		Details,
		AccountId,
		SignedExtraProvider,
		SignedExtra,
		MerkleProofEntries,
	> where
	AccountId: Encode,
	BlockNumber: Encode,
	Call: Encode,
	Digest: Encode,
	Details: Bump + Encode,
	SignedExtraProvider: Get<SignedExtra>,
	SignedExtra: Encode,
	MerkleProofEntries: AsRef<[ProofEntry<BlockNumber>]>,
{
	// TODO: Error handling
	type Error = ();
	/// The proof must be a list of Merkle leaves that have been previously
	/// verified by a different verifier.
	type Proof = (MerkleProofEntries, DidSignature);
	/// The `Details` that are part of the identity details must implement the
	/// `Bump` trait.
	type IdentityDetails = IdentityDetails<Digest, Details>;
	/// The type of the submitter's accounts.
	type Submitter = AccountId;
	/// Successful verifications return the verification key used to validate
	/// the provided signature and its relationship to the DID subject.
	type VerificationResult = (DidVerificationKey, DidVerificationKeyRelationship);

	fn verify_proof_for_call_against_entry(
		call: &Call,
		_subject: &Subject,
		submitter: &Self::Submitter,
		proof_entry: &mut Self::IdentityDetails,
		proof: &Self::Proof,
	) -> Result<Self::VerificationResult, Self::Error> {
		let encoded_payload = (call, proof_entry.details(), submitter, SignedExtraProvider::get()).encode();
		// Only consider verification keys from the set of revealed merkle leaves.
		let mut proof_verification_keys = proof.0.as_ref().iter().filter_map(
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

/// A type that chains a DID signature verification, as provided by
/// `MerkleRevealedDidSignatureVerifier`, and a call filtering logic based on
/// the type of key used in the signature.
/// Verification bails out early in case of invalid DID signatures. Otherwise,
/// the retrived key and its relationship is passed to the call verifier to do
/// some additional lookups on the call.
/// The `CallVerifier` only performs internal checks, while all input and output
/// types are taken from the provided `DidSignatureVerifier` type.
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
	/// The input proof is the same accepted by the `DidSignatureVerifier`.
	type Proof = <DidSignatureVerifier as IdentityProofVerifier<Call, Subject>>::Proof;
	/// The identity details are the same accepted by the
	/// `DidSignatureVerifier`.
	type IdentityDetails = <DidSignatureVerifier as IdentityProofVerifier<Call, Subject>>::IdentityDetails;
	/// The submitter address is the same accepted by the
	/// `DidSignatureVerifier`.
	type Submitter = <DidSignatureVerifier as IdentityProofVerifier<Call, Subject>>::Submitter;
	/// The verification result is the same accepted by the
	/// `DidSignatureVerifier`.
	type VerificationResult = <DidSignatureVerifier as IdentityProofVerifier<Call, Subject>>::VerificationResult;

	fn verify_proof_for_call_against_entry(
		call: &Call,
		subject: &Subject,
		submitter: &Self::Submitter,
		proof_entry: &mut Self::IdentityDetails,
		proof: &Self::Proof,
	) -> Result<Self::VerificationResult, Self::Error> {
		let did_signing_key =
			DidSignatureVerifier::verify_proof_for_call_against_entry(call, subject, submitter, proof_entry, proof)
				.map_err(|_| ())?;
		CallVerifier::check_call_origin_info(call, &did_signing_key).map_err(|_| ())?;
		Ok(did_signing_key)
	}
}

/// A type that chains a Merkle proof verification with a DID signature
/// verification. The required input of this type is a tuple (A, B) where A is
/// the type of input required by the `MerkleProofVerifier` and B is a
/// `DidSignature.
/// The successful output of this type is the output type of the
/// `MerkleProofVerifier`, meaning that DID signature verification happens
/// internally and does not transform the result in any way.
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
		IdentityDetails = <MerkleProofVerifier as IdentityProofVerifier<Call, Subject>>::IdentityDetails,
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
	type IdentityDetails = <DidSignatureVerifier as IdentityProofVerifier<Call, Subject>>::IdentityDetails;
	type Submitter = <MerkleProofVerifier as IdentityProofVerifier<Call, Subject>>::Submitter;
	type VerificationResult = <MerkleProofVerifier as IdentityProofVerifier<Call, Subject>>::VerificationResult;

	fn verify_proof_for_call_against_entry(
		call: &Call,
		subject: &Subject,
		submitter: &Self::Submitter,
		proof_entry: &mut Self::IdentityDetails,
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
