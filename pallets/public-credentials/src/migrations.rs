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

use frame_support::traits::{fungible::Inspect, ReservableCurrency};
use kilt_support::migration::switch_reserved_to_hold;
use sp_runtime::DispatchResult;

use crate::{AccountIdOf, Config, CredentialIdOf, Credentials, CurrencyOf, Error, HoldReason, SubjectIdOf};

pub fn update_balance_for_public_credentials<T: Config>(
	key: &SubjectIdOf<T>,
	key2: &CredentialIdOf<T>,
) -> DispatchResult
where
	<T as Config>::Currency:
		ReservableCurrency<T::AccountId, Balance = <<T as Config>::Currency as Inspect<AccountIdOf<T>>>::Balance>,
{
	let details = Credentials::<T>::get(key, key2).ok_or(Error::<T>::NotFound)?;
	switch_reserved_to_hold::<AccountIdOf<T>, CurrencyOf<T>>(
		&details.deposit.owner,
		&HoldReason::Deposit.into(),
		details.deposit.amount,
	)
}

#[cfg(test)]
pub mod test {

	use ctype::mock::get_ctype_hash;
	use frame_support::{
		assert_noop,
		traits::{fungible::InspectHold, ReservableCurrency},
	};
	use sp_core::Get;
	use sp_runtime::traits::Zero;

	use crate::{
		migrations::update_balance_for_public_credentials, mock::*, AccountIdOf, Config, CredentialIdOf, Credentials,
		Error, HoldReason,
	};

	#[test]
	fn test_setup() {
		let attester = sr25519_did_from_seed(&ALICE_SEED);

		let ctype_hash_1 = get_ctype_hash::<Test>(true);
		let subject_id: <Test as Config>::SubjectId = SUBJECT_ID_00;
		let mut new_credential =
			generate_base_credential_entry::<Test>(ACCOUNT_00, 0, attester.clone(), Some(ctype_hash_1), None);
		new_credential.authorization_id = Some(attester.clone());

		let credential_id: CredentialIdOf<Test> = CredentialIdOf::<Test>::default();
		let deposit: Balance = <Test as Config>::Deposit::get();

		ExtBuilder::default()
			.with_balances(vec![(ACCOUNT_00, deposit + MIN_BALANCE)])
			.with_public_credentials(vec![(subject_id, credential_id, new_credential)])
			.with_ctypes(vec![(ctype_hash_1, attester)])
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
	fn test_balance_migration_public_credential() {
		let attester = sr25519_did_from_seed(&ALICE_SEED);

		let ctype_hash_1 = get_ctype_hash::<Test>(true);
		let subject_id: <Test as Config>::SubjectId = SUBJECT_ID_00;
		let subject_id2: <Test as Config>::SubjectId = SUBJECT_ID_01;
		let mut new_credential =
			generate_base_credential_entry::<Test>(ACCOUNT_00, 0, attester.clone(), Some(ctype_hash_1), None);
		new_credential.authorization_id = Some(attester.clone());

		let credential_id: CredentialIdOf<Test> = CredentialIdOf::<Test>::default();
		let deposit: Balance = <Test as Config>::Deposit::get();

		ExtBuilder::default()
			.with_balances(vec![(ACCOUNT_00, deposit + MIN_BALANCE)])
			.with_public_credentials(vec![(subject_id, credential_id, new_credential)])
			.with_ctypes(vec![(ctype_hash_1, attester)])
			.build_and_execute_with_sanity_tests(|| {
				kilt_support::migration::translate_holds_to_reserve::<Test>(HoldReason::Deposit.into());
				let public_credentials_pre_migration = Credentials::<Test>::get(subject_id, credential_id);

				let reserved_pre_migration =
					<<Test as Config>::Currency as ReservableCurrency<AccountIdOf<Test>>>::reserved_balance(
						&ACCOUNT_00,
					);

				//public credentials should be in storage
				assert!(public_credentials_pre_migration.is_some());

				// before the migration the deposit should be reserved.
				assert_eq!(
					reserved_pre_migration,
					public_credentials_pre_migration.clone().unwrap().deposit.amount
				);

				assert!(update_balance_for_public_credentials::<Test>(&subject_id, &credential_id).is_ok());

				let public_credentials_post_migration = Credentials::<Test>::get(subject_id, credential_id);

				let reserved_post_migration =
					<<Test as Config>::Currency as ReservableCurrency<AccountIdOf<Test>>>::reserved_balance(
						&ACCOUNT_00,
					);

				let balance_on_hold = <<Test as Config>::Currency as InspectHold<AccountIdOf<Test>>>::balance_on_hold(
					&HoldReason::Deposit.into(),
					&ACCOUNT_00,
				);

				//Delegation should be still in the storage
				assert!(public_credentials_post_migration.is_some());

				// ... and it should be the same
				assert_eq!(public_credentials_post_migration, public_credentials_pre_migration);

				// Since reserved balance count to hold balance, it should not be zero
				assert!(!reserved_post_migration.is_zero());

				// ... and be as much as the hold balance
				assert_eq!(reserved_post_migration, balance_on_hold);

				// should throw error if public credential does not exist
				assert_noop!(
					update_balance_for_public_credentials::<Test>(&subject_id2, &credential_id),
					Error::<Test>::NotFound
				);
			})
	}
}
