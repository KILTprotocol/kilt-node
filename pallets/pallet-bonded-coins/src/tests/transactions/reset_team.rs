use frame_support::{assert_err, assert_ok, traits::fungibles::roles::Inspect};
use frame_system::RawOrigin;

use crate::{
	mock::{runtime::*, *},
	types::{PoolManagingTeam, PoolStatus},
	Error as BondingPalletErrors,
};

#[test]
fn resets_team() {
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
		.with_pools(vec![(pool_id.clone(), pool_details.clone())])
		.build()
		.execute_with(|| {
			let manager_origin = RawOrigin::Signed(ACCOUNT_00).into();

			assert_ok!(BondingPallet::reset_team(
				manager_origin,
				pool_id.clone(),
				PoolManagingTeam {
					admin: ACCOUNT_00,
					freezer: ACCOUNT_01,
				},
				0
			));

			assert_eq!(Assets::admin(DEFAULT_BONDED_CURRENCY_ID), Some(ACCOUNT_00));
			assert_eq!(Assets::freezer(DEFAULT_BONDED_CURRENCY_ID), Some(ACCOUNT_01));
			assert_eq!(Assets::owner(DEFAULT_BONDED_CURRENCY_ID), Some(pool_id.clone()));
			assert_eq!(Assets::issuer(DEFAULT_BONDED_CURRENCY_ID), Some(pool_id));
		})
}

#[test]
fn does_not_change_team_when_not_live() {
	let pool_details = generate_pool_details(
		vec![DEFAULT_BONDED_CURRENCY_ID],
		get_linear_bonding_curve(),
		false,
		Some(PoolStatus::Refunding),
		Some(ACCOUNT_00),
		None,
		None,
	);
	let pool_id = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	ExtBuilder::default()
		.with_pools(vec![(pool_id.clone(), pool_details.clone())])
		.build()
		.execute_with(|| {
			let manager_origin = RawOrigin::Signed(ACCOUNT_00).into();

			assert_err!(
				BondingPallet::reset_team(
					manager_origin,
					pool_id.clone(),
					PoolManagingTeam {
						admin: ACCOUNT_00,
						freezer: ACCOUNT_00,
					},
					0
				),
				BondingPalletErrors::<Test>::PoolNotLive
			);

			assert_eq!(Assets::admin(DEFAULT_BONDED_CURRENCY_ID), Some(pool_id));
		})
}

#[test]
fn only_manager_can_change_team() {
	let curve = get_linear_bonding_curve();

	let manager = AccountId::new([10u8; 32]);
	let pool_details = generate_pool_details(
		vec![DEFAULT_BONDED_CURRENCY_ID],
		curve,
		false,
		Some(PoolStatus::Active),
		Some(manager.clone()),
		None,
		Some(ACCOUNT_00),
	);
	let pool_id = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);
	ExtBuilder::default()
		.with_pools(vec![(pool_id.clone(), pool_details.clone())])
		.build()
		.execute_with(|| {
			let owner_origin = RawOrigin::Signed(ACCOUNT_00).into();
			let other_origin = RawOrigin::Signed(ACCOUNT_01).into();

			assert_err!(
				BondingPallet::reset_team(
					owner_origin,
					pool_id.clone(),
					PoolManagingTeam {
						admin: ACCOUNT_00,
						freezer: ACCOUNT_00,
					},
					0
				),
				BondingPalletErrors::<Test>::NoPermission
			);

			assert_err!(
				BondingPallet::reset_team(
					other_origin,
					pool_id.clone(),
					PoolManagingTeam {
						admin: ACCOUNT_00,
						freezer: ACCOUNT_00,
					},
					0
				),
				BondingPalletErrors::<Test>::NoPermission
			);

			assert_eq!(Assets::admin(DEFAULT_BONDED_CURRENCY_ID), Some(pool_id));
		})
}

#[test]
fn handles_currency_idx_out_of_bounds() {
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
		.with_pools(vec![(pool_id.clone(), pool_details.clone())])
		.build()
		.execute_with(|| {
			let manager_origin = RawOrigin::Signed(ACCOUNT_00).into();

			assert_err!(
				BondingPallet::reset_team(
					manager_origin,
					pool_id.clone(),
					PoolManagingTeam {
						admin: ACCOUNT_00,
						freezer: ACCOUNT_00,
					},
					2
				),
				BondingPalletErrors::<Test>::IndexOutOfBounds
			);
		})
}
