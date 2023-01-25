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

use crate::{
	linkable_account::LinkableAccountId, migration_state::MigrationState, AccountIdOf, Config,
	ConnectedAccounts as ConnectedAccountsV2, ConnectedDids as ConnectedDidsV2, ConnectionRecordOf, DidIdentifierOf,
	Error, MigrationStateStore, Pallet,
};

use frame_support::{
	ensure, storage_alias,
	traits::{Get, GetStorageVersion, OnRuntimeUpgrade},
	Blake2_128Concat,
};
use sp_runtime::DispatchError;
use sp_std::{marker::PhantomData, vec, vec::Vec};

#[cfg(feature = "runtime-benchmarks")]
use {
	crate::CurrencyOf, frame_support::traits::ReservableCurrency, kilt_support::deposit::Deposit,
	sp_runtime::DispatchResult,
};

/// Keytype changed from `AccountId` to `LinkableAccountId` changed in V3
#[storage_alias]
type ConnectedDids<T: Config> = StorageMap<Pallet<T>, Blake2_128Concat, AccountIdOf<T>, ConnectionRecordOf<T>>;
/// Second keytype changed from `AccountId` to `LinkableAccountId` changed in V3
#[storage_alias]
type ConnectedAccounts<T: Config> =
	StorageDoubleMap<Pallet<T>, Blake2_128Concat, DidIdentifierOf<T>, Blake2_128Concat, AccountIdOf<T>, ()>;

/// Migrate the `ConnectedDids` and `ConnectedAccounts` key types for a given
/// `AccountId`.
pub(crate) fn do_migrate_account_id<T: Config>(
	account_id: AccountIdOf<T>,
	linkable_account: LinkableAccountId,
) -> Result<Option<DidIdentifierOf<T>>, crate::pallet::Error<T>> {
	ConnectedDids::<T>::take(&account_id)
		.map(|did_record| {
			ConnectedDidsV2::<T>::insert(&linkable_account, did_record.clone());
			if let Some(v) = ConnectedAccounts::<T>::take(&did_record.did, &account_id) {
				ConnectedAccountsV2::<T>::insert(&did_record.did, linkable_account, v);
				Ok(did_record.did)
			} else {
				Err(crate::Error::<T>::MigrationIssue)
			}
		})
		.transpose()
}

/// Iterates over both old typed storage maps `ConnectedDids`,
/// `ConnectedAccounts` and checks whether any raw storage key still exists in
/// the low level storage.
pub(crate) fn do_verify_migration<T: Config>() -> Result<(), DispatchError> {
	check_storage_size::<T>()?;

	ensure!(
		check_did_migration::<T>(None).is_empty() && check_account_migration::<T>(None).is_empty(),
		Error::<T>::MigrationKeysPersist
	);

	Ok(())
}

/// Sanity check that both new typed storage maps `ConnectedDids`,
/// `ConnectedAccounts` have as many keys as their old counter parts with
/// different key types.
pub(crate) fn check_storage_size<T: Config>() -> Result<(), DispatchError> {
	ensure!(
		ConnectedAccounts::<T>::iter_keys().count() == ConnectedAccountsV2::<T>::iter_keys().count(),
		Error::<T>::MigrationStorageSizeMismatch
	);
	ensure!(
		ConnectedDids::<T>::iter_keys().count() == ConnectedDidsV2::<T>::iter_keys().count(),
		Error::<T>::MigrationStorageSizeMismatch
	);

	Ok(())
}

/// Iterates over old connected did storage map and checks whether any raw key
/// still exists in the low level storage.
///
/// Since the new `ConnectedDidsV2` and old `ConnectedDids` typed storage maps
/// have the same pallet and storage prefixes, both result in the same final
/// storage map key. For some reason, keys in the new map can still be iterated
/// over in the old one. E.g., the new keytype `LinkableAccountId` can be
/// decoded into the old one `AccountId32` such that both maps have the same
/// number of keys despite killing every old key during the migration.
///
/// However, we can check the old raw keys which should not exist in storage
/// after migrating, e.g. `unhashed::exists(old_raw_key)` is expected to be
/// false.
pub(crate) fn check_did_migration<T: Config>(maybe_last_key: Option<Vec<u8>>) -> Vec<(AccountIdOf<T>, Vec<u8>)> {
	if let Some(last_key) = maybe_last_key {
		log::debug!("Resuming check_did_migration from last_key: {:?}", last_key);
		ConnectedDids::<T>::iter_keys_from(last_key)
	} else {
		log::debug!("First check of ConnectedDids: {:?}", ConnectedDids::<T>::iter().count());
		ConnectedDids::<T>::iter_keys()
	}
	.filter_map(|acc_id| {
		let key = ConnectedDids::<T>::hashed_key_for(&acc_id);
		if frame_support::storage::unhashed::exists(key.as_ref()) {
			Some((acc_id, key))
		} else {
			None
		}
	})
	.collect()
}

/// Iterates over old connected account storage map and checks whether any raw
/// key still exists in the low level storage.
///
/// Since the new `ConnectedAccountsV2` and old `ConnectedAccounts` typed
/// storage maps have the same pallet and storage prefixes, both result in the
/// same final storage map key. For some reason, keys in the new map can still
/// be iterated over in the old one. E.g., the new keytype `(DidIdentifier,
/// LinkableAccountId)` can be decoded into the old one `(DidIdentifier,
/// AccountId)` such that both maps have the same number of keys despite killing
/// every old key during the migration.
///
/// However, we can check the old raw keys which should not exist in storage
/// after migrating, e.g. `unhashed::exists(old_raw_key)` is expected to be
/// false.
pub(crate) fn check_account_migration<T: Config>(maybe_last_key: Option<Vec<u8>>) -> Vec<(AccountIdOf<T>, Vec<u8>)> {
	if let Some(last_key) = maybe_last_key {
		log::debug!("Resuming check_account_migration from last_key: {:?}", last_key);
		ConnectedAccounts::<T>::iter_keys_from(last_key)
	} else {
		log::debug!(
			"First check of ConnectedAccounts: {:?}",
			ConnectedAccounts::<T>::iter().count()
		);
		ConnectedAccounts::<T>::iter_keys()
	}
	.filter_map(|(did_id, acc_id)| {
		let key = ConnectedAccounts::<T>::hashed_key_for(&did_id, &acc_id);
		if frame_support::storage::unhashed::exists(key.as_ref()) {
			Some((acc_id, key))
		} else {
			None
		}
	})
	.collect()
}

pub struct EthereumMigration<T>(PhantomData<T>);

impl<T: crate::pallet::Config> OnRuntimeUpgrade for EthereumMigration<T>
where
	T::AccountId: Into<LinkableAccountId>,
{
	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		if Pallet::<T>::on_chain_storage_version() == Pallet::<T>::current_storage_version() {
			// already on version 3
			<T as frame_system::Config>::DbWeight::get().reads_writes(1, 0)
		} else {
			log::info!("🔎 DidLookup: Initiating migration");
			MigrationStateStore::<T>::set(MigrationState::Upgrading);
			// TODO: Do we want to migrate storage version inside verify_migration?
			Pallet::<T>::current_storage_version().put::<Pallet<T>>();

			T::DbWeight::get().reads_writes(1, 2)
		}
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		assert!(
			Pallet::<T>::on_chain_storage_version() < Pallet::<T>::current_storage_version(),
			"On-chain storage of DID lookup pallet already bumped"
		);
		assert!(
			!MigrationStateStore::<T>::get().is_not_started(),
			"Migration flag already set"
		);

		log::info!("🔎 DidLookup: Pre migration checks successful");

		Ok(vec![])
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_pre_state: Vec<u8>) -> Result<(), &'static str> {
		assert_eq!(
			Pallet::<T>::on_chain_storage_version(),
			Pallet::<T>::current_storage_version(),
			"On-chain storage of DID lookup pallet was not bumped"
		);
		assert!(
			MigrationStateStore::<T>::get().is_in_progress(),
			"Migration flag was not set"
		);

		log::info!("🔎 DidLookup: Post migration checks successful");

		Ok(())
	}
}

#[cfg(feature = "runtime-benchmarks")]
pub(crate) fn add_legacy_association<T: Config>(
	sender: AccountIdOf<T>,
	did_identifier: DidIdentifierOf<T>,
	account: AccountIdOf<T>,
) -> DispatchResult {
	let deposit = Deposit {
		owner: sender,
		amount: T::Deposit::get(),
	};
	let record = crate::ConnectionRecord {
		deposit,
		did: did_identifier.clone(),
	};

	CurrencyOf::<T>::reserve(&record.deposit.owner, record.deposit.amount)?;

	ConnectedDids::<T>::mutate(&account, |did_entry| {
		if let Some(old_connection) = did_entry.replace(record) {
			ConnectedAccounts::<T>::remove(&old_connection.did, &account);
			kilt_support::free_deposit::<AccountIdOf<T>, CurrencyOf<T>>(&old_connection.deposit);
		}
	});
	ConnectedAccounts::<T>::insert(&did_identifier, &account, ());

	Ok(())
}

#[cfg(test)]
mod tests {
	use kilt_support::{deposit::Deposit, mock::SubjectId};

	use super::*;
	use crate::{mock::*, BalanceOf, ConnectionRecord, CurrencyOf, Error};
	use frame_support::{assert_noop, assert_ok, traits::ReservableCurrency};
	use sp_runtime::traits::Zero;

	pub(crate) fn insert_raw_connection<T: Config>(
		sender: AccountIdOf<T>,
		did_identifier: DidIdentifierOf<T>,
		account: AccountIdOf<T>,
		deposit: BalanceOf<T>,
	) {
		let deposit = Deposit {
			owner: sender,
			amount: deposit,
		};
		let record = ConnectionRecord {
			deposit,
			did: did_identifier.clone(),
		};

		CurrencyOf::<T>::reserve(&record.deposit.owner, record.deposit.amount)
			.expect("Account should have enough balance");

		ConnectedDids::<T>::mutate(&account, |did_entry| {
			if let Some(old_connection) = did_entry.replace(record) {
				ConnectedAccounts::<T>::remove(&old_connection.did, &account);
				kilt_support::free_deposit::<AccountIdOf<T>, CurrencyOf<T>>(&old_connection.deposit);
			}
		});
		ConnectedAccounts::<T>::insert(&did_identifier, &account, ());
	}

	#[test]
	fn single_account_migration_works() {
		ExtBuilder::default()
			.with_balances(vec![
				(ACCOUNT_00, <Test as crate::Config>::Deposit::get() * 50),
				(ACCOUNT_01, <Test as crate::Config>::Deposit::get() * 50),
			])
			.build()
			.execute_with(|| {
				insert_raw_connection::<Test>(
					ACCOUNT_00,
					DID_00,
					ACCOUNT_00,
					<Test as crate::Config>::Deposit::get() * 2,
				);

				// Check pre migration status (one entry not migrated, zero migrated entries)
				assert_eq!(ConnectedDids::<Test>::get(ACCOUNT_00).unwrap().did, DID_00);
				assert_eq!(ConnectedDids::<Test>::iter_keys().count(), 1);
				assert!(ConnectedDidsV2::<Test>::iter_keys().count().is_zero());
				let did_check_pre = check_did_migration::<Test>(None);
				assert_eq!(did_check_pre.len(), 1);
				assert_eq!(did_check_pre.get(0).unwrap().0, ACCOUNT_00);

				assert!(ConnectedAccounts::<Test>::contains_key(DID_00, ACCOUNT_00));
				assert_eq!(ConnectedAccounts::<Test>::iter_keys().count(), 1);
				assert!(ConnectedAccountsV2::<Test>::iter_keys().count().is_zero());
				let account_check_pre = check_account_migration::<Test>(None);
				assert_eq!(account_check_pre.len(), 1);
				assert_eq!(account_check_pre.get(0).unwrap().0, ACCOUNT_00);

				assert_noop!(
					DidLookup::try_finalize_migration(RuntimeOrigin::signed(ACCOUNT_00), 4),
					Error::<Test>::MigrationStorageSizeMismatch
				);

				// Migrate
				assert_ok!(DidLookup::migrate_account_id(
					RuntimeOrigin::signed(ACCOUNT_01),
					ACCOUNT_00
				));
				assert_ok!(DidLookup::try_finalize_migration(RuntimeOrigin::signed(ACCOUNT_00), 4));

				// Check post migration status
				assert!(check_did_migration::<Test>(None).is_empty());
				// This would fail since decoding magically works:
				// assert!(ConnectedDids::<Test>::iter_keys().count().is_zero());
				assert!(!ConnectedDids::<Test>::contains_key(ACCOUNT_00));
				assert_eq!(ConnectedDidsV2::<Test>::get(LINKABLE_ACCOUNT_00).unwrap().did, DID_00);
				assert_eq!(ConnectedDidsV2::<Test>::iter_keys().count(), 1);

				assert!(check_account_migration::<Test>(None).is_empty());
				// This would fail since decoding magically works:
				// assert!(ConnectedAccounts::<Test>::iter_keys().count().is_zero());
				assert!(!ConnectedAccounts::<Test>::contains_key(DID_00, ACCOUNT_00));
				assert!(ConnectedAccountsV2::<Test>::contains_key(DID_00, LINKABLE_ACCOUNT_00));
				assert_eq!(ConnectedAccountsV2::<Test>::iter_keys().count(), 1);
			})
	}

	#[test]
	fn multiple_account_migration_works() {
		ExtBuilder::default()
			.with_balances(vec![
				(ACCOUNT_00, <Test as crate::Config>::Deposit::get() * 50),
				(ACCOUNT_01, <Test as crate::Config>::Deposit::get() * 50),
			])
			.build()
			.execute_with(|| {
				insert_raw_connection::<Test>(
					ACCOUNT_00,
					DID_00,
					ACCOUNT_00,
					<Test as crate::Config>::Deposit::get() * 2,
				);
				insert_raw_connection::<Test>(
					ACCOUNT_01,
					DID_01,
					ACCOUNT_01,
					<Test as crate::Config>::Deposit::get() * 2,
				);

				// Check pre migration status
				assert_eq!(ConnectedDids::<Test>::iter_keys().count(), 2);
				assert!(ConnectedDidsV2::<Test>::iter_keys().count().is_zero());
				let did_check = check_did_migration::<Test>(None);
				assert_eq!(did_check.len(), 2);

				// Check iteration from first raw_key
				let did_check_cached =
					check_did_migration::<Test>(did_check.get(0).map(|(_, raw_key)| raw_key.clone()));
				assert_eq!(did_check_cached.len(), 1);
				assert_eq!(did_check.get(1), did_check_cached.get(0));

				assert_eq!(ConnectedAccounts::<Test>::iter_keys().count(), 2);
				assert!(ConnectedAccountsV2::<Test>::iter_keys().count().is_zero());
				assert_eq!(check_account_migration::<Test>(None).len(), 2);
				assert_noop!(
					DidLookup::try_finalize_migration(RuntimeOrigin::signed(ACCOUNT_00), 4),
					Error::<Test>::MigrationStorageSizeMismatch
				);

				// Migrate 1/2
				assert_ok!(DidLookup::migrate_account_id(
					RuntimeOrigin::signed(ACCOUNT_01),
					ACCOUNT_00
				));
				assert_noop!(
					DidLookup::try_finalize_migration(RuntimeOrigin::signed(ACCOUNT_00), 4),
					Error::<Test>::MigrationStorageSizeMismatch
				);
				assert_eq!(ConnectedDidsV2::<Test>::iter_keys().count(), 1);
				assert_eq!(check_did_migration::<Test>(None).len(), 1);
				assert_eq!(ConnectedAccountsV2::<Test>::iter_keys().count(), 1);
				assert_eq!(check_account_migration::<Test>(None).len(), 1);

				// Migrate 2/2
				assert_ok!(DidLookup::migrate_account_id(
					RuntimeOrigin::signed(ACCOUNT_00),
					ACCOUNT_01
				));
				assert_ok!(DidLookup::try_finalize_migration(RuntimeOrigin::signed(ACCOUNT_00), 4));

				// Check post migration status
				assert!(check_did_migration::<Test>(None).is_empty());
				assert_eq!(ConnectedDidsV2::<Test>::iter_keys().count(), 2);
				assert!(check_account_migration::<Test>(None).is_empty());
				assert_eq!(ConnectedAccountsV2::<Test>::iter_keys().count(), 2);
			})
	}

	#[test]
	fn check_did_account_migration_works() {
		let accounts: Vec<AccountId> = (0u8..10u8).map(|i| AccountId::new([i; 32])).collect();
		let dids: Vec<DidIdentifierOf<Test>> = accounts.clone().into_iter().map(SubjectId).collect();

		ExtBuilder::default()
			.with_balances(
				accounts
					.clone()
					.into_iter()
					.map(|acc| (acc, <Test as crate::Config>::Deposit::get() * 50))
					.collect(),
			)
			.build()
			.execute_with(|| {
				for i in 0..accounts.len() {
					insert_raw_connection::<Test>(
						accounts[i].clone(),
						dids[i].clone(),
						accounts[i].clone(),
						<Test as crate::Config>::Deposit::get() * 2,
					);
				}

				let check_did_uncashed = check_did_migration::<Test>(None);
				// iterate over i+j steps where i+j < checked_did_uncashed.len(), e.g. for j =
				// 0, it checks step by step.
				for i in 0..check_did_uncashed.len() {
					for j in 0..check_did_uncashed.len() - i {
						let index = i + j;
						let last_raw_key = check_did_uncashed.get(index).map(|(_, k)| k.clone());
						let check_did_cashed = check_did_migration::<Test>(last_raw_key);
						assert_eq!(check_did_cashed.len(), 10 - 1 - index, "i {}, j {}", i, j);
						assert_eq!(
							check_did_cashed.get(0),
							check_did_uncashed.get(index + 1),
							"i {}, j {}",
							i,
							j
						);
					}
				}

				let check_acc_uncashed = check_account_migration::<Test>(None);
				// iterate over i+j steps where i+j < checked_did_uncashed.len(), e.g. for j =
				// 0, it checks step by step.
				for i in 0..check_acc_uncashed.len() {
					for j in 0..check_acc_uncashed.len() - i {
						let index = i + j;
						let last_raw_key = check_acc_uncashed.get(index).map(|(_, k)| k.clone());
						let check_did_cashed = check_account_migration::<Test>(last_raw_key);
						assert_eq!(check_did_cashed.len(), 10 - 1 - index, "i {}, j {}", i, j);
						assert_eq!(
							check_did_cashed.get(0),
							check_acc_uncashed.get(index + 1),
							"i {}, j {}",
							i,
							j
						);
					}
				}
			})
	}
}
