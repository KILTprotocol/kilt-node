// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2021 BOTLabs GmbH

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

use codec::{Decode, Encode};

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[derive(Debug, Clone, Encode, Decode)]
pub struct BackingInfo<T: Config>
where
	<T as pallet_scored_pool::Config>::Score: From<<T as pallet_balances::Config>::Balance>,
{
	pub candidancy_term: T::BlockNumber,
	pub amount: T::Balance,
}

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		dispatch::DispatchResultWithPostInfo,
		pallet_prelude::*,
		traits::{LockIdentifier, LockableCurrency, WithdrawReasons},
	};
	use frame_system::pallet_prelude::*;
	use pallet_balances::{BalanceLock, Locks};
	use sp_runtime::traits::{Saturating, StaticLookup, Zero};

	pub const BACKING_ID: LockIdentifier = *b"kiltback";

	/// Configure the pallet by specifying the parameters and types on which it
	/// depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_balances::Config + pallet_scored_pool::Config
	where
		<Self as pallet_scored_pool::Config>::Score: From<<Self as pallet_balances::Config>::Balance>,
	{
		/// Because this pallet emits events, it depends on the runtime's
		/// definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// The upper limit for the number of candidates a backer can back.
		type BackingCandidateLimit: Get<u32>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	/// Candidates and there total amount of backing
	#[pallet::storage]
	#[pallet::getter(fn candidates)]
	pub type Candidates<T> = StorageMap<
		_,
		Blake2_128Concat,
		<T as frame_system::Config>::AccountId,
		(
			<T as frame_system::Config>::BlockNumber,
			<T as pallet_balances::Config>::Balance,
		),
	>;

	/// Backers and the amount of backing for each backed candidate. A backer
	/// can only back up to `Config::BackingCandidateLimit` candidates.
	#[pallet::storage]
	#[pallet::getter(fn backers)]
	pub type Backing<T> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		<T as frame_system::Config>::AccountId,
		Blake2_128Concat,
		<T as frame_system::Config>::AccountId,
		crate::BackingInfo<T>,
	>;

	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config>
	where
		<T as pallet_scored_pool::Config>::Score: From<<T as pallet_balances::Config>::Balance>,
	{
		/// Event documentation should end with an array that provides
		/// descriptive names for event parameters. [something, who]
		SubmitedCandidancy(T::AccountId),
		RevokedCandidancy(T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		CandidateNotFound,
		InsufficientBalance,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> where
		<T as pallet_scored_pool::Config>::Score: From<<T as pallet_balances::Config>::Balance>
	{
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		<T as pallet_scored_pool::Config>::Score: From<<T as pallet_balances::Config>::Balance>,
	{
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn submit_candidancy(origin: OriginFor<T>) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin.clone())?;

			pallet_scored_pool::Pallet::<T>::submit_candidacy(origin)?;
			<Candidates<T>>::insert(
				who.clone(),
				(
					frame_system::Pallet::<T>::block_number(),
					<T as pallet_balances::Config>::Balance::zero(),
				),
			);

			Self::deposit_event(Event::SubmitedCandidancy(who));
			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn withdraw_candidacy(origin: OriginFor<T>, index: u32) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin.clone())?;

			pallet_scored_pool::Pallet::<T>::withdraw_candidacy(origin, index)?;
			<Candidates<T>>::remove(who.clone());
			// TODO: Event!!

			Self::deposit_event(Event::RevokedCandidancy(who));
			Ok(().into())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn set_backing_for_candidate(
			origin: OriginFor<T>,
			candidate: <T::Lookup as StaticLookup>::Source,
			index: u32,
			amount: T::Balance,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin.clone())?;
			let candidancy_id = <T::Lookup as StaticLookup>::lookup(candidate.clone())?;
			let (candidancy_term, candidancy_backing) =
				<Candidates<T>>::get(&candidancy_id).ok_or(Error::<T>::CandidateNotFound)?;

			// TODO: ensure that BackingCandidateLimit is not exceded!
			let current_backing = <Backing<T>>::get(&who, &candidancy_id).unwrap_or_else(|| crate::BackingInfo {
				candidancy_term,
				amount: <T as pallet_balances::Config>::Balance::zero(),
			});

			let new_backing = crate::BackingInfo::<T> {
				candidancy_term,
				amount,
			};

			let current_locked = Pallet::<T>::currently_locked(&who);

			let new_locked = current_locked
				.saturating_sub(current_backing.amount)
				.saturating_add(amount);

			let new_candidancy_backing = if candidancy_term == current_backing.candidancy_term {
				candidancy_backing
					.saturating_sub(current_backing.amount)
					.saturating_add(new_backing.amount)
			} else {
				candidancy_backing.saturating_add(new_backing.amount)
			};

			// Either we reduce the locked amount or the account has enough free balance to
			// increase the lock
			ensure!(
				new_locked < current_locked
					|| new_locked.saturating_sub(current_locked) <= pallet_balances::Pallet::<T>::free_balance(&who),
				Error::<T>::InsufficientBalance
			);
			<pallet_balances::Pallet<T> as LockableCurrency<T::AccountId>>::set_lock(
				BACKING_ID,
				&who,
				new_locked,
				WithdrawReasons::except(WithdrawReasons::TRANSACTION_PAYMENT),
			);

			<Candidates<T>>::mutate(&candidancy_id, |old| {
				if let Some(mut old) = old {
					old.1 = new_candidancy_backing
				};
			});

			if new_backing.amount.is_zero() {
				// TODO: Event!!
				<Backing<T>>::remove(&who, candidancy_id);
			} else {
				// TODO: Event!!
				<Backing<T>>::insert(&who, candidancy_id, new_backing);
			}

			pallet_scored_pool::Pallet::<T>::score(origin, candidate, index, new_candidancy_backing.into())?;

			Self::deposit_event(Event::RevokedCandidancy(who));
			Ok(().into())
		}
	}

	impl<T: Config> Pallet<T>
	where
		<T as pallet_scored_pool::Config>::Score: From<<T as pallet_balances::Config>::Balance>,
	{
		fn currently_locked(who: &T::AccountId) -> T::Balance {
			Locks::<T>::get(&who)
				.iter()
				.find(|BalanceLock::<<T as pallet_balances::Config>::Balance> { id, .. }| id == &BACKING_ID)
				.map(|lock| lock.amount)
				.unwrap_or_else(<T as pallet_balances::Config>::Balance::zero)
		}
	}
}
