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

use frame_support::{assert_ok, traits::fungible::Mutate};
use parity_scale_codec::Encode;
use runtime_common::{constants::KILT, AccountId, Balance};
use sp_core::H256;
use xcm::{v3::prelude::OriginKind, DoubleEncoded, VersionedXcm};
use xcm_emulator::{assert_expected_events, Chain, Network, TestExt};

use crate::{
	mock::{
		network::MockNetworkPolkadot,
		para_chains::{AssetHubPolkadot, AssetHubPolkadotPallet, Spiritnet},
		relay_chains::Polkadot,
	},
	tests::spiritnet::did_pallets::utils::{
		construct_basic_transact_xcm_message, create_mock_ctype, create_mock_did_from_account,
		get_asset_hub_sovereign_account, get_sibling_destination_spiritnet,
	},
};

fn get_xcm_message_add_public_credential(
	origin_kind: OriginKind,
	withdraw_balance: Balance,
	ctype_hash: H256,
) -> VersionedXcm<()> {
	let asset_hub_sovereign_account: sp_runtime::AccountId32 = get_asset_hub_sovereign_account();

	let subject_id = b"did:asset:eip155:1.slip44:60".to_vec();

	let credential = public_credentials::mock::generate_base_public_credential_creation_op::<spiritnet_runtime::Runtime>(
		subject_id.try_into().unwrap(),
		ctype_hash,
		Default::default(),
	);

	let call: DoubleEncoded<()> = <Spiritnet as Chain>::RuntimeCall::Did(did::Call::dispatch_as {
		did_identifier: asset_hub_sovereign_account,
		call: Box::new(<Spiritnet as Chain>::RuntimeCall::PublicCredentials(
			public_credentials::Call::add {
				credential: Box::new(credential),
			},
		)),
	})
	.encode()
	.into();

	construct_basic_transact_xcm_message(origin_kind, withdraw_balance, call)
}

#[test]
fn test_create_public_credential_from_asset_hub_successful() {
	MockNetworkPolkadot::reset();

	let sudo_origin = <AssetHubPolkadot as Chain>::RuntimeOrigin::root();
	let asset_hub_sovereign_account = get_asset_hub_sovereign_account();
	let ctype_hash_value = H256([0; 32]);

	let init_balance = KILT * 10;

	let xcm_issue_public_credential_msg =
		get_xcm_message_add_public_credential(OriginKind::SovereignAccount, KILT, ctype_hash_value);

	let destination = get_sibling_destination_spiritnet();

	Spiritnet::execute_with(|| {
		create_mock_did_from_account(asset_hub_sovereign_account.clone());
		create_mock_ctype(ctype_hash_value, asset_hub_sovereign_account.clone());
		<spiritnet_runtime::Balances as Mutate<AccountId>>::set_balance(&asset_hub_sovereign_account, init_balance);
	});

	AssetHubPolkadot::execute_with(|| {
		assert_ok!(<AssetHubPolkadot as AssetHubPolkadotPallet>::PolkadotXcm::send(
			sudo_origin,
			Box::new(destination),
			Box::new(xcm_issue_public_credential_msg)
		));

		type RuntimeEvent = <AssetHubPolkadot as Chain>::RuntimeEvent;
		assert_expected_events!(
			AssetHubPolkadot,
			vec![
				RuntimeEvent::PolkadotXcm(pallet_xcm::Event::Sent { .. }) => {},
			]
		);
	});

	#[cfg(not(feature = "runtime-benchmarks"))]
	Spiritnet::execute_with(|| {
		type SpiritnetRuntimeEvent = <Spiritnet as Chain>::RuntimeEvent;

		assert_expected_events!(
			Spiritnet,
			vec![
				SpiritnetRuntimeEvent::XcmpQueue(cumulus_pallet_xcmp_queue::Event::Success { .. }) => {},
				SpiritnetRuntimeEvent::Did(did::Event::DidCallDispatched(account, result)) => {
					account: account == &asset_hub_sovereign_account,
					result: result.is_ok(),
				},
				SpiritnetRuntimeEvent::PublicCredentials(public_credentials::Event::CredentialStored{ .. }) => {

				},
			]
		);
	});

	Polkadot::execute_with(|| {
		assert_eq!(Polkadot::events().len(), 0);
	});
}

#[test]
fn test_create_public_credential_from_asset_hub_unsuccessful() {
	let origin_kind_list = vec![OriginKind::Native, OriginKind::Superuser, OriginKind::Xcm];

	let sudo_origin = <AssetHubPolkadot as Chain>::RuntimeOrigin::root();
	let init_balance = KILT * 100;
	let ctype_hash_value = H256([0; 32]);

	let destination = get_sibling_destination_spiritnet();
	let asset_hub_sovereign_account = get_asset_hub_sovereign_account();

	for origin_kind in origin_kind_list {
		MockNetworkPolkadot::reset();

		Polkadot::execute_with(|| {
			create_mock_did_from_account(asset_hub_sovereign_account.clone());
			create_mock_ctype(ctype_hash_value, asset_hub_sovereign_account.clone());
			<spiritnet_runtime::Balances as Mutate<AccountId>>::set_balance(&asset_hub_sovereign_account, init_balance);
		});

		let xcm_issue_public_credential_msg =
			get_xcm_message_add_public_credential(origin_kind, KILT, ctype_hash_value);

		AssetHubPolkadot::execute_with(|| {
			assert_ok!(<AssetHubPolkadot as AssetHubPolkadotPallet>::PolkadotXcm::send(
				sudo_origin.clone(),
				Box::new(destination.clone()),
				Box::new(xcm_issue_public_credential_msg.clone())
			));

			type RuntimeEvent = <AssetHubPolkadot as Chain>::RuntimeEvent;
			assert_expected_events!(
				AssetHubPolkadot,
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
						| SpiritnetRuntimeEvent::DidLookup(pallet_did_lookup::Event::AssociationEstablished(_, _))
				)
			});

			assert!(!is_event_present)
		});

		Polkadot::execute_with(|| {
			assert_eq!(Polkadot::events().len(), 0);
		});
	}
}
