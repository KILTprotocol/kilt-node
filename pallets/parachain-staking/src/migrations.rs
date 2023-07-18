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
	traits::{fungible::freeze::Mutate as MutateFreeze, LockIdentifier, LockableCurrency},
};
use pallet_balances::{BalanceLock, Freezes, Locks};
use sp_runtime::{SaturatedConversion, WeakBoundedVec};

use crate::{
	types::{AccountIdOf, CurrencyOf},
	Config, FreezeReason,
};

const STAKING_ID: LockIdentifier = *b"kiltpstk";

pub fn update_or_create_freeze<T: Config>(user_id: &T::AccountId) -> DispatchResult
where
	CurrencyOf<T>: LockableCurrency<AccountIdOf<T>>,
{
	let locks: WeakBoundedVec<
		BalanceLock<<T as pallet_balances::Config>::Balance>,
		<T as pallet_balances::Config>::MaxLocks,
	> = Locks::<T>::get(&user_id);

	debug_assert!(!locks.is_empty(), "No locks");

	locks
		.iter()
		.filter(|lock| lock.id == STAKING_ID)
		.try_for_each(|lock| -> DispatchResult {
			<CurrencyOf<T> as LockableCurrency<AccountIdOf<T>>>::remove_lock(STAKING_ID, &user_id);

			let are_freezes_stored = Freezes::<T>::get(&user_id)
				.iter()
				.any(|freeze| freeze.id == <T as Config>::FreezeIdentifier::from(FreezeReason::Staking).into());

			if are_freezes_stored {
				<CurrencyOf<T> as MutateFreeze<AccountIdOf<T>>>::extend_freeze(
					&<T as crate::Config>::FreezeIdentifier::from(FreezeReason::Staking),
					&user_id,
					lock.amount.saturated_into(),
				)
			} else {
				<CurrencyOf<T> as MutateFreeze<AccountIdOf<T>>>::set_freeze(
					&<T as crate::Config>::FreezeIdentifier::from(FreezeReason::Staking),
					&user_id,
					lock.amount.saturated_into(),
				)
			}
		})
}

#[cfg(test)]
pub mod test {
	use frame_support::traits::{
		fungible::{Inspect, InspectFreeze},
		tokens::{Fortitude, Preservation},
	};
	use pallet_balances::{Freezes, Locks};
	use sp_runtime::traits::Zero;

	use crate::{migrations::update_or_create_freeze, mock::*, Config, FreezeReason};

	#[test]
	#[should_panic(expected = "No locks")]
	fn test_balance_migration_staking() {
		ExtBuilder::default()
			.with_balances(vec![(1, 10), (2, 100), (3, 100)])
			.with_collators(vec![(1, 10), (3, 10)])
			.with_delegators(vec![(2, 1, 100)])
			.build_and_execute_with_sanity_tests(|| {
				translate_freezes_to_locks();

				// after the translation, there should be no freezes but locks
				let count_freezes_pre_migration = Freezes::<Test>::iter().count();
				let count_locks_pre_migration = Locks::<Test>::iter().count();

				assert!(count_freezes_pre_migration.is_zero());
				assert_eq!(count_locks_pre_migration, 3);

				let reducible_balance_user_1 =
					pallet_balances::Pallet::<Test>::reducible_balance(&1, Preservation::Preserve, Fortitude::Polite);
				let reducible_balance_user_2 =
					pallet_balances::Pallet::<Test>::reducible_balance(&2, Preservation::Preserve, Fortitude::Polite);
				let reducible_balance_user_3 =
					pallet_balances::Pallet::<Test>::reducible_balance(&3, Preservation::Preserve, Fortitude::Polite);

				assert_eq!(reducible_balance_user_1, 0);
				assert_eq!(reducible_balance_user_2, 0);
				assert_eq!(reducible_balance_user_3, 90);

				assert!(update_or_create_freeze::<Test>(&1).is_ok());
				assert!(update_or_create_freeze::<Test>(&2).is_ok());
				assert!(update_or_create_freeze::<Test>(&3).is_ok());

				let froozen_balance_1 = pallet_balances::Pallet::<Test>::balance_frozen(
					&<Test as Config>::FreezeIdentifier::from(FreezeReason::Staking),
					&1,
				);

				let froozen_balance_2 = pallet_balances::Pallet::<Test>::balance_frozen(
					&<Test as Config>::FreezeIdentifier::from(FreezeReason::Staking),
					&2,
				);

				let froozen_balance_3 = pallet_balances::Pallet::<Test>::balance_frozen(
					&<Test as Config>::FreezeIdentifier::from(FreezeReason::Staking),
					&3,
				);

				assert_eq!(froozen_balance_1, 10);
				assert_eq!(froozen_balance_2, 100);
				assert_eq!(froozen_balance_3, 10);

				//Nothing should happen
				assert!(update_or_create_freeze::<Test>(&1).is_err());
			})
	}
}
