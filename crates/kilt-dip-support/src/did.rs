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
use frame_support::ensure;
use pallet_dip_consumer::{identity::IdentityDetails, traits::IdentityProofVerifier};
use pallet_dip_provider::traits::IdentityProvider;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_core::{ConstU64, Get, RuntimeDebug};
use sp_runtime::traits::CheckedSub;
use sp_std::marker::PhantomData;

use crate::{
	merkle::RevealedDidKey,
	traits::{Bump, DidDipOriginFilter},
};

#[derive(Encode, Decode, RuntimeDebug, Clone, Eq, PartialEq, TypeInfo)]
pub struct TimeBoundDidSignature<BlockNumber> {
	pub signature: DidSignature,
	pub block_number: BlockNumber,
}

#[derive(Encode, Decode, RuntimeDebug, Clone, Eq, PartialEq, TypeInfo)]
pub struct MerkleLeavesAndDidSignature<MerkleLeaves, BlockNumber> {
	pub merkle_leaves: MerkleLeaves,
	pub did_signature: TimeBoundDidSignature<BlockNumber>,
}

/// A type that verifies a DID signature over some DID keys revealed by a
/// previously-verified Merkle proof. It requires the `Details` type to
/// implement the `Bump` trait to avoid replay attacks. The basic verification
/// logic verifies that the signature has been generated over the encoded tuple
/// (call, identity details, submitter_address, submission_block_number,
/// genesis_hash). Additional details can be added to the end of the tuple by
/// providing a `SignedExtraProvider`.
pub struct MerkleRevealedDidSignatureVerifier<
	KeyId,
	BlockNumber,
	Digest,
	Details,
	AccountId,
	MerkleProofEntries,
	BlockNumberProvider,
	const SIGNATURE_VALIDITY: u64,
	GenesisHashProvider,
	Hash,
	SignedExtraProvider = (),
	SignedExtra = (),
>(
	#[allow(clippy::type_complexity)]
	PhantomData<(
		KeyId,
		BlockNumber,
		Digest,
		Details,
		AccountId,
		MerkleProofEntries,
		BlockNumberProvider,
		ConstU64<SIGNATURE_VALIDITY>,
		GenesisHashProvider,
		Hash,
		SignedExtraProvider,
		SignedExtra,
	)>,
);

impl<
		Call,
		Subject,
		KeyId,
		BlockNumber,
		Digest,
		Details,
		AccountId,
		MerkleProofEntries,
		BlockNumberProvider,
		const SIGNATURE_VALIDITY: u64,
		GenesisHashProvider,
		Hash,
		SignedExtraProvider,
		SignedExtra,
	> IdentityProofVerifier<Call, Subject>
	for MerkleRevealedDidSignatureVerifier<
		KeyId,
		BlockNumber,
		Digest,
		Details,
		AccountId,
		MerkleProofEntries,
		BlockNumberProvider,
		SIGNATURE_VALIDITY,
		GenesisHashProvider,
		Hash,
		SignedExtraProvider,
		SignedExtra,
	> where
	AccountId: Encode,
	BlockNumber: Encode + CheckedSub + Into<u64> + PartialOrd + sp_std::fmt::Debug,
	Call: Encode,
	Digest: Encode,
	Details: Bump + Encode,
	MerkleProofEntries: AsRef<[RevealedDidKey<KeyId, BlockNumber>]>,
	BlockNumberProvider: Get<BlockNumber>,
	GenesisHashProvider: Get<Hash>,
	Hash: Encode,
	SignedExtraProvider: Get<SignedExtra>,
	SignedExtra: Encode,
{
	// TODO: Error handling
	type Error = ();
	/// The proof must be a list of Merkle leaves that have been previously
	/// verified by the Merkle proof verifier, and the additional DID signature.
	type Proof = MerkleLeavesAndDidSignature<MerkleProofEntries, BlockNumber>;
	/// The `Details` that are part of the identity details must implement the
	/// `Bump` trait.
	type IdentityDetails = IdentityDetails<Digest, Details>;
	/// The type of the submitter's accounts.
	type Submitter = AccountId;
	/// Successful verifications return the verification key used to validate
	/// the provided signature and its relationship to the DID subject.
	type VerificationResult = (DidVerificationKey, DidVerificationKeyRelationship);

	fn verify_proof_for_call_against_details(
		call: &Call,
		_subject: &Subject,
		submitter: &Self::Submitter,
		identity_details: &mut Self::IdentityDetails,
		proof: &Self::Proof,
	) -> Result<Self::VerificationResult, Self::Error> {
		let block_number = BlockNumberProvider::get();
		let is_signature_fresh =
			if let Some(blocks_ago_from_now) = block_number.checked_sub(&proof.did_signature.block_number) {
				// False if the signature is too old.
				blocks_ago_from_now.into() <= SIGNATURE_VALIDITY
			} else {
				// Signature generated at a future time, not possible to verify.
				false
			};

		ensure!(is_signature_fresh, ());
		let encoded_payload = (
			call,
			&identity_details.details,
			submitter,
			&proof.did_signature.block_number,
			GenesisHashProvider::get(),
			SignedExtraProvider::get(),
		)
			.encode();
		// Only consider verification keys from the set of revealed keys.
		let mut proof_verification_keys = proof.merkle_leaves.as_ref().iter().filter_map(|RevealedDidKey { relationship, details: DidPublicKeyDetails { key, .. }, .. } | {
			let DidPublicKey::PublicVerificationKey(key) = key else { return None };
			Some((key, DidVerificationKeyRelationship::try_from(*relationship).expect("Should never fail to build a VerificationRelationship from the given DidKeyRelationship because we have already made sure the conditions hold.")))
		});
		let valid_signing_key = proof_verification_keys.find(|(verification_key, _)| {
			verification_key
				.verify_signature(&encoded_payload, &proof.did_signature.signature)
				.is_ok()
		});
		let Some((key, relationship)) = valid_signing_key else { return Err(()) };
		identity_details.details.bump();
		Ok((key.clone(), relationship))
	}
}

/// A type that chains a DID signature verification, as provided by
/// `MerkleRevealedDidSignatureVerifier`, and a call filtering logic based on
/// the type of key used in the signature.
/// Verification bails out early in case of invalid DID signatures. Otherwise,
/// the retrieved key and its relationship is passed to the call verifier to do
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
	CallVerifier: DidDipOriginFilter<Call, OriginInfo = DidSignatureVerifier::VerificationResult>,
{
	// FIXME: Better error handling
	type Error = ();
	/// The input proof is the same accepted by the `DidSignatureVerifier`.
	type Proof = DidSignatureVerifier::Proof;
	/// The identity details are the same accepted by the
	/// `DidSignatureVerifier`.
	type IdentityDetails = DidSignatureVerifier::IdentityDetails;
	/// The submitter address is the same accepted by the
	/// `DidSignatureVerifier`.
	type Submitter = DidSignatureVerifier::Submitter;
	/// The verification result is the same accepted by the
	/// `DidSignatureVerifier`.
	type VerificationResult = DidSignatureVerifier::VerificationResult;

	fn verify_proof_for_call_against_details(
		call: &Call,
		subject: &Subject,
		submitter: &Self::Submitter,
		identity_details: &mut Self::IdentityDetails,
		proof: &Self::Proof,
	) -> Result<Self::VerificationResult, Self::Error> {
		let did_signing_key = DidSignatureVerifier::verify_proof_for_call_against_details(
			call,
			subject,
			submitter,
			identity_details,
			proof,
		)
		.map_err(|_| ())?;
		CallVerifier::check_call_origin_info(call, &did_signing_key).map_err(|_| ())?;
		Ok(did_signing_key)
	}
}

pub struct CombinedIdentityResult<OutputA, OutputB, OutputC> {
	pub a: OutputA,
	pub b: OutputB,
	pub c: OutputC,
}

impl<OutputA, OutputB, OutputC> From<(OutputA, OutputB, OutputC)>
	for CombinedIdentityResult<OutputA, OutputB, OutputC>
{
	fn from(value: (OutputA, OutputB, OutputC)) -> Self {
		Self {
			a: value.0,
			b: value.1,
			c: value.2,
		}
	}
}

impl<OutputA, OutputB, OutputC> CombinedIdentityResult<OutputA, OutputB, OutputC>
where
	OutputB: Default,
	OutputC: Default,
{
	pub fn from_a(a: OutputA) -> Self {
		Self {
			a,
			b: OutputB::default(),
			c: OutputC::default(),
		}
	}
}

impl<OutputA, OutputB, OutputC> CombinedIdentityResult<OutputA, OutputB, OutputC>
where
	OutputA: Default,
	OutputC: Default,
{
	pub fn from_b(b: OutputB) -> Self {
		Self {
			a: OutputA::default(),
			b,
			c: OutputC::default(),
		}
	}
}

impl<OutputA, OutputB, OutputC> CombinedIdentityResult<OutputA, OutputB, OutputC>
where
	OutputA: Default,
	OutputB: Default,
{
	pub fn from_c(c: OutputC) -> Self {
		Self {
			a: OutputA::default(),
			b: OutputB::default(),
			c,
		}
	}
}

pub struct CombineIdentityFrom<A, B, C>(PhantomData<(A, B, C)>);

impl<Identifier, A, B, C> IdentityProvider<Identifier> for CombineIdentityFrom<A, B, C>
where
	A: IdentityProvider<Identifier>,
	B: IdentityProvider<Identifier>,
	C: IdentityProvider<Identifier>,
{
	// TODO: Proper error handling
	type Error = ();
	type Success = CombinedIdentityResult<Option<A::Success>, Option<B::Success>, Option<C::Success>>;

	fn retrieve(identifier: &Identifier) -> Result<Option<Self::Success>, Self::Error> {
		match (
			A::retrieve(identifier),
			B::retrieve(identifier),
			C::retrieve(identifier),
		) {
			// If no details is returned, return None for the whole result
			(Ok(None), Ok(None), Ok(None)) => Ok(None),
			// Otherwise, return `Some` or `None` depending on each result
			(Ok(ok_a), Ok(ok_b), Ok(ok_c)) => Ok(Some(CombinedIdentityResult {
				a: ok_a,
				b: ok_b,
				c: ok_c,
			})),
			// If any of them returns an `Err`, return an `Err`
			_ => Err(()),
		}
	}
}
