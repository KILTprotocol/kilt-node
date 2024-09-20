use frame_support::{
	assert_ok,
	traits::{
		fungibles::{metadata::Inspect as InspectMetaData, roles::Inspect as InspectRoles},
		ContainsPair,
	},
};
use sp_runtime::BoundedVec;

use crate::{
	mock::{runtime::*, *},
	types::{PoolStatus, TokenMeta},
	NextAssetId, Pools,
};

#[test]
fn test_create_pool() {
	let curve = get_linear_bonding_curve();
	let state = PoolStatus::Active;

	let token_meta = TokenMeta {
		decimals: 10,
		name: BoundedVec::try_from("BTC".as_bytes().to_vec()).expect("creating name should not fail"),
		symbol: BoundedVec::try_from("BTC".as_bytes().to_vec()).expect("creating symbol should not fail"),
		min_balance: 1,
	};

	let currencies = BoundedVec::try_from(vec![token_meta.clone()]).expect("creating currencies should not fail");

	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, UNIT_NATIVE * 10)])
		.with_collateral_asset_id(DEFAULT_COLLATERAL_CURRENCY_ID)
		.with_metadata(vec![(DEFAULT_COLLATERAL_CURRENCY_ID, DEFAULT_COLLATERAL_DENOMINATION)])
		.build()
		.execute_with(|| {
			let current_asset_id = NextAssetId::<Test>::get();
			// Create a pool with the linear bonding curve
			assert_ok!(BondingPallet::create_pool(
				RuntimeOrigin::signed(ACCOUNT_00),
				curve.clone(),
				currencies,
				false,
				state.clone(),
				ACCOUNT_00
			));

			let count_pools = Pools::<Test>::iter().count();

			// we should have one additional pool
			assert_eq!(count_pools, 1);

			let pool_id = calculate_pool_id(vec![current_asset_id]);

			let details = Pools::<Test>::get(&pool_id).expect("Pool should exist");

			// Do some basic checks on the [PoolDetails] struct.
			assert_eq!(details.manager, ACCOUNT_00);
			assert_eq!(details.curve, curve);
			assert_eq!(details.state, state);
			// we have created only one currency
			assert_eq!(details.bonded_currencies.len(), 1);
			assert_eq!(details.bonded_currencies[0], 0);
			assert_eq!(details.transferable, false);

			// The next possible asset id should be 1
			assert_eq!(NextAssetId::<Test>::get(), 1);

			let currency_id = details.bonded_currencies[0];

			// created metadata should match
			let decimals = <Assets as InspectMetaData<AccountId>>::decimals(currency_id);
			let name = <Assets as InspectMetaData<AccountId>>::name(currency_id);
			let symbol = <Assets as InspectMetaData<AccountId>>::symbol(currency_id);

			assert_eq!(decimals, token_meta.decimals);
			assert_eq!(name, token_meta.name.into_inner());
			assert_eq!(symbol, token_meta.symbol.into_inner());

			// check roles of created assets TODO needs to be changed later.
			let owner = <Assets as InspectRoles<AccountId>>::owner(currency_id).expect("Owner should be set");
			let admin = <Assets as InspectRoles<AccountId>>::admin(currency_id).expect("Admin should be set");
			let issuer = <Assets as InspectRoles<AccountId>>::issuer(currency_id).expect("Issuer should be set");
			let freezer = <Assets as InspectRoles<AccountId>>::freezer(currency_id).expect("Freezer should be set");

			assert_eq!(owner, pool_id);
			assert_eq!(admin, pool_id);
			assert_eq!(issuer, pool_id);
			assert_eq!(freezer, pool_id);

			// Supply should be zero
			let total_supply = Assets::total_supply(currency_id);
			assert_eq!(total_supply, 0);

			// check if pool_account is created.
			assert!(Assets::contains(&DEFAULT_COLLATERAL_CURRENCY_ID, &pool_id));
		});
}
