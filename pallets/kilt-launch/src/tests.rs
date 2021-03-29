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

use crate::{mock::*, BalanceLocks, Error, LockedBalance, UnlockingAt};
use frame_support::{assert_noop, assert_ok, traits::OnInitialize};
use kilt_primitives::BlockNumber;
use pallet_balances::{BalanceLock, Locks, Reasons};
use pallet_vesting::{Call::vest, VestingInfo};

// TODO: Maybe add a setter function to the pallet for testing

#[test]
fn check_build_genesis_config() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.vest_alice_bob()
		.lock_alice_bob()
		.build()
		.execute_with(|| {
			let alice_free_balance = Balances::free_balance(&ALICE);
			let bob_free_balance = Balances::free_balance(&BOB);
			assert_eq!(alice_free_balance, 10_000); // Account 1 has free balance
			assert_eq!(bob_free_balance, 10_000); // Account 2 has free balance
			let alice_vesting_schedule = VestingInfo {
				locked: 1000,
				per_block: 100, // Vesting over 10 blocks
				starting_block: 0,
			};
			let bob_vesting_schedule = VestingInfo {
				locked: 1000,
				per_block: 50, // Vesting over 20 blocks
				starting_block: 0,
			};
			assert_eq!(Vesting::vesting(&ALICE), Some(alice_vesting_schedule)); // Account 1 has a vesting schedule
			assert_eq!(Vesting::vesting(&BOB), Some(bob_vesting_schedule));

			// TEST CUSTOM LOCK
			let alice_balance_lock = LockedBalance::<Test> {
				block: 100,
				amount: 1111,
			};
			let bob_balance_lock = LockedBalance::<Test> {
				block: 1337,
				amount: 2222,
			};
			assert_eq!(BalanceLocks::<Test>::get(&ALICE), Some(alice_balance_lock));
			assert_eq!(BalanceLocks::<Test>::get(&BOB), Some(bob_balance_lock));
			assert_eq!(UnlockingAt::<Test>::get(100), None);
			assert_eq!(UnlockingAt::<Test>::get(1337), None);

			// TEST LOCKS
			assert_eq!(Locks::<Test>::get(&ALICE).len(), 0);
			assert_eq!(Locks::<Test>::get(&BOB).len(), 0);
		});
}

#[test]
fn check_user_claim_vested_locked() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.vest_alice_bob()
		.lock_alice_bob()
		.build()
		.execute_with(|| {
			assert_ok!(KiltLaunch::accept_user_account_claim(
				Origin::signed(TRANSFER_ACCOUNT),
				ALICE,
				USER_1
			));
			System::set_block_number(2);

			// check for desired death of allocation account
			assert_eq!(Balances::free_balance(ALICE), 0);
			assert_eq!(Vesting::vesting(&ALICE), None);
			assert_eq!(BalanceLocks::<Test>::get(&ALICE), None);
			assert!(!frame_system::Account::<Test>::contains_key(&ALICE));

			// Balance migration
			let user1_free_balance = Balances::free_balance(&USER_1);
			assert_eq!(user1_free_balance, 10_000); // Account 1 has free balance

			// Vesting migration
			let user1_vesting_schedule = VestingInfo {
				locked: 1000,
				per_block: 100, // Vesting over 10 blocks
				starting_block: 0,
			};
			assert_eq!(Vesting::vesting(&USER_1), Some(user1_vesting_schedule)); // Account 1 has a vesting schedule

			// Balance lock migration
			assert_eq!(BalanceLocks::<Test>::get(&USER_1), None);
			assert_eq!(UnlockingAt::<Test>::get(100), Some(vec![USER_1]));

			// Check locks
			let user1_balance_locks = Locks::<Test>::get(&USER_1);
			assert_eq!(user1_balance_locks.len(), 2);
			for BalanceLock { id, amount, reasons } in user1_balance_locks {
				match id {
					crate::VESTING_ID => {
						assert_eq!(amount, 1000 - 100);
						assert_eq!(reasons, Reasons::Misc);
					}
					crate::KILT_LAUNCH_ID => {
						assert_eq!(amount, 1111);
						assert_eq!(reasons, Reasons::Misc);
					}
					_ => panic!("Unexpected balance lock id {:?}", id),
				};
			}

			// Reach balance lock limit
			System::set_block_number(100);
			<KiltLaunch as OnInitialize<BlockNumber>>::on_initialize(System::block_number());
			assert_eq!(UnlockingAt::<Test>::get(100), None);
			assert_eq!(Locks::<Test>::get(&USER_1).len(), 1);

			// Reach vesting limit
			System::set_block_number(1000);
			// TODO: Uncomment once `vest` is public which is the case on master
			// but not on rococo-v1 as of 2021-03-26
			// assert_ok!(Vesting::vest(Origin::signed(USER_1)));
			// assert_eq!(Vesting::vesting(&USER_1), None);
			// assert_eq!(Locks::<Test>::get(&USER_1).len(), 0);
		});
}

// fn check_user_claim_vested() {
// 	ExtBuilder::default()
// 		.one_hundred_for_alice_n_bob()
// 		.vest_alice_bob()
// 		.build()
// 		.execute_with(|| {
// 			assert_ok!(KiltLaunch::accept_user_account_claim(
// 				Origin::signed(TRANSFER_ACCOUNT),
// 				ALICE,
// 				USER_1
// 			));
// 			System::set_block_number(2);

// 			// check for desired death of allocation account
// 			assert_eq!(Balances::free_balance(ALICE), 0);
// 			assert_eq!(Vesting::vesting(&ALICE), None);
// 			assert_eq!(BalanceLocks::<Test>::get(&ALICE), None);
// 			assert!(!frame_system::Account::<Test>::contains_key(&ALICE));

// 			// Balance migration
// 			let user1_free_balance = Balances::free_balance(&USER_1);
// 			assert_eq!(user1_free_balance, 10_000); // Account 1 has free balance

// 			// Vesting migration
// 			let user1_vesting_schedule = VestingInfo {
// 				locked: 1000,
// 				per_block: 100, // Vesting over 10 blocks
// 				starting_block: 0,
// 			};
// 			assert_eq!(Vesting::vesting(&USER_1), Some(user1_vesting_schedule)); //
// Account 1 has a vesting schedule

// 			// Balance lock migration
// 			assert_eq!(BalanceLocks::<Test>::get(&USER_1), None);
// 			assert_eq!(UnlockingAt::<Test>::get(100), Some(vec![USER_1]));

// 			// Check locks
// 			let user1_balance_locks = Locks::<Test>::get(&USER_1);
// 			assert_eq!(user1_balance_locks.len(), 2);
// 			for BalanceLock { id, amount, reasons } in user1_balance_locks {
// 				match id {
// 					crate::VESTING_ID => {
// 						assert_eq!(amount, 1000 - 100);
// 						assert_eq!(reasons, Reasons::Misc);
// 					}
// 					crate::KILT_LAUNCH_ID => {
// 						assert_eq!(amount, 1111);
// 						assert_eq!(reasons, Reasons::Misc);
// 					}
// 					_ => panic!("Unexpected balance lock id {:?}", id),
// 				};
// 			}
// 		});
// }

// Test cases
// Successfully build genesis
// All assertions when building genesis
// all cases of (balance, vested, locks)
// claiming_process:
// storage killed?
// locks set?
// free balance available at first block (vesting)
// force_unlock
