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
use xcm::{DoubleEncoded, VersionedXcm};
use xcm_emulator::{assert_expected_events, OriginKind, Parachain, TestExt};

use crate::{
	mock::{
		network::MockNetworkRococo,
		para_chains::{AssetHubRococo, AssetHubRococoPallet, Peregrine},
		relay_chains::Rococo,
	},
	tests::peregrine::did_pallets::utils::{
		construct_basic_transact_xcm_message, create_mock_ctype, create_mock_did_from_account,
		get_asset_hub_sovereign_account, get_sibling_destination_peregrine,
	},
};

fn get_xcm_message_add_public_credential(
	origin_kind: OriginKind,
	withdraw_balance: Balance,
	ctype_hash: H256,
) -> VersionedXcm<()> {
	let asset_hub_sovereign_account = get_asset_hub_sovereign_account();

	let subject_id = b"did:asset:eip155:1.slip44:60".to_vec();

	let credential = public_credentials::mock::generate_base_public_credential_creation_op::<peregrine_runtime::Runtime>(
		subject_id.try_into().unwrap(),
		ctype_hash,
		Default::default(),
	);

	let call: DoubleEncoded<()> = <Peregrine as Parachain>::RuntimeCall::Did(did::Call::dispatch_as {
		did_identifier: asset_hub_sovereign_account,
		call: Box::new(<Peregrine as Parachain>::RuntimeCall::PublicCredentials(
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
	MockNetworkRococo::reset();

	let sudo_origin = <AssetHubRococo as Parachain>::RuntimeOrigin::root();
	let asset_hub_sovereign_account = get_asset_hub_sovereign_account();
	let ctype_hash_value = H256([0; 32]);

	let init_balance = KILT * 10;

	let xcm_issue_public_credential_msg =
		get_xcm_message_add_public_credential(OriginKind::SovereignAccount, KILT, ctype_hash_value);

	let destination = get_sibling_destination_peregrine();

	Peregrine::execute_with(|| {
		create_mock_did_from_account(asset_hub_sovereign_account.clone());
		create_mock_ctype(ctype_hash_value, asset_hub_sovereign_account.clone());
		<peregrine_runtime::Balances as Mutate<AccountId>>::set_balance(&asset_hub_sovereign_account, init_balance);
	});

	AssetHubRococo::execute_with(|| {
		assert_ok!(<AssetHubRococo as AssetHubRococoPallet>::PolkadotXcm::send(
			sudo_origin,
			Box::new(destination),
			Box::new(xcm_issue_public_credential_msg)
		));

		type RuntimeEvent = <AssetHubRococo as Parachain>::RuntimeEvent;
		assert_expected_events!(
			AssetHubRococo,
			vec![
				RuntimeEvent::PolkadotXcm(pallet_xcm::Event::Sent { .. }) => {},
			]
		);
	});

	#[cfg(not(feature = "runtime-benchmarks"))]
	Peregrine::execute_with(|| {
		type PeregrineRuntimeEvent = <Peregrine as Parachain>::RuntimeEvent;

		assert_expected_events!(
			Peregrine,
			vec![
				PeregrineRuntimeEvent::XcmpQueue(cumulus_pallet_xcmp_queue::Event::Success { .. }) => {},
				PeregrineRuntimeEvent::Did(did::Event::DidCallDispatched(account, result)) => {
					account: account == &asset_hub_sovereign_account,
					result: result.is_ok(),
				},
				PeregrineRuntimeEvent::PublicCredentials(public_credentials::Event::CredentialStored{ .. }) => {

				},
			]
		);
	});

	Rococo::execute_with(|| {
		assert_eq!(Rococo::events().len(), 0);
	});
}

#[test]
fn test_create_public_credential_from_asset_hub_unsuccessful() {
	let origin_kind_list = vec![OriginKind::Native, OriginKind::Superuser, OriginKind::Xcm];

	let sudo_origin = <AssetHubRococo as Parachain>::RuntimeOrigin::root();
	let init_balance = KILT * 100;
	let ctype_hash_value = H256([0; 32]);

	let destination = get_sibling_destination_peregrine();
	let asset_hub_sovereign_account = get_asset_hub_sovereign_account();

	for origin_kind in origin_kind_list {
		MockNetworkRococo::reset();

		Peregrine::execute_with(|| {
			create_mock_did_from_account(asset_hub_sovereign_account.clone());
			create_mock_ctype(ctype_hash_value, asset_hub_sovereign_account.clone());
			<peregrine_runtime::Balances as Mutate<AccountId>>::set_balance(&asset_hub_sovereign_account, init_balance);
		});

		let xcm_issue_public_credential_msg =
			get_xcm_message_add_public_credential(origin_kind, KILT, ctype_hash_value);

		AssetHubRococo::execute_with(|| {
			assert_ok!(<AssetHubRococo as AssetHubRococoPallet>::PolkadotXcm::send(
				sudo_origin.clone(),
				Box::new(destination.clone()),
				Box::new(xcm_issue_public_credential_msg.clone())
			));

			type RuntimeEvent = <AssetHubRococo as Parachain>::RuntimeEvent;
			assert_expected_events!(
				AssetHubRococo,
				vec![
					RuntimeEvent::PolkadotXcm(pallet_xcm::Event::Sent { .. }) => {},
				]
			);
		});

		Peregrine::execute_with(|| {
			type PeregrineRuntimeEvent = <Peregrine as Parachain>::RuntimeEvent;

			let is_event_present = Peregrine::events().iter().any(|event| {
				matches!(
					event,
					PeregrineRuntimeEvent::Did(did::Event::DidCallDispatched(_, _))
						| PeregrineRuntimeEvent::DidLookup(pallet_did_lookup::Event::AssociationEstablished(_, _))
				)
			});

			assert!(!is_event_present)
		});

		Rococo::execute_with(|| {
			assert_eq!(Rococo::events().len(), 0);
		});
	}
}
