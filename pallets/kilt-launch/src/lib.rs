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
//! A simple module providing means of adding balance locks in the genesis block
//! and automatically removing these afterwards.
//!
//! ### Dispatchable Functions
//!
//! - `force_unlock` - Remove all locks for a given block, can only be called by
//!   sudo.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
use frame_support::traits::GenesisBuild;
use frame_support::{
	dispatch::DispatchResultWithPostInfo,
	pallet_prelude::*,
	sp_runtime::traits::Zero,
	storage::types::StorageMap,
	traits::{Currency, ExistenceRequirement::AllowDeath, LockIdentifier, LockableCurrency, Vec, WithdrawReasons},
	transactional, StorageMap as StorageMapTrait,
};
pub use pallet::*;
use pallet_balances::Locks;
use pallet_vesting::{Vesting, VestingInfo};
use sp_runtime::traits::Convert;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

const KILT_LAUNCH_ID: LockIdentifier = *b"kiltcoin";
const VESTING_ID: LockIdentifier = *b"vesting ";

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_system::pallet_prelude::*;

	#[derive(Debug, Encode, Decode, PartialEq, Eq)]
	pub struct LockedBalance<T: Config> {
		pub block: T::BlockNumber,
		pub amount: <T as pallet_balances::Config>::Balance,
	}

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_balances::Config + pallet_vesting::Config {
		/// Because this pallet emits events, it depends on the runtime's
		/// definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub balance_locks: Vec<(T::AccountId, T::BlockNumber, <T as pallet_balances::Config>::Balance)>,
		pub transfer_account: T::AccountId,
		pub vesting: Vec<(T::AccountId, T::BlockNumber, BalanceOf<T>)>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				balance_locks: Default::default(),
				transfer_account: Default::default(),
				vesting: Default::default(),
			}
		}
	}

	// Balance type based on pallet_vesting
	type BalanceOf<T> =
		<<T as pallet_vesting::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			// Generate initial custom locking configuration
			// * who - Account which setting the custom lock for
			// * length - Number of blocks from  until removal of the lock
			// * amount - Number of tokens which are locked
			for (ref who, length, locked) in self.balance_locks.iter() {
				let balance = <pallet_balances::Module<T>>::free_balance(who);
				assert!(!balance.is_zero(), "Currencies must be init'd before locking");
				assert!(
					balance >= *locked,
					"Locked balance must not exceed total balance for address {:?}",
					who
				);

				// Add unlock block to storage
				<BalanceLocks<T>>::insert(
					who,
					LockedBalance::<T> {
						block: *length,
						amount: *locked,
					},
				);
				// Instead of setting the lock now, we do so in
				// `claiming_process`
			}

			// Generate initial vesting configuration, taken from pallet_vesting
			// * who - Account which we are generating vesting configuration for
			// * begin - Block when the account will start to vest
			// * length - Number of blocks from `begin` until fully vested
			// * liquid - Number of units which can be spent before vesting begins =
			//   total_balance - vesting_balance + 1
			for &(ref who, length, locked) in self.vesting.iter() {
				let balance = <<T as pallet_vesting::Config>::Currency as Currency<
					<T as frame_system::Config>::AccountId,
				>>::free_balance(who);
				assert!(!balance.is_zero(), "Currencies must be init'd before vesting");
				assert!(
					balance >= locked,
					"Vested balance must not exceed total balance for address {:?}",
					who
				);
				let length_as_balance = T::BlockNumberToBalance::convert(length);
				let per_block = locked / length_as_balance.max(sp_runtime::traits::One::one());

				Vesting::<T>::insert(
					who,
					VestingInfo::<BalanceOf<T>, T::BlockNumber> {
						locked,
						per_block,
						starting_block: T::BlockNumber::zero(),
					},
				);
				// Instead of setting the lock now, we do so in
				// `claiming_process`
			}

			// Set the transfer account which has a subset of the powers of root
			<TransferAccount<T>>::put(self.transfer_account.clone());
		}
	}

	/// Account which is permitted to do token tranfers in PoA phase.
	///
	/// Required for the claiming process.
	#[pallet::storage]
	#[pallet::getter(fn get_transfer_account)]
	pub type TransferAccount<T> = StorageValue<_, <T as frame_system::Config>::AccountId>;

	/// Maps a block to account ids which have their balance locked.
	///
	/// Required for automatic unlocking once the block number is reached in
	/// `on_initialize`.
	#[pallet::storage]
	#[pallet::getter(fn get_unlocking_at)]
	pub type UnlockingAt<T> = StorageMap<
		_,
		Blake2_128Concat,
		<T as frame_system::Config>::BlockNumber,
		Vec<<T as frame_system::Config>::AccountId>,
	>;

	/// Maps an account to the (block, balance) pair in which the latter can be
	/// unlocked.
	///
	/// Required for the claiming process.
	#[pallet::storage]
	#[pallet::getter(fn get_unlocking_block)]
	pub type BalanceLocks<T> =
		StorageMap<_, Blake2_128Concat, <T as frame_system::Config>::AccountId, LockedBalance<T>>;

	#[pallet::event]
	#[pallet::metadata(T::BlockNumber = "BlockNumber")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		Unlocked(T::BlockNumber, u64),
	}

	#[pallet::error]
	pub enum Error<T> {
		Unauthorized,
		UnexpectedLocks,
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
		/// Enable removal of KILT balance locks via sudo
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn force_unlock(origin: OriginFor<T>, block: T::BlockNumber) -> DispatchResultWithPostInfo {
			let _ = ensure_root(origin)?;

			Ok(Some(Self::unlock_balance(block)).into())
		}

		/// Transfer tokens to an account owned by the claimer.
		///
		/// If the source account has vesting or a custom lock enabled,
		/// everything is migrated automatically. Additionally, we unlock the
		/// balance which can already be unlocked from vesting. This should
		/// enable the user to pay the transaction fees for the next call of
		/// `vest` which is always required to be explicitly called in order to
		/// unlock balance from vesting.
		///
		/// Note: Setting the custom lock actually only occurs in this call (and
		/// not when building the genesis block) to avoid overhead from handling
		/// locks when migrating.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(7, 7))]
		#[transactional]
		pub fn claiming_process(
			origin: OriginFor<T>,
			source: T::AccountId,
			dest: T::AccountId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// The extrinsic has to be called by the TransferAccount
			ensure!(Some(who) == <TransferAccount<T>>::get(), Error::<T>::Unauthorized);

			// There should be no locks for the source address
			ensure!(Locks::<T>::get(&source).len().is_zero(), Error::<T>::UnexpectedLocks);

			// Check for vesting and custom locks
			let vesting_info = Vesting::<T>::take(&source);

			// Transfer to destination addess
			let amount = <pallet_balances::Module<T>>::total_balance(&source);
			let _ =
				<pallet_balances::Module<T> as Currency<T::AccountId>>::transfer(&source, &dest, amount, AllowDeath)?;

			// Migrate vesting info and set the corresponding vesting lock
			if let Some(vesting) = vesting_info {
				Vesting::<T>::insert(&dest, vesting);
				// Only lock funds from now until vesting expires.
				// Enables the user to have funds before actively calling `vest` if claimed
				// after the genesis block.
				//
				// Logic was taken from pallet_vesting.
				let reasons = WithdrawReasons::TRANSFER | WithdrawReasons::RESERVE;
				let now = <frame_system::Module<T>>::block_number();
				let locked_now = vesting.locked_at::<<T as pallet_vesting::Config>::BlockNumberToBalance>(now);
				<<T as pallet_vesting::Config>::Currency as LockableCurrency<T::AccountId>>::set_lock(
					VESTING_ID,
					&dest,
					locked_now.into(),
					reasons,
				);
			}

			// Set the KILT custom lock
			if let Some(LockedBalance::<T> {
				block: unlock_block,
				amount: unlock_amount,
			}) = <BalanceLocks<T>>::take(&dest)
			{
				// Allow transaction fees to be paid from locked balance, e.g., prohibit all
				// withdraws except `WithdrawReasons::TRANSACTION_PAYMENT`
				<pallet_balances::Module<T>>::set_lock(
					KILT_LAUNCH_ID,
					&dest,
					unlock_amount,
					WithdrawReasons::except(WithdrawReasons::TRANSACTION_PAYMENT),
				);
				<UnlockingAt<T>>::append(unlock_block, &dest);
			}
			// TODO: Add meaningful information
			Ok(Some(0).into())
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Remove KILT balance locks for the specified block
	fn unlock_balance(block: T::BlockNumber) -> Weight {
		if let Some(unlocking_balance) = <UnlockingAt<T>>::take(block) {
			// Remove locks for all accounts
			for account in unlocking_balance.iter() {
				<pallet_balances::Module<T>>::remove_lock(KILT_LAUNCH_ID, account);
			}

			Self::deposit_event(Event::Unlocked(block, unlocking_balance.len() as u64));
			T::DbWeight::get().reads_writes(1, (unlocking_balance.len() + 1) as u64)
		} else {
			T::DbWeight::get().reads(1)
		}
	}
}
