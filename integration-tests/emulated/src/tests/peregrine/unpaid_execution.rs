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
use did::did_details::DidVerificationKey;
use frame_support::{
	assert_ok,
	traits::fungible::{Inspect, Mutate},
	weights::Weight,
};
use parity_scale_codec::Encode;
use rococo_emulated_chain::RococoRelayPallet;
use runtime_common::{constants::EXISTENTIAL_DEPOSIT, AccountId};
use xcm::{
	lts::{
		prelude::{OriginKind, Transact, UnpaidExecution},
		Junction, Junctions, ParentThen, WeightLimit, Xcm,
	},
	DoubleEncoded, VersionedLocation, VersionedXcm,
};
use xcm_emulator::{assert_expected_events, Chain, Network, Parachain, TestExt};

use crate::mock::{
	network::{AssetHub, MockNetwork, Peregrine, Rococo},
	para_chains::PeregrineParachainParaPallet,
};

#[test]
fn test_unpaid_execution_from_asset_hub_to_peregrine() {
	MockNetwork::reset();

	let sudo_origin = <AssetHub as Chain>::RuntimeOrigin::root();
	let parachain_destination: VersionedLocation =
		ParentThen(Junctions::X1([Junction::Parachain(Peregrine::para_id().into())].into())).into();

	let weight_limit = WeightLimit::Unlimited;
	let check_origin = None;

	let xcm = VersionedXcm::from(Xcm(vec![UnpaidExecution {
		weight_limit,
		check_origin,
	}]));

	//Send XCM message from Parachain
	AssetHub::execute_with(|| {
		assert_ok!(<AssetHub as AssetHubRococoParaPallet>::PolkadotXcm::send(
			sudo_origin,
			Box::new(parachain_destination),
			Box::new(xcm)
		));

		type RuntimeEvent = <AssetHub as Chain>::RuntimeEvent;

		assert_expected_events!(
			AssetHub,
			vec![
				RuntimeEvent::PolkadotXcm(pallet_xcm::Event::Sent { .. }) => {},
			]
		);
	});

	Peregrine::execute_with(|| {
		type PeregrineRuntimeEvent = <Peregrine as Chain>::RuntimeEvent;
		assert_expected_events!(
			Peregrine,
			vec![
				PeregrineRuntimeEvent::MessageQueue(pallet_message_queue::Event::ProcessingFailed { .. }) => {},
			]
		);
	});

	// No event on Rococo. message is meant for Peregrine
	Rococo::execute_with(|| {
		assert_eq!(Rococo::events().len(), 0);
	});
}

#[test]
fn test_unpaid_execution_from_rococo_to_peregrine() {
	MockNetwork::reset();

	let sudo_origin = <Rococo as Chain>::RuntimeOrigin::root();
	let parachain_destination: VersionedLocation =
		Junctions::X1([Junction::Parachain(Peregrine::para_id().into())].into()).into();
	let init_balance = <peregrine_runtime::Runtime as did::Config>::BaseDeposit::get()
		+ <peregrine_runtime::Runtime as did::Config>::Fee::get()
		+ EXISTENTIAL_DEPOSIT;

	let weight_limit = WeightLimit::Unlimited;
	let check_origin = None;

	let rococo_sovereign_account = Peregrine::sovereign_account_id_of(Peregrine::parent_location());

	let call: DoubleEncoded<()> = <Peregrine as Chain>::RuntimeCall::Did(did::Call::create_from_account {
		authentication_key: DidVerificationKey::Account(rococo_sovereign_account.clone()),
	})
	.encode()
	.into();

	let xcm = VersionedXcm::from(Xcm(vec![
		UnpaidExecution {
			weight_limit,
			check_origin,
		},
		Transact {
			origin_kind: OriginKind::SovereignAccount,
			require_weight_at_most: Weight::from_parts(10_000_600_000_000, 200_000_000_000),
			call,
		},
	]));

	Peregrine::execute_with(|| {
		// DID creation takes a deposit of 2 KILT coins + Fees. We have to give them to
		// the sovereign account. Otherwise, the extrinsic will fail.
		<peregrine_runtime::Balances as Mutate<AccountId>>::set_balance(&rococo_sovereign_account, init_balance);
	});

	//Send XCM message from relaychain
	Rococo::execute_with(|| {
		assert_ok!(<Rococo as RococoRelayPallet>::XcmPallet::send(
			sudo_origin,
			Box::new(parachain_destination),
			Box::new(xcm)
		));

		type RuntimeEvent = <Rococo as Chain>::RuntimeEvent;

		assert_expected_events!(
			Rococo,
			vec![
				RuntimeEvent::XcmPallet(pallet_xcm::Event::Sent { .. }) => {},
			]
		);
	});

	Peregrine::execute_with(|| {
		type PeregrineRuntimeEvent = <Peregrine as Chain>::RuntimeEvent;
		assert_expected_events!(
			Peregrine,
			vec![
				PeregrineRuntimeEvent::MessageQueue(pallet_message_queue::Event::Processed { success: true,  .. }) => {},
				PeregrineRuntimeEvent::Did(did::Event::DidCreated(account, did_identifier)) => {
					account: account == &rococo_sovereign_account,
					did_identifier:  did_identifier == &rococo_sovereign_account,
				},
			]
		);

		// Since the user have not paid any tx fees, we expect that the free balance is
		// the ED
		let balance_after_transfer: u128 = <<Peregrine as PeregrineParachainParaPallet>::Balances as Inspect<
			AccountId,
		>>::balance(&rococo_sovereign_account);

		assert_eq!(balance_after_transfer, EXISTENTIAL_DEPOSIT);
	});

	// No event on AssetHub. message is meant for Peregrine
	AssetHub::execute_with(|| {
		assert_eq!(AssetHub::events().len(), 0);
	});
}
