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

//! Module to deal with cross-chain KILT DIDs.

use did::{
	did_details::{DidPublicKey, DidPublicKeyDetails, DidVerificationKey},
	DidSignature, DidVerificationKeyRelationship,
};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_core::RuntimeDebug;
use sp_runtime::traits::CheckedSub;
use sp_std::vec::Vec;

use crate::{
	merkle::RevealedDidKey,
	traits::{DidSignatureVerifierContext, DipCallOriginFilter, Incrementable},
};

/// Type returned by the Merkle proof verifier component of the DIP consumer
/// after verifying a DIP Merkle proof.
#[derive(Encode, Decode, RuntimeDebug, Clone, Eq, PartialEq, TypeInfo)]
pub(crate) struct RevealedDidKeysAndSignature<RevealedDidKeys, BlockNumber> {
	/// The keys revelaed in the Merkle proof.
	pub merkle_leaves: RevealedDidKeys,
	/// The [`DIDSignature`] + consumer chain block number to which the DID
	/// signature is anchored.
	pub did_signature: TimeBoundDidSignature<BlockNumber>,
}

/// A DID signature anchored to a specific block height.
#[derive(Encode, Decode, RuntimeDebug, Clone, Eq, PartialEq, TypeInfo)]
pub struct TimeBoundDidSignature<BlockNumber> {
	/// The signature.
	pub signature: DidSignature,
	/// The block number, in the context of the local executor, to which the
	/// signature is anchored.
	pub block_number: BlockNumber,
}

#[cfg(feature = "runtime-benchmarks")]
impl<BlockNumber, Context> kilt_support::traits::GetWorstCase<Context> for TimeBoundDidSignature<BlockNumber>
where
	DidSignature: kilt_support::traits::GetWorstCase<Context>,
	BlockNumber: Default,
{
	fn worst_case(context: Context) -> Self {
		Self {
			signature: DidSignature::worst_case(context),
			block_number: BlockNumber::default(),
		}
	}
}

pub enum RevealedDidKeysSignatureAndCallVerifierError {
	SignatureNotFresh,
	SignatureUnverifiable,
	OriginCheckFailed,
	Internal,
}

impl From<RevealedDidKeysSignatureAndCallVerifierError> for u8 {
	fn from(value: RevealedDidKeysSignatureAndCallVerifierError) -> Self {
		match value {
			RevealedDidKeysSignatureAndCallVerifierError::SignatureNotFresh => 0,
			RevealedDidKeysSignatureAndCallVerifierError::SignatureUnverifiable => 1,
			RevealedDidKeysSignatureAndCallVerifierError::OriginCheckFailed => 2,
			RevealedDidKeysSignatureAndCallVerifierError::Internal => u8::MAX,
		}
	}
}

/// Function that tries to verify a DID signature over a given payload by
/// using one of the DID keys revealed in the Merkle proof. This verifier is
/// typically used in conjunction with a verifier that takes a user-provided
/// input Merkle proof, verifies it, and transforms it into a struct that this
/// and other verifiers can easily consume, e.g., a list of DID keys.
/// The generic types are the following:
/// * `Call`: The call to be dispatched on the local chain after verifying the
///   DID signature.
/// * `Submitter`: The blockchain account (**not** the identity subject)
///   submitting the cross-chain transaction (and paying for its execution
///   fees).
/// * `DidLocalDetails`: Any information associated to the identity subject that
///   is stored locally, e.g., under the `IdentityEntries` map of the
///   `pallet-dip-consumer` pallet.
/// * `MerkleProofEntries`: The type returned by the Merkle proof verifier that
///   includes the identity parts revealed in the Merkle proof.
/// * `ContextProvider`: Provides additional local context (e.g., current block
///   number) to verify the DID signature.
/// * `RemoteKeyId`: Definition of a DID key ID as specified by the provider.
/// * `RemoteAccountId`: Definition of a linked account ID as specified by the
///   provider.
/// * `RemoteBlockNumber`: Definition of a block number on the provider chain.
/// * `CallVerifier`: A type specifying whether the provided `Call` can be
///   dispatched with the information provided in the DIP proof.
pub(crate) fn verify_did_signature_for_call<
	Call,
	Submitter,
	DidLocalDetails,
	MerkleProofEntries,
	ContextProvider,
	RemoteKeyId,
	RemoteAccountId,
	RemoteBlockNumber,
	CallVerifier,
>(
	call: &Call,
	submitter: &Submitter,
	local_details: &mut Option<DidLocalDetails>,
	merkle_revealed_did_signature: RevealedDidKeysAndSignature<MerkleProofEntries, ContextProvider::BlockNumber>,
) -> Result<
	(DidVerificationKey<RemoteAccountId>, DidVerificationKeyRelationship),
	RevealedDidKeysSignatureAndCallVerifierError,
>
where
	Call: Encode,
	Submitter: Encode,
	ContextProvider: DidSignatureVerifierContext,
	ContextProvider::BlockNumber: Encode + CheckedSub + From<u16> + PartialOrd,
	ContextProvider::Hash: Encode,
	ContextProvider::SignedExtra: Encode,
	DidLocalDetails: Incrementable + Default + Encode,
	RemoteAccountId: Clone,
	MerkleProofEntries: sp_std::borrow::Borrow<[RevealedDidKey<RemoteKeyId, RemoteBlockNumber, RemoteAccountId>]>,
	CallVerifier:
		DipCallOriginFilter<Call, OriginInfo = (DidVerificationKey<RemoteAccountId>, DidVerificationKeyRelationship)>,
{
	cfg_if::cfg_if! {
		if #[cfg(feature = "runtime-benchmarks")] {
			{}
		} else {
			let block_number = ContextProvider::current_block_number();
			let is_signature_fresh = if let Some(blocks_ago_from_now) =
				block_number.checked_sub(&merkle_revealed_did_signature.did_signature.block_number)
			{
				// False if the signature is too old.
				blocks_ago_from_now <= ContextProvider::SIGNATURE_VALIDITY.into()
			} else {
				// Signature generated at a future time, not possible to verify.
				false
			};
			frame_support::ensure!(
				is_signature_fresh,
				RevealedDidKeysSignatureAndCallVerifierError::SignatureNotFresh,
			);
		}
	}
	let encoded_payload = (
		call,
		&local_details,
		submitter,
		&merkle_revealed_did_signature.did_signature.block_number,
		ContextProvider::genesis_hash(),
		ContextProvider::signed_extra(),
	)
		.encode();
	// Only consider verification keys from the set of revealed keys.
	let proof_verification_keys: Vec<(DidVerificationKey<RemoteAccountId>, DidVerificationKeyRelationship)> = merkle_revealed_did_signature.merkle_leaves.borrow().iter().filter_map(|RevealedDidKey {
			relationship, details: DidPublicKeyDetails { key, .. }, .. } | {
				let DidPublicKey::PublicVerificationKey(key) = key else { return None };
				if let Ok(vr) = DidVerificationKeyRelationship::try_from(*relationship) {
					// TODO: Fix this logic to avoid cloning
					Some(Ok((key.clone(), vr)))
				} else {
					cfg_if::cfg_if! {
						if #[cfg(feature = "runtime-benchmarks")] {
							None
						} else {
							log::error!("Should never fail to build a VerificationRelationship from the given DidKeyRelationship because we have already made sure the conditions hold.");
							Some(Err(RevealedDidKeysSignatureAndCallVerifierError::Internal))
						}
					}
				}
			}).collect::<Result<_, _>>()?;
	let valid_signing_key = proof_verification_keys.iter().find(|(verification_key, _)| {
		verification_key
			.verify_signature(&encoded_payload, &merkle_revealed_did_signature.did_signature.signature)
			.is_ok()
	});
	cfg_if::cfg_if! {
		if #[cfg(feature = "runtime-benchmarks")] {
			let default = (
				DidVerificationKey::Ed25519(sp_core::ed25519::Public::from_raw([0u8; 32])),
				DidVerificationKeyRelationship::Authentication,
			);
			let (key, relationship) = valid_signing_key.unwrap_or(&default);
		} else {
			let (key, relationship) = valid_signing_key.ok_or(RevealedDidKeysSignatureAndCallVerifierError::SignatureUnverifiable)?;
		}
	}

	if let Some(details) = local_details {
		details.increment();
	} else {
		*local_details = Some(DidLocalDetails::default());
	};
	let res = CallVerifier::check_call_origin_info(call, &(key.clone(), *relationship));
	cfg_if::cfg_if! {
		if #[cfg(feature = "runtime-benchmarks")] {
			drop(res);
		} else {
			res.map_err(|_| RevealedDidKeysSignatureAndCallVerifierError::OriginCheckFailed)?;
		}
	}
	Ok((key.clone(), *relationship))
}
