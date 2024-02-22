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

use did::{DidVerificationKeyRelationship, KeyIdOf};
use dip_provider_runtime_template::{AccountId as ProviderAccountId, Runtime as ProviderRuntime};
use frame_support::traits::Contains;
use frame_system::{pallet_prelude::BlockNumberFor, EnsureSigned};
use kilt_dip_primitives::{
	traits::DipCallOriginFilter, KiltVersionedParachainVerifier, RelayStateRootsViaRelayStorePallet, RevealedDidKey,
};
use pallet_dip_consumer::traits::IdentityProofVerifier;
use rococo_runtime::Runtime as RelaychainRuntime;
use sp_core::ConstU32;
use sp_std::marker::PhantomData;

use crate::{weights, AccountId, DidIdentifier, Runtime, RuntimeCall, RuntimeOrigin};

pub type MerkleProofVerifierOutput = <ProofVerifier as IdentityProofVerifier<Runtime>>::VerificationResult;
/// The verifier logic assumes the provider is a sibling KILT parachain, the relaychain is a Rococo relaychain, and
/// that a KILT subject can provide DIP proof that reveal at most 10 DID keys
/// and 10 linked accounts (defaults provided by the
/// `KiltVersionedParachainVerifier` type). Calls that do not pass the
/// [`DipCallFilter`] will be discarded early on in the verification process.
pub type ProofVerifier = KiltVersionedParachainVerifier<
	RelaychainRuntime,
	RelayStateRootsViaRelayStorePallet<Runtime>,
	2_000,
	ProviderRuntime,
	DipCallFilter<KeyIdOf<ProviderRuntime>, BlockNumberFor<ProviderRuntime>, ProviderAccountId>,
>;

impl pallet_dip_consumer::Config for Runtime {
	type DipCallOriginFilter = PreliminaryDipOriginFilter;
	// Any signed origin can submit a cross-chain DIP tx, since subject
	// authentication (and optional binding to the tx submitter) is performed in the
	// DIP proof verification step.
	type DispatchOriginCheck = EnsureSigned<AccountId>;
	type Identifier = DidIdentifier;
	// Local identity info contains a simple `u128` representing a nonce. This means
	// that two cross-chain operations targeting the same chain and with the same
	// nonce cannot be both successfully evaluated.
	type LocalIdentityInfo = u128;
	type ProofVerifier = ProofVerifier;
	type RuntimeCall = RuntimeCall;
	type RuntimeOrigin = RuntimeOrigin;
	type WeightInfo = weights::pallet_dip_consumer::WeightInfo<Runtime>;
}

/// A preliminary DID call filter that only allows dispatching of extrinsics
/// from the [`pallet_postit::Pallet`] pallet.
pub struct PreliminaryDipOriginFilter;

impl Contains<RuntimeCall> for PreliminaryDipOriginFilter {
	#[cfg(not(feature = "runtime-benchmarks"))]
	fn contains(t: &RuntimeCall) -> bool {
		matches!(
			t,
			RuntimeCall::PostIt { .. }
				| RuntimeCall::Utility(pallet_utility::Call::batch { .. })
				| RuntimeCall::Utility(pallet_utility::Call::batch_all { .. })
				| RuntimeCall::Utility(pallet_utility::Call::force_batch { .. })
		)
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn contains(_t: &RuntimeCall) -> bool {
		true
	}
}

/// Calls to the [`pallet_postit::Pallet`] pallet or batches containing only
/// calls to the [`pallet_postit::Pallet`] pallet will go through if authorized
/// by a DID's authentication key. Everything else will fail.
fn derive_verification_key_relationship(call: &RuntimeCall) -> Option<DidVerificationKeyRelationship> {
	match call {
		RuntimeCall::PostIt { .. } => Some(DidVerificationKeyRelationship::Authentication),
		#[cfg(feature = "runtime-benchmarks")]
		RuntimeCall::System(frame_system::Call::remark { .. }) => Some(DidVerificationKeyRelationship::Authentication),
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

/// Errors generated by calls that do not pass the filter.
pub enum DipCallFilterError {
	/// The call cannot be dispatched with the provided origin.
	BadOrigin,
	/// The call could be dispatched with the provided origin, but it has been
	/// authorized with the wrong DID key.
	WrongVerificationRelationship,
}

impl From<DipCallFilterError> for u8 {
	fn from(value: DipCallFilterError) -> Self {
		match value {
			DipCallFilterError::BadOrigin => 1,
			DipCallFilterError::WrongVerificationRelationship => 2,
		}
	}
}

/// A call filter that requires calls to the [`pallet_postit::Pallet`] pallet to
/// be authorized with a DID signature generated with a key of a given
/// verification relationship.
pub struct DipCallFilter<ProviderDidKeyId, ProviderBlockNumber, ProviderAccountId>(
	PhantomData<(ProviderDidKeyId, ProviderBlockNumber, ProviderAccountId)>,
);

impl<ProviderDidKeyId, ProviderBlockNumber, ProviderAccountId> DipCallOriginFilter<RuntimeCall>
	for DipCallFilter<ProviderDidKeyId, ProviderBlockNumber, ProviderAccountId>
{
	type Error = DipCallFilterError;
	type OriginInfo = RevealedDidKey<ProviderDidKeyId, ProviderBlockNumber, ProviderAccountId>;
	type Success = ();

	// Accepts only a DipOrigin for the DidLookup pallet calls.
	fn check_call_origin_info(call: &RuntimeCall, info: &Self::OriginInfo) -> Result<Self::Success, Self::Error> {
		let revealed_key_relationship: DidVerificationKeyRelationship = info
			.relationship
			.try_into()
			.map_err(|_| DipCallFilterError::WrongVerificationRelationship)?;
		let expected_key_relationship =
			single_key_relationship([call].into_iter()).map_err(|_| DipCallFilterError::BadOrigin)?;
		if revealed_key_relationship == expected_key_relationship {
			Ok(())
		} else {
			Err(DipCallFilterError::WrongVerificationRelationship)
		}
	}
}

impl pallet_relay_store::Config for Runtime {
	// The pallet stores the last 100 relaychain state roots, making state proofs
	// valid for at most 100 * 6 = 600 seconds.
	type MaxRelayBlocksStored = ConstU32<100>;
	type WeightInfo = weights::pallet_relay_store::WeightInfo<Runtime>;
}
