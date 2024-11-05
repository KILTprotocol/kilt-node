use core::ops::Sub;

use frame_support::{assert_err, assert_ok};
use frame_system::{pallet_prelude::OriginFor, RawOrigin};
use pallet_assets::{Error as AssetsPalletErrors, Event as AssetsPalletEvents};
use sp_runtime::BoundedVec;

use crate::{
	mock::{calculate_pool_id, get_linear_bonding_curve_input, runtime::*, ACCOUNT_00, DEFAULT_COLLATERAL_CURRENCY_ID},
	types::PoolStatus,
	Event as BondingPalletEvents, NextAssetId, Pools, TokenMetaOf,
};

#[test]
fn single_currency() {
	let initial_balance = 100_000_000_000_000_000u128;
	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, initial_balance)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.build()
		.execute_with(|| {
			assert_eq!(NextAssetId::<Test>::get(), 0);
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

			assert_eq!(NextAssetId::<Test>::get(), 1);

			assert_eq!(
				Balances::free_balance(ACCOUNT_00),
				initial_balance.sub(BondingPallet::calculate_pool_deposit(1))
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

#[test]
fn multi_currency() {
	let initial_balance = 100_000_000_000_000_000u128;
	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, initial_balance)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.build()
		.execute_with(|| {
			assert_eq!(NextAssetId::<Test>::get(), 0);
			assert_eq!(initial_balance, Balances::free_balance(ACCOUNT_00));

			let origin = RawOrigin::Signed(ACCOUNT_00).into();
			let curve = get_linear_bonding_curve_input();

			let bonded_token = TokenMetaOf::<Test> {
				name: BoundedVec::truncate_from(b"Bitcoin".to_vec()),
				symbol: BoundedVec::truncate_from(b"btc".to_vec()),
				min_balance: 1,
			};

			let bonded_tokens = vec![bonded_token; 3];

			assert_eq!(bonded_tokens.len(), 3);

			assert_ok!(BondingPallet::create_pool(
				origin,
				curve,
				DEFAULT_COLLATERAL_CURRENCY_ID,
				BoundedVec::truncate_from(bonded_tokens),
				10,
				true
			));

			assert_eq!(NextAssetId::<Test>::get(), 3);

			let pool_id = calculate_pool_id(vec![0, 1, 2]);

			let details = Pools::<Test>::get(pool_id).unwrap();

			assert_eq!(BondingPallet::get_currencies_number(&details), 3);
			assert_eq!(details.bonded_currencies, vec![0, 1, 2]);

			assert_eq!(
				Balances::free_balance(ACCOUNT_00),
				initial_balance.sub(BondingPallet::calculate_pool_deposit(3))
			);
		});
}

#[test]
fn can_create_identical_pools() {
	let initial_balance = 100_000_000_000_000_000u128;
	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, initial_balance)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.build()
		.execute_with(|| {
			assert_eq!(NextAssetId::<Test>::get(), 0);

			let origin: OriginFor<Test> = RawOrigin::Signed(ACCOUNT_00).into();
			let curve = get_linear_bonding_curve_input();

			let bonded_token = TokenMetaOf::<Test> {
				name: BoundedVec::truncate_from(b"Bitcoin".to_vec()),
				symbol: BoundedVec::truncate_from(b"btc".to_vec()),
				min_balance: 1,
			};

			assert_ok!(BondingPallet::create_pool(
				origin.clone(),
				curve.clone(),
				DEFAULT_COLLATERAL_CURRENCY_ID,
				BoundedVec::truncate_from(vec![bonded_token.clone()]),
				10,
				true
			));

			assert_ok!(BondingPallet::create_pool(
				origin,
				curve,
				DEFAULT_COLLATERAL_CURRENCY_ID,
				BoundedVec::truncate_from(vec![bonded_token]),
				10,
				true
			));

			assert_eq!(NextAssetId::<Test>::get(), 2);

			let details1 = Pools::<Test>::get(calculate_pool_id(vec![0])).unwrap();
			let details2 = Pools::<Test>::get(calculate_pool_id(vec![1])).unwrap();

			assert_eq!(details1.bonded_currencies, vec![0]);
			assert_eq!(details2.bonded_currencies, vec![1]);
		});
}

#[test]
fn fails_if_collateral_not_exists() {
	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, 100_000_000_000_000_000u128)])
		.build()
		.execute_with(|| {
			let origin = RawOrigin::Signed(ACCOUNT_00).into();
			let curve = get_linear_bonding_curve_input();

			let bonded_token = TokenMetaOf::<Test> {
				name: BoundedVec::truncate_from(b"Bitcoin".to_vec()),
				symbol: BoundedVec::truncate_from(b"btc".to_vec()),
				min_balance: 1,
			};

			assert_err!(
				BondingPallet::create_pool(
					origin,
					curve,
					DEFAULT_COLLATERAL_CURRENCY_ID,
					BoundedVec::truncate_from(vec![bonded_token]),
					10,
					true
				),
				AssetsPalletErrors::<Test>::Unknown
			);
		})
}
