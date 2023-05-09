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

use super::*;

use did::Did;
use dip_support::latest::MerkleProof;
use frame_support::{assert_ok, weights::Weight};
use frame_system::RawOrigin;
use pallet_did_lookup::linkable_account::LinkableAccountId;
use runtime_common::dip::provider::{CompleteMerkleProof, DidMerkleRootGenerator};
use sp_core::Pair;
use xcm::latest::{
	Junction::Parachain,
	Junctions::{Here, X1},
	ParentThen,
};
use xcm_emulator::TestExt;

use cumulus_pallet_xcmp_queue::Event as XcmpEvent;
use dip_consumer_runtime_template::{
	DidIdentifier, DidLookup, DipConsumer, Runtime as ConsumerRuntime, RuntimeCall as ConsumerRuntimeCall,
	RuntimeEvent, System,
};
use dip_provider_runtime_template::{AccountId as ProviderAccountId, DipProvider, Runtime as ProviderRuntime};

#[test]
fn commit_identity() {
	Network::reset();

	let did: DidIdentifier = para::provider::did_auth_key().public().into();

	// 1. Send identity proof from DIP provider to DIP consumer.
	ProviderParachain::execute_with(|| {
		assert_ok!(DipProvider::commit_identity(
			RawOrigin::Signed(ProviderAccountId::from([0u8; 32])).into(),
			did.clone(),
			Box::new(ParentThen(X1(Parachain(para::consumer::PARA_ID))).into()),
			Box::new((Here, 1_000_000_000).into()),
			Weight::from_ref_time(4_000),
		));
	});
	// 2. Verify that the proof has made it to the DIP consumer.
	ConsumerParachain::execute_with(|| {
		// 2.1 Verify that there was no XCM error.
		assert!(!System::events().iter().any(|r| matches!(
			r.event,
			RuntimeEvent::XcmpQueue(XcmpEvent::Fail {
				error: _,
				message_hash: _,
				weight: _
			})
		)));
		// 2.2 Verify the proof digest was stored correctly.
		assert!(DipConsumer::identity_proofs(&did).is_some());
	});
	// 3. Call an extrinsic on the consumer chain with a valid proof
	let did_details = ProviderParachain::execute_with(|| {
		Did::get(&did).expect("DID details should be stored on the provider chain.")
	});
	// 3.1 Generate a proof
	let CompleteMerkleProof { proof, .. } = DidMerkleRootGenerator::<ProviderRuntime>::generate_proof(
		&did_details,
		[did_details.authentication_key].iter(),
	)
	.expect("Proof generation should not fail");
	// 3.2 Call the `dispatch_as` extrinsic on the consumer chain with the generated
	// proof
	ConsumerParachain::execute_with(|| {
		assert_ok!(DipConsumer::dispatch_as(
			RawOrigin::Signed(para::consumer::DISPATCHER_ACCOUNT).into(),
			did.clone(),
			(
				MerkleProof {
					blinded: proof.blinded,
					revealed: proof.revealed,
				}
				.into(),
				para::provider::did_auth_key().sign(&[0u8]).into()
			),
			Box::new(ConsumerRuntimeCall::DidLookup(pallet_did_lookup::Call::<
				ConsumerRuntime,
			>::associate_sender {})),
		));
		// Verify the account -> DID link exists and contains the right information
		let linked_did = DidLookup::connected_dids::<LinkableAccountId>(para::consumer::DISPATCHER_ACCOUNT.into())
			.map(|link| link.did);
		assert_eq!(linked_did, Some(did));
	});
}
