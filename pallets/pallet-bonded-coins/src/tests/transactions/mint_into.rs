use core::u128;

use frame_support::{
	assert_err, assert_ok,
	traits::{
		fungibles::Inspect,
		tokens::{Fortitude, Preservation},
	},
};
use frame_system::{pallet_prelude::OriginFor, RawOrigin};
use sp_runtime::{ArithmeticError, TokenError};

use crate::{
	curves::{polynomial::PolynomialParameters, Curve},
	mock::{runtime::*, *},
	types::{Locks, PoolStatus},
	Error,
};

fn collateral_at_supply(supply: u128) -> u128 {
	supply.pow(2) + 3 * supply
}

#[test]
fn mint_first_coin() {
	let pool_id = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	let curve = get_linear_bonding_curve();

	let initial_balance = 100u128;
	let amount_to_mint = 1u128;
	let expected_price = collateral_at_supply(amount_to_mint);

	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![(DEFAULT_COLLATERAL_CURRENCY_ID, ACCOUNT_00, initial_balance)])
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

			assert_ok!(BondingPallet::mint_into(
				origin,
				pool_id.clone(),
				0,
				ACCOUNT_00,
				amount_to_mint,
				expected_price,
				1
			));

			assert_eq!(
				<Test as crate::Config>::Fungibles::total_balance(DEFAULT_COLLATERAL_CURRENCY_ID, &ACCOUNT_00),
				initial_balance - expected_price
			);

			assert_eq!(
				<Test as crate::Config>::Fungibles::total_balance(DEFAULT_COLLATERAL_CURRENCY_ID, &pool_id),
				expected_price
			);

			assert_eq!(
				<Test as crate::Config>::Fungibles::total_balance(DEFAULT_BONDED_CURRENCY_ID, &ACCOUNT_00),
				amount_to_mint
			);
			// Balance should not be frozen
			assert_eq!(
				<Test as crate::Config>::Fungibles::reducible_balance(
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
	let pool_id = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	let curve = get_linear_bonding_curve();

	let initial_balance = u128::MAX;

	let amount_to_mint = (2_u128.pow(127) as f64).sqrt() as u128; // TODO: what exactly is the theoretical maximum?
	let expected_price = collateral_at_supply(amount_to_mint);

	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![(DEFAULT_COLLATERAL_CURRENCY_ID, ACCOUNT_00, initial_balance)])
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

			assert_ok!(BondingPallet::mint_into(
				origin,
				pool_id.clone(),
				0,
				ACCOUNT_00,
				amount_to_mint,
				expected_price,
				1
			));

			assert_eq!(
				<Test as crate::Config>::Fungibles::total_balance(DEFAULT_BONDED_CURRENCY_ID, &ACCOUNT_00),
				amount_to_mint
			);

			assert_eq!(
				<Test as crate::Config>::Fungibles::total_balance(DEFAULT_COLLATERAL_CURRENCY_ID, &pool_id),
				expected_price,
			);
		})
}

#[test]
fn mint_multiple_currencies() {
	let currencies = vec![DEFAULT_BONDED_CURRENCY_ID, DEFAULT_BONDED_CURRENCY_ID + 1];
	let pool_id = calculate_pool_id(&currencies);

	let curve = get_linear_bonding_curve();

	let amount_to_mint = 10u128.pow(10);
	let expected_price = collateral_at_supply(amount_to_mint);
	let expected_price_second_mint = collateral_at_supply(amount_to_mint * 2) - expected_price;

	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![(DEFAULT_COLLATERAL_CURRENCY_ID, ACCOUNT_00, u128::MAX)])
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
			assert_eq!(
				<Test as crate::Config>::Fungibles::total_balance(DEFAULT_COLLATERAL_CURRENCY_ID, &pool_id),
				expected_price
			);
			// minting account should hold balance of amount_to_mint
			assert_eq!(
				<Test as crate::Config>::Fungibles::total_balance(currencies[0], &ACCOUNT_00),
				amount_to_mint
			);

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
			assert_eq!(
				<Test as crate::Config>::Fungibles::total_balance(DEFAULT_COLLATERAL_CURRENCY_ID, &pool_id),
				expected_price + expected_price_second_mint
			);
			// minting account should hold balance of amount_to_mint
			assert_eq!(
				<Test as crate::Config>::Fungibles::total_balance(currencies[1], &ACCOUNT_00),
				amount_to_mint
			);
		})
}

#[test]
fn mint_large_supply() {
	let pool_id = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	let curve = get_linear_bonding_curve();

	let initial_balance = u128::MAX;
	let initial_supply = (2_u128.pow(127) as f64).sqrt() as u128; // TODO: what exactly is the theoretical maximum?

	let amount_to_mint = 1;
	let expected_price = collateral_at_supply(initial_supply + amount_to_mint) - collateral_at_supply(initial_supply);

	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![(DEFAULT_COLLATERAL_CURRENCY_ID, ACCOUNT_00, initial_balance)])
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

			assert_ok!(BondingPallet::mint_into(
				origin,
				pool_id.clone(),
				0,
				ACCOUNT_00,
				amount_to_mint,
				expected_price,
				1
			));

			assert_eq!(
				<Test as crate::Config>::Fungibles::total_balance(DEFAULT_BONDED_CURRENCY_ID, &ACCOUNT_00),
				amount_to_mint
			);

			assert_eq!(
				<Test as crate::Config>::Fungibles::total_balance(DEFAULT_COLLATERAL_CURRENCY_ID, &pool_id),
				expected_price,
			);
		})
}

#[test]
fn multiple_mints_vs_combined_mint() {
	let currency_id1 = DEFAULT_BONDED_CURRENCY_ID;
	let currency_id2 = 2;
	let pool_id1 = calculate_pool_id(&[currency_id1]);
	let pool_id2 = calculate_pool_id(&[currency_id2]);

	let curve = get_linear_bonding_curve();

	let amount_to_mint = 11u128.pow(10);

	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, u128::MAX)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![(DEFAULT_COLLATERAL_CURRENCY_ID, ACCOUNT_00, u128::MAX)])
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
				u128::MAX,
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
					u128::MAX,
					1
				));
			}

			assert_eq!(
				<Test as crate::Config>::Fungibles::total_balance(currency_id1, &ACCOUNT_00),
				<Test as crate::Config>::Fungibles::total_balance(currency_id2, &ACCOUNT_00),
			);

			assert_eq!(
				<Test as crate::Config>::Fungibles::total_balance(DEFAULT_COLLATERAL_CURRENCY_ID, &pool_id1),
				<Test as crate::Config>::Fungibles::total_balance(DEFAULT_COLLATERAL_CURRENCY_ID, &pool_id2),
			);
		})
}

#[test]
fn mint_with_frozen_balance() {
	let pool_id = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	let initial_balance = u128::MAX;
	let amount_to_mint = 10u128.pow(10);

	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![(DEFAULT_COLLATERAL_CURRENCY_ID, ACCOUNT_00, initial_balance)])
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

			assert_ok!(BondingPallet::mint_into(
				origin.clone(),
				pool_id.clone(),
				0,
				ACCOUNT_00,
				amount_to_mint,
				initial_balance,
				1
			));

			assert_eq!(
				<Test as crate::Config>::Fungibles::total_balance(DEFAULT_BONDED_CURRENCY_ID, &ACCOUNT_00),
				amount_to_mint
			);

			// Check that balance is frozen
			assert_eq!(
				<Test as crate::Config>::Fungibles::reducible_balance(
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
				initial_balance,
				1
			));
		})
}

#[test]
fn mint_on_locked_pool() {
	let pool_id = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	let initial_balance = u128::MAX;
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
	let pool_id = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

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
				BondingPallet::mint_into(origin, pool_id, 0, ACCOUNT_00, 1, 2, 1),
				Error::<Test>::PoolNotLive
			);
		});
}

#[test]
fn mint_exceeding_max_collateral_cost() {
	let pool_id = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

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

			// Mint operation would cost more than allowed max_cost
			assert_err!(
				BondingPallet::mint_into(origin, pool_id, 0, ACCOUNT_00, 10u128.pow(10), 1, 1),
				Error::<Test>::Slippage
			);
		});
}

#[test]
fn mint_invalid_currency_index() {
	let pool_id = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

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
				BondingPallet::mint_into(origin, pool_id, 5, ACCOUNT_00, 1, 2, 1),
				Error::<Test>::IndexOutOfBounds
			);
		});
}

#[test]
fn mint_without_collateral() {
	let pool_id = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

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
	let pool_id = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	let curve = Curve::Polynomial(PolynomialParameters {
		m: Float::from_num(0),
		n: Float::from_num(0),
		o: Float::from_num(0.1),
	});

	let amount_to_mint = 10u128.pow(20);

	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![(DEFAULT_COLLATERAL_CURRENCY_ID, ACCOUNT_00, u128::MAX)])
		// .with_bonded_balance(vec![(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_00, 10u128.pow(32))])
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
			let origin: OriginFor<Test> = RawOrigin::Signed(ACCOUNT_00).into();

			// repeatedly mint until we hit balance that cannot be represented
			let mut result = Ok(());
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
				<Test as crate::Config>::Fungibles::total_balance(DEFAULT_BONDED_CURRENCY_ID, &ACCOUNT_00),
				amount_to_mint * (mints - 1)
			);

			// Make sure the pool is not stuck
			assert_ok!(BondingPallet::burn_into(
				origin.clone(),
				pool_id.clone(),
				0,
				ACCOUNT_00,
				amount_to_mint,
				1,
				1
			));
		})
}
