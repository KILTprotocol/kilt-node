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
//! A simple pallet providing means of setting up KILT balance locks and
//! vesting schedules for unowned accounts in the genesis block. These should
//! later be migrated to user-owned accounts via the extrinsic
//! `migrate_genesis_account` which has to be signed by a specific account
//! called `TransferAccount`. The latter is also set in the genesis block
//! and can be changed by calling the sudo extrinsic `change_transfer_account`.
//!
//! - [`Config`]
//! - [`Call`]
//! - [`Pallet`]
//!
//! ## Overview
//!
//! The KILT Launch pallet provides functions for:
//!
//! - Setting vesting information and KILT balance lock for unowned accounts in
//!   genesis block.
//! - Migrating vesting/KILT balance lock from unowned accounts to user-owned
//!   accounts.
//! - Transfer locked tokens from user-owned account to another. NOTE: This will
//!   be made available shortly before we remove the sudo key.
//! - Forcedly (requires sudo) changing the `TransferAccount`.
//! - Forcedly (requires sudo) removing the KILT balance lock.
//!
//! ### Terminology
//!
//! - **Lock:** A freeze on a specified amount of an account's free balance
//!   until a specified block number. Multiple locks always operate over the
//!   same funds, so they "overlay" rather than "stack".
//!
//! - **KILT balance lock:** A Lock with a KILT specific identifier which is
//!   automatically removed when reaching the specified block number.
//!
//! - **Unowned account:** An endowed account for which potentially vesting or
//!   the KILT balance lock is set up in the genesis block.
//!
//! - **User-owned account:** A regular account which was created by an entity
//!   which wants to claim their tokens (potentially with vesting/KILT balance
//!   lock) from an unowned account.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `migrate_genesis_account` - Migrate vesting or the KILT balance lock from
//!   an unowned account to a user-owned account. Requires signature of a
//!   special account `TransferAccount` which does not have any other super
//!   powers.
//! - `migrate_multiple_genesis_accounts` - Migrate vesting or the KILT balance
//!   lock from a list of unowned accounts to the same target user-owned
//!   account. Requires signature of a special account `TransferAccount` which
//!   does not have any other super powers.
//! - `locked_transfer` - Transfer locked tokens from one user-owned account to
//!   another user-owned account. This will be made available shortly before
//!   removing the sudo key because the purpose of the lock to disable
//!   transferability of the amount.
//! - `change_transfer_account` - Change the transfer account. Can only be
//!   called by sudo.
//! - `force_unlock` - Remove all locks for a given block. Can only be called by
//!   sudo.
//!
//! ## Genesis config
//!
//! The KiltLaunch pallet depends on the [`GenesisConfig`].
//!
//! ## Assumptions
//!
//! * All accounts provided with balance and potentially vesting or a KILT
//!   balance lock in the genesis block are not owned by anyone and have to be
//!   migrated to accounts which are owned by users.
//! * All unowned accounts have either vesting, the KILT balance lock or neither
//!   of both. This assumption is neither checked, nor forced, nor does any code
//!   break if it does not hold true.
//! * Vesting starts at genesis block for all unowned addresses which should be
//!   migrated to user-owned accounts. This assumption is checked during
//!   migration.
//! * All KILT balance locks end at the same block for all unowned addresses
//!   which should be migrated to user-owned accounts. This assumption is
//!   checked during migration and locked transfer.
//! * The total number of accounts for which a KILT balance lock is set up is at
//!   most `MaxClaims`, for us it will be ~6. This assumption is not checked
//!   when appending to `UnlockedAt`.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]

pub use pallet::*;
#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;
pub mod default_weights;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	pub use crate::default_weights::WeightInfo;
	#[cfg(feature = "std")]
	use frame_support::traits::GenesisBuild;
	use frame_support::{
		dispatch::DispatchResultWithPostInfo,
		inherent::Vec,
		pallet_prelude::*,
		sp_runtime::traits::{StaticLookup, Zero},
		storage::types::StorageMap,
		traits::{Currency, ExistenceRequirement::AllowDeath, Get, LockIdentifier, LockableCurrency, WithdrawReasons},
		transactional,
	};
	use frame_system::pallet_prelude::*;
	use pallet_balances::{BalanceLock, Locks};
	use pallet_vesting::{Vesting, VestingInfo};
	use sp_runtime::traits::{CheckedDiv, Convert, SaturatedConversion, Saturating};
	use sp_std::convert::TryInto;

	pub const KILT_LAUNCH_ID: LockIdentifier = *b"kiltlnch";
	pub const VESTING_ID: LockIdentifier = *b"vesting ";

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
		///
		/// Note: Benchmarks will need to be re-run and weights adjusted if this
		/// changes.
		#[pallet::constant]
		type MaxClaims: Get<u32>;

		/// Amount of Balance which will be made available for each account
		/// which has either vesting or locking such that transaction fees can
		/// be paid from this.
		type UsableBalance: Get<<Self as pallet_balances::Config>::Balance>;

		/// Weight information for extrinsics in this pallet.
		type WeightInfo: WeightInfo;
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
			// * who - Account which we are setting the custom lock for
			// * length - Number of blocks from  until removal of the lock
			// * locked - Number of tokens which are locked
			for (ref who, length, locked) in self.balance_locks.iter() {
				if !length.is_zero() {
					let balance = <pallet_balances::Pallet<T>>::free_balance(who);
					assert!(!balance.is_zero(), "Currencies must be init'd before locking");
					assert!(
						balance >= *locked,
						"Locked balance must not exceed total balance for address {:?}",
						who.to_string()
					);
					assert!(
						!<BalanceLocks<T>>::contains_key(who),
						"Account with address {:?} must not occur twice in locking",
						who.to_string()
					);

					// Add unlock block to storage
					<BalanceLocks<T>>::insert(
						who,
						LockedBalance::<T> {
							block: *length,
							amount: (*locked).saturating_sub(T::UsableBalance::get()),
						},
					);
					// Instead of setting the lock now, we do so in
					// `migrate_genesis_account`, see there for explanation
				}
				// Add all accounts to UnownedAccount storage
				<UnownedAccount<T>>::insert(&who, ());
			}

			// Generate initial vesting configuration, taken from pallet_vesting
			// * who - Account which we are generating vesting configuration for
			// * begin - Block when the account will start to vest
			// * length - Number of blocks from `begin` until fully vested
			for &(ref who, length, locked) in self.vesting.iter() {
				if !length.is_zero() {
					let balance = <<T as pallet_vesting::Config>::Currency as Currency<
						<T as frame_system::Config>::AccountId,
					>>::free_balance(who);
					assert!(!balance.is_zero(), "Currencies must be init'd before vesting");
					assert!(
						balance >= locked,
						"Vested balance must not exceed total balance for address {:?}",
						who.to_string()
					);
					assert!(
						!<Vesting<T>>::contains_key(who),
						"Account with address {:?} must not occur twice in vesting",
						who.to_string()
					);

					let length_as_balance = T::BlockNumberToBalance::convert(length);
					let per_block = locked.checked_div(&length_as_balance).unwrap_or(locked);

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
				// Add all accounts to UnownedAccount storage
				<UnownedAccount<T>>::insert(&who, ());
			}

			// Set the transfer account which has a subset of the powers of root
			<TransferAccount<T>>::put(self.transfer_account.clone());
		}
	}

	/// Account which is permitted to do token transfers in PoA phase.
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
		BoundedVec<<T as frame_system::Config>::AccountId, <T as Config>::MaxClaims>,
	>;

	/// Maps an account id to the (block, balance) pair in which the latter can
	/// be unlocked.
	///
	/// Required for the claiming process.
	#[pallet::storage]
	#[pallet::getter(fn get_unlocking_block)]
	pub type BalanceLocks<T> =
		StorageMap<_, Blake2_128Concat, <T as frame_system::Config>::AccountId, LockedBalance<T>>;

	/// Maps an unowned account id to an empty value which reflects whether it
	/// is a genesis account which should be migrated, if it exists.
	///
	/// Required for the claiming process.
	#[pallet::storage]
	#[pallet::getter(fn unowned_account)]
	pub type UnownedAccount<T> =
		StorageMap<_, Blake2_128Concat, <T as frame_system::Config>::AccountId, (), OptionQuery>;

	#[pallet::event]
	#[pallet::metadata(T::BlockNumber = "BlockNumber", T::AccountId = "AccountId", T::Balance = "Balance")]
	#[pallet::generate_deposit(pub fn deposit_event)]
	pub enum Event<T: Config> {
		/// A KILT balance lock has been removed in the corresponding block.
		/// \[block, len\]
		Unlocked(T::BlockNumber, u32),
		/// An account transferred their locked balance to another account.
		/// \[from, value, target\]
		LockedTransfer(T::AccountId, T::Balance, T::AccountId),
		/// A KILT balance lock has been set. \[who, value, until\]
		AddedKiltLock(T::AccountId, T::Balance, T::BlockNumber),
		/// Vesting has been added to an account. \[who, per_block, total\]
		AddedVesting(T::AccountId, BalanceOf<T>, BalanceOf<T>),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// The source address does not have KILT balance lock which is
		/// required for `locked_transfer`.
		BalanceLockNotFound,
		/// The source and destination address have limits for their custom KILT
		/// balance lock and thus cannot be merged. Should never be thrown.
		ConflictingLockingBlocks,
		/// The source and destination address differ in their vesting starting
		/// blocks and thus cannot be merged. Should never be thrown.
		ConflictingVestingStarts,
		/// When migrating multiple accounts to the same target, the size of the
		/// list of source addresses should never exceed `MaxClaims`.
		MaxClaimsExceeded,
		/// The source address does not have any balance lock at all which is
		/// required for `locked_transfer`.
		ExpectedLocks,
		/// The source address has less balance available than the locked amount
		/// which should be transferred in `locked_transfer`.
		InsufficientBalance,
		/// The source address has less locked balance than the amount which
		/// should be transferred in `locked_transfer`.
		InsufficientLockedBalance,
		/// The source address is not a valid address which was set up as an
		/// unowned account in the genesis build.
		NotUnownedAccount,
		/// The target address should not be the source address.
		SameDestination,
		/// The signing account is not the transfer account.
		Unauthorized,
		/// The source address has a balance lock and thus cannot be migrated.
		UnexpectedLocks,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(now: T::BlockNumber) -> Weight {
			Self::unlock_balance(now).into()
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Forcedly remove KILT balance locks via sudo for the specified block
		/// number.
		///
		/// The dispatch origin must be Root.
		///
		/// Emits `Unlocked`.
		///
		/// # <weight>
		/// - The transaction's complexity is proportional to the size of
		///   storage entries in `UnlockingAt` (N) which is practically uncapped
		///   but in theory it should be `MaxClaims` at most.
		/// ---------
		/// Weight: O(N) where N is the number of accounts for which the lock
		/// will be removed for the given block.
		/// - Reads: UnlockingAt, [Origin Account]
		/// - Kills: UnlockingAt (if N > 0), Locks (if N > 0), BalanceLocks (if
		///   N > 0)
		/// # </weight>
		#[pallet::weight(<T as pallet::Config>::WeightInfo::force_unlock(T::MaxClaims::get()))]
		pub fn force_unlock(origin: OriginFor<T>, block: T::BlockNumber) -> DispatchResultWithPostInfo {
			ensure_root(origin)?;

			let weight = <T as pallet::Config>::WeightInfo::force_unlock(Self::unlock_balance(block));

			Ok(Some(weight).into())
		}

		/// Forcedly change the transfer account to the specified account.
		///
		/// The dispatch origin must be Root.
		///
		/// # <weight>
		/// Weight: O(1)
		/// - Reads: [Origin Account]
		/// - Writes: TransferAccount
		/// # </weight>
		#[pallet::weight(<T as pallet::Config>::WeightInfo::change_transfer_account())]
		pub fn change_transfer_account(
			origin: OriginFor<T>,
			transfer_account: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResult {
			ensure_root(origin)?;
			let transfer_account = T::Lookup::lookup(transfer_account)?;

			<TransferAccount<T>>::put(transfer_account);

			Ok(())
		}

		/// Transfer tokens and vesting information or the KILT balance lock
		/// from an unowned source address to an account owned by the target.
		///
		/// If vesting info or a KILT balance lock has been set up for the
		/// source account in the genesis block via `GenesisBuild`, then
		/// the corresponding locked/vested information and balance is migrated
		/// automatically. Please note that even though this extrinsic supports
		/// migrating both the KILT balance lock as well as vesting in one call,
		/// all source accounts should only contain either a KILT balance lock
		/// or vesting.
		///
		/// Additionally, for vesting we already unlock the
		/// usable balance until the current block. This should enable the user
		/// to pay the transaction fees for the next call of `vest` which is
		/// always required to be explicitly called in order to unlock (more)
		/// balance from vesting.
		///
		/// NOTE: Setting the KILT balance lock actually only occurs in this
		/// call (and not when building the genesis block in `GenesisBuild`) to
		/// avoid overhead from handling locks when migrating. We can do so
		/// because all target accounts are not owned by anyone and thus these
		/// cannot sign and/or call any extrinsics.
		///
		/// The dispatch origin must be TransferAccount.
		///
		/// Emits either `AddedVesting` or `AddedKiltLock`.
		///
		/// # <weight>
		/// Weight: O(1)
		/// - Reads: [Origin Account], TransferAccount, Locks, Balance, Vesting,
		///   BalanceLocks
		/// - Writes: Locks, Balance, UnownedAccount, Vesting (if source is
		///   vesting), BalanceLocks (if source is locking), UnlockingAt (if
		///   source is locking)
		/// - Kills (for source): Locks, Balance, UnownedAccount, Vesting (if
		///   source is vesting), BalanceLocks (if source is locking)
		/// # </weight>
		#[pallet::weight(<T as pallet::Config>::WeightInfo::migrate_genesis_account_vesting().max(<T as pallet::Config>::WeightInfo::migrate_genesis_account_locking()))]
		#[transactional]
		pub fn migrate_genesis_account(
			origin: OriginFor<T>,
			source: <T::Lookup as StaticLookup>::Source,
			target: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;

			// The extrinsic has to be called by the TransferAccount
			ensure!(Some(who) == <TransferAccount<T>>::get(), Error::<T>::Unauthorized);

			let source = T::Lookup::lookup(source)?;
			let target = T::Lookup::lookup(target)?;

			ensure!(source != target, Error::<T>::SameDestination);
			ensure!(
				<UnownedAccount<T>>::contains_key(&source),
				Error::<T>::NotUnownedAccount
			);

			Ok(Some(Self::migrate_user(&source, &target)?).into())
		}

		/// Transfer all balances, vesting information and KILT balance locks
		/// from multiple source addresses to the same target address.
		///
		/// See `migrate_genesis_account` for details as we run the same logic
		/// for each source address.
		///
		/// The dispatch origin must be TransferAccount.
		///
		/// Emits N events which are either `AddedVesting` or `AddedKiltLock`.
		///
		/// # <weight>
		/// - The transaction's complexity is proportional to the size of
		///   `sources` (N) which is capped at CompactAssignments::LIMIT
		///   (MaxClaims)
		/// ---------
		/// Weight: O(N) where N is the number of source addresses.
		/// - Reads: [Origin Account], TransferAccount, UnownedAccount, Locks,
		///   Balance, Vesting, BalanceLocks
		/// - Writes: Locks, Balance, Vesting (if any source is vesting),
		///   BalanceLocks (if aby source is locking), UnlockingAt (if any
		///   source is locking)
		/// - Kills (for sources): Locks, Balance, UnownedAccount, Vesting (if
		///   any source is vesting), BalanceLocks (if any source is locking)
		/// # </weight>
		#[pallet::weight(<T as pallet::Config>::WeightInfo::migrate_multiple_genesis_accounts_vesting(T::MaxClaims::get()).max(<T as pallet::Config>::WeightInfo::migrate_multiple_genesis_accounts_locking(T::MaxClaims::get())))]
		#[transactional]
		pub fn migrate_multiple_genesis_accounts(
			origin: OriginFor<T>,
			sources: Vec<<T::Lookup as StaticLookup>::Source>,
			target: <T::Lookup as StaticLookup>::Source,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
			let target = T::Lookup::lookup(target)?;

			// The extrinsic has to be called by the TransferAccount
			ensure!(Some(who) == <TransferAccount<T>>::get(), Error::<T>::Unauthorized);

			ensure!(
				sources.len() < T::MaxClaims::get().saturated_into::<usize>(),
				Error::<T>::MaxClaimsExceeded
			);

			let mut post_weight: Weight = 0;
			for s in sources.clone().into_iter() {
				let source = T::Lookup::lookup(s)?;
				ensure!(source != target, Error::<T>::SameDestination);
				ensure!(
					<UnownedAccount<T>>::contains_key(&source),
					Error::<T>::NotUnownedAccount
				);
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
		///
		/// Emits `LockedTransfer` and if target does not have KILT balance
		/// lockup prior to transfer `AddedKiltLock`.
		///
		/// # <weight>
		/// Weight: O(1)
		/// - Reads: [Origin Account], Locks, Balance, BalanceLocks, UnlockingAt
		/// - Writes: Locks, Balance, BalanceLocks, UnlockingAt
		/// - Kills (if source transfers all locked balance): Locks,
		///   BalanceLocks, UnlockingAt
		/// # </weight>
		#[pallet::weight(<T as pallet::Config>::WeightInfo::locked_transfer())]
		#[transactional]
		pub fn locked_transfer(
			origin: OriginFor<T>,
			target: <T::Lookup as StaticLookup>::Source,
			amount: <T as pallet_balances::Config>::Balance,
		) -> DispatchResultWithPostInfo {
			let source = ensure_signed(origin)?;
			let target = T::Lookup::lookup(target)?;

			ensure!(target != source, Error::<T>::SameDestination);

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
			ensure!(!locks.is_empty(), Error::<T>::ExpectedLocks);

			if let Some(lock) = locks
				.iter()
				.find(|BalanceLock::<<T as pallet_balances::Config>::Balance> { id, .. }| id == &KILT_LAUNCH_ID)
			{
				ensure!(lock.amount >= amount, Error::<T>::InsufficientLockedBalance);

				// We can subtract because of the above check, but let's be safe
				let amount_new = lock.amount.saturating_sub(amount);

				if amount_new <= T::ExistentialDeposit::get() {
					// If the lock equals the ExistentialDeposit, we want to remove the lock because
					// if amount_new == 0, `set_lock` would be no-op
					<pallet_balances::Pallet<T>>::remove_lock(KILT_LAUNCH_ID, &source);

					// Transfer amount + dust to target
					<pallet_balances::Pallet<T> as Currency<T::AccountId>>::transfer(
						&source,
						&target,
						lock.amount,
						AllowDeath,
					)?;
				} else {
					// Reduce source's lock amount to enable token transfer
					<pallet_balances::Pallet<T>>::set_lock(KILT_LAUNCH_ID, &source, amount_new, WithdrawReasons::all());

					// Transfer amount to target
					<pallet_balances::Pallet<T> as Currency<T::AccountId>>::transfer(
						&source, &target, amount, AllowDeath,
					)?;
				}

				Self::deposit_event(Event::LockedTransfer(source.clone(), amount, target.clone()));

				// Set locks in target and remove/update storage entries for source
				Ok(Some(Self::migrate_kilt_balance_lock(&source, &target, Some(amount))?).into())
			} else {
				Err(Error::<T>::BalanceLockNotFound.into())
			}
		}
	}

	impl<T: Config> Pallet<T> {
		/// Remove KILT balance locks for the specified block
		fn unlock_balance(block: T::BlockNumber) -> u32 {
			if let Some(unlocking_balance) = <UnlockingAt<T>>::take(block) {
				// Remove locks for all accounts
				for account in unlocking_balance.iter() {
					<pallet_balances::Pallet<T>>::remove_lock(KILT_LAUNCH_ID, account);
					<BalanceLocks<T>>::remove(account);
				}

				Self::deposit_event(Event::Unlocked(block, unlocking_balance.len().saturated_into::<u32>()));
				// Safe because `UnlockingAt` will be ~6 in our case
				unlocking_balance.len().saturated_into::<u32>()
			} else {
				0
			}
		}

		/// Transfers all balance of the source to the target address and sets
		/// up vesting or the KILT balance lock if any of the two were set up
		/// for the source address.
		///
		/// Note: Expects the source address to be an unowned address which was
		/// set up in the genesis block via `GenesisBuild` and should be claimed
		/// by a user to migrate to their account.
		fn migrate_user(source: &T::AccountId, target: &T::AccountId) -> Result<Weight, DispatchError> {
			// There should be no locks for the source address
			ensure!(Locks::<T>::get(source).len().is_zero(), Error::<T>::UnexpectedLocks);

			// Transfer to target address
			let amount = <pallet_balances::Pallet<T>>::total_balance(source);
			<pallet_balances::Pallet<T> as Currency<T::AccountId>>::transfer(source, target, amount, AllowDeath)?;

			// Migrate vesting info and set the corresponding vesting lock if necessary
			let mut post_weight: Weight = Self::migrate_vesting(source, target)?;

			// Set the KILT custom lock if necessary
			post_weight += Self::migrate_kilt_balance_lock(source, target, None)?;

			<UnownedAccount<T>>::remove(&source);
			post_weight += T::DbWeight::get().writes(1);

			Ok(post_weight)
		}

		/// Migrate the vesting schedule from one account to another, if it was
		/// set in the genesis block via `GenesisBuild`, and set the
		/// corresponding vesting lock.
		///
		/// We already unlock all available funds between the starting and the
		/// current block. This enables the user to be able to pay for
		/// transactions. One of these would be `pallet_vesting::vest()` which
		/// has to be called actively to unlock more of the vested funds.
		fn migrate_vesting(source: &T::AccountId, target: &T::AccountId) -> Result<Weight, DispatchError> {
			let source_vesting = if let Some(source_vesting) = Vesting::<T>::take(source) {
				source_vesting
			} else {
				return Ok(T::DbWeight::get().reads(1));
			};

			// Check for an already existing vesting schedule for the target account
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
			} else {
				// If vesting hasn't been set up for target account, we can default to the one
				// of the source account
				source_vesting
			};
			Vesting::<T>::insert(target, vesting);
			// Only lock funds from now until vesting expires.
			// Enables the user to have funds before actively calling `vest` if claimed
			// after the genesis block.
			//
			// Logic was taken from pallet_vesting.

			// Disallow transfers and reserves from vested tokens which are still locked
			let reasons = WithdrawReasons::TRANSFER | WithdrawReasons::RESERVE;
			let now = <frame_system::Pallet<T>>::block_number();
			let locked_now = vesting.locked_at::<<T as pallet_vesting::Config>::BlockNumberToBalance>(now);
			<<T as pallet_vesting::Config>::Currency as LockableCurrency<T::AccountId>>::set_lock(
				VESTING_ID, target, locked_now, reasons,
			);
			Self::deposit_event(Event::AddedVesting(target.clone(), vesting.per_block, vesting.locked));
			Ok(<T as pallet::Config>::WeightInfo::migrate_genesis_account_vesting())
		}

		/// Set the KILT balance lock for the target address which should
		/// always be a user-owned account address.
		///
		/// Can be called during the migration of unowned "genesis" addresses to
		/// user-owned account addresses in `migrate_user` as well when an
		/// account wants to transfer their locked tokens to another account in
		/// `locked_transfer`.
		fn migrate_kilt_balance_lock(
			source: &T::AccountId,
			target: &T::AccountId,
			// Only used for `locked_transfer`, e.g., it is `None` for migration
			max_amount: Option<<T as pallet_balances::Config>::Balance>,
		) -> Result<Weight, DispatchError> {
			let LockedBalance::<T> {
				block: unlock_block,
				amount: source_amount,
			} = if let Some(lock) = <BalanceLocks<T>>::get(&source) {
				lock
			} else {
				return Ok(T::DbWeight::get().reads(1));
			};

			// In case of a `locked_transfer`, we might only want to unlock a certain amount
			// Otherwise, this will always be the source's locked amount
			let max_add_amount = source_amount.min(max_amount.unwrap_or(source_amount));

			// We don't need to transfer any locks if the lock already expired. So we bail
			// early
			if unlock_block <= frame_system::Pallet::<T>::block_number() {
				// But we still need to reduce the old lock or remove it, if it's consumed
				// completely.
				if max_add_amount == source_amount {
					<BalanceLocks<T>>::remove(&source);
				} else {
					<BalanceLocks<T>>::insert(
						&source,
						LockedBalance::<T> {
							block: unlock_block,
							amount: source_amount.saturating_sub(max_add_amount),
						},
					)
				}
				return Ok(T::DbWeight::get().reads(1));
			}

			// Check for an already existing KILT balance lock on the target
			// account which would be the case if the claimer requests migration from
			// multiple source accounts to the same target
			let target_amount = if let Some(target_lock) = <BalanceLocks<T>>::take(&target) {
				// Should never throw because there is a single locking period (6 months)
				ensure!(target_lock.block == unlock_block, Error::<T>::ConflictingLockingBlocks);

				// We don't need to append `UnlockingAt` because we require both locks to end at
				// the same block
				// We can simply sum `amount` because of the above requirement and the check
				// that source != target in the corresponding extrinsics
				target_lock.amount.saturating_add(max_add_amount)
			} else {
				// If no custom lock has been set up for target account, we can default to the
				// one of the source account and append it to `UnlockingAt`
				<UnlockingAt<T>>::try_append(unlock_block, &target).map_err(|_| Error::<T>::MaxClaimsExceeded)?;
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
			// Disallow anything from being paid by custom lock
			<pallet_balances::Pallet<T>>::set_lock(KILT_LAUNCH_ID, target, target_amount, WithdrawReasons::all());

			// Update or remove lock storage items corresponding to the source address
			if max_add_amount == source_amount {
				<BalanceLocks<T>>::remove(&source);

				// Only needs to be handled in the case of a `locked_transfer`, e.g., when
				// `max_amount` is set because else the source address is never added to
				// `UnlockingAt`
				if max_amount.is_some() {
					<UnlockingAt<T>>::try_mutate(unlock_block, |maybe_bv| -> DispatchResult {
						if let Some(bv) = maybe_bv {
							*bv = bv
								.clone()
								.into_inner()
								.into_iter()
								.filter(|acc_id| acc_id != source)
								.collect::<Vec<T::AccountId>>()
								.try_into()
								.map_err(|_| Error::<T>::MaxClaimsExceeded)?
						}
						Ok(())
					})?;
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
			Ok(<T as pallet::Config>::WeightInfo::migrate_genesis_account_locking())
		}
	}
}
