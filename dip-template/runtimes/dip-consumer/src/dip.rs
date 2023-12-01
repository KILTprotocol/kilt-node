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

use did::{did_details::DidVerificationKey, DidVerificationKeyRelationship};
use dip_provider_runtime_template::{AccountId as ProviderAccountId, Runtime as ProviderRuntime};
use frame_support::traits::Contains;
use frame_system::EnsureSigned;
use kilt_dip_support::{
	traits::DipCallOriginFilter, KiltVersionedSiblingProviderVerifier, RelayStateRootsViaRelayStorePallet,
};
use pallet_dip_consumer::traits::IdentityProofVerifier;
use sp_core::ConstU32;
use sp_runtime::traits::BlakeTwo256;

use crate::{AccountId, DidIdentifier, Runtime, RuntimeCall, RuntimeOrigin};

pub type MerkleProofVerifierOutput = <ProofVerifier as IdentityProofVerifier<Runtime>>::VerificationResult;
pub type ProofVerifier = KiltVersionedSiblingProviderVerifier<
	ProviderRuntime,
	ConstU32<2_000>,
	RelayStateRootsViaRelayStorePallet<Runtime>,
	BlakeTwo256,
	DipCallFilter,
	10,
	10,
	50,
>;

impl pallet_dip_consumer::Config for Runtime {
	type DipCallOriginFilter = PreliminaryDipOriginFilter;
	type DispatchOriginCheck = EnsureSigned<AccountId>;
	type Identifier = DidIdentifier;
	type LocalIdentityInfo = u128;
	type ProofVerifier = ProofVerifier;
	type RuntimeCall = RuntimeCall;
	type RuntimeOrigin = RuntimeOrigin;
	type WeightInfo = ();
}

pub struct PreliminaryDipOriginFilter;

impl Contains<RuntimeCall> for PreliminaryDipOriginFilter {
	fn contains(t: &RuntimeCall) -> bool {
		matches!(
			t,
			RuntimeCall::PostIt { .. }
				| RuntimeCall::Utility(pallet_utility::Call::batch { .. })
				| RuntimeCall::Utility(pallet_utility::Call::batch_all { .. })
				| RuntimeCall::Utility(pallet_utility::Call::force_batch { .. })
		)
	}
}

fn derive_verification_key_relationship(call: &RuntimeCall) -> Option<DidVerificationKeyRelationship> {
	match call {
		RuntimeCall::PostIt { .. } => Some(DidVerificationKeyRelationship::Authentication),
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

pub enum DipCallFilterError {
	BadOrigin,
	WrongVerificationRelationship,
}

pub struct DipCallFilter;

impl DipCallOriginFilter<RuntimeCall> for DipCallFilter {
	type Error = DipCallFilterError;
	type OriginInfo = (DidVerificationKey<ProviderAccountId>, DidVerificationKeyRelationship);
	type Success = ();

	// Accepts only a DipOrigin for the DidLookup pallet calls.
	fn check_call_origin_info(call: &RuntimeCall, info: &Self::OriginInfo) -> Result<Self::Success, Self::Error> {
		let key_relationship =
			single_key_relationship([call].into_iter()).map_err(|_| DipCallFilterError::BadOrigin)?;
		if info.1 == key_relationship {
			Ok(())
		} else {
			Err(DipCallFilterError::WrongVerificationRelationship)
		}
	}
}

impl pallet_relay_store::Config for Runtime {
	type MaxRelayBlocksStored = ConstU32<100>;
	type WeightInfo = ();
}
