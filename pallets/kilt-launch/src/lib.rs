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

//! # KILT Launch Pallet
//!
//! A simple pallet providing means of setting up custom KILT balance locks and
//! vesting schedules for unowned accounts in the genesis block. These should
//! later be migrated to user-owned accounts via the extrinsic
//! `migrate_genesis_account`.
//!
//! ### Dispatchable Functions
//!
//! - `migrate_genesis_account` - Migrate vesting or the KILT custom lock from
//!   an unowned account to a user-owned account. Requires signature of a
//!   special account `TransferAccount` which does not have any other super
//!   powers.
//! - `change_transfer_account` - Change the transfer account. Can only be
//!   called by sudo.
//! - `force_unlock` - Remove all locks for a given block. Can only be called by
//!   sudo.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
use frame_support::traits::GenesisBuild;
use frame_support::{
	dispatch::DispatchResultWithPostInfo,
	pallet_prelude::*,
	sp_runtime::traits::{StaticLookup, Zero},
	storage::types::StorageMap,
	traits::{Currency, ExistenceRequirement::AllowDeath, Get, LockIdentifier, LockableCurrency, Vec, WithdrawReasons},
	transactional, StorageMap as StorageMapTrait,
};
pub use pallet::*;
use pallet_balances::{BalanceLock, Locks};
use pallet_vesting::{Vesting, VestingInfo};
use sp_runtime::traits::{Convert, Saturating};
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

// TODO: Add benchmarking
// #[cfg(feature = "runtime-benchmarks")]
// mod benchmarking;

pub const KILT_LAUNCH_ID: LockIdentifier = *b"kiltlock";
pub const VESTING_ID: LockIdentifier = *b"vesting ";

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_system::pallet_prelude::*;

	#[derive(Debug, Encode, Decode, PartialEq, Eq, Clone)]
	pub struct LockedBalance<T: Config> {
		pub block: T::BlockNumber,
		pub amount: <T as pallet_balances::Config>::Balance,
	}

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_balances::Config + pallet_vesting::Config {
		/// Because this pallet emits events, it depends on the runtime's
		/// definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// Maximum number of claims which can be migrated in a single call.
		/// Used for weight estimation.

		/// NOTE:
		/// + Benchmarks will need to be re-run and weights adjusted if this
		/// changes. + This pallet assumes that dependents keep to the limit
		/// without enforcing it.
		type MaxClaims: Get<usize>;
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
	pub type BalanceOf<T> =
		<<T as pallet_vesting::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			// Generate initial custom locking configuration
			// * who - Account which setting the custom lock for
			// * length - Number of blocks from  until removal of the lock
			// * amount - Number of tokens which are locked
			for (ref who, length, locked) in self.balance_locks.iter() {
				if !length.is_zero() {
					let balance = <pallet_balances::Pallet<T>>::free_balance(who);
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
					// `migrate_genesis_account`, see there for explanation
				}
			}

			// Generate initial vesting configuration, taken from pallet_vesting
			// * who - Account which we are generating vesting configuration for
			// * begin - Block when the account will start to vest
			// * length - Number of blocks from `begin` until fully vested
			// * liquid - Number of units which can be spent before vesting begins =
			//   total_balance - vesting_balance + 1
			for &(ref who, length, locked) in self.vesting.iter() {
				if !length.is_zero() {
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
					// `migrate_genesis_account`, see there for explanation
				}
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

	/// Maps an account id to the (block, balance) pair in which the latter can
	/// be unlocked.
	///
	/// Required for the claiming process.
	#[pallet::storage]
	#[pallet::getter(fn get_unlocking_block)]
	pub type BalanceLocks<T> =
		StorageMap<_, Blake2_128Concat, <T as frame_system::Config>::AccountId, LockedBalance<T>>;

	#[pallet::event]
	#[pallet::metadata(T::BlockNumber = "BlockNumber", T::AccountId = "AccountId", T::Balance = "Balance")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// A KILT balance lock has been removed in the corresponding block. \[block, len\]
		Unlocked(T::BlockNumber, u64),
		// An account transfered their locked balance to another account. \[from, value, target\]
		LockedTransfer(T::AccountId, T::Balance, T::AccountId),
		// A KILT balance lock has been set. \[who, value, until\]
		AddedKiltLock(T::AccountId, T::Balance, T::BlockNumber),
		// Vesting has been added to an account. \[who, per_block, total\]
		AddedVesting(T::AccountId, BalanceOf<T>, BalanceOf<T>),
	}

	#[pallet::error]
	pub enum Error<T> {
		Unauthorized,
		UnexpectedLocks,
		ConflictingLockingBlocks,
		ConflictingVestingStarts,
		ExceedsMaxClaims,
		InsufficientBalance,
		InsufficientLockedBalance,
		BalanceLockNotFound,
		ExpectedLocks,
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

		/// Enable change of the transfer account via sudo
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn change_transfer_account(
			origin: OriginFor<T>,
			transfer_account: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;
			let transfer_account = T::Lookup::lookup(transfer_account)?;

			<TransferAccount<T>>::put(transfer_account);

			Ok(Some(T::DbWeight::get().writes(1)).into())
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
		/// locks when migrating. We can do so because all target accounts
		/// are not owned by anyone and thus these cannot sign and/or call any
		/// extrinsics.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(9, 7))]
		#[transactional]
		pub(super) fn migrate_genesis_account(
			origin: OriginFor<T>,
			source: <T::Lookup as StaticLookup>::Source,
			target: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let source = T::Lookup::lookup(source)?;
			let target = T::Lookup::lookup(target)?;

			// The extrinsic has to be called by the TransferAccount
			ensure!(Some(who) == <TransferAccount<T>>::get(), Error::<T>::Unauthorized);

			Ok(Some(Self::migrate_user(&source, &target)?).into())
		}

		/// Transfer all balances, vesting and custom locks for multiple source
		/// addresses to the same target address.
		///
		/// See `migrate_genesis_account` for details as we run the same logic
		/// for each account id in sources.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(9, 7))]
		#[transactional]
		pub(super) fn migrate_multiple_genesis_accounts(
			origin: OriginFor<T>,
			sources: Vec<<T::Lookup as StaticLookup>::Source>,
			target: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let target = T::Lookup::lookup(target)?;

			// The extrinsic has to be called by the TransferAccount
			ensure!(Some(who) == <TransferAccount<T>>::get(), Error::<T>::Unauthorized);

			ensure!(sources.len() < T::MaxClaims::get(), Error::<T>::ExceedsMaxClaims);

			// TODO: How to do this with map?
			let mut post_weight: Weight = 0;
			for s in sources.into_iter() {
				let source = T::Lookup::lookup(s)?;
				post_weight += Self::migrate_user(&source, &target)?;
			}

			Ok(Some(post_weight).into())
		}

		/// Transfer KILT locked tokens to another account similar to
		/// `pallet_vesting::vested_transfer`.
		///
		/// Expects the source to have a KILT balance lock and at least the
		/// specified amount available as balance locked with LockId
		/// `KILT_LAUNCH_ID`.
		///
		/// Calls `migrate_kilt_balance_lock` internally.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(2, 3))]
		#[transactional]
		pub(super) fn locked_transfer(
			origin: OriginFor<T>,
			target: <T::Lookup as StaticLookup>::Source,
			amount: <T as pallet_balances::Config>::Balance,
		) -> DispatchResultWithPostInfo {
			let source = ensure_signed(origin)?;
			let target = T::Lookup::lookup(target)?;

			// The correct check would be `ensure_can_withdraw` but since we expect `amount`
			// to be locked, we just check the total balance until we remove the lock below
			ensure!(
				<pallet_balances::Pallet<T>>::total_balance(&source) >= amount,
				Error::<T>::InsufficientBalance
			);
			ensure!(
				<BalanceLocks<T>>::get(&source).is_some(),
				Error::<T>::BalanceLockNotFound
			);

			let locks = Locks::<T>::get(&source);
			ensure!(locks.len() > 0, Error::<T>::ExpectedLocks);

			if let Some(lock) = locks
				.iter()
				.find(|BalanceLock::<<T as pallet_balances::Config>::Balance> { id, .. }| id == &KILT_LAUNCH_ID)
			{
				ensure!(lock.amount >= amount, Error::<T>::InsufficientLockedBalance);

				// We can substract because of the above check
				let amount_new = T::ExistentialDeposit::get().max(lock.amount - amount);

				if amount_new <= T::ExistentialDeposit::get() {
					// If the lock equals the ExistentialDeposit, we want to remove the lock because
					// if amount_new == 0, `set_lock` would be no-op
					<pallet_balances::Pallet<T>>::remove_lock(KILT_LAUNCH_ID, &source);

					// Transfer amount + dust to target
					let _ = <pallet_balances::Pallet<T> as Currency<T::AccountId>>::transfer(
						&source,
						&target,
						lock.amount,
						AllowDeath,
					)?;
				} else {
					// Reduce source's lock amount to enable token transfer
					<pallet_balances::Pallet<T>>::set_lock(
						KILT_LAUNCH_ID,
						&source,
						amount_new,
						WithdrawReasons::except(WithdrawReasons::TRANSACTION_PAYMENT),
					);

					// Transfer amount to target
					let _ = <pallet_balances::Pallet<T> as Currency<T::AccountId>>::transfer(
						&source, &target, amount, AllowDeath,
					)?;
				}

				Self::deposit_event(Event::LockedTransfer(source.clone(), amount, target.clone()));

				// Set locks in target and remove/update storage entries for source
				Ok(Some(Self::migrate_kilt_balance_lock(&source, &target, Some(amount))?).into())
			} else {
				frame_support::fail!(Error::<T>::BalanceLockNotFound)
			}
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Remove KILT balance locks for the specified block
	fn unlock_balance(block: T::BlockNumber) -> Weight {
		if let Some(unlocking_balance) = <UnlockingAt<T>>::take(block) {
			// Remove locks for all accounts
			for account in unlocking_balance.iter() {
				<pallet_balances::Pallet<T>>::remove_lock(KILT_LAUNCH_ID, account);
				<BalanceLocks<T>>::remove(account);
			}

			Self::deposit_event(Event::Unlocked(block, unlocking_balance.len() as u64));
			T::DbWeight::get().reads_writes(1, (2 * unlocking_balance.len() + 1) as u64)
		} else {
			T::DbWeight::get().reads(1)
		}
	}

	/// Transfers all balance of the source to the target address and sets up
	/// vesting or the custom KILT balance lock if any of the two were set up
	/// for the source address.
	///
	/// Note: Expects the source address to be an unowned address which was set
	/// up in the Genesis block and should be claimed by a user to migrate to
	/// their account.
	fn migrate_user(source: &T::AccountId, target: &T::AccountId) -> Result<Weight, DispatchError> {
		// There should be no locks for the source address
		ensure!(Locks::<T>::get(source).len().is_zero(), Error::<T>::UnexpectedLocks);

		// Transfer to target addess
		let amount = <pallet_balances::Pallet<T>>::total_balance(source);
		let _ = <pallet_balances::Pallet<T> as Currency<T::AccountId>>::transfer(source, target, amount, AllowDeath)?;

		// Migrate vesting info and set the corresponding vesting lock if necessary
		let mut post_weight: Weight = Self::migrate_vesting(source, target)?;

		// Set the KILT custom lock if necessary
		post_weight += Self::migrate_kilt_balance_lock(source, target, None)?;

		// TODO: Add meaningful information
		Ok(post_weight)
	}

	/// Migrate the vesting schedule from one account to another if it was set
	/// in the GenesisBlock and set the corresponding vesting lock.
	///
	/// We already unlock all available funds until the current block.
	fn migrate_vesting(source: &T::AccountId, target: &T::AccountId) -> Result<Weight, DispatchError> {
		if let Some(source_vesting) = Vesting::<T>::take(source) {
			// Check for an already existing vesting schedule on the target account
			// which would be the case if the claimer requests migration from multiple
			// source accounts to the same target
			let vesting = if let Some(VestingInfo {
				locked,
				per_block,
				starting_block,
			}) = Vesting::<T>::take(&target)
			{
				// Should never throw because all source accounts start vesting in genesis block
				ensure!(
					starting_block == source_vesting.starting_block,
					Error::<T>::ConflictingVestingStarts
				);
				VestingInfo {
					// We can simply sum `locked` and `per_block` because of the above requirement
					locked: locked.saturating_add(source_vesting.locked),
					per_block: per_block.saturating_add(source_vesting.per_block),
					starting_block,
				}
			}
			// If vesting hasn't been set up for target account, we can default to the one of the source
			// account
			else {
				source_vesting
			};
			Vesting::<T>::insert(target, vesting);
			// Only lock funds from now until vesting expires.
			// Enables the user to have funds before actively calling `vest` if claimed
			// after the genesis block.
			//
			// Logic was taken from pallet_vesting.

			// TODO: Check whether we want to switch to
			// WithdrawReasons::except(WithdrawReasons::TRANSACTION_PAYMENT) to allow for tx
			// fees to be paid from vesting-locked tokens
			let reasons = WithdrawReasons::TRANSFER | WithdrawReasons::RESERVE;
			let now = <frame_system::Pallet<T>>::block_number();
			let locked_now = vesting.locked_at::<<T as pallet_vesting::Config>::BlockNumberToBalance>(now);
			<<T as pallet_vesting::Config>::Currency as LockableCurrency<T::AccountId>>::set_lock(
				VESTING_ID,
				target,
				locked_now.into(),
				reasons,
			);
			Self::deposit_event(Event::AddedVesting(target.clone(), vesting.per_block, vesting.locked));
		}
		// TODO: Add meaningful weight
		Ok(0)
	}

	/// Set the custom KILT balance lock for the target address which should
	/// always be a user-owned account address.
	///
	/// Can be called during the migration of unowned "genesis" addresses to
	/// user-owned account addresses in `migrate_user` as well when an account
	/// wants to transfer their locked tokens to another oner in
	/// `locked_transfer`.
	fn migrate_kilt_balance_lock(
		source: &T::AccountId,
		target: &T::AccountId,
		// Only used for `locked_transfer`
		max_amount: Option<<T as pallet_balances::Config>::Balance>,
	) -> Result<Weight, DispatchError> {
		if let Some(LockedBalance::<T> {
			block: unlock_block,
			amount: source_amount,
		}) = <BalanceLocks<T>>::get(&source)
		{
			// In case of a `locked_transfer`, we might only want to unlock a certain amount
			// Otherwise, this will always be the source's locked amount
			let max_add_amount = source_amount.min(max_amount.unwrap_or(source_amount));

			// Check for an already existing custom KILT balance lock on the target
			// account which would be the case if the claimer requests migration from
			// multiple source accounts to the same target
			let target_amount = if let Some(target_lock) = <BalanceLocks<T>>::take(&target) {
				// Should never throw because there is a single locking periods (6 months)
				ensure!(target_lock.block == unlock_block, Error::<T>::ConflictingLockingBlocks);

				// We don't need to append `UnlockingAt` because we require both locks to end at
				// the same block
				// We can simply sum `amount` because of the above requirement
				target_lock.amount.saturating_add(max_add_amount)
			}
			// If no custom lock has been set up for target account, we can default to the one of the source
			// account and append it to `UnlockingAt`
			else {
				<UnlockingAt<T>>::append(unlock_block, &target);
				max_add_amount
			};

			// Set target lock in case another account should be migrated to this target
			// address at a later stage
			<BalanceLocks<T>>::insert(
				target,
				LockedBalance::<T> {
					amount: target_amount,
					block: unlock_block,
				},
			);
			// Allow transaction fees to be paid from locked balance, e.g., prohibit all
			// withdraws except `WithdrawReasons::TRANSACTION_PAYMENT`
			<pallet_balances::Pallet<T>>::set_lock(
				KILT_LAUNCH_ID,
				&target,
				target_amount,
				WithdrawReasons::except(WithdrawReasons::TRANSACTION_PAYMENT),
			);

			// Update or remove lock storage items corresponding to the source address
			if max_add_amount == source_amount {
				<BalanceLocks<T>>::remove(&source);

				// Only needs to be handled in the case of a `locked_transfer`, e.g., when
				// `max_amount` is set because else the source address is never added to
				// `UnlockingAt`
				if max_amount.is_some() {
					let remove_source_map: Vec<T::AccountId> = <UnlockingAt<T>>::take(unlock_block)
						.unwrap_or_default()
						.into_iter()
						.filter_map(|acc_id| if &acc_id == source { None } else { Some(acc_id) })
						.collect();
					<UnlockingAt<T>>::insert(unlock_block, remove_source_map);
				}
			} else {
				// Reduce the locked amount
				//
				// Note: The update of the real balance lock with id `KILT_LAUNCH_ID` already
				// happens in `locked_transfer` because it is required for the token transfer
				<BalanceLocks<T>>::insert(
					&source,
					LockedBalance::<T> {
						block: unlock_block,
						amount: source_amount.saturating_sub(max_add_amount),
					},
				)
			}

			Self::deposit_event(Event::AddedKiltLock(target.clone(), target_amount, unlock_block));
		}
		// TODO: Add meaningful weight
		Ok(0)
	}
}
