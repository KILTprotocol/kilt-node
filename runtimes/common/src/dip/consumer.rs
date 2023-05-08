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

use core::fmt::Debug;

use did::{
	did_details::{DidPublicKey, DidPublicKeyDetails, DidVerificationKey},
	DidSignature, DidVerificationKeyRelationship,
};
use dip_support::{v1, VersionedIdentityProof};
use frame_support::{BoundedVec, RuntimeDebug};
use pallet_dip_consumer::traits::IdentityProofVerifier;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_core::ConstU32;
use sp_runtime::traits::{CheckedAdd, One, Zero};
use sp_std::{collections::btree_map::BTreeMap, marker::PhantomData, vec::Vec};
use sp_trie::{verify_trie_proof, LayoutV1};

use crate::{
	dip::{provider, KeyDetailsKey, KeyDetailsValue, KeyReferenceKey, KeyRelationship, ProofLeaf},
	AccountId, Hash,
};

// TODO: Avoid repetition of the same key if it appears multiple times, e.g., by
// having a vector of `KeyRelationship` instead.
#[derive(Clone, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen, Encode, Decode)]
pub struct ProofEntry<BlockNumber> {
	pub key: DidPublicKeyDetails<BlockNumber>,
	pub relationship: KeyRelationship,
}

// Contains the list of revealed public keys after a given merkle proof has been
// correctly verified.
#[derive(Clone, RuntimeDebug, PartialEq, Eq, TypeInfo, MaxEncodedLen, Encode, Decode)]
pub struct VerificationResult<BlockNumber>(pub BoundedVec<ProofEntry<BlockNumber>, ConstU32<10>>);

impl<BlockNumber> From<Vec<ProofEntry<BlockNumber>>> for VerificationResult<BlockNumber>
where
	BlockNumber: Debug,
{
	fn from(value: Vec<ProofEntry<BlockNumber>>) -> Self {
		Self(value.try_into().expect("Failed to put Vec into BoundedVec"))
	}
}

pub struct DidMerkleProofVerifier<Hasher, BlockNumber, Details>(PhantomData<(Hasher, BlockNumber, Details)>);

impl<Call, Subject, Hasher, BlockNumber, Details> IdentityProofVerifier<Call, Subject>
	for DidMerkleProofVerifier<Hasher, BlockNumber, Details>
where
	BlockNumber: Encode + Clone + Debug,
	Hasher: sp_core::Hasher,
	Hasher::Out: From<Hash>,
{
	// TODO: Proper error handling
	type Error = ();
	type Proof = VersionedIdentityProof<Vec<provider::BlindedValue>, ProofLeaf<Hash, BlockNumber>>;
	type ProofEntry = pallet_dip_consumer::proof::ProofEntry<Hash, Details>;
	type Submitter = AccountId;
	type VerificationResult = VerificationResult<BlockNumber>;

	fn verify_proof_for_call_against_entry(
		_call: &Call,
		_subject: &Subject,
		_submitter: &Self::Submitter,
		proof_entry: &mut Self::ProofEntry,
		proof: &Self::Proof,
	) -> Result<Self::VerificationResult, Self::Error> {
		let proof: v1::MerkleProof<_, _> = proof.clone().try_into()?;
		// TODO: more efficient by removing cloning and/or collecting.
		// Did not find another way of mapping a Vec<(Vec<u8>, Vec<u8>)> to a
		// Vec<(Vec<u8>, Option<Vec<u8>>)>.
		let proof_leaves = proof
			.revealed
			.iter()
			.map(|leaf| (leaf.encoded_key(), Some(leaf.encoded_value())))
			.collect::<Vec<(Vec<u8>, Option<Vec<u8>>)>>();
		verify_trie_proof::<LayoutV1<Hasher>, _, _, _>(&proof_entry.digest.into(), &proof.blinded, &proof_leaves)
			.map_err(|_| ())?;

		// At this point, we know the proof is valid. We just need to map the revealed
		// leaves to something the consumer can easily operate on.

		// Create a map of the revealed public keys
		//TODO: Avoid cloning, and use a map of references for the lookup
		let public_keys: BTreeMap<Hash, DidPublicKeyDetails<BlockNumber>> = proof
			.revealed
			.clone()
			.into_iter()
			.filter_map(|leaf| {
				if let ProofLeaf::KeyDetails(KeyDetailsKey(key_id), KeyDetailsValue(key_details)) = leaf {
					Some((key_id, key_details))
				} else {
					None
				}
			})
			.collect();
		// Create a list of the revealed keys by consuming the provided key reference
		// leaves, and looking up the full details from the just-built `public_keys`
		// map.
		let keys: Vec<ProofEntry<BlockNumber>> = proof
			.revealed
			.into_iter()
			.filter_map(|leaf| {
				if let ProofLeaf::KeyReference(KeyReferenceKey(key_id, key_relationship), _) = leaf {
					// TODO: Better error handling.
					let key_details = public_keys
						.get(&key_id)
						.expect("Key ID should be present in the map of revealed public keys.");
					Some(ProofEntry {
						key: key_details.clone(),
						relationship: key_relationship,
					})
				} else {
					None
				}
			})
			.collect();
		Ok(keys.into())
	}
}

pub trait Bump {
	fn bump(&mut self);
}

impl<T> Bump for T
where
	T: CheckedAdd + Zero + One,
{
	// FIXME: Better implementation?
	fn bump(&mut self) {
		if let Some(new) = self.checked_add(&Self::one()) {
			*self = new;
		} else {
			*self = Self::zero();
		}
	}
}

// Verifies a DID signature over the call details, which is the encoded tuple of
// (call, proof_entry.details(), submitter address).
pub struct DidSignatureVerifier<BlockNumber, Details>(PhantomData<(BlockNumber, Details)>);

impl<Call, Subject, BlockNumber, Details> IdentityProofVerifier<Call, Subject>
	for DidSignatureVerifier<BlockNumber, Details>
where
	BlockNumber: Encode,
	Call: Encode,
	Details: Bump + Encode,
{
	// TODO: Error handling
	type Error = ();
	type Proof = (Vec<ProofEntry<BlockNumber>>, DidSignature);
	type ProofEntry = pallet_dip_consumer::proof::ProofEntry<Hash, Details>;
	type Submitter = AccountId;
	type VerificationResult = (DidVerificationKey, DidVerificationKeyRelationship);

	fn verify_proof_for_call_against_entry(
		call: &Call,
		_subject: &Subject,
		submitter: &Self::Submitter,
		proof_entry: &mut Self::ProofEntry,
		proof: &Self::Proof,
	) -> Result<Self::VerificationResult, Self::Error> {
		let encoded_payload = (call, proof_entry.details(), submitter).encode();
		let mut proof_verification_keys = proof.0.iter().filter_map(
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

pub trait DidDipOriginFilter<Call> {
	type Error;
	type OriginInfo;
	type Success;

	fn check_call_origin_info(call: &Call, info: &Self::OriginInfo) -> Result<Self::Success, Self::Error>;
}

// Verifies a DID signature over the call details AND verifies whether the call
// could be dispatched with the provided signature.
pub struct DidSignatureAndCallVerifier<BlockNumber, Details, CallVerifier>(
	PhantomData<(BlockNumber, Details, CallVerifier)>,
);

impl<Call, Subject, BlockNumber, Details, CallVerifier> IdentityProofVerifier<Call, Subject>
	for DidSignatureAndCallVerifier<BlockNumber, Details, CallVerifier>
where
	BlockNumber: Encode,
	Call: Encode,
	CallVerifier: DidDipOriginFilter<
		Call,
		OriginInfo = <DidSignatureVerifier<BlockNumber, Details> as IdentityProofVerifier<Call, Subject>>::VerificationResult,
	>,
	Details: Bump + Encode,
{
	// FIXME: Better error handling
	type Error = ();
	type Proof = <DidSignatureVerifier<BlockNumber, Details> as IdentityProofVerifier<Call, Subject>>::Proof;
	type ProofEntry = <DidSignatureVerifier<BlockNumber, Details> as IdentityProofVerifier<Call, Subject>>::ProofEntry;
	type Submitter = <DidSignatureVerifier<BlockNumber, Details> as IdentityProofVerifier<Call, Subject>>::Submitter;
	type VerificationResult =
		<DidSignatureVerifier<BlockNumber, Details> as IdentityProofVerifier<Call, Subject>>::VerificationResult;

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
	MerkleProofVerifier: IdentityProofVerifier<Call, Subject>
		+ pallet_dip_consumer::traits::IdentityProofVerifier<Call, sp_runtime::AccountId32>,
	DidSignatureVerifier: IdentityProofVerifier<
		Call,
		Subject,
		Proof = <MerkleProofVerifier as IdentityProofVerifier<Call, Subject>>::VerificationResult,
		ProofEntry = <MerkleProofVerifier as IdentityProofVerifier<Call, Subject>>::ProofEntry,
		Submitter = <MerkleProofVerifier as IdentityProofVerifier<Call, Subject>>::Submitter,
	>,
{
	// FIXME: Better error handling
	type Error = ();
	type Proof = <MerkleProofVerifier as IdentityProofVerifier<Call, Subject>>::Proof;
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
			MerkleProofVerifier::verify_proof_for_call_against_entry(call, subject, submitter, proof_entry, proof)
				.map_err(|_| ())?;
		DidSignatureVerifier::verify_proof_for_call_against_entry(
			call,
			subject,
			submitter,
			proof_entry,
			&merkle_proof_verification,
		)
		.map_err(|_| ())?;
		Ok(merkle_proof_verification)
	}
}
