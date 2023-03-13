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

use frame_support::{sp_tracing, traits::GenesisBuild};
use polkadot_primitives::runtime_api::runtime_decl_for_ParachainHost::ParachainHostV3;
use sp_io::TestExternalities;
use xcm::latest::{prelude::*, Weight};
use xcm_emulator::{decl_test_network, decl_test_parachain, decl_test_relay_chain};
use xcm_executor::traits::Convert;

const INITIAL_BALANCE: dip_receiver_runtime_template::Balance = 1_000_000_000;

const SENDER_PARA_ID: u32 = 2000;
const RECEIVER_PARA_ID: u32 = 2001;

fn default_parachains_host_configuration(
) -> polkadot_runtime_parachains::configuration::HostConfiguration<polkadot_primitives::v2::BlockNumber> {
	use polkadot_primitives::v2::{MAX_CODE_SIZE, MAX_POV_SIZE};

	polkadot_runtime_parachains::configuration::HostConfiguration {
		minimum_validation_upgrade_delay: 5,
		validation_upgrade_cooldown: 10u32,
		validation_upgrade_delay: 10,
		code_retention_period: 1200,
		max_code_size: MAX_CODE_SIZE,
		max_pov_size: MAX_POV_SIZE,
		max_head_data_size: 32 * 1024,
		group_rotation_frequency: 20,
		chain_availability_period: 4,
		thread_availability_period: 4,
		max_upward_queue_count: 8,
		max_upward_queue_size: 1024 * 1024,
		max_downward_message_size: 1024,
		ump_service_total_weight: Weight::from_ref_time(4 * 1_000_000_000),
		max_upward_message_size: 50 * 1024,
		max_upward_message_num_per_candidate: 5,
		hrmp_sender_deposit: 0,
		hrmp_recipient_deposit: 0,
		hrmp_channel_max_capacity: 8,
		hrmp_channel_max_total_size: 8 * 1024,
		hrmp_max_parachain_inbound_channels: 4,
		hrmp_max_parathread_inbound_channels: 4,
		hrmp_channel_max_message_size: 1024 * 1024,
		hrmp_max_parachain_outbound_channels: 4,
		hrmp_max_parathread_outbound_channels: 4,
		hrmp_max_message_num_per_candidate: 5,
		dispute_period: 6,
		no_show_slots: 2,
		n_delay_tranches: 25,
		needed_approvals: 2,
		relay_vrf_modulo_samples: 2,
		zeroth_delay_tranche_width: 0,
		..Default::default()
	}
}

fn rococo_ext() -> TestExternalities {
	use rococo_runtime::{Runtime, System};

	let mut t = frame_system::GenesisConfig::default()
		.build_storage::<Runtime>()
		.unwrap();

	polkadot_runtime_parachains::configuration::GenesisConfig::<Runtime> {
		config: default_parachains_host_configuration(),
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext
}

fn sender_parachain_account_on_receiver_chain() -> dip_receiver_runtime_template::AccountId {
	dip_receiver_runtime_template::xcm_config::LocationToAccountId::convert(
		ParentThen(X1(Parachain(SENDER_PARA_ID))).into(),
	)
	.expect("Conversion of account from sender parachain to receiver parachain should not fail.")
}

fn sender_para_ext() -> TestExternalities {
	use dip_sender_runtime_template::{Runtime, System};

	let mut t = frame_system::GenesisConfig::default()
		.build_storage::<Runtime>()
		.unwrap();

	let parachain_info_config = parachain_info::GenesisConfig {
		parachain_id: SENDER_PARA_ID.into(),
	};

	<parachain_info::GenesisConfig as GenesisBuild<Runtime, _>>::assimilate_storage(&parachain_info_config, &mut t)
		.unwrap();

	let mut ext = TestExternalities::new(t);
	ext.execute_with(|| {
		System::set_block_number(1);
	});
	ext
}

fn receiver_para_ext() -> TestExternalities {
	use dip_receiver_runtime_template::{Runtime, System};

	let mut t = frame_system::GenesisConfig::default()
		.build_storage::<Runtime>()
		.unwrap();

	let parachain_info_config = parachain_info::GenesisConfig {
		parachain_id: SENDER_PARA_ID.into(),
	};

	<parachain_info::GenesisConfig as GenesisBuild<Runtime, _>>::assimilate_storage(&parachain_info_config, &mut t)
		.unwrap();

	pallet_balances::GenesisConfig::<Runtime> {
		balances: vec![(sender_parachain_account_on_receiver_chain(), INITIAL_BALANCE)],
	}
	.assimilate_storage(&mut t)
	.unwrap();

	let mut ext = TestExternalities::new(t);
	ext.execute_with(|| {
		System::set_block_number(1);
	});
	ext
}

decl_test_relay_chain! {
	pub struct RococoChain {
		Runtime = rococo_runtime::Runtime,
		XcmConfig = rococo_runtime::xcm_config::XcmConfig,
		new_ext = rococo_ext(),
	}
}

decl_test_parachain! {
	pub struct SenderParachain {
		Runtime = dip_sender_runtime_template::Runtime,
		RuntimeOrigin = dip_sender_runtime_template::RuntimeOrigin,
		XcmpMessageHandler = dip_sender_runtime_template::XcmpQueue,
		DmpMessageHandler = dip_sender_runtime_template::DmpQueue,
		new_ext = sender_para_ext(),
	}
}

decl_test_parachain! {
	pub struct ReceiverParachain {
		Runtime = dip_receiver_runtime_template::Runtime,
		RuntimeOrigin = dip_receiver_runtime_template::RuntimeOrigin,
		XcmpMessageHandler = dip_receiver_runtime_template::XcmpQueue,
		DmpMessageHandler = dip_receiver_runtime_template::DmpQueue,
		new_ext = receiver_para_ext(),
	}
}

decl_test_network! {
	pub struct Network {
		relay_chain = RococoChain,
		parachains = vec![
			(2_000, SenderParachain),
			(2_001, ReceiverParachain),
		],
	}
}

#[cfg(test)]
mod test {
	use super::*;

	use dip_receiver_runtime_template::{DipReceiver, Runtime as ReceiverRuntime};
	use dip_sender_runtime_template::{DipSender, Runtime as SenderRuntime};
	use rococo_runtime::System as RelaySystem;

	use codec::Encode;
	use frame_support::assert_ok;
	use frame_system::RawOrigin;
	use xcm_emulator::TestExt;

	#[test]
	fn dmp() {
		Network::reset();

		SenderParachain::execute_with(|| {
			assert_ok!(DipSender::commit_identity(
				RawOrigin::Signed(dip_sender_runtime_template::AccountId::from([0u8; 32])).into(),
				dip_sender_runtime_template::AccountId::from([0u8; 32]),
				Box::new(ParentThen(X1(Parachain(RECEIVER_PARA_ID))).into()),
				Box::new((Here, 1_000_000_000).into()),
				Weight::from_ref_time(4_000_000),
			));
		});
		ReceiverParachain::execute_with(|| {
			let details = dip_receiver::IdentityProofs::<ReceiverRuntime>::iter();
			println!("Details: {:?}", details.collect::<Vec<_>>());
		})
	}
}
