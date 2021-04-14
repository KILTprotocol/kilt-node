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
use crate::{BalanceLocks, BalanceOf, LockedBalance, Pallet as KiltLaunch, KILT_LAUNCH_ID};
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, vec, whitelisted_caller};
use frame_support::inherent::Vec;
use frame_system::{Pallet as System, RawOrigin};
use pallet_vesting::{Vesting, VestingInfo};

const SEED: u32 = 0;
const AMOUNT: u32 = 10000;
const PER_BLOCK: u32 = 100;
const UNLOCK_BLOCK: u32 = 1337;
// type Currency<T> = pallet_balances::Pallet<T> as Currency<T::AccountId>;
type Lookup<T> = <<T as frame_system::Config>::Lookup as StaticLookup>::Source;

/// Account to lookup type of system trait.
fn as_lookup<T: Config>(account: T::AccountId) -> Lookup<T> {
	T::Lookup::unlookup(account)
}

fn add_locks<T: Config>(who: &T::AccountId, lock_id: [u8; 8]) {
	// for id in 0..n {
	// 	let lock_id = [id; 8];
	let locked = 100u32;
	let reasons = WithdrawReasons::TRANSFER | WithdrawReasons::RESERVE;
	T::Currency::set_lock(lock_id, who, locked.into(), reasons);
	// }
}

/// Mock the Pallet's GenesisBuild and return pairs consisting of AccountId and
/// LookupSource for the transfer account, `n` vesting addresses and `n` locking
/// addresses.
fn genesis_setup<T: Config>(
	n: u32,
) -> Result<
	(
		(T::AccountId, <T::Lookup as StaticLookup>::Source),
		Vec<(T::AccountId, <T::Lookup as StaticLookup>::Source)>,
		Vec<(T::AccountId, <T::Lookup as StaticLookup>::Source)>,
	),
	&'static str,
> {
	System::<T>::set_block_number(0u32.into());

	// Setup transfer account
	let transfer: T::AccountId = account("transfer", 0, SEED);
	let transfer_lookup: <T::Lookup as StaticLookup>::Source = as_lookup::<T>(transfer.clone());
	KiltLaunch::<T>::change_transfer_account(RawOrigin::Root.into(), transfer_lookup.clone())?;

	// Create `n` genesis accounts each for vesting and locking
	let (v_accs, l_accs) = (0..n).into_iter().fold((vec![], vec![]), |mut acc, i| {
		let vest_acc: T::AccountId = account("vesting_{:?}", i, SEED);
		let lock_acc: T::AccountId = account("locking", i, SEED);
		let vest_lookup: <T::Lookup as StaticLookup>::Source = as_lookup::<T>(vest_acc.clone());
		let lock_lookup: <T::Lookup as StaticLookup>::Source = as_lookup::<T>(lock_acc.clone());

		// Set balance
		<pallet_balances::Pallet<T> as Currency<T::AccountId>>::make_free_balance_be(&vest_acc, AMOUNT.into());
		<pallet_balances::Pallet<T> as Currency<T::AccountId>>::make_free_balance_be(&lock_acc, AMOUNT.into());

		// Set vesting/locking info
		add_vesting_schedule::<T>(&vest_acc);
		add_lock_info::<T>(&lock_acc);

		acc.0.push((vest_acc, vest_lookup));
		acc.1.push((lock_acc, lock_lookup));
		acc
	});

	Ok(((transfer, transfer_lookup), v_accs, l_accs))
}

/// Mock the setup of a vesting schedule from the Pallet's GenesisBuild.
fn add_vesting_schedule<T: Config>(who: &T::AccountId) -> Result<(), &'static str> {
	Vesting::<T>::insert(
		who,
		VestingInfo::<BalanceOf<T>, T::BlockNumber> {
			locked: AMOUNT.into(),
			per_block: PER_BLOCK.into(),
			starting_block: T::BlockNumber::zero(),
		},
	);
	Ok(())
}

/// Mock the setup of the KILT custom lock from the Pallet's GenesisBuild.
fn add_lock_info<T: Config>(who: &T::AccountId) -> Result<(), &'static str> {
	BalanceLocks::<T>::insert(
		who,
		LockedBalance::<T> {
			block: UNLOCK_BLOCK.into(),
			amount: AMOUNT.into(),
		},
	);
	Ok(())
}

benchmarks! {
	where_clause { where T: core::fmt::Debug }

	change_transfer_account {
		let transfer_account: T::AccountId = account("transfer_new", 0, SEED);
		let transfer_account_lookup: <T::Lookup as StaticLookup>::Source = as_lookup::<T>(transfer_account.clone());
	}: _(RawOrigin::Root, transfer_account_lookup)
	verify {
		assert_eq!(TransferAccount::<T>::get(), Some(transfer_account));
	}

	migrate_genesis_account_vesting {
		let target: T::AccountId = account("target", 0, SEED);
		let target_lookup: <T::Lookup as StaticLookup>::Source = as_lookup::<T>(target.clone());

		let ((transfer, transfer_lookup), s, _) = genesis_setup::<T>(1).expect("Genesis setup failure");
		let (source, source_lookup) = s.get(0).expect("Locking source should not be empty").clone();

	}: migrate_genesis_account(RawOrigin::Signed(transfer), source_lookup.clone(), target_lookup)
	verify {
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
		let (source, source_lookup) = s.get(0).expect("Locking source should not be empty").clone();

	}: migrate_genesis_account(RawOrigin::Signed(transfer), source_lookup.clone(), target_lookup)
	verify {
		assert!(!BalanceLocks::<T>::contains_key(source), "BalanceLock not removed");
		assert_eq!(BalanceLocks::<T>::get(&target), Some(LockedBalance::<T> {
			block: UNLOCK_BLOCK.into(),
			amount: AMOUNT.into(),
		}), "BalanceLock not migrated");
		assert_eq!(Locks::<T>::get(&target).len(), 1, "Lock not set");
	}

	migrate_multiple_genesis_accounts_vesting {
		let n in 0 .. T::MaxClaims::get() as u32;

		let target: T::AccountId = account("target", 0, SEED);
		let target_lookup: <T::Lookup as StaticLookup>::Source = as_lookup::<T>(target.clone());

		let ((transfer, transfer_lookup), s, _) = genesis_setup::<T>(n).expect("Genesis setup failure");
		let source_lookups: Vec<<T::Lookup as StaticLookup>::Source> = s.into_iter().map(|(_, lookup)| lookup).collect();

	}: migrate_multiple_genesis_accounts(RawOrigin::Signed(transfer), source_lookups.clone(), target_lookup)
	verify {
		// assert!(!BalanceLocks::<T>::contains_key(source), "BalanceLock not removed");
		assert_eq!(BalanceLocks::<T>::get(&target), Some(LockedBalance::<T> {
			block: UNLOCK_BLOCK.into(),
			amount: ((n + 1) * AMOUNT).into(),
		}), "BalanceLock not migrated");
		assert_eq!(Locks::<T>::get(&target).len(), 1, "Lock not set");
	}

	migrate_multiple_genesis_accounts_locking {
		let n in 0 .. T::MaxClaims::get() as u32;

		let target: T::AccountId = account("target", 0, SEED);
		let target_lookup: <T::Lookup as StaticLookup>::Source = as_lookup::<T>(target.clone());

		let ((transfer, transfer_lookup), _, s) = genesis_setup::<T>(n).expect("Genesis setup failure");
		let source_lookups: Vec<<T::Lookup as StaticLookup>::Source> = s.into_iter().map(|(_, lookup)| lookup).collect();

	}: migrate_multiple_genesis_accounts(RawOrigin::Signed(transfer), source_lookups.clone(), target_lookup)
	verify {
		assert_eq!(Vesting::<T>::get(&target), Some(VestingInfo::<BalanceOf<T>, T::BlockNumber> {
			locked: ((n + 1) * AMOUNT).into(),
			per_block: ((n + 1) * PER_BLOCK).into(),
			starting_block: T::BlockNumber::zero(),
		}), "Vesting schedule not migrated");
		assert_eq!(Locks::<T>::get(&target).len(), 1, "Lock not set");
	}

}

impl_benchmark_test_suite!(Template, crate::mock::ExtBuilder::default().build(), crate::mock::Test,);
