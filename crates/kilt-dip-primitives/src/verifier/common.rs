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

use pallet_dip_provider::IdentityCommitmentVersion;
use parity_scale_codec::Encode;
use sp_core::storage::StorageKey;

pub mod latest {
	pub use super::v0::{DipMerkleProofAndDidSignature, ParachainRootStateProof};
}

pub mod v0 {
	use did::did_details::DidPublicKey;
	use parity_scale_codec::{Decode, Encode};
	use scale_info::TypeInfo;
	use sp_core::RuntimeDebug;
	use sp_runtime::traits::Hash;

	use crate::{
		did::{DidSignatureVerificationError, TimeBoundDidSignature},
		merkle::{
			DidKeyRelationship, DidMerkleProof, DidMerkleProofVerificationError, RevealedDidKey,
			RevealedDidMerkleProofLeaf,
		},
		utils::OutputOf,
		BoundedBlindedValue,
	};

	#[derive(Encode, Decode, PartialEq, Eq, PartialOrd, Ord, RuntimeDebug, TypeInfo, Clone)]
	pub struct ParachainRootStateProof<RelayBlockHeight> {
		/// The relaychain block height for which the proof has been generated.
		pub(crate) relay_block_height: RelayBlockHeight,
		/// The raw state proof.
		pub(crate) proof: BoundedBlindedValue<u8>,
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl<RelayBlockHeight, Context> kilt_support::traits::GetWorstCase<Context>
		for ParachainRootStateProof<RelayBlockHeight>
	where
		RelayBlockHeight: Default,
	{
		fn worst_case(context: Context) -> Self {
			Self {
				relay_block_height: RelayBlockHeight::default(),
				proof: BoundedBlindedValue::worst_case(context),
			}
		}
	}

	#[derive(Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo, Clone)]
	pub struct DipMerkleProofAndDidSignature<KeyId, AccountId, BlockNumber, Web3Name, LinkedAccountId, LocalBlockNumber> {
		/// The DIP Merkle proof revealing some leaves about the DID subject's
		/// identity.
		pub(crate) leaves: DidMerkleProof<KeyId, AccountId, BlockNumber, Web3Name, LinkedAccountId>,
		/// The cross-chain DID signature.
		pub(crate) signature: TimeBoundDidSignature<LocalBlockNumber>,
	}

	impl<KeyId, AccountId, BlockNumber, Web3Name, LinkedAccountId, LocalBlockNumber>
		DipMerkleProofAndDidSignature<KeyId, AccountId, BlockNumber, Web3Name, LinkedAccountId, LocalBlockNumber>
	where
		KeyId: Encode,
		AccountId: Encode,
		BlockNumber: Encode,
		Web3Name: Encode,
		LinkedAccountId: Encode,
	{
		pub fn verify_merkle_proof_against_commitment<Hasher>(
			self,
			commitment: &OutputOf<Hasher>,
			max_leaves_revealed: usize,
		) -> Result<
			RevealedLeavesAndDidSignature<KeyId, AccountId, BlockNumber, Web3Name, LinkedAccountId, LocalBlockNumber>,
			DidMerkleProofVerificationError,
		>
		where
			Hasher: Hash,
		{
			let revealed_leaves = self
				.leaves
				.verify_against_commitment::<Hasher>(commitment, max_leaves_revealed)?;
			Ok(RevealedLeavesAndDidSignature {
				signature: self.signature,
				leaves: revealed_leaves.into_iter().collect(),
			})
		}
	}

	pub struct RevealedLeavesAndDidSignature<KeyId, AccountId, BlockNumber, Web3Name, LinkedAccountId, LocalBlockNumber> {
		pub(crate) leaves: Vec<RevealedDidMerkleProofLeaf<KeyId, AccountId, BlockNumber, Web3Name, LinkedAccountId>>,
		pub(crate) signature: TimeBoundDidSignature<LocalBlockNumber>,
	}

	impl<KeyId, AccountId, BlockNumber, Web3Name, LinkedAccountId, LocalBlockNumber>
		RevealedLeavesAndDidSignature<KeyId, AccountId, BlockNumber, Web3Name, LinkedAccountId, LocalBlockNumber>
	where
		LocalBlockNumber: PartialOrd + Clone,
	{
		pub fn extract_signing_key_for_payload(
			&self,
			payload: &[u8],
			current_block_number: LocalBlockNumber,
		) -> Result<&RevealedDidKey<KeyId, BlockNumber, AccountId>, DidSignatureVerificationError> {
			let signature = self
				.signature
				.clone()
				.extract_signature_if_not_expired(current_block_number)?;

			let mut revealed_did_verification_keys = self.leaves.iter().filter_map(|leaf| {
				// Skip if the leaf is not a DID key leaf.
				let RevealedDidMerkleProofLeaf::DidKey(did_key) = leaf else {
					return None;
				};
				// Skip if the DID key is not a verification key.
				let DidPublicKey::PublicVerificationKey(_) = did_key.details.key else {
					return None;
				};
				// Skip if the verification relationship is not for signatures (should never
				// fail, but we check just to be sure).
				let DidKeyRelationship::Verification(_) = did_key.relationship else {
					return None;
				};
				Some(did_key)
			});

			let signing_key_details = revealed_did_verification_keys
				.find(|did_key| {
					let DidPublicKey::PublicVerificationKey(ref mapped_key) = did_key.details.key else {
						return false;
					};
					mapped_key.verify_signature(payload, &signature).is_ok()
				})
				.ok_or(DidSignatureVerificationError::SignerNotFound)?;
			Ok(signing_key_details)
		}
	}
}

pub(crate) fn calculate_parachain_head_storage_key(para_id: u32) -> StorageKey {
	StorageKey(
		[
			frame_support::storage::storage_prefix(b"Paras", b"Heads").as_slice(),
			sp_io::hashing::twox_64(para_id.encode().as_ref()).as_slice(),
			para_id.encode().as_slice(),
		]
		.concat(),
	)
}

pub(crate) fn calculate_dip_identity_commitment_storage_key_for_runtime<Runtime>(
	subject: &Runtime::Identifier,
	version: IdentityCommitmentVersion,
) -> StorageKey
where
	Runtime: pallet_dip_provider::Config,
{
	StorageKey(pallet_dip_provider::IdentityCommitments::<Runtime>::hashed_key_for(
		subject, version,
	))
}
