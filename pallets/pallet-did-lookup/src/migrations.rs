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
	pallet_prelude::{DispatchResult, ValueQuery},
	storage_alias,
	traits::{Get, GetStorageVersion, OnRuntimeUpgrade, ReservableCurrency, StorageVersion},
};
use kilt_support::{migration::switch_reserved_to_hold, Deposit};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{AccountId32, SaturatedConversion};
use sp_std::marker::PhantomData;

use crate::{
	linkable_account::LinkableAccountId, AccountIdOf, Config, ConnectedDids, ConnectionRecord, CurrencyOf, Error,
	HoldReason, Pallet,
};

/// A unified log target for did-lookup-migration operations.
pub const LOG_TARGET: &str = "runtime::pallet-did-lookup::migrations";

#[derive(Clone, Debug, Eq, PartialEq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum MixedStorageKey {
	V1(AccountId32),
	V2(LinkableAccountId),
}

#[derive(Clone, Debug, Eq, PartialEq, Encode, Decode, MaxEncodedLen, TypeInfo, Default)]
pub enum MigrationState {
	/// The migration was successful.
	Done,

	/// The storage has still the old layout, the migration wasn't started yet
	#[default]
	PreUpgrade,

	/// The upgrade is in progress and did migrate all storage up to the
	/// `MixedStorageKey`.
	Upgrading(MixedStorageKey),
}

impl MigrationState {
	pub fn is_done(&self) -> bool {
		matches!(self, MigrationState::Done)
	}

	pub fn is_in_progress(&self) -> bool {
		!matches!(self, MigrationState::Done)
	}
}

#[storage_alias]
type MigrationStateStore<T: Config> = StorageValue<Pallet<T>, MigrationState, ValueQuery>;

pub struct CleanupMigration<T>(PhantomData<T>);

impl<T: crate::pallet::Config> OnRuntimeUpgrade for CleanupMigration<T> {
	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		if Pallet::<T>::on_chain_storage_version() == StorageVersion::new(3) {
			log::info!("🔎 DidLookup: Initiating migration");
			MigrationStateStore::<T>::kill();
			StorageVersion::new(4).put::<Pallet<T>>();

			T::DbWeight::get().reads_writes(1, 2)
		} else {
			// wrong storage version
			log::info!(
				target: LOG_TARGET,
				"Migration did not execute. This probably should be removed"
			);
			<T as frame_system::Config>::DbWeight::get().reads(1)
		}
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<sp_std::vec::Vec<u8>, &'static str> {
		use sp_std::vec;

		assert_eq!(
			Pallet::<T>::on_chain_storage_version(),
			StorageVersion::new(3),
			"On-chain storage version should be 3 before the migration"
		);
		assert!(MigrationStateStore::<T>::exists(), "Migration state should exist");

		log::info!(target: LOG_TARGET, "🔎 DidLookup: Pre migration checks successful");

		Ok(vec![])
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_pre_state: sp_std::vec::Vec<u8>) -> Result<(), &'static str> {
		assert_eq!(
			Pallet::<T>::on_chain_storage_version(),
			StorageVersion::new(4),
			"On-chain storage version should be updated"
		);
		assert!(!MigrationStateStore::<T>::exists(), "Migration state should be deleted");

		log::info!(target: LOG_TARGET, "🔎 DidLookup: Post migration checks successful");

		Ok(())
	}
}

pub fn update_balance_for_entry<T: Config>(key: &LinkableAccountId) -> DispatchResult
where
	<T as Config>::Currency: ReservableCurrency<T::AccountId>,
{
	ConnectedDids::<T>::try_mutate(key, |details| {
		if let Some(d) = details {
			*d = ConnectionRecord {
				deposit: Deposit {
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
	use frame_support::traits::{fungible::InspectHold, ReservableCurrency};
	use sp_runtime::traits::Zero;

	use crate::{migrations::update_balance_for_entry, mock::*, AccountIdOf, Config, ConnectedDids, HoldReason};

	#[test]
	fn test_setup() {
		ExtBuilder::default()
			.with_balances(vec![
				(ACCOUNT_00, <Test as crate::Config>::Deposit::get() * 50),
				(ACCOUNT_01, <Test as crate::Config>::Deposit::get() * 50),
			])
			.with_connections(vec![(ACCOUNT_00, DID_00, LINKABLE_ACCOUNT_00)])
			.build_and_execute_with_sanity_tests(|| {
				kilt_support::migration::translate_holds_to_reserve::<Test>(HoldReason::Deposit.into());

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
	fn test_balance_migration_did_lookup() {
		ExtBuilder::default()
			.with_balances(vec![
				(ACCOUNT_00, <Test as crate::Config>::Deposit::get() * 50),
				(ACCOUNT_01, <Test as crate::Config>::Deposit::get() * 50),
			])
			.with_connections(vec![(ACCOUNT_00, DID_00, LINKABLE_ACCOUNT_00)])
			.build_and_execute_with_sanity_tests(|| {
				kilt_support::migration::translate_holds_to_reserve::<Test>(HoldReason::Deposit.into());

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

				let connected_did_pre_migration = ConnectedDids::<Test>::get(LINKABLE_ACCOUNT_00);

				let balance_on_reserve_pre_migration = <<Test as Config>::Currency as ReservableCurrency<
					AccountIdOf<Test>,
				>>::reserved_balance(&ACCOUNT_00);

				//Delegation should be in storage
				assert!(connected_did_pre_migration.is_some());

				// before the migration the deposit should be reserved.
				assert_eq!(
					balance_on_reserve_pre_migration,
					connected_did_pre_migration.unwrap().deposit.amount
				);

				assert!(update_balance_for_entry::<Test>(&LINKABLE_ACCOUNT_00).is_ok());

				let connected_did_post_migration = ConnectedDids::<Test>::get(LINKABLE_ACCOUNT_00);

				let balance_on_reserve_post_migration = <<Test as Config>::Currency as ReservableCurrency<
					AccountIdOf<Test>,
				>>::reserved_balance(&ACCOUNT_00);

				let balance_on_hold = <<Test as Config>::Currency as InspectHold<AccountIdOf<Test>>>::balance_on_hold(
					&HoldReason::Deposit.into(),
					&ACCOUNT_00,
				);

				//Delegation should be still in the storage
				assert!(connected_did_post_migration.is_some());

				// Since reserved balance count to hold balance, it should not be zero
				assert!(!balance_on_reserve_post_migration.is_zero());

				// ... and be as much as the hold balance
				assert_eq!(balance_on_reserve_post_migration, balance_on_hold);

				// Nothing should happen
				assert!(update_balance_for_entry::<Test>(&LINKABLE_ACCOUNT_00).is_err());
			})
	}
}
