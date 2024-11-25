use crate::{
	mock::{runtime::*, *},
	types::PoolStatus,
	Error, Event, Pools,
};
use frame_support::{
	assert_err, assert_ok,
	traits::{
		fungible::InspectHold,
		fungibles::{Destroy, Inspect},
	},
};
use frame_system::{pallet_prelude::OriginFor, RawOrigin};

#[test]
fn anyone_can_call_finish_destroy() {
	let pool_details = generate_pool_details(
		vec![DEFAULT_BONDED_CURRENCY_ID],
		get_linear_bonding_curve(),
		true,
		Some(PoolStatus::Destroying),
		None,
		Some(DEFAULT_COLLATERAL_CURRENCY_ID),
		Some(ACCOUNT_01), // owner must hold native asset so we can reserve deposit
	);
	let pool_id = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	ExtBuilder::default()
		.with_pools(vec![(pool_id.clone(), pool_details.clone())])
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT), (ACCOUNT_01, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.build()
		.execute_with(|| {
			// Assets need to be in destroying state if pool is in destroying state
			<Assets as Destroy<_>>::start_destroy(DEFAULT_BONDED_CURRENCY_ID, None)
				.expect("failed to set up test state: asset cannot be set to destroying");

			let origin = RawOrigin::Signed(ACCOUNT_00).into();

			assert_ok!(BondingPallet::finish_destroy(origin, pool_id.clone(), 1));

			// Verify that the pool state entry has been removed
			assert_eq!(Pools::<Test>::get(&pool_id), None);

			// Verify the expected event has been deposited
			System::assert_has_event(Event::Destroyed { id: pool_id }.into());

			// Verify that the bonded asset class has been destroyed
			assert!(!Assets::asset_exists(DEFAULT_BONDED_CURRENCY_ID));

			// Verify that deposit has been freed
			assert_eq!(
				<Test as crate::Config>::DepositCurrency::total_balance_on_hold(&ACCOUNT_01),
				0
			);
		});
}

#[test]
fn owner_receives_collateral() {
	let pool_details = generate_pool_details(
		vec![DEFAULT_BONDED_CURRENCY_ID],
		get_linear_bonding_curve(),
		true,
		Some(PoolStatus::Destroying),
		None,
		Some(DEFAULT_COLLATERAL_CURRENCY_ID),
		Some(ACCOUNT_01), // owner must hold native asset so we can reserve deposit
	);
	let pool_id = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	let remaining_collateral: u128 = 100_000;

	ExtBuilder::default()
		.with_pools(vec![(pool_id.clone(), pool_details.clone())])
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT), (ACCOUNT_01, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![(
			DEFAULT_COLLATERAL_CURRENCY_ID,
			pool_id.clone(),
			remaining_collateral,
		)])
		.build()
		.execute_with(|| {
			// Assets need to be in destroying state if pool is in destroying state
			<Assets as Destroy<_>>::start_destroy(DEFAULT_BONDED_CURRENCY_ID, None)
				.expect("failed to set up test state: asset cannot be set to destroying");

			let origin = RawOrigin::Signed(ACCOUNT_00).into();

			assert_ok!(BondingPallet::finish_destroy(origin, pool_id.clone(), 1));

			// Verify that the remaining collateral has been moved to the owner
			assert_eq!(
				Assets::total_balance(DEFAULT_COLLATERAL_CURRENCY_ID, &ACCOUNT_01),
				remaining_collateral
			);
		});
}

#[test]
fn works_if_asset_is_gone() {
	let pool_details = generate_pool_details(
		vec![DEFAULT_BONDED_CURRENCY_ID],
		get_linear_bonding_curve(),
		true,
		Some(PoolStatus::Destroying),
		None,
		Some(DEFAULT_COLLATERAL_CURRENCY_ID),
		Some(ACCOUNT_01), // owner must hold native asset so we can reserve deposit
	);
	let pool_id = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	ExtBuilder::default()
		.with_pools(vec![(pool_id.clone(), pool_details.clone())])
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT), (ACCOUNT_01, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.build()
		.execute_with(|| {
			// Assets need to be in destroying state if pool is in destroying state
			<Assets as Destroy<_>>::start_destroy(DEFAULT_BONDED_CURRENCY_ID, None)
				.expect("failed to set up test state: asset cannot be set to destroying");
			// Assets need to be in destroying state if pool is in destroying state
			<Assets as Destroy<_>>::finish_destroy(DEFAULT_BONDED_CURRENCY_ID)
				.expect("failed to set up test state: asset cannot be set to destroying");

			let origin = RawOrigin::Signed(ACCOUNT_00).into();

			assert_ok!(BondingPallet::finish_destroy(origin, pool_id.clone(), 1));

			// Verify that the pool state entry has been removed
			assert_eq!(Pools::<Test>::get(&pool_id), None);

			// Verify the expected event has been deposited
			System::assert_has_event(Event::Destroyed { id: pool_id }.into());
		});
}

#[test]
fn fails_on_incorrect_state() {
	let pool_details = generate_pool_details(
		vec![DEFAULT_BONDED_CURRENCY_ID],
		get_linear_bonding_curve(),
		true,
		Some(PoolStatus::Active),
		None,
		Some(DEFAULT_COLLATERAL_CURRENCY_ID),
		Some(ACCOUNT_00), // owner must hold native asset so we can reserve deposit
	);
	let pool_id = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);
	let origin: OriginFor<Test> = RawOrigin::Signed(ACCOUNT_00).into();

	ExtBuilder::default()
		.with_pools(vec![(pool_id.clone(), pool_details.clone())])
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.build()
		.execute_with(|| {
			// Assets need to be in destroying state if pool is in destroying state
			<Assets as Destroy<_>>::start_destroy(DEFAULT_BONDED_CURRENCY_ID, None)
				.expect("failed to set up test state: asset cannot be set to destroying");

			assert_err!(
				BondingPallet::finish_destroy(origin.clone(), pool_id.clone(), 1),
				Error::<Test>::LivePool
			);

			Pools::<Test>::mutate(&pool_id, |details| {
				details.as_mut().unwrap().state.start_refund();
			});

			assert_err!(
				BondingPallet::finish_destroy(origin.clone(), pool_id.clone(), 1),
				Error::<Test>::LivePool
			);

			Pools::<Test>::mutate(&pool_id, |details| {
				details.as_mut().unwrap().state.start_destroy();
			});

			assert_ok!(BondingPallet::finish_destroy(origin, pool_id.clone(), 1));
		});
}

#[test]
fn fails_if_assets_cannot_be_destroyed() {
	let pool_details = generate_pool_details(
		vec![DEFAULT_BONDED_CURRENCY_ID],
		get_linear_bonding_curve(),
		true,
		Some(PoolStatus::Destroying),
		None,
		Some(DEFAULT_COLLATERAL_CURRENCY_ID),
		Some(ACCOUNT_00), // owner must hold native asset so we can reserve deposit
	);
	let pool_id = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);
	let origin: OriginFor<Test> = RawOrigin::Signed(ACCOUNT_00).into();

	ExtBuilder::default()
		.with_pools(vec![(pool_id.clone(), pool_details.clone())])
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![(DEFAULT_BONDED_CURRENCY_ID, ACCOUNT_00, 100_000)])
		.build()
		.execute_with(|| {
			// Fails because asset is not in destroying state
			assert!(BondingPallet::finish_destroy(origin.clone(), pool_id.clone(), 1).is_err());

			// Assets need to be in destroying state if pool is in destroying state
			<Assets as Destroy<_>>::start_destroy(DEFAULT_BONDED_CURRENCY_ID, None)
				.expect("failed to set asset to destroying state");

			// Fails because asset has active accounts attached to it
			assert!(BondingPallet::finish_destroy(origin.clone(), pool_id.clone(), 1).is_err());

			<Assets as Destroy<_>>::destroy_accounts(DEFAULT_BONDED_CURRENCY_ID, 100)
				.expect("failed to destroy accounts");

			// now we should be good to go
			assert_ok!(BondingPallet::finish_destroy(origin, pool_id.clone(), 1));

			// Verify that the bonded asset class has been destroyed
			assert!(!Assets::asset_exists(DEFAULT_BONDED_CURRENCY_ID));
		});
}

#[test]
fn fails_on_invalid_arguments() {
	let currencies = vec![
		DEFAULT_BONDED_CURRENCY_ID,
		DEFAULT_BONDED_CURRENCY_ID + 1,
		DEFAULT_BONDED_CURRENCY_ID + 2,
	];
	let pool_details = generate_pool_details(
		currencies.clone(),
		get_linear_bonding_curve(),
		false,
		Some(PoolStatus::Destroying),
		None,
		None,
		Some(ACCOUNT_00),
	);
	let pool_id = calculate_pool_id(&currencies);

	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT)])
		.with_pools(vec![(pool_id.clone(), pool_details.clone())])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		// .with_bonded_balance(vec![(DEFAULT_COLLATERAL_CURRENCY_ID, pool_id.clone(), 100_000)])
		.build()
		.execute_with(|| {
			// All assets need to be in destroying state if pool is in destroying state
			currencies.into_iter().for_each(|id| {
				<Assets as Destroy<_>>::start_destroy(id, None)
					.expect("failed to set up test state: asset cannot be set to destroying");
			});

			let origin: OriginFor<Test> = RawOrigin::Signed(ACCOUNT_00).into();

			assert_err!(
				BondingPallet::finish_destroy(origin.clone(), pool_id.clone(), 1),
				Error::<Test>::CurrencyCount
			);

			assert_err!(
				BondingPallet::finish_destroy(origin.clone(), pool_id.clone(), 2),
				Error::<Test>::CurrencyCount
			);

			assert_ok!(BondingPallet::finish_destroy(origin.clone(), pool_id.clone(), 3));

			// Pool no longer exists
			assert_err!(
				BondingPallet::finish_destroy(origin, pool_id.clone(), 3),
				Error::<Test>::PoolUnknown
			);
		});
}
