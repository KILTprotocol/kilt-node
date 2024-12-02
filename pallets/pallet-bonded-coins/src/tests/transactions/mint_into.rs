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
	traits::{
		fungibles::Inspect,
		tokens::{Fortitude, Preservation},
	},
};
use frame_system::{pallet_prelude::OriginFor, RawOrigin};
use sp_runtime::{assert_eq_error_rate, ArithmeticError, TokenError};

use crate::{
	curves::{polynomial::PolynomialParameters, Curve},
	mock::{runtime::*, *},
	types::{Locks, PoolStatus},
	AccountIdOf, Error,
};

#[test]
fn mint_first_coin() {
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	let curve = get_linear_bonding_curve();

	let initial_collateral = 100u128;
	let amount_to_mint = 1u128;
	// Add one to the expected price to account for rounding
	let expected_price = mocks_curve_get_collateral_at_supply(amount_to_mint) + 1;

	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![(DEFAULT_COLLATERAL_CURRENCY_ID, ACCOUNT_00, initial_collateral)])
		.with_pools(vec![(
			pool_id.clone(),
			generate_pool_details(
				vec![DEFAULT_BONDED_CURRENCY_ID],
				curve,
				true,
				None,
				None,
				Some(DEFAULT_COLLATERAL_CURRENCY_ID),
				None,
				None,
			),
		)])
		.build()
		.execute_with(|| {
			let origin = RawOrigin::Signed(ACCOUNT_00).into();

			assert_ok!(BondingPallet::mint_into(
				origin,
				pool_id.clone(),
				0,
				ACCOUNT_00,
				amount_to_mint,
				expected_price,
				1
			));

			assert_eq_error_rate!(
				Assets::total_balance(DEFAULT_COLLATERAL_CURRENCY_ID, &ACCOUNT_00),
				initial_collateral - expected_price,
				MAX_ERROR.mul_floor(expected_price)
			);

			assert_eq_error_rate!(
				Assets::total_balance(DEFAULT_COLLATERAL_CURRENCY_ID, &pool_id),
				expected_price,
				MAX_ERROR.mul_floor(expected_price)
			);

			assert_eq!(
				Assets::total_balance(DEFAULT_BONDED_CURRENCY_ID, &ACCOUNT_00),
				amount_to_mint
			);
			// Balance should not be frozen
			assert_eq!(
				Assets::reducible_balance(
					DEFAULT_BONDED_CURRENCY_ID,
					&ACCOUNT_00,
					Preservation::Expendable,
					Fortitude::Polite
				),
				amount_to_mint
			);
		})
}

#[test]
fn mint_large_quantity() {
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	let curve = get_linear_bonding_curve();

	let initial_collateral = u128::MAX / 2;
	// Overflows will likely occur when squaring the total supply, which happens on
	// an I75 representation of the balance, scaled down by its denomination
	let denomination = 10u128.pow(DEFAULT_BONDED_DENOMINATION.into());
	let amount_to_mint = (2_u128.pow(74).saturating_mul(denomination) as f64).sqrt() as u128;
	let expected_price = mocks_curve_get_collateral_at_supply(amount_to_mint);

	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![(DEFAULT_COLLATERAL_CURRENCY_ID, ACCOUNT_00, initial_collateral)])
		.with_pools(vec![(
			pool_id.clone(),
			generate_pool_details(
				vec![DEFAULT_BONDED_CURRENCY_ID],
				curve,
				true,
				None,
				None,
				Some(DEFAULT_COLLATERAL_CURRENCY_ID),
				None,
				None,
			),
		)])
		.build()
		.execute_with(|| {
			let origin = RawOrigin::Signed(ACCOUNT_00).into();

			assert_ok!(BondingPallet::mint_into(
				origin,
				pool_id.clone(),
				0,
				ACCOUNT_00,
				amount_to_mint,
				expected_price + MAX_ERROR.mul_ceil(expected_price),
				1
			));

			assert_eq!(
				Assets::total_balance(DEFAULT_BONDED_CURRENCY_ID, &ACCOUNT_00),
				amount_to_mint
			);

			assert_eq_error_rate!(
				Assets::total_balance(DEFAULT_COLLATERAL_CURRENCY_ID, &pool_id),
				expected_price,
				MAX_ERROR.mul_floor(expected_price)
			);
		})
}

#[test]
fn mint_to_other() {
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	let initial_collateral = ONE_HUNDRED_KILT;
	let amount_to_mint = 100_000;

	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT), (ACCOUNT_01, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![(DEFAULT_COLLATERAL_CURRENCY_ID, ACCOUNT_00, initial_collateral)])
		.with_pools(vec![(
			pool_id.clone(),
			generate_pool_details(
				vec![DEFAULT_BONDED_CURRENCY_ID],
				get_linear_bonding_curve(),
				true,
				None,
				None,
				Some(DEFAULT_COLLATERAL_CURRENCY_ID),
				None,
				None,
			),
		)])
		.build()
		.execute_with(|| {
			let holder_origin = RawOrigin::Signed(ACCOUNT_00).into();
			let broke_origin = RawOrigin::Signed(ACCOUNT_01).into();

			// The broke origin should not be able to burn anything, even if the beneficiary
			// is the holder

			assert_err!(
				BondingPallet::mint_into(
					broke_origin,
					pool_id.clone(),
					0,
					ACCOUNT_00,
					amount_to_mint,
					initial_collateral,
					1
				),
				TokenError::FundsUnavailable
			);

			assert_ok!(BondingPallet::mint_into(
				holder_origin,
				pool_id.clone(),
				0,
				ACCOUNT_01,
				amount_to_mint,
				initial_collateral,
				1
			));

			assert_eq!(
				Assets::total_balance(DEFAULT_BONDED_CURRENCY_ID, &ACCOUNT_01),
				amount_to_mint
			);

			assert_ne!(Assets::total_balance(DEFAULT_COLLATERAL_CURRENCY_ID, &pool_id), 0);

			assert!(Assets::total_balance(DEFAULT_COLLATERAL_CURRENCY_ID, &ACCOUNT_00) < initial_collateral);
		})
}

#[test]
fn mint_multiple_currencies() {
	let currencies = vec![DEFAULT_BONDED_CURRENCY_ID, DEFAULT_BONDED_CURRENCY_ID + 1];
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&currencies);

	let curve = get_linear_bonding_curve();

	let amount_to_mint = 10u128.pow(10);
	let expected_price = mocks_curve_get_collateral_at_supply(amount_to_mint);
	let expected_price_second_mint = mocks_curve_get_collateral_at_supply(amount_to_mint * 2) - expected_price;

	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![(
			DEFAULT_COLLATERAL_CURRENCY_ID,
			ACCOUNT_00,
			(expected_price + expected_price_second_mint) * 2,
		)])
		.with_pools(vec![(
			pool_id.clone(),
			generate_pool_details(
				currencies.clone(),
				curve,
				true,
				None,
				None,
				Some(DEFAULT_COLLATERAL_CURRENCY_ID),
				None,
				None,
			),
		)])
		.build()
		.execute_with(|| {
			let origin: OriginFor<Test> = RawOrigin::Signed(ACCOUNT_00).into();

			assert_ok!(BondingPallet::mint_into(
				origin.clone(),
				pool_id.clone(),
				0,
				ACCOUNT_00,
				amount_to_mint,
				expected_price,
				2
			));

			// pool collateral should now hold the expected price
			assert_eq_error_rate!(
				Assets::total_balance(DEFAULT_COLLATERAL_CURRENCY_ID, &pool_id),
				expected_price,
				MAX_ERROR.mul_floor(expected_price)
			);
			// minting account should hold balance of amount_to_mint
			assert_eq!(Assets::total_balance(currencies[0], &ACCOUNT_00), amount_to_mint);

			assert_ok!(BondingPallet::mint_into(
				origin,
				pool_id.clone(),
				1,
				ACCOUNT_00,
				amount_to_mint,
				expected_price_second_mint,
				2
			));
			// pool collateral should now hold the expected price of first and second mint
			assert_eq_error_rate!(
				Assets::total_balance(DEFAULT_COLLATERAL_CURRENCY_ID, &pool_id),
				expected_price + expected_price_second_mint,
				MAX_ERROR.mul_floor(expected_price + expected_price_second_mint)
			);
			// minting account should hold balance of amount_to_mint
			assert_eq!(Assets::total_balance(currencies[1], &ACCOUNT_00), amount_to_mint);
		})
}

#[test]
fn mint_large_supply() {
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	let curve = get_linear_bonding_curve();

	// the bottleneck is the fixed type with a capacity of 2^74 = 1.89 * 10^22; as
	// part of the calculations, the (denomination-scaled) supply is squared.
	// (2^69 / 10^10)^2 = 3.48 * 10^21, which leaves around one magnitude of room
	// for multiplications & additions.
	let initial_supply = 2_u128.pow(69);

	let amount_to_mint = 1u128;
	let expected_price = mocks_curve_get_collateral_at_supply(initial_supply + amount_to_mint)
		- mocks_curve_get_collateral_at_supply(initial_supply);
	let initial_collateral = expected_price * 2;

	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT), (ACCOUNT_01, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![
			(DEFAULT_COLLATERAL_CURRENCY_ID, ACCOUNT_00, initial_collateral),
			(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_01, initial_supply),
		])
		.with_pools(vec![(
			pool_id.clone(),
			generate_pool_details(
				vec![DEFAULT_BONDED_CURRENCY_ID],
				curve,
				true,
				None,
				None,
				Some(DEFAULT_COLLATERAL_CURRENCY_ID),
				None,
				None,
			),
		)])
		.build()
		.execute_with(|| {
			let origin = RawOrigin::Signed(ACCOUNT_00).into();

			assert_ok!(BondingPallet::mint_into(
				origin,
				pool_id.clone(),
				0,
				ACCOUNT_00,
				amount_to_mint,
				// add some collateral to the expected price to account for rounding
				expected_price + MAX_ERROR.mul_ceil(expected_price),
				1
			));

			assert_eq!(
				Assets::total_balance(DEFAULT_BONDED_CURRENCY_ID, &ACCOUNT_00),
				amount_to_mint
			);

			assert_eq_error_rate!(
				Assets::total_balance(DEFAULT_COLLATERAL_CURRENCY_ID, &pool_id),
				expected_price,
				MAX_ERROR.mul_floor(expected_price)
			);
		})
}

#[test]
fn multiple_mints_vs_combined_mint() {
	let currency_id1 = DEFAULT_BONDED_CURRENCY_ID;
	let currency_id2 = 2;
	let pool_id1: AccountIdOf<Test> = calculate_pool_id(&[currency_id1]);
	let pool_id2: AccountIdOf<Test> = calculate_pool_id(&[currency_id2]);

	let curve = get_linear_bonding_curve();

	let amount_to_mint = 11u128.pow(10);
	let account_collateral = 10u128.pow(20);

	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, u128::MAX)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![(DEFAULT_COLLATERAL_CURRENCY_ID, ACCOUNT_00, account_collateral)])
		.with_pools(vec![
			(
				pool_id1.clone(),
				generate_pool_details(
					vec![currency_id1],
					curve.clone(),
					true,
					None,
					None,
					Some(DEFAULT_COLLATERAL_CURRENCY_ID),
					None,
					None,
				),
			),
			(
				pool_id2.clone(),
				generate_pool_details(
					vec![currency_id2],
					curve,
					true,
					None,
					None,
					Some(DEFAULT_COLLATERAL_CURRENCY_ID),
					None,
					None,
				),
			),
		])
		.build()
		.execute_with(|| {
			let origin: OriginFor<Test> = RawOrigin::Signed(ACCOUNT_00).into();

			// pool 1: 1 mint of 10 * amount
			assert_ok!(BondingPallet::mint_into(
				origin.clone(),
				pool_id1.clone(),
				0,
				ACCOUNT_00,
				amount_to_mint * 10,
				account_collateral / 2,
				1
			));

			// pool 2: 10 mints of amount
			for _ in 0..10 {
				assert_ok!(BondingPallet::mint_into(
					origin.clone(),
					pool_id2.clone(),
					0,
					ACCOUNT_00,
					amount_to_mint,
					account_collateral / 2 / 10,
					1
				));
			}

			assert_eq!(
				Assets::total_balance(currency_id1, &ACCOUNT_00),
				Assets::total_balance(currency_id2, &ACCOUNT_00),
			);

			// multiple mints should result into a higher or equal amount of collateral than
			// a single mint
			assert!(
				Assets::total_balance(DEFAULT_COLLATERAL_CURRENCY_ID, &pool_id1)
					<= Assets::total_balance(DEFAULT_COLLATERAL_CURRENCY_ID, &pool_id2),
			);
		})
}

#[test]
fn mint_with_frozen_balance() {
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	let initial_collateral = 10u128.pow(20);
	let amount_to_mint = 10u128.pow(10);

	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![(DEFAULT_COLLATERAL_CURRENCY_ID, ACCOUNT_00, initial_collateral)])
		.with_pools(vec![(
			pool_id.clone(),
			generate_pool_details(
				vec![DEFAULT_BONDED_CURRENCY_ID],
				get_linear_bonding_curve(),
				false, // Non-transferable
				None,
				None,
				Some(DEFAULT_COLLATERAL_CURRENCY_ID),
				None,
				None,
			),
		)])
		.build()
		.execute_with(|| {
			let origin: OriginFor<Test> = RawOrigin::Signed(ACCOUNT_00).into();

			assert_ok!(BondingPallet::mint_into(
				origin.clone(),
				pool_id.clone(),
				0,
				ACCOUNT_00,
				amount_to_mint,
				initial_collateral,
				1
			));

			assert_eq!(
				Assets::total_balance(DEFAULT_BONDED_CURRENCY_ID, &ACCOUNT_00),
				amount_to_mint
			);

			// Check that balance is frozen
			assert_eq!(
				Assets::reducible_balance(
					DEFAULT_BONDED_CURRENCY_ID,
					&ACCOUNT_00,
					Preservation::Expendable,
					Fortitude::Polite
				),
				0
			);

			// check that we can mint again into a frozen account
			assert_ok!(BondingPallet::mint_into(
				origin,
				pool_id,
				0,
				ACCOUNT_00,
				amount_to_mint,
				initial_collateral,
				1
			));

			// Check that balance is still frozen
			assert_eq!(
				Assets::reducible_balance(
					DEFAULT_BONDED_CURRENCY_ID,
					&ACCOUNT_00,
					Preservation::Expendable,
					Fortitude::Polite
				),
				0
			);
		})
}

#[test]
fn mint_on_locked_pool() {
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	let initial_balance = u128::MAX / 3;
	let amount_to_mint = 10u128.pow(10);

	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT), (ACCOUNT_01, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![
			(DEFAULT_COLLATERAL_CURRENCY_ID, ACCOUNT_00, initial_balance),
			(DEFAULT_COLLATERAL_CURRENCY_ID, ACCOUNT_01, initial_balance),
		])
		.with_pools(vec![(
			pool_id.clone(),
			generate_pool_details(
				vec![DEFAULT_BONDED_CURRENCY_ID],
				get_linear_bonding_curve(),
				true,
				Some(PoolStatus::Locked(Locks {
					allow_mint: false,
					..Default::default()
				})),
				Some(ACCOUNT_00), // manager account
				Some(DEFAULT_COLLATERAL_CURRENCY_ID),
				Some(ACCOUNT_00),
				None,
			),
		)])
		.build()
		.execute_with(|| {
			let origin = RawOrigin::Signed(ACCOUNT_01).into();
			let manager_origin = RawOrigin::Signed(ACCOUNT_00).into();

			assert_err!(
				BondingPallet::mint_into(
					origin,
					pool_id.clone(),
					0,
					ACCOUNT_01,
					amount_to_mint,
					initial_balance,
					1
				),
				Error::<Test>::NoPermission
			);

			assert_ok!(BondingPallet::mint_into(
				manager_origin,
				pool_id,
				0,
				ACCOUNT_01,
				amount_to_mint,
				initial_balance,
				1
			));
		});
}

#[test]
fn mint_invalid_pool_id() {
	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT)])
		.build()
		.execute_with(|| {
			let invalid_pool_id = calculate_pool_id(&[999]); // Assume 999 is an invalid currency ID
			let origin = RawOrigin::Signed(ACCOUNT_00).into();

			assert_err!(
				BondingPallet::mint_into(origin, invalid_pool_id, 0, ACCOUNT_00, 1, 2, 1),
				Error::<Test>::PoolUnknown
			);
		})
}

#[test]
fn mint_in_refunding_pool() {
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	ExtBuilder::default()
		.with_pools(vec![(
			pool_id.clone(),
			generate_pool_details(
				vec![DEFAULT_BONDED_CURRENCY_ID],
				get_linear_bonding_curve(),
				true,
				Some(PoolStatus::Refunding),
				None,
				Some(DEFAULT_COLLATERAL_CURRENCY_ID),
				Some(ACCOUNT_00),
				None,
			),
		)])
		.build()
		.execute_with(|| {
			let origin = RawOrigin::Signed(ACCOUNT_00).into();
			assert_err!(
				BondingPallet::mint_into(origin, pool_id, 0, ACCOUNT_00, 1, 2, 1),
				Error::<Test>::PoolNotLive
			);
		});
}

#[test]
fn mint_exceeding_max_collateral_cost() {
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_pools(vec![(
			pool_id.clone(),
			generate_pool_details(
				vec![DEFAULT_BONDED_CURRENCY_ID],
				get_linear_bonding_curve(),
				true,
				None,
				None,
				Some(DEFAULT_COLLATERAL_CURRENCY_ID),
				None,
				None,
			),
		)])
		.build()
		.execute_with(|| {
			let origin = RawOrigin::Signed(ACCOUNT_00).into();

			// Mint operation would cost more than allowed max_cost
			assert_err!(
				BondingPallet::mint_into(origin, pool_id, 0, ACCOUNT_00, 10u128.pow(10), 1, 1),
				Error::<Test>::Slippage
			);
		});
}

#[test]
fn mint_with_zero_cost() {
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	let curve: Curve<Float> = Curve::Polynomial(PolynomialParameters {
		m: Float::from_num(0),
		n: Float::from_num(0),
		o: Float::from_num(0),
	});
	// with an o < 1 a mint of 1 should result in less than 1 collateral returned
	let mint_amount = 1u128;

	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![(DEFAULT_COLLATERAL_CURRENCY_ID, ACCOUNT_00, ONE_HUNDRED_KILT)])
		.with_pools(vec![(
			pool_id.clone(),
			generate_pool_details(
				vec![DEFAULT_BONDED_CURRENCY_ID],
				curve,
				true,
				None,
				None,
				Some(DEFAULT_COLLATERAL_CURRENCY_ID),
				None,
				None,
			),
		)])
		.build()
		.execute_with(|| {
			let origin = RawOrigin::Signed(ACCOUNT_00).into();

			assert_err!(
				BondingPallet::mint_into(origin, pool_id.clone(), 0, ACCOUNT_00, mint_amount, u128::MAX, 1),
				Error::<Test>::ZeroCollateral
			);
		});
}

#[test]
fn mint_invalid_currency_index() {
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	ExtBuilder::default()
		.with_pools(vec![(
			pool_id.clone(),
			generate_pool_details(
				vec![DEFAULT_BONDED_CURRENCY_ID],
				get_linear_bonding_curve(),
				true,
				None,
				None,
				Some(DEFAULT_COLLATERAL_CURRENCY_ID),
				None,
				None,
			),
		)])
		.build()
		.execute_with(|| {
			let origin = RawOrigin::Signed(ACCOUNT_00).into();

			// Index beyond array length
			assert_err!(
				BondingPallet::mint_into(origin, pool_id, 5, ACCOUNT_00, 1, 2, 1),
				Error::<Test>::IndexOutOfBounds
			);
		});
}

#[test]
fn mint_without_collateral() {
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![(DEFAULT_COLLATERAL_CURRENCY_ID, ACCOUNT_01, u128::MAX / 2)])
		.with_pools(vec![(
			pool_id.clone(),
			generate_pool_details(
				vec![DEFAULT_BONDED_CURRENCY_ID],
				get_linear_bonding_curve(),
				true,
				None,
				None,
				Some(DEFAULT_COLLATERAL_CURRENCY_ID),
				None,
				None,
			),
		)])
		.build()
		.execute_with(|| {
			let origin = RawOrigin::Signed(ACCOUNT_00).into();

			assert_err!(
				BondingPallet::mint_into(origin, pool_id, 0, ACCOUNT_00, 10u128.pow(10), u128::MAX, 1),
				TokenError::FundsUnavailable
			);
		});
}

#[test]
fn mint_more_than_fixed_can_represent() {
	// denomination is 10
	// capacity of I75F53 is 1.8+e22
	// -> we need to get beyond 1.8+e32
	// check that we can still burn afterwards
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	let curve = Curve::Polynomial(PolynomialParameters {
		m: Float::from_num(0),
		n: Float::from_num(0),
		o: Float::from_num(0.1),
	});

	let amount_to_mint = 10u128.pow(20);

	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![(
			DEFAULT_COLLATERAL_CURRENCY_ID,
			ACCOUNT_00,
			u128::MAX - amount_to_mint, /* due to a bug in the assets pallet, transfers silently fail if the total
			                             * supply + transferred amount > u128::MAX */
		)])
		.with_pools(vec![(
			pool_id.clone(),
			generate_pool_details(
				vec![DEFAULT_BONDED_CURRENCY_ID],
				curve,
				true,
				None,
				None,
				Some(DEFAULT_COLLATERAL_CURRENCY_ID),
				None,
				None,
			),
		)])
		.build()
		.execute_with(|| {
			let origin: OriginFor<Test> = RawOrigin::Signed(ACCOUNT_00).into();

			// repeatedly mint until we hit balance that cannot be represented
			let mut result = Ok(().into());
			let mut mints = 0;
			while result.is_ok() {
				result = BondingPallet::mint_into(
					origin.clone(),
					pool_id.clone(),
					0,
					ACCOUNT_00,
					amount_to_mint,
					u128::MAX,
					1,
				);
				mints += 1;
			}

			assert!(mints > 2);
			assert_err!(result, ArithmeticError::Overflow);

			assert_eq!(
				Assets::total_balance(DEFAULT_BONDED_CURRENCY_ID, &ACCOUNT_00),
				amount_to_mint * (mints - 1)
			);

			// Make sure the pool is not stuck
			assert_ok!(BondingPallet::burn_into(
				origin,
				pool_id.clone(),
				0,
				ACCOUNT_00,
				amount_to_mint,
				1,
				1
			));
		})
}
