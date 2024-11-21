use crate::{
	mock::{runtime::*, *},
	traits::FreezeAccounts,
	types::PoolStatus,
	Error, Event,
};
use frame_support::{assert_err, assert_ok, assert_storage_noop, traits::fungibles::Inspect};
use frame_system::{pallet_prelude::OriginFor, RawOrigin};

#[test]
fn refund_account_works() {
	let pool_details = generate_pool_details(
		vec![DEFAULT_BONDED_CURRENCY_ID],
		get_linear_bonding_curve(),
		true,
		Some(PoolStatus::Refunding),
		Some(ACCOUNT_00),
		Some(DEFAULT_COLLATERAL_CURRENCY_ID),
		Some(ACCOUNT_00),
	);
	let pool_id = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	let total_collateral = 10u128.pow(10);

	ExtBuilder::default()
		.with_pools(vec![(pool_id.clone(), pool_details.clone())])
		.with_native_balances(vec![(ACCOUNT_01, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![
			(DEFAULT_COLLATERAL_CURRENCY_ID, pool_id.clone(), total_collateral),
			(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_01, total_collateral * 10),
		])
		.build()
		.execute_with(|| {
			let origin = RawOrigin::Signed(ACCOUNT_01).into();

			assert_ok!(BondingPallet::refund_account(origin, pool_id.clone(), ACCOUNT_01, 0, 1));

			assert_eq!(
				<Test as crate::Config>::Fungibles::total_balance(DEFAULT_BONDED_CURRENCY_ID, &ACCOUNT_01),
				0
			);

			assert_eq!(
				<Test as crate::Config>::Fungibles::total_balance(DEFAULT_COLLATERAL_CURRENCY_ID, &ACCOUNT_01),
				total_collateral
			);

			// There's only one account, so this should complete the refund
			System::assert_has_event(Event::<Test>::RefundComplete { id: pool_id }.into());
		});
}

#[test]
fn refund_account_works_on_frozen() {
	let pool_details = generate_pool_details(
		vec![DEFAULT_BONDED_CURRENCY_ID],
		get_linear_bonding_curve(),
		false,
		Some(PoolStatus::Refunding),
		Some(ACCOUNT_00),
		Some(DEFAULT_COLLATERAL_CURRENCY_ID),
		Some(ACCOUNT_00),
	);
	let pool_id = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	let total_collateral = 10u128.pow(10);

	ExtBuilder::default()
		.with_pools(vec![(pool_id.clone(), pool_details.clone())])
		.with_native_balances(vec![(ACCOUNT_01, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![
			(DEFAULT_COLLATERAL_CURRENCY_ID, pool_id.clone(), total_collateral),
			(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_01, total_collateral * 10),
		])
		.build()
		.execute_with(|| {
			<<Test as crate::Config>::Fungibles as FreezeAccounts<_, _>>::freeze(
				&DEFAULT_BONDED_CURRENCY_ID,
				&ACCOUNT_01,
			)
			.expect("failed to freeze account prior to testing");

			let origin = RawOrigin::Signed(ACCOUNT_01).into();

			assert_ok!(BondingPallet::refund_account(origin, pool_id.clone(), ACCOUNT_01, 0, 1));

			assert_eq!(
				<Test as crate::Config>::Fungibles::total_balance(DEFAULT_BONDED_CURRENCY_ID, &ACCOUNT_01),
				0
			);

			assert_eq!(
				<Test as crate::Config>::Fungibles::total_balance(DEFAULT_COLLATERAL_CURRENCY_ID, &ACCOUNT_01),
				total_collateral
			);

			// There's only one account, so this should complete the refund
			System::assert_has_event(Event::<Test>::RefundComplete { id: pool_id }.into());
		});
}

#[test]
fn refund_account_works_with_large_supply() {
	let currencies = vec![DEFAULT_BONDED_CURRENCY_ID, DEFAULT_BONDED_CURRENCY_ID + 1];
	let pool_id = calculate_pool_id(&currencies);
	let pool_details = generate_pool_details(
		currencies.clone(),
		get_linear_bonding_curve(),
		true,
		Some(PoolStatus::Refunding),
		Some(ACCOUNT_00),
		Some(DEFAULT_COLLATERAL_CURRENCY_ID),
		Some(ACCOUNT_00),
	);

	let total_collateral = u128::MAX / 2;

	ExtBuilder::default()
		.with_pools(vec![(pool_id.clone(), pool_details.clone())])
		.with_native_balances(vec![(ACCOUNT_01, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![
			(DEFAULT_COLLATERAL_CURRENCY_ID, pool_id.clone(), total_collateral),
			(currencies[0], ACCOUNT_01, u128::MAX / 3 * 2),
			(currencies[1], ACCOUNT_01, u128::MAX / 3 * 2),
		])
		.build()
		.execute_with(|| {
			let origin = RawOrigin::Signed(ACCOUNT_01).into();

			assert_ok!(BondingPallet::refund_account(origin, pool_id.clone(), ACCOUNT_01, 0, 2));

			assert_eq!(
				<Test as crate::Config>::Fungibles::total_balance(currencies[0], &ACCOUNT_01),
				0
			);

			assert_eq!(
				<Test as crate::Config>::Fungibles::total_balance(DEFAULT_COLLATERAL_CURRENCY_ID, &ACCOUNT_01),
				total_collateral / 2
			);

			// At this point we've only refunded one currency, not the other
			assert_eq!(
				events()
					.into_iter()
					.find(|ev| { matches!(ev, Event::<Test>::RefundComplete { id: _ }) }),
				None
			);
		});
}

#[test]
fn balance_is_burnt_even_if_no_collateral_received() {
	let pool_details = generate_pool_details(
		vec![DEFAULT_BONDED_CURRENCY_ID],
		get_linear_bonding_curve(),
		true,
		Some(PoolStatus::Refunding),
		Some(ACCOUNT_00),
		Some(DEFAULT_COLLATERAL_CURRENCY_ID),
		Some(ACCOUNT_00),
	);
	let pool_id = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	let total_collateral = 10u128;

	ExtBuilder::default()
		.with_pools(vec![(pool_id.clone(), pool_details.clone())])
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT), (ACCOUNT_01, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![
			(DEFAULT_COLLATERAL_CURRENCY_ID, pool_id.clone(), total_collateral),
			(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_00, 20),
			(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_01, 1), // 10 / 21 = 0.48 -> no collateral for you
		])
		.build()
		.execute_with(|| {
			let origin = RawOrigin::Signed(ACCOUNT_01).into();

			assert_ok!(BondingPallet::refund_account(origin, pool_id.clone(), ACCOUNT_01, 0, 1));

			assert_eq!(
				<Test as crate::Config>::Fungibles::total_balance(DEFAULT_BONDED_CURRENCY_ID, &ACCOUNT_01),
				0
			);

			assert_eq!(
				<Test as crate::Config>::Fungibles::total_balance(DEFAULT_COLLATERAL_CURRENCY_ID, &ACCOUNT_01),
				0
			);

			// At this point we've only refunded one of two accounts
			assert_eq!(
				events()
					.into_iter()
					.find(|ev| { matches!(ev, Event::<Test>::RefundComplete { id: _ }) }),
				None
			);
		});
}

#[test]
fn refund_account_fails_when_pool_not_refunding() {
	let pool_details = generate_pool_details(
		vec![DEFAULT_BONDED_CURRENCY_ID],
		get_linear_bonding_curve(),
		false,
		Some(PoolStatus::Active),
		Some(ACCOUNT_00),
		None,
		None,
	);
	let pool_id = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_01, ONE_HUNDRED_KILT)])
		.with_pools(vec![(pool_id.clone(), pool_details.clone())])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![
			(DEFAULT_COLLATERAL_CURRENCY_ID, pool_id.clone(), ONE_HUNDRED_KILT),
			(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_01, ONE_HUNDRED_KILT),
		])
		.build()
		.execute_with(|| {
			let origin: OriginFor<Test> = RawOrigin::Signed(ACCOUNT_01).into();

			// Ensure the refund_account call fails due to pool not being in refunding state
			assert_err!(
				BondingPallet::refund_account(origin, pool_id.clone(), ACCOUNT_01, 0, 1),
				Error::<Test>::NotRefunding
			);
		});
}

#[test]
fn refund_account_no_balance() {
	let pool_details = generate_pool_details(
		vec![DEFAULT_BONDED_CURRENCY_ID],
		get_linear_bonding_curve(),
		true,
		Some(PoolStatus::Refunding),
		Some(ACCOUNT_00),
		Some(DEFAULT_COLLATERAL_CURRENCY_ID),
		Some(ACCOUNT_00),
	);
	let pool_id = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	ExtBuilder::default()
		.with_pools(vec![(pool_id.clone(), pool_details.clone())])
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT), (ACCOUNT_01, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![
			(DEFAULT_COLLATERAL_CURRENCY_ID, pool_id.clone(), ONE_HUNDRED_KILT),
			(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_00, ONE_HUNDRED_KILT),
		])
		.build()
		.execute_with(|| {
			let origin = RawOrigin::Signed(ACCOUNT_01).into();

			// Ensure the refund_account call fails when there is no balance to be
			// refunded
			assert_storage_noop!(assert!(BondingPallet::refund_account(
				origin,
				pool_id.clone(),
				ACCOUNT_01,
				0,
				1
			)
			.is_err()));
		});
}
