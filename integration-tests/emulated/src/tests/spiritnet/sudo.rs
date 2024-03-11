use crate::{
	mock::{
		network::MockNetworkPolkadot,
		para_chains::{spiritnet, AssetHubPolkadot, AssetHubPolkadotPallet, Spiritnet},
		relay_chains::{Polkadot, PolkadotPallet},
	},
	utils::UNIT,
};
use asset_hub_polkadot_runtime::System as AssetHubSystem;
use frame_support::{assert_ok, traits::fungible::Mutate};
use parity_scale_codec::Encode;
use runtime_common::AccountId;
use xcm::{v3::WeightLimit, DoubleEncoded, VersionedMultiLocation, VersionedXcm};
use xcm_emulator::{
	assert_expected_events, Here,
	Instruction::{BuyExecution, Transact, UnpaidExecution, WithdrawAsset},
	Junction, Junctions, OriginKind, Parachain, ParentThen, RelayChain, TestExt, Weight, Xcm,
};

#[test]
fn test_sudo_call_from_relay_chain_to_spiritnet() {
	MockNetworkPolkadot::reset();

	let code = vec![];

	let call: DoubleEncoded<()> = <Spiritnet as Parachain>::RuntimeCall::System(frame_system::Call::set_code { code })
		.encode()
		.into();
	let sudo_origin = <Polkadot as RelayChain>::RuntimeOrigin::root();
	let parachain_destination = Polkadot::child_location_of(spiritnet::PARA_ID.into()).into_versioned();

	let weight_limit = WeightLimit::Unlimited;
	let require_weight_at_most = Weight::from_parts(1600000000000, 200000);
	let check_origin = None;
	let origin_kind = OriginKind::Superuser;

	// the relay chain would submit an unpaid execution request.
	let xcm = VersionedXcm::from(Xcm(vec![
		UnpaidExecution {
			weight_limit,
			check_origin,
		},
		Transact {
			origin_kind,
			require_weight_at_most,
			call,
		},
	]));

	//Send XCM message from relay chain
	Polkadot::execute_with(|| {
		assert_ok!(<Polkadot as PolkadotPallet>::XcmPallet::send(
			sudo_origin,
			Box::new(parachain_destination),
			Box::new(xcm)
		));

		type RuntimeEvent = <Polkadot as RelayChain>::RuntimeEvent;

		assert_expected_events!(
			Polkadot,
			vec![
				RuntimeEvent::XcmPallet(pallet_xcm::Event::Sent { .. }) => {},
			]
		);
	});

	Spiritnet::execute_with(|| {
		type SpiritnetRuntimeEvent = <Spiritnet as Parachain>::RuntimeEvent;

		let events = Spiritnet::events();

		println!("{:?}", events);

		assert_expected_events!(
			Spiritnet,
			vec![
				SpiritnetRuntimeEvent::DmpQueue(cumulus_pallet_dmp_queue::Event::ExecutedDownward {
					outcome: xcm::v3::Outcome::Error(xcm::v3::Error::Barrier),
					..
				}) => {},
			]
		);
	});

	// No event on the AssetHub message is meant for relay chain
	AssetHubPolkadot::execute_with(|| {
		assert_eq!(AssetHubSystem::events().len(), 0);
	});
}

#[test]
fn test_sudo_call_from_asset_hub_to_spiritnet() {
	MockNetworkPolkadot::reset();

	// create the sovereign account of AssetHub
	let asset_hub_sovereign_account =
		Spiritnet::sovereign_account_id_of(Spiritnet::sibling_location_of(AssetHubPolkadot::para_id()));

	let code = vec![];

	let call: DoubleEncoded<()> = <Spiritnet as Parachain>::RuntimeCall::System(frame_system::Call::set_code { code })
		.encode()
		.into();
	let sudo_origin = <AssetHubPolkadot as Parachain>::RuntimeOrigin::root();
	let parachain_destination: VersionedMultiLocation =
		ParentThen(Junctions::X1(Junction::Parachain(spiritnet::PARA_ID))).into();

	let weight_limit = WeightLimit::Unlimited;
	let require_weight_at_most = Weight::from_parts(1600000000000, 200000);
	let origin_kind = OriginKind::Superuser;
	let init_balance = UNIT * 10;

	let xcm = VersionedXcm::from(Xcm(vec![
		WithdrawAsset((Here, init_balance).into()),
		BuyExecution {
			fees: (Here, init_balance).into(),
			weight_limit,
		},
		Transact {
			origin_kind,
			require_weight_at_most,
			call,
		},
	]));

	// give the sovereign account of AssetHub some coins.
	Spiritnet::execute_with(|| {
		<spiritnet_runtime::Balances as Mutate<AccountId>>::set_balance(&asset_hub_sovereign_account, init_balance);
	});

	//Send XCM message from AssetHub
	AssetHubPolkadot::execute_with(|| {
		assert_ok!(<AssetHubPolkadot as AssetHubPolkadotPallet>::PolkadotXcm::send(
			sudo_origin,
			Box::new(parachain_destination),
			Box::new(xcm)
		));

		type RuntimeEvent = <AssetHubPolkadot as Parachain>::RuntimeEvent;

		assert_expected_events!(
			AssetHubPolkadot,
			vec![
				RuntimeEvent::PolkadotXcm(pallet_xcm::Event::Sent { .. }) => {},
			]
		);
	});

	Spiritnet::execute_with(|| {
		type SpiritnetRuntimeEvent = <Spiritnet as Parachain>::RuntimeEvent;

		assert_expected_events!(
			Spiritnet,
			vec![
				SpiritnetRuntimeEvent::XcmpQueue(cumulus_pallet_xcmp_queue::Event::Fail {
					error: xcm::v3::Error::NoPermission,
					..
				}) => {},
			]
		);
	});

	// No event on the relaychain (message is meant for asset hub)
	Polkadot::execute_with(|| {
		assert_eq!(Polkadot::events().len(), 0);
	});
}
