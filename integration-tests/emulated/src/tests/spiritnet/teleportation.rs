use crate::{
	mock::{
		network::MockNetworkPolkadot,
		para_chains::{AssetHubPolkadot, Spiritnet},
		relay_chains::Polkadot,
	},
	utils::get_account_id_from_seed,
};
use frame_support::assert_noop;
use frame_support::dispatch::RawOrigin;
use integration_tests_common::{asset_hub_polkadot, polkadot::ED, ALICE, BOB};
use sp_core::sr25519;
use spiritnet_runtime::PolkadotXcm as SpiritnetXcm;
use xcm::v3::WeightLimit;
use xcm_emulator::{Here, Junction, Junctions, ParentThen, TestExt, X1};

#[test]
fn test_teleport_asset_from_regular_spiritnet_account_to_asset_hub() {
	MockNetworkPolkadot::reset();

	let alice_account_id = get_account_id_from_seed::<sr25519::Public>(ALICE);
	let bob_account_id = get_account_id_from_seed::<sr25519::Public>(BOB);

	Spiritnet::execute_with(|| {
		assert_noop!(
			SpiritnetXcm::limited_teleport_assets(
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
			pallet_xcm::Error::<spiritnet_runtime::Runtime>::Filtered
		);
	});
	// No event on the relaychain Message is for AssetHub
	Polkadot::execute_with(|| {
		assert_eq!(Polkadot::events().len(), 0);
	});
	// Fails on AssetHub since spiritnet is not a trusted registrar.
	AssetHubPolkadot::execute_with(|| {
		assert_eq!(AssetHubPolkadot::events().len(), 0);
	});
}
