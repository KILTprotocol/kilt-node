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
	did_details::{DidPublicKey, DidPublicKeyDetails},
	DidSignature,
};
use frame_support::ensure;
use sp_core::ConstU32;
use sp_runtime::{traits::SaturatedConversion, BoundedVec};
use sp_std::vec::Vec;

use crate::{
	merkle_proofs::v0::{
		input_common::TimeBoundDidSignature,
		output_common::{DidKeyRelationship, DipOriginInfo, RevealedDidKey, RevealedDidMerkleProofLeaf},
	},
	Error,
};

#[cfg(test)]
mod tests;

/// A DIP proof whose information has been verified but that contains a
/// cross-chain [`TimeBoundDidSignature`] that still needs verification.
///
/// The generic types indicate the following:
/// * `KiltDidKeyId`: The DID key ID type configured by the KILT chain.
/// * `KiltAccountId`: The `AccountId` type configured by the KILT chain.
/// * `KiltBlockNumber`: The `BlockNumber` type configured by the KILT chain.
/// * `KiltWeb3Name`: The web3name type configured by the KILT chain.
/// * `KiltLinkableAccountId`: The linkable account ID type configured by the
///   KILT chain.
/// * `ConsumerBlockNumber`: The `BlockNumber` definition of the consumer
///   parachain.
/// * `MAX_REVEALED_LEAVES_COUNT`: The maximum number of leaves revealable in
///   the proof.
#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(test, derive(Clone))]
pub struct DipRevealedDetailsAndUnverifiedDidSignature<
	KiltDidKeyId,
	KiltAccountId,
	KiltBlockNumber,
	KiltWeb3Name,
	KiltLinkableAccountId,
	ConsumerBlockNumber,
	const MAX_REVEALED_LEAVES_COUNT: u32,
> {
	/// The parts of the subject's DID details revealed in the DIP proof.
	pub(crate) revealed_leaves: BoundedVec<
		RevealedDidMerkleProofLeaf<KiltDidKeyId, KiltAccountId, KiltBlockNumber, KiltWeb3Name, KiltLinkableAccountId>,
		ConstU32<MAX_REVEALED_LEAVES_COUNT>,
	>,
	/// The cross-chain DID signature.
	pub(crate) signature: TimeBoundDidSignature<ConsumerBlockNumber>,
}

impl<
		KiltDidKeyId,
		KiltAccountId,
		KiltBlockNumber,
		KiltWeb3Name,
		KiltLinkableAccountId,
		ConsumerBlockNumber,
		const MAX_REVEALED_LEAVES_COUNT: u32,
	>
	DipRevealedDetailsAndUnverifiedDidSignature<
		KiltDidKeyId,
		KiltAccountId,
		KiltBlockNumber,
		KiltWeb3Name,
		KiltLinkableAccountId,
		ConsumerBlockNumber,
		MAX_REVEALED_LEAVES_COUNT,
	> where
	ConsumerBlockNumber: PartialOrd,
{
	/// Verifies that the DIP proof signature is anchored to a block that has
	/// not passed on the consumer chain.
	pub fn verify_signature_time(
		self,
		block_number: &ConsumerBlockNumber,
	) -> Result<
		DipRevealedDetailsAndVerifiedDidSignatureFreshness<
			KiltDidKeyId,
			KiltAccountId,
			KiltBlockNumber,
			KiltWeb3Name,
			KiltLinkableAccountId,
			MAX_REVEALED_LEAVES_COUNT,
		>,
		Error,
	> {
		ensure!(self.signature.valid_until >= *block_number, Error::InvalidSignatureTime);
		Ok(DipRevealedDetailsAndVerifiedDidSignatureFreshness {
			revealed_leaves: self.revealed_leaves,
			signature: self.signature.signature,
		})
	}
}

/// A DIP proof whose information has been verified and whose signature has been
/// verified not to be expired, but that yet does not contain information as to
/// which of the revealed keys has generated the signature.
///
/// The generic types indicate the following:
/// * `KiltDidKeyId`: The DID key ID type configured by the KILT chain.
/// * `KiltAccountId`: The `AccountId` type configured by the KILT chain.
/// * `KiltBlockNumber`: The `BlockNumber` type configured by the KILT chain.
/// * `KiltWeb3Name`: The web3name type configured by the KILT chain.
/// * `KiltLinkableAccountId`: The linkable account ID type configured by the
///   KILT chain.
/// * `MAX_REVEALED_LEAVES_COUNT`: The maximum number of leaves revealable in
///   the proof.
#[derive(Debug, PartialEq, Eq)]
pub struct DipRevealedDetailsAndVerifiedDidSignatureFreshness<
	KiltDidKeyId,
	KiltAccountId,
	KiltBlockNumber,
	KiltWeb3Name,
	KiltLinkableAccountId,
	const MAX_REVEALED_LEAVES_COUNT: u32,
> {
	/// The parts of the subject's DID details revealed in the DIP proof.
	pub(crate) revealed_leaves: BoundedVec<
		RevealedDidMerkleProofLeaf<KiltDidKeyId, KiltAccountId, KiltBlockNumber, KiltWeb3Name, KiltLinkableAccountId>,
		ConstU32<MAX_REVEALED_LEAVES_COUNT>,
	>,
	/// The cross-chain DID signature without time information.
	pub(crate) signature: DidSignature,
}

impl<
		KiltDidKeyId,
		KiltAccountId,
		KiltBlockNumber,
		KiltWeb3Name,
		KiltLinkableAccountId,
		const MAX_REVEALED_LEAVES_COUNT: u32,
	>
	DipRevealedDetailsAndVerifiedDidSignatureFreshness<
		KiltDidKeyId,
		KiltAccountId,
		KiltBlockNumber,
		KiltWeb3Name,
		KiltLinkableAccountId,
		MAX_REVEALED_LEAVES_COUNT,
	>
{
	/// Iterates over the revealed DID leaves to find the ones that generated a
	/// valid signature for the provided payload.
	pub fn retrieve_signing_leaves_for_payload(
		self,
		payload: &[u8],
	) -> Result<
		DipOriginInfo<
			KiltDidKeyId,
			KiltAccountId,
			KiltBlockNumber,
			KiltWeb3Name,
			KiltLinkableAccountId,
			MAX_REVEALED_LEAVES_COUNT,
		>,
		Error,
	> {
		let revealed_verification_keys = self.revealed_leaves.iter().filter(|leaf| {
			matches!(
				leaf,
				RevealedDidMerkleProofLeaf::DidKey(RevealedDidKey {
					relationship: DidKeyRelationship::Verification(_),
					..
				})
			)
		});
		let signing_leaves_indices: Vec<_> = revealed_verification_keys
			.enumerate()
			.filter(|(_, revealed_verification_key)| {
				let RevealedDidMerkleProofLeaf::DidKey(RevealedDidKey {
					details:
						DidPublicKeyDetails {
							key: DidPublicKey::PublicVerificationKey(verification_key),
							..
						},
					..
				}) = revealed_verification_key
				else {
					return false;
				};
				verification_key.verify_signature(payload, &self.signature).is_ok()
			})
			.map(|(index, _)| u32::saturated_from(index))
			.collect();

		ensure!(!signing_leaves_indices.is_empty(), Error::InvalidDidKeyRevealed);

		let signing_leaves_indices_vector = signing_leaves_indices.try_into().map_err(|_| {
			log::error!(target: "dip::consumer::DipRevealedDetailsAndVerifiedDidSignatureFreshnessV0", "Failed to convert vector of signing leaf indices into BoundedVec<u8, {MAX_REVEALED_LEAVES_COUNT}>.");
			Error::Internal
		})?;

		Ok(DipOriginInfo {
			revealed_leaves: self.revealed_leaves,
			signing_leaves_indices: signing_leaves_indices_vector,
		})
	}
}
