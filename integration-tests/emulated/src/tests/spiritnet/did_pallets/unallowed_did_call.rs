use crate::{
	mock::{
		network::MockNetworkPolkadot,
		para_chains::{AssetHubPolkadot, AssetHubPolkadotPallet, Spiritnet},
		relay_chains::Polkadot,
	},
	tests::spiritnet::did_pallets::utils::{
		construct_xcm_message, create_mock_did, get_asset_hub_sovereign_account, get_sibling_destination_spiritnet,
	},
};
use frame_support::{assert_ok, traits::fungible::Mutate};
use parity_scale_codec::Encode;
use runtime_common::{constants::KILT, AccountId, Balance};
use xcm::{DoubleEncoded, VersionedXcm};
use xcm_emulator::{assert_expected_events, OriginKind, Parachain, TestExt};

fn get_xcm_message_system_remark(origin_kind: OriginKind, withdraw_balance: Balance) -> VersionedXcm<()> {
	let asset_hub_sovereign_account = get_asset_hub_sovereign_account();

	let call: DoubleEncoded<()> = <Spiritnet as Parachain>::RuntimeCall::Did(did::Call::dispatch_as {
		did_identifier: asset_hub_sovereign_account,
		call: Box::new(<Spiritnet as Parachain>::RuntimeCall::System(
			frame_system::Call::remark { remark: vec![] },
		)),
	})
	.encode()
	.into();

	construct_xcm_message(origin_kind, withdraw_balance, call)
}

fn get_xcm_message_recursion(origin_kind: OriginKind, withdraw_balance: Balance) -> VersionedXcm<()> {
	let asset_hub_sovereign_account = get_asset_hub_sovereign_account();

	let call: DoubleEncoded<()> = <Spiritnet as Parachain>::RuntimeCall::Did(did::Call::dispatch_as {
		did_identifier: asset_hub_sovereign_account.clone(),
		call: Box::new(<Spiritnet as Parachain>::RuntimeCall::Did(did::Call::dispatch_as {
			did_identifier: asset_hub_sovereign_account,
			call: Box::new(<Spiritnet as Parachain>::RuntimeCall::System(
				frame_system::Call::remark { remark: vec![] },
			)),
		})),
	})
	.encode()
	.into();

	construct_xcm_message(origin_kind, withdraw_balance, call)
}

#[test]
fn test_not_allowed_did_call() {
	let origin_kind_list = vec![
		OriginKind::Native,
		OriginKind::Superuser,
		OriginKind::Xcm,
		OriginKind::SovereignAccount,
	];

	let sudo_origin = <AssetHubPolkadot as Parachain>::RuntimeOrigin::root();
	let init_balance = KILT * 100;

	let destination = get_sibling_destination_spiritnet();
	let asset_hub_sovereign_account = get_asset_hub_sovereign_account();

	for origin_kind in origin_kind_list {
		MockNetworkPolkadot::reset();

		Spiritnet::execute_with(|| {
			create_mock_did();
			<spiritnet_runtime::Balances as Mutate<AccountId>>::set_balance(&asset_hub_sovereign_account, init_balance);
		});

		let xcm_claim_w3n_call = get_xcm_message_system_remark(origin_kind, KILT);

		AssetHubPolkadot::execute_with(|| {
			assert_ok!(<AssetHubPolkadot as AssetHubPolkadotPallet>::PolkadotXcm::send(
				sudo_origin.clone(),
				Box::new(destination.clone()),
				Box::new(xcm_claim_w3n_call.clone())
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

			// All calls should have [NoPermission] error
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

		Polkadot::execute_with(|| {
			assert_eq!(Polkadot::events().len(), 0);
		});
	}
}

#[test]
fn test_recursion_did_call() {
	let origin_kind_list = vec![
		OriginKind::Native,
		OriginKind::Superuser,
		OriginKind::Xcm,
		OriginKind::SovereignAccount,
	];

	let sudo_origin = <AssetHubPolkadot as Parachain>::RuntimeOrigin::root();
	let init_balance = KILT * 100;

	let destination = get_sibling_destination_spiritnet();
	let asset_hub_sovereign_account = get_asset_hub_sovereign_account();

	for origin_kind in origin_kind_list {
		MockNetworkPolkadot::reset();

		Spiritnet::execute_with(|| {
			create_mock_did();
			<spiritnet_runtime::Balances as Mutate<AccountId>>::set_balance(&asset_hub_sovereign_account, init_balance);
		});

		let xcm_claim_w3n_call = get_xcm_message_recursion(origin_kind, KILT);

		AssetHubPolkadot::execute_with(|| {
			assert_ok!(<AssetHubPolkadot as AssetHubPolkadotPallet>::PolkadotXcm::send(
				sudo_origin.clone(),
				Box::new(destination.clone()),
				Box::new(xcm_claim_w3n_call.clone())
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

			// All calls should have [NoPermission] error
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

		Polkadot::execute_with(|| {
			assert_eq!(Polkadot::events().len(), 0);
		});
	}
}
