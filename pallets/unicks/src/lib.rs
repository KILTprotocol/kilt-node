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

pub mod kilt_unicks;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use codec::FullCodec;
	use frame_support::{
		pallet_prelude::*,
		traits::{Currency, ReservableCurrency, StorageVersion},
		Blake2_128Concat,
	};
	use frame_system::pallet_prelude::*;
	use sp_std::{fmt::Debug, vec::Vec};

	use kilt_support::{deposit::Deposit, traits::CallSources};

	use crate::kilt_unicks::UnickOwnership;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	pub type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;
	pub type BlockNumberFor<T> = <T as frame_system::Config>::BlockNumber;
	pub type UnickOwnerOf<T> = <T as Config>::UnickOwner;
	pub type UnickInput<T> = BoundedVec<u8, <T as Config>::MaxUnickLength>;
	pub type UnickOf<T> = <T as Config>::Unick;
	pub type UnickOwnershipOf<T> =
		UnickOwnership<UnickOwnerOf<T>, Deposit<AccountIdOf<T>, BalanceOf<T>>, BlockNumberFor<T>>;

	type CurrencyOf<T> = <T as Config>::Currency;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	// Unick -> Ownership
	#[pallet::storage]
	#[pallet::getter(fn owner)]
	pub type Owner<T> = StorageMap<_, Blake2_128Concat, UnickOf<T>, UnickOwnershipOf<T>>;

	// Owner -> Unick
	#[pallet::storage]
	#[pallet::getter(fn unicks)]
	pub type Unicks<T> = StorageMap<_, Twox64Concat, UnickOwnerOf<T>, UnickOf<T>>;

	// Unick -> ()
	#[pallet::storage]
	#[pallet::getter(fn is_blacklisted)]
	pub type Blacklist<T> = StorageMap<_, Blake2_128Concat, UnickOf<T>, ()>;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The origin allowed to blacklist unicks.
		type BlacklistOrigin: EnsureOrigin<Self::Origin>;
		/// The currency type to reserve and release deposits.
		type Currency: ReservableCurrency<AccountIdOf<Self>>;
		/// The amount of KILT to deposit to claim a unick.
		#[pallet::constant]
		type Deposit: Get<BalanceOf<Self>>;
		/// The overarching event type.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		/// The min encoded length of a unick.
		#[pallet::constant]
		type MinUnickLength: Get<u32>;
		/// The max encoded length of a unick.
		#[pallet::constant]
		type MaxUnickLength: Get<u32>;
		/// The type of origin after a successful origin check.
		type OriginSuccess: CallSources<AccountIdOf<Self>, UnickOwnerOf<Self>>;
		/// The origin allowed to perform regular operations.
		type RegularOrigin: EnsureOrigin<Success = Self::OriginSuccess, <Self as frame_system::Config>::Origin>;
		/// The type of a unick.
		type Unick: FullCodec + Debug + PartialEq + Clone + TypeInfo + TryFrom<Vec<u8>, Error = Error<Self>>;
		/// The type of a unick owner.
		type UnickOwner: Parameter + Default;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new unick has been claimed.
		UnickClaimed { owner: UnickOwnerOf<T>, unick: UnickOf<T> },
		/// A unick has been released.
		UnickReleased { owner: UnickOwnerOf<T>, unick: UnickOf<T> },
		/// A unick has been blacklisted.
		UnickBlacklisted { unick: UnickOf<T> },
		/// A unick has been unblacklisted.
		UnickUnblacklisted { unick: UnickOf<T> },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The tx submitter does not have enough fund to pay for the deposit.
		InsufficientFunds,
		/// The specified unick has already been previously claimed.
		UnickAlreadyClaimed,
		/// The specified unick does not exist.
		UnickNotFound,
		/// The specified owner already owns a unick.
		OwnerAlreadyExisting,
		/// The specified unick has been blacklisted and cannot be interacted
		/// with.
		UnickBlacklisted,
		/// The specified unick is not currently blacklisted.
		UnickNotBlacklisted,
		/// The specified unick has already been previously blacklisted.
		UnickAlreadyBlacklisted,
		/// The actor cannot performed the specified operation.
		NotAuthorized,
		/// A valid unick cannot be decoded from the provided input.
		InvalidUnickFormat,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Assign the specified unick to the owner as specified in the origin.
		///
		/// The unick must not have already been claimed by someone else and the
		/// owner must not already own another unick.
		///
		/// Emits `UnickClaimed` if the operation is carried out successfully.
		///
		/// # <weight>
		/// Weight: O(1)
		/// - Reads: Unicks, Owner, Blacklist storage entries + available
		///   currency check + origin check
		/// - Writes: Unicks, Owner storage entries + currency deposit reserve
		/// # </weight>
		#[pallet::weight(0)]
		pub fn claim(origin: OriginFor<T>, unick: UnickInput<T>) -> DispatchResult {
			let origin = T::RegularOrigin::ensure_origin(origin)?;
			let payer = origin.sender();
			let owner = origin.subject();

			let decoded_unick = Self::check_claiming_preconditions(unick, &owner, &payer)?;

			// No failure beyond this point

			Self::register_unick(decoded_unick.clone(), owner.clone(), payer);
			Self::deposit_event(Event::<T>::UnickClaimed {
				owner,
				unick: decoded_unick,
			});

			Ok(())
		}

		/// Release the provided unick from its owner.
		///
		/// The origin must be the owner of the specified unick.
		///
		/// Emits `UnickReleased` if the operation is carried out successfully.
		///
		/// # <weight>
		/// Weight: O(1)
		/// - Reads: Owner storage entry + origin check
		/// - Writes: Unicks, Owner storage entries + currency deposit release
		/// # </weight>
		#[pallet::weight(0)]
		pub fn release_by_owner(origin: OriginFor<T>, unick: UnickInput<T>) -> DispatchResult {
			let origin = T::RegularOrigin::ensure_origin(origin)?;
			let owner = origin.subject();

			let unick = UnickOf::<T>::try_from(unick.into_inner()).map_err(DispatchError::from)?;

			Self::check_releasing_preconditions_for_owner(&unick, &owner)?;

			// No failure beyond this point

			Self::unregister_unick(&unick);
			Self::deposit_event(Event::<T>::UnickReleased { owner, unick });

			Ok(())
		}

		/// Release the provided unick from its owner.
		///
		/// The origin must be the account that paid for the unick's deposit.
		///
		/// Emits `UnickReleased` if the operation is carried out successfully.
		///
		/// # <weight>
		/// Weight: O(1)
		/// - Reads: Owner storage entry + origin check
		/// - Writes: Unicks, Owner storage entries + currency deposit release
		/// # </weight>
		#[pallet::weight(0)]
		pub fn release_by_payer(origin: OriginFor<T>, unick: UnickInput<T>) -> DispatchResult {
			let caller = ensure_signed(origin)?;

			let unick = UnickOf::<T>::try_from(unick.into_inner()).map_err(DispatchError::from)?;

			Self::check_releasing_preconditions_for_caller(&unick, &caller)?;

			// No failure beyond this point

			let UnickOwnershipOf::<T> { owner, .. } = Self::unregister_unick(&unick);
			Self::deposit_event(Event::<T>::UnickReleased { owner, unick });

			Ok(())
		}

		/// Blacklist a unick.
		///
		/// A blacklisted unick cannot be claimed by anyone. The unick's deposit
		/// is returned to the original payer.
		///
		/// The origin must be the blacklist origin.
		///
		/// Emits `UnickBlacklisted` if the operation is carried out
		/// successfully.
		///
		/// # <weight>
		/// Weight: O(1)
		/// - Reads: Blacklist, Owner, Unicks storage entries + origin check
		/// - Writes: Unicks, Owner, Blacklist storage entries + currency
		///   deposit release
		/// # </weight>
		#[pallet::weight(0)]
		pub fn blacklist(origin: OriginFor<T>, unick: UnickInput<T>) -> DispatchResult {
			T::BlacklistOrigin::ensure_origin(origin)?;

			let unick = UnickOf::<T>::try_from(unick.into_inner()).map_err(DispatchError::from)?;

			// No failure beyond this point

			// Unregister (including returning the deposit) only if the unick was assigned
			// to someone.
			if Self::check_blacklisting_preconditions(&unick)? {
				Self::unregister_unick(&unick);
			}

			Self::blacklist_unick(&unick);
			Self::deposit_event(Event::<T>::UnickBlacklisted { unick });

			Ok(())
		}

		/// Unblacklist a unick.
		///
		/// Make a unick claimable again.
		///
		/// The origin must be the blacklist origin.
		///
		/// Emits `UnickUnblacklisted` if the operation is carried out
		/// successfully.
		///
		/// # <weight>
		/// Weight: O(1)
		/// - Reads: Blacklist storage entry + origin check
		/// - Writes: Blacklist storage entry deposit release
		/// # </weight>
		#[pallet::weight(0)]
		pub fn unblacklist(origin: OriginFor<T>, unick: UnickInput<T>) -> DispatchResult {
			T::BlacklistOrigin::ensure_origin(origin)?;

			let unick = UnickOf::<T>::try_from(unick.into_inner()).map_err(DispatchError::from)?;

			Self::check_unblacklisting_preconditions(&unick)?;

			// No failure beyond this point

			Self::unblacklist_unick(&unick);
			Self::deposit_event(Event::<T>::UnickUnblacklisted { unick });

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// Verify that the claiming preconditions are verified. Specifically:
		/// - The unick input data can be decoded as a valid unick
		/// - The unick does not already exist
		/// - The owner does not already own a unick
		/// - The unick has not been blacklisted
		/// - The tx submitter has enough funds to pay the deposit
		fn check_claiming_preconditions(
			unick: UnickInput<T>,
			owner: &UnickOwnerOf<T>,
			deposit_payer: &AccountIdOf<T>,
		) -> Result<UnickOf<T>, DispatchError> {
			let unick = UnickOf::<T>::try_from(unick.into_inner()).map_err(DispatchError::from)?;

			ensure!(!Unicks::<T>::contains_key(&owner), Error::<T>::OwnerAlreadyExisting);
			ensure!(!Owner::<T>::contains_key(&unick), Error::<T>::UnickAlreadyClaimed);
			ensure!(!Blacklist::<T>::contains_key(&unick), Error::<T>::UnickBlacklisted);

			ensure!(
				<T::Currency as ReservableCurrency<AccountIdOf<T>>>::can_reserve(deposit_payer, T::Deposit::get()),
				Error::<T>::InsufficientFunds
			);

			Ok(unick)
		}

		/// Assign a unick to the provided owner reserving the deposit from the
		/// provided account. This function must be called after
		/// `check_claiming_preconditions` as it does not verify all the
		/// preconditions again.
		pub(crate) fn register_unick(unick: UnickOf<T>, owner: UnickOwnerOf<T>, deposit_payer: AccountIdOf<T>) {
			let deposit = Deposit {
				owner: deposit_payer,
				amount: T::Deposit::get(),
			};
			let block_number = frame_system::Pallet::<T>::block_number();

			CurrencyOf::<T>::reserve(&deposit.owner, deposit.amount).unwrap();

			Unicks::<T>::insert(&owner, unick.clone());
			Owner::<T>::insert(
				&unick,
				UnickOwnershipOf::<T> {
					owner,
					claimed_at: block_number,
					deposit,
				},
			);
		}

		/// Verify that the releasing preconditions for an owner are verified.
		/// Specifically:
		/// - The unick exists (i.e., it has been previous claimed)
		/// - The caller owns the given unick
		fn check_releasing_preconditions_for_owner(
			unick: &UnickOf<T>,
			owner: &UnickOwnerOf<T>,
		) -> Result<(), DispatchError> {
			let UnickOwnership {
				owner: stored_owner, ..
			} = Owner::<T>::get(unick).ok_or(Error::<T>::UnickNotFound)?;

			ensure!(owner == &stored_owner, Error::<T>::NotAuthorized);

			Ok(())
		}

		/// Verify that the releasing preconditions for a deposit payer are
		/// verified. Specifically:
		/// - The unick exists (i.e., it has been previous claimed)
		/// - The caller owns the unick's deposit
		fn check_releasing_preconditions_for_caller(
			unick: &UnickOf<T>,
			caller: &AccountIdOf<T>,
		) -> Result<(), DispatchError> {
			let UnickOwnership { deposit, .. } = Owner::<T>::get(unick).ok_or(Error::<T>::UnickNotFound)?;

			ensure!(caller == &deposit.owner, Error::<T>::NotAuthorized);

			Ok(())
		}

		/// Release the provided unick and returns the deposit to the original
		/// payer. This function must be called after
		/// `check_releasing_preconditions` as it does not verify all the
		/// preconditions again.
		fn unregister_unick(unick: &UnickOf<T>) -> UnickOwnershipOf<T> {
			let unick_ownership = Owner::<T>::take(unick).unwrap();
			Unicks::<T>::remove(&unick_ownership.owner);

			kilt_support::free_deposit::<AccountIdOf<T>, CurrencyOf<T>>(&unick_ownership.deposit);

			unick_ownership
		}

		/// Verify that the blacklisting preconditions are verified.
		/// Specifically:
		/// - The unick must not be already blacklisted
		///
		/// If the preconditions are verified, return
		/// whether the unick being blacklisted is currently assigned to someone
		/// or not.
		fn check_blacklisting_preconditions(unick: &UnickOf<T>) -> Result<bool, DispatchError> {
			ensure!(
				!Blacklist::<T>::contains_key(&unick),
				Error::<T>::UnickAlreadyBlacklisted
			);

			Ok(Owner::<T>::contains_key(unick))
		}

		/// Blacklist the provided unick. This function must be called after
		/// `check_blacklisting_preconditions` as it does not verify all the
		/// preconditions again.
		pub(crate) fn blacklist_unick(unick: &UnickOf<T>) {
			Blacklist::<T>::insert(&unick, ());
		}

		/// Verify that the unblacklisting preconditions are verified.
		/// Specifically:
		/// - The unick must have already been blacklisted
		fn check_unblacklisting_preconditions(unick: &UnickOf<T>) -> Result<(), DispatchError> {
			ensure!(Blacklist::<T>::contains_key(&unick), Error::<T>::UnickNotBlacklisted);

			Ok(())
		}

		/// Unblacklist the provided unick. This function must be called after
		/// `check_unblacklisting_preconditions` as it does not verify all the
		/// preconditions again.
		fn unblacklist_unick(unick: &UnickOf<T>) {
			Blacklist::<T>::remove(unick);
		}
	}
}
