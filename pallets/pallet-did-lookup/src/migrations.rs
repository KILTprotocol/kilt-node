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
	pallet_prelude::ValueQuery,
	storage_alias,
	traits::{Get, GetStorageVersion, OnRuntimeUpgrade, ReservableCurrency, StorageVersion},
	weights::Weight,
};
use kilt_support::{
	deposit::{HFIdentifier, Pallets},
	migration::{has_user_holds, switch_reserved_to_hold},
};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::AccountId32;
use sp_std::marker::PhantomData;

use crate::{
	linkable_account::LinkableAccountId, AccountIdOf, Config, ConnectedDids, ConnectionRecordOf, CurrencyOf, Pallet,
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

pub struct BalanceMigration<T>(PhantomData<T>);

impl<T: crate::pallet::Config> OnRuntimeUpgrade for BalanceMigration<T>
where
	<T as Config>::Currency: ReservableCurrency<T::AccountId>,
{
	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		log::info!("Did lookup: Initiating migration");
		if is_upgraded::<T>() {
			return do_migration::<T>();
		}
		log::info!("Did lookup: No migration needed. This file should be deleted.");
		<T as frame_system::Config>::DbWeight::get().reads_writes(0, 0)
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<sp_std::vec::Vec<u8>, &'static str> {
		use frame_support::ensure;
		use sp_std::vec;

		let has_all_user_no_holds = ConnectedDids::<T>::iter_values()
			.map(|details| {
				has_user_holds::<AccountIdOf<T>, CurrencyOf<T>>(
					&details.deposit.owner,
					&HFIdentifier::Deposit(Pallets::DidLookup),
				)
			})
			.all(|user| user);

		ensure!(
			has_all_user_no_holds,
			"Pre Upgrade Did lookup: there are users with holds!"
		);

		log::info!("Did: Pre migration checks successful");

		Ok(vec![])
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_pre_state: sp_std::vec::Vec<u8>) -> Result<(), &'static str> {
		use frame_support::{ensure, traits::fungible::InspectHold};
		use kilt_support::test_utils::log_and_return_error_message;
		use sp_runtime::SaturatedConversion;

		ConnectedDids::<T>::iter().try_for_each(|(key, details)| -> Result<(), &'static str> {
			let hold_balance: u128 = <T as Config>::Currency::balance_on_hold(
				&HFIdentifier::Deposit(Pallets::DidLookup),
				&details.deposit.owner,
			)
			.saturated_into();
			ensure!(
				details.deposit.amount.saturated_into::<u128>() <= hold_balance,
				log_and_return_error_message(scale_info::prelude::format!(
					"Did lookup: Hold balance is not matching for connected did {:?}. Expected hold: {:?}. Real hold: {:?}",
					key, details.deposit.amount, hold_balance
				))
			);

			ensure!(!is_upgraded::<T>(), "Did lookup: Users have still no holds");

			Ok(())
		})?;
		log::info!("Did lookup: Post migration checks successful");
		Ok(())
	}
}

/// Checks if there is an user, who has still reserved balance and no holds. If
/// yes, the migration is not executed yet.
fn is_upgraded<T: Config>() -> bool
where
	<T as Config>::Currency: ReservableCurrency<T::AccountId>,
{
	ConnectedDids::<T>::iter_values()
		.map(|details: ConnectionRecordOf<T>| {
			has_user_holds::<AccountIdOf<T>, CurrencyOf<T>>(
				&details.deposit.owner,
				&HFIdentifier::Deposit(Pallets::DidLookup),
			)
		})
		.all(|user| user)
}

fn do_migration<T: Config>() -> Weight
where
	<T as Config>::Currency: ReservableCurrency<T::AccountId>,
{
	ConnectedDids::<T>::iter()
		.map(|(key, did_details)| -> Weight {
			let deposit = did_details.deposit;
			let error = switch_reserved_to_hold::<AccountIdOf<T>, CurrencyOf<T>>(
				deposit.owner,
				&HFIdentifier::Deposit(Pallets::DidLookup),
				deposit.amount,
			);

			if error.is_ok() {
				return <T as frame_system::Config>::DbWeight::get().reads_writes(1, 1);
			}

			log::error!(
				"Did lookup: Could not convert reserves to hold from connected did: {:?} ",
				key,
			);

			<T as frame_system::Config>::DbWeight::get().reads_writes(0, 0)
		})
		.fold(Weight::zero(), |acc, next| acc.saturating_add(next))
}
