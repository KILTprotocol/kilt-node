use crate::{
	mock::{
		network::MockNetworkPolkadot,
		para_chains::{spiritnet, AssetHubPolkadot, AssetHubPolkadotPallet, Spiritnet},
		relay_chains::Polkadot,
	},
	utils::UNIT,
};
use frame_support::traits::fungible::Inspect;
use frame_support::{assert_ok, traits::fungible::Mutate};
use parity_scale_codec::Encode;
use runtime_common::AccountId;
use xcm::{v3::WeightLimit, DoubleEncoded, VersionedMultiLocation, VersionedXcm};
use xcm_emulator::{
	assert_expected_events, Here,
	Instruction::{BuyExecution, RefundSurplus, Transact, WithdrawAsset},
	Junction, Junctions, OriginKind, Parachain, ParentThen, TestExt, Weight, Xcm,
};

#[test]
fn test_did_creation_from_asset_hub() {
	MockNetworkPolkadot::reset();

	// create the sovereign account of AssetHub
	let asset_hub_sovereign_account =
		Spiritnet::sovereign_account_id_of(Spiritnet::sibling_location_of(AssetHubPolkadot::para_id()));

	let call: DoubleEncoded<()> = <Spiritnet as Parachain>::RuntimeCall::Did(did::Call::create_from_account {
		authentication_key: did::did_details::DidVerificationKey::Account(asset_hub_sovereign_account.clone()),
	})
	.encode()
	.into();

	let sudo_origin = <AssetHubPolkadot as Parachain>::RuntimeOrigin::root();

	let parachain_destination: VersionedMultiLocation =
		ParentThen(Junctions::X1(Junction::Parachain(spiritnet::PARA_ID))).into();

	// the Weight parts are copied from logs.
	let require_weight_at_most = Weight::from_parts(10_000_600_000_000, 200_000_000_000);
	let origin_kind = OriginKind::SovereignAccount;

	let init_balance = UNIT * 10;
	let withdraw_balance = init_balance / 2;

	let xcm = VersionedXcm::from(Xcm(vec![
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
		// refund back the withdraw assets.
		RefundSurplus,
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
				SpiritnetRuntimeEvent::XcmpQueue(cumulus_pallet_xcmp_queue::Event::Success { .. }) => {},
				SpiritnetRuntimeEvent::Did(did::Event::DidCreated(_, _)) => {},
			]
		);

		// we also expect that the sovereignAccount of AssetHub has some coins now
		let balance_after_xcm_call: u128 =
			<<Spiritnet as Parachain>::Balances as Inspect<AccountId>>::balance(&asset_hub_sovereign_account).into();

		// since a did is created some of the free balance should now be on hold. Therefore the balance should be less.
		assert!(balance_after_xcm_call < init_balance);
	});

	// No event on the relaychain (message is meant for Spiritnet)
	Polkadot::execute_with(|| {
		assert_eq!(Polkadot::events().len(), 0);
	});
}
