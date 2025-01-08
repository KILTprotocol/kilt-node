// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2024 BOTLabs GmbH

// The KILT Blockchain is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The KILT Blockchain is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

// If you feel like getting in touch with us, you can do so at info@botlabs.org
use frame_support::{
	assert_err, assert_ok,
	traits::fungibles::{
		metadata::Inspect as InspectMetadata, roles::Inspect as InspectRoles, Inspect as InspectFungibles,
	},
};
use frame_system::{pallet_prelude::OriginFor, RawOrigin};
use pallet_assets::Error as AssetsPalletErrors;
use sp_runtime::{bounded_vec, ArithmeticError, BoundedVec};
use sp_std::ops::Sub;

use crate::{
	mock::{runtime::*, *},
	types::{Locks, PoolStatus},
	AccountIdOf, Event as BondingPalletEvents, Pools, TokenMetaOf,
};

#[test]
fn single_currency() {
	let initial_balance = ONE_HUNDRED_KILT;
	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, initial_balance)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.build_and_execute_with_sanity_tests(|| {
			let origin = RawOrigin::Signed(ACCOUNT_00).into();
			let curve = get_linear_bonding_curve_input();

			let bonded_token = TokenMetaOf::<Test> {
				name: BoundedVec::truncate_from(b"Bitcoin".to_vec()),
				symbol: BoundedVec::truncate_from(b"btc".to_vec()),
				min_balance: 1,
			};

			let new_asset_id = NextAssetId::<BondingPallet>::get();

			assert_ok!(BondingPallet::create_pool(
				origin,
				curve,
				DEFAULT_COLLATERAL_CURRENCY_ID,
				bounded_vec![bonded_token],
				DEFAULT_BONDED_DENOMINATION,
				true,
				1,
			));

			let pool_id: AccountIdOf<Test> = calculate_pool_id(&[new_asset_id]);

			let details = Pools::<Test>::get(&pool_id).unwrap();

			assert!(details.is_owner(&ACCOUNT_00));
			assert!(details.is_manager(&ACCOUNT_00));
			assert!(details.transferable);
			assert_eq!(
				details.state,
				PoolStatus::Locked(Locks {
					allow_mint: false,
					allow_burn: false,
				})
			);
			assert_eq!(details.denomination, DEFAULT_BONDED_DENOMINATION);
			assert_eq!(details.collateral, DEFAULT_COLLATERAL_CURRENCY_ID);
			assert_eq!(details.bonded_currencies, vec![new_asset_id]);

			// collateral is id 0, new bonded currency should be 1, next is 2
			assert_eq!(NextAssetId::<BondingPallet>::get(), new_asset_id + 1);

			assert_eq!(
				Balances::free_balance(ACCOUNT_00),
				initial_balance.sub(BondingPallet::calculate_pool_deposit(1))
			);

			System::assert_has_event(BondingPalletEvents::PoolCreated { id: pool_id.clone() }.into());

			// Check creation
			assert!(Assets::asset_exists(new_asset_id));
			// Check team
			assert_eq!(Assets::owner(new_asset_id), Some(pool_id.clone()));
			assert_eq!(Assets::admin(new_asset_id), Some(pool_id.clone()));
			assert_eq!(Assets::issuer(new_asset_id), Some(pool_id.clone()));
			assert_eq!(Assets::freezer(new_asset_id), Some(pool_id));
			// Check metadata
			assert_eq!(Assets::decimals(new_asset_id), DEFAULT_BONDED_DENOMINATION);
			assert_eq!(Assets::name(new_asset_id), b"Bitcoin");
			assert_eq!(Assets::symbol(new_asset_id), b"btc");
		});
}

#[test]
fn multi_currency() {
	let initial_balance = ONE_HUNDRED_KILT;
	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, initial_balance)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.build_and_execute_with_sanity_tests(|| {
			let origin = RawOrigin::Signed(ACCOUNT_00).into();
			let curve = get_linear_bonding_curve_input();

			let bonded_token = TokenMetaOf::<Test> {
				name: BoundedVec::truncate_from(b"Bitcoin".to_vec()),
				symbol: BoundedVec::truncate_from(b"btc".to_vec()),
				min_balance: 1,
			};

			let bonded_tokens = bounded_vec![bonded_token; 3];

			let next_asset_id = NextAssetId::<BondingPallet>::get();

			assert_ok!(BondingPallet::create_pool(
				origin,
				curve,
				DEFAULT_COLLATERAL_CURRENCY_ID,
				bonded_tokens,
				DEFAULT_BONDED_DENOMINATION,
				true,
				1
			));

			assert_eq!(NextAssetId::<BondingPallet>::get(), next_asset_id + 3);

			let new_assets = Vec::from_iter(next_asset_id..next_asset_id + 3);
			let pool_id: AccountIdOf<Test> = calculate_pool_id(&new_assets);

			let details = Pools::<Test>::get(pool_id.clone()).unwrap();

			assert_eq!(BondingPallet::get_currencies_number(&details), 3);
			assert_eq!(details.bonded_currencies, new_assets);

			assert_eq!(
				Balances::free_balance(ACCOUNT_00),
				initial_balance.sub(BondingPallet::calculate_pool_deposit(3))
			);

			for new_asset_id in new_assets {
				assert!(Assets::asset_exists(new_asset_id));
				assert_eq!(Assets::owner(new_asset_id), Some(pool_id.clone()));
			}
		});
}

#[test]
fn can_create_identical_pools() {
	let initial_balance = ONE_HUNDRED_KILT;
	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, initial_balance)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.build_and_execute_with_sanity_tests(|| {
			let origin: OriginFor<Test> = RawOrigin::Signed(ACCOUNT_00).into();
			let curve = get_linear_bonding_curve_input();

			let bonded_token = TokenMetaOf::<Test> {
				name: BoundedVec::truncate_from(b"Bitcoin".to_vec()),
				symbol: BoundedVec::truncate_from(b"btc".to_vec()),
				min_balance: 1,
			};

			let next_asset_id = NextAssetId::<BondingPallet>::get();

			assert_ok!(BondingPallet::create_pool(
				origin.clone(),
				curve.clone(),
				DEFAULT_COLLATERAL_CURRENCY_ID,
				bounded_vec![bonded_token.clone()],
				DEFAULT_BONDED_DENOMINATION,
				true,
				1
			));

			assert_ok!(BondingPallet::create_pool(
				origin,
				curve,
				DEFAULT_COLLATERAL_CURRENCY_ID,
				bounded_vec![bonded_token],
				DEFAULT_BONDED_DENOMINATION,
				true,
				1
			));

			assert_eq!(NextAssetId::<BondingPallet>::get(), next_asset_id + 2);

			let details1 =
				Pools::<Test>::get(calculate_pool_id::<AssetId, AccountIdOf<Test>>(&[next_asset_id])).unwrap();
			let details2 =
				Pools::<Test>::get(calculate_pool_id::<AssetId, AccountIdOf<Test>>(&[next_asset_id + 1])).unwrap();

			assert_eq!(details1.bonded_currencies, vec![next_asset_id]);
			assert_eq!(details2.bonded_currencies, vec![next_asset_id + 1]);

			assert!(Assets::asset_exists(next_asset_id));
			assert!(Assets::asset_exists(next_asset_id + 1));
		});
}

#[test]
fn fails_if_collateral_not_exists() {
	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT)])
		.build_and_execute_with_sanity_tests(|| {
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
					true,
					1
				),
				AssetsPalletErrors::<Test>::Unknown
			);
		})
}

#[test]
fn cannot_create_circular_pool() {
	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT)])
		.build_and_execute_with_sanity_tests(|| {
			let origin = RawOrigin::Signed(ACCOUNT_00).into();
			let curve = get_linear_bonding_curve_input();

			let bonded_token = TokenMetaOf::<Test> {
				name: BoundedVec::truncate_from(b"Bitcoin".to_vec()),
				symbol: BoundedVec::truncate_from(b"btc".to_vec()),
				min_balance: 1,
			};

			let next_asset_id = NextAssetId::<BondingPallet>::get();

			assert_err!(
				BondingPallet::create_pool(
					origin,
					curve,
					// try specifying the id of the currency to be created as collateral
					next_asset_id,
					bounded_vec![bonded_token],
					DEFAULT_BONDED_DENOMINATION,
					true,
					1
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
		.build_and_execute_with_sanity_tests(|| {
			NextAssetId::<BondingPallet>::set(u32::MAX);

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
					true,
					1
				),
				ArithmeticError::Overflow
			);
		});
}
