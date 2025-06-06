// KILT Blockchain – <https://kilt.io>
// Copyright (C) 2025, KILT Foundation

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

// If you feel like getting in touch with us, you can do so at <hello@kilt.io>

use crate::Error;

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(test, derive(enum_iterator::Sequence))]
pub enum DipRelaychainStateProofVerifierError<DidOriginError> {
	UnsupportedVersion,
	ProofComponentTooLarge(u8),
	ProofVerification(Error),
	DidOriginError(DidOriginError),
	Internal,
}

impl<DidOriginError> From<DipRelaychainStateProofVerifierError<DidOriginError>> for u16
where
	DidOriginError: Into<u8>,
{
	#[allow(clippy::as_conversions)]
	#[allow(clippy::arithmetic_side_effects)]
	fn from(value: DipRelaychainStateProofVerifierError<DidOriginError>) -> Self {
		match value {
			// DO NOT USE 0
			// Errors of different sub-parts are separated by a `u8::MAX`.
			// A value of 0 would make it confusing whether it's the previous sub-part error (u8::MAX)
			// or the new sub-part error (u8::MAX + 0).
			DipRelaychainStateProofVerifierError::UnsupportedVersion => 1,
			DipRelaychainStateProofVerifierError::ProofComponentTooLarge(component_id) => {
				u8::MAX as u16 + component_id as u16
			}
			DipRelaychainStateProofVerifierError::ProofVerification(error) => {
				u8::MAX as u16 * 2 + u8::from(error) as u16
			}
			DipRelaychainStateProofVerifierError::DidOriginError(error) => u8::MAX as u16 * 3 + error.into() as u16,
			DipRelaychainStateProofVerifierError::Internal => u16::MAX,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn dip_relaychain_state_proof_verifier_error_value_never_zero() {
		assert!(
			enum_iterator::all::<DipRelaychainStateProofVerifierError<u8>>().all(|e| u16::from(e) != 0),
			"One of the u8 values for the error is 0, which is not allowed."
		);
	}

	#[test]
	fn dip_relaychain_state_proof_verifier_error_value_not_duplicated() {
		enum_iterator::all::<DipRelaychainStateProofVerifierError<u8>>().fold(
			sp_std::collections::btree_set::BTreeSet::<u16>::new(),
			|mut values, new_value| {
				let new_encoded_value = u16::from(new_value);
				// DidOriginError is generic, and we cannot test its constraints in this unit
				// test, so we skip it.
				if new_encoded_value == u8::MAX as u16 * 3 {
					return values;
				}
				assert!(
					values.insert(new_encoded_value),
					"Failed to add unique value {:#?} for error variant",
					new_encoded_value
				);
				values
			},
		);
	}
}
