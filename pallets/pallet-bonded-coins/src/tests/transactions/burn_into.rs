use frame_support::{
	assert_err, assert_ok,
	traits::{
		fungibles::Inspect,
		tokens::{Fortitude, Preservation},
	},
};
use frame_system::{pallet_prelude::OriginFor, RawOrigin};
use sp_core::bounded_vec;
use sp_runtime::{assert_eq_error_rate, traits::Scale, TokenError};

use crate::{
	mock::{runtime::*, *},
	types::{Locks, PoolStatus},
	AccountIdOf, Error, PoolDetailsOf,
};

// should not be u128::MAX, as a bug in the assets pallet results in transfers
// failing if amount + total supply > u128::MAX
const LARGE_BALANCE: u128 = u128::MAX / 10;

#[test]
fn burn_first_coin() {
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	let amount_to_burn: u128 = 1;
	let expected_price =
		(2 * amount_to_burn.pow(2) + 3 * amount_to_burn) * 10u128.pow(DEFAULT_COLLATERAL_DENOMINATION.into());

	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![
			(DEFAULT_COLLATERAL_CURRENCY_ID, pool_id.clone(), LARGE_BALANCE),
			(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_00, amount_to_burn),
		])
		.with_pools(vec![(
			pool_id.clone(),
			PoolDetailsOf::<Test> {
				curve: get_linear_bonding_curve(),
				manager: None,
				transferable: true,
				bonded_currencies: bounded_vec![DEFAULT_BONDED_CURRENCY_ID],
				state: PoolStatus::Active,
				collateral_id: DEFAULT_COLLATERAL_CURRENCY_ID,
				denomination: 0,
				owner: ACCOUNT_99,
			},
		)])
		.build()
		.execute_with(|| {
			let origin = RawOrigin::Signed(ACCOUNT_00).into();

			assert_ok!(BondingPallet::burn_into(
				origin,
				pool_id,
				0,
				ACCOUNT_00,
				amount_to_burn,
				expected_price - 1, // rounding down may be happening in the conversion to fixed
				1
			));

			assert_eq_error_rate!(
				Assets::total_balance(DEFAULT_COLLATERAL_CURRENCY_ID, &ACCOUNT_00,),
				expected_price, // Collateral returned
				MAX_ERROR.mul_floor(expected_price)
			);

			assert_eq!(
				Assets::total_balance(DEFAULT_BONDED_CURRENCY_ID, &ACCOUNT_00),
				0 // Burnt amount removed
			);
		});
}

#[test]
fn burn_to_other() {
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	let initial_supply = 100_000;
	let amount_to_burn = initial_supply / 2;

	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![
			(DEFAULT_COLLATERAL_CURRENCY_ID, pool_id.clone(), LARGE_BALANCE),
			(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_00, initial_supply),
		])
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
			),
		)])
		.build()
		.execute_with(|| {
			let holder_origin = RawOrigin::Signed(ACCOUNT_00).into();
			let broke_origin = RawOrigin::Signed(ACCOUNT_01).into();

			// The broke origin should not be able to burn anything, even if the beneficiary
			// is the holder
			assert_err!(
				BondingPallet::burn_into(broke_origin, pool_id.clone(), 0, ACCOUNT_00, amount_to_burn, 0, 1),
				TokenError::FundsUnavailable
			);

			// The holder origin should be able to burn their funds and send the collateral
			// to the non-funded account
			assert_ok!(BondingPallet::burn_into(
				holder_origin,
				pool_id.clone(),
				0,
				ACCOUNT_01,
				amount_to_burn,
				0,
				1
			));

			assert_eq!(
				Assets::total_balance(DEFAULT_BONDED_CURRENCY_ID, &ACCOUNT_00),
				initial_supply - amount_to_burn
			);

			assert!(Assets::total_balance(DEFAULT_COLLATERAL_CURRENCY_ID, &ACCOUNT_01) > 0);
		})
}

#[test]
fn burn_large_supply() {
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	let curve = get_linear_bonding_curve();

	let initial_supply = (2_u128.pow(127) as f64).sqrt() as u128; // TODO: what exactly is the theoretical maximum?
	let amount_to_burn = 10u128.pow(10);

	let expected_price = mocks_curve_get_collateral_at_supply(initial_supply)
		- mocks_curve_get_collateral_at_supply(initial_supply - amount_to_burn);

	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![
			(DEFAULT_COLLATERAL_CURRENCY_ID, pool_id.clone(), LARGE_BALANCE),
			(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_00, initial_supply),
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
			),
		)])
		.build()
		.execute_with(|| {
			let origin = RawOrigin::Signed(ACCOUNT_00).into();

			assert_ok!(BondingPallet::burn_into(
				origin,
				pool_id.clone(),
				0,
				ACCOUNT_00,
				amount_to_burn,
				expected_price - 1,
				1
			));

			assert_eq!(
				Assets::total_balance(DEFAULT_BONDED_CURRENCY_ID, &ACCOUNT_00),
				initial_supply - amount_to_burn
			);

			assert_eq_error_rate!(
				Assets::total_balance(DEFAULT_COLLATERAL_CURRENCY_ID, &pool_id),
				LARGE_BALANCE - expected_price,
				MAX_ERROR.mul_floor(expected_price)
			);
		})
}

#[test]
fn burn_large_quantity() {
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	let curve = get_linear_bonding_curve();
	let denomination = 10u128.pow(DEFAULT_BONDED_DENOMINATION.into());
	// Overflows will likely occur when squaring the total supply, which happens on
	// an I75 representation of the balance, scaled down by its denomination
	let amount_to_burn = (2_u128.pow(74).mul(denomination) as f64).sqrt() as u128;
	let expected_price = mocks_curve_get_collateral_at_supply(amount_to_burn);

	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![
			(DEFAULT_COLLATERAL_CURRENCY_ID, pool_id.clone(), LARGE_BALANCE),
			(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_00, amount_to_burn),
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
			),
		)])
		.build()
		.execute_with(|| {
			let origin = RawOrigin::Signed(ACCOUNT_00).into();

			assert_ok!(BondingPallet::burn_into(
				origin,
				pool_id.clone(),
				0,
				ACCOUNT_00,
				amount_to_burn,
				expected_price - 1,
				1
			));

			assert_eq!(Assets::total_balance(DEFAULT_BONDED_CURRENCY_ID, &ACCOUNT_00), 0);

			assert_eq_error_rate!(
				Assets::total_balance(DEFAULT_COLLATERAL_CURRENCY_ID, &pool_id),
				LARGE_BALANCE - expected_price,
				MAX_ERROR.mul_floor(expected_price)
			);
		})
}

#[test]
fn burn_multiple_currencies() {
	let currencies = vec![DEFAULT_BONDED_CURRENCY_ID, DEFAULT_BONDED_CURRENCY_ID + 1];

	let pool_id: AccountIdOf<Test> = calculate_pool_id(&currencies);

	let amount_to_burn = 10_000u128;
	let expected_price_second_burn = mocks_curve_get_collateral_at_supply(amount_to_burn);
	let expected_price_first_burn =
		mocks_curve_get_collateral_at_supply(amount_to_burn * 2) - expected_price_second_burn;

	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![
			(
				DEFAULT_COLLATERAL_CURRENCY_ID,
				pool_id.clone(),
				(expected_price_first_burn + expected_price_second_burn) * 2,
			),
			(currencies[0], ACCOUNT_00, amount_to_burn),
			(currencies[1], ACCOUNT_00, amount_to_burn),
		])
		.with_pools(vec![(
			pool_id.clone(),
			generate_pool_details(
				currencies.clone(),
				get_linear_bonding_curve(),
				true,
				None,
				None,
				Some(DEFAULT_COLLATERAL_CURRENCY_ID),
				None,
			),
		)])
		.build()
		.execute_with(|| {
			let origin: OriginFor<Test> = RawOrigin::Signed(ACCOUNT_00).into();

			assert_ok!(BondingPallet::burn_into(
				origin.clone(),
				pool_id.clone(),
				0,
				ACCOUNT_00,
				amount_to_burn,
				expected_price_first_burn - 1,
				2
			));
			// Burning account should now hold the expected amount of collateral
			assert_eq_error_rate!(
				Assets::total_balance(DEFAULT_COLLATERAL_CURRENCY_ID, &ACCOUNT_00),
				expected_price_first_burn,
				MAX_ERROR.mul_floor(expected_price_first_burn)
			);
			// Bonded token balance should have dropped to 0
			assert_eq!(Assets::total_balance(currencies[0], &ACCOUNT_00), 0);

			assert_ok!(BondingPallet::burn_into(
				origin,
				pool_id.clone(),
				1,
				ACCOUNT_00,
				amount_to_burn,
				expected_price_second_burn - 1,
				2
			));
			// Burning account should now hold the expected amount of collateral from first
			// and second burn
			assert_eq_error_rate!(
				Assets::total_balance(DEFAULT_COLLATERAL_CURRENCY_ID, &ACCOUNT_00),
				expected_price_first_burn + expected_price_second_burn,
				MAX_ERROR.mul_floor(expected_price_second_burn)
			);
			// Bonded token balance should have dropped to 0
			assert_eq!(Assets::total_balance(DEFAULT_BONDED_CURRENCY_ID, &ACCOUNT_00), 0);
		})
}

#[test]
fn multiple_burns_vs_combined_burn() {
	let currency_1 = DEFAULT_BONDED_CURRENCY_ID;
	let currency_2 = DEFAULT_BONDED_CURRENCY_ID + 1;

	let pool_id1: AccountIdOf<Test> = calculate_pool_id(&[currency_1]);
	let pool_id2: AccountIdOf<Test> = calculate_pool_id(&[currency_2]);

	let amount_to_burn = 11u128.pow(10);

	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![
			(DEFAULT_COLLATERAL_CURRENCY_ID, pool_id1.clone(), LARGE_BALANCE),
			(DEFAULT_COLLATERAL_CURRENCY_ID, pool_id2.clone(), LARGE_BALANCE),
			(currency_1, ACCOUNT_00, amount_to_burn * 10),
			(currency_2, ACCOUNT_00, amount_to_burn * 10),
		])
		.with_pools(vec![
			(
				pool_id1.clone(),
				generate_pool_details(
					vec![currency_1],
					get_linear_bonding_curve(),
					true,
					None,
					None,
					Some(DEFAULT_COLLATERAL_CURRENCY_ID),
					None,
				),
			),
			(
				pool_id2.clone(),
				generate_pool_details(
					vec![currency_2],
					get_linear_bonding_curve(),
					true,
					None,
					None,
					Some(DEFAULT_COLLATERAL_CURRENCY_ID),
					None,
				),
			),
		])
		.build()
		.execute_with(|| {
			let origin: OriginFor<Test> = RawOrigin::Signed(ACCOUNT_00).into();

			// pool 1: 1 burn of 10 * amount
			assert_ok!(BondingPallet::burn_into(
				origin.clone(),
				pool_id1.clone(),
				0,
				ACCOUNT_00,
				amount_to_burn * 10,
				0,
				1
			));

			let balance_after_first_burn = Assets::total_balance(DEFAULT_COLLATERAL_CURRENCY_ID, &ACCOUNT_00);

			assert_ne!(balance_after_first_burn, 0u128);

			// pool 2: 10 burns of amount
			for _ in 0..10 {
				assert_ok!(BondingPallet::burn_into(
					origin.clone(),
					pool_id2.clone(),
					0,
					ACCOUNT_00,
					amount_to_burn,
					0,
					1
				));
			}

			let balance_after_second_burn = Assets::total_balance(DEFAULT_COLLATERAL_CURRENCY_ID, &ACCOUNT_00);

			assert_eq!(
				balance_after_second_burn - balance_after_first_burn,
				balance_after_first_burn
			);

			assert_eq!(
				Assets::total_balance(DEFAULT_COLLATERAL_CURRENCY_ID, &pool_id1,),
				Assets::total_balance(DEFAULT_COLLATERAL_CURRENCY_ID, &pool_id2,)
			);
		})
}

#[test]
fn multiple_mints_vs_combined_burn() {
	let currency_id = DEFAULT_BONDED_CURRENCY_ID;
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&[currency_id]);

	let curve = get_linear_bonding_curve();

	let amount_to_mint = 11u128.pow(10);

	let expected_prize = mocks_curve_get_collateral_at_supply(10 * amount_to_mint);

	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, LARGE_BALANCE)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![(DEFAULT_COLLATERAL_CURRENCY_ID, ACCOUNT_00, expected_prize)])
		.with_pools(vec![(
			pool_id.clone(),
			generate_pool_details(
				vec![currency_id],
				curve.clone(),
				true,
				None,
				None,
				Some(DEFAULT_COLLATERAL_CURRENCY_ID),
				None,
			),
		)])
		.build()
		.execute_with(|| {
			let origin: OriginFor<Test> = RawOrigin::Signed(ACCOUNT_00).into();

			// step one: 10 mints of amount
			for _ in 0..10 {
				assert_ok!(BondingPallet::mint_into(
					origin.clone(),
					pool_id.clone(),
					0,
					ACCOUNT_00,
					amount_to_mint,
					u128::MAX,
					1
				));
			}

			assert_eq!(Assets::total_balance(currency_id, &ACCOUNT_00), amount_to_mint * 10,);

			// step 2: 1 burn of 10 * amount
			assert_ok!(BondingPallet::burn_into(
				origin.clone(),
				pool_id.clone(),
				0,
				ACCOUNT_00,
				amount_to_mint * 10,
				0,
				1
			));

			assert_eq!(Assets::total_balance(currency_id, &ACCOUNT_00), 0,);

			assert_eq!(Assets::total_balance(DEFAULT_COLLATERAL_CURRENCY_ID, &pool_id), 0,);
		})
}

#[test]
fn burn_with_frozen_balance() {
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	let amount_to_burn = 10u128.pow(10); // must be smaller than initial_balance / 2
	let initial_balance = 3 * amount_to_burn;

	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![
			(DEFAULT_COLLATERAL_CURRENCY_ID, pool_id.clone(), LARGE_BALANCE),
			(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_00, initial_balance),
		])
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
			),
		)])
		.build()
		.execute_with(|| {
			let origin: OriginFor<Test> = RawOrigin::Signed(ACCOUNT_00).into();

			assert_ok!(BondingPallet::burn_into(
				origin.clone(),
				pool_id.clone(),
				0,
				ACCOUNT_00,
				amount_to_burn,
				1,
				1
			));

			assert_eq!(
				Assets::total_balance(DEFAULT_BONDED_CURRENCY_ID, &ACCOUNT_00),
				initial_balance - amount_to_burn
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

			// check that we can still burn when account is frozen
			assert_ok!(BondingPallet::burn_into(
				origin.clone(),
				pool_id.clone(),
				0,
				ACCOUNT_00,
				amount_to_burn,
				1,
				1
			));

			let account_balance = Assets::total_balance(DEFAULT_BONDED_CURRENCY_ID, &ACCOUNT_00);

			assert_eq!(account_balance, initial_balance - (2 * amount_to_burn));

			// check that we can burn the account's entire holdings
			assert_ok!(BondingPallet::burn_into(
				origin,
				pool_id,
				0,
				ACCOUNT_00,
				account_balance,
				1,
				1
			));
		})
}

#[test]
fn burn_on_locked_pool() {
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	let initial_balance = 10 * 10u128.pow(10);
	let amount_to_burn = 10u128.pow(10);

	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT), (ACCOUNT_01, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![
			(DEFAULT_COLLATERAL_CURRENCY_ID, pool_id.clone(), LARGE_BALANCE),
			(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_00, initial_balance),
			(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_01, initial_balance),
		])
		.with_pools(vec![(
			pool_id.clone(),
			generate_pool_details(
				vec![DEFAULT_BONDED_CURRENCY_ID],
				get_linear_bonding_curve(),
				true,
				Some(PoolStatus::Locked(Locks {
					allow_burn: false,
					..Default::default()
				})),
				Some(ACCOUNT_00), // manager account
				Some(DEFAULT_COLLATERAL_CURRENCY_ID),
				None,
			),
		)])
		.build()
		.execute_with(|| {
			let origin = RawOrigin::Signed(ACCOUNT_01).into();
			let manager_origin = RawOrigin::Signed(ACCOUNT_00).into();

			assert_err!(
				BondingPallet::burn_into(origin, pool_id.clone(), 0, ACCOUNT_01, amount_to_burn, 1, 1),
				Error::<Test>::NoPermission
			);

			assert_ok!(BondingPallet::burn_into(
				manager_origin,
				pool_id,
				0,
				ACCOUNT_01,
				amount_to_burn,
				1,
				1
			));
		});
}

#[test]
fn burn_invalid_pool_id() {
	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT)])
		.build()
		.execute_with(|| {
			let invalid_pool_id = calculate_pool_id(&[999]); // Nonexistent pool
			let origin = RawOrigin::Signed(ACCOUNT_00).into();

			assert_err!(
				BondingPallet::burn_into(origin, invalid_pool_id, 0, ACCOUNT_00, 1, 1, 1),
				Error::<Test>::PoolUnknown
			);
		});
}

#[test]
fn burn_in_refunding_pool() {
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
			),
		)])
		.build()
		.execute_with(|| {
			let origin = RawOrigin::Signed(ACCOUNT_00).into();
			assert_err!(
				BondingPallet::burn_into(origin, pool_id, 0, ACCOUNT_00, 1, 2, 1),
				Error::<Test>::PoolNotLive
			);
		});
}

#[test]
fn burn_not_hitting_minimum() {
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
			),
		)])
		.build()
		.execute_with(|| {
			let origin = RawOrigin::Signed(ACCOUNT_00).into();

			// burn operation would return less than minimum return
			assert_err!(
				BondingPallet::burn_into(origin, pool_id, 0, ACCOUNT_00, 1, 100000, 1),
				Error::<Test>::Slippage
			);
		});
}

#[test]
fn burn_invalid_currency_index() {
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
			),
		)])
		.build()
		.execute_with(|| {
			let origin = RawOrigin::Signed(ACCOUNT_00).into();

			// Index beyond array length
			assert_err!(
				BondingPallet::burn_into(origin, pool_id, 5, ACCOUNT_00, 1, 2, 1),
				Error::<Test>::IndexOutOfBounds
			);
		});
}

#[test]
fn burn_beyond_balance() {
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![
			(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_00, 1), // Only 1 unit available
			(DEFAULT_COLLATERAL_CURRENCY_ID, pool_id.clone(), LARGE_BALANCE),
		])
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
			),
		)])
		.build()
		.execute_with(|| {
			let origin = RawOrigin::Signed(ACCOUNT_00).into();

			assert_err!(
				BondingPallet::burn_into(origin, pool_id, 0, ACCOUNT_00, 2, 0, 1), // Attempt to burn 2 units
				TokenError::FundsUnavailable
			);
		});
}
