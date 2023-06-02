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

use crate::para::consumer::{provider_dispatcher_account_on_consumer, provider_parachain_account_on_consumer};

use super::*;

use did::{Did, DidRawOrigin, DidSignature};
use frame_support::{assert_ok, weights::Weight};
use frame_system::RawOrigin;
use kilt_dip_support::{
	did::{MerkleLeavesAndDidSignature, TimeBoundDidSignature},
	merkle::MerkleProof,
};
use pallet_did_lookup::{linkable_account::LinkableAccountId, ConnectedAccounts};
use pallet_web3_names::{Names, Owner};
use parity_scale_codec::Encode;
use runtime_common::dip::{
	did::Web3OwnershipOf,
	merkle::{CompleteMerkleProof, DidMerkleRootGenerator},
};
use sp_core::Pair;
use sp_runtime::traits::Zero;
use xcm::v3::{
	Junction::Parachain,
	Junctions::{Here, X1},
	ParentThen,
};
use xcm_emulator::TestExt;

use cumulus_pallet_xcmp_queue::Event as XcmpEvent;
use dip_consumer_runtime_template::{
	Balances, BlockNumber, DidIdentifier, DidLookup, DipConsumer, Runtime as ConsumerRuntime,
	RuntimeCall as ConsumerRuntimeCall, RuntimeEvent, System,
};
use dip_provider_runtime_template::{DipProvider, Runtime as ProviderRuntime};

#[test]
fn commit_identity() {
	Network::reset();

	let did: DidIdentifier = para::provider::did_auth_key().public().into();
	let provider_parachain_on_consumer_parachain_balance_before =
		ConsumerParachain::execute_with(|| Balances::free_balance(provider_parachain_account_on_consumer()));
	let dispatcher_on_consumer_parachain_balance_before =
		ConsumerParachain::execute_with(|| Balances::free_balance(provider_dispatcher_account_on_consumer()));

	// 1. Send identity commitment from DIP provider to DIP consumer.
	ProviderParachain::execute_with(|| {
		assert_ok!(DipProvider::commit_identity(
			DidRawOrigin {
				id: did.clone(),
				submitter: para::provider::DISPATCHER_ACCOUNT
			}
			.into(),
			did.clone(),
			Box::new(ParentThen(X1(Parachain(para::consumer::PARA_ID))).into()),
			Box::new((Here, 1_000_000_000).into()),
			Weight::from_ref_time(4_000),
		));
	});
	// 2. Verify that the commitment has made it to the DIP consumer.
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
		// 2.3 Verify that the provider parachain sovereign account balance has not
		// changed.
		let provider_parachain_on_consumer_parachain_balance_after =
			Balances::free_balance(provider_parachain_account_on_consumer());
		assert_eq!(
			provider_parachain_on_consumer_parachain_balance_before,
			provider_parachain_on_consumer_parachain_balance_after
		);
		// 2.4 Verify that the dispatcher's account balance on the consumer parachain
		// has decreased.
		let dispatcher_on_consumer_parachain_balance_after =
			Balances::free_balance(provider_dispatcher_account_on_consumer());
		assert!(dispatcher_on_consumer_parachain_balance_after < dispatcher_on_consumer_parachain_balance_before);
	});
	// 3. Call an extrinsic on the consumer chain with a valid proof and signature
	let did_details = ProviderParachain::execute_with(|| {
		Did::get(&did).expect("DID details should be stored on the provider chain.")
	});
	println!(
		"Complete DID details encoded size: {:?} bytes",
		did_details.encoded_size()
	);
	let (web3_name, ownership_details) = ProviderParachain::execute_with(|| {
		let web3_name =
			Names::<ProviderRuntime>::get(&did).expect("Web3name should be linked to the DID on the provider chain.");
		let ownership_details = Owner::<ProviderRuntime>::get(&web3_name)
			.expect("Web3name details should be present for the retrieved web3name.");
		(web3_name, ownership_details)
	});
	println!(
		"Web3name and ownership size: ({:?}, {:?}) bytes",
		web3_name.encoded_size(),
		ownership_details.encoded_size(),
	);
	let linked_accounts = ProviderParachain::execute_with(|| {
		ConnectedAccounts::<ProviderRuntime>::iter_key_prefix(&did).collect::<Vec<_>>()
	});
	println!("Linked accounts size: {:?} bytes", linked_accounts.encoded_size());
	let call = ConsumerRuntimeCall::DidLookup(pallet_did_lookup::Call::<ConsumerRuntime>::associate_sender {});
	// 3.1 Generate a proof
	let CompleteMerkleProof { proof, .. } = DidMerkleRootGenerator::<ProviderRuntime>::generate_proof(
		&(
			Some(did_details.clone()),
			Some(Web3OwnershipOf::<ProviderRuntime> {
				web3_name,
				claimed_at: ownership_details.claimed_at,
			}),
			Some(linked_accounts.clone()),
		)
			.into(),
		[
			did_details.authentication_key,
			did_details.attestation_key.unwrap(),
			did_details.delegation_key.unwrap(),
		]
		.iter(),
		true,
		linked_accounts.iter(),
	)
	.expect("Proof generation should not fail");
	println!(
		"Complete merkle proof size: {:?} bytes. Blinded part: {:?} bytes. Revealed part: {:?} bytes.",
		proof.encoded_size(),
		proof.blinded.encoded_size(),
		proof.revealed.encoded_size()
	);
	// 3.2 Generate a DID signature
	let genesis_hash =
		ConsumerParachain::execute_with(|| frame_system::Pallet::<ConsumerRuntime>::block_hash(BlockNumber::zero()));
	let system_block = ConsumerParachain::execute_with(frame_system::Pallet::<ConsumerRuntime>::block_number);
	let payload = (
		call.clone(),
		0u128,
		para::consumer::DISPATCHER_ACCOUNT,
		system_block,
		genesis_hash,
	);
	let signature: DidSignature = para::provider::did_auth_key().sign(&payload.encode()).into();
	// 3.3 Call the `dispatch_as` extrinsic on the consumer chain with the generated
	// proof
	ConsumerParachain::execute_with(|| {
		assert_ok!(DipConsumer::dispatch_as(
			RawOrigin::Signed(para::consumer::DISPATCHER_ACCOUNT).into(),
			did.clone(),
			MerkleLeavesAndDidSignature {
				merkle_leaves: MerkleProof {
					blinded: proof.blinded,
					revealed: proof.revealed,
				},
				did_signature: TimeBoundDidSignature {
					signature,
					block_number: system_block
				}
			},
			Box::new(call),
		));
		// Verify the account -> DID link exists and contains the right information
		let linked_did = DidLookup::connected_dids::<LinkableAccountId>(para::consumer::DISPATCHER_ACCOUNT.into())
			.map(|link| link.did);
		assert_eq!(linked_did, Some(did.clone()));
		// Verify that the details of the DID subject have been bumped
		let details = DipConsumer::identity_proofs(&did).map(|entry| entry.details);
		assert_eq!(details, Some(1u128));
	});
}
