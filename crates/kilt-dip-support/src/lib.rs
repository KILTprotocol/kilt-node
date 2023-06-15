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

use pallet_dip_consumer::traits::IdentityProofVerifier;
use sp_std::marker::PhantomData;

use crate::did::MerkleLeavesAndDidSignature;

pub mod did;
pub mod merkle;
pub mod traits;
pub mod xcm;

/// A type that chains a Merkle proof verification with a DID signature
/// verification. The required input of this type is a tuple (A, B) where A is
/// the type of input required by the `MerkleProofVerifier` and B is a
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
		identity_details: &mut Self::IdentityDetails,
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
