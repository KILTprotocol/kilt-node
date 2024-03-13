use crate::{
	mock::{
		network::MockNetworkRococo,
		para_chains::{peregrine, AssetHubRococo, AssetHubRococoPallet, Peregrine},
		relay_chains::Rococo,
	},
	utils::UNIT,
};
use frame_support::traits::fungible::Inspect;
use frame_support::{assert_ok, traits::fungible::Mutate};
use parity_scale_codec::Encode;
use rococo_runtime::System as RococoSystem;
use runtime_common::AccountId;
use xcm::{v3::WeightLimit, DoubleEncoded, VersionedMultiLocation, VersionedXcm};
use xcm_emulator::{
	assert_expected_events, Here,
	Instruction::{BuyExecution, Transact, WithdrawAsset},
	Junction, Junctions, OriginKind, Parachain, ParentThen, TestExt, Weight, Xcm,
};

#[test]
fn test_did_creation_from_asset_hub() {
	MockNetworkRococo::reset();

	// create the sovereign account of AssetHub
	let asset_hub_sovereign_account =
		Peregrine::sovereign_account_id_of(Peregrine::sibling_location_of(AssetHubRococo::para_id()));

	let call: DoubleEncoded<()> = <Peregrine as Parachain>::RuntimeCall::Did(did::Call::create_from_account {
		authentication_key: did::did_details::DidVerificationKey::Account(asset_hub_sovereign_account.clone()),
	})
	.encode()
	.into();

	let sudo_origin = <AssetHubRococo as Parachain>::RuntimeOrigin::root();

	let parachain_destination: VersionedMultiLocation =
		ParentThen(Junctions::X1(Junction::Parachain(peregrine::PARA_ID))).into();

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
				PeregrineRuntimeEvent::XcmpQueue(cumulus_pallet_xcmp_queue::Event::Success { .. }) => {},
				PeregrineRuntimeEvent::Did(did::Event::DidCreated(_, _)) => {},
			]
		);

		// we also expect that the sovereignAccount of AssetHub has some coins now
		let balance_after_xcm_call: u128 =
			<<Peregrine as Parachain>::Balances as Inspect<AccountId>>::balance(&asset_hub_sovereign_account);

		// since a did is created some of the free balance should now be on hold. Therefore the balance should be less.
		assert!(balance_after_xcm_call < init_balance);
	});

	// No event on the relaychain (message is meant for Peregrine)
	Rococo::execute_with(|| {
		assert_eq!(RococoSystem::events().len(), 0);
	});
}
