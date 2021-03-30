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

use crate::{mock::*, BalanceLocks, LockedBalance, TransferAccount, UnlockingAt};
use frame_support::{assert_noop, assert_ok, traits::OnInitialize};
use kilt_primitives::BlockNumber;
use pallet_balances::{BalanceLock, Locks, Reasons};
#[allow(unused_imports)]
use pallet_vesting::{Call::vest, VestingInfo};

#[test]
fn check_build_genesis_config() {
	ExtBuilder::default()
		.init_balance_for_pseudos()
		.pseudos_vest_all()
		.pseudos_lock_something()
		.build()
		.execute_with(|| {
			let alice_free_balance = Balances::free_balance(&PSEUDO_1);
			let bob_free_balance = Balances::free_balance(&PSEUDO_2);

			// Check balance
			assert_eq!(alice_free_balance, 10_000);
			assert_eq!(bob_free_balance, 10_000);
			let alice_vesting_schedule = VestingInfo {
				locked: 10_000,
				// Vesting over 10 blocks
				per_block: 1000,
				starting_block: 0,
			};
			let bob_vesting_schedule = VestingInfo {
				locked: 10_000,
				// Vesting over 20 blocks
				per_block: 500,
				starting_block: 0,
			};
			assert_eq!(Vesting::vesting(&PSEUDO_1), Some(alice_vesting_schedule));
			assert_eq!(Vesting::vesting(&PSEUDO_2), Some(bob_vesting_schedule));

			// Check balance locks
			let alice_balance_lock = LockedBalance::<Test> {
				block: 100,
				amount: 1111,
			};
			let bob_balance_lock = LockedBalance::<Test> {
				block: 1337,
				amount: 2222,
			};
			assert_eq!(BalanceLocks::<Test>::get(&PSEUDO_1), Some(alice_balance_lock));
			assert_eq!(BalanceLocks::<Test>::get(&PSEUDO_2), Some(bob_balance_lock));
			assert_eq!(UnlockingAt::<Test>::get(100), None);
			assert_eq!(UnlockingAt::<Test>::get(1337), None);

			// Ensure there are no locks on pseudo accounts
			assert_eq!(Locks::<Test>::get(&PSEUDO_1).len(), 0);
			assert_eq!(Locks::<Test>::get(&PSEUDO_2).len(), 0);
		});
}

#[test]
fn check_user_claim_locked() {
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
			ensure_migration_works(&PSEUDO_1, &USER_1, None, Some(user1_locked_info));

			// Balance migration
			let user1_free_balance = Balances::free_balance(&USER_1);
			assert_eq!(user1_free_balance, 10_000);

			// Should not be able to transfer any tokens because all are locked
			assert_noop!(
				Balances::transfer(Origin::signed(USER_1), PSEUDO_2, ExistentialDeposit::get() + 1),
				pallet_balances::Error::<Test, ()>::LiquidityRestrictions
			);

			// TODO: Add positive check for staking once it has been added

			// Reach balance lock limit
			System::set_block_number(100);
			<KiltLaunch as OnInitialize<BlockNumber>>::on_initialize(System::block_number());
			assert_eq!(UnlockingAt::<Test>::get(100), None);
			assert_eq!(Locks::<Test>::get(&USER_1).len(), 0);

			// Should be able to transfer all tokens but ExistentialDeposit
			assert_ok!(Balances::transfer(
				Origin::signed(USER_1),
				PSEUDO_2,
				user1_free_balance - ExistentialDeposit::get()
			));
		});
}
#[test]
fn check_user_claim_vested() {
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
			ensure_migration_works(&PSEUDO_1, &USER_1, Some(user1_vesting_schedule), None);

			// Balance migration
			let user1_free_balance = Balances::free_balance(&USER_1);
			assert_eq!(user1_free_balance, 10_000);

			// Should be able to transfer what's unlocked already
			// Note: In the test we migrate in the 1st block, thus we assert that the first
			// `per_block` amount is already available for transfers
			assert_ok!(Balances::transfer(
				Origin::signed(USER_1),
				PSEUDO_2,
				user1_vesting_schedule.per_block
			));
			// Should not be able to transfer more than which is unlocked in first block
			assert_noop!(
				Balances::transfer(Origin::signed(USER_1), PSEUDO_2, 1001),
				pallet_balances::Error::<Test, ()>::LiquidityRestrictions
			);

			// TODO: Add positive check for staking once it has been added

			// Reach vesting limit
			System::set_block_number(1000);
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
fn check_force_unlock() {
	ExtBuilder::default()
		.init_balance_for_pseudos()
		.pseudos_lock_something()
		.build()
		.execute_with(|| {
			assert_ok!(KiltLaunch::accept_user_account_claim(
				Origin::signed(TRANSFER_ACCOUNT),
				PSEUDO_1,
				USER_1
			));
			assert_eq!(BalanceLocks::<Test>::get(&USER_1), None);

			let user1_balance_locks = Locks::<Test>::get(&USER_1);
			assert_eq!(user1_balance_locks.len(), 1);
			for BalanceLock { id, amount, reasons } in user1_balance_locks {
				match id {
					crate::KILT_LAUNCH_ID => {
						assert_eq!(amount, 1111);
						assert_eq!(reasons, Reasons::Misc);
					}
					_ => panic!("Unexpected balance lock id {:?}", id),
				};
			}

			assert_ok!(KiltLaunch::force_unlock(Origin::root(), 100));
			assert_eq!(BalanceLocks::<Test>::get(&USER_1), None);
			assert_eq!(Locks::<Test>::get(&USER_1).len(), 0);
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
