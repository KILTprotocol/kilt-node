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

use crate::{did_details::DidDetails, AccountIdOf, Config, CurrencyOf, Did, HoldReason};

pub fn do_migration<T: Config>(who: T::AccountId, max_migrations: usize) -> usize
where
	<T as Config>::Currency: ReservableCurrency<T::AccountId>,
{
	let executed_migrations = Did::<T>::iter()
		.filter(|(_, details)| details.deposit.owner == who && details.deposit.version.is_none())
		.take(max_migrations)
		.map(|(key, did_details)| {
			// switch reserves to hold.
			let deposit = did_details.deposit;
			let result = switch_reserved_to_hold::<AccountIdOf<T>, CurrencyOf<T>>(
				deposit.owner,
				&HoldReason::Deposit.into(),
				deposit.amount.saturated_into(),
			);

			// update the deposit
			Did::<T>::mutate(key.clone(), |details| {
				if let Some(d) = details {
					*d = DidDetails {
						deposit: Deposit {
							version: Some(1),
							owner: d.deposit.owner.clone(),
							amount: d.deposit.amount,
						},
						..did_details
					}
				}
			});

			debug_assert!(
				result.is_ok(),
				"Did: Could not convert reserves to hold from Did: {:?} error: {:?}",
				key,
				result
			);
		})
		.count();

	max_migrations.saturating_sub(executed_migrations)
}

#[cfg(test)]
pub mod test {
	use frame_support::traits::{
		fungible::{Inspect, InspectHold},
		ReservableCurrency,
	};
	use sp_core::Pair;
	use sp_runtime::traits::Zero;

	use crate::{
		self as did, did_details::DidVerificationKey, migrations::do_migration, mock::*, mock_utils::*, AccountIdOf,
		Config, Did, HoldReason,
	};

	#[test]
	fn test_setup() {
		let auth_key = get_ed25519_authentication_key(true);
		let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());

		let mut did_details =
			generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(alice_did.clone()));
		did_details.deposit.version = None;
		did_details.deposit.amount = <Test as did::Config>::BaseDeposit::get();

		let balance = <Test as did::Config>::BaseDeposit::get()
			+ <Test as did::Config>::Fee::get()
			+ <<Test as did::Config>::Currency as Inspect<did::AccountIdOf<Test>>>::minimum_balance();
		ExtBuilder::default()
			.with_balances(vec![(alice_did.clone(), balance)])
			.with_dids(vec![(alice_did.clone(), did_details)])
			.build_and_execute_with_sanity_tests(None, || {
				translate_holds_to_reserve();

				// before the migration the balance should be reseved and not on
				// hold.
				let hold_balance_setup =
					<<Test as Config>::Currency as InspectHold<AccountIdOf<Test>>>::balance_on_hold(
						&HoldReason::Deposit.into(),
						&alice_did,
					);

				let reserved_balacne_setup =
					<<Test as Config>::Currency as ReservableCurrency<AccountIdOf<Test>>>::reserved_balance(&alice_did);

				assert_eq!(hold_balance_setup, 0);
				assert_eq!(reserved_balacne_setup, <Test as Config>::BaseDeposit::get());
			})
	}

	#[test]
	fn test_balance_migration_did() {
		let auth_key = get_ed25519_authentication_key(true);
		let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());

		let mut did_details =
			generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(alice_did.clone()));
		did_details.deposit.version = None;
		did_details.deposit.amount = <Test as did::Config>::BaseDeposit::get();

		let balance = <Test as did::Config>::BaseDeposit::get()
			+ <Test as did::Config>::Fee::get()
			+ <<Test as did::Config>::Currency as Inspect<did::AccountIdOf<Test>>>::minimum_balance();
		ExtBuilder::default()
			.with_balances(vec![(alice_did.clone(), balance)])
			.with_dids(vec![(alice_did.clone(), did_details)])
			.build_and_execute_with_sanity_tests(None, || {
				translate_holds_to_reserve();

				let did_pre_migration = Did::<Test>::get(alice_did.clone());

				let balance_on_reserve_pre_migration = <<Test as Config>::Currency as ReservableCurrency<
					AccountIdOf<Test>,
				>>::reserved_balance(&alice_did.clone());

				//did should be in storage
				assert!(did_pre_migration.is_some());

				//before the migration the version should be none.
				assert!(did_pre_migration.clone().unwrap().deposit.version.is_none());

				// before the migration the deposit should be reserved.
				assert_eq!(
					balance_on_reserve_pre_migration,
					did_pre_migration.unwrap().deposit.amount
				);

				let remaining_migrations = do_migration::<Test>(alice_did.clone(), 1);

				assert_eq!(remaining_migrations, 0);

				let did_post_migration = Did::<Test>::get(alice_did.clone());

				let balance_on_reserve_post_migration =
					<<Test as Config>::Currency as ReservableCurrency<AccountIdOf<Test>>>::reserved_balance(&alice_did);

				let balance_on_hold = <<Test as Config>::Currency as InspectHold<AccountIdOf<Test>>>::balance_on_hold(
					&HoldReason::Deposit.into(),
					&alice_did.clone(),
				);

				//did should be still in the storage
				assert!(did_post_migration.is_some());

				// Since reserved balance count to hold balance, it should not be zero
				assert!(!balance_on_reserve_post_migration.is_zero());

				// ... and be as much as the hold balance
				assert_eq!(balance_on_reserve_post_migration, balance_on_hold);

				//... and the version should be 1.
				assert!(did_post_migration.clone().unwrap().deposit.version.is_some());
				assert!(did_post_migration.unwrap().deposit.version.unwrap() == 1);

				// Nothing should happen
				let remaining_migrations = do_migration::<Test>(alice_did.clone(), 1);

				assert_eq!(remaining_migrations, 1);
			});
	}
}
