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

use scale_info::TypeInfo;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, TypeInfo)]
#[cfg_attr(test, derive(enum_iterator::Sequence))]
pub enum MerkleProofError {
	InvalidProof,
	RequiredLeafNotRevealed,
	ResultDecoding,
}

impl From<MerkleProofError> for u8 {
	fn from(value: MerkleProofError) -> Self {
		match value {
			// DO NOT USE 0
			// Errors of different sub-parts are separated by a `u8::MAX`.
			// A value of 0 would make it confusing whether it's the previous sub-part error (u8::MAX)
			// or the new sub-part error (u8::MAX + 0).
			MerkleProofError::InvalidProof => 1,
			MerkleProofError::RequiredLeafNotRevealed => 2,
			MerkleProofError::ResultDecoding => 3,
		}
	}
}

#[test]
fn merkle_proof_error_value_never_zero() {
	assert!(
		enum_iterator::all::<MerkleProofError>().all(|e| u8::from(e) != 0),
		"One of the u8 values for the error is 0, which is not allowed."
	);
}

#[test]
fn merkle_proof_error_value_not_duplicated() {
	enum_iterator::all::<MerkleProofError>().fold(
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
