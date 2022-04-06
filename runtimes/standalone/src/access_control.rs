// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2022 BOTLabs GmbH

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

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::{traits::InstanceFilter, ensure, RuntimeDebug};

use crate::Call;

impl did::DeriveDidCallAuthorizationVerificationType for Call {
	fn derive_verification_key_relationship(&self) -> did::DeriveDidVerificationTypeResult {
		fn single_key_relationship(calls: &[Call]) -> did::DeriveDidVerificationTypeResult {
			let first_call_key = calls
				.get(0)
				.ok_or(did::RelationshipDeriveError::InvalidCallParameter)?
				.derive_verification_key_relationship()?;

			match first_call_key {
				// If the first call in the batch requires an inline signature, only certain DID operations can be part
				// of the batch.
				did::DidVerificationType::Inline => {
					let are_calls_allowed = calls
						.iter()
						.skip(1)
						.all(|call| {
							matches!(
								call,
								Call::Did(
									did::Call::set_delegation_key { .. }
										| did::Call::set_attestation_key { .. } | did::Call::add_key_agreement_key { .. }
										| did::Call::add_service_endpoint { .. }
								)
							)
						});
					ensure!(are_calls_allowed, did::RelationshipDeriveError::InvalidCallParameter);
					// The verification logic for the whole batch is therefore `inline`
					Ok(first_call_key)
				}
				// If the first call in the batch requires a stored key, then all calls must have the same key
				// relationship. This means that Inline calls are not allowed, e.g., it is not possible to squeeze in a
				// DID creation operation.
				did::DidVerificationType::StoredVerificationKey(key_rel) => calls
					.iter()
					.skip(1)
					.map(Call::derive_verification_key_relationship)
					.try_fold(
						did::DidVerificationType::with_verification_key(key_rel),
						|acc, next| match next {
							// Step successful if the next key relationship is of the same type as the current one.
							Ok(key_type) if key_type == acc => Ok(acc),
							Err(_) => next,
							_ => Err(did::RelationshipDeriveError::InvalidCallParameter),
						},
					),
			}
		}

		match self {
			Call::Attestation { .. } => Ok(did::DidVerificationType::with_verification_key(
				did::DidVerificationKeyRelationship::AssertionMethod,
			)),
			Call::Ctype { .. } => Ok(did::DidVerificationType::with_verification_key(
				did::DidVerificationKeyRelationship::AssertionMethod,
			)),
			Call::Delegation { .. } => Ok(did::DidVerificationType::with_verification_key(
				did::DidVerificationKeyRelationship::CapabilityDelegation,
			)),
			// DID creation requires inline verification.
			Call::Did(did::Call::create { .. }) => Ok(did::DidVerificationType::inline()),
			Call::Did { .. } => Ok(did::DidVerificationType::with_verification_key(
				did::DidVerificationKeyRelationship::Authentication,
			)),
			Call::Web3Names { .. } => Ok(did::DidVerificationType::with_verification_key(
				did::DidVerificationKeyRelationship::Authentication,
			)),
			Call::DidLookup { .. } => Ok(did::DidVerificationType::with_verification_key(
				did::DidVerificationKeyRelationship::Authentication,
			)),
			Call::Utility(pallet_utility::Call::batch { calls }) => single_key_relationship(&calls[..]),
			Call::Utility(pallet_utility::Call::batch_all { calls }) => single_key_relationship(&calls[..]),
			#[cfg(not(feature = "runtime-benchmarks"))]
			_ => Err(did::RelationshipDeriveError::NotCallableByDid),
			// By default, returns the authentication key
			#[cfg(feature = "runtime-benchmarks")]
			_ => Ok(did::DidVerificationKeyRelationship::Authentication),
		}
	}

	// Always return a System::remark() extrinsic call
	#[cfg(feature = "runtime-benchmarks")]
	fn get_call_for_did_call_benchmark() -> Call {
		Call::System(frame_system::Call::remark { remark: vec![] })
	}
}
