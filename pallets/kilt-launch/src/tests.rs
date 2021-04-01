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

use crate::{mock::*, BalanceLocks, Error, LockedBalance, TransferAccount, UnlockingAt, KILT_LAUNCH_ID, VESTING_ID};
use frame_support::{
	assert_noop, assert_ok,
	traits::{LockableCurrency, OnInitialize, WithdrawReasons},
	StorageMap,
};
use kilt_primitives::{AccountId, BlockNumber};
use pallet_balances::{BalanceLock, Locks, Reasons};
#[allow(unused_imports)]
use pallet_vesting::{Call::vest, Vesting as VestingStorage, VestingInfo};

#[test]
fn check_build_genesis_config() {
	ExtBuilder::default()
		.init_balance_for_pseudos()
		.pseudos_vest_all()
		.pseudos_lock_something()
		.build()
		.execute_with(|| {
			// Check balances
			assert_eq!(Balances::free_balance(&PSEUDO_1), 10_000);
			assert_eq!(Balances::free_balance(&PSEUDO_2), 10_000);
			assert_eq!(Balances::free_balance(&PSEUDO_3), 300_000);
			// Locked balance should be usable for fees
			assert_eq!(Balances::usable_balance_for_fees(&PSEUDO_1), 10_000);
			assert_eq!(Balances::usable_balance_for_fees(&PSEUDO_2), 10_000);
			assert_eq!(Balances::usable_balance_for_fees(&PSEUDO_3), 300_000);
			// There should be nothing reserved
			assert_eq!(Balances::reserved_balance(&PSEUDO_1), 0);
			assert_eq!(Balances::reserved_balance(&PSEUDO_2), 0);
			assert_eq!(Balances::reserved_balance(&PSEUDO_3), 0);

			// Check vesting
			let pseudo_1_vesting = VestingInfo {
				locked: 10_000,
				// Vesting over 10 blocks
				per_block: 1000,
				starting_block: 0,
			};
			let pseudo_2_vesting = VestingInfo {
				locked: 10_000,
				// Vesting over 20 blocks
				per_block: 500,
				starting_block: 0,
			};
			let pseudo_3_vesting = VestingInfo {
				locked: 300_000,
				// Vesting over 20 blocks
				per_block: 10_000,
				starting_block: 0,
			};
			assert_eq!(Vesting::vesting(&PSEUDO_1), Some(pseudo_1_vesting));
			assert_eq!(Vesting::vesting(&PSEUDO_2), Some(pseudo_2_vesting));
			assert_eq!(Vesting::vesting(&PSEUDO_3), Some(pseudo_3_vesting));

			// Check balance locks
			let pseudo_1_lock = LockedBalance::<Test> {
				block: 100,
				amount: 1111,
			};
			let pseudo_2_lock = LockedBalance::<Test> {
				block: 1337,
				amount: 2222,
			};
			assert_eq!(BalanceLocks::<Test>::get(&PSEUDO_1), Some(pseudo_1_lock));
			assert_eq!(BalanceLocks::<Test>::get(&PSEUDO_2), Some(pseudo_2_lock));
			assert_eq!(BalanceLocks::<Test>::get(&PSEUDO_3), None);
			assert_eq!(UnlockingAt::<Test>::get(100), None);
			assert_eq!(UnlockingAt::<Test>::get(1337), None);

			// Ensure there are no locks on pseudo accounts
			assert_eq!(Locks::<Test>::get(&PSEUDO_1).len(), 0);
			assert_eq!(Locks::<Test>::get(&PSEUDO_2).len(), 0);
			assert_eq!(Locks::<Test>::get(&PSEUDO_3).len(), 0);
		});
}

#[test]
fn check_migrate_single_account_locked() {
	ExtBuilder::default()
		.init_balance_for_pseudos()
		.pseudos_lock_all()
		.build()
		.execute_with(|| {
			let user1_locked_info = LockedBalance {
				block: 100,
				amount: 10_000,
			};
			// Migration of balance locks
			ensure_single_migration_works(&PSEUDO_1, &USER_1, None, Some(user1_locked_info));

			// Reach balance lock limit
			System::set_block_number(100);
			<KiltLaunch as OnInitialize<BlockNumber>>::on_initialize(System::block_number());
			assert_eq!(UnlockingAt::<Test>::get(100), None);
			assert_eq!(Locks::<Test>::get(&USER_1).len(), 0);

			// Should be able to transfer all tokens but ExistentialDeposit
			assert_ok!(Balances::transfer(
				Origin::signed(USER_1),
				PSEUDO_2,
				10_000 - ExistentialDeposit::get()
			));
		});
}

// TODO: Add test for check_migrate_single_account_twice_locked

// TODO: Add test for check_migrate_accounts_locked

#[test]
fn check_migrate_single_account_vested() {
	ExtBuilder::default()
		.init_balance_for_pseudos()
		.pseudos_vest_all()
		.build()
		.execute_with(|| {
			let user1_vesting_schedule = VestingInfo {
				locked: 10_000,
				per_block: 1000, // Vesting over 10 blocks
				starting_block: 0,
			};

			// Migration of vesting info and balance locks
			ensure_single_migration_works(&PSEUDO_1, &USER_1, Some(user1_vesting_schedule), None);

			// Reach vesting limit
			System::set_block_number(10);
			// TODO: Uncomment once `vest` is public which is the case on master
			// but not on rococo-v1 as of 2021-03-26
			// assert_ok!(Vesting::vest(Origin::signed(USER_1)));
			// assert_eq!(Vesting::vesting(&USER_1), None);
			// assert_eq!(Locks::<Test>::get(&USER_1).len(), 0);
			// // Should be able to transfer the remaining tokens
			// assert_ok!(Balances::transfer(
			// 	Origin::signed(USER_1),
			// 	BOB,
			// 	user1_vesting_schedule.locked - user1_vesting_schedule.per_block
			// ));
		});
}

#[test]
fn check_migrate_single_account_twice_vested() {
	ExtBuilder::default()
		.init_balance_for_pseudos()
		.pseudos_vest_all()
		.build()
		.execute_with(|| {
			// Migration of vesting info from pseudo_1 to user_1
			let mut user1_vesting_schedule = VestingInfo {
				locked: 10_000,
				per_block: 1000, // Vesting over 10 blocks
				starting_block: 0,
			};
			ensure_single_migration_works(&PSEUDO_1, &USER_1, Some(user1_vesting_schedule), None);

			// Migration of vesting info from pseudo_2 with different vesting period to
			// user_1
			user1_vesting_schedule = VestingInfo {
				locked: user1_vesting_schedule.locked + 10_000,
				per_block: user1_vesting_schedule.per_block + 500, // Vesting over 10 blocks
				starting_block: 0,
			};
			ensure_single_migration_works(&PSEUDO_2, &USER_1, Some(user1_vesting_schedule), None);

			// Reach first vesting limit
			System::set_block_number(10);
			// TODO: Uncomment once `vest` is public which is the case on master
			// but not on rococo-v1 as of 2021-03-26
			// assert_ok!(Vesting::vest(Origin::signed(USER_1)));
			// assert_eq!(Vesting::vesting(&USER_1), Some(5000));
			// assert_eq!(Locks::<Test>::get(&USER_1).len(), 1);
			// // Should be able to transfer the remaining tokens
			// assert_ok!(Balances::transfer(
			// 	Origin::signed(USER_1),
			// 	BOB,
			// 	15_000 - user1_vesting_schedule.per_block
			// ));

			// Reach second vesting limit
			System::set_block_number(20);
			// TODO: Uncomment once `vest` is public which is the case on master
			// but not on rococo-v1 as of 2021-03-26
			// assert_ok!(Vesting::vest(Origin::signed(USER_1)));
			// assert_eq!(Vesting::vesting(&USER_1), None);
			// assert_eq!(Locks::<Test>::get(&USER_1).len(), 0);
			// // Should be able to transfer the remaining tokens
			// assert_ok!(Balances::transfer(
			// 	Origin::signed(USER_1),
			// 	BOB,
			// 	5000
			// ));
		});
}

#[test]
fn check_migrate_accounts_vested() {
	ExtBuilder::default()
		.init_balance_for_pseudos()
		.pseudos_vest_all()
		.build()
		.execute_with(|| {
			assert_noop!(
				KiltLaunch::migrate_multiple_genesis_accounts(
					Origin::signed(USER_1),
					vec![PSEUDO_1, PSEUDO_2, PSEUDO_3],
					USER_1
				),
				Error::<Test>::Unauthorized
			);
			assert_ok!(KiltLaunch::migrate_multiple_genesis_accounts(
				Origin::signed(TRANSFER_ACCOUNT),
				vec![PSEUDO_1, PSEUDO_2, PSEUDO_3],
				USER_1
			));

			let vesting_info = VestingInfo {
				locked: 10_000 + 10_000 + 300_000,
				per_block: 10_000 / 10 + 10_000 / 20 + 300_000 / 30,
				starting_block: 0,
			};

			// Check vesting info migration
			assert_eq!(Vesting::vesting(&USER_1), Some(vesting_info));

			// Check correct setting of lock
			let balance_locks = Locks::<Test>::get(&USER_1);
			assert_eq!(balance_locks.len(), 1);
			for BalanceLock { id, amount, reasons } in balance_locks {
				match id {
					crate::VESTING_ID => {
						assert_eq!(amount, vesting_info.locked - vesting_info.per_block);
						assert_eq!(reasons, Reasons::Misc);
					}
					_ => panic!("Unexpected balance lock id {:?}", id),
				};
			}

			// Check balance migration
			assert_eq!(Balances::free_balance(&USER_1), vesting_info.locked);
			// locked balance should be usable for fees
			assert_eq!(Balances::usable_balance_for_fees(&USER_1), vesting_info.locked);
			// locked balance should not be usable for anything but fees and other locks
			assert_eq!(Balances::usable_balance(&USER_1), vesting_info.per_block);
			// there should be nothing reserved
			assert_eq!(Balances::reserved_balance(&USER_1), 0);

			// TODO: Add positive check for staking once it has been added

			// Reach vesting limits
			for block in vec![10, 20, 29] {
				System::set_block_number(block);
				// TODO: Uncomment once `vest` is public which is the case on
				// master but not on rococo-v1 as of 2021-03-26
				// assert_ok!(Vesting::vest(Origin::signed(USER_1)));
				// assert_eq!(Vesting::vesting(&USER_1),
				// Some(vesting_info.locked - vesting_info.per_block * block));
				// // Should be able to transfer the remaining tokens
				// assert_eq!(Balances::usable_balance(&USER_1),
				// vesting_info.per_block * block);
			}
			System::set_block_number(30);
			// assert_ok!(Vesting::vest(Origin::signed(USER_1)));
			// assert_eq!(Vesting::vesting(&USER_1), None);
			// assert_eq!(Locks::<Test>::get(&USER_1).len(), 0);
		});
}

#[test]
fn check_negative_migrate_accounts_vested() {
	ExtBuilder::default()
		.init_balance_for_pseudos()
		.pseudos_vest_all()
		.build()
		.execute_with(|| {
			// Migrate too many accounts
			assert_noop!(
				KiltLaunch::migrate_multiple_genesis_accounts(
					Origin::signed(TRANSFER_ACCOUNT),
					vec![PSEUDO_1, PSEUDO_2, PSEUDO_3, PSEUDO_4],
					USER_1
				),
				Error::<Test>::ExceedsMaxClaims
			);

			// Set up vesting with conflicting start block
			let pseudo_4_vesting = VestingInfo {
				locked: 10_000,
				per_block: 1,
				starting_block: 1,
			};
			VestingStorage::<Test>::insert(PSEUDO_4, pseudo_4_vesting);
			assert_noop!(
				KiltLaunch::migrate_multiple_genesis_accounts(
					Origin::signed(TRANSFER_ACCOUNT),
					vec![PSEUDO_1, PSEUDO_4],
					USER_1
				),
				Error::<Test>::ConflictingVestingStarts
			);

			// Set a vesting lock which should not be there from the Genesis builder
			<<Test as pallet_vesting::Config>::Currency as LockableCurrency<AccountId>>::set_lock(
				VESTING_ID,
				&PSEUDO_4,
				1,
				WithdrawReasons::TRANSFER | WithdrawReasons::RESERVE,
			);
			assert_noop!(
				KiltLaunch::migrate_multiple_genesis_accounts(Origin::signed(TRANSFER_ACCOUNT), vec![PSEUDO_4], USER_1),
				Error::<Test>::UnexpectedLocks
			);
		});
}

#[test]
fn check_negative_migrate_accounts_locked() {
	ExtBuilder::default()
		.init_balance_for_pseudos()
		.pseudos_lock_all()
		.build()
		.execute_with(|| {
			// Migrate two accounts with different ending blocks
			assert_noop!(
				KiltLaunch::migrate_multiple_genesis_accounts(
					Origin::signed(TRANSFER_ACCOUNT),
					vec![PSEUDO_1, PSEUDO_2],
					USER_1
				),
				Error::<Test>::ConflictingLockingBlocks
			);

			// Add a lock to pseudo2 which should not be there
			Balances::set_lock(
				KILT_LAUNCH_ID,
				&PSEUDO_2,
				1,
				WithdrawReasons::TRANSFER | WithdrawReasons::RESERVE,
			);
			assert_noop!(
				KiltLaunch::migrate_multiple_genesis_accounts(Origin::signed(TRANSFER_ACCOUNT), vec![PSEUDO_2], USER_1),
				Error::<Test>::UnexpectedLocks
			);
		});
}

#[test]
fn check_force_unlock() {
	ExtBuilder::default()
		.init_balance_for_pseudos()
		.pseudos_lock_all()
		.build()
		.execute_with(|| {
			let user1_locked_info = LockedBalance {
				block: 100,
				amount: 10_000,
			};
			ensure_single_migration_works(&PSEUDO_1, &USER_1, None, Some(user1_locked_info));

			assert_ok!(KiltLaunch::force_unlock(Origin::root(), 100));
			assert_eq!(BalanceLocks::<Test>::get(&USER_1), None);
			assert_eq!(Locks::<Test>::get(&USER_1).len(), 0);
			assert_eq!(Balances::usable_balance(&USER_1), 10_000);
		});
}

#[test]
fn check_change_transfer_account() {
	ExtBuilder::default()
		.init_balance_for_pseudos()
		.build()
		.execute_with(|| {
			assert_eq!(TransferAccount::<Test>::get(), Some(TRANSFER_ACCOUNT));
			assert_ok!(KiltLaunch::change_transfer_account(Origin::root(), PSEUDO_1));
			assert_eq!(TransferAccount::<Test>::get(), Some(PSEUDO_1));
		});
}
