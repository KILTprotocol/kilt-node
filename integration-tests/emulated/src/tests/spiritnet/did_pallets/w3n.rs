// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2024 BOTLabs GmbH

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

use asset_hub_rococo_emulated_chain::AssetHubRococoParaPallet;
use frame_support::{assert_ok, traits::fungible::Mutate};
use parity_scale_codec::Encode;
use runtime_common::{constants::KILT, AccountId, Balance};
use sp_runtime::BoundedVec;
use xcm::{lts::prelude::OriginKind, DoubleEncoded, VersionedXcm};
use xcm_emulator::{assert_expected_events, Chain, Network, TestExt};

use crate::{
	mock::network::{AssetHub, MockNetwork, Rococo, Spiritnet},
	tests::spiritnet::did_pallets::utils::{
		construct_basic_transact_xcm_message, create_mock_did_from_account, get_asset_hub_sovereign_account,
		get_sibling_destination_spiritnet,
	},
};

fn get_xcm_message_claim_w3n(origin_kind: OriginKind, withdraw_balance: Balance) -> VersionedXcm<()> {
	let asset_hub_sovereign_account = get_asset_hub_sovereign_account();

	let call: DoubleEncoded<()> = <Spiritnet as Chain>::RuntimeCall::Did(did::Call::dispatch_as {
		did_identifier: asset_hub_sovereign_account,
		call: Box::new(<Spiritnet as Chain>::RuntimeCall::Web3Names(
			pallet_web3_names::Call::claim {
				name: BoundedVec::try_from(b"adelo".to_vec()).unwrap(),
			},
		)),
	})
	.encode()
	.into();

	construct_basic_transact_xcm_message(origin_kind, withdraw_balance, call)
}

#[test]
fn test_claim_w3n_from_asset_hub_successful() {
	MockNetwork::reset();

	let sudo_origin = <AssetHub as Chain>::RuntimeOrigin::root();
	let asset_hub_sovereign_account = get_asset_hub_sovereign_account();

	let init_balance = KILT * 10;

	let xcm_claim_w3n_msg = get_xcm_message_claim_w3n(OriginKind::SovereignAccount, KILT);
	let destination = get_sibling_destination_spiritnet();

	Spiritnet::execute_with(|| {
		create_mock_did_from_account(asset_hub_sovereign_account.clone());
		<spiritnet_runtime::Balances as Mutate<AccountId>>::set_balance(&asset_hub_sovereign_account, init_balance);
	});

	AssetHub::execute_with(|| {
		assert_ok!(<AssetHub as AssetHubRococoParaPallet>::PolkadotXcm::send(
			sudo_origin,
			Box::new(destination),
			Box::new(xcm_claim_w3n_msg)
		));

		type RuntimeEvent = <AssetHub as Chain>::RuntimeEvent;
		assert_expected_events!(
			AssetHub,
			vec![
				RuntimeEvent::PolkadotXcm(pallet_xcm::Event::Sent { .. }) => {},
			]
		);
	});

	Spiritnet::execute_with(|| {
		type SpiritnetRuntimeEvent = <Spiritnet as Chain>::RuntimeEvent;

		assert_expected_events!(
			Spiritnet,
			vec![
				SpiritnetRuntimeEvent::MessageQueue(pallet_message_queue::Event::Processed { success: true, .. }) => {},
				SpiritnetRuntimeEvent::Did(did::Event::DidCallDispatched(account, result)) => {
					account: account == &asset_hub_sovereign_account,
					result: result.is_ok(),
				},
				SpiritnetRuntimeEvent::Web3Names(pallet_web3_names::Event::Web3NameClaimed{owner, name: _}) => {
					owner: owner == &asset_hub_sovereign_account,
				},
			]
		);
	});

	Rococo::execute_with(|| {
		assert_eq!(Rococo::events().len(), 0);
	});
}

#[test]
fn test_claim_w3n_from_asset_hub_unsuccessful() {
	let origin_kind_list = vec![OriginKind::Native, OriginKind::Superuser, OriginKind::Xcm];

	let sudo_origin = <AssetHub as Chain>::RuntimeOrigin::root();
	let init_balance = KILT * 100;

	let destination = get_sibling_destination_spiritnet();
	let asset_hub_sovereign_account = get_asset_hub_sovereign_account();

	for origin_kind in origin_kind_list {
		MockNetwork::reset();

		Spiritnet::execute_with(|| {
			create_mock_did_from_account(asset_hub_sovereign_account.clone());
			<spiritnet_runtime::Balances as Mutate<AccountId>>::set_balance(&asset_hub_sovereign_account, init_balance);
		});

		let xcm_claim_w3n_msg = get_xcm_message_claim_w3n(origin_kind, KILT);

		AssetHub::execute_with(|| {
			assert_ok!(<AssetHub as AssetHubRococoParaPallet>::PolkadotXcm::send(
				sudo_origin.clone(),
				Box::new(destination.clone()),
				Box::new(xcm_claim_w3n_msg.clone())
			));

			type RuntimeEvent = <AssetHub as Chain>::RuntimeEvent;
			assert_expected_events!(
				AssetHub,
				vec![
					RuntimeEvent::PolkadotXcm(pallet_xcm::Event::Sent { .. }) => {},
				]
			);
		});

		Spiritnet::execute_with(|| {
			type SpiritnetRuntimeEvent = <Spiritnet as Chain>::RuntimeEvent;

			let is_event_present = Spiritnet::events().iter().any(|event| {
				matches!(
					event,
					SpiritnetRuntimeEvent::Did(did::Event::DidCallDispatched(_, _))
						| SpiritnetRuntimeEvent::Web3Names(pallet_web3_names::Event::Web3NameClaimed { .. })
				)
			});

			assert!(!is_event_present)
		});

		Rococo::execute_with(|| {
			assert_eq!(Rococo::events().len(), 0);
		});
	}
}
