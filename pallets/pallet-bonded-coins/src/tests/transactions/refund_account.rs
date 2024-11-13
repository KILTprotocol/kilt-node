use crate::{
	mock::{runtime::*, *},
	types::PoolStatus,
	Error,
};
use frame_support::{assert_err, assert_ok, traits::fungibles::Inspect};
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
		.with_native_balances(vec![(ACCOUNT_01, u128::MAX)])
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
		.with_native_balances(vec![(ACCOUNT_01, u128::MAX)])
		.with_pools(vec![(pool_id.clone(), pool_details.clone())])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![
			(DEFAULT_COLLATERAL_CURRENCY_ID, pool_id.clone(), u128::MAX),
			(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_01, u128::MAX),
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
		.with_native_balances(vec![(ACCOUNT_00, u128::MAX / 2), (ACCOUNT_01, u128::MAX / 2)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![
			(DEFAULT_COLLATERAL_CURRENCY_ID, pool_id.clone(), u128::MAX / 10),
			(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_00, u128::MAX / 10),
		])
		.build()
		.execute_with(|| {
			let origin = RawOrigin::Signed(ACCOUNT_01).into();

			// Ensure the refund_account call acts as no-op when there is no balance to be
			// refunded
			assert_ok!(BondingPallet::refund_account(origin, pool_id.clone(), ACCOUNT_01, 0, 1),);
		});
}
