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

#[cfg(feature = "try-runtime")]
use crate::{ConnectedAccounts as ConnectedAccountsV2, ConnectedDids as ConnectedDidsV2};

use codec::Encode;
use frame_support::{
	migration::move_prefix,
	storage::{storage_prefix, unhashed},
	storage_alias,
	traits::{Get, GetStorageVersion, OnRuntimeUpgrade, PalletInfoAccess, StorageVersion},
	Blake2_128Concat,
};
use sp_std::marker::PhantomData;

#[cfg(feature = "try-runtime")]
use frame_support::traits::OnRuntimeUpgradeHelpersExt;
#[cfg(feature = "try-runtime")]
use sp_runtime::SaturatedConversion;

#[storage_alias]
type ConnectedDids<T: Config> = StorageMap<Pallet<T>, Blake2_128Concat, AccountIdOf<T>, ConnectionRecordOf<T>>;
#[storage_alias]
type ConnectedAccounts<T: Config> =
	StorageDoubleMap<Pallet<T>, Blake2_128Concat, DidIdentifierOf<T>, Blake2_128Concat, AccountIdOf<T>, ()>;
#[storage_alias]
type TmpConnectedDids<T: Config> = StorageMap<Pallet<T>, Blake2_128Concat, LinkableAccountId, ConnectionRecordOf<T>>;
#[storage_alias]
type TmpConnectedAccounts<T: Config> =
	StorageDoubleMap<Pallet<T>, Blake2_128Concat, DidIdentifierOf<T>, Blake2_128Concat, LinkableAccountId, ()>;

// Inspired by frame_support::storage::migration::move_storage_from_pallet
fn move_storage<P: PalletInfoAccess>(old_storage_name: &[u8], new_storage_name: &[u8]) {
	let pallet_name = <P as PalletInfoAccess>::name();

	let old_prefix = storage_prefix(pallet_name.as_bytes(), old_storage_name);
	let new_prefix = storage_prefix(pallet_name.as_bytes(), new_storage_name);
	move_prefix(&old_prefix, &new_prefix);

	if let Some(value) = unhashed::get_raw(&old_prefix) {
		unhashed::put_raw(&new_prefix, &value);
		unhashed::kill(&old_prefix);
	}
}

pub struct EthereumMigration<T>(PhantomData<T>);

impl<T: crate::pallet::Config> OnRuntimeUpgrade for EthereumMigration<T>
where
	T::AccountId: Into<LinkableAccountId>,
{
	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		if Pallet::<T>::current_storage_version() == StorageVersion::new(3) {
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
			ConnectedDids::<T>::drain().for_each(|(acc_id32, value)| {
				log::debug!(
					"🔎 #{:?} Migrating ConnectedDid for account id {:?}",
					connected_dids.encode(),
					acc_id32.encode()
				);
				let acc_id: LinkableAccountId = acc_id32.into();
				TmpConnectedDids::<T>::insert(acc_id, value);
				connected_dids += 1;
			});
			log::info!("🔎 DidLookup: Migrated all {:?} ConnectedDids", connected_dids);

			// Migrate accounts
			ConnectedAccounts::<T>::drain().for_each(|(did_id, acc_id32, val)| {
				log::debug!(
					"🔎 #{:?} Migrating ConnectedAccount for did_id {:?} and account id {:?}",
					connected_accounts,
					did_id.encode(),
					acc_id32.encode()
				);
				let acc_id: LinkableAccountId = acc_id32.into();
				TmpConnectedAccounts::<T>::insert(did_id, acc_id, val);
				connected_accounts += 1;
			});
			log::info!("🔎 DidLookup: Migrated all {:?} ConnectedAccounts", connected_accounts);

			// Move TmpStorage
			move_storage::<Pallet<T>>(b"TmpConnectedDids", b"ConnectedDids");
			move_storage::<Pallet<T>>(b"TmpConnectedAccounts", b"ConnectedAccounts");

			StorageVersion::new(3).put::<Pallet<T>>();

			<T as frame_system::Config>::DbWeight::get().reads_writes(
				(connected_dids.saturating_add(connected_accounts)).saturating_mul(2),
				(connected_dids.saturating_add(connected_accounts))
					.saturating_mul(2)
					.saturating_add(1),
			)
		}
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		assert_eq!(Pallet::<T>::on_chain_storage_version(), 2);

		// Store number of connected DIDs in temp storage
		let connected_did_count: u64 = ConnectedDids::<T>::iter_keys().count().saturated_into();
		Self::set_temp_storage(connected_did_count, "pre_connected_did_count");
		log::info!(
			"🔎 DidLookup pre migration: Number of connected DIDs {:?}",
			connected_did_count
		);

		// Store number of connected accounts in temp storage
		let connected_account_count: u64 = ConnectedAccounts::<T>::iter_keys().count().saturated_into();
		Self::set_temp_storage(connected_account_count, "pre_connected_account_count");
		log::info!(
			"🔎 DidLookup pre migration: Number of connected accounts {:?}",
			connected_account_count
		);
		Ok(())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		assert_eq!(Pallet::<T>::on_chain_storage_version(), 3);

		// Check number of connected DIDs and accounts against pre-check result
		let pre_connected_did_count = Self::get_temp_storage("pre_connected_did_count").unwrap_or(0u64);
		let pre_connected_account_count = Self::get_temp_storage("pre_connected_account_count").unwrap_or(0u64);
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
