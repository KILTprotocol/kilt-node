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
	use did::did_details::{DidCreationDetails, DidDetails, DidVerificationKey};
	pub(crate) use dip_sender_runtime_template::{DidIdentifier, DmpQueue, Runtime, RuntimeOrigin, XcmpQueue};

	use super::*;

	pub const PARA_ID: u32 = 2_000;

	fn generate_did_details(auth_key: DidVerificationKey) -> DidDetails<Runtime> {
		use did::did_details::{DidEncryptionKey, DidVerificationKey};
		use dip_sender_runtime_template::AccountId;
		use kilt_support::Deposit;
		use sp_core::{ecdsa, ed25519, sr25519, DeriveJunction, Pair};
		use sp_std::collections::btree_set::BTreeSet;

		let att_key: DidVerificationKey = sr25519::Pair::from_seed(&[100u8; 32]).public().into();
		let del_key: DidVerificationKey = ecdsa::Pair::from_seed(&[101u8; 32]).public().into();

		let mut base_details = DidDetails::new(auth_key, 0u64.into(), )
		DidDetails::from_creation_details(
			DidCreationDetails {
				did,
				// Not relevant
				submitter: AccountId::new([0u8; 32]),
				new_key_agreement_keys: BTreeSet::from_iter([DidEncryptionKey::X25519([3u8; 32])].into_iter())
					.try_into()
					.unwrap(),
				new_attestation_key: Some(att_key.public().into()),
				new_delegation_key: Some(del_key.public().into()),
				new_service_details: vec![],
			},
			auth_key.public().into(),
		)
		.unwrap()
	}

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
		let (did, details) = generate_did_details();
		ext.execute_with(|| {
			did::pallet::Did::<Runtime>::insert(&did, details);
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
