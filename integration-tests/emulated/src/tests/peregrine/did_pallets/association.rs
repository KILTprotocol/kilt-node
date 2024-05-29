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
use xcm::{v3::prelude::OriginKind, DoubleEncoded, VersionedXcm};
use xcm_emulator::{assert_expected_events, Chain, Network, TestExt};

use crate::{
	mock::{
		network::MockNetworkRococo,
		para_chains::{AssetHubRococo, AssetHubRococoPallet, Peregrine},
		relay_chains::Rococo,
	},
	tests::peregrine::did_pallets::utils::{
		construct_basic_transact_xcm_message, create_mock_did_from_account, get_asset_hub_sovereign_account,
		get_sibling_destination_peregrine,
	},
};

fn get_xcm_message_add_association(origin_kind: OriginKind, withdraw_balance: Balance) -> VersionedXcm<()> {
	let asset_hub_sovereign_account = get_asset_hub_sovereign_account();

	let call: DoubleEncoded<()> = <Peregrine as Chain>::RuntimeCall::Did(did::Call::dispatch_as {
		did_identifier: asset_hub_sovereign_account,
		call: Box::new(<Peregrine as Chain>::RuntimeCall::DidLookup(
			pallet_did_lookup::Call::associate_sender {},
		)),
	})
	.encode()
	.into();

	construct_basic_transact_xcm_message(origin_kind, withdraw_balance, call)
}

#[test]
fn test_create_association_from_asset_hub_successful() {
	MockNetworkRococo::reset();

	let sudo_origin = <AssetHubRococo as Chain>::RuntimeOrigin::root();
	let asset_hub_sovereign_account = get_asset_hub_sovereign_account();

	let init_balance = KILT * 10;

	let xcm_add_association_msg = get_xcm_message_add_association(OriginKind::SovereignAccount, KILT);
	let destination = get_sibling_destination_peregrine();

	Peregrine::execute_with(|| {
		create_mock_did_from_account(asset_hub_sovereign_account.clone());
		<peregrine_runtime::Balances as Mutate<AccountId>>::set_balance(&asset_hub_sovereign_account, init_balance);
	});

	AssetHubRococo::execute_with(|| {
		assert_ok!(<AssetHubRococo as AssetHubRococoPallet>::PolkadotXcm::send(
			sudo_origin,
			Box::new(destination),
			Box::new(xcm_add_association_msg)
		));

		type RuntimeEvent = <AssetHubRococo as Chain>::RuntimeEvent;
		assert_expected_events!(
			AssetHubRococo,
			vec![
				RuntimeEvent::PolkadotXcm(pallet_xcm::Event::Sent { .. }) => {},
			]
		);
	});

	#[cfg(not(feature = "runtime-benchmarks"))]
	Peregrine::execute_with(|| {
		type PeregrineRuntimeEvent = <Peregrine as Chain>::RuntimeEvent;

		assert_expected_events!(
			Peregrine,
			vec![
				PeregrineRuntimeEvent::XcmpQueue(cumulus_pallet_xcmp_queue::Event::Success { .. }) => {},
				PeregrineRuntimeEvent::Did(did::Event::DidCallDispatched(account, result)) => {
					account: account == &asset_hub_sovereign_account,
					result: result.is_ok(),
				},
				PeregrineRuntimeEvent::DidLookup(pallet_did_lookup::Event::AssociationEstablished(owner, did)) => {
					owner: owner.as_ref() == <sp_runtime::AccountId32 as AsRef<[u8]>>::as_ref(&asset_hub_sovereign_account),
					did: did == &asset_hub_sovereign_account,
				},
			]
		);
	});

	Rococo::execute_with(|| {
		assert_eq!(Rococo::events().len(), 0);
	});
}

#[test]
fn test_create_association_from_asset_hub_unsuccessful() {
	let origin_kind_list = vec![OriginKind::Native, OriginKind::Superuser, OriginKind::Xcm];

	let sudo_origin = <AssetHubRococo as Chain>::RuntimeOrigin::root();
	let init_balance = KILT * 100;

	let destination = get_sibling_destination_peregrine();
	let asset_hub_sovereign_account = get_asset_hub_sovereign_account();

	for origin_kind in origin_kind_list {
		MockNetworkRococo::reset();

		Peregrine::execute_with(|| {
			create_mock_did_from_account(asset_hub_sovereign_account.clone());
			<peregrine_runtime::Balances as Mutate<AccountId>>::set_balance(&asset_hub_sovereign_account, init_balance);
		});

		let xcm_add_association_msg = get_xcm_message_add_association(origin_kind, KILT);

		AssetHubRococo::execute_with(|| {
			assert_ok!(<AssetHubRococo as AssetHubRococoPallet>::PolkadotXcm::send(
				sudo_origin.clone(),
				Box::new(destination.clone()),
				Box::new(xcm_add_association_msg.clone())
			));

			type RuntimeEvent = <AssetHubRococo as Chain>::RuntimeEvent;
			assert_expected_events!(
				AssetHubRococo,
				vec![
					RuntimeEvent::PolkadotXcm(pallet_xcm::Event::Sent { .. }) => {},
				]
			);
		});

		Peregrine::execute_with(|| {
			type PeregrineRuntimeEvent = <Peregrine as Chain>::RuntimeEvent;

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
