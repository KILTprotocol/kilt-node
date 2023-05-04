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

use did::DidVerificationKeyRelationship;
use dip_support::VersionedIdentityProof;
use pallet_dip_consumer::traits::DipCallOriginFilter;
use runtime_common::dip::{
	consumer::{DidMerkleProofVerifier, VerificationResult},
	ProofLeaf,
};
use sp_std::vec::Vec;

use crate::{AccountId, BlockNumber, DidIdentifier, Hash, Hasher, Runtime, RuntimeCall, RuntimeEvent, RuntimeOrigin};

impl pallet_dip_consumer::Config for Runtime {
	type DipCallOriginFilter = DipCallFilter;
	type Identifier = DidIdentifier;
	type IdentityDetails = u128;
	type Proof = VersionedIdentityProof<Vec<Vec<u8>>, ProofLeaf<Hash, BlockNumber>>;
	type ProofDigest = Hash;
	type ProofVerifier = DidMerkleProofVerifier<Hash, BlockNumber, Hasher, u128, AccountId>;
	type RuntimeCall = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
}

fn derive_verification_key_relationship(call: &RuntimeCall) -> Option<DidVerificationKeyRelationship> {
	match call {
		RuntimeCall::DidLookup { .. } => Some(DidVerificationKeyRelationship::Authentication),
		RuntimeCall::Utility(pallet_utility::Call::batch { calls }) => single_key_relationship(calls).ok(),
		RuntimeCall::Utility(pallet_utility::Call::batch_all { calls }) => single_key_relationship(calls).ok(),
		RuntimeCall::Utility(pallet_utility::Call::force_batch { calls }) => single_key_relationship(calls).ok(),
		_ => None,
	}
}

// Taken and adapted from `impl
// did::DeriveDidCallAuthorizationVerificationKeyRelationship for RuntimeCall`
// in Spiritnet/Peregrine runtime.
fn single_key_relationship(calls: &[RuntimeCall]) -> Result<DidVerificationKeyRelationship, ()> {
	let first_call_relationship = calls.get(0).and_then(derive_verification_key_relationship).ok_or(())?;
	calls
		.iter()
		.skip(1)
		.map(derive_verification_key_relationship)
		.try_fold(first_call_relationship, |acc, next| {
			if next == Some(acc) {
				Ok(acc)
			} else {
				Err(())
			}
		})
}

pub struct DipCallFilter;

impl DipCallOriginFilter<RuntimeCall> for DipCallFilter {
	type Error = ();
	type Proof = VerificationResult<BlockNumber>;
	type Success = ();

	// Accepts only a DipOrigin for the DidLookup pallet calls.
	fn check_proof(call: RuntimeCall, proof: Self::Proof) -> Result<Self::Success, Self::Error> {
		let key_relationship = single_key_relationship(&[call])?;
		if proof.0.iter().any(|l| l.relationship == key_relationship.into()) {
			Ok(())
		} else {
			Err(())
		}
	}
}

#[cfg(test)]
mod dip_call_origin_filter_tests {
	use super::*;

	use frame_support::assert_err;

	#[test]
	fn test_key_relationship_derivation() {
		// Can call DidLookup functions with an authentication key
		let did_lookup_call = RuntimeCall::DidLookup(pallet_did_lookup::Call::associate_sender {});
		assert_eq!(
			single_key_relationship(&[did_lookup_call]),
			Ok(DidVerificationKeyRelationship::Authentication)
		);
		// Can't call System functions with a DID key (hence a DIP origin)
		let system_call = RuntimeCall::System(frame_system::Call::remark { remark: vec![] });
		assert_err!(single_key_relationship(&[system_call]), ());
		// Can't call empty batch with a DID key
		let empty_batch_call = RuntimeCall::Utility(pallet_utility::Call::batch_all { calls: vec![] });
		assert_err!(single_key_relationship(&[empty_batch_call]), ());
		// Can call batch with a DipLookup with an authentication key
		let did_lookup_batch_call = RuntimeCall::Utility(pallet_utility::Call::batch_all {
			calls: vec![pallet_did_lookup::Call::associate_sender {}.into()],
		});
		assert_eq!(
			single_key_relationship(&[did_lookup_batch_call]),
			Ok(DidVerificationKeyRelationship::Authentication)
		);
		// Can't call a batch with different required keys
		let did_lookup_batch_call = RuntimeCall::Utility(pallet_utility::Call::batch_all {
			calls: vec![
				// Authentication key
				pallet_did_lookup::Call::associate_sender {}.into(),
				// No key
				frame_system::Call::remark { remark: vec![] }.into(),
			],
		});
		assert_err!(single_key_relationship(&[did_lookup_batch_call]), ());
	}
}
