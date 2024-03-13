use crate::{
	mock::{
		network::MockNetworkRococo,
		para_chains::{peregrine, AssetHubRococo, AssetHubRococoPallet, Peregrine},
		relay_chains::{Rococo, RococoPallet},
	},
	utils::UNIT,
};
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
fn test_sudo_call_from_relay_chain_to_peregrine() {
	MockNetworkRococo::reset();

	let code = vec![];

	let call: DoubleEncoded<()> = <Peregrine as Parachain>::RuntimeCall::System(frame_system::Call::set_code { code })
		.encode()
		.into();
	let sudo_origin = <Rococo as RelayChain>::RuntimeOrigin::root();
	let parachain_destination = Rococo::child_location_of(peregrine::PARA_ID.into()).into_versioned();

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
	Rococo::execute_with(|| {
		assert_ok!(<Rococo as RococoPallet>::XcmPallet::send(
			sudo_origin,
			Box::new(parachain_destination),
			Box::new(xcm)
		));

		type RuntimeEvent = <Rococo as RelayChain>::RuntimeEvent;

		assert_expected_events!(
			Rococo,
			vec![
				RuntimeEvent::XcmPallet(pallet_xcm::Event::Sent { .. }) => {},
			]
		);
	});

	Peregrine::execute_with(|| {
		type PeregrineRuntimeEvent = <Peregrine as Parachain>::RuntimeEvent;

		assert_expected_events!(
			Peregrine,
			vec![
				PeregrineRuntimeEvent::DmpQueue(cumulus_pallet_dmp_queue::Event::ExecutedDownward {
					outcome: xcm::v3::Outcome::Error(xcm::v3::Error::Barrier),
					..
				}) => {},
			]
		);
	});

	// No event on the AssetHub message is meant for relay chain
	AssetHubRococo::execute_with(|| {
		assert_eq!(AssetHubRococo::events().len(), 0);
	});
}

#[test]
fn test_sudo_call_from_asset_hub_to_peregrine() {
	MockNetworkRococo::reset();

	// create the sovereign account of AssetHub
	let asset_hub_sovereign_account =
		Peregrine::sovereign_account_id_of(Peregrine::sibling_location_of(AssetHubRococo::para_id()));

	let code = vec![];

	let call: DoubleEncoded<()> = <Peregrine as Parachain>::RuntimeCall::System(frame_system::Call::set_code { code })
		.encode()
		.into();
	let sudo_origin = <AssetHubRococo as Parachain>::RuntimeOrigin::root();
	let parachain_destination: VersionedMultiLocation =
		ParentThen(Junctions::X1(Junction::Parachain(peregrine::PARA_ID))).into();

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
	Peregrine::execute_with(|| {
		<peregrine_runtime::Balances as Mutate<AccountId>>::set_balance(&asset_hub_sovereign_account, init_balance);
	});

	//Send XCM message from AssetHub
	AssetHubRococo::execute_with(|| {
		assert_ok!(<AssetHubRococo as AssetHubRococoPallet>::PolkadotXcm::send(
			sudo_origin,
			Box::new(parachain_destination),
			Box::new(xcm)
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

		assert_expected_events!(
			Peregrine,
			vec![
				PeregrineRuntimeEvent::XcmpQueue(cumulus_pallet_xcmp_queue::Event::Fail {
					error: xcm::v3::Error::NoPermission,
					..
				}) => {},
			]
		);
	});

	// No event on the relaychain (message is meant for asset hub)
	Rococo::execute_with(|| {
		assert_eq!(Rococo::events().len(), 0);
	});
}
