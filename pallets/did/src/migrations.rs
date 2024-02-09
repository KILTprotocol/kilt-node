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

use crate::{AccountIdOf, Config, CurrencyOf, Did, DidIdentifierOf, Error, HoldReason};

pub fn update_balance_for_did<T: Config>(key: &DidIdentifierOf<T>) -> DispatchResult
where
	<T as Config>::Currency:
		ReservableCurrency<T::AccountId, Balance = <<T as Config>::Currency as Inspect<AccountIdOf<T>>>::Balance>,
{
	let details = Did::<T>::get(key).ok_or(Error::<T>::NotFound)?;
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
		traits::{
			fungible::{Inspect, InspectHold},
			ReservableCurrency,
		},
	};
	use sp_core::Pair;
	use sp_runtime::traits::Zero;

	use crate::{
		self as did, did_details::DidVerificationKey, migrations::update_balance_for_did, mock::*, mock_utils::*,
		AccountIdOf, Config, Did, Error, HoldReason,
	};

	#[test]
	fn test_setup() {
		let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
		let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());

		let mut did_details =
			generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(alice_did.clone()));
		did_details.deposit.amount = <Test as did::Config>::BaseDeposit::get();

		let balance = <Test as did::Config>::BaseDeposit::get()
			+ <Test as did::Config>::Fee::get()
			+ <<Test as did::Config>::Currency as Inspect<did::AccountIdOf<Test>>>::minimum_balance();
		ExtBuilder::default()
			.with_balances(vec![(alice_did.clone(), balance)])
			.with_dids(vec![(alice_did.clone(), did_details)])
			.build_and_execute_with_sanity_tests(None, || {
				let hold_balance_pre_migration =
					<<Test as Config>::Currency as InspectHold<AccountIdOf<Test>>>::balance_on_hold(
						&HoldReason::Deposit.into(),
						&alice_did,
					);

				assert_eq!(hold_balance_pre_migration, <Test as did::Config>::BaseDeposit::get());

				kilt_support::migration::translate_holds_to_reserve::<Test>(HoldReason::Deposit.into());

				let hold_balance = <<Test as Config>::Currency as InspectHold<AccountIdOf<Test>>>::balance_on_hold(
					&HoldReason::Deposit.into(),
					&alice_did,
				);

				let reserved_balance =
					<<Test as Config>::Currency as ReservableCurrency<AccountIdOf<Test>>>::reserved_balance(&alice_did);

				assert!(hold_balance.is_zero());
				assert_eq!(reserved_balance, <Test as Config>::BaseDeposit::get());
			})
	}

	#[test]
	fn test_balance_migration_did() {
		let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
		let auth_key2 = get_ed25519_authentication_key(&AUTH_SEED_1);
		let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
		let bob_did = get_did_identifier_from_ed25519_key(auth_key2.public());

		let mut did_details =
			generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(alice_did.clone()));
		did_details.deposit.amount = <Test as did::Config>::BaseDeposit::get();

		let balance = <Test as did::Config>::BaseDeposit::get()
			+ <Test as did::Config>::Fee::get()
			+ <<Test as did::Config>::Currency as Inspect<did::AccountIdOf<Test>>>::minimum_balance();
		ExtBuilder::default()
			.with_balances(vec![(alice_did.clone(), balance)])
			.with_dids(vec![(alice_did.clone(), did_details)])
			.build_and_execute_with_sanity_tests(None, || {
				kilt_support::migration::translate_holds_to_reserve::<Test>(HoldReason::Deposit.into());

				let did_pre_migration = Did::<Test>::get(alice_did.clone());

				let reserved_pre_migration =
					<<Test as Config>::Currency as ReservableCurrency<AccountIdOf<Test>>>::reserved_balance(
						&alice_did.clone(),
					);

				//did should be in storage
				assert!(did_pre_migration.is_some());

				// before the migration the deposit should be reserved.
				assert_eq!(
					reserved_pre_migration,
					did_pre_migration.clone().unwrap().deposit.amount
				);

				assert!(update_balance_for_did::<Test>(&alice_did.clone()).is_ok());

				let did_post_migration = Did::<Test>::get(alice_did.clone());

				let reserved_post_migration =
					<<Test as Config>::Currency as ReservableCurrency<AccountIdOf<Test>>>::reserved_balance(&alice_did);

				let balance_on_hold = <<Test as Config>::Currency as InspectHold<AccountIdOf<Test>>>::balance_on_hold(
					&HoldReason::Deposit.into(),
					&alice_did.clone(),
				);

				//did should be still in the storage
				assert!(did_post_migration.is_some());

				// ... and should be the same
				assert_eq!(did_post_migration, did_pre_migration);

				// Since reserved balance count to hold balance, it should not be zero
				assert!(!reserved_post_migration.is_zero());

				// ... and be as much as the hold balance
				assert_eq!(reserved_post_migration, balance_on_hold);

				// should throw error if did does not exist
				assert_noop!(update_balance_for_did::<Test>(&bob_did), Error::<Test>::NotFound);
			});
	}
}
