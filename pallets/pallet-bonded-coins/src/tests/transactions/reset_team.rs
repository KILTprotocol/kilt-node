use frame_support::{assert_err, assert_ok};
use frame_system::RawOrigin;
use pallet_assets::Event as AssetsPalletEvents;

use crate::{
	mock::{runtime::*, *},
	types::{PoolManagingTeam, PoolStatus},
	Error as BondingPalletErrors, Pools,
};

#[test]
fn resets_team() {
	let pool_details = generate_pool_details(
		vec![0],
		get_linear_bonding_curve(),
		false,
		Some(PoolStatus::Active),
		Some(ACCOUNT_00),
		None,
		None,
	);
	let pool_id = calculate_pool_id(&[0]);

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

			System::assert_has_event(
				AssetsPalletEvents::<Test>::TeamChanged {
					asset_id: 0,
					issuer: pool_id,
					admin: ACCOUNT_00,
					freezer: ACCOUNT_01,
				}
				.into(),
			);
		})
}

#[test]
fn does_not_change_team_when_not_live() {
	let pool_details = generate_pool_details(
		vec![0],
		get_linear_bonding_curve(),
		false,
		Some(PoolStatus::Refunding),
		Some(ACCOUNT_00),
		None,
		None,
	);
	let pool_id = calculate_pool_id(&[0]);

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
		})
}

#[test]
fn only_manager_can_change_team() {
	let curve = get_linear_bonding_curve();

	let manager = AccountId::new([10u8; 32]);
	let pool_details = generate_pool_details(
		vec![0],
		curve,
		false,
		Some(PoolStatus::Active),
		Some(manager.clone()),
		None,
		Some(ACCOUNT_00),
	);
	let pool_id = calculate_pool_id(&[0]);
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

			let new_details = Pools::<Test>::get(&pool_id).unwrap();
			assert_eq!(new_details.manager, Some(manager));
		})
}

#[test]
fn handles_currency_idx_out_of_bounds() {
	let pool_details = generate_pool_details(
		vec![0],
		get_linear_bonding_curve(),
		false,
		Some(PoolStatus::Active),
		Some(ACCOUNT_00),
		None,
		None,
	);
	let pool_id = calculate_pool_id(&[0]);

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
