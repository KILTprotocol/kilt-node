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

use scale_info::TypeInfo;

use crate::state_proofs::MerkleProofError;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, TypeInfo)]
#[cfg_attr(test, derive(enum_iterator::Sequence))]
pub enum Error {
	InvalidRelayHeader,
	RelayBlockNotFound,
	RelayStateRootNotFound,
	InvalidDidMerkleProof,
	TooManyLeavesRevealed,
	InvalidSignatureTime,
	InvalidDidKeyRevealed,
	ParaHeadMerkleProof(MerkleProofError),
	DipCommitmentMerkleProof(MerkleProofError),
	Internal,
}

impl From<Error> for u8 {
	fn from(value: Error) -> Self {
		match value {
			// DO NOT USE 0
			// Errors of different sub-parts are separated by a `u8::MAX`.
			// A value of 0 would make it confusing whether it's the previous sub-part error (u8::MAX)
			// or the new sub-part error (u8::MAX + 0).
			Error::InvalidRelayHeader => 1,
			Error::RelayBlockNotFound => 2,
			Error::RelayStateRootNotFound => 3,
			Error::InvalidDidMerkleProof => 4,
			Error::TooManyLeavesRevealed => 5,
			Error::InvalidSignatureTime => 6,
			Error::InvalidDidKeyRevealed => 7,
			Error::ParaHeadMerkleProof(error) => match error {
				MerkleProofError::InvalidProof => 11,
				MerkleProofError::RequiredLeafNotRevealed => 12,
				MerkleProofError::ResultDecoding => 13,
			},
			Error::DipCommitmentMerkleProof(error) => match error {
				MerkleProofError::InvalidProof => 21,
				MerkleProofError::RequiredLeafNotRevealed => 22,
				MerkleProofError::ResultDecoding => 23,
			},
			Error::Internal => u8::MAX,
		}
	}
}

#[test]
fn error_value_never_zero() {
	assert!(
		enum_iterator::all::<Error>().all(|e| u8::from(e) != 0),
		"One of the u8 values for the error is 0, which is not allowed."
	);
}

#[test]
fn error_value_not_duplicated() {
	enum_iterator::all::<Error>().fold(
		sp_std::collections::btree_set::BTreeSet::<u8>::new(),
		|mut values, new_value| {
			let new_encoded_value = u8::from(new_value);
			assert!(
				values.insert(new_encoded_value),
				"Failed to add unique value {:#?} for error variant",
				new_encoded_value
			);
			values
		},
	);
}
