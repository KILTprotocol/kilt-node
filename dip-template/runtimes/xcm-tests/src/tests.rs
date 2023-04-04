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

use dip_receiver_runtime_template::{
	AccountId as ReceiverAccountId, DidIdentifier as ReceiverDidIdentifier, DipReceiver,
};
use dip_sender_runtime_template::DipSender;

use frame_support::{assert_ok, weights::Weight};
use frame_system::RawOrigin;
use xcm::latest::{
	Junction::Parachain,
	Junctions::{Here, X1},
	ParentThen,
};
use xcm_emulator::TestExt;

#[test]
fn commit_identity() {
	Network::reset();

	ReceiverParachain::execute_with(|| {
		use dip_receiver_runtime_template::Balances;
		use para::receiver::sender_parachain_account;

		let sender_balance = Balances::free_balance(sender_parachain_account());
		println!("Sender balance: {:?}", sender_balance);
	});

	// 1. Send identity proof from DIP sender to DIP receiver.
	SenderParachain::execute_with(|| {
		assert_ok!(DipSender::commit_identity(
			RawOrigin::Signed(ReceiverAccountId::from([0u8; 32])).into(),
			ReceiverDidIdentifier::from([0u8; 32]),
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
		// 2.2 Verify the proof digest is the same that was sent.
		let details = DipReceiver::identity_proofs(dip_sender_runtime_template::AccountId::from([0u8; 32]));
		assert_eq!(details, Some([0u8; 32]));
	});
}
