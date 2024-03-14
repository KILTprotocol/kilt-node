use crate::{
	mock::{
		network::MockNetworkRococo,
		para_chains::{AssetHubRococo, Peregrine},
		relay_chains::Rococo,
	},
	utils::get_account_id_from_seed,
};
use frame_support::assert_noop;
use frame_support::dispatch::RawOrigin;
use integration_tests_common::{asset_hub_polkadot, polkadot::ED, ALICE, BOB};
use peregrine_runtime::PolkadotXcm as PeregrineXcm;
use sp_core::sr25519;
use xcm::v3::WeightLimit;
use xcm_emulator::{Here, Junction, Junctions, ParentThen, TestExt, X1};

#[test]
fn test_teleport_asset_from_regular_peregrine_account_to_asset_hub() {
	MockNetworkRococo::reset();

	let alice_account_id = get_account_id_from_seed::<sr25519::Public>(ALICE);
	let bob_account_id = get_account_id_from_seed::<sr25519::Public>(BOB);

	Peregrine::execute_with(|| {
		assert_noop!(
			PeregrineXcm::limited_teleport_assets(
				RawOrigin::Signed(alice_account_id.clone()).into(),
				Box::new(ParentThen(Junctions::X1(Junction::Parachain(asset_hub_polkadot::PARA_ID))).into()),
				Box::new(
					X1(Junction::AccountId32 {
						network: None,
						id: bob_account_id.into()
					})
					.into()
				),
				Box::new((Here, 1000 * ED).into()),
				0,
				WeightLimit::Unlimited,
			),
			pallet_xcm::Error::<peregrine_runtime::Runtime>::Filtered
		);
	});
	// No event on the relaychain Message is for AssetHub
	Rococo::execute_with(|| {
		assert_eq!(Rococo::events().len(), 0);
	});
	// AssetHub should not receive any message, since the message is filtered out.
	AssetHubRococo::execute_with(|| {
		assert_eq!(AssetHubRococo::events().len(), 0);
	});
}
