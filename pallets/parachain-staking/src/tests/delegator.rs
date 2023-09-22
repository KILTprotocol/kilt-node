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

use std::convert::TryInto;

use frame_support::{assert_noop, assert_ok, storage::bounded_btree_map::BoundedBTreeMap, traits::fungible::Inspect};

use pallet_balances::Error as BalancesError;

use sp_runtime::{traits::Zero, SaturatedConversion};

use crate::{
	mock::{
		events, last_event, roll_to, Balances, BlockNumber, ExtBuilder, RuntimeOrigin, StakePallet, System, Test,
		DECIMALS,
	},
	set::OrderedSet,
	types::{BalanceOf, DelegationCounter, Stake, StakeOf},
	Config, Error, Event, Event as StakeEvent,
};

#[test]
fn multiple_delegations() {
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
			(11, 100),
			(12, 100),
			// new
			(13, 100),
			(14, 100),
			(15, 100),
			(16, 100),
			(17, 100),
			(18, 100),
			(99, 1),
		])
		.with_collators(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 10)])
		.with_delegators(vec![(6, 1, 10), (7, 1, 10), (8, 2, 10), (9, 2, 10), (10, 1, 10)])
		.set_blocks_per_round(5)
		.build_and_execute_with_sanity_tests(|| {
			assert_ok!(StakePallet::set_max_selected_candidates(RuntimeOrigin::root(), 5));
			roll_to(
				8,
				vec![Some(1), Some(2), Some(3), Some(4), Some(5), Some(1), Some(2), Some(3)],
			);
			// chooses top MaxSelectedCandidates (5), in order
			assert_eq!(StakePallet::selected_candidates().into_inner(), vec![1, 2, 3, 4, 5]);
			let mut expected = vec![Event::MaxSelectedCandidatesSet(2, 5), Event::NewRound(5, 1)];
			assert_eq!(events(), expected);
			assert_noop!(
				StakePallet::join_delegators(RuntimeOrigin::signed(13), 2, 2),
				Error::<Test>::DelegationBelowMin,
			);
			assert_ok!(StakePallet::join_delegators(RuntimeOrigin::signed(13), 2, 10));
			assert_ok!(StakePallet::join_delegators(RuntimeOrigin::signed(14), 4, 10));
			assert_ok!(StakePallet::join_delegators(RuntimeOrigin::signed(15), 3, 10));
			assert_eq!(StakePallet::selected_candidates().into_inner(), vec![1, 2, 4, 3, 5]);
			assert_noop!(
				StakePallet::join_delegators(RuntimeOrigin::signed(6), 5, 10),
				Error::<Test>::AlreadyDelegating,
			);

			roll_to(
				16,
				vec![Some(1), Some(2), Some(3), Some(4), Some(5), Some(1), Some(2), Some(3)],
			);
			assert_eq!(StakePallet::selected_candidates().into_inner(), vec![1, 2, 4, 3, 5]);
			let mut new = vec![
				Event::Delegation(13, 10, 2, 50),
				Event::Delegation(14, 10, 4, 30),
				Event::Delegation(15, 10, 3, 30),
				Event::NewRound(10, 2),
				Event::NewRound(15, 3),
			];
			expected.append(&mut new);
			assert_eq!(events(), expected);

			roll_to(21, vec![Some(1), Some(2), Some(3), Some(4), Some(5)]);
			assert_ok!(StakePallet::join_delegators(RuntimeOrigin::signed(16), 2, 80));
			assert_noop!(
				StakePallet::join_delegators(RuntimeOrigin::signed(99), 3, 11),
				BalancesError::<Test>::InsufficientBalance
			);
			assert_noop!(
				StakePallet::join_delegators(RuntimeOrigin::signed(17), 2, 10),
				Error::<Test>::TooManyDelegators
			);
			// kick 13 by staking 1 more (11 > 10)
			assert!(StakePallet::unstaking(13).is_empty());
			assert_ok!(StakePallet::join_delegators(RuntimeOrigin::signed(17), 2, 11));
			assert!(StakePallet::delegator_state(13).is_none());
			assert_eq!(StakePallet::unstaking(13).get(&23), Some(&10u128));
			// kick 9 by staking 1 more (11 > 10)
			assert!(StakePallet::unstaking(9).is_empty());
			assert!(StakePallet::rewards(9).is_zero());
			assert_ok!(StakePallet::join_delegators(RuntimeOrigin::signed(11), 2, 11));
			// 11 should be initiated with the same rewarded counter as the authored counter
			// by their collator 2
			assert_eq!(StakePallet::blocks_rewarded(2), StakePallet::blocks_authored(11));

			assert!(StakePallet::delegator_state(9).is_none());
			assert_eq!(StakePallet::unstaking(9).get(&23), Some(&10u128));
			assert!(!StakePallet::candidate_pool(2)
				.unwrap()
				.delegators
				.contains(&StakeOf::<Test> { owner: 9, amount: 10 }));

			roll_to(26, vec![Some(1), Some(2), Some(3), Some(4), Some(5)]);
			assert_eq!(StakePallet::selected_candidates().into_inner(), vec![2, 1, 4, 3, 5]);
			let mut new2 = vec![
				Event::NewRound(20, 4),
				Event::Delegation(16, 80, 2, 130),
				Event::DelegationReplaced(17, 11, 13, 10, 2, 131),
				Event::Delegation(17, 11, 2, 131),
				Event::DelegationReplaced(11, 11, 9, 10, 2, 132),
				Event::Delegation(11, 11, 2, 132),
				Event::NewRound(25, 5),
			];
			expected.append(&mut new2);
			assert_eq!(events(), expected);
			assert_ok!(StakePallet::init_leave_candidates(RuntimeOrigin::signed(2)));
			assert_eq!(StakePallet::selected_candidates().into_inner(), vec![1, 4, 3, 5]);
			assert_eq!(last_event(), StakeEvent::CollatorScheduledExit(5, 2, 7));

			roll_to(31, vec![Some(1), Some(2), Some(3), Some(4), Some(5)]);
			let mut new3 = vec![
				Event::LeftTopCandidates(2),
				Event::CollatorScheduledExit(5, 2, 7),
				Event::NewRound(30, 6),
			];
			expected.append(&mut new3);
			assert_eq!(events(), expected);

			// test join_delegator errors
			assert_ok!(StakePallet::join_delegators(RuntimeOrigin::signed(18), 1, 10));
			assert_noop!(
				StakePallet::join_delegators(RuntimeOrigin::signed(12), 1, 10),
				Error::<Test>::TooManyDelegators
			);
			assert_ok!(StakePallet::join_delegators(RuntimeOrigin::signed(12), 1, 11));

			// verify that delegations are removed after collator leaves, not before
			assert!(StakePallet::candidate_pool(2)
				.unwrap()
				.delegators
				.contains(&StakeOf::<Test> { owner: 8, amount: 10 }));
			assert!(StakePallet::candidate_pool(2)
				.unwrap()
				.delegators
				.contains(&StakeOf::<Test> { owner: 17, amount: 11 }));
			assert_eq!(StakePallet::delegator_state(8).unwrap().amount, 10);
			assert_eq!(StakePallet::delegator_state(17).unwrap().amount, 11);
			assert_eq!(Balances::usable_balance(8), 90);
			assert_eq!(Balances::usable_balance(17), 89);
			assert_eq!(Balances::balance(&8), 100);
			assert_eq!(Balances::balance(&17), 100);

			roll_to(35, vec![Some(1), Some(2), Some(3), Some(4)]);
			assert_ok!(StakePallet::execute_leave_candidates(RuntimeOrigin::signed(2), 2));
			let mut unbonding_8: BoundedBTreeMap<BlockNumber, BalanceOf<Test>, <Test as Config>::MaxUnstakeRequests> =
				BoundedBTreeMap::new();
			assert_ok!(unbonding_8.try_insert(35u64 + <Test as Config>::StakeDuration::get() as u64, 10));
			assert_eq!(StakePallet::unstaking(8), unbonding_8);
			let mut unbonding_17: BoundedBTreeMap<BlockNumber, BalanceOf<Test>, <Test as Config>::MaxUnstakeRequests> =
				BoundedBTreeMap::new();
			assert_ok!(unbonding_17.try_insert(35u64 + <Test as Config>::StakeDuration::get() as u64, 11));
			assert_eq!(StakePallet::unstaking(17), unbonding_17);

			roll_to(37, vec![Some(1), Some(2)]);
			assert!(StakePallet::delegator_state(8).is_none());
			assert!(StakePallet::delegator_state(17).is_none());
			assert_ok!(StakePallet::unlock_unstaked(RuntimeOrigin::signed(8), 8));
			assert_ok!(StakePallet::unlock_unstaked(RuntimeOrigin::signed(17), 17));
			assert_noop!(
				StakePallet::unlock_unstaked(RuntimeOrigin::signed(12), 12),
				Error::<Test>::UnstakingIsEmpty
			);
			assert_eq!(Balances::usable_balance(17), 100);
			assert_eq!(Balances::usable_balance(8), 100);
			assert_eq!(Balances::balance(&17), 100);
			assert_eq!(Balances::balance(&8), 100);
		});
}

#[test]
fn delegators_bond() {
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
		])
		.with_collators(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 10)])
		.with_delegators(vec![(6, 1, 10), (7, 1, 10), (8, 2, 10), (9, 2, 10)])
		.set_blocks_per_round(5)
		.build_and_execute_with_sanity_tests(|| {
			roll_to(4, vec![]);
			assert_noop!(
				StakePallet::join_delegators(RuntimeOrigin::signed(6), 2, 50),
				Error::<Test>::AlreadyDelegating
			);
			assert_noop!(
				StakePallet::delegator_stake_more(RuntimeOrigin::signed(1), 50),
				Error::<Test>::DelegatorNotFound
			);
			assert_noop!(
				StakePallet::delegator_stake_less(RuntimeOrigin::signed(1), 50),
				Error::<Test>::DelegatorNotFound
			);
			assert_noop!(
				StakePallet::delegator_stake_less(RuntimeOrigin::signed(6), 11),
				Error::<Test>::Underflow
			);
			assert_noop!(
				StakePallet::delegator_stake_less(RuntimeOrigin::signed(6), 8),
				Error::<Test>::DelegationBelowMin
			);
			assert_ok!(StakePallet::delegator_stake_more(RuntimeOrigin::signed(6), 10));
			assert_noop!(
				StakePallet::delegator_stake_more(RuntimeOrigin::signed(6), 81),
				BalancesError::<Test>::InsufficientBalance
			);
			assert_noop!(
				StakePallet::join_delegators(RuntimeOrigin::signed(10), 1, 4),
				Error::<Test>::DelegationBelowMin
			);

			roll_to(9, vec![]);
			assert_eq!(Balances::usable_balance(6), 80);
			assert_ok!(StakePallet::init_leave_candidates(RuntimeOrigin::signed(1)));
			assert!(StakePallet::candidate_pool(1)
				.unwrap()
				.can_exit(1 + <Test as Config>::ExitQueueDelay::get()));

			roll_to(31, vec![]);
			assert!(StakePallet::is_delegator(&6));
			assert_ok!(StakePallet::execute_leave_candidates(RuntimeOrigin::signed(1), 1));
			assert!(!StakePallet::is_delegator(&6));
			assert_eq!(Balances::usable_balance(6), 80);
			assert_eq!(Balances::balance(&6), 100);
		});
}

#[test]
fn should_leave_delegators() {
	ExtBuilder::default()
		.with_balances(vec![(1, 100), (2, 100), (3, 100)])
		.with_collators(vec![(1, 100), (3, 10)])
		.with_delegators(vec![(2, 1, 100)])
		.build_and_execute_with_sanity_tests(|| {
			assert_ok!(StakePallet::leave_delegators(RuntimeOrigin::signed(2)));
			assert!(StakePallet::delegator_state(2).is_none());
			assert!(!StakePallet::candidate_pool(1)
				.unwrap()
				.delegators
				.contains(&StakeOf::<Test> { owner: 2, amount: 100 }));
			assert_noop!(
				StakePallet::leave_delegators(RuntimeOrigin::signed(2)),
				Error::<Test>::DelegatorNotFound
			);
			assert_noop!(
				StakePallet::leave_delegators(RuntimeOrigin::signed(1)),
				Error::<Test>::DelegatorNotFound
			);
		});
}

#[test]
#[should_panic]
fn should_deny_low_delegator_stake() {
	ExtBuilder::default()
		.with_balances(vec![(1, 10 * DECIMALS), (2, 10 * DECIMALS), (3, 10 * DECIMALS), (4, 1)])
		.with_collators(vec![(1, 10 * DECIMALS), (2, 10 * DECIMALS)])
		.with_delegators(vec![(4, 2, 1)])
		.build_and_execute_with_sanity_tests(|| {});
}

#[test]
fn kick_delegator_with_full_unstaking() {
	ExtBuilder::default()
		.with_balances(vec![
			(1, 200),
			(2, 200),
			(3, 200),
			(4, 200),
			(5, 420),
			(6, 200),
			(7, 100),
		])
		.with_collators(vec![(1, 200), (7, 10)])
		.with_delegators(vec![(2, 1, 200), (3, 1, 200), (4, 1, 200), (5, 1, 200)])
		.build_and_execute_with_sanity_tests(|| {
			let max_unstake_reqs: usize = <Test as Config>::MaxUnstakeRequests::get()
				.saturating_sub(1)
				.saturated_into();
			// Fill unstake requests
			for block in 1u64..1u64.saturating_add(max_unstake_reqs as u64) {
				System::set_block_number(block);
				assert_ok!(StakePallet::delegator_stake_less(RuntimeOrigin::signed(5), 1));
			}
			assert_eq!(StakePallet::unstaking(5).into_inner().len(), max_unstake_reqs);

			// Additional unstake should fail
			System::set_block_number(100);
			assert_noop!(
				StakePallet::delegator_stake_less(RuntimeOrigin::signed(5), 1),
				Error::<Test>::NoMoreUnstaking
			);

			// Fill last unstake request by replacing delegator
			assert_ok!(StakePallet::join_delegators(RuntimeOrigin::signed(6), 1, 200));
			assert_eq!(StakePallet::unstaking(5).into_inner().len(), max_unstake_reqs + 1);
			assert!(!StakePallet::is_delegator(&5));

			// Cannot join with full unstaking
			assert_noop!(
				StakePallet::join_delegators(RuntimeOrigin::signed(5), 1, 100),
				Error::<Test>::CannotJoinBeforeUnlocking
			);
			assert_ok!(StakePallet::unlock_unstaked(RuntimeOrigin::signed(5), 5));
			assert_ok!(StakePallet::join_delegators(RuntimeOrigin::signed(5), 1, 220));
		});
}

#[test]
fn exceed_delegations_per_round() {
	ExtBuilder::default()
		.with_balances(vec![(1, 100), (2, 100), (3, 100)])
		.with_collators(vec![(1, 100), (3, 10)])
		.with_delegators(vec![(2, 1, 100)])
		.build_and_execute_with_sanity_tests(|| {
			// leave and re-join to set counter to 2 (= MaxDelegationsPerRound)
			assert_ok!(StakePallet::leave_delegators(RuntimeOrigin::signed(2)));
			assert_ok!(StakePallet::join_delegators(RuntimeOrigin::signed(2), 1, 100));
			assert_ok!(StakePallet::leave_delegators(RuntimeOrigin::signed(2)));
			// reached max delegations in this round
			assert_noop!(
				StakePallet::join_delegators(RuntimeOrigin::signed(2), 1, 100),
				Error::<Test>::DelegationsPerRoundExceeded
			);

			// roll to next round to clear DelegationCounter
			roll_to(5, vec![]);
			assert_eq!(
				StakePallet::last_delegation(2),
				DelegationCounter { round: 0, counter: 2 }
			);
			assert_ok!(StakePallet::join_delegators(RuntimeOrigin::signed(2), 1, 100));
			// counter should be reset because the round changed
			assert_eq!(
				StakePallet::last_delegation(2),
				DelegationCounter { round: 1, counter: 1 }
			);
			// leave and re-join to set counter to 2 (= MaxDelegationsPerRound))
			assert_ok!(StakePallet::leave_delegators(RuntimeOrigin::signed(2)));
			assert_ok!(StakePallet::join_delegators(RuntimeOrigin::signed(2), 1, 100));
			assert_noop!(
				StakePallet::join_delegators(RuntimeOrigin::signed(2), 1, 100),
				Error::<Test>::AlreadyDelegating
			);
			assert_ok!(StakePallet::leave_delegators(RuntimeOrigin::signed(2)));
			assert_noop!(
				StakePallet::join_delegators(RuntimeOrigin::signed(2), 1, 100),
				Error::<Test>::DelegationsPerRoundExceeded
			);
			assert_eq!(
				StakePallet::last_delegation(2),
				DelegationCounter { round: 1, counter: 2 }
			);
		});
}

#[test]
fn prioritize_delegators() {
	ExtBuilder::default()
		.with_balances(vec![
			(1, 1000),
			(2, 1000),
			(3, 1000),
			(4, 1000),
			(5, 1000),
			(6, 1000),
			(7, 1000),
			(8, 1000),
			(9, 1000),
		])
		.with_collators(vec![(1, 100), (2, 100), (3, 100)])
		.with_delegators(vec![(4, 2, 100), (7, 2, 100), (6, 2, 100)])
		.build_and_execute_with_sanity_tests(|| {
			assert_eq!(StakePallet::selected_candidates().into_inner(), vec![2, 1]);
			assert_eq!(
				StakePallet::candidate_pool(2).unwrap().delegators,
				OrderedSet::from_sorted_set(
					vec![
						StakeOf::<Test> { owner: 4, amount: 100 },
						StakeOf::<Test> { owner: 7, amount: 100 },
						StakeOf::<Test> { owner: 6, amount: 100 },
					]
					.try_into()
					.unwrap()
				)
			);
			assert_ok!(StakePallet::join_delegators(RuntimeOrigin::signed(5), 2, 110));
			assert_eq!(
				StakePallet::candidate_pool(2).unwrap().delegators,
				OrderedSet::from_sorted_set(
					vec![
						StakeOf::<Test> { owner: 5, amount: 110 },
						StakeOf::<Test> { owner: 4, amount: 100 },
						StakeOf::<Test> { owner: 7, amount: 100 },
						StakeOf::<Test> { owner: 6, amount: 100 },
					]
					.try_into()
					.unwrap()
				)
			);

			// delegate_less
			assert_ok!(StakePallet::delegator_stake_less(RuntimeOrigin::signed(5), 10));
			assert_eq!(
				StakePallet::candidate_pool(2).unwrap().delegators,
				OrderedSet::from_sorted_set(
					vec![
						StakeOf::<Test> { owner: 5, amount: 100 },
						StakeOf::<Test> { owner: 4, amount: 100 },
						StakeOf::<Test> { owner: 7, amount: 100 },
						StakeOf::<Test> { owner: 6, amount: 100 },
					]
					.try_into()
					.unwrap()
				)
			);

			// delegate_more
			assert_ok!(StakePallet::delegator_stake_more(RuntimeOrigin::signed(6), 10));
			assert_eq!(
				StakePallet::candidate_pool(2).unwrap().delegators,
				OrderedSet::from_sorted_set(
					vec![
						StakeOf::<Test> { owner: 6, amount: 110 },
						StakeOf::<Test> { owner: 5, amount: 100 },
						StakeOf::<Test> { owner: 4, amount: 100 },
						StakeOf::<Test> { owner: 7, amount: 100 },
					]
					.try_into()
					.unwrap()
				)
			);
			assert_ok!(StakePallet::delegator_stake_more(RuntimeOrigin::signed(7), 10));
			assert_eq!(
				StakePallet::candidate_pool(2).unwrap().delegators,
				OrderedSet::from_sorted_set(
					vec![
						StakeOf::<Test> { owner: 6, amount: 110 },
						StakeOf::<Test> { owner: 7, amount: 110 },
						StakeOf::<Test> { owner: 5, amount: 100 },
						StakeOf::<Test> { owner: 4, amount: 100 },
					]
					.try_into()
					.unwrap()
				)
			);
		});
}

#[test]
fn replace_lowest_delegator() {
	ExtBuilder::default()
		.with_balances(vec![
			(1, 100),
			(2, 100),
			(3, 100),
			(4, 100),
			(5, 100),
			(6, 100),
			(7, 100),
		])
		.with_collators(vec![(1, 100), (7, 10)])
		.with_delegators(vec![(2, 1, 51), (3, 1, 51), (4, 1, 51), (5, 1, 50)])
		.build_and_execute_with_sanity_tests(|| {
			assert_eq!(
				StakePallet::candidate_pool(1).unwrap().delegators.len() as u32,
				<Test as Config>::MaxDelegatorsPerCollator::get()
			);

			// 6 replaces 5
			assert_ok!(StakePallet::join_delegators(RuntimeOrigin::signed(6), 1, 51));
			assert!(StakePallet::delegator_state(5).is_none());
			assert_eq!(
				StakePallet::candidate_pool(1)
					.unwrap()
					.delegators
					.into_bounded_vec()
					.into_inner(),
				vec![
					Stake { owner: 2, amount: 51 },
					Stake { owner: 3, amount: 51 },
					Stake { owner: 4, amount: 51 },
					Stake { owner: 6, amount: 51 }
				]
			);

			// 5 attempts to replace 6 with more balance than available
			frame_support::assert_noop!(
				StakePallet::join_delegators(RuntimeOrigin::signed(5), 1, 101),
				BalancesError::<Test>::InsufficientBalance
			);
			assert!(StakePallet::delegator_state(6).is_some());
		})
}
