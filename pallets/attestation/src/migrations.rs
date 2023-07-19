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

use frame_support::{ensure, pallet_prelude::DispatchResult, traits::ReservableCurrency};
use kilt_support::{migration::switch_reserved_to_hold, Deposit};
use sp_runtime::SaturatedConversion;

use crate::{AccountIdOf, AttestationDetails, Attestations, ClaimHashOf, Config, CurrencyOf, Error, HoldReason};

pub fn update_balance_for_entry<T: Config>(key: &ClaimHashOf<T>) -> DispatchResult
where
	<T as Config>::Currency: ReservableCurrency<T::AccountId>,
{
	Attestations::<T>::try_mutate(key, |details| {
		if let Some(d) = details {
			ensure!(d.deposit.version.is_none(), Error::<T>::Migration);

			*d = AttestationDetails {
				deposit: Deposit {
					version: Some(1),
					owner: d.deposit.owner.clone(),
					amount: d.deposit.amount,
				},
				..d.clone()
			};

			switch_reserved_to_hold::<AccountIdOf<T>, CurrencyOf<T>>(
				d.clone().deposit.owner,
				&HoldReason::Deposit.into(),
				d.deposit.amount.saturated_into(),
			)
		} else {
			Err(Error::<T>::NotFound.into())
		}
	})
}

#[cfg(test)]
pub mod test {
	use ctype::mock::get_ctype_hash;
	use frame_support::traits::{fungible::InspectHold, ReservableCurrency};
	use sp_runtime::traits::Zero;

	use crate::{
		migrations::update_balance_for_entry, mock::*, AccountIdOf, Attestations, AttesterOf, Config, HoldReason,
	};

	#[test]
	fn test_setup() {
		let attester: AttesterOf<Test> = sr25519_did_from_seed(&ALICE_SEED);
		let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_01);
		let ctype_hash = get_ctype_hash::<Test>(true);
		let mut attestations = generate_base_attestation::<Test>(attester.clone(), ACCOUNT_00);
		attestations.deposit.version = None;

		ExtBuilder::default()
			.with_ctypes(vec![(ctype_hash, attester)])
			.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get() * 100)])
			.with_attestations(vec![(claim_hash, attestations)])
			.build_and_execute_with_sanity_tests(|| {
				translate_holds_to_reserve();

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

				assert_eq!(hold_balance, 0);
				assert_eq!(reserved_balance, <Test as Config>::Deposit::get());
			})
	}

	#[test]
	fn test_balance_migration_attestation() {
		let attester: AttesterOf<Test> = sr25519_did_from_seed(&ALICE_SEED);
		let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_01);
		let ctype_hash = get_ctype_hash::<Test>(true);
		let mut attestations = generate_base_attestation::<Test>(attester.clone(), ACCOUNT_00);
		attestations.deposit.version = None;

		ExtBuilder::default()
			.with_ctypes(vec![(ctype_hash, attester)])
			.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get() * 100)])
			.with_attestations(vec![(claim_hash, attestations)])
			.build_and_execute_with_sanity_tests(|| {
				translate_holds_to_reserve();

				let attestation_pre_migration = Attestations::<Test>::get(claim_hash);

				let balance_on_reserve_pre_migration = <<Test as Config>::Currency as ReservableCurrency<
					AccountIdOf<Test>,
				>>::reserved_balance(&ACCOUNT_00);

				//attestations should be in storage
				assert!(attestation_pre_migration.is_some());

				//before the migration the version should be none.
				assert!(attestation_pre_migration.clone().unwrap().deposit.version.is_none());

				// before the migration the deposit should be reserved.
				assert_eq!(
					balance_on_reserve_pre_migration,
					attestation_pre_migration.unwrap().deposit.amount
				);

				assert!(update_balance_for_entry::<Test>(&claim_hash).is_ok());

				let attestation_post_migration = Attestations::<Test>::get(claim_hash);

				let balance_on_reserve_post_migration = <<Test as Config>::Currency as ReservableCurrency<
					AccountIdOf<Test>,
				>>::reserved_balance(&ACCOUNT_00);

				let balance_on_hold =
					<<Test as Config>::Currency as InspectHold<AccountIdOf<Test>>>::total_balance_on_hold(&ACCOUNT_00);

				//attestations should be still in the storage
				assert!(attestation_post_migration.is_some());

				// Since reserved balance count to hold balance, it should not be zero
				assert!(!balance_on_reserve_post_migration.is_zero());

				// ... and be as much as the hold balance
				assert_eq!(balance_on_reserve_post_migration, balance_on_hold);

				//... and the version should be 1.
				assert!(attestation_post_migration.clone().unwrap().deposit.version.is_some());
				assert!(attestation_post_migration.unwrap().deposit.version.unwrap() == 1);

				// Nothing should happen
				assert!(update_balance_for_entry::<Test>(&claim_hash).is_err());
			});
	}
}
