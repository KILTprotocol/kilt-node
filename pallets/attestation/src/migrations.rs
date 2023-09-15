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

use crate::{AccountIdOf, Attestations, ClaimHashOf, Config, CurrencyOf, Error, HoldReason};

pub fn update_balance_for_attestation<T: Config>(key: &ClaimHashOf<T>) -> DispatchResult
where
	<T as Config>::Currency:
		ReservableCurrency<T::AccountId, Balance = <<T as Config>::Currency as Inspect<AccountIdOf<T>>>::Balance>,
{
	let details = Attestations::<T>::get(key).ok_or(Error::<T>::NotFound)?;
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
	use sp_runtime::traits::Zero;

	use crate::{
		migrations::update_balance_for_attestation, mock::*, AccountIdOf, Attestations, AttesterOf, Config, Error,
		HoldReason,
	};

	#[test]
	fn test_setup() {
		let attester: AttesterOf<Test> = sr25519_did_from_public_key(&ALICE_SEED);
		let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_01);
		let ctype_hash = get_ctype_hash::<Test>(true);
		let attestations = generate_base_attestation::<Test>(attester.clone(), ACCOUNT_00);

		ExtBuilder::default()
			.with_ctypes(vec![(ctype_hash, attester)])
			.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get() * 100)])
			.with_attestations(vec![(claim_hash, attestations)])
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
	fn test_balance_migration_attestation() {
		let attester: AttesterOf<Test> = sr25519_did_from_public_key(&ALICE_SEED);
		let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_01);
		let claim_hash2 = claim_hash_from_seed(CLAIM_HASH_SEED_02);
		let ctype_hash = get_ctype_hash::<Test>(true);
		let attestations = generate_base_attestation::<Test>(attester.clone(), ACCOUNT_00);

		ExtBuilder::default()
			.with_ctypes(vec![(ctype_hash, attester)])
			.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get() * 100)])
			.with_attestations(vec![(claim_hash, attestations)])
			.build_and_execute_with_sanity_tests(|| {
				kilt_support::migration::translate_holds_to_reserve::<Test>(HoldReason::Deposit.into());

				let attestation_pre_migration = Attestations::<Test>::get(claim_hash);

				let reserved_pre_migration =
					<<Test as Config>::Currency as ReservableCurrency<AccountIdOf<Test>>>::reserved_balance(
						&ACCOUNT_00,
					);

				//attestations should be in storage
				assert!(attestation_pre_migration.is_some());

				// before the migration the deposit should be reserved.
				assert_eq!(
					reserved_pre_migration,
					attestation_pre_migration.clone().unwrap().deposit.amount
				);

				assert!(update_balance_for_attestation::<Test>(&claim_hash).is_ok());

				let attestation_post_migration = Attestations::<Test>::get(claim_hash);

				let reserved_post_migration =
					<<Test as Config>::Currency as ReservableCurrency<AccountIdOf<Test>>>::reserved_balance(
						&ACCOUNT_00,
					);

				let balance_on_hold =
					<<Test as Config>::Currency as InspectHold<AccountIdOf<Test>>>::total_balance_on_hold(&ACCOUNT_00);

				//attestations should be still in the storage
				assert!(attestation_post_migration.is_some());

				// ... and should be the same
				assert_eq!(attestation_post_migration, attestation_pre_migration);

				// Since reserved balance count to hold balance, it should not be zero
				assert!(!reserved_post_migration.is_zero());

				// ... and be as much as the hold balance
				assert_eq!(reserved_post_migration, balance_on_hold);

				// should throw error if claim hash does not exist
				assert_noop!(
					update_balance_for_attestation::<Test>(&claim_hash2),
					Error::<Test>::NotFound
				);
			});
	}
}
