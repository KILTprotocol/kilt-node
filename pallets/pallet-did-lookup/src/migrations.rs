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
	MigrationStateStore, Pallet,
};

use frame_support::{
	storage::KeyPrefixIterator,
	storage_alias,
	traits::{Get, GetStorageVersion, OnRuntimeUpgrade, StorageVersion},
	Blake2_128Concat, ReversibleStorageHasher, StoragePrefixedMap, LOG_TARGET,
};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{AccountId32, DispatchError};
use sp_std::marker::PhantomData;

use sp_std::vec::Vec;

/// Keytype changed from `AccountId` to `LinkableAccountId` changed in V3
#[storage_alias]
pub(crate) type ConnectedDids<T: Config> =
	StorageMap<Pallet<T>, Blake2_128Concat, AccountIdOf<T>, ConnectionRecordOf<T>>;
/// Second keytype changed from `AccountId` to `LinkableAccountId` changed in V3
#[storage_alias]
type ConnectedAccounts<T: Config> =
	StorageDoubleMap<Pallet<T>, Blake2_128Concat, DidIdentifierOf<T>, Blake2_128Concat, AccountIdOf<T>, ()>;

#[derive(Clone, Debug, Eq, PartialEq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum MixedStorageKey {
	V1(AccountId32),
	V2(LinkableAccountId),
}

/// Get an iterator that distinguishes between new and old storage keys.
///
/// This is only possible because old storage keys are 32 bytes long and new
/// storage keys either 21 or 33 bytes.
pub(crate) fn get_mixed_storage_iterator<T: Config>(
	previous_key: Option<Vec<u8>>,
) -> KeyPrefixIterator<MixedStorageKey> {
	let previous_key = previous_key.unwrap_or_else(|| ConnectedDids::<T>::final_prefix().to_vec());
	KeyPrefixIterator::new(
		ConnectedDids::<T>::final_prefix().to_vec(),
		previous_key,
		|raw_key_without_prefix| {
			let mut key_material = Blake2_128Concat::reverse(raw_key_without_prefix);
			match key_material.len() {
				// old keys are 32 bytes
				32 => Ok(MixedStorageKey::V1(AccountId32::decode(&mut key_material).map_err(
					|e| {
						log::warn!("Unable to decode V1 storage key");
						e
					},
				)?)),

				// new keys are 33 or 21 bytes
				33 | 21 => Ok(MixedStorageKey::V2(
					LinkableAccountId::decode(&mut key_material).map_err(|e| {
						log::warn!("Unable to decode V2 storage key");
						e
					})?,
				)),

				// This should not happen
				l => {
					log::error!("Unknown size {}", l);
					Err(parity_scale_codec::Error::from("Unknown key size"))
				}
			}
		},
	)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum MigrationProgress {
	Noop,
	ProcessedUntil(MixedStorageKey),
	Finished,
}

impl MigrationProgress {
	#[cfg(test)]
	fn last_processed_key(self) -> Option<MixedStorageKey> {
		match self {
			Self::ProcessedUntil(key) => Some(key),
			_ => None,
		}
	}
}

pub(crate) fn do_migrate<T: Config>(
	limit: u32,
	previous_key: Option<MixedStorageKey>,
) -> Result<MigrationProgress, DispatchError>
where
	T::AccountId: From<AccountId32>,
	T::AccountId: Into<AccountId32>,
{
	// convert MixedStorageKey into an actual raw storage key
	let raw_previous_key = previous_key.map(|mixed_key| match mixed_key {
		MixedStorageKey::V1(key) => ConnectedDids::<T>::hashed_key_for(T::AccountId::from(key)),
		MixedStorageKey::V2(key) => ConnectedDidsV2::<T>::hashed_key_for(key),
	});

	let mut key_iterator = get_mixed_storage_iterator::<T>(raw_previous_key);
	let mut new_previous_key = MigrationProgress::Noop;

	for _ in 0..limit {
		let mixed_key = if let Some(mixed_key) = key_iterator.next() {
			mixed_key
		} else {
			return Ok(MigrationProgress::Finished);
		};

		match mixed_key.clone() {
			MixedStorageKey::V1(acc) => {
				migrate_account_id::<T>(acc.into())?;
			}
			MixedStorageKey::V2(_) => {
				log::debug!("Skipping already migrated account link")
			}
		}
		new_previous_key = MigrationProgress::ProcessedUntil(mixed_key);
	}

	Ok(new_previous_key)
}

/// Migrate the `ConnectedDids` and `ConnectedAccounts` key types for a given
/// `AccountId`.
pub fn migrate_account_id<T: Config>(account_id: AccountIdOf<T>) -> Result<DidIdentifierOf<T>, DispatchError>
where
	T::AccountId: Into<AccountId32>,
{
	let linkable_account = LinkableAccountId::AccountId32(account_id.clone().into());

	// ConnectedDids -> remove v1 entry and add v2 entry
	let connection_record = ConnectedDids::<T>::take(&account_id).ok_or(crate::Error::<T>::Migration)?;
	ConnectedDidsV2::<T>::insert(&linkable_account, connection_record.clone());

	// ConnectedAccounts -> remove v1 entry and add v2 entry
	ConnectedAccounts::<T>::take(&connection_record.did, &account_id).ok_or(crate::Error::<T>::Migration)?;
	ConnectedAccountsV2::<T>::insert(&connection_record.did, linkable_account, ());

	Ok(connection_record.did)
}

pub struct EthereumMigration<T>(PhantomData<T>);

impl<T: crate::pallet::Config> OnRuntimeUpgrade for EthereumMigration<T>
where
	T::AccountId: Into<LinkableAccountId>,
{
	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		if Pallet::<T>::on_chain_storage_version() == StorageVersion::new(2) {
			log::info!("🔎 DidLookup: Initiating migration");
			MigrationStateStore::<T>::set(MigrationState::PreUpgrade);
			Pallet::<T>::current_storage_version().put::<Pallet<T>>();

			T::DbWeight::get().reads_writes(1, 2)
		} else {
			// wrong storage version
			log::info!(
				target: LOG_TARGET,
				"Migration did not execute. This probably should be removed"
			);
			<T as frame_system::Config>::DbWeight::get().reads_writes(1, 0)
		}
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		use sp_std::vec;

		assert_eq!(
			Pallet::<T>::on_chain_storage_version(),
			StorageVersion::new(2),
			"On-chain storage version should be 2 (last version without ethereum linking)"
		);
		assert_eq!(
			MigrationStateStore::<T>::get(),
			MigrationState::PreUpgrade,
			"Migration state already set"
		);

		log::info!(target: LOG_TARGET, "🔎 DidLookup: Pre migration checks successful");

		Ok(vec![])
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_pre_state: Vec<u8>) -> Result<(), &'static str> {
		assert_eq!(
			Pallet::<T>::on_chain_storage_version(),
			StorageVersion::new(3),
			"On-chain storage version should be updated"
		);
		assert!(
			MigrationStateStore::<T>::get().is_in_progress(),
			"Migration flag was not set"
		);

		log::info!(target: LOG_TARGET, "🔎 DidLookup: Post migration checks successful");

		Ok(())
	}
}

#[cfg(any(feature = "runtime-benchmarks", test))]
pub(crate) fn add_legacy_association<T: Config>(
	sender: AccountIdOf<T>,
	did_identifier: DidIdentifierOf<T>,
	account: AccountIdOf<T>,
	deposit: crate::BalanceOf<T>,
) {
	use crate::{ConnectionRecord, CurrencyOf};
	use frame_support::traits::ReservableCurrency;
	use kilt_support::deposit::Deposit;

	let deposit = Deposit {
		owner: sender,
		amount: deposit,
	};
	let record = ConnectionRecord {
		deposit,
		did: did_identifier.clone(),
	};

	CurrencyOf::<T>::reserve(&record.deposit.owner, record.deposit.amount).expect("Account should have enough balance");

	ConnectedDids::<T>::mutate(&account, |did_entry| {
		if let Some(old_connection) = did_entry.replace(record) {
			ConnectedAccounts::<T>::remove(&old_connection.did, &account);
			kilt_support::free_deposit::<AccountIdOf<T>, CurrencyOf<T>>(&old_connection.deposit);
		}
	});
	ConnectedAccounts::<T>::insert(&did_identifier, &account, ());
}

#[cfg(all(test, feature = "std"))]
mod tests {
	use frame_support::assert_ok;
	use kilt_support::deposit::Deposit;
	use test_log::test;

	use crate::{
		linkable_account::LinkableAccountId,
		mock::{generate_acc20, generate_acc32, generate_did, insert_raw_connection, ExtBuilder, Test},
		Config, ConnectionRecord,
	};

	use super::*;

	// test that we can iterate over storage that contains both old storage keys and
	// new storage keys
	#[test]
	fn should_distinguish_v1_v2() {
		let deposit_account = || generate_acc32(usize::MAX);

		ExtBuilder::default()
			.with_balances(vec![(deposit_account(), <Test as Config>::Deposit::get() * 50_000)])
			.build()
			.execute_with(|| {
				for i in 0..3 {
					add_legacy_association::<Test>(
						deposit_account(),
						generate_did(i),
						generate_acc32(i),
						<Test as Config>::Deposit::get(),
					);
				}

				for i in 3..7 {
					insert_raw_connection::<Test>(
						deposit_account(),
						generate_did(i),
						generate_acc20(i).into(),
						<Test as Config>::Deposit::get(),
					);
				}

				for i in 7..12 {
					insert_raw_connection::<Test>(
						deposit_account(),
						generate_did(i),
						generate_acc32(i).into(),
						<Test as Config>::Deposit::get(),
					);
				}

				assert_eq!(
					get_mixed_storage_iterator::<Test>(None).fold((0usize, 0usize, 0usize), |acc, key| match key {
						MixedStorageKey::V1(_) => (acc.0 + 1, acc.1, acc.2),
						MixedStorageKey::V2(LinkableAccountId::AccountId20(_)) => (acc.0, acc.1 + 1, acc.2),
						MixedStorageKey::V2(LinkableAccountId::AccountId32(_)) => (acc.0, acc.1, acc.2 + 1),
					}),
					(3usize, 4usize, 5usize),
					"The iterator should classify the keys correctly."
				);
			})
	}

	#[test]
	fn single_account_migration() {
		let deposit_account = || generate_acc32(usize::MAX);

		ExtBuilder::default()
			.with_balances(vec![(deposit_account(), <Test as Config>::Deposit::get() * 50_000)])
			.build()
			.execute_with(|| {
				add_legacy_association::<Test>(
					deposit_account(),
					generate_did(0),
					generate_acc32(0),
					<Test as Config>::Deposit::get(),
				);

				assert_eq!(
					get_mixed_storage_iterator::<Test>(None).fold((0usize, 0usize, 0usize), |acc, key| match key {
						MixedStorageKey::V1(_) => (acc.0 + 1, acc.1, acc.2),
						MixedStorageKey::V2(LinkableAccountId::AccountId20(_)) => (acc.0, acc.1 + 1, acc.2),
						MixedStorageKey::V2(LinkableAccountId::AccountId32(_)) => (acc.0, acc.1, acc.2 + 1),
					}),
					(1usize, 0usize, 0usize),
					"We should only have V1 keys"
				);

				assert_ok!(migrate_account_id::<Test>(generate_acc32(0)));

				assert_eq!(
					get_mixed_storage_iterator::<Test>(None).fold((0usize, 0usize, 0usize), |acc, key| match key {
						MixedStorageKey::V1(_) => (acc.0 + 1, acc.1, acc.2),
						MixedStorageKey::V2(LinkableAccountId::AccountId20(_)) => (acc.0, acc.1 + 1, acc.2),
						MixedStorageKey::V2(LinkableAccountId::AccountId32(_)) => (acc.0, acc.1, acc.2 + 1),
					}),
					(0usize, 0usize, 1usize),
					"We should only have V2 keys"
				);

				// The store should contain the expected version 2 storage entries
				assert_eq!(
					ConnectedAccountsV2::<Test>::get(
						generate_did(0),
						LinkableAccountId::AccountId32(generate_acc32(0))
					),
					Some(())
				);
				assert_eq!(
					ConnectedDidsV2::<Test>::get(LinkableAccountId::AccountId32(generate_acc32(0))),
					Some(ConnectionRecord {
						deposit: Deposit {
							amount: <Test as Config>::Deposit::get(),
							owner: deposit_account()
						},
						did: generate_did(0)
					})
				);

				// The version 1 storage entries should be removed
				assert_eq!(ConnectedDids::<Test>::get(generate_acc32(0)), None);
				assert_eq!(ConnectedAccounts::<Test>::get(generate_did(0), generate_acc32(0)), None);
			})
	}

	#[test]
	fn partial_migration() {
		let deposit_account = || generate_acc32(usize::MAX);

		ExtBuilder::default()
			.with_balances(vec![(deposit_account(), <Test as Config>::Deposit::get() * 50_000)])
			.build()
			.execute_with(|| {
				for i in 0..50 {
					add_legacy_association::<Test>(
						deposit_account(),
						generate_did(i),
						generate_acc32(i),
						<Test as Config>::Deposit::get(),
					);
				}
				assert_eq!(
					get_mixed_storage_iterator::<Test>(None).fold((0usize, 0usize, 0usize), |acc, key| match key {
						MixedStorageKey::V1(_) => (acc.0 + 1, acc.1, acc.2),
						MixedStorageKey::V2(LinkableAccountId::AccountId20(_)) => (acc.0, acc.1 + 1, acc.2),
						MixedStorageKey::V2(LinkableAccountId::AccountId32(_)) => (acc.0, acc.1, acc.2 + 1),
					}),
					(50usize, 0usize, 0usize),
					"We should only have V1 keys"
				);

				let previous_key = do_migrate::<Test>(10, None).expect("Migration must work");

				// Since we also iterate over already migrated keys, we don't get 10 migrated
				// accounts with a limit of 10.
				assert_eq!(
					get_mixed_storage_iterator::<Test>(None).fold((0usize, 0usize, 0usize), |acc, key| match key {
						MixedStorageKey::V1(_) => (acc.0 + 1, acc.1, acc.2),
						MixedStorageKey::V2(LinkableAccountId::AccountId20(_)) => (acc.0, acc.1 + 1, acc.2),
						MixedStorageKey::V2(LinkableAccountId::AccountId32(_)) => (acc.0, acc.1, acc.2 + 1),
					}),
					(40usize, 0usize, 10usize),
					"There should be migration progress"
				);

				let previous_key =
					do_migrate::<Test>(10, previous_key.last_processed_key()).expect("Migration must work");

				assert_eq!(
					get_mixed_storage_iterator::<Test>(None).fold((0usize, 0usize, 0usize), |acc, key| match key {
						MixedStorageKey::V1(_) => (acc.0 + 1, acc.1, acc.2),
						MixedStorageKey::V2(LinkableAccountId::AccountId20(_)) => (acc.0, acc.1 + 1, acc.2),
						MixedStorageKey::V2(LinkableAccountId::AccountId32(_)) => (acc.0, acc.1, acc.2 + 1),
					}),
					(30usize, 0usize, 20usize),
					"There should be migration progress"
				);

				assert_ok!(do_migrate::<Test>(10, previous_key.last_processed_key()));

				assert_eq!(
					get_mixed_storage_iterator::<Test>(None).fold((0usize, 0usize, 0usize), |acc, key| match key {
						MixedStorageKey::V1(_) => (acc.0 + 1, acc.1, acc.2),
						MixedStorageKey::V2(LinkableAccountId::AccountId20(_)) => (acc.0, acc.1 + 1, acc.2),
						MixedStorageKey::V2(LinkableAccountId::AccountId32(_)) => (acc.0, acc.1, acc.2 + 1),
					}),
					(22usize, 0usize, 28usize),
					"There should be migration progress"
				);
			})
	}
}
