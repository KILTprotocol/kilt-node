use crate::{
	mock::{
		network::MockNetworkRococo,
		para_chains::{peregrine, AssetHubRococo, AssetHubRococoPallet, Peregrine},
		relay_chains::Rococo,
	},
	utils::UNIT,
};
use did::did_details::DidVerificationKey;
use frame_support::traits::fungible::hold::Inspect;
use frame_support::{assert_ok, traits::fungible::Mutate};
use parity_scale_codec::Encode;
use rococo_runtime::System as RococoSystem;
use runtime_common::{AccountId, Balance};
use xcm::{v3::WeightLimit, DoubleEncoded, VersionedMultiLocation, VersionedXcm};
use xcm_emulator::{
	assert_expected_events, Here,
	Instruction::{BuyExecution, Transact, WithdrawAsset},
	Junction, Junctions, OriginKind, Parachain, ParentThen, TestExt, Weight, Xcm,
};

fn get_asset_hub_sovereign_account() -> AccountId {
	Peregrine::sovereign_account_id_of(Peregrine::sibling_location_of(AssetHubRococo::para_id()))
}

fn get_xcm_message(origin_kind: OriginKind, withdraw_balance: Balance) -> VersionedXcm<()> {
	let asset_hub_sovereign_account = get_asset_hub_sovereign_account();

	let call: DoubleEncoded<()> = <Peregrine as Parachain>::RuntimeCall::Did(did::Call::create_from_account {
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
	ParentThen(Junctions::X1(Junction::Parachain(peregrine::PARA_ID))).into()
}

#[test]
fn test_did_creation_from_asset_hub_successful() {
	MockNetworkRococo::reset();

	let sudo_origin = <AssetHubRococo as Parachain>::RuntimeOrigin::root();

	let init_balance = UNIT * 10;
	let withdraw_balance = init_balance / 2;

	let xcm = get_xcm_message(OriginKind::SovereignAccount, withdraw_balance);
	let destination = get_destination();

	let asset_hub_sovereign_account = get_asset_hub_sovereign_account();

	Peregrine::execute_with(|| {
		<peregrine_runtime::Balances as Mutate<AccountId>>::set_balance(&asset_hub_sovereign_account, init_balance);
	});

	AssetHubRococo::execute_with(|| {
		assert_ok!(<AssetHubRococo as AssetHubRococoPallet>::PolkadotXcm::send(
			sudo_origin,
			Box::new(destination.clone()),
			Box::new(xcm.clone())
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
				PeregrineRuntimeEvent::Did(did::Event::DidCreated(account, did_identifier)) => {
					account: account == &asset_hub_sovereign_account,
					did_identifier:  did_identifier == &asset_hub_sovereign_account,
				},
			]
		);

		let balance_on_hold = <<Peregrine as Parachain>::Balances as Inspect<AccountId>>::balance_on_hold(
			&peregrine_runtime::RuntimeHoldReason::from(did::HoldReason::Deposit),
			&asset_hub_sovereign_account,
		);

		assert_eq!(balance_on_hold, UNIT * 2);
	});

	Rococo::execute_with(|| {
		assert_eq!(RococoSystem::events().len(), 0);
	});
}

#[test]
fn test_did_creation_from_asset_hub_unsuccessful() {
	MockNetworkRococo::reset();

	let sudo_origin = <AssetHubRococo as Parachain>::RuntimeOrigin::root();

	let init_balance = UNIT * 10;
	let withdraw_balance = init_balance / 2;

	let asset_hub_sovereign_account = get_asset_hub_sovereign_account();
	let destination = get_destination();

	let origin_kind_list = vec![OriginKind::Xcm, OriginKind::Superuser, OriginKind::Native];

	for origin in origin_kind_list {
		let xcm = get_xcm_message(origin, withdraw_balance);
		// give the sovereign account of AssetHub some coins.
		Peregrine::execute_with(|| {
			<peregrine_runtime::Balances as Mutate<AccountId>>::set_balance(&asset_hub_sovereign_account, init_balance);
		});

		//Send XCM message from AssetHub
		AssetHubRococo::execute_with(|| {
			assert_ok!(<AssetHubRococo as AssetHubRococoPallet>::PolkadotXcm::send(
				sudo_origin.clone(),
				Box::new(destination.clone()),
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

			// we still expect that the xcm message is send.
			assert_expected_events!(
				Peregrine,
				vec![
					PeregrineRuntimeEvent::XcmpQueue(cumulus_pallet_xcmp_queue::Event::Success { .. }) => {},
				]
			);

			// ... but we also expect that the extrinsic will fail since it is no signed runtime origin. So there should no [DidCreated] event
			let is_create_event_present = Peregrine::events().iter().any(|event| match event {
				PeregrineRuntimeEvent::Did(did::Event::<peregrine_runtime::Runtime>::DidCreated(_, _)) => true,
				_ => false,
			});

			assert!(
				!is_create_event_present,
				"Create event for an unsupported origin is found"
			);
		});

		// No event on the relaychain (message is meant for Peregrine)
		Rococo::execute_with(|| {
			assert_eq!(RococoSystem::events().len(), 0);
		});
	}
}
