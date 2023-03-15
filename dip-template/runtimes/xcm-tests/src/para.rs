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

use crate::{_Messenger, _hrmp_channel_parachain_inherent_data, _process_messages};
use frame_support::traits::GenesisBuild;
use sp_io::TestExternalities;
use xcm_emulator::decl_test_parachain;

pub(super) mod sender {
	pub(crate) use dip_sender_runtime_template::{DmpQueue, Runtime, RuntimeOrigin, XcmpQueue};

	use super::*;

	pub const PARA_ID: u32 = 2_000;

	pub(crate) fn para_ext() -> TestExternalities {
		use dip_sender_runtime_template::System;

		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();

		let parachain_info_config = parachain_info::GenesisConfig {
			parachain_id: PARA_ID.into(),
		};

		<parachain_info::GenesisConfig as GenesisBuild<Runtime, _>>::assimilate_storage(&parachain_info_config, &mut t)
			.unwrap();

		let mut ext = TestExternalities::new(t);
		ext.execute_with(|| {
			System::set_block_number(1);
		});
		ext
	}

	decl_test_parachain! {
		pub struct SenderParachain {
			Runtime = Runtime,
			RuntimeOrigin = RuntimeOrigin,
			XcmpMessageHandler = XcmpQueue,
			DmpMessageHandler = DmpQueue,
			new_ext = para_ext(),
		}
	}
}

pub(super) mod receiver {
	pub(crate) use dip_receiver_runtime_template::{
		AccountId, AssetTransactorLocationConverter, Balance, DmpQueue, Runtime, RuntimeOrigin, XcmpQueue,
	};

	use xcm::latest::{Junction::Parachain, Junctions::X1, ParentThen};
	use xcm_executor::traits::Convert;

	use super::*;

	pub const PARA_ID: u32 = 2_001;
	const INITIAL_BALANCE: Balance = 1_000_000_000;

	pub(crate) fn sender_parachain_account() -> AccountId {
		AssetTransactorLocationConverter::convert(ParentThen(X1(Parachain(sender::PARA_ID))).into())
			.expect("Conversion of account from sender parachain to receiver parachain should not fail.")
	}

	pub(crate) fn para_ext() -> TestExternalities {
		use dip_receiver_runtime_template::System;

		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();

		let parachain_info_config = parachain_info::GenesisConfig {
			parachain_id: PARA_ID.into(),
		};

		<parachain_info::GenesisConfig as GenesisBuild<Runtime, _>>::assimilate_storage(&parachain_info_config, &mut t)
			.unwrap();

		pallet_balances::GenesisConfig::<Runtime> {
			balances: vec![(sender_parachain_account(), INITIAL_BALANCE)],
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut ext = TestExternalities::new(t);
		ext.execute_with(|| {
			System::set_block_number(1);
		});
		ext
	}
}
