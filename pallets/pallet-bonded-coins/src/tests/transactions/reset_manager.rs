// KILT Blockchain – <https://kilt.io>
// Copyright (C) 2025, KILT Foundation

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

// If you feel like getting in touch with us, you can do so at <hello@kilt.io>
use frame_support::{assert_err, assert_ok};
use frame_system::RawOrigin;

use crate::{
	mock::{runtime::*, *},
	types::PoolStatus,
	AccountIdOf, Error as BondingPalletErrors, Event as BondingPalletEvents, Pools,
};

#[test]
fn changes_manager() {
	let curve = get_linear_bonding_curve();

	let pool_details = generate_pool_details(
		vec![DEFAULT_BONDED_CURRENCY_ID],
		curve,
		false,
		Some(PoolStatus::Active),
		Some(ACCOUNT_00),
		None,
		None,
		None,
	);
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);
	ExtBuilder::default()
		.with_pools(vec![(pool_id.clone(), pool_details.clone())])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.build_and_execute_with_sanity_tests(|| {
			let origin = RawOrigin::Signed(ACCOUNT_00).into();
			assert_ok!(BondingPallet::reset_manager(origin, pool_id.clone(), Some(ACCOUNT_01)));

			System::assert_has_event(
				BondingPalletEvents::ManagerUpdated {
					id: pool_id.clone(),
					manager: Some(ACCOUNT_01),
				}
				.into(),
			);

			let new_details = Pools::<Test>::get(&pool_id).unwrap();
			assert_eq!(new_details.manager, Some(ACCOUNT_01));
			assert_eq!(new_details.owner, pool_details.owner)
		})
}

#[test]
fn only_manager_can_change_manager() {
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
		None,
	);
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);
	ExtBuilder::default()
		.with_pools(vec![(pool_id.clone(), pool_details)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT)])
		.build_and_execute_with_sanity_tests(|| {
			let owner_origin = RawOrigin::Signed(ACCOUNT_00).into();
			let other_origin = RawOrigin::Signed(ACCOUNT_01).into();

			assert_err!(
				BondingPallet::reset_manager(owner_origin, pool_id.clone(), Some(ACCOUNT_00)),
				BondingPalletErrors::<Test>::NoPermission
			);

			assert_err!(
				BondingPallet::reset_manager(other_origin, pool_id.clone(), Some(ACCOUNT_00)),
				BondingPalletErrors::<Test>::NoPermission
			);

			let new_details = Pools::<Test>::get(&pool_id).unwrap();
			assert_eq!(new_details.manager, Some(manager));
		})
}

#[test]
fn cant_change_manager_if_pool_nonexistent() {
	let pool_id: AccountIdOf<Test> = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);
	ExtBuilder::default().build().execute_with(|| {
		let origin = RawOrigin::Signed(ACCOUNT_00).into();

		assert!(Pools::<Test>::get(&pool_id).is_none());

		assert_err!(
			BondingPallet::reset_manager(origin, pool_id.clone(), Some(ACCOUNT_00)),
			BondingPalletErrors::<Test>::PoolUnknown
		);
	})
}
