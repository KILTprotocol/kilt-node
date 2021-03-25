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
use frame_support::{assert_noop, assert_ok};
use pallet_balances::Locks;
use pallet_vesting::VestingInfo;

// TODO: Maybe add a setter function to the pallet for testing

#[test]
fn check_build_genesis_config() {
	ExtBuilder::default()
		.one_hundred_for_alice_n_bob()
		.vest_alice_bob()
		.lock_alice_bob()
		.build()
		.execute_with(|| {
			let user1_free_balance = Balances::free_balance(&ALICE);
			let user2_free_balance = Balances::free_balance(&BOB);
			assert_eq!(user1_free_balance, 10_000); // Account 1 has free balance
			assert_eq!(user2_free_balance, 10_000); // Account 2 has free balance
			let user1_vesting_schedule = VestingInfo {
				locked: 1000,
				per_block: 100, // Vesting over 10 blocks
				starting_block: 0,
			};
			let user2_vesting_schedule = VestingInfo {
				locked: 1000,
				per_block: 50, // Vesting over 20 blocks
				starting_block: 0,
			};
			assert_eq!(Vesting::vesting(&ALICE), Some(user1_vesting_schedule)); // Account 1 has a vesting schedule
			assert_eq!(Vesting::vesting(&BOB), Some(user2_vesting_schedule));

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
fn check_user_claim() {
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
		});
}

// Test cases
// Successfully build genesis
// All assertions when building genesis
// all cases of (balance, vested, locks)
// claiming_process:
// storage killed?
// locks set?
// free balance available at first block (vesting)
// force_unlock
