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
use rococo_emulated_chain::RococoRelayPallet;
use runtime_common::{constants::KILT, AccountId, Balance};
use xcm::{lts::prelude::*, opaque::*, DoubleEncoded, VersionedLocation};
use xcm_emulator::{assert_expected_events, Chain, Network, Parachain, RelayChain, TestExt, Weight};

use crate::mock::network::{AssetHub, MockNetwork, Rococo, Spiritnet};

fn get_sovereign_account_id_of_asset_hub() -> AccountId {
	Spiritnet::sovereign_account_id_of(Spiritnet::sibling_location_of(AssetHub::para_id()))
}

fn get_parachain_destination_from_parachain() -> VersionedLocation {
	ParentThen(Junctions::X1([Junction::Parachain(Spiritnet::para_id().into())].into())).into()
}

fn get_parachain_destination_from_relay_chain() -> VersionedLocation {
	Rococo::child_location_of(Spiritnet::para_id()).into_versioned()
}

fn get_unpaid_xcm_message(origin_kind: OriginKind) -> VersionedXcm {
	let code = vec![];
	let call: DoubleEncoded<()> = <Spiritnet as Chain>::RuntimeCall::System(frame_system::Call::set_code { code })
		.encode()
		.into();
	let weight_limit = WeightLimit::Unlimited;
	let require_weight_at_most = Weight::from_parts(1600000000000, 200000);

	VersionedXcm::from(Xcm(vec![
		UnpaidExecution {
			weight_limit,
			check_origin: None,
		},
		Transact {
			origin_kind,
			require_weight_at_most,
			call,
		},
	]))
}

fn get_paid_xcm_message(init_balance: Balance, origin_kind: OriginKind) -> VersionedXcm {
	let code = vec![];

	let call: DoubleEncoded<()> = <Spiritnet as Chain>::RuntimeCall::System(frame_system::Call::set_code { code })
		.encode()
		.into();
	let weight_limit = WeightLimit::Unlimited;
	let require_weight_at_most = Weight::from_parts(1600000000000, 200000);
	let withdraw_asset = init_balance / 2;

	VersionedXcm::from(Xcm(vec![
		WithdrawAsset((Here, withdraw_asset).into()),
		BuyExecution {
			fees: (Here, withdraw_asset).into(),
			weight_limit,
		},
		Transact {
			origin_kind,
			require_weight_at_most,
			call,
		},
	]))
}

/// Sudo calls from other chains should not be whitelisted and therefore fail.
#[test]
fn test_sudo_call_from_relay_chain_to_spiritnet() {
	let sudo_origin = <Rococo as Chain>::RuntimeOrigin::root();
	let parachain_destination = get_parachain_destination_from_relay_chain();

	let origin_kind_list = vec![
		OriginKind::Superuser,
		OriginKind::Native,
		OriginKind::SovereignAccount,
		OriginKind::Xcm,
	];

	for origin_kind in origin_kind_list {
		MockNetwork::reset();

		let xcm = get_unpaid_xcm_message(origin_kind);

		Rococo::execute_with(|| {
			assert_ok!(<Rococo as RococoRelayPallet>::XcmPallet::send(
				sudo_origin.clone(),
				Box::new(parachain_destination.clone()),
				Box::new(xcm.clone()),
			));

			type RuntimeEvent = <Rococo as Chain>::RuntimeEvent;

			assert_expected_events!(
				Rococo,
				vec![
					RuntimeEvent::XcmPallet(pallet_xcm::Event::Sent { .. }) => {},
				]
			);
		});

		Spiritnet::execute_with(|| {
			type SpiritnetRuntimeEvent = <Spiritnet as Chain>::RuntimeEvent;

			assert_expected_events!(
				Spiritnet,
				vec![
					// SpiritnetRuntimeEvent::DmpQueue(cumulus_pallet_dmp_queue::Event::ExecutedDownward {
					// 	outcome: xcm::v3::Outcome::Incomplete(_, xcm::v3::Error::NoPermission),
					// 	..
					// }) => {},
				]
			);
		});

		// No events on other parachains. Message was for the relaychain
		AssetHub::execute_with(|| {
			assert_eq!(AssetHub::events().len(), 0);
		});
	}
}

/// Sudo calls from other chains should not be whitelisted and therefore fail.
#[test]
fn test_sudo_call_from_asset_hub_to_spiritnet() {
	let asset_hub_sovereign_account = get_sovereign_account_id_of_asset_hub();

	let sudo_origin = <AssetHub as Chain>::RuntimeOrigin::root();
	let parachain_destination = get_parachain_destination_from_parachain();
	let init_balance = KILT * 10;

	let origin_kind_list = vec![
		OriginKind::Superuser,
		OriginKind::Native,
		OriginKind::SovereignAccount,
		OriginKind::Xcm,
	];

	for origin_kind in origin_kind_list {
		MockNetwork::reset();
		let xcm = get_paid_xcm_message(init_balance, origin_kind);

		// Give some coins to pay the fees
		Spiritnet::execute_with(|| {
			<spiritnet_runtime::Balances as Mutate<AccountId>>::set_balance(&asset_hub_sovereign_account, init_balance);
		});

		// Send msg to Spiritnet
		AssetHub::execute_with(|| {
			assert_ok!(<AssetHub as AssetHubRococoParaPallet>::PolkadotXcm::send(
				sudo_origin.clone(),
				Box::new(parachain_destination.clone()),
				Box::new(xcm.clone())
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
					// SpiritnetRuntimeEvent::XcmpQueue(cumulus_pallet_xcmp_queue::Event::Fail {
					// 	error: xcm::v3::Error::NoPermission,
					// 	..
					// }) => {},
				]
			);
		});

		// No events on the relaychain. Message was for Spiritnet
		Rococo::execute_with(|| {
			assert_eq!(Rococo::events().len(), 0);
		});
	}
}
