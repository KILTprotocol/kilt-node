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
mod utils;

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

	use crate::{types::UnickOwnership, utils::validate_unick};

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	pub(crate) type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	pub(crate) type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
	pub(crate) type CurrencyOf<T> = <T as Config>::Currency;
	pub(crate) type DidIdentifierOf<T> = <T as Config>::DidIdentifier;
	pub(crate) type UnickOf<T> = <T as Config>::Unick;
	pub(crate) type UnickOwnershipOf<T> = UnickOwnership<DidIdentifierOf<T>, Deposit<AccountIdOf<T>, BalanceOf<T>>>;

	enum UnickReleaseCaller<'a, 'b, T: Config> {
		DepositPayer(&'a AccountIdOf<T>),
		UnickOwner(&'b DidIdentifierOf<T>),
	}

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
		type BlacklistOrigin: EnsureOrigin<Self::Origin>;
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
		UnickReleased {
			owner: DidIdentifierOf<T>,
			unick: UnickOf<T>,
		},
		UnickBlacklisted {
			unick: UnickOf<T>,
		},
		UnickUnblacklisted {
			unick: UnickOf<T>,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		InsufficientFunds,
		UnickAlreadyClaimed,
		UnickNotFound,
		OwnerAlreadyExisting,
		UnickBlacklisted,
		UnickNotBlacklisted,
		NotAuthorized,
		UnickAlreadyBlacklisted,
		InternalError,
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

			Self::check_claiming_preconditions(&unick, &owner, &payer)?;

			// No failure beyond this point

			Self::register_unick_unsafe(unick.clone(), owner.clone(), payer);
			Self::deposit_event(Event::<T>::UnickClaimed { owner, unick });

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn release_by_owner(origin: OriginFor<T>, unick: UnickOf<T>) -> DispatchResult {
			let origin = T::RegularOrigin::ensure_origin(origin)?;
			let owner = origin.subject();

			Self::check_releasing_preconditions(&unick, UnickReleaseCaller::UnickOwner(&owner))?;

			// No failure beyond this point

			Self::unregister_unick_unsafe(&unick);
			Self::deposit_event(Event::<T>::UnickReleased { owner, unick });

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn release_by_payer(origin: OriginFor<T>, unick: UnickOf<T>) -> DispatchResult {
			let caller = ensure_signed(origin)?;

			Self::check_releasing_preconditions(&unick, UnickReleaseCaller::DepositPayer(&caller))?;

			// No failure beyond this point

			let UnickOwnershipOf::<T> { owner, .. } = Self::unregister_unick_unsafe(&unick);
			Self::deposit_event(Event::<T>::UnickReleased { owner, unick });

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn blacklist(origin: OriginFor<T>, unick: UnickOf<T>) -> DispatchResult {
			T::BlacklistOrigin::ensure_origin(origin)?;

			Self::check_blacklisting_preconditions(&unick)?;

			// No failure beyond this point

			Self::blacklist_unick_unsafe(&unick);
			Self::deposit_event(Event::<T>::UnickBlacklisted { unick });

			Ok(())
		}

		#[pallet::weight(0)]
		pub fn unblacklist(origin: OriginFor<T>, unick: UnickOf<T>) -> DispatchResult {
			T::BlacklistOrigin::ensure_origin(origin)?;

			Self::check_unblacklisting_preconditions(&unick)?;

			// No failure beyond this point

			Self::unblacklist_unick_unsafe(&unick);
			Self::deposit_event(Event::<T>::UnickUnblacklisted { unick });

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn check_claiming_preconditions(
			unick: &UnickOf<T>,
			owner: &DidIdentifierOf<T>,
			deposit_payer: &AccountIdOf<T>,
		) -> Result<(), DispatchError> {
			ensure!(!Unicks::<T>::contains_key(&owner), Error::<T>::UnickAlreadyClaimed);
			ensure!(!Owner::<T>::contains_key(&unick), Error::<T>::OwnerAlreadyExisting);
			ensure!(!Blacklist::<T>::contains_key(&unick), Error::<T>::UnickBlacklisted);
			validate_unick::<T>(unick)?;

			ensure!(
				<T::Currency as ReservableCurrency<AccountIdOf<T>>>::can_reserve(
					deposit_payer,
					<T as Config>::Deposit::get()
				),
				Error::<T>::InsufficientFunds
			);

			Ok(())
		}

		fn register_unick_unsafe(unick: UnickOf<T>, owner: DidIdentifierOf<T>, deposit_payer: AccountIdOf<T>) {
			let deposit = Deposit {
				owner: deposit_payer,
				amount: T::Deposit::get(),
			};

			// Preconditions tested beforehand. Panic if this is not
			CurrencyOf::<T>::reserve(&deposit.owner, deposit.amount).unwrap();

			Unicks::<T>::insert(&owner, unick.clone());
			Owner::<T>::insert(&unick, UnickOwnershipOf::<T> { owner, deposit });
		}

		fn check_releasing_preconditions(
			unick: &UnickOf<T>,
			caller: UnickReleaseCaller<T>,
		) -> Result<(), DispatchError> {
			let UnickOwnership { owner, deposit } = Owner::<T>::get(unick).ok_or(Error::<T>::UnickNotFound)?;

			match caller {
				UnickReleaseCaller::DepositPayer(caller) => {
					if caller == &deposit.owner {
						Ok(())
					} else {
						Err(Error::<T>::NotAuthorized.into())
					}
				}
				UnickReleaseCaller::UnickOwner(caller) => {
					if caller == &owner {
						Ok(())
					} else {
						Err(Error::<T>::NotAuthorized.into())
					}
				}
			}
		}

		fn unregister_unick_unsafe(unick: &UnickOf<T>) -> UnickOwnershipOf<T> {
			let unick_ownership = Owner::<T>::take(unick).unwrap();
			Unicks::<T>::remove(&unick_ownership.owner);

			kilt_support::free_deposit::<AccountIdOf<T>, CurrencyOf<T>>(&unick_ownership.deposit);

			unick_ownership
		}

		fn check_blacklisting_preconditions(unick: &UnickOf<T>) -> Result<(), DispatchError> {
			ensure!(
				Blacklist::<T>::contains_key(&unick),
				Error::<T>::UnickAlreadyBlacklisted
			);
			let UnickOwnership { owner, .. } = Owner::<T>::get(unick).ok_or(Error::<T>::UnickNotFound)?;
			ensure!(Unicks::<T>::contains_key(&owner), Error::<T>::InternalError);

			Ok(())
		}

		fn blacklist_unick_unsafe(unick: &UnickOf<T>) -> UnickOwnershipOf<T> {
			let unick_ownership = Self::unregister_unick_unsafe(unick);
			Blacklist::<T>::insert(&unick, ());

			unick_ownership
		}

		fn check_unblacklisting_preconditions(unick: &UnickOf<T>) -> Result<(), DispatchError> {
			ensure!(Blacklist::<T>::contains_key(&unick), Error::<T>::UnickNotBlacklisted);

			Ok(())
		}

		fn unblacklist_unick_unsafe(unick: &UnickOf<T>) {
			Blacklist::<T>::remove(unick);
		}
	}
}
