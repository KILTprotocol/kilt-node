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

//! # Balance Locks Module
//!
//! A simple module providing means of adding balance locks in the genesis block and automatically
//! removing these afterwards.
//!
//! ### Dispatchable Functions
//!
//! - `force_unlock` - Remove all locks for a given block, can only be called by sudo.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
use frame_support::traits::GenesisBuild;
use frame_support::{
	dispatch::DispatchResultWithPostInfo,
	pallet_prelude::*,
	sp_runtime::traits::Zero,
	storage::types::StorageMap,
	traits::{
		Currency, ExistenceRequirement::AllowDeath, LockIdentifier, LockableCurrency, Vec,
		WithdrawReasons,
	},
	StorageMap as StorageMapTrait,
};
pub use pallet::*;
use pallet_balances::{BalanceLock, Locks};
use pallet_vesting::Vesting;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

const LOCK_ID: LockIdentifier = *b"InitKilt";

// type BalanceOf<T> =
// 	<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_system::pallet_prelude::*;

	#[pallet::config]
	// pub trait Config: frame_system::Config + pallet_balances::Config {
	pub trait Config:
		frame_system::Config + pallet_balances::Config + pallet_vesting::Config
	{
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		// type Currency =
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub balance_locks: Vec<(T::AccountId, T::BlockNumber)>,
		pub transfer_account: T::AccountId,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				balance_locks: Default::default(),
				transfer_account: Default::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			for (ref who, length) in self.balance_locks.iter() {
				let balance = <pallet_balances::Module<T>>::free_balance(who);
				assert!(
					!balance.is_zero(),
					"Currencies must be init'd before vesting"
				);

				// allow transaction fees to be paid from locked balance, e.g.,
				// prohibit all withdraws except `WithdrawReasons::TRANSACTION_PAYMENT`
				<pallet_balances::Module<T>>::set_lock(
					LOCK_ID,
					who,
					balance,
					WithdrawReasons::except(WithdrawReasons::TRANSACTION_PAYMENT),
				);
				<UnlockingAt<T>>::append(length, who);

				// add unlock block to storage
				<UnlockingBlock<T>>::insert(who, length);

				// set vesting information
			}

			// set transfer account which has a subset of the powers the root account has
			<TransferAccount<T>>::put(self.transfer_account.clone());
		}
	}

	/// Account which is permitted to do token tranfers in PoA phase.
	///
	/// Required for the claiming process.
	#[pallet::storage]
	#[pallet::getter(fn get_transfer_account)]
	pub type TransferAccount<T> = StorageValue<_, <T as frame_system::Config>::AccountId>;

	/// Maps a block to a account ids.
	///
	/// Required for automatic unlocking once the block number is reached in `on_initialize`.
	#[pallet::storage]
	#[pallet::getter(fn get_unlocking_at)]
	pub type UnlockingAt<T> = StorageMap<
		_,
		Blake2_128Concat,
		<T as frame_system::Config>::BlockNumber,
		Vec<<T as frame_system::Config>::AccountId>,
	>;

	/// Maps an account to the block in which balance can be unlocked.
	///
	/// Required for the claiming process.
	#[pallet::storage]
	#[pallet::getter(fn get_unlocking_block)]
	pub type UnlockingBlock<T> = StorageMap<
		_,
		Blake2_128Concat,
		<T as frame_system::Config>::AccountId,
		<T as frame_system::Config>::BlockNumber,
	>;

	#[pallet::event]
	#[pallet::metadata(T::BlockNumber = "BlockNumber")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Unlocked(T::BlockNumber, u64),
	}

	#[pallet::error]
	pub enum Error<T> {
		Unauthorized,
		MultipleLocks,
		MissingVestingSchedule,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(now: T::BlockNumber) -> Weight {
			Self::unlock_balance(now)
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Enable removal of KILT balance locks for sudo
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn force_unlock(
			origin: OriginFor<T>,
			block: T::BlockNumber,
		) -> DispatchResultWithPostInfo {
			let _ = ensure_root(origin)?;

			Ok(Some(Self::unlock_balance(block)).into())
		}

		/// Transfer tokens to an account owned by the claimer
		///
		/// If the source account has vesting or a custom lock enabled,
		/// everything is migrated automatically.
		/// Additionally, we unlock the balance which can already be unlocked from vesting.
		///
		/// Note: Actually, setting the custom lock only occurs in this call to avoid overhead.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn claiming_process(
			origin: OriginFor<T>,
			source: T::AccountId,
			dest: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			ensure!(
				Some(who) == <TransferAccount<T>>::get(),
				Error::<T>::Unauthorized
			);

			// get and remove locks from source
			// potential candidates: vesting, custom lock
			// TODO: Test whether this automatically removes all locks from the storage
			let locks = Locks::<T>::take(&source);
			ensure!(locks.len() <= 1, Error::<T>::MultipleLocks);
			let transfer_amount = <pallet_balances::Module<T>>::total_balance(&source);

			// there can only be a single lock (from vesting)
			// the custom lock is set during claiming process
			// the staking lock can only be actively set by the user after claiming
			// the voting lock can only be actively set by the user after claiming
			if let Some(BalanceLock::<<T as pallet_balances::Config>::Balance> { id, .. }) =
				locks.get(0)
			{
				// remove source lock to enable transfer
				<pallet_balances::Module<T>>::remove_lock(*id, &dest);

				// transfer to dest before migrating vesting schedule
				let _ = <pallet_balances::Module<T> as Currency<T::AccountId>>::transfer(
					&source,
					&dest,
					transfer_amount,
					AllowDeath,
				)?;

				// migrate lock and vesting schedule
				// TODO: Check whether take clears the storage
				let vesting =
					Vesting::<T>::take(&source).ok_or(Error::<T>::MissingVestingSchedule)?;
				Vesting::<T>::insert(&dest, vesting);

				// automatically unlock the current amount which can be unlocked
				// enables the user to have funds before actively calling `vest`
				// if claimed after the genesis block
				// logic taken from pallet_vesting
				let reasons = WithdrawReasons::TRANSFER | WithdrawReasons::RESERVE;
				let now = <frame_system::Module<T>>::block_number();
				let locked_now =
					vesting.locked_at::<<T as pallet_vesting::Config>::BlockNumberToBalance>(now);
				<<T as pallet_vesting::Config>::Currency as LockableCurrency<T::AccountId>>::set_lock(
					*id,
					&dest,
					locked_now.into(),
					reasons,
				);
			} else {
				// transfer to dest
				let _ = <pallet_balances::Module<T> as Currency<T::AccountId>>::transfer(
					&source,
					&dest,
					transfer_amount,
					AllowDeath,
				)?;
			}

			// check for custom lock
			if let Some(block) = <UnlockingBlock<T>>::take(&dest) {
				<UnlockingAt<T>>::append(block, &dest);
			}

			// <T as Currency<T::AccountId>>
			Ok(Some(0).into())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Remove KILT balance locks for the specified block
	fn unlock_balance(block: T::BlockNumber) -> Weight {
		if let Some(unlocking_balance) = <UnlockingAt<T>>::get(block) {
			// remove locks for all accounts
			for account in unlocking_balance.iter() {
				<pallet_balances::Module<T>>::remove_lock(LOCK_ID, account);
			}
			// remove storage entry
			<UnlockingAt<T>>::remove(block);

			Self::deposit_event(Event::Unlocked(block, unlocking_balance.len() as u64));
			T::DbWeight::get().reads_writes(1, (unlocking_balance.len() + 1) as u64)
		} else {
			T::DbWeight::get().reads(1)
		}
	}
}

// impl From<Reasons> for WithdrawReasons {
// 	fn from(r: Reasons) -> WithdrawReasons {
// 		match r {
// 			Reasons::Fee => WithdrawReasons::TRANSACTION_PAYMENT,
// 			Reasons::Misc => WithdrawReasons::ALL,
// 			Reasons::All => WithdrawReasons::ALL,
// 		}
// 	}
// }
