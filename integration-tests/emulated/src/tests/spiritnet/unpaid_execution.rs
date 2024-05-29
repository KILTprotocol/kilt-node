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

use did::did_details::DidVerificationKey;
use frame_support::{
	assert_ok,
	traits::fungible::{Inspect, Mutate},
	weights::Weight,
};
use parity_scale_codec::Encode;
use runtime_common::{constants::EXISTENTIAL_DEPOSIT, AccountId};
use xcm::{
	v3::{
		prelude::{OriginKind, Transact, UnpaidExecution},
		Junction, Junctions, Outcome, ParentThen, WeightLimit, Xcm,
	},
	DoubleEncoded, VersionedMultiLocation, VersionedXcm,
};
use xcm_emulator::{assert_expected_events, Chain, Network, Parachain, TestExt};

use crate::mock::{
	network::MockNetworkPolkadot,
	para_chains::{spiritnet, AssetHubPolkadot, AssetHubPolkadotPallet, Spiritnet, SpiritnetPallet},
	relay_chains::{Polkadot, PolkadotPallet},
};

#[test]
fn test_unpaid_execution_to_spiritnet() {
	MockNetworkPolkadot::reset();

	let sudo_origin = <AssetHubPolkadot as Chain>::RuntimeOrigin::root();
	let parachain_destination: VersionedMultiLocation =
		ParentThen(Junctions::X1(Junction::Parachain(spiritnet::PARA_ID))).into();

	let weight_limit = WeightLimit::Unlimited;
	let check_origin = None;

	let xcm = VersionedXcm::from(Xcm(vec![UnpaidExecution {
		weight_limit,
		check_origin,
	}]));

	//Send XCM message from AssetHub
	AssetHubPolkadot::execute_with(|| {
		assert_ok!(<AssetHubPolkadot as AssetHubPolkadotPallet>::PolkadotXcm::send(
			sudo_origin,
			Box::new(parachain_destination),
			Box::new(xcm)
		));

		type RuntimeEvent = <AssetHubPolkadot as Chain>::RuntimeEvent;

		assert_expected_events!(
			AssetHubPolkadot,
			vec![
				RuntimeEvent::PolkadotXcm(pallet_xcm::Event::Sent { .. }) => {},
			]
		);
	});

	// Execution should be blocked by barrier
	Spiritnet::execute_with(|| {
		type SpiritnetRuntimeEvent = <Spiritnet as Chain>::RuntimeEvent;
		assert_expected_events!(
			Spiritnet,
			vec![
				SpiritnetRuntimeEvent::XcmpQueue(cumulus_pallet_xcmp_queue::Event::Fail {
					error: xcm::v3::Error::Barrier,
					..
				}) => {},
			]
		);
	});

	// No event on the Polkadot message is meant for Spiritnet
	Polkadot::execute_with(|| {
		assert_eq!(Polkadot::events().len(), 0);
	});
}

#[test]
fn test_unpaid_execution_from_polkadot_to_spiritnet() {
	MockNetworkPolkadot::reset();

	let sudo_origin = <Polkadot as Chain>::RuntimeOrigin::root();
	let parachain_destination: VersionedMultiLocation = Junctions::X1(Junction::Parachain(spiritnet::PARA_ID)).into();
	let init_balance = <spiritnet_runtime::Runtime as did::Config>::BaseDeposit::get()
		+ <spiritnet_runtime::Runtime as did::Config>::Fee::get()
		+ EXISTENTIAL_DEPOSIT;

	let weight_limit = WeightLimit::Unlimited;
	let check_origin = None;

	let polkadot_sovereign_account = Spiritnet::sovereign_account_id_of(Spiritnet::parent_location());

	let call: DoubleEncoded<()> = <Spiritnet as Chain>::RuntimeCall::Did(did::Call::create_from_account {
		authentication_key: DidVerificationKey::Account(polkadot_sovereign_account.clone()),
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

	Spiritnet::execute_with(|| {
		// DID creation takes a deposit of 2 KILT coins + Fees. We have to give them to
		// the sovereign account. Otherwise, the extrinsic will fail.
		<<Spiritnet as SpiritnetPallet>::Balances as Mutate<AccountId>>::set_balance(
			&polkadot_sovereign_account,
			init_balance,
		);
	});

	// Submit XCM msg from relaychain
	Polkadot::execute_with(|| {
		assert_ok!(<Polkadot as PolkadotPallet>::XcmPallet::send(
			sudo_origin,
			Box::new(parachain_destination),
			Box::new(xcm)
		));

		type RuntimeEvent = <Polkadot as Chain>::RuntimeEvent;

		assert_expected_events!(
			Polkadot,
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
				SpiritnetRuntimeEvent::DmpQueue(cumulus_pallet_dmp_queue::Event::ExecutedDownward {
					outcome: Outcome::Complete(_),
					..
				}) => {},
				SpiritnetRuntimeEvent::Did(did::Event::DidCreated(account, did_identifier)) => {
					account: account == &polkadot_sovereign_account,
					did_identifier:  did_identifier == &polkadot_sovereign_account,
				},
			]
		);

		// Since the user have not paid any tx fees, we expect that the free balance is
		// the ED
		let balance_after_transfer =
			<<Spiritnet as SpiritnetPallet>::Balances as Inspect<AccountId>>::balance(&polkadot_sovereign_account);

		assert_eq!(balance_after_transfer, EXISTENTIAL_DEPOSIT);
	});

	// No event on AssetHubPolkadot. message is meant for Spiritnet
	AssetHubPolkadot::execute_with(|| {
		assert_eq!(AssetHubPolkadot::events().len(), 0);
	});
}
