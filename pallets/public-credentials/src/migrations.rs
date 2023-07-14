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

use crate::{AccountIdOf, Config, CredentialEntry, Credentials, CurrencyOf, HoldReason};

pub fn do_migration<T: Config>(who: T::AccountId, max_migrations: usize) -> usize
where
	<T as Config>::Currency: ReservableCurrency<T::AccountId>,
{
	let executed_migrations = Credentials::<T>::iter()
		.filter(|(_, _, details)| details.deposit.owner == who && details.deposit.version.is_none())
		.take(max_migrations)
		.map(|(key1, key2, delegation_details)| {
			// switch reserves to hold.
			let deposit = delegation_details.deposit;
			let result = switch_reserved_to_hold::<AccountIdOf<T>, CurrencyOf<T>>(
				deposit.owner,
				&HoldReason::Deposit.into(),
				deposit.amount.saturated_into(),
			);

			// update the deposit
			Credentials::<T>::mutate(key1.clone(), key2.clone(), |details| {
				if let Some(d) = details {
					*d = CredentialEntry {
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
				"Delegation: Could not convert reserves to hold from Delegation: {:?}, {:?} error: {:?}",
				key1,
				key2,
				result
			);
		})
		.count();

	max_migrations.saturating_sub(executed_migrations)
}

#[cfg(test)]
pub mod test {

	use ctype::mock::get_ctype_hash;
	use frame_support::traits::{fungible::InspectHold, ReservableCurrency};
	use sp_core::Get;
	use sp_runtime::traits::Zero;

	use crate::{migrations::do_migration, mock::*, AccountIdOf, Config, CredentialIdOf, Credentials, HoldReason};

	#[test]
	fn test_setup() {
		let attester = sr25519_did_from_seed(&ALICE_SEED);

		let ctype_hash_1 = get_ctype_hash::<Test>(true);
		let subject_id: <Test as Config>::SubjectId = SUBJECT_ID_00;
		let mut new_credential =
			generate_base_credential_entry::<Test>(ACCOUNT_00, 0, attester.clone(), Some(ctype_hash_1), None);
		new_credential.authorization_id = Some(attester.clone());
		new_credential.deposit.version = None;

		let credential_id: CredentialIdOf<Test> = CredentialIdOf::<Test>::default();
		let deposit: Balance = <Test as Config>::Deposit::get();

		ExtBuilder::default()
			.with_balances(vec![(ACCOUNT_00, deposit + MIN_BALANCE)])
			.with_public_credentials(vec![(subject_id, credential_id, new_credential)])
			.with_ctypes(vec![(ctype_hash_1, attester)])
			.build_and_execute_with_sanity_tests(|| {
				translate_holds_to_reserve();

				// before the migration the balance should be reseved and not on
				// hold.
				let hold_balance_setup =
					<<Test as Config>::Currency as InspectHold<AccountIdOf<Test>>>::balance_on_hold(
						&HoldReason::Deposit.into(),
						&ACCOUNT_00,
					);

				let reserved_balacne_setup =
					<<Test as Config>::Currency as ReservableCurrency<AccountIdOf<Test>>>::reserved_balance(
						&ACCOUNT_00,
					);

				assert_eq!(hold_balance_setup, 0);
				assert_eq!(reserved_balacne_setup, <Test as Config>::Deposit::get());
			})
	}

	#[test]
	fn test_balance_migration_public_credential() {
		let attester = sr25519_did_from_seed(&ALICE_SEED);

		let ctype_hash_1 = get_ctype_hash::<Test>(true);
		let subject_id: <Test as Config>::SubjectId = SUBJECT_ID_00;
		let mut new_credential =
			generate_base_credential_entry::<Test>(ACCOUNT_00, 0, attester.clone(), Some(ctype_hash_1), None);
		new_credential.authorization_id = Some(attester.clone());
		new_credential.deposit.version = None;

		let credential_id: CredentialIdOf<Test> = CredentialIdOf::<Test>::default();
		let deposit: Balance = <Test as Config>::Deposit::get();

		ExtBuilder::default()
			.with_balances(vec![(ACCOUNT_00, deposit + MIN_BALANCE)])
			.with_public_credentials(vec![(subject_id, credential_id, new_credential)])
			.with_ctypes(vec![(ctype_hash_1, attester)])
			.build_and_execute_with_sanity_tests(|| {
				translate_holds_to_reserve();
				let delegation_pre_migration = Credentials::<Test>::get(subject_id, credential_id);

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

				let remaining_migrations = do_migration::<Test>(ACCOUNT_00, 1);
				assert_eq!(remaining_migrations, 0);

				let delegation_post_migration = Credentials::<Test>::get(subject_id, credential_id);

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

				//Nothing should happen
				let remaining_migrations = do_migration::<Test>(ACCOUNT_00, 1);
				assert_eq!(remaining_migrations, 1);
			})
	}
}
