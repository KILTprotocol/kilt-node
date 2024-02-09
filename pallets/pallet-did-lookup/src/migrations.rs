// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2023 BOTLabs GmbH

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

use frame_support::{
	pallet_prelude::DispatchResult,
	traits::{fungible::Inspect, ReservableCurrency},
};
use kilt_support::migration::switch_reserved_to_hold;

use crate::{linkable_account::LinkableAccountId, AccountIdOf, Config, ConnectedDids, CurrencyOf, Error, HoldReason};

pub fn update_balance_for_did_lookup<T: Config>(key: &LinkableAccountId) -> DispatchResult
where
	<T as Config>::Currency:
		ReservableCurrency<T::AccountId, Balance = <<T as Config>::Currency as Inspect<AccountIdOf<T>>>::Balance>,
{
	let details = ConnectedDids::<T>::get(key).ok_or(Error::<T>::NotFound)?;
	switch_reserved_to_hold::<AccountIdOf<T>, CurrencyOf<T>>(
		&details.deposit.owner,
		&HoldReason::Deposit.into(),
		details.deposit.amount,
	)
}

#[cfg(test)]
pub mod test {
	use frame_support::{
		assert_noop,
		traits::{fungible::InspectHold, ReservableCurrency},
	};
	use sp_runtime::traits::Zero;

	use crate::{
		migrations::update_balance_for_did_lookup, mock::*, AccountIdOf, Config, ConnectedDids, Error, HoldReason,
	};

	#[test]
	fn test_setup() {
		ExtBuilder::default()
			.with_balances(vec![
				(ACCOUNT_00, <Test as crate::Config>::Deposit::get() * 50),
				(ACCOUNT_01, <Test as crate::Config>::Deposit::get() * 50),
			])
			.with_connections(vec![(ACCOUNT_00, DID_00, LINKABLE_ACCOUNT_00)])
			.build_and_execute_with_sanity_tests(|| {
				let hold_balance_pre_migration =
					<<Test as Config>::Currency as InspectHold<AccountIdOf<Test>>>::balance_on_hold(
						&HoldReason::Deposit.into(),
						&ACCOUNT_00,
					);

				assert_eq!(hold_balance_pre_migration, <Test as Config>::Deposit::get());

				kilt_support::migration::translate_holds_to_reserve::<Test>(HoldReason::Deposit.into());

				let hold_balance = <<Test as Config>::Currency as InspectHold<AccountIdOf<Test>>>::balance_on_hold(
					&HoldReason::Deposit.into(),
					&ACCOUNT_00,
				);

				let reserved_balance =
					<<Test as Config>::Currency as ReservableCurrency<AccountIdOf<Test>>>::reserved_balance(
						&ACCOUNT_00,
					);

				assert!(hold_balance.is_zero());
				assert_eq!(reserved_balance, <Test as Config>::Deposit::get());
			})
	}

	#[test]
	fn test_balance_migration_did_lookup() {
		ExtBuilder::default()
			.with_balances(vec![
				(ACCOUNT_00, <Test as crate::Config>::Deposit::get() * 50),
				(ACCOUNT_01, <Test as crate::Config>::Deposit::get() * 50),
			])
			.with_connections(vec![(ACCOUNT_00, DID_00, LINKABLE_ACCOUNT_00)])
			.build_and_execute_with_sanity_tests(|| {
				kilt_support::migration::translate_holds_to_reserve::<Test>(HoldReason::Deposit.into());

				// before the migration the balance should be reseved and not on
				// hold.
				let hold_balance = <<Test as Config>::Currency as InspectHold<AccountIdOf<Test>>>::balance_on_hold(
					&HoldReason::Deposit.into(),
					&ACCOUNT_00,
				);

				let reserved_balance =
					<<Test as Config>::Currency as ReservableCurrency<AccountIdOf<Test>>>::reserved_balance(
						&ACCOUNT_00,
					);

				assert!(hold_balance.is_zero());
				assert_eq!(reserved_balance, <Test as Config>::Deposit::get());

				let connected_did_pre_migration = ConnectedDids::<Test>::get(LINKABLE_ACCOUNT_00);

				let reserved_pre_migration =
					<<Test as Config>::Currency as ReservableCurrency<AccountIdOf<Test>>>::reserved_balance(
						&ACCOUNT_00,
					);

				//Connected did should be in storage
				assert!(connected_did_pre_migration.is_some());

				// before the migration the deposit should be reserved.
				assert_eq!(
					reserved_pre_migration,
					connected_did_pre_migration.clone().unwrap().deposit.amount
				);

				assert!(update_balance_for_did_lookup::<Test>(&LINKABLE_ACCOUNT_00).is_ok());

				let connected_did_post_migration = ConnectedDids::<Test>::get(LINKABLE_ACCOUNT_00);

				let reserved_post_migration =
					<<Test as Config>::Currency as ReservableCurrency<AccountIdOf<Test>>>::reserved_balance(
						&ACCOUNT_00,
					);

				let balance_on_hold = <<Test as Config>::Currency as InspectHold<AccountIdOf<Test>>>::balance_on_hold(
					&HoldReason::Deposit.into(),
					&ACCOUNT_00,
				);

				//Delegation should be still in the storage
				assert!(connected_did_post_migration.is_some());

				// ... and it should be the same
				assert_eq!(connected_did_post_migration, connected_did_pre_migration);

				// Since reserved balance count to hold balance, it should not be zero
				assert!(!reserved_post_migration.is_zero());

				// ... and be as much as the hold balance
				assert_eq!(reserved_post_migration, balance_on_hold);

				// should throw error if connected did does not exist
				assert_noop!(
					update_balance_for_did_lookup::<Test>(&LINKABLE_ACCOUNT_01),
					Error::<Test>::NotFound
				);
			})
	}
}
