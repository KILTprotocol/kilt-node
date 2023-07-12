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

pub use pallet::*;

// #[cfg(feature = "runtime-benchmarks")] TODO
// mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		dispatch::DispatchResultWithPostInfo,
		pallet_prelude::*,
		traits::{LockableCurrency, ReservableCurrency},
		PalletError,
	};
	use frame_system::pallet_prelude::*;

	type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

	#[derive(Encode, Decode, TypeInfo, PalletError, Debug, Clone, PartialEq)]
	pub enum PalletToMigrate {
		Attestation,
		Delegation,
		Did,
		Lookup,
		W3n,
		Staking,
		Credentials,
	}

	#[pallet::config]
	pub trait Config:
		frame_system::Config
		+ attestation::Config
		+ delegation::Config
		+ did::Config
		+ pallet_did_lookup::Config
		+ pallet_web3_names::Config
		+ parachain_staking::Config
		+ public_credentials::Config
	{
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::event]
	pub enum Event<T: Config> {
		UserUpdated(T::AccountId),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Error if a migraion failes.
		Migration(PalletToMigrate),
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		<T as attestation::Config>::Currency: ReservableCurrency<AccountIdOf<T>>,
		<T as delegation::Config>::Currency: ReservableCurrency<AccountIdOf<T>>,
		<T as did::Config>::Currency: ReservableCurrency<AccountIdOf<T>>,
		<T as pallet_did_lookup::Config>::Currency: ReservableCurrency<AccountIdOf<T>>,
		<T as pallet_web3_names::Config>::Currency: ReservableCurrency<AccountIdOf<T>>,
		<T as parachain_staking::Config>::Currency: ReservableCurrency<AccountIdOf<T>>,
		<T as parachain_staking::Config>::Currency: LockableCurrency<AccountIdOf<T>>,
		<T as public_credentials::Config>::Currency: ReservableCurrency<AccountIdOf<T>>,
	{
		#[pallet::call_index(0)]
		#[pallet::weight(Weight::from_parts(10_000, 0))]
		pub fn update_users(
			origin: OriginFor<T>,
			user: T::AccountId,
			pallet_to_migrate: PalletToMigrate,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;

			match pallet_to_migrate {
				PalletToMigrate::Attestation => attestation::migrations::do_migration::<T>(user),
				PalletToMigrate::Delegation => delegation::migrations::do_migration::<T>(user),
				PalletToMigrate::Did => did::migrations::do_migration::<T>(user),
				PalletToMigrate::Lookup => pallet_did_lookup::migrations::do_migration::<T>(user),
				PalletToMigrate::W3n => pallet_web3_names::migrations::do_migration::<T>(user),
				PalletToMigrate::Staking => parachain_staking::migrations::do_migration::<T>(user),
				PalletToMigrate::Credentials => public_credentials::migrations::do_migration::<T>(user),
			}

			Ok(().into())
		}
	}
}
