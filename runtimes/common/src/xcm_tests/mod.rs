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

use frame_support::sp_tracing;
use sp_io::TestExternalities;
use xcm::latest::prelude::*;
use xcm_executor::traits::Convert;
use xcm_simulator::{decl_test_network, decl_test_parachain, decl_test_relay_chain};

pub mod parachain;
pub mod relaychain;

const ALICE: parachain::AccountId = parachain::AccountId::new([0u8; 32]);
const INITIAL_BALANCE: parachain::Balance = 1_000_000_000;

const SENDER_PARA_ID: u32 = 2000;
const RECEIVER_PARA_ID: u32 = 2001;

fn relay_ext() -> TestExternalities {
	use relaychain::Runtime;

	let t = frame_system::GenesisConfig::default()
		.build_storage::<Runtime>()
		.unwrap();

	TestExternalities::new(t)
}

fn sender_account_on_receiver_chain() -> parachain::AccountId {
	parachain::LocationToAccountId::convert(ParentThen(X1(Parachain(SENDER_PARA_ID))).into())
		.expect("Conversion of account from sender parachain to receiver parachain should not fail.")
}

fn sender_para_ext() -> TestExternalities {
	use parachain::sender::{MsgQueue, Runtime, System};

	let mut t = frame_system::GenesisConfig::default()
		.build_storage::<Runtime>()
		.unwrap();

	pallet_balances::GenesisConfig::<Runtime> {
		balances: vec![(ALICE, INITIAL_BALANCE)],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = TestExternalities::new(t);
	ext.execute_with(|| {
		sp_tracing::try_init_simple();
		System::set_block_number(1);
		MsgQueue::set_para_id(SENDER_PARA_ID.into());
	});
	ext
}

fn receiver_para_ext() -> TestExternalities {
	use parachain::receiver::{MsgQueue, Runtime, System};

	let mut t = frame_system::GenesisConfig::default()
		.build_storage::<Runtime>()
		.unwrap();

	pallet_balances::GenesisConfig::<Runtime> {
		balances: vec![(sender_account_on_receiver_chain(), INITIAL_BALANCE)],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = TestExternalities::new(t);
	ext.execute_with(|| {
		sp_tracing::try_init_simple();
		System::set_block_number(1);
		MsgQueue::set_para_id(RECEIVER_PARA_ID.into());
	});
	ext
}

decl_test_relay_chain! {
	pub struct RelayChain {
		Runtime = relaychain::Runtime,
		XcmConfig = relaychain::XcmConfig,
		new_ext = relay_ext(),
	}
}

decl_test_parachain! {
	pub struct SenderParachain {
		Runtime = parachain::sender::Runtime,
		XcmpMessageHandler = parachain::sender::MsgQueue,
		DmpMessageHandler = parachain::sender::MsgQueue,
		new_ext = sender_para_ext(),
	}
}

decl_test_parachain! {
	pub struct ReceiverParachain {
		Runtime = parachain::receiver::Runtime,
		XcmpMessageHandler = parachain::receiver::MsgQueue,
		DmpMessageHandler = parachain::receiver::MsgQueue,
		new_ext = receiver_para_ext(),
	}
}

decl_test_network! {
	pub struct MockNet {
		relay_chain = RelayChain,
		parachains = vec![
			(SENDER_PARA_ID, SenderParachain),
			(RECEIVER_PARA_ID, ReceiverParachain),
		],
	}
}

// This whole module should already be feature-gated, but we feature-gate the
// tests for future-proofness.
#[cfg(test)]
mod tests {
	use super::*;

	use frame_support::assert_ok;
	use frame_system::RawOrigin;
	use xcm_simulator::TestExt;

	use dip_support::latest::Proof;
	use parachain::{receiver::Runtime as ReceiverRuntime, sender::Runtime as SenderRuntime};

	const ALICE_DID_IDENTIFIER: parachain::Identifier = *b"id/alice";

	#[test]
	fn first_test() {
		SenderParachain::execute_with(|| {
			// 1. Send Alice identity commitment over to receiver parachain
			assert_ok!(dip_sender::Pallet::<SenderRuntime>::commit_identity(
				RawOrigin::Signed(ALICE).into(),
				ALICE_DID_IDENTIFIER,
				Box::new((Parent, Parachain(RECEIVER_PARA_ID)).into()),
			));
		});

		ReceiverParachain::execute_with(|| {
			// 2. Verify Alice's identity exists on parachain 2, and that her
			// balance has been decreased accordingly on parachain 2.
			assert_eq!(
				dip_receiver::Pallet::<ReceiverRuntime>::identity_proofs(ALICE_DID_IDENTIFIER),
				// Sender parachain uses the `DefaultIdentityProofGenerator` which returns the default for the type of
				// the proof value.
				Some(<SenderRuntime as dip_sender::Config>::ProofOutput::default())
			);
			// 3. Verify that Alice can use her DID on parachain B by calling
			// the extrinsic of the test pallet.
			assert_ok!(dip_receiver::Pallet::<ReceiverRuntime>::dispatch_as(
				RawOrigin::Signed(ALICE).into(),
				ALICE_DID_IDENTIFIER,
				// Test runtime always returns true for proofs.
				Proof::default().into(),
				Box::new(parachain::mock_dip_enabled_pallet::Call::<ReceiverRuntime>::test_origin {}.into())
			));
		});
	}
}
