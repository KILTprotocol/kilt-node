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
use xcm_simulator::{decl_test_network, decl_test_parachain, decl_test_relay_chain};

pub mod parachain;
pub mod relaychain;

const ALICE: parachain::AccountId = parachain::AccountId::new([0u8; 32]);
const INITIAL_BALANCE: parachain::Balance = 1_000_000_000;

const SENDER_PARA_ID: u32 = 2000;
const RECEIVER_PARA_ID: u32 = 2001;

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

	let t = frame_system::GenesisConfig::default()
		.build_storage::<Runtime>()
		.unwrap();

	let mut ext = TestExternalities::new(t);
	ext.execute_with(|| {
		sp_tracing::try_init_simple();
		System::set_block_number(1);
		MsgQueue::set_para_id(RECEIVER_PARA_ID.into());
	});
	ext
}

fn relay_ext() -> TestExternalities {
	use relaychain::Runtime;

	let t = frame_system::GenesisConfig::default()
		.build_storage::<Runtime>()
		.unwrap();

	TestExternalities::new(t)
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

decl_test_relay_chain! {
	pub struct RelayChain {
		Runtime = relaychain::Runtime,
		XcmConfig = relaychain::XcmConfig,
		new_ext = relay_ext(),
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
