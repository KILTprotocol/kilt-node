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

use frame_support::traits::ReservableCurrency;
use kilt_support::{migration::switch_reserved_to_hold, Deposit};
use sp_runtime::SaturatedConversion;

use crate::{web3_name::Web3NameOwnership, AccountIdOf, Config, CurrencyOf, HoldReason, Owner};

pub fn do_migration<T: Config>(who: T::AccountId)
where
	<T as Config>::Currency: ReservableCurrency<T::AccountId>,
{
	Owner::<T>::iter()
		.filter(|(_, details)| details.deposit.owner == who && details.deposit.version.is_none())
		.for_each(|(key, w3n_details)| {
			// switch reserves to hold.
			let deposit = w3n_details.deposit;
			let result = switch_reserved_to_hold::<AccountIdOf<T>, CurrencyOf<T>>(
				deposit.owner,
				&HoldReason::Deposit.into(),
				deposit.amount.saturated_into(),
			);

			// update the deposit
			Owner::<T>::mutate(key.clone(), |details| {
				if let Some(d) = details {
					*d = Web3NameOwnership {
						deposit: Deposit {
							version: Some(1),
							owner: d.deposit.owner.clone(),
							amount: d.deposit.amount,
						},
						..w3n_details
					}
				}
			});

			debug_assert!(
				result.is_ok(),
				"W3n: Could not convert reserves to hold from W3n: {:?} error: {:?}",
				key,
				result
			);
		});
}

#[cfg(test)]
pub mod test {
	use frame_support::traits::{fungible::InspectHold, ReservableCurrency};
	use sp_runtime::traits::Zero;

	use crate::{migrations::do_migration, mock::*, AccountIdOf, Config, HoldReason, Owner};

	#[test]
	fn test_balance_migration_w3n() {
		let web3_name_00 = get_web3_name(WEB3_NAME_00_INPUT);
		ExtBuilder::default()
			.with_balances(vec![(ACCOUNT_00, Web3NameDeposit::get() * 2)])
			.with_web3_names(vec![(DID_00, web3_name_00.clone(), ACCOUNT_00)])
			.build_and_execute_with_sanity_tests(true, || {
				let delegation_pre_migration = Owner::<Test>::get(web3_name_00.clone());

				let balance_on_reserve_pre_migration = <<Test as Config>::Currency as ReservableCurrency<
					AccountIdOf<Test>,
				>>::reserved_balance(&ACCOUNT_00);

				//Delegation should be in storage
				assert!(delegation_pre_migration.is_some());

				//before the migration the version should be none.
				assert!(delegation_pre_migration.clone().unwrap().deposit.version.is_none());

				// before the migration the deposit should be reserved.
				assert_eq!(
					balance_on_reserve_pre_migration,
					delegation_pre_migration.unwrap().deposit.amount
				);

				do_migration::<Test>(ACCOUNT_00);

				let delegation_post_migration = Owner::<Test>::get(web3_name_00);

				let balance_on_reserve_post_migration = <<Test as Config>::Currency as ReservableCurrency<
					AccountIdOf<Test>,
				>>::reserved_balance(&ACCOUNT_00);

				let balance_on_hold = <<Test as Config>::Currency as InspectHold<AccountIdOf<Test>>>::balance_on_hold(
					&HoldReason::Deposit.into(),
					&ACCOUNT_00,
				);

				//Delegation should be still in the storage
				assert!(delegation_post_migration.is_some());

				// Since reserved balance count to hold balance, it should not be zero
				assert!(!balance_on_reserve_post_migration.is_zero());

				// ... and be as much as the hold balance
				assert_eq!(balance_on_reserve_post_migration, balance_on_hold);

				//... and the version should be 1.
				assert!(delegation_post_migration.clone().unwrap().deposit.version.is_some());
				assert!(delegation_post_migration.unwrap().deposit.version.unwrap() == 1);
			})
	}
}
