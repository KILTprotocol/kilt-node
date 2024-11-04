use core::ops::Sub;

use frame_support::assert_ok;
use frame_system::RawOrigin;
use pallet_assets::Event as AssetsPalletEvents;
use sp_runtime::BoundedVec;

use crate::{
	mock::{calculate_pool_id, get_linear_bonding_curve_input, runtime::*, ACCOUNT_00, DEFAULT_COLLATERAL_CURRENCY_ID},
	types::PoolStatus,
	Event as BondingPalletEvents, NextAssetId, Pools, TokenMetaOf,
};

#[test]
fn creates_pool() {
	let initial_balance = 100_000_000_000_000_000u128;
	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, initial_balance)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.build()
		.execute_with(|| {
			assert!(NextAssetId::<Test>::get() == 0);
			assert_eq!(initial_balance, Balances::free_balance(ACCOUNT_00));

			let origin = RawOrigin::Signed(ACCOUNT_00).into();
			let curve = get_linear_bonding_curve_input();

			let bonded_token = TokenMetaOf::<Test> {
				name: BoundedVec::truncate_from(b"Bitcoin".to_vec()),
				symbol: BoundedVec::truncate_from(b"btc".to_vec()),
				min_balance: 1,
			};

			assert_ok!(BondingPallet::create_pool(
				origin,
				curve,
				DEFAULT_COLLATERAL_CURRENCY_ID,
				BoundedVec::truncate_from(vec![bonded_token]),
				10,
				true
			));

			let pool_id = calculate_pool_id(vec![0]);

			let details = Pools::<Test>::get(&pool_id).unwrap();

			assert!(details.is_owner(&ACCOUNT_00));
			assert!(details.is_manager(&ACCOUNT_00));
			assert!(details.transferable);
			assert_eq!(details.state, PoolStatus::Locked(Default::default()));
			assert_eq!(details.denomination, 10);
			assert_eq!(details.collateral_id, DEFAULT_COLLATERAL_CURRENCY_ID);
			assert_eq!(details.bonded_currencies.len(), 1);
			assert_eq!(details.bonded_currencies[0], 0);

			assert!(NextAssetId::<Test>::get() == 1);

			assert!(
				Balances::free_balance(ACCOUNT_00) == initial_balance.sub(BondingPallet::calculate_pool_deposit(1))
			);

			System::assert_has_event(BondingPalletEvents::PoolCreated { id: pool_id.clone() }.into());

			// TODO: check events or storage of linked pallets?
			System::assert_has_event(
				AssetsPalletEvents::ForceCreated {
					asset_id: 0,
					owner: pool_id.clone(),
				}
				.into(),
			);

			System::assert_has_event(
				AssetsPalletEvents::MetadataSet {
					asset_id: 0,
					name: b"Bitcoin".into(),
					symbol: b"btc".into(),
					decimals: 10,
					is_frozen: false,
				}
				.into(),
			);

			System::assert_has_event(
				AssetsPalletEvents::Touched {
					asset_id: DEFAULT_COLLATERAL_CURRENCY_ID,
					who: pool_id.clone(),
					depositor: ACCOUNT_00,
				}
				.into(),
			);
		});
}
