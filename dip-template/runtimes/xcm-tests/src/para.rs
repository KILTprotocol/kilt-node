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

pub(super) mod provider {
	pub(crate) use dip_provider_runtime_template::{DidIdentifier, DmpQueue, Runtime, RuntimeOrigin, XcmpQueue};

	use did::did_details::{DidDetails, DidEncryptionKey, DidVerificationKey};
	use dip_provider_runtime_template::{AccountId, Balance, BlockNumber, System, Web3Name, UNIT};
	use kilt_support::deposit::Deposit;
	use pallet_did_lookup::{linkable_account::LinkableAccountId, ConnectionRecord};
	use pallet_web3_names::web3_name::Web3NameOwnership;
	use sp_core::{ecdsa, ed25519, sr25519, Pair};
	use sp_runtime::{
		traits::{One, Zero},
		AccountId32, SaturatedConversion,
	};

	use super::*;

	pub const PARA_ID: u32 = 2_000;
	pub const DISPATCHER_ACCOUNT: AccountId = AccountId::new([190u8; 32]);
	const INITIAL_BALANCE: Balance = 100_000 * UNIT;

	pub(crate) fn did_auth_key() -> ed25519::Pair {
		ed25519::Pair::from_seed(&[200u8; 32])
	}

	fn generate_did_details() -> DidDetails<Runtime> {
		let auth_key: DidVerificationKey = did_auth_key().public().into();
		let att_key: DidVerificationKey = sr25519::Pair::from_seed(&[100u8; 32]).public().into();
		let del_key: DidVerificationKey = ecdsa::Pair::from_seed(&[101u8; 32]).public().into();

		let mut details = DidDetails::new(
			auth_key,
			0u32,
			Deposit {
				amount: 1u64.into(),
				owner: AccountId::new([1u8; 32]),
			},
		)
		.unwrap();
		details.update_attestation_key(att_key, 0u32).unwrap();
		details.update_delegation_key(del_key, 0u32).unwrap();
		let max_key_agreement_key_count: u8 =
			<Runtime as did::Config>::MaxTotalKeyAgreementKeys::get().saturated_into();
		(1u8..max_key_agreement_key_count).for_each(|s| {
			details
				.add_key_agreement_key(DidEncryptionKey::X25519([s; 32]), 0u32)
				.unwrap();
		});
		details
	}

	pub(crate) fn para_ext() -> TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();

		let parachain_info_config = parachain_info::GenesisConfig {
			parachain_id: PARA_ID.into(),
		};

		<parachain_info::GenesisConfig as GenesisBuild<Runtime, _>>::assimilate_storage(&parachain_info_config, &mut t)
			.unwrap();

		pallet_balances::GenesisConfig::<Runtime> {
			balances: vec![(DISPATCHER_ACCOUNT, INITIAL_BALANCE)],
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut ext = TestExternalities::new(t);
		let did: DidIdentifier = did_auth_key().public().into();
		let details = generate_did_details();
		let acc: AccountId32 = did_auth_key().public().into();
		let web3_name: Web3Name = b"test".to_vec().try_into().unwrap();
		ext.execute_with(|| {
			did::pallet::Did::<Runtime>::insert(&did, details);
			pallet_did_lookup::pallet::ConnectedDids::<Runtime>::insert(
				LinkableAccountId::from(acc.clone()),
				ConnectionRecord {
					did: did.clone(),
					deposit: Deposit {
						amount: Balance::one(),
						owner: acc.clone(),
					},
				},
			);
			pallet_did_lookup::pallet::ConnectedAccounts::<Runtime>::insert(
				&did,
				LinkableAccountId::from(acc.clone()),
				(),
			);
			pallet_web3_names::pallet::Owner::<Runtime>::insert(
				&web3_name,
				Web3NameOwnership {
					claimed_at: BlockNumber::zero(),
					owner: did.clone(),
					deposit: Deposit {
						amount: Balance::one(),
						owner: acc.clone(),
					},
				},
			);
			pallet_web3_names::pallet::Names::<Runtime>::insert(did, web3_name);
			System::set_block_number(1);
		});
		ext
	}

	decl_test_parachain! {
		pub struct ProviderParachain {
			Runtime = Runtime,
			RuntimeOrigin = RuntimeOrigin,
			XcmpMessageHandler = XcmpQueue,
			DmpMessageHandler = DmpQueue,
			new_ext = para_ext(),
		}
	}
}

pub(super) mod consumer {
	pub(crate) use dip_consumer_runtime_template::{
		AccountId, AssetTransactorLocationConverter, Balance, DmpQueue, Runtime, RuntimeOrigin, XcmpQueue, UNIT,
	};

	use dip_consumer_runtime_template::System;
	use xcm::v3::{
		Junction::{AccountId32, Parachain},
		Junctions::X2,
		ParentThen,
	};
	use xcm_executor::traits::Convert;

	use super::*;

	pub const PARA_ID: u32 = 2_001;
	pub const DISPATCHER_ACCOUNT: AccountId = AccountId::new([90u8; 32]);
	const INITIAL_BALANCE: Balance = 100_000 * UNIT;

	pub(crate) fn provider_dispatcher_account_on_consumer() -> AccountId {
		AssetTransactorLocationConverter::convert(
			ParentThen(X2(
				Parachain(provider::PARA_ID),
				AccountId32 {
					network: None,
					id: provider::DISPATCHER_ACCOUNT.into(),
				},
			))
			.into(),
		)
		.expect("Conversion of account from provider parachain to consumer parachain should not fail.")
	}

	pub(crate) fn para_ext() -> TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();

		let parachain_info_config = parachain_info::GenesisConfig {
			parachain_id: PARA_ID.into(),
		};

		<parachain_info::GenesisConfig as GenesisBuild<Runtime, _>>::assimilate_storage(&parachain_info_config, &mut t)
			.unwrap();

		pallet_balances::GenesisConfig::<Runtime> {
			balances: vec![
				(provider_dispatcher_account_on_consumer(), INITIAL_BALANCE),
				(DISPATCHER_ACCOUNT, INITIAL_BALANCE),
			],
		}
		.assimilate_storage(&mut t)
		.unwrap();

		let mut ext = TestExternalities::new(t);
		ext.execute_with(|| {
			System::set_block_number(1);
		});
		ext
	}

	#[cfg(test)]
	pub(crate) use test_utils::*;

	#[cfg(test)]
	mod test_utils {
		use super::*;

		use polkadot_parachain::primitives::Sibling;
		use xcm::v3::Junctions::X1;
		use xcm_builder::SiblingParachainConvertsVia;

		pub(crate) fn provider_parachain_account_on_consumer() -> AccountId {
			SiblingParachainConvertsVia::<Sibling, AccountId>::convert(
				ParentThen(X1(Parachain(provider::PARA_ID))).into(),
			)
			.expect("Conversion of account from provider parachain to consumer parachain should not fail.")
		}
	}
}
