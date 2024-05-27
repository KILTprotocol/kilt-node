// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2024 BOTLabs GmbH

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

use did::KeyIdOf;
use frame_system::pallet_prelude::BlockNumberFor;
use kilt_dip_primitives::DidMerkleProof;
use pallet_did_lookup::linkable_account::LinkableAccountId;
use pallet_dip_provider::{
	traits::{IdentityCommitmentGenerator, IdentityProvider},
	IdentityCommitmentVersion, IdentityOf,
};
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_std::marker::PhantomData;

use crate::dip::did::LinkedDidInfoOf;

pub mod v0;

#[cfg(test)]
mod tests;

/// Type of the Merkle proof revealing parts of the DIP identity of a given DID
/// subject.
pub type DidMerkleProofOf<T> = DidMerkleProof<
	KeyIdOf<T>,
	<T as frame_system::Config>::AccountId,
	BlockNumberFor<T>,
	<T as pallet_web3_names::Config>::Web3Name,
	LinkableAccountId,
>;

/// Type of a complete DIP Merkle proof.
#[derive(Encode, Decode, RuntimeDebug, PartialEq, Eq, TypeInfo)]
pub struct CompleteMerkleProof<Root, Proof> {
	/// The Merkle root.
	pub root: Root,
	/// The Merkle proof revealing parts of the commitment that verify against
	/// the provided root.
	pub proof: Proof,
}

#[derive(Clone, RuntimeDebug, Encode, Decode, TypeInfo, PartialEq)]
pub enum DidMerkleProofError {
	UnsupportedVersion,
	KeyNotFound,
	LinkedAccountNotFound,
	Web3NameNotFound,
	TooManyLeaves,
	Internal,
}

impl From<DidMerkleProofError> for u16 {
	fn from(value: DidMerkleProofError) -> Self {
		match value {
			// DO NOT USE 0
			// Errors of different sub-parts are separated by a `u8::MAX`.
			// A value of 0 would make it confusing whether it's the previous sub-part error (u8::MAX)
			// or the new sub-part error (u8::MAX + 0).
			DidMerkleProofError::UnsupportedVersion => 1,
			DidMerkleProofError::KeyNotFound => 2,
			DidMerkleProofError::LinkedAccountNotFound => 3,
			DidMerkleProofError::Web3NameNotFound => 4,
			DidMerkleProofError::TooManyLeaves => 5,
			DidMerkleProofError::Internal => u16::MAX,
		}
	}
}

/// Type implementing the [`IdentityCommitmentGenerator`] and generating a
/// Merkle root of the provided identity details, according to the description
/// provided in the [README.md](./README.md),
pub struct DidMerkleRootGenerator<T>(PhantomData<T>);

impl<Runtime, const MAX_LINKED_ACCOUNT: u32> IdentityCommitmentGenerator<Runtime> for DidMerkleRootGenerator<Runtime>
where
	Runtime: did::Config + pallet_did_lookup::Config + pallet_web3_names::Config + pallet_dip_provider::Config,
	Runtime::IdentityProvider: IdentityProvider<Runtime, Success = LinkedDidInfoOf<Runtime, MAX_LINKED_ACCOUNT>>,
{
	type Error = DidMerkleProofError;
	type Output = Runtime::Hash;

	fn generate_commitment(
		_identifier: &Runtime::Identifier,
		identity: &IdentityOf<Runtime>,
		version: IdentityCommitmentVersion,
	) -> Result<Self::Output, Self::Error> {
		match version {
			0 => v0::generate_commitment::<Runtime, MAX_LINKED_ACCOUNT>(identity),
			_ => Err(DidMerkleProofError::UnsupportedVersion),
		}
	}
}

impl<Runtime> DidMerkleRootGenerator<Runtime>
where
	Runtime: did::Config + pallet_did_lookup::Config + pallet_web3_names::Config,
{
	pub fn generate_proof<'a, K, A, const MAX_LINKED_ACCOUNT: u32>(
		identity: &LinkedDidInfoOf<Runtime, MAX_LINKED_ACCOUNT>,
		version: IdentityCommitmentVersion,
		key_ids: K,
		should_include_web3_name: bool,
		account_ids: A,
	) -> Result<CompleteMerkleProof<Runtime::Hash, DidMerkleProofOf<Runtime>>, DidMerkleProofError>
	where
		K: Iterator<Item = &'a KeyIdOf<Runtime>>,
		A: Iterator<Item = &'a LinkableAccountId>,
	{
		match version {
			0 => v0::generate_proof(identity, key_ids, should_include_web3_name, account_ids),
			_ => Err(DidMerkleProofError::UnsupportedVersion),
		}
	}
}
