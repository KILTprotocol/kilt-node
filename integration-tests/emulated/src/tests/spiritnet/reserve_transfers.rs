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

use asset_hub_rococo_emulated_chain::genesis::ED;
use cumulus_pallet_xcmp_queue::Event as XcmpQueueEvent;
use emulated_integration_tests_common::{accounts::ALICE, impls::cumulus_pallet_parachain_system::Module};
use frame_support::{assert_err, assert_ok, dispatch::RawOrigin, traits::fungible::Inspect};
use runtime_common::AccountId;
use sp_core::sr25519;
use sp_runtime::{traits::Zero, DispatchError, ModuleError};
use spiritnet_runtime::PolkadotXcm as SpiritnetXcm;
use xcm::lts::{
	prelude::{Here, Junction, Junctions, Parent, ParentThen},
	WeightLimit,
};

use xcm_emulator::{assert_expected_events, Chain, Network, Parachain, TestExt};

use crate::{
	mock::{
		network::{AssetHub, MockNetwork, Rococo, Spiritnet},
		para_chains::SpiritnetParachainParaPallet,
	},
	utils::get_account_id_from_seed,
};

/// Test that a reserved transfer to the relaychain is failing. We don't want to
/// allow transfers to the relaychain since the funds might be lost.
#[test]
fn test_reserve_asset_transfer_from_regular_spiritnet_account_to_relay() {
	MockNetwork::reset();

	let balance_to_transfer = 100_000_000_000_000 * ED;

	let alice_account = get_account_id_from_seed::<sr25519::Public>(ALICE);

	// Submit XCM msg
	Spiritnet::execute_with(|| {
		assert_err!(
			SpiritnetXcm::limited_reserve_transfer_assets(
				RawOrigin::Signed(alice_account.clone()).into(),
				Box::new(Parent.into()),
				Box::new(
					Junctions::X1(
						[Junction::AccountId32 {
							network: None,
							id: alice_account.into()
						}]
						.into()
					)
					.into()
				),
				Box::new((Here, balance_to_transfer).into()),
				0,
				WeightLimit::Unlimited,
			),
			sp_runtime::DispatchError::Module(sp_runtime::ModuleError {
				index: 83,
				error: [24, 0, 0, 0,],
				message: Some("LocalExecutionIncomplete")
			})
		);
	});

	// No message should reach the relaychain.
	Rococo::execute_with(|| {
		assert_eq!(Rococo::events().len(), 0);
	})
}

#[test]
fn test_reserve_asset_transfer_from_regular_spiritnet_account_to_asset_hub() {
	MockNetwork::reset();

	let alice_account_id = get_account_id_from_seed::<sr25519::Public>(ALICE);
	let asset_hub_sovereign_account =
		Spiritnet::sovereign_account_id_of(Spiritnet::sibling_location_of(AssetHub::para_id()));

	let balance_to_transfer = 100_000_000_000_000 * ED;

	Spiritnet::execute_with(|| {
		// the sovereign_account of AssetHub should have no coins.
		let balance_before_transfer =
			<<Spiritnet as SpiritnetParachainParaPallet>::Balances as Inspect<AccountId>>::balance(
				&asset_hub_sovereign_account,
			);

		assert!(balance_before_transfer.is_zero());

		// submit xcm message
		assert_ok!(SpiritnetXcm::limited_reserve_transfer_assets(
			RawOrigin::Signed(alice_account_id.clone()).into(),
			Box::new(ParentThen(Junctions::X1([Junction::Parachain(AssetHub::para_id().into())].into())).into()),
			Box::new(
				Junctions::X1(
					[Junction::AccountId32 {
						network: None,
						id: asset_hub_sovereign_account.clone().into()
					}]
					.into()
				)
				.into()
			),
			Box::new((Here, balance_to_transfer).into()),
			0,
			WeightLimit::Unlimited,
		));

		type RuntimeEvent = <Spiritnet as Chain>::RuntimeEvent;

		// we expect to have the [Complete] event.
		assert_expected_events!(
			Spiritnet,
			vec![
				RuntimeEvent::PolkadotXcm(pallet_xcm::Event::Attempted {
					outcome: xcm::latest::Outcome::Complete { .. }
				}) => {},
			]
		);

		// we also expect that the sovereignAccount of AssetHub has some coins now
		let balance_after_transfer =
			<<Spiritnet as SpiritnetParachainParaPallet>::Balances as Inspect<AccountId>>::balance(
				&asset_hub_sovereign_account,
			);

		assert_eq!(balance_after_transfer, balance_to_transfer);
	});

	// No event on the relaychain. Message is for AssetHub.
	Rococo::execute_with(|| {
		assert_eq!(Rococo::events().len(), 0);
	});

	// Fails on AssetHub since spiritnet is not a trusted registrar
	AssetHub::execute_with(|| {
		type RuntimeEvent = <AssetHub as Chain>::RuntimeEvent;

		assert_expected_events!(
			AssetHub,
			vec![
			//RuntimeEvent::XcmpQueue(XcmpQueueEvent::Fail { .. }) => {},
			]
		);
	});
}
