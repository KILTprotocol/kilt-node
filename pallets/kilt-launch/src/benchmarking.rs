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

use super::*;

#[allow(unused)]
use crate::{BalanceLocks, BalanceOf, LockedBalance, Pallet as KiltLaunch, UnownedAccount, KILT_LAUNCH_ID};
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, vec, whitelist_account, Zero};
use frame_support::{
	inherent::Vec,
	traits::{Currency, Get},
};
use frame_system::{Pallet as System, RawOrigin};
use kilt_primitives::{constants::KILT, Balance};
use pallet_balances::Locks;
use pallet_vesting::{Vesting, VestingInfo};
use sp_runtime::traits::StaticLookup;
use sp_std::convert::TryFrom;

const SEED: u32 = 0;
const AMOUNT: Balance = KILT;
const PER_BLOCK: u32 = 100;
const UNLOCK_BLOCK: u32 = 1337;

type Lookup<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;

/// Account to lookup type of system trait.
fn as_lookup<T: Config>(account: T::AccountId) -> Lookup<T> {
	T::Lookup::unlookup(account)
}

type GenesisSetup<T> = (
	(
		<T as frame_system::Config>::AccountId,
		<<T as frame_system::Config>::Lookup as StaticLookup>::Source,
	),
	Vec<(
		<T as frame_system::Config>::AccountId,
		<<T as frame_system::Config>::Lookup as StaticLookup>::Source,
	)>,
	Vec<(
		<T as frame_system::Config>::AccountId,
		<<T as frame_system::Config>::Lookup as StaticLookup>::Source,
	)>,
);

/// Mock the Pallet's GenesisBuild and return pairs consisting of AccountId and
/// LookupSource for the transfer account, `n` vesting addresses and `n` locking
/// addresses.
fn genesis_setup<T: Config>(n: u32) -> Result<GenesisSetup<T>, &'static str>
where
	Balance: Into<<T as pallet_balances::Config>::Balance>,
	Balance:
		Into<<<T as pallet_vesting::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance>,
{
	System::<T>::set_block_number(0u32.into());

	// Setup transfer account
	let transfer: T::AccountId = account("transfer", 0, SEED);
	let transfer_lookup: <T::Lookup as StaticLookup>::Source = as_lookup::<T>(transfer.clone());
	KiltLaunch::<T>::change_transfer_account(RawOrigin::Root.into(), transfer_lookup.clone())?;

	// Create `n` genesis accounts each for vesting and locking
	let (v_accs, l_accs) = (1..=n).into_iter().fold((vec![], vec![]), |mut acc, i| {
		let vest_acc: T::AccountId = account("vesting_{:?}", i, SEED);
		let lock_acc: T::AccountId = account("locking", i, SEED);
		let vest_lookup: <T::Lookup as StaticLookup>::Source = as_lookup::<T>(vest_acc.clone());
		let lock_lookup: <T::Lookup as StaticLookup>::Source = as_lookup::<T>(lock_acc.clone());

		// Set balance
		<pallet_balances::Pallet<T> as Currency<T::AccountId>>::make_free_balance_be(&vest_acc, AMOUNT.into());
		<pallet_balances::Pallet<T> as Currency<T::AccountId>>::make_free_balance_be(&lock_acc, AMOUNT.into());
		UnownedAccount::<T>::insert(&vest_acc, ());
		UnownedAccount::<T>::insert(&lock_acc, ());

		// Set vesting info by mocking the Pallet's GenesisBuild
		Vesting::<T>::insert(
			&vest_acc,
			VestingInfo::<BalanceOf<T>, T::BlockNumber> {
				locked: AMOUNT.into(),
				per_block: PER_BLOCK.into(),
				starting_block: T::BlockNumber::zero(),
			},
		);
		// Set locking info by mocking the Pallet's GenesisBuild
		BalanceLocks::<T>::insert(
			&lock_acc,
			LockedBalance::<T> {
				block: UNLOCK_BLOCK.into(),
				amount: AMOUNT.into(),
			},
		);

		assert_eq!(<pallet_balances::Pallet<T>>::total_balance(&vest_acc), AMOUNT.into());
		assert_eq!(<pallet_balances::Pallet<T>>::total_balance(&lock_acc), AMOUNT.into());
		acc.0.push((vest_acc, vest_lookup));
		acc.1.push((lock_acc, lock_lookup));
		acc
	});

	Ok(((transfer, transfer_lookup), v_accs, l_accs))
}

benchmarks! {
	where_clause { where T: core::fmt::Debug, Balance: Into<<T as pallet_balances::Config>::Balance>, Balance: Into<<<T as pallet_vesting::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance>}

	change_transfer_account {
		let transfer_account: T::AccountId = account("transfer_new", 0, SEED);
		let transfer_account_lookup: <T::Lookup as StaticLookup>::Source = as_lookup::<T>(transfer_account.clone());
	}: _(RawOrigin::Root, transfer_account_lookup)
	verify {
		assert_eq!(TransferAccount::<T>::get(), Some(transfer_account));
	}

	// Worst case: UnlockingAt has MaxClaims entries
	force_unlock {
		let n in 1 .. T::MaxClaims::get() - 1;

		let ((transfer, _), _, s) = genesis_setup::<T>(n).expect("Genesis setup failure");
		whitelist_account!(transfer);

		// Migrate balance locks 1 by 1 to fill UnlockingAt
		for (c, (_, source_lookup)) in s.into_iter().enumerate() {
			let target: T::AccountId = account("target", u32::try_from(c).unwrap(), SEED);
			let target_lookup: <T::Lookup as StaticLookup>::Source = as_lookup::<T>(target);
			KiltLaunch::<T>::migrate_genesis_account(RawOrigin::Signed(transfer.clone()).into(), source_lookup, target_lookup)?;
		}
		assert_eq!(UnlockingAt::<T>::get::<T::BlockNumber>(UNLOCK_BLOCK.into()).expect("UnlockingAt should not be empty").len(), n as usize);
	}: _(RawOrigin::Root, UNLOCK_BLOCK.into())
	verify {
		assert!(!UnlockingAt::<T>::contains_key::<T::BlockNumber>(UNLOCK_BLOCK.into()));
	}

	// Worst case: target already has locked balance pre-transfer, source still has locked balance left post-transfer
	locked_transfer {
		let ((transfer, _), _, s) = genesis_setup::<T>(3).expect("Genesis setup failure");
		whitelist_account!(transfer);
		let mut locked_lookups: Vec<<T::Lookup as StaticLookup>::Source> = s.into_iter().map(|(_, lookup)| lookup).collect();
		let locked_lookup = locked_lookups.split_off(2);

		// Set custom lock with amount `2 * AMOUNT` for source
		let source: T::AccountId = account("source", 0, SEED);
		let source_lookup: <T::Lookup as StaticLookup>::Source = as_lookup::<T>(source.clone());
		KiltLaunch::<T>::migrate_multiple_genesis_accounts(RawOrigin::Signed(transfer.clone()).into(), locked_lookups, source_lookup)?;
		assert_eq!(BalanceLocks::<T>::get(&source), Some(LockedBalance::<T> {
			block: UNLOCK_BLOCK.into(),
			amount: (2 * AMOUNT).into(),
		}), "Source BalanceLock not set");
		assert_eq!(UnlockingAt::<T>::get::<T::BlockNumber>(UNLOCK_BLOCK.into()).expect("UnlockingAt should not be empty"), vec![source.clone()]);

		// Set custom lock with amount `AMOUNT` for target
		let target: T::AccountId = account("target", 0, SEED);
		let target_lookup: <T::Lookup as StaticLookup>::Source = as_lookup::<T>(target.clone());
		KiltLaunch::<T>::migrate_multiple_genesis_accounts(RawOrigin::Signed(transfer).into(), locked_lookup, target_lookup.clone())?;
		assert_eq!(BalanceLocks::<T>::get(&target), Some(LockedBalance::<T> {
			block: UNLOCK_BLOCK.into(),
			amount: AMOUNT.into(),
		}), "Target BalanceLock not set");
		assert_eq!(UnlockingAt::<T>::get::<T::BlockNumber>(UNLOCK_BLOCK.into()).expect("UnlockingAt should not be empty"), vec![source.clone(), target.clone()]);

		// Transfer AMOUNT from source to target
	}: _(RawOrigin::Signed(source.clone()), target_lookup, AMOUNT.into())
	verify {
		assert_eq!(UnlockingAt::<T>::get::<T::BlockNumber>(UNLOCK_BLOCK.into()).expect("UnlockingAt should not be empty"), vec![source.clone(), target.clone()]);
		assert_eq!(BalanceLocks::<T>::get(&source), Some(LockedBalance::<T> {
			block: UNLOCK_BLOCK.into(),
			amount: AMOUNT.into(),
		}), "Source BalanceLock not updated");
		assert_eq!(BalanceLocks::<T>::get(&target), Some(LockedBalance::<T> {
			block: UNLOCK_BLOCK.into(),
			amount: (2 * AMOUNT).into(),
		}), "Target BalanceLock not updated");
	}

	migrate_genesis_account_vesting {
		let target: T::AccountId = account("target", 0, SEED);
		let target_lookup: <T::Lookup as StaticLookup>::Source = as_lookup::<T>(target.clone());

		let ((transfer, transfer_lookup), s, _) = genesis_setup::<T>(1).expect("Genesis setup failure");
		whitelist_account!(transfer);
		let (source, source_lookup) = s.get(0).expect("Locking source should not be empty").clone();
	}: migrate_genesis_account(RawOrigin::Signed(transfer), source_lookup.clone(), target_lookup)
	verify {
		assert!(UnownedAccount::<T>::get(&source).is_none());
		assert!(!Vesting::<T>::contains_key(source), "Vesting schedule not removed");
		assert_eq!(Vesting::<T>::get(&target), Some(VestingInfo::<BalanceOf<T>, T::BlockNumber> {
			locked: AMOUNT.into(),
			per_block: PER_BLOCK.into(),
			starting_block: T::BlockNumber::zero(),
		}), "Vesting schedule not migrated");
		assert_eq!(Locks::<T>::get(&target).len(), 1, "Lock not set");
	}

	migrate_genesis_account_locking {
		let target: T::AccountId = account("target", 0, SEED);
		let target_lookup: <T::Lookup as StaticLookup>::Source = as_lookup::<T>(target.clone());

		let ((transfer, transfer_lookup), _, s) = genesis_setup::<T>(1).expect("Genesis setup failure");
		whitelist_account!(transfer);
		let (source, source_lookup) = s.get(0).expect("Locking source should not be empty").clone();
	}: migrate_genesis_account(RawOrigin::Signed(transfer), source_lookup.clone(), target_lookup)
	verify {
		assert!(UnownedAccount::<T>::get(&source).is_none());
		assert!(!BalanceLocks::<T>::contains_key(source), "BalanceLock not removed");
		assert_eq!(BalanceLocks::<T>::get(&target), Some(LockedBalance::<T> {
			block: UNLOCK_BLOCK.into(),
			amount: AMOUNT.into(),
		}), "BalanceLock not migrated");
		assert_eq!(Locks::<T>::get(&target).len(), 1, "Lock not set");
	}

	migrate_multiple_genesis_accounts_vesting {
		let n in 1 .. T::MaxClaims::get() - 1;

		let target: T::AccountId = account("target", 0, SEED);
		let target_lookup: <T::Lookup as StaticLookup>::Source = as_lookup::<T>(target.clone());

		let ((transfer, transfer_lookup), s, _) = genesis_setup::<T>(n).expect("Genesis setup failure");
		let source_lookups: Vec<<T::Lookup as StaticLookup>::Source> = s.into_iter().map(|(_, lookup)| lookup).collect();
	}: migrate_multiple_genesis_accounts(RawOrigin::Signed(transfer), source_lookups.clone(), target_lookup)
	verify {
		assert_eq!(Vesting::<T>::get(&target), Some(VestingInfo::<BalanceOf<T>, T::BlockNumber> {
			locked: (n as u128 * AMOUNT).into(),
			per_block: (n * PER_BLOCK).into(),
			starting_block: T::BlockNumber::zero(),
		}), "Vesting schedule not migrated");
		assert_eq!(Locks::<T>::get(&target).len(), 1, "Lock not set");
	}

	migrate_multiple_genesis_accounts_locking {
		let n in 1 .. T::MaxClaims::get() - 1;

		let target: T::AccountId = account("target", 0, SEED);
		let target_lookup: <T::Lookup as StaticLookup>::Source = as_lookup::<T>(target.clone());

		let ((transfer, transfer_lookup), _, s) = genesis_setup::<T>(n).expect("Genesis setup failure");
		whitelist_account!(transfer);
		let source_lookups: Vec<<T::Lookup as StaticLookup>::Source> = s.into_iter().map(|(_, lookup)| lookup).collect();
	}: migrate_multiple_genesis_accounts(RawOrigin::Signed(transfer), source_lookups.clone(), target_lookup)
	verify {
		assert_eq!(BalanceLocks::<T>::get(&target), Some(LockedBalance::<T> {
			block: UNLOCK_BLOCK.into(),
			amount: (n as u128 * AMOUNT).into(),
		}), "BalanceLock not migrated");
		assert_eq!(Locks::<T>::get(&target).len(), 1, "Lock not set");
	}
}

impl_benchmark_test_suite!(
	KiltLaunch,
	crate::mock::ExtBuilder::default().build(),
	crate::mock::Test,
);
