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

use did::{did_details::DidVerificationKey, DidVerificationKeyRelationship, KeyIdOf};
use dip_provider_runtime_template::{Runtime as ProviderRuntime, Web3Name};
use frame_support::traits::Contains;
use kilt_dip_support::{
	traits::{DipCallOriginFilter, FrameSystemDidSignatureContext},
	DipSiblingProviderStateProofVerifier, KiltDipCommitmentsForDipProviderPallet, RococoStateRootsViaRelayStorePallet,
};
use pallet_did_lookup::linkable_account::LinkableAccountId;
use pallet_dip_consumer::traits::IdentityProofVerifier;
use sp_core::ConstU32;
use sp_runtime::traits::BlakeTwo256;

use crate::{AccountId, DidIdentifier, Runtime, RuntimeCall, RuntimeOrigin};

pub type ProofVerifier = DipSiblingProviderStateProofVerifier<
	RococoStateRootsViaRelayStorePallet<Runtime>,
	ConstU32<2_000>,
	KiltDipCommitmentsForDipProviderPallet<ProviderRuntime>,
	AccountId,
	BlakeTwo256,
	KeyIdOf<ProviderRuntime>,
	Web3Name,
	LinkableAccountId,
	10,
	10,
	u128,
	// Signatures are valid for 50 blocks
	FrameSystemDidSignatureContext<Runtime, 50>,
	DipCallFilter,
>;

impl pallet_dip_consumer::Config for Runtime {
	type DipCallOriginFilter = PreliminaryDipOriginFilter;
	type Identifier = DidIdentifier;
	type IdentityProof = <ProofVerifier as IdentityProofVerifier<RuntimeCall, DidIdentifier>>::Proof;
	type LocalIdentityInfo = u128;
	type ProofVerifier = ProofVerifier;
	type RuntimeCall = RuntimeCall;
	type RuntimeOrigin = RuntimeOrigin;
}

pub struct PreliminaryDipOriginFilter;

impl Contains<RuntimeCall> for PreliminaryDipOriginFilter {
	fn contains(t: &RuntimeCall) -> bool {
		matches!(
			t,
			RuntimeCall::DidLookup { .. }
				| RuntimeCall::Utility(pallet_utility::Call::batch { .. })
				| RuntimeCall::Utility(pallet_utility::Call::batch_all { .. })
				| RuntimeCall::Utility(pallet_utility::Call::force_batch { .. })
		)
	}
}

fn derive_verification_key_relationship(call: &RuntimeCall) -> Option<DidVerificationKeyRelationship> {
	match call {
		RuntimeCall::DidLookup { .. } => Some(DidVerificationKeyRelationship::Authentication),
		RuntimeCall::Utility(pallet_utility::Call::batch { calls }) => single_key_relationship(calls.iter()).ok(),
		RuntimeCall::Utility(pallet_utility::Call::batch_all { calls }) => single_key_relationship(calls.iter()).ok(),
		RuntimeCall::Utility(pallet_utility::Call::force_batch { calls }) => single_key_relationship(calls.iter()).ok(),
		_ => None,
	}
}

// Taken and adapted from `impl
// did::DeriveDidCallAuthorizationVerificationKeyRelationship for RuntimeCall`
// in Spiritnet/Peregrine runtime.
fn single_key_relationship<'a>(
	calls: impl Iterator<Item = &'a RuntimeCall>,
) -> Result<DidVerificationKeyRelationship, ()> {
	let mut calls = calls.peekable();
	let first_call_relationship = calls
		.peek()
		.and_then(|k| derive_verification_key_relationship(k))
		.ok_or(())?;
	calls
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
	type OriginInfo = (DidVerificationKey, DidVerificationKeyRelationship);
	type Success = ();

	// Accepts only a DipOrigin for the DidLookup pallet calls.
	fn check_call_origin_info(call: &RuntimeCall, info: &Self::OriginInfo) -> Result<Self::Success, Self::Error> {
		let key_relationship = single_key_relationship([call].into_iter())?;
		if info.1 == key_relationship {
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
			single_key_relationship(vec![did_lookup_call].iter()),
			Ok(DidVerificationKeyRelationship::Authentication)
		);
		// Can't call System functions with a DID key (hence a DIP origin)
		let system_call = RuntimeCall::System(frame_system::Call::remark { remark: vec![] });
		assert_err!(single_key_relationship(vec![system_call].iter()), ());
		// Can't call empty batch with a DID key
		let empty_batch_call = RuntimeCall::Utility(pallet_utility::Call::batch_all { calls: vec![] });
		assert_err!(single_key_relationship(vec![empty_batch_call].iter()), ());
		// Can call batch with a DipLookup with an authentication key
		let did_lookup_batch_call = RuntimeCall::Utility(pallet_utility::Call::batch_all {
			calls: vec![pallet_did_lookup::Call::associate_sender {}.into()],
		});
		assert_eq!(
			single_key_relationship(vec![did_lookup_batch_call].iter()),
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
		assert_err!(single_key_relationship(vec![did_lookup_batch_call].iter()), ());
	}
}

impl pallet_relay_store::Config for Runtime {
	type MaxRelayBlocksStored = ConstU32<100>;
}
