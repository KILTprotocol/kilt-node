use frame_support::{assert_err, assert_ok};
use frame_system::RawOrigin;

use crate::{
	mock::{runtime::*, *},
	types::PoolStatus,
	Error as BondingPalletErrors, Event as BondingPalletEvents, Pools,
};

#[test]
fn changes_manager() {
	let curve = get_linear_bonding_curve();

	let pool_details = generate_pool_details(
		vec![0],
		curve,
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
	let pool_id = calculate_pool_id(&[0]);
	ExtBuilder::default().build().execute_with(|| {
		let origin = RawOrigin::Signed(ACCOUNT_00).into();

		assert!(Pools::<Test>::get(&pool_id).is_none());

		assert_err!(
			BondingPallet::reset_manager(origin, pool_id.clone(), Some(ACCOUNT_00)),
			BondingPalletErrors::<Test>::PoolUnknown
		);
	})
}
