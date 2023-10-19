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
use parity_scale_codec::{Decode, Encode};
use scale_info::{
	prelude::string::{String, ToString},
	TypeInfo,
};
use sp_core::RuntimeDebug;
use sp_runtime::traits::CheckedSub;
use sp_std::{marker::PhantomData, vec::Vec};

use crate::{
	merkle::RevealedDidKey,
	traits::{Bump, DidSignatureVerifierContext, DipCallOriginFilter},
};

#[derive(Encode, Decode, RuntimeDebug, Clone, Eq, PartialEq, TypeInfo)]
pub(crate) struct RevealedDidKeysAndSignature<RevealedDidKeys, BlockNumber> {
	pub merkle_leaves: RevealedDidKeys,
	pub did_signature: TimeBoundDidSignature<BlockNumber>,
}

#[derive(Encode, Decode, RuntimeDebug, Clone, Eq, PartialEq, TypeInfo)]
pub struct TimeBoundDidSignature<BlockNumber> {
	pub signature: DidSignature,
	pub block_number: BlockNumber,
}

pub(crate) struct RevealedDidKeysSignatureAndCallVerifier<
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
	#[allow(clippy::type_complexity)]
	PhantomData<(
		Call,
		Submitter,
		DidLocalDetails,
		MerkleProofEntries,
		ContextProvider,
		RemoteKeyId,
		RemoteAccountId,
		RemoteBlockNumber,
		CallVerifier,
	)>,
);

impl<
		Call,
		Submitter,
		DidLocalDetails,
		MerkleProofEntries,
		ContextProvider,
		RemoteKeyId,
		RemoteAccountId,
		RemoteBlockNumber,
		CallVerifier,
	>
	RevealedDidKeysSignatureAndCallVerifier<
		Call,
		Submitter,
		DidLocalDetails,
		MerkleProofEntries,
		ContextProvider,
		RemoteKeyId,
		RemoteAccountId,
		RemoteBlockNumber,
		CallVerifier,
	> where
	Call: Encode,
	Submitter: Encode,
	ContextProvider: DidSignatureVerifierContext,
	ContextProvider::BlockNumber: Encode + CheckedSub + From<u16> + PartialOrd,
	ContextProvider::Hash: Encode,
	ContextProvider::SignedExtra: Encode,
	DidLocalDetails: Bump + Default + Encode,
	RemoteAccountId: Clone,
	MerkleProofEntries: sp_std::borrow::Borrow<[RevealedDidKey<RemoteKeyId, RemoteBlockNumber, RemoteAccountId>]>,
	CallVerifier: DipCallOriginFilter<
		Call,
		OriginInfo = (DidVerificationKey<RemoteAccountId>, DidVerificationKeyRelationship),
		Error = String,
	>,
{
	#[allow(clippy::result_unit_err)]
	pub(crate) fn verify_did_signature_for_call(
		call: &Call,
		submitter: &Submitter,
		local_details: &mut Option<DidLocalDetails>,
		merkle_revealed_did_signature: RevealedDidKeysAndSignature<MerkleProofEntries, ContextProvider::BlockNumber>,
	) -> Result<(DidVerificationKey<RemoteAccountId>, DidVerificationKeyRelationship), String> {
		let block_number = ContextProvider::block_number();
		let is_signature_fresh = if let Some(blocks_ago_from_now) =
			block_number.checked_sub(&merkle_revealed_did_signature.did_signature.block_number)
		{
			// False if the signature is too old.
			blocks_ago_from_now <= ContextProvider::SIGNATURE_VALIDITY.into()
		} else {
			// Signature generated at a future time, not possible to verify.
			false
		};
		ensure!(
			is_signature_fresh,
			"Failed to verify provided DID signature because is not fresh enough."
		);
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
					log::error!("Should never fail to build a VerificationRelationship from the given DidKeyRelationship because we have already made sure the conditions hold.");
					Some(Err("InternalError"))
				}
			}).collect::<Result<_, _>>()?;
		let valid_signing_key = proof_verification_keys.iter().find(|(verification_key, _)| {
			verification_key
				.verify_signature(&encoded_payload, &merkle_revealed_did_signature.did_signature.signature)
				.is_ok()
		});
		let Some((key, relationship)) = valid_signing_key else {
			return Err("Failed to verify the call signature using any of the revealed keys.".to_string());
		};
		if let Some(details) = local_details {
			details.bump();
		} else {
			*local_details = Some(DidLocalDetails::default());
		};
		CallVerifier::check_call_origin_info(call, &(key.clone(), *relationship))?;
		Ok((key.clone(), *relationship))
	}
}
