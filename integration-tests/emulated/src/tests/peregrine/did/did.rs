use crate::{
	mock::{
		network::MockNetworkRococo,
		para_chains::{AssetHubRococo, AssetHubRococoPallet, Peregrine},
		relay_chains::Rococo,
	},
	tests::peregrine::did::utils::{
		construct_xcm_message, get_asset_hub_sovereign_account, get_sibling_destination_peregrine,
	},
};
use did::did_details::DidVerificationKey;
use frame_support::traits::fungible::hold::Inspect;
use frame_support::{assert_ok, traits::fungible::Mutate};
use parity_scale_codec::Encode;
use rococo_runtime::System as RococoSystem;
use runtime_common::{constants::KILT, AccountId, Balance};
use xcm::{DoubleEncoded, VersionedXcm};
use xcm_emulator::{assert_expected_events, OriginKind, Parachain, TestExt};

fn get_xcm_message_create_did(origin_kind: OriginKind, withdraw_balance: Balance) -> VersionedXcm<()> {
	let asset_hub_sovereign_account = get_asset_hub_sovereign_account();

	let call: DoubleEncoded<()> = <Peregrine as Parachain>::RuntimeCall::Did(did::Call::create_from_account {
		authentication_key: DidVerificationKey::Account(asset_hub_sovereign_account.clone()),
	})
	.encode()
	.into();

	construct_xcm_message(origin_kind, withdraw_balance, call)
}

#[test]
fn test_did_creation_from_asset_hub_successful() {
	MockNetworkRococo::reset();

	let sudo_origin = <AssetHubRococo as Parachain>::RuntimeOrigin::root();

	let init_balance = KILT * 10;
	let withdraw_balance = init_balance / 2;

	let xcm = get_xcm_message_create_did(OriginKind::SovereignAccount, withdraw_balance);
	let destination = get_sibling_destination_peregrine();

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

		assert_eq!(
			balance_on_hold,
			<peregrine_runtime::Runtime as did::Config>::BaseDeposit::get()
		);
	});

	Rococo::execute_with(|| {
		assert_eq!(RococoSystem::events().len(), 0);
	});
}

#[test]
fn test_did_creation_from_asset_hub_unsuccessful() {
	let sudo_origin = <AssetHubRococo as Parachain>::RuntimeOrigin::root();

	let init_balance = KILT * 100;
	let withdraw_balance = init_balance / 2;

	let asset_hub_sovereign_account = get_asset_hub_sovereign_account();
	let destination = get_sibling_destination_peregrine();

	let origin_kind_list = vec![OriginKind::Xcm, OriginKind::Superuser, OriginKind::Native];

	for origin in origin_kind_list {
		MockNetworkRococo::reset();

		Peregrine::execute_with(|| {
			<peregrine_runtime::Balances as Mutate<AccountId>>::set_balance(&asset_hub_sovereign_account, init_balance);
		});

		let xcm = get_xcm_message_create_did(origin, withdraw_balance);

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
