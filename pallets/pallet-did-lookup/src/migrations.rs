// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2022 BOTLabs GmbH

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

use crate::{linkable_account::LinkableAccountId, AccountIdOf, Config, ConnectionRecordOf, DidIdentifierOf, Pallet};

use crate::{ConnectedAccounts as ConnectedAccountsV2, ConnectedDids as ConnectedDidsV2};

use frame_support::{
	storage_alias,
	traits::{Get, GetStorageVersion, OnRuntimeUpgrade},
	Blake2_128Concat,
};
use sp_std::{marker::PhantomData, vec};

#[cfg(feature = "try-runtime")]
use {
	frame_support::{
		codec::{Decode, Encode},
		inherent::Vec,
	},
	sp_runtime::SaturatedConversion,
};

#[storage_alias]
type ConnectedDids<T: Config> = StorageMap<Pallet<T>, Blake2_128Concat, AccountIdOf<T>, ConnectionRecordOf<T>>;
#[storage_alias]
type ConnectedAccounts<T: Config> =
	StorageDoubleMap<Pallet<T>, Blake2_128Concat, DidIdentifierOf<T>, Blake2_128Concat, AccountIdOf<T>, ()>;

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
			let mut connected_dids = 0u64;
			let mut connected_accounts = 0u64;

			// Migrate connected DIDs
			// We should not write to the same storage item during drain because it can lead
			// to undefined results. Thus, we write to a temporary storage and move that at
			// the end. Else we iterate over every key more or less twice.
			let mut connected_dids_buffer = vec![];
			for (acc_id32, value) in ConnectedDids::<T>::drain() {
				let acc_id: LinkableAccountId = acc_id32.into();
				connected_dids_buffer.push((acc_id, value));
				connected_dids = connected_dids.saturating_add(1);
			}
			for (acc_id, value) in &connected_dids_buffer {
				ConnectedDidsV2::<T>::insert(acc_id, value);
			}
			log::info!("🔎 DidLookup: Migrated all ConnectedDids");

			// Migrate accounts
			let mut connected_accounts_buffer = vec![];
			for (did_id, acc_id32, val) in ConnectedAccounts::<T>::drain() {
				let acc_id: LinkableAccountId = acc_id32.into();
				connected_accounts_buffer.push((did_id, acc_id, val));
				connected_accounts = connected_accounts.saturating_add(1);
			}
			for (did_id, acc_id, val) in &connected_accounts_buffer {
				ConnectedAccountsV2::<T>::insert(did_id, acc_id, val);
			}
			log::info!("🔎 DidLookup: Migrated all ConnectedAccounts");

			Pallet::<T>::current_storage_version().put::<Pallet<T>>();

			<T as frame_system::Config>::DbWeight::get().reads_writes(
				// read every entry in ConnectedDids and ConnectedAccounts
				connected_dids
					.saturating_add(connected_accounts)
					// read the storage version
					.saturating_add(1),
				// for every storage entry remove the old + put the new entries
				(connected_dids.saturating_add(connected_accounts))
					.saturating_mul(2)
					// +1 for updating the storage version
					.saturating_add(1),
			)
		}
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
		assert!(Pallet::<T>::on_chain_storage_version() < Pallet::<T>::current_storage_version());

		// Store number of connected DIDs in temp storage
		let connected_did_count: u64 = ConnectedDids::<T>::iter_keys().count().saturated_into();
		log::info!(
			"🔎 DidLookup pre migration: Number of connected DIDs {:?}",
			connected_did_count
		);

		// Store number of connected accounts in temp storage
		let connected_account_count: u64 = ConnectedAccounts::<T>::iter_keys().count().saturated_into();
		log::info!(
			"🔎 DidLookup pre migration: Number of connected accounts {:?}",
			connected_account_count
		);
		Ok((connected_did_count, connected_account_count).encode())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(pre_state: Vec<u8>) -> Result<(), &'static str> {
		assert_eq!(
			Pallet::<T>::on_chain_storage_version(),
			Pallet::<T>::current_storage_version()
		);

		// Check number of connected DIDs and accounts against pre-check result
		let (pre_connected_did_count, pre_connected_account_count): (u64, u64) =
			Decode::decode(&mut pre_state.as_slice())
				.expect("the state parameter should be something that was generated by pre_upgrade");
		assert_eq!(
			ConnectedDidsV2::<T>::iter().count().saturated_into::<u64>(),
			pre_connected_did_count,
			"Number of connected DIDs does not match"
		);
		assert_eq!(
			ConnectedAccountsV2::<T>::iter_keys().count().saturated_into::<u64>(),
			pre_connected_account_count,
			"Number of connected accounts does not match"
		);
		log::info!("🔎 DidLookup: Post migration checks successful");

		Ok(())
	}
}
