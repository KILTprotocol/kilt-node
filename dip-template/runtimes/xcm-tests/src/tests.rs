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
use dip_support::latest::Proof;
use frame_support::{assert_ok, weights::Weight};
use frame_system::RawOrigin;
use runtime_common::dip::sender::{CompleteMerkleProof, DidMerkleRootGenerator};
use sp_core::Pair;
use xcm::latest::{
	Junction::Parachain,
	Junctions::{Here, X1},
	ParentThen,
};
use xcm_emulator::TestExt;

use dip_receiver_runtime_template::{
	AccountId as ReceiverAccountId, DidIdentifier, DipReceiver, Runtime as ReceiverRuntime,
	RuntimeCall as ReceiverRuntimeCall,
};
use dip_sender_runtime_template::{AccountId as SenderAccountId, DipSender, Runtime as SenderRuntime};

#[test]
fn commit_identity() {
	Network::reset();

	let did: DidIdentifier = para::sender::did_auth_key().public().into();

	ReceiverParachain::execute_with(|| {
		use dip_receiver_runtime_template::Balances;
		use para::receiver::sender_parachain_account;

		let sender_balance = Balances::free_balance(sender_parachain_account());
		println!("Sender balance: {:?}", sender_balance);
	});

	// 1. Send identity proof from DIP sender to DIP receiver.
	SenderParachain::execute_with(|| {
		assert_ok!(DipSender::commit_identity(
			RawOrigin::Signed(SenderAccountId::from([0u8; 32])).into(),
			did.clone(),
			Box::new(ParentThen(X1(Parachain(para::receiver::PARA_ID))).into()),
			Box::new((Here, 1_000_000_000).into()),
			Weight::from_ref_time(4_000),
		));
	});
	// 2. Verify that the proof has made it to the DIP receiver.
	ReceiverParachain::execute_with(|| {
		use cumulus_pallet_xcmp_queue::Event as XcmpEvent;
		use dip_receiver_runtime_template::{RuntimeEvent, System};

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
		assert!(DipReceiver::identity_proofs(&did).is_some());
	});
	// 3. Call an extrinsic on the receiver chain with a valid proof
	let did_details =
		SenderParachain::execute_with(|| Did::get(&did).expect("DID details should be stored on the sender chain."));
	// 3.1 Generate a proof
	let CompleteMerkleProof { proof, .. } =
		DidMerkleRootGenerator::<SenderRuntime>::generate_proof(&did_details, [did_details.authentication_key].iter())
			.expect("Proof generation should not fail");
	// 3.2 Call the `dispatch_as` extrinsic on the receiver chain with the generated
	// proof
	ReceiverParachain::execute_with(|| {
		assert_ok!(DipReceiver::dispatch_as(
			RawOrigin::Signed(ReceiverAccountId::new([100u8; 32])).into(),
			did.clone(),
			Proof {
				blinded: proof.blinded,
				revealed: proof.revealed,
			}
			.into(),
			Box::new(ReceiverRuntimeCall::DidLookup(pallet_did_lookup::Call::<
				ReceiverRuntime,
			>::associate_sender {})),
		));
	});
}
