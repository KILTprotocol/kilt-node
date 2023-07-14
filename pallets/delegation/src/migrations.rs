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

use crate::{AccountIdOf, Config, CurrencyOf, DelegationNode, DelegationNodes, HoldReason};

pub fn do_migration<T: Config>(who: <T as frame_system::Config>::AccountId, max_migrations: usize) -> usize
where
	<T as Config>::Currency: ReservableCurrency<T::AccountId>,
{
	DelegationNodes::<T>::iter()
		.filter(|(_, details)| details.deposit.owner == who && details.deposit.version.is_none())
		.take(max_migrations)
		.map(|(key, delegation_details)| {
			// switch reserves to hold.
			let deposit = delegation_details.deposit;
			let result = switch_reserved_to_hold::<AccountIdOf<T>, CurrencyOf<T>>(
				deposit.owner,
				&HoldReason::Deposit.into(),
				deposit.amount.saturated_into(),
			);

			// update the deposit
			DelegationNodes::<T>::mutate(key, |details| {
				if let Some(d) = details {
					*d = DelegationNode {
						deposit: Deposit {
							version: Some(1),
							owner: d.deposit.owner.clone(),
							amount: d.deposit.amount,
						},
						..delegation_details
					}
				}
			});

			debug_assert!(
				result.is_ok(),
				"Delegation: Could not convert reserves to hold from Delegation: {:?} error: {:?}",
				key,
				result
			);
		})
		.count()
}

#[cfg(test)]
pub mod test {
	use frame_support::traits::{fungible::InspectHold, ReservableCurrency};
	use sp_runtime::traits::Zero;

	use crate::{migrations::do_migration, mock::*, AccountIdOf, Config, DelegationNodes, HoldReason};

	#[test]
	fn test_balance_migration_delegation() {
		let user_1 = ed25519_did_from_seed(&ALICE_SEED);
		let user_2 = ed25519_did_from_seed(&BOB_SEED);

		let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
		let hierarchy_details = generate_base_delegation_hierarchy_details::<Test>();

		let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
		let mut delegation_details =
			generate_base_delegation_node::<Test>(hierarchy_root_id, user_2, Some(hierarchy_root_id), ACCOUNT_01);
		delegation_details.deposit.version = None;

		ExtBuilder::default()
			.with_ctypes(vec![(hierarchy_details.ctype_hash, user_1.clone())])
			.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, user_1, ACCOUNT_00)])
			.with_delegations(vec![(delegation_id, delegation_details)])
			.with_balances(vec![
				(ACCOUNT_00, <Test as Config>::Deposit::get()),
				(ACCOUNT_01, <Test as Config>::Deposit::get()),
				(ACCOUNT_02, <Test as Config>::Deposit::get()),
			])
			.build_and_execute_with_sanity_tests(|| {
				translate_holds_to_reserve();

				// before the migration the balance should be reseved and not on
				// hold.
				let hold_balance_setup =
					<<Test as Config>::Currency as InspectHold<AccountIdOf<Test>>>::balance_on_hold(
						&HoldReason::Deposit.into(),
						&ACCOUNT_01,
					);

				let reserved_balacne_setup =
					<<Test as Config>::Currency as ReservableCurrency<AccountIdOf<Test>>>::reserved_balance(
						&ACCOUNT_01,
					);

				assert_eq!(hold_balance_setup, 0);
				assert_eq!(reserved_balacne_setup, <Test as Config>::Deposit::get());

				let delegation_pre_migration = DelegationNodes::<Test>::get(delegation_id);

				let balance_on_reserve_pre_migration = <<Test as Config>::Currency as ReservableCurrency<
					AccountIdOf<Test>,
				>>::reserved_balance(&ACCOUNT_01);

				//Delegation should be in storage
				assert!(delegation_pre_migration.is_some());

				//before the migration the version should be none.
				assert!(delegation_pre_migration.clone().unwrap().deposit.version.is_none());

				// before the migration the deposit should be reserved.
				assert_eq!(
					balance_on_reserve_pre_migration,
					delegation_pre_migration.unwrap().deposit.amount
				);

				do_migration::<Test>(ACCOUNT_01, 1);

				let delegation_post_migration = DelegationNodes::<Test>::get(delegation_id);

				let balance_on_reserve_post_migration = <<Test as Config>::Currency as ReservableCurrency<
					AccountIdOf<Test>,
				>>::reserved_balance(&ACCOUNT_01);

				let balance_on_hold = <<Test as Config>::Currency as InspectHold<AccountIdOf<Test>>>::balance_on_hold(
					&HoldReason::Deposit.into(),
					&ACCOUNT_01,
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
			});
	}
}
