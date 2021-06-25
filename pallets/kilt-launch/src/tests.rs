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

use crate::{
	mock::*, BalanceLocks, Error, LockedBalance, TransferAccount, UnlockingAt, UnownedAccount, KILT_LAUNCH_ID,
	VESTING_ID,
};

use frame_support::{
	assert_noop, assert_ok,
	traits::{Currency, LockableCurrency, OnInitialize, WithdrawReasons},
};
use kilt_primitives::{AccountId, BlockNumber};
use pallet_balances::{BalanceLock, Locks, Reasons};
#[allow(unused_imports)]
use pallet_vesting::{Call::vest, Vesting as VestingStorage, VestingInfo};
use sp_runtime::traits::Zero;

#[test]
fn check_build_genesis_config() {
	ExtBuilder::default()
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
				amount: 1111 - <Test as crate::Config>::UsableBalance::get(),
			};
			let pseudo_2_lock = LockedBalance::<Test> {
				block: 1337,
				amount: 2222 - <Test as crate::Config>::UsableBalance::get(),
			};
			assert_eq!(BalanceLocks::<Test>::get(&PSEUDO_1), Some(pseudo_1_lock));
			assert_eq!(BalanceLocks::<Test>::get(&PSEUDO_2), Some(pseudo_2_lock));
			assert!(BalanceLocks::<Test>::get(&PSEUDO_3).is_none());
			assert!(UnlockingAt::<Test>::get(100).is_none());
			assert!(UnlockingAt::<Test>::get(1337).is_none());

			// Ensure there are no locks on pseudo accounts
			assert!(Locks::<Test>::get(&PSEUDO_1).len().is_zero());
			assert!(Locks::<Test>::get(&PSEUDO_2).len().is_zero());
			assert!(Locks::<Test>::get(&PSEUDO_3).len().is_zero());

			// Ensure all pseudo accounts are unowned accounts
			assert!(UnownedAccount::<Test>::get(&PSEUDO_1).is_some());
			assert!(UnownedAccount::<Test>::get(&PSEUDO_2).is_some());
			assert!(UnownedAccount::<Test>::get(&PSEUDO_3).is_some());
		});

	// check only vesting
	ExtBuilder::default().pseudos_vest_all().build().execute_with(|| {
		assert!(UnownedAccount::<Test>::get(&PSEUDO_1).is_some());
		assert!(UnownedAccount::<Test>::get(&PSEUDO_2).is_some());
		assert!(UnownedAccount::<Test>::get(&PSEUDO_3).is_some());
	});

	// check only locks
	ExtBuilder::default().pseudos_lock_all().build().execute_with(|| {
		assert!(UnownedAccount::<Test>::get(&PSEUDO_1).is_some());
		assert!(UnownedAccount::<Test>::get(&PSEUDO_2).is_some());
		assert!(UnownedAccount::<Test>::get(&PSEUDO_3).is_some());
	});

	// check lengths of 0
	ExtBuilder::default()
		.vest(vec![(PSEUDO_1, 0, 100)])
		.lock_balance(vec![(PSEUDO_2, 0, 100)])
		.build()
		.execute_with(|| {
			assert!(UnownedAccount::<Test>::get(&PSEUDO_1).is_some());
			assert!(UnownedAccount::<Test>::get(&PSEUDO_2).is_some());
		});
}

#[test]
fn check_migrate_single_account_locked() {
	ExtBuilder::default().pseudos_lock_all().build().execute_with(|| {
		assert_noop!(
			KiltLaunch::migrate_genesis_account(Origin::signed(TRANSFER_ACCOUNT), PSEUDO_1, PSEUDO_1),
			Error::<Test>::SameDestination
		);
		assert_noop!(
			KiltLaunch::migrate_genesis_account(Origin::signed(TRANSFER_ACCOUNT), USER, PSEUDO_1),
			Error::<Test>::NotUnownedAccount
		);

		let user_locked_info = LockedBalance {
			block: 100,
			amount: 10_000 - <Test as crate::Config>::UsableBalance::get(),
		};
		// Migration of balance locks
		ensure_single_migration_works(&PSEUDO_1, &USER, None, Some((user_locked_info, 0)));

		// Reach balance lock limit
		System::set_block_number(100);
		<KiltLaunch as OnInitialize<BlockNumber>>::on_initialize(System::block_number());
		assert!(UnlockingAt::<Test>::get(100).is_none());
		assert!(Locks::<Test>::get(&USER).len().is_zero());

		// Should be able to transfer all tokens but ExistentialDeposit
		assert_ok!(Balances::transfer(
			Origin::signed(USER),
			PSEUDO_2,
			10_000 - ExistentialDeposit::get()
		));
	});
}

#[test]
fn check_migrate_single_locked_account_after_unlock_block() {
	ExtBuilder::default().pseudos_lock_all().build().execute_with(|| {
		// Reach balance lock limit
		System::set_block_number(101);

		let user_locked_info = LockedBalance {
			block: 100,
			amount: 10_000 - <Test as crate::Config>::UsableBalance::get(),
		};
		// Migration of balance locks
		ensure_single_migration_works(&PSEUDO_1, &USER, None, Some((user_locked_info, 0)));

		assert!(UnlockingAt::<Test>::get(100).is_none());
		assert!(Locks::<Test>::get(&USER).len().is_zero());

		// Should be able to transfer all tokens but ExistentialDeposit
		assert_ok!(Balances::transfer(
			Origin::signed(USER),
			PSEUDO_2,
			10_000 - ExistentialDeposit::get()
		));
	});
}

#[test]
fn check_migrate_single_account_locked_twice() {
	ExtBuilder::default().pseudos_lock_all().build().execute_with(|| {
		let mut user_locked_info = LockedBalance {
			block: 100,
			amount: 10_000 - <Test as crate::Config>::UsableBalance::get(),
		};
		// Migrate pseudo1 lock
		ensure_single_migration_works(&PSEUDO_1, &USER, None, Some((user_locked_info, 0)));

		user_locked_info = LockedBalance {
			block: 100,
			amount: 10_000 + 300_000 - 2 * <Test as crate::Config>::UsableBalance::get(),
		};
		// Migrate pseudo2 lock
		ensure_single_migration_works(
			&PSEUDO_3,
			&USER,
			None,
			Some((
				user_locked_info,
				// Since we migrated twice, we need to account for the extra UsableBalance when asserting
				<Test as crate::Config>::UsableBalance::get(),
			)),
		);

		// Reach balance lock limit
		System::set_block_number(100);
		<KiltLaunch as OnInitialize<BlockNumber>>::on_initialize(System::block_number());
		assert!(UnlockingAt::<Test>::get(100).is_none());
		assert!(Locks::<Test>::get(&USER).len().is_zero());

		// Should be able to transfer all tokens but ExistentialDeposit
		assert_ok!(Balances::transfer(
			Origin::signed(USER),
			PSEUDO_2,
			310_000 - ExistentialDeposit::get()
		));
	});
}

#[test]
fn check_migrate_accounts_locked() {
	ExtBuilder::default().pseudos_lock_all().build().execute_with(|| {
		assert_noop!(
			KiltLaunch::migrate_multiple_genesis_accounts(
				Origin::signed(USER),
				vec![PSEUDO_1, PSEUDO_2, PSEUDO_3],
				USER
			),
			Error::<Test>::Unauthorized
		);

		// Migrate two accounts with same end block
		let locked_info = LockedBalance {
			block: 100,
			amount: 10_000 + 300_000 - 2 * <Test as crate::Config>::UsableBalance::get(),
		};
		assert_ok!(KiltLaunch::migrate_multiple_genesis_accounts(
			Origin::signed(TRANSFER_ACCOUNT),
			vec![PSEUDO_1, PSEUDO_3],
			USER
		));

		// Check unlocking info migration
		assert_eq!(UnlockingAt::<Test>::get(100), Some(vec![USER]));
		assert_eq!(BalanceLocks::<Test>::get(&USER), Some(locked_info.clone()));

		// Check correct setting of lock
		let balance_locks = Locks::<Test>::get(&USER);
		assert_eq!(balance_locks.len(), 1);
		for BalanceLock { id, amount, reasons } in balance_locks {
			match id {
				crate::KILT_LAUNCH_ID => {
					assert_eq!(amount, locked_info.amount);
					assert_eq!(reasons, Reasons::All);
				}
				_ => panic!("Unexpected balance lock id {:?}", id),
			};
		}

		// Check balance migration
		assert_balance(
			USER,
			locked_info.amount + 2 * <Test as crate::Config>::UsableBalance::get(),
			2 * <Test as crate::Config>::UsableBalance::get(),
			2 * <Test as crate::Config>::UsableBalance::get(),
			false,
		);

		// TODO: Add positive check for staking once it has been added

		// Reach balance lock limit
		System::set_block_number(100);
		<KiltLaunch as OnInitialize<BlockNumber>>::on_initialize(System::block_number());
		assert!(UnlockingAt::<Test>::get(100).is_none());
		assert!(Locks::<Test>::get(&USER).len().is_zero());
		assert_balance(
			USER,
			locked_info.amount + 2 * <Test as crate::Config>::UsableBalance::get(),
			locked_info.amount + 2 * <Test as crate::Config>::UsableBalance::get(),
			locked_info.amount + 2 * <Test as crate::Config>::UsableBalance::get(),
			true,
		);
	});
}

#[test]
fn check_locked_transfer() {
	ExtBuilder::default()
		.pseudos_lock_all()
		.build()
		.execute_with(|| {
			let locked_info = LockedBalance {
				block: 100,
				amount: 10_000 - <Test as crate::Config>::UsableBalance::get(),
			};
			// Migration of balance locks
			ensure_single_migration_works(&PSEUDO_1, &USER, None, Some((locked_info.clone(), 0)));
			assert_eq!(
				Locks::<Test>::get(&USER),
				vec![BalanceLock {
					id: KILT_LAUNCH_ID,
					amount: locked_info.amount,
					reasons: Reasons::All,
				}]
			);

			// Cannot transfer from source to source
			assert_noop!(
				KiltLaunch::locked_transfer(Origin::signed(USER), USER, 1),
				Error::<Test>::SameDestination
			);

			// Cannot transfer without a KILT balance lock
			assert_noop!(
				KiltLaunch::locked_transfer(Origin::signed(PSEUDO_4), USER, 1),
				Error::<Test>::BalanceLockNotFound
			);

			// Add 1 free balance to enable to pay for tx fees
			<<Test as pallet_vesting::Config>::Currency as Currency<<Test as frame_system::Config>::AccountId>>::make_free_balance_be(&USER, locked_info.amount + 1 + <Test as crate::Config>::UsableBalance::get());
			// Cannot transfer more locked than which is locked
			assert_noop!(
				KiltLaunch::locked_transfer(Origin::signed(USER), PSEUDO_1, locked_info.amount + 1 + <Test as crate::Config>::UsableBalance::get()),
				Error::<Test>::InsufficientLockedBalance
			);

			// Locked_Transfer everything but 3000
			assert_ok!(KiltLaunch::locked_transfer(
				Origin::signed(USER),
				PSEUDO_1,
				locked_info.amount - 3000
			));
			assert_eq!(
				Locks::<Test>::get(&USER),
				vec![BalanceLock {
					id: KILT_LAUNCH_ID,
					amount: 3000,
					reasons: Reasons::All,
				}]
			);
			assert_eq!(
				Locks::<Test>::get(&PSEUDO_1),
				vec![BalanceLock {
					id: KILT_LAUNCH_ID,
					amount: locked_info.amount - 3000,
					reasons: Reasons::All,
				}]
			);
			assert_eq!(UnlockingAt::<Test>::get(100), Some(vec![USER, PSEUDO_1]));
			assert_balance(PSEUDO_1, locked_info.amount - 3000, 0, 0, false);

			// Locked_Transfer rest
			assert_ok!(KiltLaunch::locked_transfer(Origin::signed(USER), PSEUDO_1, 3000));
			assert_eq!(Locks::<Test>::get(&USER), vec![]);
			assert_eq!(
				Locks::<Test>::get(&PSEUDO_1),
				vec![BalanceLock {
					id: KILT_LAUNCH_ID,
					amount: locked_info.amount,
					reasons: Reasons::All,
				}]
			);
			assert!(BalanceLocks::<Test>::get(&USER).is_none());
			assert_eq!(BalanceLocks::<Test>::get(&PSEUDO_1), Some(locked_info.clone()));
			assert_eq!(UnlockingAt::<Test>::get(100), Some(vec![PSEUDO_1]));
			assert_balance(PSEUDO_1, locked_info.amount, 0, 0, false);

			// Reach balance lock limit
			System::set_block_number(100);
			<KiltLaunch as OnInitialize<BlockNumber>>::on_initialize(System::block_number());
			assert!(UnlockingAt::<Test>::get(100).is_none());
			assert!(Locks::<Test>::get(&PSEUDO_1).len().is_zero());
			assert_balance(PSEUDO_1, locked_info.amount, locked_info.amount, locked_info.amount, true);
		});
}

#[test]
fn check_migrate_single_account_vested() {
	ExtBuilder::default().pseudos_vest_all().build().execute_with(|| {
		assert_noop!(
			KiltLaunch::migrate_genesis_account(Origin::signed(TRANSFER_ACCOUNT), PSEUDO_1, PSEUDO_1),
			Error::<Test>::SameDestination
		);
		assert_noop!(
			KiltLaunch::migrate_genesis_account(Origin::signed(TRANSFER_ACCOUNT), USER, PSEUDO_1),
			Error::<Test>::NotUnownedAccount
		);

		let user_vesting_schedule = VestingInfo {
			locked: 10_000,
			per_block: 1000, // Vesting over 10 blocks
			starting_block: 0,
		};

		// Migration of vesting info and balance locks
		ensure_single_migration_works(&PSEUDO_1, &USER, Some(user_vesting_schedule), None);

		// Reach vesting limit
		System::set_block_number(10);

		assert_ok!(Vesting::vest(Origin::signed(USER)));
		assert!(Vesting::vesting(&USER).is_none());
		assert!(Locks::<Test>::get(&USER).len().is_zero());
		// Should be able to transfer the remaining tokens
		assert_ok!(Balances::transfer(
			Origin::signed(USER),
			PSEUDO_1,
			user_vesting_schedule.locked - user_vesting_schedule.per_block
		));
	});
}

#[test]
fn check_migrate_single_account_twice_vested() {
	ExtBuilder::default().pseudos_vest_all().build().execute_with(|| {
		// Migration of vesting info from pseudo_1 to user_1
		let mut user_vesting_schedule = VestingInfo {
			locked: 10_000,
			per_block: 1000, // Vesting over 10 blocks
			starting_block: 0,
		};
		ensure_single_migration_works(&PSEUDO_1, &USER, Some(user_vesting_schedule), None);

		// Migration of vesting info from pseudo_2 with different vesting period to
		// user_1
		user_vesting_schedule = VestingInfo {
			locked: user_vesting_schedule.locked + 10_000,
			per_block: user_vesting_schedule.per_block + 500, // Vesting over 10 blocks
			starting_block: 0,
		};
		ensure_single_migration_works(&PSEUDO_2, &USER, Some(user_vesting_schedule), None);

		// Reach first vesting limit
		System::set_block_number(10);
		assert_ok!(Vesting::vest(Origin::signed(USER)));
		assert_eq!(Vesting::vesting(&USER), Some(user_vesting_schedule));
		assert_eq!(Locks::<Test>::get(&USER).len(), 1);
		assert_balance(
			USER,
			user_vesting_schedule.locked,
			user_vesting_schedule.locked,
			user_vesting_schedule.locked - 500 * 10,
			false,
		);

		// Reach second vesting limit
		System::set_block_number(20);
		assert_ok!(Vesting::vest(Origin::signed(USER)));
		assert!(Vesting::vesting(&USER).is_none());
		assert!(Locks::<Test>::get(&USER).len().is_zero());
		// Should be able to transfer the remaining tokens
		assert_balance(
			USER,
			user_vesting_schedule.locked,
			user_vesting_schedule.locked,
			user_vesting_schedule.locked,
			true,
		);
	});
}

#[test]
fn check_migrate_accounts_vested() {
	ExtBuilder::default().pseudos_vest_all().build().execute_with(|| {
		assert_noop!(
			KiltLaunch::migrate_multiple_genesis_accounts(
				Origin::signed(USER),
				vec![PSEUDO_1, PSEUDO_2, PSEUDO_3],
				USER
			),
			Error::<Test>::Unauthorized
		);
		assert_noop!(
			KiltLaunch::migrate_multiple_genesis_accounts(
				Origin::signed(TRANSFER_ACCOUNT),
				vec![PSEUDO_1, USER],
				PSEUDO_2
			),
			Error::<Test>::NotUnownedAccount
		);

		assert_ok!(KiltLaunch::migrate_multiple_genesis_accounts(
			Origin::signed(TRANSFER_ACCOUNT),
			vec![PSEUDO_1, PSEUDO_2, PSEUDO_3],
			USER
		));

		let vesting_info = VestingInfo {
			locked: 10_000 + 10_000 + 300_000,
			per_block: 10_000 / 10 + 10_000 / 20 + 300_000 / 30,
			starting_block: 0,
		};

		// Check vesting info migration
		assert_eq!(Vesting::vesting(&USER), Some(vesting_info));

		// Check correct setting of lock
		let balance_locks = Locks::<Test>::get(&USER);
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
		assert_balance(
			USER,
			vesting_info.locked,
			vesting_info.locked,
			vesting_info.per_block,
			false,
		);

		// TODO: Add positive check for staking once it has been added

		// Reach vesting limits
		for block in &[9, 10, 15, 20, 27] {
			System::set_block_number(*block);
			assert_ok!(Vesting::vest(Origin::signed(USER)));
			assert_eq!(
				Locks::<Test>::get(USER),
				vec![BalanceLock {
					id: VESTING_ID,
					amount: vesting_info.locked - vesting_info.per_block * (*block as u128),
					reasons: Reasons::Misc
				}]
			);
			assert_eq!(Vesting::vesting(&USER), Some(vesting_info));
			assert_balance(
				USER,
				vesting_info.locked,
				vesting_info.locked,
				vesting_info.per_block * (*block as u128),
				false,
			);
		}
		System::set_block_number(30);
		assert_ok!(Vesting::vest(Origin::signed(USER)));
		assert!(Vesting::vesting(&USER).is_none());
		assert!(Locks::<Test>::get(&USER).len().is_zero());
	});
}

#[test]
fn check_negative_migrate_accounts_vested() {
	ExtBuilder::default().pseudos_vest_all().build().execute_with(|| {
		// Migrate from source to source
		assert_noop!(
			KiltLaunch::migrate_multiple_genesis_accounts(Origin::signed(TRANSFER_ACCOUNT), vec![PSEUDO_1, USER], USER),
			Error::<Test>::SameDestination
		);

		// Migrate too many accounts
		assert_noop!(
			KiltLaunch::migrate_multiple_genesis_accounts(
				Origin::signed(TRANSFER_ACCOUNT),
				vec![PSEUDO_1, PSEUDO_2, PSEUDO_3, PSEUDO_4],
				USER
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
				USER
			),
			Error::<Test>::NotUnownedAccount
		);
		UnownedAccount::<Test>::insert(PSEUDO_4, ());
		assert_noop!(
			KiltLaunch::migrate_multiple_genesis_accounts(
				Origin::signed(TRANSFER_ACCOUNT),
				vec![PSEUDO_1, PSEUDO_4],
				USER
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
			KiltLaunch::migrate_multiple_genesis_accounts(Origin::signed(TRANSFER_ACCOUNT), vec![PSEUDO_4], USER),
			Error::<Test>::UnexpectedLocks
		);
	});
}

#[test]
fn check_negative_migrate_accounts_locked() {
	ExtBuilder::default().pseudos_lock_all().build().execute_with(|| {
		// Migrate from source to source
		assert_noop!(
			KiltLaunch::migrate_multiple_genesis_accounts(Origin::signed(TRANSFER_ACCOUNT), vec![PSEUDO_1, USER], USER),
			Error::<Test>::SameDestination
		);

		// Migrate two accounts with different ending blocks
		assert_noop!(
			KiltLaunch::migrate_multiple_genesis_accounts(
				Origin::signed(TRANSFER_ACCOUNT),
				vec![PSEUDO_1, PSEUDO_2],
				USER
			),
			Error::<Test>::ConflictingLockingBlocks
		);

		// Add a lock to pseudo2 which should not be there
		Balances::set_lock(KILT_LAUNCH_ID, &PSEUDO_2, 1, WithdrawReasons::all());
		assert_noop!(
			KiltLaunch::migrate_multiple_genesis_accounts(Origin::signed(TRANSFER_ACCOUNT), vec![PSEUDO_2], USER),
			Error::<Test>::UnexpectedLocks
		);
	});
}

#[test]
fn check_force_unlock() {
	ExtBuilder::default().pseudos_lock_all().build().execute_with(|| {
		let user_locked_info = LockedBalance {
			block: 100,
			amount: 10_000 - <Test as crate::Config>::UsableBalance::get(),
		};
		ensure_single_migration_works(&PSEUDO_1, &USER, None, Some((user_locked_info, 0)));

		assert_ok!(KiltLaunch::force_unlock(Origin::root(), 100));
		assert!(BalanceLocks::<Test>::get(&USER).is_none());
		assert!(Locks::<Test>::get(&USER).len().is_zero());
		assert_eq!(Balances::usable_balance(&USER), 10_000);
	});
}

#[test]
fn check_change_transfer_account() {
	ExtBuilder::default().build().execute_with(|| {
		assert_eq!(TransferAccount::<Test>::get(), Some(TRANSFER_ACCOUNT));
		assert_ok!(KiltLaunch::change_transfer_account(Origin::root(), PSEUDO_1));
		assert_eq!(TransferAccount::<Test>::get(), Some(PSEUDO_1));
	});
}

#[test]
#[should_panic = "Currencies must be init'd before locking"]
fn check_genesis_panic_locking_balance() {
	ExtBuilder::default().build_panic(vec![], vec![(PSEUDO_1, 100, 10_000)], vec![]);
}
#[test]
#[should_panic = "Currencies must be init'd before vesting"]
fn check_genesis_panic_vesting_balance() {
	ExtBuilder::default().build_panic(vec![], vec![], vec![(PSEUDO_1, 100, 10_000)]);
}

#[test]
#[should_panic = "Locked balance must not exceed total balance for address \"5C4hrfjw9DjXZTzV3MwzrrAr9P1MJhSrvWGWqi1eSuyUpnhM\""]
fn check_genesis_panic_locking_amount() {
	ExtBuilder::default().build_panic(vec![(PSEUDO_1, 10_000)], vec![(PSEUDO_1, 100, 10_001)], vec![]);
}
#[test]
#[should_panic = "Vested balance must not exceed total balance for address \"5C4hrfjw9DjXZTzV3MwzrrAr9P1MJhSrvWGWqi1eSuyUpnhM\""]
fn check_genesis_panic_vesting_amount() {
	ExtBuilder::default().build_panic(vec![(PSEUDO_1, 10_000)], vec![], vec![(PSEUDO_1, 100, 10_001)]);
}

#[test]
#[should_panic = "Account with address \"5C4hrfjw9DjXZTzV3MwzrrAr9P1MJhSrvWGWqi1eSuyUpnhM\" must not occur twice in locking"]
fn check_genesis_panic_locking_same_acc() {
	ExtBuilder::default().build_panic(
		vec![(PSEUDO_1, 10_000)],
		vec![(PSEUDO_1, 100, 10_000), (PSEUDO_1, 1337, 10_000)],
		vec![],
	);
}
#[test]
#[should_panic = "Account with address \"5C4hrfjw9DjXZTzV3MwzrrAr9P1MJhSrvWGWqi1eSuyUpnhM\" must not occur twice in vesting"]
fn check_genesis_vesting_locking_same_acc() {
	ExtBuilder::default().build_panic(
		vec![(PSEUDO_1, 10_000)],
		vec![],
		vec![(PSEUDO_1, 100, 10_000), (PSEUDO_1, 1337, 10_000)],
	);
}
