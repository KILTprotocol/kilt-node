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
use frame_support::{assert_err, assert_ok, traits::fungibles::roles::Inspect};
use frame_system::RawOrigin;

use crate::{
	mock::{runtime::*, *},
	traits::ResetTeam,
	types::{PoolManagingTeam, PoolStatus},
	AccountIdOf, Error as BondingPalletErrors,
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
		None,
	);
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	ExtBuilder::default()
		.with_pools(vec![(pool_id.clone(), pool_details)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.build_and_execute_with_sanity_tests(|| {
			let manager_origin = RawOrigin::Signed(ACCOUNT_00).into();

			assert_ok!(BondingPallet::reset_team(
				manager_origin,
				pool_id.clone(),
				PoolManagingTeam {
					admin: ACCOUNT_00,
					freezer: ACCOUNT_01,
				},
				1
			));

			assert_eq!(Assets::admin(DEFAULT_BONDED_CURRENCY_ID), Some(ACCOUNT_00));
			assert_eq!(Assets::freezer(DEFAULT_BONDED_CURRENCY_ID), Some(ACCOUNT_01));
			assert_eq!(Assets::owner(DEFAULT_BONDED_CURRENCY_ID), Some(pool_id.clone()));
			assert_eq!(Assets::issuer(DEFAULT_BONDED_CURRENCY_ID), Some(pool_id));
		})
}

#[test]
fn resets_owner_if_changed() {
	let pool_details = generate_pool_details(
		vec![DEFAULT_BONDED_CURRENCY_ID],
		get_linear_bonding_curve(),
		false,
		Some(PoolStatus::Active),
		Some(ACCOUNT_00),
		None,
		None,
		None,
	);
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	ExtBuilder::default()
		.with_pools(vec![(pool_id.clone(), pool_details)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.build_and_execute_with_sanity_tests(|| {
			Assets::reset_team(
				DEFAULT_BONDED_CURRENCY_ID,
				ACCOUNT_00,
				pool_id.clone(),
				pool_id.clone(),
				pool_id.clone(),
			)
			.expect("Failed to use reset_team trait");

			assert_eq!(Assets::owner(DEFAULT_BONDED_CURRENCY_ID), Some(ACCOUNT_00));
			assert_eq!(Assets::admin(DEFAULT_BONDED_CURRENCY_ID), Some(pool_id.clone()));
			assert_eq!(Assets::issuer(DEFAULT_BONDED_CURRENCY_ID), Some(pool_id.clone()));
			assert_eq!(Assets::freezer(DEFAULT_BONDED_CURRENCY_ID), Some(pool_id.clone()));

			let manager_origin = RawOrigin::Signed(ACCOUNT_00).into();

			assert_ok!(BondingPallet::reset_team(
				manager_origin,
				pool_id.clone(),
				PoolManagingTeam {
					admin: pool_id.clone(),
					freezer: pool_id.clone(),
				},
				1
			));

			assert_eq!(Assets::admin(DEFAULT_BONDED_CURRENCY_ID), Some(pool_id.clone()));
			assert_eq!(Assets::freezer(DEFAULT_BONDED_CURRENCY_ID), Some(pool_id.clone()));
			assert_eq!(Assets::owner(DEFAULT_BONDED_CURRENCY_ID), Some(pool_id.clone()));
			assert_eq!(Assets::issuer(DEFAULT_BONDED_CURRENCY_ID), Some(pool_id));
		})
}

#[test]
fn resets_team_for_all() {
	let currencies = vec![DEFAULT_BONDED_CURRENCY_ID, DEFAULT_BONDED_CURRENCY_ID + 1];

	let pool_details = generate_pool_details(
		currencies.clone(),
		get_linear_bonding_curve(),
		false,
		Some(PoolStatus::Active),
		Some(ACCOUNT_00),
		None,
		None,
		None,
	);
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&currencies);

	ExtBuilder::default()
		.with_pools(vec![(pool_id.clone(), pool_details)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.build_and_execute_with_sanity_tests(|| {
			let manager_origin = RawOrigin::Signed(ACCOUNT_00).into();

			assert_ok!(BondingPallet::reset_team(
				manager_origin,
				pool_id.clone(),
				PoolManagingTeam {
					admin: ACCOUNT_00,
					freezer: ACCOUNT_01,
				},
				2
			));

			assert_eq!(Assets::admin(currencies[0]), Some(ACCOUNT_00));
			assert_eq!(Assets::freezer(currencies[0]), Some(ACCOUNT_01));
			assert_eq!(Assets::owner(currencies[0]), Some(pool_id.clone()));
			assert_eq!(Assets::issuer(currencies[0]), Some(pool_id.clone()));

			assert_eq!(Assets::admin(currencies[1]), Some(ACCOUNT_00));
			assert_eq!(Assets::freezer(currencies[1]), Some(ACCOUNT_01));
			assert_eq!(Assets::owner(currencies[1]), Some(pool_id.clone()));
			assert_eq!(Assets::issuer(currencies[1]), Some(pool_id));
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
		None,
	);
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	ExtBuilder::default()
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_pools(vec![(pool_id.clone(), pool_details)])
		.build_and_execute_with_sanity_tests(|| {
			let manager_origin = RawOrigin::Signed(ACCOUNT_00).into();

			assert_err!(
				BondingPallet::reset_team(
					manager_origin,
					pool_id.clone(),
					PoolManagingTeam {
						admin: ACCOUNT_00,
						freezer: ACCOUNT_00,
					},
					1
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
		Some(manager),
		None,
		Some(ACCOUNT_00),
		None,
	);
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);
	ExtBuilder::default()
		.with_pools(vec![(pool_id.clone(), pool_details)])
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.build_and_execute_with_sanity_tests(|| {
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
					1
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
					1
				),
				BondingPalletErrors::<Test>::NoPermission
			);

			assert_eq!(Assets::admin(DEFAULT_BONDED_CURRENCY_ID), Some(pool_id));
		})
}

#[test]
fn handles_currency_number_incorrect() {
	let pool_details = generate_pool_details(
		vec![DEFAULT_BONDED_CURRENCY_ID],
		get_linear_bonding_curve(),
		false,
		Some(PoolStatus::Active),
		Some(ACCOUNT_00),
		None,
		None,
		None,
	);
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	ExtBuilder::default()
		.with_pools(vec![(pool_id.clone(), pool_details)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.build_and_execute_with_sanity_tests(|| {
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
				BondingPalletErrors::<Test>::CurrencyCount
			);
		})
}
