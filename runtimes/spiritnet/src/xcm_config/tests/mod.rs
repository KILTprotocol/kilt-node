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

mod parachains;
mod relaychain;
mod utils;

use crate::{
	xcm_config::tests::{
		parachains::AssetHubPolkadot,
		relaychain::{polkadot::ED, Polkadot, System as PolkadotSystem},
	},
	PolkadotXcm as SpiritnetXcm,
};
use asset_hub_polkadot_runtime::{RuntimeEvent as AssetHubRuntimeEvent, System as AssetHubSystem};
use cumulus_pallet_xcmp_queue::Event as XcmpQueueEvent;
use frame_support::{assert_err, assert_ok};
use frame_system::RawOrigin;
use parachains::{RuntimeEvent as PeregrineRuntimeEvent, SpiritnetPolkadot, System as PeregrineSystem};
use polkadot_parachain::primitives::Sibling;
use polkadot_primitives::{AccountId, Balance};
use polkadot_service::chain_spec::get_account_id_from_seed;
use runtime_common::constants::EXISTENTIAL_DEPOSIT;
use sp_core::{sr25519, Get};
use sp_runtime::{DispatchError, ModuleError};
use xcm::prelude::*;
use xcm_emulator::{decl_test_networks, BridgeMessageHandler, Parachain, RelayChain, TestExt};
use xcm_executor::traits::ConvertLocation;

use self::{
	parachains::asset_hub_polkadot::{self, PARA_ID},
	relaychain::accounts::{ALICE, BOB},
};

decl_test_networks! {
	pub struct PolkadotNetwork {
		relay_chain = Polkadot,
		parachains = vec![
			SpiritnetPolkadot,
			AssetHubPolkadot,
		],
		bridge = ()
	}
}

/// Test that a reserved transfer to the relaychain is failing. We don't want to
/// allow transfers to the relaychain since the funds might be lost.
#[test]
fn test_reserve_asset_transfer_from_regular_account_to_relay() {
	PolkadotNetwork::reset();

	let alice_account_id_on_peregrine = get_account_id_from_seed::<sr25519::Public>(ALICE);

	SpiritnetPolkadot::execute_with(|| {
		assert_ok!(SpiritnetXcm::limited_reserve_transfer_assets(
			RawOrigin::Signed(alice_account_id_on_peregrine.clone()).into(),
			Box::new(Parent.into()),
			Box::new(
				X1(AccountId32 {
					network: None,
					id: alice_account_id_on_peregrine.into()
				})
				.into()
			),
			Box::new((Here, 1_000_000).into()),
			0,
			WeightLimit::Unlimited,
		));
		assert!(matches!(
			PeregrineSystem::events()
				.first()
				.expect("An event should be emitted when sending an XCM message.")
				.event,
			PeregrineRuntimeEvent::PolkadotXcm(pallet_xcm::Event::Attempted {
				outcome: xcm::latest::Outcome::Error(xcm::latest::Error::Barrier)
			})
		));
	});
	// No message should reach the relaychain.
	Polkadot::execute_with(|| {
		assert_eq!(PolkadotSystem::events().len(), 0);
	})
}

/// Test that a reserved transfer to the relaychain is failing. We don't want to
/// allow transfers to the relaychain since the funds might be lost.
#[test]
fn test_reserve_asset_transfer_from_regular_account_to_asset_hub() {
	PolkadotNetwork::reset();

	let alice_account_id = get_account_id_from_seed::<sr25519::Public>(ALICE);
	let bob_account_id = get_account_id_from_seed::<sr25519::Public>(BOB);

	SpiritnetPolkadot::execute_with(|| {
		assert_ok!(SpiritnetXcm::limited_reserve_transfer_assets(
			RawOrigin::Signed(alice_account_id.clone()).into(),
			Box::new(ParentThen(Junctions::X1(Junction::Parachain(asset_hub_polkadot::PARA_ID))).into()),
			Box::new(
				X1(AccountId32 {
					network: None,
					id: bob_account_id.into()
				})
				.into()
			),
			Box::new((Here, 1000 * EXISTENTIAL_DEPOSIT).into()),
			0,
			WeightLimit::Unlimited,
		));

		assert!(
			matches!(
				PeregrineSystem::events()
					.last()
					.expect("An event should be emitted when sending an XCM message.")
					.event,
				PeregrineRuntimeEvent::PolkadotXcm(pallet_xcm::Event::Attempted {
					outcome: xcm::latest::Outcome::Complete(_)
				})
			),
			"Didn't match {:?}",
			PeregrineSystem::events().last()
		);
	});
	// No event on the relaychain (message is meant for asset hub)
	Polkadot::execute_with(|| {
		assert_eq!(PolkadotSystem::events().len(), 0);
	});
	// Fails on AsssetHub since spiritnet is not a trusted registrar.
	AssetHubPolkadot::execute_with(|| {
		assert!(
			matches!(
				AssetHubSystem::events()
					.last()
					.expect("An event should be emitted when sending an XCM message.")
					.event,
				AssetHubRuntimeEvent::XcmpQueue(XcmpQueueEvent::Fail { .. })
			),
			"Didn't match {:?}",
			AssetHubSystem::events().last()
		);
	});
}

#[test]
fn test_teleport_asset_from_regular_account_to_asset_hub() {
	PolkadotNetwork::reset();

	let alice_account_id = get_account_id_from_seed::<sr25519::Public>(ALICE);
	let bob_account_id = get_account_id_from_seed::<sr25519::Public>(BOB);

	asset_hub_polkadot::force_create_asset_call(
		ParentThen(Junctions::X1(Junction::Parachain(PARA_ID))).into(),
		alice_account_id.clone(),
		true,
		0,
	);

	SpiritnetPolkadot::execute_with(|| {
		assert_err!(
			SpiritnetXcm::limited_teleport_assets(
				RawOrigin::Signed(alice_account_id.clone()).into(),
				Box::new(ParentThen(Junctions::X1(Junction::Parachain(asset_hub_polkadot::PARA_ID))).into()),
				Box::new(
					X1(AccountId32 {
						network: None,
						id: bob_account_id.into()
					})
					.into()
				),
				Box::new((Here, 1000 * EXISTENTIAL_DEPOSIT).into()),
				0,
				WeightLimit::Unlimited,
			),
			DispatchError::Module(ModuleError {
				index: 83,
				error: [2, 0, 0, 0],
				message: Some("Filtered")
			})
		);
	});
	// No event on the relaychain (message is meant for asset hub)
	Polkadot::execute_with(|| {
		assert_eq!(PolkadotSystem::events().len(), 0);
	});
	// Fails on AsssetHub since spiritnet is not a trusted registrar.
	AssetHubPolkadot::execute_with(|| {
		assert_eq!(AssetHubSystem::events().len(), 0);
	});
}

// TODO: Receive funds from assetHub
// TODO: Disallow root calls from other chains.
// TODO: create a DID from another chain
// TODO: use a DID (e.g. CType creation)
