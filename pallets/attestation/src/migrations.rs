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

use crate::{AccountIdOf, AttestationDetails, Attestations, Config, CurrencyOf, HoldReason};

pub fn do_migration<T: Config>(who: T::AccountId, max_migrations: usize) -> usize
where
	<T as Config>::Currency: ReservableCurrency<T::AccountId>,
{
	Attestations::<T>::iter()
		.filter(|(_, details)| details.deposit.owner == who && details.deposit.version.is_none())
		.take(max_migrations)
		.map(|(key, attestations_detail)| {
			// switch reserves to hold.
			let deposit = attestations_detail.deposit;
			let result = switch_reserved_to_hold::<AccountIdOf<T>, CurrencyOf<T>>(
				deposit.owner,
				&HoldReason::Deposit.into(),
				deposit.amount.saturated_into(),
			);

			// update the deposit
			Attestations::<T>::mutate(key, |details| {
				if let Some(d) = details {
					*d = AttestationDetails {
						deposit: Deposit {
							version: Some(1),
							owner: d.deposit.owner.clone(),
							amount: d.deposit.amount,
						},
						..attestations_detail
					};
				}
			});

			debug_assert!(
				result.is_ok(),
				" Attestation: Could not convert reserves to hold from attestation: {:?} error: {:?}",
				key,
				result
			);
		})
		.count()
}

#[cfg(test)]
pub mod test {
	use ctype::mock::get_ctype_hash;
	use frame_support::traits::{fungible::InspectHold, ReservableCurrency};
	use sp_runtime::traits::Zero;

	use crate::{migrations::do_migration, mock::*, AccountIdOf, Attestations, AttesterOf, Config, HoldReason};

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
			.build_and_execute_with_sanity_tests(true, || {
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

				do_migration::<Test>(ACCOUNT_00, 1);

				let attestation_post_migration = Attestations::<Test>::get(claim_hash);

				let balance_on_reserve_post_migration = <<Test as Config>::Currency as ReservableCurrency<
					AccountIdOf<Test>,
				>>::reserved_balance(&ACCOUNT_00);

				let balance_on_hold = <<Test as Config>::Currency as InspectHold<AccountIdOf<Test>>>::balance_on_hold(
					&HoldReason::Deposit.into(),
					&ACCOUNT_00,
				);

				//attestations should be still in the storage
				assert!(attestation_post_migration.is_some());

				// Since reserved balance count to hold balance, it should not be zero
				assert!(!balance_on_reserve_post_migration.is_zero());

				// ... and be as much as the hold balance
				assert_eq!(balance_on_reserve_post_migration, balance_on_hold);

				//... and the version should be 1.
				assert!(attestation_post_migration.clone().unwrap().deposit.version.is_some());
				assert!(attestation_post_migration.unwrap().deposit.version.unwrap() == 1);
			});
	}
}
