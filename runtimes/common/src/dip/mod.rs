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

use did::{did_details::DidPublicKeyDetails, DidVerificationKeyRelationship};
use frame_support::RuntimeDebug;
use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::vec::Vec;

pub mod receiver;
pub mod sender;

#[cfg(test)]
mod tests;

#[derive(Clone, RuntimeDebug, Encode, Decode, PartialEq, Eq, TypeInfo, PartialOrd, Ord)]
pub enum KeyRelationship {
	Encryption,
	Verification(DidVerificationKeyRelationship),
}

impl From<DidVerificationKeyRelationship> for KeyRelationship {
	fn from(value: DidVerificationKeyRelationship) -> Self {
		Self::Verification(value)
	}
}

#[derive(Clone, Encode, Decode, PartialEq, Eq, PartialOrd, Ord, RuntimeDebug, TypeInfo)]
pub struct KeyReferenceKey<KeyId>(pub KeyId, pub KeyRelationship);
#[derive(Clone, Encode, Decode, PartialEq, Eq, PartialOrd, Ord, RuntimeDebug, TypeInfo)]
pub struct KeyReferenceValue;

#[derive(Clone, Encode, Decode, PartialEq, Eq, PartialOrd, Ord, RuntimeDebug, TypeInfo)]
pub struct KeyDetailsKey<KeyId>(pub KeyId);
#[derive(Clone, Encode, Decode, PartialEq, Eq, PartialOrd, Ord, RuntimeDebug, TypeInfo)]
pub struct KeyDetailsValue<BlockNumber>(pub DidPublicKeyDetails<BlockNumber>);

#[derive(Clone, Encode, Decode, PartialEq, Eq, PartialOrd, Ord, RuntimeDebug, TypeInfo)]
pub enum ProofLeaf<KeyId, BlockNumber> {
	// The key and value for the leaves of a merkle proof that contain a reference
	// (by ID) to the key details, provided in a separate leaf.
	KeyReference(KeyReferenceKey<KeyId>, KeyReferenceValue),
	// The key and value for the leaves of a merkle proof that contain the actual
	// details of a DID public key. The key is the ID of the key, and the value is its details, including creation
	// block number.
	KeyDetails(KeyDetailsKey<KeyId>, KeyDetailsValue<BlockNumber>),
}

impl<KeyId, BlockNumber> ProofLeaf<KeyId, BlockNumber>
where
	KeyId: Encode,
	BlockNumber: Encode,
{
	pub(crate) fn encoded_key(&self) -> Vec<u8> {
		match self {
			ProofLeaf::KeyReference(key, _) => key.encode(),
			ProofLeaf::KeyDetails(key, _) => key.encode(),
		}
	}

	pub(crate) fn encoded_value(&self) -> Vec<u8> {
		match self {
			ProofLeaf::KeyReference(_, value) => value.encode(),
			ProofLeaf::KeyDetails(_, value) => value.encode(),
		}
	}
}
