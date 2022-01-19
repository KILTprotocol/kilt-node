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

//! # Pallet storing unique nickname <-> DID links for user-friendly DID
//! nicknames.

#![cfg_attr(not(feature = "std"), no_std)]

mod types;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		pallet_prelude::*,
		traits::{Currency, ReservableCurrency, StorageVersion},
		Blake2_128Concat,
	};
	use frame_system::pallet_prelude::*;

	use kilt_support::{deposit::Deposit, traits::CallSources};

	use crate::types::UnickOwnership;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	pub(crate) type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
	pub(crate) type CurrencyOf<T> = <T as Config>::Currency;
	pub(crate) type DidIdentifierOf<T> = <T as Config>::DidIdentifier;
	pub(crate) type UnickOf<T> = <T as Config>::Unick;
	pub(crate) type UnickOwnershipOf<T> = UnickOwnership<DidIdentifierOf<T>, Deposit<AccountIdOf<T>, BalanceOf<T>>>;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	// Unick -> DID
	#[pallet::storage]
	#[pallet::getter(fn owner)]
	pub type Owner<T> = StorageMap<_, Blake2_128Concat, UnickOf<T>, UnickOwnershipOf<T>>;

	// DID -> Unick
	#[pallet::storage]
	#[pallet::getter(fn unicks)]
	pub type Unicks<T> = StorageMap<_, Twox64Concat, DidIdentifierOf<T>, UnickOf<T>>;

	// Unick -> ()
	#[pallet::storage]
	#[pallet::getter(fn is_blacklisted)]
	pub type Blacklist<T> = StorageMap<_, Blake2_128Concat, UnickOf<T>, ()>;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		#[pallet::constant]
		type Deposit: Get<BalanceOf<Self>>;
		type Currency: Currency<AccountIdOf<Self>> + ReservableCurrency<AccountIdOf<Self>>;
		type DidIdentifier: Parameter + Default;
		type FreezeOrigin: EnsureOrigin<Self::Origin>;
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		#[pallet::constant]
		type MaxUnickLength: Get<u32>;
		type OriginSuccess: CallSources<AccountIdOf<Self>, DidIdentifierOf<Self>>;
		type RegularOrigin: EnsureOrigin<Success = Self::OriginSuccess, <Self as frame_system::Config>::Origin>;
		type Unick: Parameter;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		UnickClaimed {
			owner: DidIdentifierOf<T>,
			unick: UnickOf<T>,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		InsufficientFunds,
		UnickAlreadyClaimed,
		OwnerAlreadyExisting,
		UnickBlacklisted,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		pub fn claim(origin: OriginFor<T>, unick: UnickOf<T>) -> DispatchResult {
			let origin = T::RegularOrigin::ensure_origin(origin)?;
			let payer = origin.sender();
			let owner = origin.subject();

			ensure!(
				<T::Currency as ReservableCurrency<AccountIdOf<T>>>::can_reserve(&payer, <T as Config>::Deposit::get()),
				Error::<T>::InsufficientFunds
			);

			Self::check_claiming_preconditions(&unick, &owner)?;

			// No failure beyond this point

			Self::register_unick(unick.clone(), owner.clone(), payer);
			Self::deposit_event(Event::<T>::UnickClaimed { owner, unick });

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn release_by_owner(_origin: OriginFor<T>, _unick: UnickOf<T>) -> DispatchResult {
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn release_by_payer(_origin: OriginFor<T>, _unick: UnickOf<T>) -> DispatchResult {
			Ok(())
		}

		#[pallet::weight(0)]
		pub fn blacklist(_origin: OriginFor<T>, _unick: UnickOf<T>) -> DispatchResult {
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn check_claiming_preconditions(unick: &UnickOf<T>, owner: &DidIdentifierOf<T>) -> Result<(), DispatchError> {
			ensure!(!Unicks::<T>::contains_key(&owner), Error::<T>::UnickAlreadyClaimed);
			ensure!(!Owner::<T>::contains_key(&unick), Error::<T>::OwnerAlreadyExisting);
			ensure!(!Blacklist::<T>::contains_key(&unick), Error::<T>::UnickBlacklisted);
			Ok(())
		}

		fn register_unick(unick: UnickOf<T>, owner: DidIdentifierOf<T>, deposit_payer: AccountIdOf<T>) {
			let deposit = Deposit {
				owner: deposit_payer,
				amount: T::Deposit::get(),
			};

			CurrencyOf::<T>::reserve(&deposit.owner, deposit.amount)?;

			Unicks::<T>::insert(&owner, unick);
			Owner::<T>::insert(&unick, UnickOwnershipOf::<T> { owner, deposit });
		}
	}
}
