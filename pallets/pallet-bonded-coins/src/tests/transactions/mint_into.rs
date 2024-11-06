use frame_support::{
	assert_ok,
	traits::{
		fungibles::Inspect,
		tokens::{Fortitude, Preservation},
	},
};
use frame_system::RawOrigin;

use crate::mock::{runtime::*, *};

#[test]
fn mint_first_coin() {
	let pool_id = calculate_pool_id(&[DEFAULT_BONDED_CURRENCY_ID]);

	let curve = get_linear_bonding_curve();

	ExtBuilder::default()
		.with_native_balances(vec![(ACCOUNT_00, ONE_HUNDRED_KILT)])
		.with_collaterals(vec![DEFAULT_COLLATERAL_CURRENCY_ID])
		.with_bonded_balance(vec![(DEFAULT_COLLATERAL_CURRENCY_ID, ACCOUNT_00, 100)])
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

			assert_ok!(BondingPallet::mint_into(origin, pool_id, 0, ACCOUNT_00, 1, 2, 1));

			assert_eq!(
				<Test as crate::Config>::Fungibles::total_balance(DEFAULT_COLLATERAL_CURRENCY_ID, &ACCOUNT_00),
				98
			);

			assert_eq!(
				<Test as crate::Config>::Fungibles::total_balance(DEFAULT_BONDED_CURRENCY_ID, &ACCOUNT_00),
				1
			);
			// Balance should not be frozen
			assert_eq!(
				<Test as crate::Config>::Fungibles::reducible_balance(
					DEFAULT_BONDED_CURRENCY_ID,
					&ACCOUNT_00,
					Preservation::Expendable,
					Fortitude::Polite
				),
				1
			);
		})
}
