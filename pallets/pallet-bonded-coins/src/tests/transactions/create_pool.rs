use frame_support::{
	assert_err, assert_ok,
	traits::fungibles::{
		metadata::Inspect as InspectMetadata, roles::Inspect as InspectRoles, Inspect as InspectFungibles,
	},
};
use frame_system::{pallet_prelude::OriginFor, RawOrigin};
use pallet_assets::Error as AssetsPalletErrors;
use sp_core::bounded_vec;
use sp_runtime::{ArithmeticError, BoundedVec};
use sp_std::ops::Sub;

use crate::{
	mock::{runtime::*, *},
	types::{Locks, PoolStatus},
	Event as BondingPalletEvents, NextAssetId, Pools, TokenMetaOf,
};

#[test]
fn single_currency() {
	let initial_balance = ONE_HUNDRED_KILT;
	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, initial_balance)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.build()
		.execute_with(|| {
			let origin = RawOrigin::Signed(ACCOUNT_00).into();
			let curve = get_linear_bonding_curve_input();

			let bonded_token = TokenMetaOf::<Test> {
				name: BoundedVec::truncate_from(b"Bitcoin".to_vec()),
				symbol: BoundedVec::truncate_from(b"btc".to_vec()),
				min_balance: 1,
			};

			let new_asset_id = NextAssetId::<Test>::get();

			assert_ok!(BondingPallet::create_pool(
				origin,
				curve,
				DEFAULT_COLLATERAL_CURRENCY_ID,
				bounded_vec![bonded_token],
				DEFAULT_BONDED_DENOMINATION,
				true
			));

			let pool_id = calculate_pool_id(&[new_asset_id]);

			let details = Pools::<Test>::get(&pool_id).unwrap();

			assert!(details.is_owner(&ACCOUNT_00));
			assert!(details.is_manager(&ACCOUNT_00));
			assert!(details.transferable);
			assert_eq!(
				details.state,
				PoolStatus::Locked(Locks {
					allow_mint: false,
					allow_burn: false,
					allow_swap: false
				})
			);
			assert_eq!(details.denomination, DEFAULT_BONDED_DENOMINATION);
			assert_eq!(details.collateral_id, DEFAULT_COLLATERAL_CURRENCY_ID);
			assert_eq!(details.bonded_currencies, vec![new_asset_id]);

			// collateral is id 0, new bonded currency should be 1, next is 2
			assert_eq!(NextAssetId::<Test>::get(), new_asset_id + 1);

			assert_eq!(
				Balances::free_balance(ACCOUNT_00),
				initial_balance.sub(BondingPallet::calculate_pool_deposit(1))
			);

			System::assert_has_event(BondingPalletEvents::PoolCreated { id: pool_id.clone() }.into());

			// Check creation
			assert!(<Test as crate::Config>::Fungibles::asset_exists(new_asset_id));
			// Check team
			assert_eq!(
				<Test as crate::Config>::Fungibles::owner(new_asset_id),
				Some(pool_id.clone())
			);
			assert_eq!(
				<Test as crate::Config>::Fungibles::admin(new_asset_id),
				Some(pool_id.clone())
			);
			assert_eq!(
				<Test as crate::Config>::Fungibles::issuer(new_asset_id),
				Some(pool_id.clone())
			);
			assert_eq!(
				<Test as crate::Config>::Fungibles::freezer(new_asset_id),
				Some(pool_id.clone())
			);
			// Check metadata
			assert_eq!(
				<Test as crate::Config>::Fungibles::decimals(new_asset_id),
				DEFAULT_BONDED_DENOMINATION
			);
			assert_eq!(<Test as crate::Config>::Fungibles::name(new_asset_id), b"Bitcoin");
			assert_eq!(<Test as crate::Config>::Fungibles::symbol(new_asset_id), b"btc");
		});
}

#[test]
fn multi_currency() {
	let initial_balance = ONE_HUNDRED_KILT;
	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, initial_balance)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.build()
		.execute_with(|| {
			let origin = RawOrigin::Signed(ACCOUNT_00).into();
			let curve = get_linear_bonding_curve_input();

			let bonded_token = TokenMetaOf::<Test> {
				name: BoundedVec::truncate_from(b"Bitcoin".to_vec()),
				symbol: BoundedVec::truncate_from(b"btc".to_vec()),
				min_balance: 1,
			};

			let bonded_tokens = bounded_vec![bonded_token; 3];

			let next_asset_id = NextAssetId::<Test>::get();

			assert_ok!(BondingPallet::create_pool(
				origin,
				curve,
				DEFAULT_COLLATERAL_CURRENCY_ID,
				bonded_tokens,
				DEFAULT_BONDED_DENOMINATION,
				true
			));

			assert_eq!(NextAssetId::<Test>::get(), next_asset_id + 3);

			let new_assets = Vec::from_iter(next_asset_id..next_asset_id + 3);
			let pool_id = calculate_pool_id(&new_assets);

			let details = Pools::<Test>::get(pool_id.clone()).unwrap();

			assert_eq!(BondingPallet::get_currencies_number(&details), 3);
			assert_eq!(details.bonded_currencies, new_assets);

			assert_eq!(
				Balances::free_balance(ACCOUNT_00),
				initial_balance.sub(BondingPallet::calculate_pool_deposit(3))
			);

			for new_asset_id in new_assets {
				assert!(<Test as crate::Config>::Fungibles::asset_exists(new_asset_id));
				assert_eq!(
					<Test as crate::Config>::Fungibles::owner(new_asset_id),
					Some(pool_id.clone())
				);
			}
		});
}

#[test]
fn can_create_identical_pools() {
	let initial_balance = ONE_HUNDRED_KILT;
	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, initial_balance)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.build()
		.execute_with(|| {
			let origin: OriginFor<Test> = RawOrigin::Signed(ACCOUNT_00).into();
			let curve = get_linear_bonding_curve_input();

			let bonded_token = TokenMetaOf::<Test> {
				name: BoundedVec::truncate_from(b"Bitcoin".to_vec()),
				symbol: BoundedVec::truncate_from(b"btc".to_vec()),
				min_balance: 1,
			};

			let next_asset_id = NextAssetId::<Test>::get();

			assert_ok!(BondingPallet::create_pool(
				origin.clone(),
				curve.clone(),
				DEFAULT_COLLATERAL_CURRENCY_ID,
				bounded_vec![bonded_token.clone()],
				DEFAULT_BONDED_DENOMINATION,
				true
			));

			assert_ok!(BondingPallet::create_pool(
				origin,
				curve,
				DEFAULT_COLLATERAL_CURRENCY_ID,
				bounded_vec![bonded_token],
				DEFAULT_BONDED_DENOMINATION,
				true
			));

			assert_eq!(NextAssetId::<Test>::get(), next_asset_id + 2);

			let details1 = Pools::<Test>::get(calculate_pool_id(&[next_asset_id])).unwrap();
			let details2 = Pools::<Test>::get(calculate_pool_id(&[next_asset_id + 1])).unwrap();

			assert_eq!(details1.bonded_currencies, vec![next_asset_id]);
			assert_eq!(details2.bonded_currencies, vec![next_asset_id + 1]);

			assert!(<Test as crate::Config>::Fungibles::asset_exists(next_asset_id));
			assert!(<Test as crate::Config>::Fungibles::asset_exists(next_asset_id + 1));
		});
}

#[test]
fn fails_if_collateral_not_exists() {
	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT)])
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
					100,
					bounded_vec![bonded_token],
					DEFAULT_BONDED_DENOMINATION,
					true
				),
				AssetsPalletErrors::<Test>::Unknown
			);
		})
}

#[test]
fn cannot_create_circular_pool() {
	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT)])
		.build()
		.execute_with(|| {
			let origin = RawOrigin::Signed(ACCOUNT_00).into();
			let curve = get_linear_bonding_curve_input();

			let bonded_token = TokenMetaOf::<Test> {
				name: BoundedVec::truncate_from(b"Bitcoin".to_vec()),
				symbol: BoundedVec::truncate_from(b"btc".to_vec()),
				min_balance: 1,
			};

			let next_asset_id = NextAssetId::<Test>::get();

			assert_err!(
				BondingPallet::create_pool(
					origin,
					curve,
					// try specifying the id of the currency to be created as collateral
					next_asset_id,
					bounded_vec![bonded_token],
					DEFAULT_BONDED_DENOMINATION,
					true
				),
				AssetsPalletErrors::<Test>::Unknown
			);
		})
}

#[test]
fn handles_asset_id_overflow() {
	let initial_balance = ONE_HUNDRED_KILT;
	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, initial_balance)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.build()
		.execute_with(|| {
			NextAssetId::<Test>::set(u32::MAX);

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
					bounded_vec![bonded_token; 2],
					DEFAULT_BONDED_DENOMINATION,
					true
				),
				ArithmeticError::Overflow
			);
		});
}
