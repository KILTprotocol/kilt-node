// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2023 BOTLabs GmbH

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

//! Unit testing

use frame_support::{assert_noop, assert_ok, storage::bounded_btree_map::BoundedBTreeMap};
use kilt_runtime_api_staking::StakingRates;
use pallet_authorship::EventHandler;
use pallet_balances::{Freezes, IdAmount};

use sp_runtime::{traits::Zero, Perquintill};

use crate::{
	mock::{roll_to, AccountId, Balance, BlockNumber, ExtBuilder, RuntimeOrigin, StakePallet, Test, DECIMALS},
	types::{BalanceOf, TotalStake},
	Config, Error, FreezeReason,
};

#[test]
fn should_update_total_stake() {
	ExtBuilder::default()
		.with_balances(vec![
			(1, 100),
			(2, 100),
			(3, 100),
			(4, 100),
			(5, 100),
			(6, 100),
			(7, 100),
			(8, 100),
			(9, 100),
			(10, 100),
			(11, 161_000_000 * DECIMALS),
		])
		.with_collators(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 10)])
		.with_delegators(vec![(7, 1, 10), (8, 2, 10), (9, 2, 10)])
		.set_blocks_per_round(5)
		.build_and_execute_with_sanity_tests(|| {
			let mut old_stake = StakePallet::total_collator_stake();
			assert_eq!(
				old_stake,
				TotalStake {
					collators: 40,
					delegators: 30,
				}
			);
			assert_ok!(StakePallet::candidate_stake_more(RuntimeOrigin::signed(1), 50));
			assert_eq!(
				StakePallet::total_collator_stake(),
				TotalStake {
					collators: old_stake.collators + 50,
					..old_stake
				}
			);

			old_stake = StakePallet::total_collator_stake();
			assert_ok!(StakePallet::candidate_stake_less(RuntimeOrigin::signed(1), 50));
			assert_eq!(
				StakePallet::total_collator_stake(),
				TotalStake {
					collators: old_stake.collators - 50,
					..old_stake
				}
			);

			old_stake = StakePallet::total_collator_stake();
			assert_ok!(StakePallet::delegator_stake_more(RuntimeOrigin::signed(7), 50));
			assert_noop!(
				StakePallet::delegator_stake_more(RuntimeOrigin::signed(7), 0),
				Error::<Test>::ValStakeZero
			);
			assert_noop!(
				StakePallet::delegator_stake_less(RuntimeOrigin::signed(7), 0),
				Error::<Test>::ValStakeZero
			);
			assert_eq!(
				StakePallet::total_collator_stake(),
				TotalStake {
					delegators: old_stake.delegators + 50,
					..old_stake
				}
			);

			old_stake = StakePallet::total_collator_stake();
			assert_ok!(StakePallet::delegator_stake_less(RuntimeOrigin::signed(7), 50));
			assert_eq!(
				StakePallet::total_collator_stake(),
				TotalStake {
					delegators: old_stake.delegators - 50,
					..old_stake
				}
			);

			old_stake = StakePallet::total_collator_stake();
			assert_ok!(StakePallet::join_delegators(RuntimeOrigin::signed(11), 1, 200));
			assert_eq!(
				StakePallet::total_collator_stake(),
				TotalStake {
					delegators: old_stake.delegators + 200,
					..old_stake
				}
			);

			old_stake = StakePallet::total_collator_stake();
			assert_eq!(StakePallet::delegator_state(11).unwrap().amount, 200);
			assert_ok!(StakePallet::leave_delegators(RuntimeOrigin::signed(11)));
			assert_eq!(
				StakePallet::total_collator_stake(),
				TotalStake {
					delegators: old_stake.delegators - 200,
					..old_stake
				}
			);

			let old_stake = StakePallet::total_collator_stake();
			assert_eq!(StakePallet::delegator_state(8).unwrap().amount, 10);
			assert_ok!(StakePallet::leave_delegators(RuntimeOrigin::signed(8)));
			assert_eq!(
				StakePallet::total_collator_stake(),
				TotalStake {
					delegators: old_stake.delegators - 10,
					..old_stake
				}
			);

			// should immediately affect total stake because collator can't be chosen in
			// active set from now on, thus delegated stake is reduced
			let old_stake = StakePallet::total_collator_stake();
			assert_eq!(StakePallet::candidate_pool(2).unwrap().total, 30);
			assert_eq!(StakePallet::candidate_pool(2).unwrap().stake, 20);
			assert_eq!(StakePallet::selected_candidates().into_inner(), vec![2, 1]);
			assert_eq!(
				StakePallet::candidate_pool(2).unwrap().stake,
				StakePallet::candidate_pool(3).unwrap().stake
			);
			assert_ok!(StakePallet::init_leave_candidates(RuntimeOrigin::signed(2)));
			let old_stake = TotalStake {
				delegators: old_stake.delegators - 10,
				// total active collator stake is unchanged because number of selected candidates is 2 and 2's
				// replacement has the same self stake as 2
				collators: old_stake.collators,
			};
			assert_eq!(StakePallet::selected_candidates().into_inner(), vec![1, 3]);
			assert_eq!(StakePallet::total_collator_stake(), old_stake);

			// shouldn't change total stake when 2 leaves
			roll_to(10, vec![]);
			assert_eq!(StakePallet::total_collator_stake(), old_stake);
			assert_ok!(StakePallet::execute_leave_candidates(RuntimeOrigin::signed(2), 2));
			assert_eq!(StakePallet::total_collator_stake(), old_stake);
		})
}

#[test]
fn unlock_unstaked() {
	// same_unstaked_as_restaked
	// block 1: stake & unstake for 100
	// block 2: stake & unstake for 100
	// should remove first entry in unstaking BoundedBTreeMap when staking in block
	// 2 should still have 100 locked until unlocking
	ExtBuilder::default()
		.with_balances(vec![(1, 10), (2, 100), (3, 100)])
		.with_collators(vec![(1, 10), (3, 10)])
		.with_delegators(vec![(2, 1, 100)])
		.build_and_execute_with_sanity_tests(|| {
			assert_ok!(StakePallet::leave_delegators(RuntimeOrigin::signed(2)));
			let mut unstaking: BoundedBTreeMap<BlockNumber, BalanceOf<Test>, <Test as Config>::MaxUnstakeRequests> =
				BoundedBTreeMap::new();
			assert_ok!(unstaking.try_insert(3, 100));
			let freeze = IdAmount {
				id: <Test as Config>::FreezeIdentifier::from(FreezeReason::Staking),
				amount: 100,
			};

			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Freezes::<Test>::get(2), vec![freeze.clone()]);
			// shouldn't be able to unlock anything
			assert_ok!(StakePallet::unlock_unstaked(RuntimeOrigin::signed(2), 2));
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Freezes::<Test>::get(2), vec![freeze.clone()]);

			// join delegators and revoke again --> consume unstaking at block 3
			roll_to(2, vec![]);
			assert_ok!(StakePallet::join_delegators(RuntimeOrigin::signed(2), 1, 100));
			assert_ok!(StakePallet::leave_delegators(RuntimeOrigin::signed(2)));
			unstaking.remove(&3);
			assert_ok!(unstaking.try_insert(4, 100));
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Freezes::<Test>::get(2), vec![freeze.clone()]);
			// shouldn't be able to unlock anything
			assert_ok!(StakePallet::unlock_unstaked(RuntimeOrigin::signed(2), 2));
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Freezes::<Test>::get(2), vec![freeze.clone()]);

			// should reduce unlocking but not unlock anything
			roll_to(3, vec![]);
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Freezes::<Test>::get(2), vec![freeze.clone()]);
			// shouldn't be able to unlock anything
			assert_ok!(StakePallet::unlock_unstaked(RuntimeOrigin::signed(2), 2));
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Freezes::<Test>::get(2), vec![freeze.clone()]);

			roll_to(4, vec![]);
			unstaking.remove(&4);
			assert_eq!(Freezes::<Test>::get(2), vec![freeze]);
			// shouldn't be able to unlock anything
			assert_ok!(StakePallet::unlock_unstaked(RuntimeOrigin::signed(2), 2));
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Freezes::<Test>::get(2), vec![]);
		});

	// less_unstaked_than_restaked
	// block 1: stake & unstake for 10
	// block 2: stake & unstake for 100
	// should remove first entry in unstaking BoundedBTreeMap when staking in block
	// 2 should still have 90 locked until unlocking in block 4
	ExtBuilder::default()
		.with_balances(vec![(1, 10), (2, 100), (10, 100)])
		.with_collators(vec![(1, 10), (10, 10)])
		.with_delegators(vec![(2, 1, 10)])
		.build_and_execute_with_sanity_tests(|| {
			assert_ok!(StakePallet::leave_delegators(RuntimeOrigin::signed(2)));
			let mut unstaking: BoundedBTreeMap<BlockNumber, BalanceOf<Test>, <Test as Config>::MaxUnstakeRequests> =
				BoundedBTreeMap::new();
			assert_ok!(unstaking.try_insert(3, 10));
			let mut lock = IdAmount {
				id: <Test as Config>::FreezeIdentifier::from(FreezeReason::Staking),
				amount: 10,
			};
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Freezes::<Test>::get(2), vec![lock.clone()]);
			// shouldn't be able to unlock anything
			assert_ok!(StakePallet::unlock_unstaked(RuntimeOrigin::signed(2), 2));
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Freezes::<Test>::get(2), vec![lock.clone()]);

			// join delegators and revoke again
			roll_to(2, vec![]);
			assert_ok!(StakePallet::join_delegators(RuntimeOrigin::signed(2), 1, 100));
			assert_ok!(StakePallet::leave_delegators(RuntimeOrigin::signed(2)));
			unstaking.remove(&3);
			assert_ok!(unstaking.try_insert(4, 100));
			lock.amount = 100;
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Freezes::<Test>::get(2), vec![lock.clone()]);
			// shouldn't be able to unlock anything
			assert_ok!(StakePallet::unlock_unstaked(RuntimeOrigin::signed(2), 2));
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Freezes::<Test>::get(2), vec![lock.clone()]);

			roll_to(3, vec![]);
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Freezes::<Test>::get(2), vec![lock.clone()]);
			// shouldn't be able to unlock anything
			assert_ok!(StakePallet::unlock_unstaked(RuntimeOrigin::signed(2), 2));
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Freezes::<Test>::get(2), vec![lock.clone()]);

			// unlock unstaked, remove lock, empty unlocking
			roll_to(4, vec![]);
			unstaking.remove(&4);
			assert_eq!(Freezes::<Test>::get(2), vec![lock]);
			assert_ok!(StakePallet::unlock_unstaked(RuntimeOrigin::signed(2), 2));
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Freezes::<Test>::get(2), vec![]);
		});

	// more_unstaked_than_restaked
	// block 1: stake & unstake for 100
	// block 2: stake & unstake for 10
	// should reduce first entry from amount 100 to 90 in unstaking BoundedBTreeMap
	// when staking in block 2
	// should have 100 locked until unlocking in block 3, then 10
	// should have 10 locked until further unlocking in block 4
	ExtBuilder::default()
		.with_balances(vec![(1, 10), (2, 100), (3, 199)])
		.with_collators(vec![(1, 10), (3, 10)])
		.with_delegators(vec![(2, 1, 100)])
		.build_and_execute_with_sanity_tests(|| {
			assert_ok!(StakePallet::leave_delegators(RuntimeOrigin::signed(2)));
			let mut unstaking: BoundedBTreeMap<BlockNumber, BalanceOf<Test>, <Test as Config>::MaxUnstakeRequests> =
				BoundedBTreeMap::new();
			assert_ok!(unstaking.try_insert(3, 100));
			let mut lock = IdAmount {
				id: <Test as Config>::FreezeIdentifier::from(FreezeReason::Staking),
				amount: 100,
			};
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Freezes::<Test>::get(2), vec![lock.clone()]);
			// shouldn't be able to unlock anything
			assert_ok!(StakePallet::unlock_unstaked(RuntimeOrigin::signed(2), 2));
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Freezes::<Test>::get(2), vec![lock.clone()]);

			// join delegators and revoke again
			roll_to(2, vec![]);
			assert_ok!(StakePallet::join_delegators(RuntimeOrigin::signed(2), 1, 10));
			assert_ok!(StakePallet::leave_delegators(RuntimeOrigin::signed(2)));
			assert_ok!(unstaking.try_insert(3, 90));
			assert_ok!(unstaking.try_insert(4, 10));
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Freezes::<Test>::get(2), vec![lock.clone()]);
			// shouldn't be able to unlock anything
			assert_ok!(StakePallet::unlock_unstaked(RuntimeOrigin::signed(2), 2));
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Freezes::<Test>::get(2), vec![lock.clone()]);

			// should reduce unlocking but not unlock anything
			roll_to(3, vec![]);
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Freezes::<Test>::get(2), vec![lock.clone()]);
			// should be able to unlock 90 of 100 from unstaking
			assert_ok!(StakePallet::unlock_unstaked(RuntimeOrigin::signed(2), 2));
			unstaking.remove(&3);
			lock.amount = 10;
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Freezes::<Test>::get(2), vec![lock.clone()]);

			roll_to(4, vec![]);
			assert_eq!(Freezes::<Test>::get(2), vec![lock]);
			// should be able to unlock 10 of remaining 10
			assert_ok!(StakePallet::unlock_unstaked(RuntimeOrigin::signed(2), 2));
			unstaking.remove(&4);
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Freezes::<Test>::get(2), vec![]);
		});

	// test_stake_less
	// block 1: stake & unstake for 100
	// block 2: stake & unstake for 10
	// should reduce first entry from amount 100 to 90 in unstaking BoundedBTreeMap
	// when staking in block 2
	// should have 100 locked until unlocking in block 3, then 10
	// should have 10 locked until further unlocking in block 4
	ExtBuilder::default()
		.with_balances(vec![(1, 200), (2, 200), (3, 100)])
		.with_collators(vec![(1, 200), (3, 10)])
		.with_delegators(vec![(2, 1, 200)])
		.build_and_execute_with_sanity_tests(|| {
			// should be able to decrease more often than MaxUnstakeRequests because it's
			// the same block and thus unstaking is increased at block 3 instead of having
			// multiple entries for the same block
			assert_ok!(StakePallet::candidate_stake_less(RuntimeOrigin::signed(1), 10));
			assert_ok!(StakePallet::candidate_stake_less(RuntimeOrigin::signed(1), 10));
			assert_ok!(StakePallet::candidate_stake_less(RuntimeOrigin::signed(1), 10));
			assert_ok!(StakePallet::candidate_stake_less(RuntimeOrigin::signed(1), 10));
			assert_ok!(StakePallet::candidate_stake_less(RuntimeOrigin::signed(1), 10));
			assert_ok!(StakePallet::candidate_stake_less(RuntimeOrigin::signed(1), 10),);
			assert_ok!(StakePallet::delegator_stake_less(RuntimeOrigin::signed(2), 10));
			assert_ok!(StakePallet::delegator_stake_less(RuntimeOrigin::signed(2), 10));
			assert_ok!(StakePallet::delegator_stake_less(RuntimeOrigin::signed(2), 10));
			assert_ok!(StakePallet::delegator_stake_less(RuntimeOrigin::signed(2), 10));
			assert_ok!(StakePallet::delegator_stake_less(RuntimeOrigin::signed(2), 10));
			assert_ok!(StakePallet::delegator_stake_less(RuntimeOrigin::signed(2), 10),);
			let mut unstaking: BoundedBTreeMap<BlockNumber, BalanceOf<Test>, <Test as Config>::MaxUnstakeRequests> =
				BoundedBTreeMap::new();
			assert_ok!(unstaking.try_insert(3, 60));
			let mut lock = IdAmount {
				id: <Test as Config>::FreezeIdentifier::from(FreezeReason::Staking),
				amount: 200,
			};
			assert_eq!(Freezes::<Test>::get(1), vec![lock.clone()]);
			assert_eq!(Freezes::<Test>::get(2), vec![lock.clone()]);
			assert_eq!(StakePallet::unstaking(1), unstaking);
			assert_eq!(StakePallet::unstaking(2), unstaking);
			// shouldn't be able to unlock anything
			assert_ok!(StakePallet::unlock_unstaked(RuntimeOrigin::signed(1), 1));
			assert_ok!(StakePallet::unlock_unstaked(RuntimeOrigin::signed(2), 2));
			assert_eq!(StakePallet::unstaking(1), unstaking);
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Freezes::<Test>::get(1), vec![lock.clone()]);
			assert_eq!(Freezes::<Test>::get(2), vec![lock.clone()]);

			roll_to(2, vec![]);
			assert_ok!(StakePallet::candidate_stake_less(RuntimeOrigin::signed(1), 10),);
			assert_ok!(StakePallet::delegator_stake_less(RuntimeOrigin::signed(2), 10),);
			assert_ok!(unstaking.try_insert(4, 10));
			assert_eq!(Freezes::<Test>::get(1), vec![lock.clone()]);
			assert_eq!(Freezes::<Test>::get(2), vec![lock.clone()]);
			assert_eq!(StakePallet::unstaking(1), unstaking);
			assert_eq!(StakePallet::unstaking(2), unstaking);
			// shouldn't be able to unlock anything
			assert_ok!(StakePallet::unlock_unstaked(RuntimeOrigin::signed(1), 1));
			assert_ok!(StakePallet::unlock_unstaked(RuntimeOrigin::signed(2), 2));
			assert_eq!(StakePallet::unstaking(1), unstaking);
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Freezes::<Test>::get(1), vec![lock.clone()]);
			assert_eq!(Freezes::<Test>::get(2), vec![lock.clone()]);

			roll_to(3, vec![]);
			assert_ok!(StakePallet::candidate_stake_less(RuntimeOrigin::signed(1), 10),);
			assert_ok!(StakePallet::delegator_stake_less(RuntimeOrigin::signed(2), 10),);
			assert_ok!(unstaking.try_insert(5, 10));
			assert_ok!(unstaking.try_insert(5, 10));
			assert_eq!(Freezes::<Test>::get(1), vec![lock.clone()]);
			assert_eq!(Freezes::<Test>::get(2), vec![lock.clone()]);
			assert_eq!(StakePallet::unstaking(1), unstaking);
			assert_eq!(StakePallet::unstaking(2), unstaking);
			// should unlock 60
			assert_ok!(StakePallet::unlock_unstaked(RuntimeOrigin::signed(1), 1));
			assert_ok!(StakePallet::unlock_unstaked(RuntimeOrigin::signed(2), 2));
			lock.amount = 140;
			unstaking.remove(&3);
			assert_eq!(StakePallet::unstaking(1), unstaking);
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Freezes::<Test>::get(1), vec![lock.clone()]);
			assert_eq!(Freezes::<Test>::get(2), vec![lock.clone()]);

			// reach MaxUnstakeRequests
			roll_to(4, vec![]);
			assert_ok!(StakePallet::candidate_stake_less(RuntimeOrigin::signed(1), 10));
			assert_ok!(StakePallet::delegator_stake_less(RuntimeOrigin::signed(2), 10));
			roll_to(5, vec![]);
			assert_ok!(StakePallet::candidate_stake_less(RuntimeOrigin::signed(1), 10));
			assert_ok!(StakePallet::delegator_stake_less(RuntimeOrigin::signed(2), 10));
			roll_to(6, vec![]);
			assert_ok!(StakePallet::candidate_stake_less(RuntimeOrigin::signed(1), 10));
			assert_ok!(StakePallet::delegator_stake_less(RuntimeOrigin::signed(2), 10));
			assert_ok!(unstaking.try_insert(6, 10));
			assert_ok!(unstaking.try_insert(7, 10));
			assert_ok!(unstaking.try_insert(8, 10));
			assert_eq!(StakePallet::unstaking(1), unstaking);
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Freezes::<Test>::get(1), vec![lock.clone()]);
			assert_eq!(Freezes::<Test>::get(2), vec![lock.clone()]);

			roll_to(7, vec![]);
			assert_noop!(
				StakePallet::candidate_stake_less(RuntimeOrigin::signed(1), 10),
				Error::<Test>::NoMoreUnstaking
			);
			assert_noop!(
				StakePallet::delegator_stake_less(RuntimeOrigin::signed(2), 10),
				Error::<Test>::NoMoreUnstaking
			);
			assert_ok!(StakePallet::unlock_unstaked(RuntimeOrigin::signed(1), 1));
			assert_ok!(StakePallet::unlock_unstaked(RuntimeOrigin::signed(2), 2));
			unstaking.remove(&4);
			unstaking.remove(&5);
			unstaking.remove(&6);
			unstaking.remove(&7);
			lock.amount = 100;
			assert_eq!(StakePallet::unstaking(1), unstaking);
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Freezes::<Test>::get(1), vec![lock.clone()]);
			assert_eq!(Freezes::<Test>::get(2), vec![lock.clone()]);
			assert_ok!(StakePallet::candidate_stake_less(RuntimeOrigin::signed(1), 40));
			assert_ok!(StakePallet::delegator_stake_less(RuntimeOrigin::signed(2), 40));
			assert_ok!(unstaking.try_insert(9, 40));
			assert_ok!(StakePallet::candidate_stake_more(RuntimeOrigin::signed(1), 30));
			assert_ok!(StakePallet::delegator_stake_more(RuntimeOrigin::signed(2), 30));
			unstaking.remove(&8);
			assert_ok!(unstaking.try_insert(9, 20));
			assert_eq!(StakePallet::unstaking(1), unstaking);
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Freezes::<Test>::get(1), vec![lock.clone()]);
			assert_eq!(Freezes::<Test>::get(2), vec![lock]);
		});
}

#[test]
fn rewards_candidate_stake_more() {
	ExtBuilder::default()
		.with_balances(vec![(1, 2 * DECIMALS), (2, DECIMALS), (3, DECIMALS), (4, 100)])
		.with_collators(vec![(1, DECIMALS), (4, 10)])
		.with_delegators(vec![(2, 1, DECIMALS), (3, 1, DECIMALS)])
		.build_and_execute_with_sanity_tests(|| {
			// note once to set counter to 1
			StakePallet::note_author(1);
			assert_eq!(StakePallet::blocks_authored(1), 2);
			assert!(StakePallet::blocks_authored(2).is_zero());
			assert!(StakePallet::blocks_authored(3).is_zero());
			(1..=3).for_each(|id| {
				assert!(StakePallet::blocks_rewarded(id).is_zero());
				assert!(StakePallet::rewards(id).is_zero());
			});

			// stake less to trigger reward incrementing for collator
			assert_ok!(StakePallet::candidate_stake_more(RuntimeOrigin::signed(1), DECIMALS));
			assert!(!StakePallet::rewards(1).is_zero());
			assert!(!StakePallet::blocks_rewarded(1).is_zero());
			// delegator reward storage should be untouched
			(2..=3).for_each(|id| {
				assert!(
					StakePallet::rewards(id).is_zero(),
					"Rewards not zero for acc_id {:?}",
					id
				);
				assert!(
					StakePallet::blocks_rewarded(id).is_zero(),
					"BlocksRewaeded not zero for acc_id {:?}",
					id
				);
			});
		});
}

#[test]
fn api_get_staking_rates() {
	let stake = 100_000 * DECIMALS;
	ExtBuilder::default()
		.with_balances(vec![(1, stake), (2, stake), (3, 2 * stake)])
		.with_collators(vec![(1, stake), (2, stake)])
		.with_delegators(vec![(3, 1, stake)])
		.with_inflation(25, 10, 25, 8, <Test as Config>::BLOCKS_PER_YEAR)
		.build_and_execute_with_sanity_tests(|| {
			let mut rates = StakingRates {
				collator_staking_rate: Perquintill::from_percent(50),
				collator_reward_rate: Perquintill::from_percent(5),
				delegator_staking_rate: Perquintill::from_percent(25),
				delegator_reward_rate: Perquintill::from_percent(8),
			};
			// collators exceed max staking rate
			assert_eq!(rates, StakePallet::get_staking_rates());

			// candidates stake less to not exceed max staking rate
			assert_ok!(StakePallet::candidate_stake_less(RuntimeOrigin::signed(1), stake / 2));
			assert_ok!(StakePallet::candidate_stake_less(RuntimeOrigin::signed(2), stake / 2));
			// delegator stakes more to exceed
			assert_ok!(StakePallet::delegator_stake_more(RuntimeOrigin::signed(3), stake));
			rates.collator_staking_rate = Perquintill::from_percent(25);
			rates.collator_reward_rate = Perquintill::from_percent(10);
			rates.delegator_staking_rate = Perquintill::from_percent(50);
			rates.delegator_reward_rate = Perquintill::from_percent(4);
			assert_eq!(rates, StakePallet::get_staking_rates());
		});
}

#[test]
fn set_max_selected_candidates_total_stake() {
	let balances: Vec<(AccountId, Balance)> = (1..19).map(|x| (x, 100)).collect();
	ExtBuilder::default()
		.with_balances(balances)
		.with_collators(vec![
			(1, 11),
			(2, 12),
			(3, 13),
			(4, 14),
			(5, 15),
			(6, 16),
			(7, 17),
			(8, 18),
		])
		.with_delegators(vec![
			(11, 1, 21),
			(12, 2, 22),
			(13, 3, 23),
			(14, 4, 24),
			(15, 5, 25),
			(16, 6, 26),
			(17, 7, 27),
			(18, 8, 28),
		])
		.build_and_execute_with_sanity_tests(|| {
			assert_eq!(
				StakePallet::total_collator_stake(),
				TotalStake {
					collators: 35,
					delegators: 55,
				}
			);

			assert_ok!(StakePallet::set_max_selected_candidates(RuntimeOrigin::root(), 3));
			assert_eq!(
				StakePallet::total_collator_stake(),
				TotalStake {
					collators: 51,
					delegators: 81,
				}
			);

			assert_ok!(StakePallet::set_max_selected_candidates(RuntimeOrigin::root(), 5));
			assert_eq!(
				StakePallet::total_collator_stake(),
				TotalStake {
					collators: 80,
					delegators: 130,
				}
			);

			assert_ok!(StakePallet::set_max_selected_candidates(RuntimeOrigin::root(), 10));
			assert_eq!(
				StakePallet::total_collator_stake(),
				TotalStake {
					collators: 116,
					delegators: 196,
				}
			);

			assert_ok!(StakePallet::set_max_selected_candidates(RuntimeOrigin::root(), 2));
			assert_eq!(
				StakePallet::total_collator_stake(),
				TotalStake {
					collators: 35,
					delegators: 55,
				}
			);
		});
}
