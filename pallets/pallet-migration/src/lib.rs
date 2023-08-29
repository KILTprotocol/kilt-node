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

#![cfg_attr(not(feature = "std"), no_std)]

pub use crate::default_weights::WeightInfo;
pub use pallet::*;

pub mod default_weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
#[cfg(any(test, feature = "runtime-benchmarks"))]
mod mock;
#[cfg(test)]
mod test;

#[frame_support::pallet]
pub mod pallet {
	use attestation::{Attestations, ClaimHashOf};
	use core::fmt::Debug;
	use delegation::{DelegationNodeIdOf, DelegationNodes};
	use did::{Did, DidIdentifierOf};
	use frame_support::{
		pallet_prelude::*,
		traits::{fungible::Inspect, Currency, ReservableCurrency},
	};
	use frame_system::pallet_prelude::*;
	use kilt_support::traits::MigrationManager;
	use pallet_did_lookup::{linkable_account::LinkableAccountId, ConnectedDids};
	use pallet_web3_names::{Owner, Web3NameOf};
	use public_credentials::{CredentialIdOf, Credentials, SubjectIdOf};
	use sp_runtime::{traits::Hash, DispatchError};
	use sp_std::vec::Vec;

	use crate::default_weights::WeightInfo;

	pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

	pub(crate) type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;

	pub type HashOf<T> = <T as frame_system::Config>::Hash;

	#[derive(Encode, Decode, TypeInfo, Debug, Clone, PartialEq)]
	pub struct EntriesToMigrate<T>
	where
		T: did::Config,
		T: delegation::Config,
		T: frame_system::Config,
		T: pallet_web3_names::Config,
		T: public_credentials::Config,
		T: Config,
	{
		pub attestation: BoundedVec<ClaimHashOf<T>, <T as Config>::MaxMigrationsPerPallet>,
		pub delegation: BoundedVec<DelegationNodeIdOf<T>, <T as Config>::MaxMigrationsPerPallet>,
		pub did: BoundedVec<DidIdentifierOf<T>, <T as Config>::MaxMigrationsPerPallet>,
		pub lookup: BoundedVec<LinkableAccountId, <T as Config>::MaxMigrationsPerPallet>,
		pub w3n: BoundedVec<Web3NameOf<T>, <T as Config>::MaxMigrationsPerPallet>,
		pub public_credentials: BoundedVec<(SubjectIdOf<T>, CredentialIdOf<T>), <T as Config>::MaxMigrationsPerPallet>,
	}

	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ attestation::Config
		+ delegation::Config
		+ did::Config
		+ pallet_did_lookup::Config
		+ pallet_web3_names::Config
		+ public_credentials::Config
		+ TypeInfo
	{
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The max amount on migrations for each pallet
		#[pallet::constant]
		type MaxMigrationsPerPallet: Get<u32>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;

		/// The currency module that takes care to release reserves
		type Currency: ReservableCurrency<AccountIdOf<Self>>;
	}

	#[pallet::error]
	pub enum Error<T> {
		KeyParse,
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::getter(fn connected_dids)]
	pub type MigratedKeys<T> = StorageMap<_, Blake2_128Concat, HashOf<T>, ()>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		EntriesUpdated(EntriesToMigrate<T>),
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		<T as attestation::Config>::Currency: ReservableCurrency<
			AccountIdOf<T>,
			Balance = <<T as attestation::Config>::Currency as Inspect<AccountIdOf<T>>>::Balance,
		>,
		<T as delegation::Config>::Currency: ReservableCurrency<
			AccountIdOf<T>,
			Balance = <<T as delegation::Config>::Currency as Inspect<AccountIdOf<T>>>::Balance,
		>,
		<T as did::Config>::Currency: ReservableCurrency<
			AccountIdOf<T>,
			Balance = <<T as did::Config>::Currency as Inspect<AccountIdOf<T>>>::Balance,
		>,
		<T as pallet_did_lookup::Config>::Currency: ReservableCurrency<
			AccountIdOf<T>,
			Balance = <<T as pallet_did_lookup::Config>::Currency as Inspect<AccountIdOf<T>>>::Balance,
		>,
		<T as pallet_web3_names::Config>::Currency: ReservableCurrency<
			AccountIdOf<T>,
			Balance = <<T as pallet_web3_names::Config>::Currency as Inspect<AccountIdOf<T>>>::Balance,
		>,
		<T as public_credentials::Config>::Currency: ReservableCurrency<
			AccountIdOf<T>,
			Balance = <<T as public_credentials::Config>::Currency as Inspect<AccountIdOf<T>>>::Balance,
		>,
	{
		#[pallet::call_index(0)]
		#[pallet::weight({
			//TODO: Placeholder.
			Weight::from_parts(1_000_000, 1_000_000)
		})]
		pub fn update_balance(origin: OriginFor<T>, requested_migrations: EntriesToMigrate<T>) -> DispatchResult {
			ensure_signed(origin)?;

			requested_migrations.attestation.iter().try_for_each(|key| {
				let is_migrated = Self::is_key_migrated(Attestations::<T>::hashed_key_for(key))?;
				if !is_migrated {
					attestation::migrations::update_balance_for_entry::<T>(key)
				} else {
					Ok(())
				}
			})?;

			requested_migrations.delegation.iter().try_for_each(|key| {
				let is_migrated = Self::is_key_migrated(DelegationNodes::<T>::hashed_key_for(key))?;
				if !is_migrated {
					delegation::migrations::update_balance_for_entry::<T>(key)
				} else {
					Ok(())
				}
			})?;

			requested_migrations.did.iter().try_for_each(|key| {
				let is_migrated = Self::is_key_migrated(Did::<T>::hashed_key_for(key))?;
				if !is_migrated {
					did::migrations::update_balance_for_entry::<T>(key)
				} else {
					Ok(())
				}
			})?;

			requested_migrations.lookup.iter().try_for_each(|key| {
				let is_migrated = Self::is_key_migrated(ConnectedDids::<T>::hashed_key_for(key))?;
				if !is_migrated {
					pallet_did_lookup::migrations::update_balance_for_entry::<T>(key)
				} else {
					Ok(())
				}
			})?;

			requested_migrations.w3n.iter().try_for_each(|key| {
				let is_migrated = Self::is_key_migrated(Owner::<T>::hashed_key_for(key))?;
				if !is_migrated {
					pallet_web3_names::migrations::update_balance_for_entry::<T>(key)
				} else {
					Ok(())
				}
			})?;

			requested_migrations
				.public_credentials
				.iter()
				.try_for_each(|(key, key2)| {
					let is_migrated = Self::is_key_migrated(Credentials::<T>::hashed_key_for(key, key2))?;
					if !is_migrated {
						public_credentials::migrations::update_balance_for_entry::<T>(key, key2)
					} else {
						Ok(())
					}
				})?;

			Self::deposit_event(Event::EntriesUpdated(requested_migrations));

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn is_key_migrated(key: Vec<u8>) -> Result<bool, DispatchError> {
			let key_hash = <T as frame_system::Config>::Hashing::hash(&key[..]);

			if MigratedKeys::<T>::contains_key(key_hash) {
				return Ok(true);
			}
			MigratedKeys::<T>::insert(key_hash, ());
			Ok(false)
		}
	}

	impl<T: Config> MigrationManager<AccountIdOf<T>, BalanceOf<T>> for Pallet<T> {
		fn exclude_key_from_migration(key: Vec<u8>) {
			let key_hash = <T as frame_system::Config>::Hashing::hash(&key[..]);
			MigratedKeys::<T>::insert(key_hash, ());
		}

		fn is_key_migrated(key: Vec<u8>) -> bool {
			let key_hash = <T as frame_system::Config>::Hashing::hash(&key[..]);
			MigratedKeys::<T>::contains_key(key_hash)
		}

		fn release_reserved_deposit(user: &AccountIdOf<T>, balance: &BalanceOf<T>) {
			<<T as Config>::Currency as ReservableCurrency<AccountIdOf<T>>>::unreserve(user, *balance);
		}
	}
}
