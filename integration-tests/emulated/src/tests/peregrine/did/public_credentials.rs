use crate::{
	mock::{
		network::MockNetworkRococo,
		para_chains::{AssetHubRococo, AssetHubRococoPallet, Peregrine},
		relay_chains::Rococo,
	},
	tests::peregrine::did::utils::{
		construct_xcm_message, create_mock_ctype, create_mock_did, get_asset_hub_sovereign_account,
		get_sibling_destination_peregrine,
	},
};
use frame_support::{assert_ok, traits::fungible::Mutate};
use kilt_asset_dids::AssetDid as AssetIdentifier;
use parity_scale_codec::Encode;
use rococo_runtime::System as RococoSystem;
use runtime_common::{constants::KILT, AccountId, Balance};
use sp_core::H256;
use sp_runtime::BoundedVec;
use xcm::{DoubleEncoded, VersionedXcm};
use xcm_emulator::{assert_expected_events, OriginKind, Parachain, TestExt};

fn get_xcm_message_add_public_credential(
	origin_kind: OriginKind,
	withdraw_balance: Balance,
	ctype_hash: H256,
) -> VersionedXcm<()> {
	let asset_hub_sovereign_account = get_asset_hub_sovereign_account();

	let subject_id = AssetIdentifier::ether_currency();

	let credential = public_credentials::mock::generate_base_public_credential_creation_op::<peregrine_runtime::Runtime>(
		BoundedVec::try_from(subject_id.encode()).unwrap(),
		ctype_hash,
		Default::default(),
	);

	let call: DoubleEncoded<()> = <Peregrine as Parachain>::RuntimeCall::Did(did::Call::dispatch_as {
		did_identifier: asset_hub_sovereign_account,
		call: Box::new(<Peregrine as Parachain>::RuntimeCall::PublicCredentials(
			public_credentials::Call::add {
				credential: Box::new(credential),
			},
		)),
	})
	.encode()
	.into();

	construct_xcm_message(origin_kind, withdraw_balance, call)
}

#[test]
fn test_create_public_credential_from_asset_hub() {
	MockNetworkRococo::reset();

	let sudo_origin = <AssetHubRococo as Parachain>::RuntimeOrigin::root();
	let asset_hub_sovereign_account = get_asset_hub_sovereign_account();
	let ctype_hash_value = H256([0; 32]);

	let init_balance = KILT * 10;

	let xcm_claim_w3n_call =
		get_xcm_message_add_public_credential(OriginKind::SovereignAccount, KILT, ctype_hash_value);

	let destination = get_sibling_destination_peregrine();

	Peregrine::execute_with(|| {
		create_mock_did();
		create_mock_ctype(ctype_hash_value.clone());
		<peregrine_runtime::Balances as Mutate<AccountId>>::set_balance(&asset_hub_sovereign_account, init_balance);
	});

	AssetHubRococo::execute_with(|| {
		assert_ok!(<AssetHubRococo as AssetHubRococoPallet>::PolkadotXcm::send(
			sudo_origin,
			Box::new(destination),
			Box::new(xcm_claim_w3n_call)
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
				PeregrineRuntimeEvent::Did(did::Event::DidCallDispatched(account, result)) => {
					account: account == &asset_hub_sovereign_account,
					result: result.is_ok(),
				},
				PeregrineRuntimeEvent::PublicCredentials(public_credentials::Event::CredentialStored{ subject_id: _, credential_id: _ }) => {

				},
			]
		);
	});

	Rococo::execute_with(|| {
		assert_eq!(RococoSystem::events().len(), 0);
	});
}

#[test]
fn test_create_public_credential_from_asset_hub_unsuccessful() {
	let origin_kind_list = vec![OriginKind::Native, OriginKind::Superuser, OriginKind::Xcm];

	let sudo_origin = <AssetHubRococo as Parachain>::RuntimeOrigin::root();
	let init_balance = KILT * 100;
	let ctype_hash_value = H256([0; 32]);

	let destination = get_sibling_destination_peregrine();
	let asset_hub_sovereign_account = get_asset_hub_sovereign_account();

	for origin_kind in origin_kind_list {
		MockNetworkRococo::reset();

		Peregrine::execute_with(|| {
			create_mock_did();
			create_mock_ctype(ctype_hash_value.clone());
			<peregrine_runtime::Balances as Mutate<AccountId>>::set_balance(&asset_hub_sovereign_account, init_balance);
		});

		let xcm_claim_w3n_call = get_xcm_message_add_public_credential(origin_kind, KILT, ctype_hash_value);

		AssetHubRococo::execute_with(|| {
			assert_ok!(<AssetHubRococo as AssetHubRococoPallet>::PolkadotXcm::send(
				sudo_origin.clone(),
				Box::new(destination.clone()),
				Box::new(xcm_claim_w3n_call.clone())
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

			let is_event_present = Peregrine::events().iter().any(|event| match event {
				PeregrineRuntimeEvent::Did(did::Event::DidCallDispatched(_, _)) => true,
				PeregrineRuntimeEvent::DidLookup(pallet_did_lookup::Event::AssociationEstablished(_, _)) => true,
				_ => false,
			});

			assert!(!is_event_present)
		});

		Rococo::execute_with(|| {
			assert_eq!(RococoSystem::events().len(), 0);
		});
	}
}
