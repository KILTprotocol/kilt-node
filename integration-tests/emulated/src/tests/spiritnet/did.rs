use crate::{
	mock::{
		network::MockNetworkPolkadot,
		para_chains::{spiritnet, AssetHubPolkadot, AssetHubPolkadotPallet, Spiritnet},
		relay_chains::Polkadot,
	},
	utils::UNIT,
};
use did::did_details::DidVerificationKey;
use frame_support::traits::fungible::hold::Inspect;
use frame_support::{assert_ok, traits::fungible::Mutate};
use parity_scale_codec::Encode;
use runtime_common::{AccountId, Balance};
use xcm::{v3::WeightLimit, DoubleEncoded, VersionedMultiLocation, VersionedXcm};
use xcm_emulator::{
	assert_expected_events, Here,
	Instruction::{BuyExecution, Transact, WithdrawAsset},
	Junction, Junctions, OriginKind, Parachain, ParentThen, TestExt, Weight, Xcm,
};

fn get_asset_hub_sovereign_account() -> AccountId {
	Spiritnet::sovereign_account_id_of(Spiritnet::sibling_location_of(AssetHubPolkadot::para_id()))
}

fn get_xcm_message(origin_kind: OriginKind, withdraw_balance: Balance) -> VersionedXcm<()> {
	let asset_hub_sovereign_account = get_asset_hub_sovereign_account();

	let call: DoubleEncoded<()> = <Spiritnet as Parachain>::RuntimeCall::Did(did::Call::create_from_account {
		authentication_key: DidVerificationKey::Account(asset_hub_sovereign_account.clone()),
	})
	.encode()
	.into();

	let require_weight_at_most = Weight::from_parts(10_000_600_000_000, 200_000_000_000);

	VersionedXcm::from(Xcm(vec![
		WithdrawAsset((Here, withdraw_balance).into()),
		BuyExecution {
			fees: (Here, withdraw_balance).into(),
			weight_limit: WeightLimit::Unlimited,
		},
		Transact {
			origin_kind,
			require_weight_at_most,
			call,
		},
	]))
}

fn get_destination() -> VersionedMultiLocation {
	ParentThen(Junctions::X1(Junction::Parachain(spiritnet::PARA_ID))).into()
}

#[test]
fn test_did_creation_from_asset_hub_successful() {
	MockNetworkPolkadot::reset();

	let sudo_origin = <AssetHubPolkadot as Parachain>::RuntimeOrigin::root();

	let init_balance = UNIT * 10;
	let withdraw_balance = init_balance / 2;

	let xcm = get_xcm_message(OriginKind::SovereignAccount, withdraw_balance);
	let destination = get_destination();

	let asset_hub_sovereign_account = get_asset_hub_sovereign_account();
	// give the sovereign account of AssetHub some coins.
	Spiritnet::execute_with(|| {
		<spiritnet_runtime::Balances as Mutate<AccountId>>::set_balance(&asset_hub_sovereign_account, init_balance);
	});

	//Send XCM message from AssetHub
	AssetHubPolkadot::execute_with(|| {
		assert_ok!(<AssetHubPolkadot as AssetHubPolkadotPallet>::PolkadotXcm::send(
			sudo_origin,
			Box::new(destination),
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
				SpiritnetRuntimeEvent::Did(did::Event::DidCreated(account, did_identifier)) => {
					account: account == &asset_hub_sovereign_account,
					did_identifier:  did_identifier == &asset_hub_sovereign_account,
				},
				SpiritnetRuntimeEvent::XcmpQueue(cumulus_pallet_xcmp_queue::Event::Success { .. }) => {},
			]
		);

		// we also expect that the sovereignAccount of AssetHub has some coins now
		let balance_on_hold = <<Spiritnet as Parachain>::Balances as Inspect<AccountId>>::balance_on_hold(
			&spiritnet_runtime::RuntimeHoldReason::from(did::HoldReason::Deposit),
			&asset_hub_sovereign_account,
		);

		// since a did is created, 2 of the free balance should now be on hold
		assert_eq!(balance_on_hold, UNIT * 2);
	});

	// No event on the relaychain (message is meant for Spiritnet)
	Polkadot::execute_with(|| {
		assert_eq!(Polkadot::events().len(), 0);
	});
}

#[test]
fn test_did_creation_from_asset_hub_unsuccessful() {
	MockNetworkPolkadot::reset();

	let sudo_origin = <AssetHubPolkadot as Parachain>::RuntimeOrigin::root();

	let asset_hub_sovereign_account = get_asset_hub_sovereign_account();
	let init_balance = UNIT * 10;
	let withdraw_balance = init_balance / 2;

	let destination = get_destination();
	let origin_kind_list = vec![OriginKind::Xcm, OriginKind::Superuser, OriginKind::Native];

	for origin in origin_kind_list {
		let xcm = get_xcm_message(origin, withdraw_balance);

		// give the sovereign account of AssetHub some coins.
		Spiritnet::execute_with(|| {
			<spiritnet_runtime::Balances as Mutate<AccountId>>::set_balance(&asset_hub_sovereign_account, init_balance);
		});

		//Send XCM message from AssetHub
		AssetHubPolkadot::execute_with(|| {
			assert_ok!(<AssetHubPolkadot as AssetHubPolkadotPallet>::PolkadotXcm::send(
				sudo_origin.clone(),
				Box::new(destination.clone()),
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

			// we still expect that the xcm message is send.
			assert_expected_events!(
				Spiritnet,
				vec![
					SpiritnetRuntimeEvent::XcmpQueue(cumulus_pallet_xcmp_queue::Event::Success { .. }) => {},
				]
			);

			// ... but we also expect that the extrinsic will fail since it is no signed runtime origin.
			let is_create_event_present = Spiritnet::events().iter().any(|event| match event {
				SpiritnetRuntimeEvent::Did(did::Event::<spiritnet_runtime::Runtime>::DidCreated(_, _)) => true,
				_ => false,
			});

			assert!(
				!is_create_event_present,
				"Create event for an unsupported origin is found"
			);
		});

		// No event on the relaychain (message is meant for Spiritnet)
		Polkadot::execute_with(|| {
			assert_eq!(Polkadot::events().len(), 0);
		});
	}
}
