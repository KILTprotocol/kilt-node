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

use std::{convert::TryInto, iter};

use frame_support::{assert_noop, assert_ok, storage::bounded_btree_map::BoundedBTreeMap, BoundedVec};
use pallet_balances::Error as BalancesError;

use sp_runtime::SaturatedConversion;

use crate::{
	mock::{
		events, last_event, roll_to, AccountId, Balance, BlockNumber, ExtBuilder, RuntimeOrigin, Session, StakePallet,
		System, Test, BLOCKS_PER_ROUND, DECIMALS,
	},
	set::OrderedSet,
	types::{BalanceOf, Candidate, CandidateStatus, Delegator, Stake, StakeOf, TotalStake},
	CandidatePool, Config, Error, Event, Event as StakeEvent,
};

#[test]
fn join_collator_candidates() {
	ExtBuilder::default()
		.with_balances(vec![
			(1, 1000),
			(2, 300),
			(3, 100),
			(4, 100),
			(5, 100),
			(6, 100),
			(7, 100),
			(8, 9),
			(9, 4),
			(10, 161_000_000 * DECIMALS),
		])
		.with_collators(vec![(1, 500), (2, 200)])
		.with_delegators(vec![(3, 1, 100), (4, 1, 100), (5, 2, 100), (6, 2, 100)])
		.build_and_execute_with_sanity_tests(|| {
			assert_eq!(CandidatePool::<Test>::count(), 2);
			assert_eq!(
				StakePallet::total_collator_stake(),
				TotalStake {
					collators: 700,
					delegators: 400
				}
			);
			assert_noop!(
				StakePallet::join_candidates(RuntimeOrigin::signed(1), 11u128,),
				Error::<Test>::CandidateExists
			);
			assert_noop!(
				StakePallet::join_delegators(RuntimeOrigin::signed(1), 1, 11u128,),
				Error::<Test>::CandidateExists
			);
			assert_noop!(
				StakePallet::join_candidates(RuntimeOrigin::signed(3), 11u128,),
				Error::<Test>::DelegatorExists
			);
			assert_noop!(
				StakePallet::join_candidates(RuntimeOrigin::signed(7), 9u128,),
				Error::<Test>::ValStakeBelowMin
			);
			assert_noop!(
				StakePallet::join_candidates(RuntimeOrigin::signed(8), 10u128,),
				BalancesError::<Test>::InsufficientBalance
			);

			assert_eq!(CandidatePool::<Test>::count(), 2);
			assert!(System::events().is_empty());

			assert_ok!(StakePallet::set_max_selected_candidates(RuntimeOrigin::root(), 5));
			assert_ok!(StakePallet::join_candidates(RuntimeOrigin::signed(7), 10u128,));
			assert_eq!(last_event(), StakeEvent::JoinedCollatorCandidates(7, 10u128));

			// MaxCollatorCandidateStake
			assert_noop!(
				StakePallet::join_candidates(RuntimeOrigin::signed(10), 161_000_000 * DECIMALS),
				Error::<Test>::ValStakeAboveMax
			);
			assert_ok!(StakePallet::join_candidates(
				RuntimeOrigin::signed(10),
				StakePallet::max_candidate_stake()
			));
			assert_eq!(CandidatePool::<Test>::count(), 4);

			assert_eq!(
				last_event(),
				StakeEvent::JoinedCollatorCandidates(10, StakePallet::max_candidate_stake(),)
			);
		});
}

#[test]
fn collator_exit_executes_after_delay() {
	ExtBuilder::default()
		.with_balances(vec![
			(1, 1000),
			(2, 300),
			(3, 110),
			(4, 100),
			(5, 100),
			(6, 100),
			(7, 100),
			(8, 9),
			(9, 4),
			(10, 10),
		])
		.with_collators(vec![(1, 500), (2, 200), (7, 100)])
		.with_delegators(vec![(3, 1, 100), (4, 1, 100), (5, 2, 100), (6, 2, 100)])
		.build_and_execute_with_sanity_tests(|| {
			assert_eq!(CandidatePool::<Test>::count(), 3);
			assert_eq!(
				StakePallet::total_collator_stake(),
				TotalStake {
					collators: 700,
					delegators: 400
				}
			);
			assert_ok!(StakePallet::set_max_selected_candidates(RuntimeOrigin::root(), 5));
			assert_eq!(
				StakePallet::total_collator_stake(),
				TotalStake {
					collators: 800,
					delegators: 400
				}
			);
			assert_eq!(StakePallet::selected_candidates().into_inner(), vec![1, 2, 7]);
			roll_to(4, vec![]);
			assert_noop!(
				StakePallet::init_leave_candidates(RuntimeOrigin::signed(3)),
				Error::<Test>::CandidateNotFound
			);

			roll_to(11, vec![]);
			assert_ok!(StakePallet::init_leave_candidates(RuntimeOrigin::signed(2)));
			// Still three, candidate didn't leave yet
			assert_eq!(CandidatePool::<Test>::count(), 3);
			assert_noop!(
				StakePallet::join_delegators(RuntimeOrigin::signed(10), 2, 10),
				Error::<Test>::CannotDelegateIfLeaving
			);
			assert_eq!(StakePallet::selected_candidates().into_inner(), vec![1, 7]);
			assert_eq!(last_event(), StakeEvent::CollatorScheduledExit(2, 2, 4));
			let info = StakePallet::candidate_pool(2).unwrap();
			assert_eq!(info.status, CandidateStatus::Leaving(4));

			roll_to(21, vec![]);
			assert_ok!(StakePallet::execute_leave_candidates(RuntimeOrigin::signed(2), 2));
			assert_eq!(CandidatePool::<Test>::count(), 2);

			// we must exclude leaving collators from rewards while
			// holding them retroactively accountable for previous faults
			// (within the last T::StakeDuration blocks)
			roll_to(25, vec![]);
			let expected = vec![
				Event::MaxSelectedCandidatesSet(2, 5),
				Event::NewRound(5, 1),
				Event::NewRound(10, 2),
				Event::LeftTopCandidates(2),
				Event::CollatorScheduledExit(2, 2, 4),
				Event::NewRound(15, 3),
				Event::NewRound(20, 4),
				Event::CandidateLeft(2, 400),
				Event::NewRound(25, 5),
			];
			assert_eq!(events(), expected);
		});
}

#[test]
fn collator_selection_chooses_top_candidates() {
	ExtBuilder::default()
		.with_balances(vec![
			(1, 1000),
			(2, 1000),
			(3, 1000),
			(4, 1000),
			(5, 1000),
			(6, 1000),
			(7, 33),
			(8, 33),
			(9, 33),
		])
		.with_collators(vec![(1, 100), (2, 90), (3, 80), (4, 70), (5, 60), (6, 50)])
		.build_and_execute_with_sanity_tests(|| {
			assert_eq!(StakePallet::selected_candidates().into_inner(), vec![1, 2]);
			assert_eq!(
				StakePallet::total_collator_stake(),
				TotalStake {
					collators: 190,
					delegators: 0
				}
			);
			assert_ok!(StakePallet::set_max_selected_candidates(RuntimeOrigin::root(), 5));
			assert_eq!(
				StakePallet::total_collator_stake(),
				TotalStake {
					collators: 400,
					delegators: 0
				}
			);
			roll_to(8, vec![]);
			// should choose top MaxSelectedCandidates (5), in order
			assert_eq!(StakePallet::selected_candidates().into_inner(), vec![1, 2, 3, 4, 5]);
			let expected = vec![Event::MaxSelectedCandidatesSet(2, 5), Event::NewRound(5, 1)];
			assert_eq!(events(), expected);
			assert_ok!(StakePallet::init_leave_candidates(RuntimeOrigin::signed(6)));
			assert_eq!(StakePallet::selected_candidates().into_inner(), vec![1, 2, 3, 4, 5],);
			assert_eq!(last_event(), StakeEvent::CollatorScheduledExit(1, 6, 3));

			roll_to(15, vec![]);
			assert_ok!(StakePallet::execute_leave_candidates(RuntimeOrigin::signed(6), 6));
			assert_eq!(StakePallet::selected_candidates().into_inner(), vec![1, 2, 3, 4, 5]);

			roll_to(21, vec![]);
			assert_ok!(StakePallet::join_candidates(RuntimeOrigin::signed(6), 69u128));
			assert_eq!(StakePallet::selected_candidates().into_inner(), vec![1, 2, 3, 4, 6]);
			assert_eq!(last_event(), StakeEvent::JoinedCollatorCandidates(6, 69u128));

			roll_to(27, vec![]);
			// should choose top MaxSelectedCandidates (5), in order
			let expected = vec![
				Event::MaxSelectedCandidatesSet(2, 5),
				Event::NewRound(5, 1),
				Event::LeftTopCandidates(6),
				Event::CollatorScheduledExit(1, 6, 3),
				// TotalCollatorStake is updated once candidate 6 left in `execute_delayed_collator_exits`
				Event::NewRound(10, 2),
				Event::NewRound(15, 3),
				Event::CandidateLeft(6, 50),
				Event::NewRound(20, 4),
				// 5 had staked 60 which was exceeded by 69 of 6
				Event::EnteredTopCandidates(6),
				Event::JoinedCollatorCandidates(6, 69),
				Event::NewRound(25, 5),
			];
			assert_eq!(events(), expected);
		});
}

#[test]
fn execute_leave_candidates_with_delay() {
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
			(10, 1000),
			(11, 1000),
			(12, 1000),
			(13, 1000),
			(14, 1000),
		])
		.with_collators(vec![
			(1, 10),
			(2, 20),
			(3, 30),
			(4, 40),
			(5, 50),
			(6, 60),
			(7, 70),
			(8, 80),
			(9, 90),
			(10, 100),
		])
		.with_delegators(vec![(11, 1, 110), (12, 1, 120), (13, 2, 130), (14, 2, 140)])
		.with_inflation(100, 15, 40, 10, BLOCKS_PER_ROUND)
		.build_and_execute_with_sanity_tests(|| {
			assert_eq!(CandidatePool::<Test>::count(), 10);
			assert_eq!(
				StakePallet::total_collator_stake(),
				TotalStake {
					collators: 30,
					delegators: 500,
				}
			);
			assert_ok!(StakePallet::set_max_selected_candidates(RuntimeOrigin::root(), 5));
			assert_eq!(
				StakePallet::total_collator_stake(),
				TotalStake {
					collators: 300,
					delegators: 500,
				}
			);

			roll_to(5, vec![]);
			// should choose top MaxSelectedCandidates (5), in order
			assert_eq!(StakePallet::selected_candidates().into_inner(), vec![2, 1, 10, 9, 8]);
			assert_ok!(StakePallet::init_leave_candidates(RuntimeOrigin::signed(10)));
			assert_ok!(StakePallet::init_leave_candidates(RuntimeOrigin::signed(9)));
			assert_ok!(StakePallet::init_leave_candidates(RuntimeOrigin::signed(1)));
			assert_ok!(StakePallet::init_leave_candidates(RuntimeOrigin::signed(7)));
			assert_ok!(StakePallet::init_leave_candidates(RuntimeOrigin::signed(6)));
			assert_ok!(StakePallet::init_leave_candidates(RuntimeOrigin::signed(5)));
			assert_ok!(StakePallet::init_leave_candidates(RuntimeOrigin::signed(8)));
			assert_ok!(StakePallet::init_leave_candidates(RuntimeOrigin::signed(2)));
			assert_eq!(StakePallet::selected_candidates().into_inner(), vec![4, 3]);
			for owner in vec![1, 2, 5, 6, 7, 8, 9, 10].iter() {
				assert!(StakePallet::candidate_pool(owner)
					.unwrap()
					.can_exit(1 + <Test as Config>::ExitQueueDelay::get()));
			}
			let total_stake = TotalStake {
				collators: 70,
				delegators: 0,
			};
			assert_eq!(StakePallet::total_collator_stake(), total_stake);
			assert_eq!(
				StakePallet::candidate_pool(1),
				Some(
					Candidate::<AccountId, Balance, <Test as Config>::MaxDelegatorsPerCollator> {
						id: 1,
						stake: 10,
						delegators: OrderedSet::from(
							vec![
								StakeOf::<Test> { owner: 11, amount: 110 },
								StakeOf::<Test> { owner: 12, amount: 120 }
							]
							.try_into()
							.unwrap()
						),
						total: 240,
						status: CandidateStatus::Leaving(3)
					}
				)
			);
			assert_eq!(
				StakePallet::candidate_pool(2),
				Some(
					Candidate::<AccountId, Balance, <Test as Config>::MaxDelegatorsPerCollator> {
						id: 2,
						stake: 20,
						delegators: OrderedSet::from(
							vec![
								StakeOf::<Test> { owner: 13, amount: 130 },
								StakeOf::<Test> { owner: 14, amount: 140 }
							]
							.try_into()
							.unwrap()
						),
						total: 290,
						status: CandidateStatus::Leaving(3)
					}
				)
			);
			for collator in 5u64..=10u64 {
				assert_eq!(
					StakePallet::candidate_pool(collator),
					Some(
						Candidate::<AccountId, Balance, <Test as Config>::MaxDelegatorsPerCollator> {
							id: collator,
							stake: collator as u128 * 10u128,
							delegators: OrderedSet::from(BoundedVec::default()),
							total: collator as u128 * 10u128,
							status: CandidateStatus::Leaving(3)
						}
					)
				);
				assert!(StakePallet::is_active_candidate(&collator).is_some());
				assert!(StakePallet::unstaking(collator).is_empty());
			}
			assert_eq!(
				StakePallet::delegator_state(11),
				Some(Delegator::<AccountId, Balance> { owner: 1, amount: 110 })
			);
			assert_eq!(
				StakePallet::delegator_state(12),
				Some(Delegator::<AccountId, Balance> { owner: 1, amount: 120 })
			);
			assert_eq!(
				StakePallet::delegator_state(13),
				Some(Delegator::<AccountId, Balance> { owner: 2, amount: 130 })
			);
			assert_eq!(
				StakePallet::delegator_state(14),
				Some(Delegator::<AccountId, Balance> { owner: 2, amount: 140 })
			);
			for delegator in 11u64..=14u64 {
				assert!(StakePallet::is_delegator(&delegator));
				assert!(StakePallet::unstaking(delegator).is_empty());
			}

			// exits cannot be executed yet but in the next round
			roll_to(10, vec![]);
			assert_eq!(StakePallet::total_collator_stake(), total_stake);
			assert_eq!(StakePallet::selected_candidates().into_inner(), vec![4, 3]);
			for owner in vec![1, 2, 5, 6, 7, 8, 9, 10].iter() {
				assert!(StakePallet::candidate_pool(owner)
					.unwrap()
					.can_exit(1 + <Test as Config>::ExitQueueDelay::get()));
				assert_noop!(
					StakePallet::execute_leave_candidates(RuntimeOrigin::signed(*owner), *owner),
					Error::<Test>::CannotLeaveYet
				);
			}
			assert_eq!(StakePallet::total_collator_stake(), total_stake);
			assert_eq!(
				StakePallet::candidate_pool(1),
				Some(
					Candidate::<AccountId, Balance, <Test as Config>::MaxDelegatorsPerCollator> {
						id: 1,
						stake: 10,
						delegators: OrderedSet::from(
							vec![
								StakeOf::<Test> { owner: 11, amount: 110 },
								StakeOf::<Test> { owner: 12, amount: 120 }
							]
							.try_into()
							.unwrap()
						),
						total: 240,
						status: CandidateStatus::Leaving(3)
					}
				)
			);
			assert_eq!(
				StakePallet::candidate_pool(2),
				Some(
					Candidate::<AccountId, Balance, <Test as Config>::MaxDelegatorsPerCollator> {
						id: 2,
						stake: 20,
						delegators: OrderedSet::from(
							vec![
								StakeOf::<Test> { owner: 13, amount: 130 },
								StakeOf::<Test> { owner: 14, amount: 140 }
							]
							.try_into()
							.unwrap()
						),
						total: 290,
						status: CandidateStatus::Leaving(3)
					}
				)
			);
			for collator in 5u64..=10u64 {
				assert_eq!(
					StakePallet::candidate_pool(collator),
					Some(
						Candidate::<AccountId, Balance, <Test as Config>::MaxDelegatorsPerCollator> {
							id: collator,
							stake: collator as u128 * 10u128,
							delegators: OrderedSet::from(BoundedVec::default()),
							total: collator as u128 * 10u128,
							status: CandidateStatus::Leaving(3)
						}
					)
				);
				assert!(StakePallet::is_active_candidate(&collator).is_some());
				assert!(StakePallet::unstaking(collator).is_empty());
			}
			assert_eq!(
				StakePallet::delegator_state(11),
				Some(Delegator::<AccountId, Balance> { owner: 1, amount: 110 })
			);
			assert_eq!(
				StakePallet::delegator_state(12),
				Some(Delegator::<AccountId, Balance> { owner: 1, amount: 120 })
			);
			assert_eq!(
				StakePallet::delegator_state(13),
				Some(Delegator::<AccountId, Balance> { owner: 2, amount: 130 })
			);
			assert_eq!(
				StakePallet::delegator_state(14),
				Some(Delegator::<AccountId, Balance> { owner: 2, amount: 140 })
			);
			for delegator in 11u64..=14u64 {
				assert!(StakePallet::is_delegator(&delegator));
				assert!(StakePallet::unstaking(delegator).is_empty());
			}

			// first five exits are executed
			roll_to(15, vec![]);
			assert_eq!(StakePallet::total_collator_stake(), total_stake);
			assert_eq!(StakePallet::selected_candidates().into_inner(), vec![4, 3]);
			for collator in vec![1u64, 2u64, 5u64, 6u64, 7u64].iter() {
				assert_ok!(StakePallet::execute_leave_candidates(
					RuntimeOrigin::signed(*collator),
					*collator
				));
				assert!(StakePallet::candidate_pool(&collator).is_none());
				assert!(StakePallet::is_active_candidate(collator).is_none());
				assert_eq!(StakePallet::unstaking(collator).len(), 1);
			}
			assert_eq!(CandidatePool::<Test>::count(), 5, "Five collators left.");

			assert_eq!(StakePallet::total_collator_stake(), total_stake);
			for delegator in 11u64..=14u64 {
				assert!(!StakePallet::is_delegator(&delegator));
				assert_eq!(StakePallet::unstaking(delegator).len(), 1);
			}

			// last 3 exits are executed
			roll_to(20, vec![]);
			for collator in 8u64..=10u64 {
				assert_ok!(StakePallet::execute_leave_candidates(
					RuntimeOrigin::signed(collator),
					collator
				));
				assert!(StakePallet::candidate_pool(collator).is_none());
				assert!(StakePallet::is_active_candidate(&collator).is_none());
				assert_eq!(StakePallet::unstaking(collator).len(), 1);
			}
			assert_eq!(CandidatePool::<Test>::count(), 2, "3 collators left.");
		});
}

#[test]
fn collators_bond() {
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
		.with_delegators(vec![(6, 1, 10), (7, 1, 10), (8, 2, 10), (9, 2, 10), (10, 1, 10)])
		.set_blocks_per_round(5)
		.build_and_execute_with_sanity_tests(|| {
			roll_to(4, vec![]);
			assert_noop!(
				StakePallet::candidate_stake_more(RuntimeOrigin::signed(6), 50),
				Error::<Test>::CandidateNotFound
			);
			assert_noop!(
				StakePallet::candidate_stake_less(RuntimeOrigin::signed(6), 50),
				Error::<Test>::CandidateNotFound
			);
			assert_ok!(StakePallet::candidate_stake_more(RuntimeOrigin::signed(1), 50));
			assert_noop!(
				StakePallet::candidate_stake_more(RuntimeOrigin::signed(1), 40),
				BalancesError::<Test>::InsufficientBalance
			);
			assert_ok!(StakePallet::init_leave_candidates(RuntimeOrigin::signed(1)));
			assert!(StakePallet::candidate_pool(1)
				.unwrap()
				.can_exit(<Test as Config>::ExitQueueDelay::get()));

			assert_noop!(
				StakePallet::candidate_stake_more(RuntimeOrigin::signed(1), 30),
				Error::<Test>::CannotStakeIfLeaving
			);
			assert_noop!(
				StakePallet::candidate_stake_less(RuntimeOrigin::signed(1), 10),
				Error::<Test>::CannotStakeIfLeaving
			);

			roll_to(30, vec![]);
			assert_ok!(StakePallet::execute_leave_candidates(RuntimeOrigin::signed(1), 1));
			assert_noop!(
				StakePallet::candidate_stake_more(RuntimeOrigin::signed(1), 40),
				Error::<Test>::CandidateNotFound
			);
			assert_ok!(StakePallet::candidate_stake_more(RuntimeOrigin::signed(2), 80));
			assert_ok!(StakePallet::candidate_stake_less(RuntimeOrigin::signed(2), 90));
			assert_ok!(StakePallet::candidate_stake_less(RuntimeOrigin::signed(3), 10));
			assert_noop!(
				StakePallet::candidate_stake_less(RuntimeOrigin::signed(2), 11),
				Error::<Test>::Underflow
			);
			assert_noop!(
				StakePallet::candidate_stake_less(RuntimeOrigin::signed(2), 1),
				Error::<Test>::ValStakeBelowMin
			);
			assert_noop!(
				StakePallet::candidate_stake_less(RuntimeOrigin::signed(3), 1),
				Error::<Test>::ValStakeBelowMin
			);
			assert_noop!(
				StakePallet::candidate_stake_less(RuntimeOrigin::signed(4), 11),
				Error::<Test>::ValStakeBelowMin
			);
			assert_ok!(StakePallet::candidate_stake_less(RuntimeOrigin::signed(4), 10));

			// MaxCollatorCandidateStake
			assert_ok!(StakePallet::join_candidates(
				RuntimeOrigin::signed(11),
				StakePallet::max_candidate_stake()
			));
			assert_noop!(
				StakePallet::candidate_stake_more(RuntimeOrigin::signed(11), 1u128),
				Error::<Test>::ValStakeAboveMax,
			);
		});
}

#[test]
#[should_panic]
fn should_deny_low_collator_stake() {
	ExtBuilder::default()
		.with_balances(vec![(1, 10 * DECIMALS), (2, 5)])
		.with_collators(vec![(1, 10 * DECIMALS), (2, 5)])
		.build_and_execute_with_sanity_tests(|| {});
}

#[test]
#[should_panic]
fn should_deny_duplicate_collators() {
	ExtBuilder::default()
		.with_balances(vec![(1, 10 * DECIMALS)])
		.with_collators(vec![(1, 10 * DECIMALS), (1, 10 * DECIMALS)])
		.build_and_execute_with_sanity_tests(|| {});
}

#[test]
fn kick_candidate_with_full_unstaking() {
	ExtBuilder::default()
		.with_balances(vec![(1, 200), (2, 200), (3, 300)])
		.with_collators(vec![(1, 200), (2, 200), (3, 200)])
		.build_and_execute_with_sanity_tests(|| {
			let max_unstake_reqs: usize = <Test as Config>::MaxUnstakeRequests::get()
				.saturating_sub(1)
				.saturated_into();
			// Fill unstake requests
			for block in 1u64..1u64.saturating_add(max_unstake_reqs as u64) {
				System::set_block_number(block);
				assert_ok!(StakePallet::candidate_stake_less(RuntimeOrigin::signed(3), 1));
			}
			assert_eq!(StakePallet::unstaking(3).into_inner().len(), max_unstake_reqs);

			// Additional unstake should fail
			System::set_block_number(100);
			assert_noop!(
				StakePallet::candidate_stake_less(RuntimeOrigin::signed(3), 1),
				Error::<Test>::NoMoreUnstaking
			);

			// Fill last unstake request by removing candidate and unstaking all stake
			assert_ok!(StakePallet::force_remove_candidate(RuntimeOrigin::root(), 3));

			// Cannot join with full unstaking
			assert_eq!(StakePallet::unstaking(3).into_inner().len(), max_unstake_reqs + 1);
			assert_noop!(
				StakePallet::join_candidates(RuntimeOrigin::signed(3), 100),
				Error::<Test>::CannotJoinBeforeUnlocking
			);
			assert_ok!(StakePallet::unlock_unstaked(RuntimeOrigin::signed(3), 3));
			assert_ok!(StakePallet::join_candidates(RuntimeOrigin::signed(3), 100));
		});
}

#[test]
fn candidate_leaves() {
	let balances: Vec<(AccountId, Balance)> = (1u64..=15u64).map(|id| (id, 100)).collect();
	ExtBuilder::default()
		.with_balances(balances)
		.with_collators(vec![(1, 100), (2, 100)])
		.with_delegators(vec![(12, 1, 100), (13, 1, 10)])
		.build_and_execute_with_sanity_tests(|| {
			assert_eq!(
				StakePallet::top_candidates()
					.into_iter()
					.map(|s| s.owner)
					.collect::<Vec<u64>>(),
				vec![1, 2]
			);
			assert_noop!(
				StakePallet::init_leave_candidates(RuntimeOrigin::signed(11)),
				Error::<Test>::CandidateNotFound
			);
			assert_noop!(
				StakePallet::init_leave_candidates(RuntimeOrigin::signed(1)),
				Error::<Test>::TooFewCollatorCandidates
			);
			// add five more collator to max fill TopCandidates
			for candidate in 3u64..11u64 {
				assert_ok!(StakePallet::join_candidates(RuntimeOrigin::signed(candidate), 100));
			}
			assert_eq!(
				StakePallet::top_candidates()
					.into_iter()
					.map(|s| s.owner)
					.collect::<Vec<u64>>(),
				(1u64..11u64).collect::<Vec<u64>>()
			);
			assert_eq!(CandidatePool::<Test>::count(), 10);
			assert_ok!(StakePallet::init_leave_candidates(RuntimeOrigin::signed(1)));
			assert_eq!(
				StakePallet::top_candidates()
					.into_iter()
					.map(|s| s.owner)
					.collect::<Vec<u64>>(),
				(2u64..11u64).collect::<Vec<u64>>()
			);
			assert_noop!(
				StakePallet::join_delegators(RuntimeOrigin::signed(15), 1, 10),
				Error::<Test>::CannotDelegateIfLeaving
			);
			assert_noop!(
				StakePallet::delegator_stake_more(RuntimeOrigin::signed(12), 1),
				Error::<Test>::CannotDelegateIfLeaving
			);
			assert_noop!(
				StakePallet::delegator_stake_less(RuntimeOrigin::signed(12), 1),
				Error::<Test>::CannotDelegateIfLeaving
			);
			assert_noop!(
				StakePallet::candidate_stake_less(RuntimeOrigin::signed(1), 1),
				Error::<Test>::CannotStakeIfLeaving
			);
			assert_noop!(
				StakePallet::candidate_stake_more(RuntimeOrigin::signed(1), 1),
				Error::<Test>::CannotStakeIfLeaving
			);
			assert_noop!(
				StakePallet::init_leave_candidates(RuntimeOrigin::signed(1)),
				Error::<Test>::AlreadyLeaving
			);
			assert_eq!(
				StakePallet::candidate_pool(1).unwrap().status,
				CandidateStatus::Leaving(2)
			);
			assert!(StakePallet::candidate_pool(1).unwrap().can_exit(2));
			assert!(!StakePallet::candidate_pool(1).unwrap().can_exit(1));
			assert!(StakePallet::candidate_pool(1).unwrap().can_exit(3));

			// next rounds starts, cannot leave yet
			roll_to(5, vec![]);
			assert_noop!(
				StakePallet::execute_leave_candidates(RuntimeOrigin::signed(2), 2),
				Error::<Test>::NotLeaving
			);
			assert_noop!(
				StakePallet::execute_leave_candidates(RuntimeOrigin::signed(2), 1),
				Error::<Test>::CannotLeaveYet
			);
			// add 11 as candidate to reach max size for TopCandidates and then try leave
			// again as 1 which should not be possible
			assert_ok!(StakePallet::join_candidates(RuntimeOrigin::signed(11), 100));
			assert_eq!(
				StakePallet::top_candidates()
					.into_iter()
					.map(|s| s.owner)
					.collect::<Vec<u64>>(),
				(2u64..12u64).collect::<Vec<u64>>()
			);
			assert_ok!(StakePallet::init_leave_candidates(RuntimeOrigin::signed(11)));
			// join back
			assert_ok!(StakePallet::cancel_leave_candidates(RuntimeOrigin::signed(1)));
			assert_eq!(
				StakePallet::top_candidates()
					.into_iter()
					.map(|s| s.owner)
					.collect::<Vec<u64>>(),
				(1u64..11u64).collect::<Vec<u64>>()
			);

			let stake: Vec<Stake<AccountId, Balance>> = (1u64..11u64)
				.zip(iter::once(210).chain(iter::repeat(100)))
				.map(|(id, amount)| StakeOf::<Test> { owner: id, amount })
				.collect();
			assert_eq!(
				StakePallet::top_candidates(),
				OrderedSet::from(stake.try_into().unwrap())
			);
			let state = StakePallet::candidate_pool(1).unwrap();
			assert_eq!(state.status, CandidateStatus::Active);
			assert_eq!(state.delegators.len(), 2);
			assert_eq!(state.total, 210);
			assert_eq!(
				state.total,
				StakePallet::top_candidates()
					.into_bounded_vec()
					.iter()
					.find(|other| other.owner == 1)
					.unwrap()
					.amount
			);
			assert_eq!(StakePallet::selected_candidates().into_inner(), vec![1, 2]);

			assert_ok!(StakePallet::init_leave_candidates(RuntimeOrigin::signed(1)));

			roll_to(15, vec![]);
			assert_ok!(StakePallet::execute_leave_candidates(RuntimeOrigin::signed(13), 1));
			let mut unstaking: BoundedBTreeMap<BlockNumber, BalanceOf<Test>, <Test as Config>::MaxUnstakeRequests> =
				BoundedBTreeMap::new();
			assert_ok!(unstaking.try_insert(17, 100));
			assert_eq!(StakePallet::unstaking(1), unstaking);
			assert_eq!(StakePallet::unstaking(12), unstaking);

			// cannot unlock yet
			roll_to(16, vec![]);
			assert_ok!(StakePallet::unlock_unstaked(RuntimeOrigin::signed(4), 1));
			assert_ok!(StakePallet::unlock_unstaked(RuntimeOrigin::signed(4), 12));
			assert_eq!(StakePallet::unstaking(1), unstaking);
			assert_eq!(StakePallet::unstaking(12), unstaking);

			// can unlock now
			roll_to(17, vec![]);
			unstaking.remove(&17);
			assert_ok!(StakePallet::unlock_unstaked(RuntimeOrigin::signed(4), 1));
			assert_ok!(StakePallet::unlock_unstaked(RuntimeOrigin::signed(4), 12));
			assert_eq!(StakePallet::unstaking(1), unstaking);
			assert_eq!(StakePallet::unstaking(12), unstaking);
		});
}

#[test]
fn increase_max_candidate_stake() {
	let max_stake = 160_000_000 * DECIMALS;
	ExtBuilder::default()
		.with_balances(vec![(1, 200_000_000 * DECIMALS), (3, 100)])
		.with_collators(vec![(1, max_stake), (3, 10)])
		.build_and_execute_with_sanity_tests(|| {
			assert_eq!(StakePallet::max_candidate_stake(), max_stake);
			assert_noop!(
				StakePallet::candidate_stake_more(RuntimeOrigin::signed(1), 1),
				Error::<Test>::ValStakeAboveMax
			);

			assert_ok!(StakePallet::set_max_candidate_stake(
				RuntimeOrigin::root(),
				max_stake + 1
			));
			assert_eq!(last_event(), StakeEvent::MaxCandidateStakeChanged(max_stake + 1));
			assert_eq!(StakePallet::max_candidate_stake(), max_stake + 1);
			assert_ok!(StakePallet::candidate_stake_more(RuntimeOrigin::signed(1), 1));
			assert_noop!(
				StakePallet::candidate_stake_more(RuntimeOrigin::signed(1), 1),
				Error::<Test>::ValStakeAboveMax
			);
		});
}

#[test]
fn decrease_max_candidate_stake() {
	ExtBuilder::default()
		.with_balances(vec![(1, 100), (2, 100), (3, 100), (4, 100), (5, 100)])
		.with_collators(vec![(1, 100), (2, 90), (3, 40)])
		.with_delegators(vec![(4, 2, 10), (5, 3, 20)])
		.build_and_execute_with_sanity_tests(|| {
			assert_eq!(StakePallet::selected_candidates().into_inner(), vec![1, 2]);
			assert_eq!(
				StakePallet::top_candidates(),
				OrderedSet::from_sorted_set(
					vec![
						StakeOf::<Test> { owner: 1, amount: 100 },
						StakeOf::<Test> { owner: 2, amount: 100 },
						StakeOf::<Test> { owner: 3, amount: 60 }
					]
					.try_into()
					.unwrap()
				)
			);

			assert_ok!(StakePallet::set_max_candidate_stake(RuntimeOrigin::root(), 100));
			assert_eq!(StakePallet::max_candidate_stake(), 100);
			assert_eq!(last_event(), StakeEvent::MaxCandidateStakeChanged(100));

			// check collator states, nothing changed
			assert_eq!(
				StakePallet::top_candidates(),
				OrderedSet::from_sorted_set(
					vec![
						StakeOf::<Test> { owner: 1, amount: 100 },
						StakeOf::<Test> { owner: 2, amount: 100 },
						StakeOf::<Test> { owner: 3, amount: 60 }
					]
					.try_into()
					.unwrap()
				)
			);

			assert_noop!(
				StakePallet::candidate_stake_more(RuntimeOrigin::signed(1), 0),
				Error::<Test>::ValStakeZero
			);
			assert_noop!(
				StakePallet::candidate_stake_less(RuntimeOrigin::signed(1), 0),
				Error::<Test>::ValStakeZero
			);
			assert_noop!(
				StakePallet::candidate_stake_more(RuntimeOrigin::signed(1), 1),
				Error::<Test>::ValStakeAboveMax
			);
			assert_ok!(StakePallet::candidate_stake_less(RuntimeOrigin::signed(1), 50));
			assert_noop!(
				StakePallet::set_max_candidate_stake(RuntimeOrigin::root(), 9),
				Error::<Test>::CannotSetBelowMin
			);
		});
}

#[test]
fn force_remove_candidate() {
	ExtBuilder::default()
		.with_balances(vec![(1, 100), (2, 100), (3, 100), (4, 100), (5, 100), (6, 100)])
		.with_collators(vec![(1, 100), (2, 100), (3, 100)])
		.with_delegators(vec![(4, 1, 50), (5, 1, 50)])
		.build_and_execute_with_sanity_tests(|| {
			assert_eq!(CandidatePool::<Test>::count(), 3);
			assert_ok!(StakePallet::join_delegators(RuntimeOrigin::signed(6), 2, 50));
			assert_eq!(StakePallet::selected_candidates().into_inner(), vec![1, 2]);
			assert!(StakePallet::unstaking(1).get(&3).is_none());
			assert!(StakePallet::unstaking(2).get(&3).is_none());
			assert!(StakePallet::unstaking(3).get(&3).is_none());

			// force remove 1
			assert!(Session::disabled_validators().is_empty());
			assert_eq!(
				StakePallet::total_collator_stake(),
				TotalStake {
					collators: 200,
					delegators: 150,
				}
			);
			assert_ok!(StakePallet::force_remove_candidate(RuntimeOrigin::root(), 1));
			// collator stake does not change since 3, who took 1's place, has staked the
			// same amount
			assert_eq!(
				StakePallet::total_collator_stake(),
				TotalStake {
					collators: 200,
					delegators: 50,
				}
			);
			assert_eq!(Session::disabled_validators(), vec![0]);
			assert_eq!(last_event(), StakeEvent::CollatorRemoved(1, 200));
			assert!(!StakePallet::top_candidates().contains(&StakeOf::<Test> { owner: 1, amount: 100 }));
			assert_eq!(StakePallet::selected_candidates().into_inner(), vec![2, 3]);
			assert_eq!(CandidatePool::<Test>::count(), 2);
			assert!(StakePallet::candidate_pool(1).is_none());
			assert!(StakePallet::delegator_state(4).is_none());
			assert!(StakePallet::delegator_state(5).is_none());
			assert_eq!(StakePallet::unstaking(1).get(&3), Some(&100));
			assert_eq!(StakePallet::unstaking(4).get(&3), Some(&50));
			assert_eq!(StakePallet::unstaking(5).get(&3), Some(&50));

			assert_noop!(
				StakePallet::force_remove_candidate(RuntimeOrigin::root(), 2),
				Error::<Test>::TooFewCollatorCandidates
			);
			assert_noop!(
				StakePallet::force_remove_candidate(RuntimeOrigin::root(), 4),
				Error::<Test>::CandidateNotFound
			);

			// session 1: expect 1 to still be in validator set but as disabled
			roll_to(5, vec![]);
			assert_eq!(Session::current_index(), 1);
			assert_eq!(Session::validators(), vec![1, 2]);
			assert_eq!(Session::disabled_validators(), vec![0]);

			// session 2: expect validator set to have changed
			roll_to(10, vec![]);
			assert_eq!(Session::validators(), vec![2, 3]);
			assert!(Session::disabled_validators().is_empty());
		});
}

#[test]
fn prioritize_collators() {
	ExtBuilder::default()
		.with_balances(vec![
			(1, 200),
			(2, 200),
			(3, 200),
			(4, 200),
			(5, 200),
			(6, 200),
			(7, 200),
		])
		.with_collators(vec![(2, 100), (3, 100)])
		.build_and_execute_with_sanity_tests(|| {
			assert_eq!(
				StakePallet::top_candidates(),
				OrderedSet::from_sorted_set(
					vec![2, 3]
						.into_iter()
						.map(|id| StakeOf::<Test> { owner: id, amount: 100 })
						.collect::<Vec<StakeOf<Test>>>()
						.try_into()
						.unwrap()
				)
			);
			assert_eq!(StakePallet::selected_candidates().into_inner(), vec![2, 3]);
			assert_ok!(StakePallet::join_candidates(RuntimeOrigin::signed(1), 100));
			assert_eq!(
				StakePallet::top_candidates(),
				OrderedSet::from_sorted_set(
					vec![2, 3, 1]
						.into_iter()
						.map(|id| StakeOf::<Test> { owner: id, amount: 100 })
						.collect::<Vec<StakeOf<Test>>>()
						.try_into()
						.unwrap()
				)
			);
			assert_eq!(StakePallet::selected_candidates().into_inner(), vec![2, 3]);
			assert_ok!(StakePallet::init_leave_candidates(RuntimeOrigin::signed(2)));
			assert_eq!(StakePallet::top_candidates().len(), 2);
			assert_eq!(StakePallet::selected_candidates().into_inner(), vec![3, 1]);
			assert_ok!(StakePallet::candidate_stake_less(RuntimeOrigin::signed(3), 10));
			assert_eq!(StakePallet::selected_candidates().into_inner(), vec![1, 3]);

			// add 6
			assert_ok!(StakePallet::join_candidates(RuntimeOrigin::signed(6), 100));
			assert_eq!(StakePallet::selected_candidates().into_inner(), vec![1, 6]);
			assert_eq!(
				StakePallet::top_candidates(),
				OrderedSet::from_sorted_set(
					vec![1, 6]
						.into_iter()
						.map(|id| StakeOf::<Test> { owner: id, amount: 100 })
						.chain(vec![StakeOf::<Test> { owner: 3, amount: 90 }])
						.collect::<Vec<StakeOf<Test>>>()
						.try_into()
						.unwrap()
				)
			);

			// add 4
			assert_ok!(StakePallet::join_candidates(RuntimeOrigin::signed(4), 100));
			assert_eq!(StakePallet::selected_candidates().into_inner(), vec![1, 6]);
			assert_eq!(
				StakePallet::top_candidates(),
				OrderedSet::from_sorted_set(
					vec![1, 6, 4]
						.into_iter()
						.map(|id| StakeOf::<Test> { owner: id, amount: 100 })
						.chain(vec![StakeOf::<Test> { owner: 3, amount: 90 }])
						.collect::<Vec<StakeOf<Test>>>()
						.try_into()
						.unwrap()
				)
			);

			// add 5
			assert_ok!(StakePallet::join_candidates(RuntimeOrigin::signed(5), 100));
			assert_eq!(StakePallet::selected_candidates().into_inner(), vec![1, 6]);
			assert_eq!(
				StakePallet::top_candidates(),
				OrderedSet::from_sorted_set(
					vec![1, 6, 4, 5]
						.into_iter()
						.map(|id| StakeOf::<Test> { owner: id, amount: 100 })
						.chain(vec![StakeOf::<Test> { owner: 3, amount: 90 }])
						.collect::<Vec<StakeOf<Test>>>()
						.try_into()
						.unwrap()
				)
			);

			// 3 stake_more
			assert_ok!(StakePallet::candidate_stake_more(RuntimeOrigin::signed(3), 20));
			assert_eq!(StakePallet::selected_candidates().into_inner(), vec![3, 1]);
			assert_eq!(
				StakePallet::top_candidates(),
				OrderedSet::from_sorted_set(
					vec![
						StakeOf::<Test> { owner: 3, amount: 110 },
						StakeOf::<Test> { owner: 1, amount: 100 },
						StakeOf::<Test> { owner: 6, amount: 100 },
						StakeOf::<Test> { owner: 4, amount: 100 },
						StakeOf::<Test> { owner: 5, amount: 100 },
					]
					.try_into()
					.unwrap()
				)
			);

			// 1 stake_less
			assert_ok!(StakePallet::candidate_stake_less(RuntimeOrigin::signed(1), 1));
			assert_eq!(StakePallet::selected_candidates().into_inner(), vec![3, 6]);
			assert_eq!(
				StakePallet::top_candidates(),
				OrderedSet::from_sorted_set(
					vec![
						StakeOf::<Test> { owner: 3, amount: 110 },
						StakeOf::<Test> { owner: 6, amount: 100 },
						StakeOf::<Test> { owner: 4, amount: 100 },
						StakeOf::<Test> { owner: 5, amount: 100 },
						StakeOf::<Test> { owner: 1, amount: 99 },
					]
					.try_into()
					.unwrap()
				)
			);

			// 7 delegates to 4
			assert_ok!(StakePallet::join_delegators(RuntimeOrigin::signed(7), 5, 20));
			assert_eq!(StakePallet::selected_candidates().into_inner(), vec![5, 3]);
			assert_eq!(
				StakePallet::top_candidates(),
				OrderedSet::from_sorted_set(
					vec![
						StakeOf::<Test> { owner: 5, amount: 120 },
						StakeOf::<Test> { owner: 3, amount: 110 },
						StakeOf::<Test> { owner: 6, amount: 100 },
						StakeOf::<Test> { owner: 4, amount: 100 },
						StakeOf::<Test> { owner: 1, amount: 99 },
					]
					.try_into()
					.unwrap()
				)
			);

			// 7 decreases delegation
			assert_ok!(StakePallet::delegator_stake_less(RuntimeOrigin::signed(7), 10));
			assert_eq!(StakePallet::selected_candidates().into_inner(), vec![5, 3]);
			assert_eq!(
				StakePallet::top_candidates(),
				OrderedSet::from_sorted_set(
					vec![
						StakeOf::<Test> { owner: 5, amount: 110 },
						StakeOf::<Test> { owner: 3, amount: 110 },
						StakeOf::<Test> { owner: 6, amount: 100 },
						StakeOf::<Test> { owner: 4, amount: 100 },
						StakeOf::<Test> { owner: 1, amount: 99 },
					]
					.try_into()
					.unwrap()
				)
			);
			assert_ok!(StakePallet::leave_delegators(RuntimeOrigin::signed(7)));
			assert_eq!(StakePallet::selected_candidates().into_inner(), vec![3, 5]);
			assert_eq!(
				StakePallet::top_candidates(),
				OrderedSet::from_sorted_set(
					vec![
						StakeOf::<Test> { owner: 3, amount: 110 },
						StakeOf::<Test> { owner: 5, amount: 100 },
						StakeOf::<Test> { owner: 6, amount: 100 },
						StakeOf::<Test> { owner: 4, amount: 100 },
						StakeOf::<Test> { owner: 1, amount: 99 },
					]
					.try_into()
					.unwrap()
				)
			);
		});
}

#[test]
fn update_total_stake_collators_stay() {
	ExtBuilder::default()
		.with_balances(vec![(1, 200), (2, 200), (3, 200), (4, 200)])
		.with_collators(vec![(1, 100), (2, 50)])
		.with_delegators(vec![(3, 1, 100), (4, 2, 50)])
		.build_and_execute_with_sanity_tests(|| {
			assert_eq!(
				StakePallet::total_collator_stake(),
				TotalStake {
					collators: 150,
					delegators: 150
				}
			);
			assert_ok!(StakePallet::candidate_stake_more(RuntimeOrigin::signed(1), 10));
			assert_eq!(
				StakePallet::total_collator_stake(),
				TotalStake {
					collators: 160,
					delegators: 150
				}
			);
			assert_ok!(StakePallet::candidate_stake_less(RuntimeOrigin::signed(2), 5));
			assert_eq!(
				StakePallet::total_collator_stake(),
				TotalStake {
					collators: 155,
					delegators: 150
				}
			);
			assert_ok!(StakePallet::delegator_stake_more(RuntimeOrigin::signed(3), 10));
			assert_eq!(
				StakePallet::total_collator_stake(),
				TotalStake {
					collators: 155,
					delegators: 160
				}
			);
			assert_ok!(StakePallet::delegator_stake_less(RuntimeOrigin::signed(4), 5));
			assert_eq!(
				StakePallet::total_collator_stake(),
				TotalStake {
					collators: 155,
					delegators: 155
				}
			);
		});
}

#[test]
fn update_total_stake_displace_collators() {
	ExtBuilder::default()
		.with_balances(vec![
			(1, 200),
			(2, 200),
			(3, 200),
			(4, 200),
			(5, 200),
			(6, 200),
			(7, 200),
			(8, 200),
			(1337, 200),
		])
		.with_collators(vec![(1, 10), (2, 20), (3, 30), (4, 40)])
		.with_delegators(vec![(5, 1, 50), (6, 2, 50), (7, 3, 55), (8, 4, 55)])
		.build_and_execute_with_sanity_tests(|| {
			assert_eq!(
				StakePallet::total_collator_stake(),
				TotalStake {
					collators: 70,
					delegators: 110
				}
			);

			// 4 is pushed out by staking less
			assert_ok!(StakePallet::candidate_stake_less(RuntimeOrigin::signed(4), 30)); // vec![(1, 10), (2, 20), (3, 30), (4, 10)]
			assert_eq!(
				StakePallet::total_collator_stake(), // collators: 50, delegators 105
				TotalStake {
					collators: 50,
					delegators: 105
				}
			);
			assert_ok!(StakePallet::delegator_stake_less(RuntimeOrigin::signed(8), 45)); // vec![(5, 1, 50), (6, 2, 50), (7, 3, 55), (8, 4, 10)]

			// 3 is pushed out by delegator staking less
			assert_ok!(StakePallet::delegator_stake_less(RuntimeOrigin::signed(7), 45)); // vec![(5, 1, 50), (6, 2, 50), (7, 3, 10), (8, 4, 10)]
			assert_eq!(
				StakePallet::total_collator_stake(),
				TotalStake {
					collators: 30,
					delegators: 100
				}
			);

			// 1 is pushed out by new candidate
			assert_ok!(StakePallet::join_candidates(RuntimeOrigin::signed(1337), 100));
			assert_eq!(
				StakePallet::total_collator_stake(),
				TotalStake {
					collators: 120,
					delegators: 50
				}
			);
		});
}

#[test]
fn update_total_stake_new_collators() {
	ExtBuilder::default()
		.with_balances(vec![(1, 100), (2, 100), (3, 100), (4, 100)])
		.with_collators(vec![(1, 100)])
		.with_delegators(vec![(4, 1, 100)])
		.build_and_execute_with_sanity_tests(|| {
			assert_eq!(
				StakePallet::total_collator_stake(),
				TotalStake {
					collators: 100,
					delegators: 100
				}
			);
			assert_ok!(StakePallet::join_candidates(RuntimeOrigin::signed(2), 100));
			assert_eq!(
				StakePallet::total_collator_stake(),
				TotalStake {
					collators: 200,
					delegators: 100
				}
			);
			assert_ok!(StakePallet::join_delegators(RuntimeOrigin::signed(3), 2, 50));
			assert_eq!(
				StakePallet::total_collator_stake(),
				TotalStake {
					collators: 200,
					delegators: 150
				}
			);
			assert_ok!(StakePallet::leave_delegators(RuntimeOrigin::signed(4)));
			assert_eq!(
				StakePallet::total_collator_stake(),
				TotalStake {
					collators: 200,
					delegators: 50
				}
			);
		});
}

#[test]
fn update_total_stake_no_collator_changes() {
	ExtBuilder::default()
		.with_balances(vec![
			(1, 200),
			(2, 200),
			(3, 200),
			(4, 200),
			(5, 200),
			(6, 200),
			(7, 200),
			(8, 200),
			(1337, 200),
		])
		.with_collators(vec![(1, 10), (2, 20), (3, 30), (4, 40)])
		.with_delegators(vec![(5, 1, 50), (6, 2, 50), (7, 3, 55), (8, 4, 55)])
		.build_and_execute_with_sanity_tests(|| {
			assert_eq!(
				StakePallet::total_collator_stake(),
				TotalStake {
					collators: 70,
					delegators: 110
				}
			);
			assert_ok!(StakePallet::candidate_stake_more(RuntimeOrigin::signed(1), 10));
			assert_eq!(
				StakePallet::total_collator_stake(),
				TotalStake {
					collators: 70,
					delegators: 110
				}
			);
			assert_ok!(StakePallet::delegator_stake_more(RuntimeOrigin::signed(5), 10));
			assert_eq!(
				StakePallet::total_collator_stake(),
				TotalStake {
					collators: 70,
					delegators: 110
				}
			);
			assert_ok!(StakePallet::candidate_stake_less(RuntimeOrigin::signed(2), 10));
			assert_eq!(
				StakePallet::total_collator_stake(),
				TotalStake {
					collators: 70,
					delegators: 110
				}
			);
			assert_ok!(StakePallet::delegator_stake_less(RuntimeOrigin::signed(6), 10));
			assert_eq!(
				StakePallet::total_collator_stake(),
				TotalStake {
					collators: 70,
					delegators: 110
				}
			);
		});
}

#[test]
fn exit_queue_with_events() {
	ExtBuilder::default()
		.with_balances(vec![
			(1, 1000),
			(2, 1000),
			(3, 1000),
			(4, 1000),
			(5, 1000),
			(6, 1000),
			(7, 33),
			(8, 33),
			(9, 33),
		])
		.with_collators(vec![(1, 100), (2, 90), (3, 80), (4, 70), (5, 60), (6, 50)])
		.with_inflation(100, 15, 40, 10, BLOCKS_PER_ROUND)
		.build_and_execute_with_sanity_tests(|| {
			assert_eq!(CandidatePool::<Test>::count(), 6);
			assert_eq!(StakePallet::selected_candidates().into_inner(), vec![1, 2]);
			assert_ok!(StakePallet::set_max_selected_candidates(RuntimeOrigin::root(), 5));
			assert_eq!(StakePallet::selected_candidates().into_inner(), vec![1, 2, 3, 4, 5]);

			roll_to(8, vec![]);
			// should choose top MaxSelectedCandidates (5), in order
			assert_eq!(StakePallet::selected_candidates().into_inner(), vec![1, 2, 3, 4, 5]);
			let mut expected = vec![Event::MaxSelectedCandidatesSet(2, 5), Event::NewRound(5, 1)];
			assert_eq!(events(), expected);
			assert_ok!(StakePallet::init_leave_candidates(RuntimeOrigin::signed(6)));
			assert_eq!(StakePallet::selected_candidates().into_inner(), vec![1, 2, 3, 4, 5]);
			assert_eq!(last_event(), StakeEvent::CollatorScheduledExit(1, 6, 3));

			roll_to(11, vec![]);
			assert_ok!(StakePallet::init_leave_candidates(RuntimeOrigin::signed(5)));
			assert_eq!(StakePallet::selected_candidates().into_inner(), vec![1, 2, 3, 4]);
			assert_eq!(last_event(), StakeEvent::CollatorScheduledExit(2, 5, 4));

			assert_eq!(CandidatePool::<Test>::count(), 6, "No collators have left yet.");
			roll_to(16, vec![]);
			assert_ok!(StakePallet::execute_leave_candidates(RuntimeOrigin::signed(6), 6));
			assert_ok!(StakePallet::init_leave_candidates(RuntimeOrigin::signed(4)));
			assert_eq!(StakePallet::selected_candidates().into_inner(), vec![1, 2, 3]);
			assert_eq!(last_event(), StakeEvent::CollatorScheduledExit(3, 4, 5));
			assert_noop!(
				StakePallet::init_leave_candidates(RuntimeOrigin::signed(4)),
				Error::<Test>::AlreadyLeaving
			);

			assert_eq!(CandidatePool::<Test>::count(), 5, "Collator #5 left.");
			roll_to(20, vec![]);
			assert_ok!(StakePallet::execute_leave_candidates(RuntimeOrigin::signed(5), 5));
			assert_eq!(StakePallet::selected_candidates().into_inner(), vec![1, 2, 3]);
			assert_eq!(CandidatePool::<Test>::count(), 4, "Two out of six collators left.");

			roll_to(26, vec![]);
			assert_ok!(StakePallet::execute_leave_candidates(RuntimeOrigin::signed(4), 4));
			assert_eq!(CandidatePool::<Test>::count(), 3, "Three out of six collators left.");

			roll_to(30, vec![]);
			let mut new_events = vec![
				Event::LeftTopCandidates(6),
				Event::CollatorScheduledExit(1, 6, 3),
				Event::NewRound(10, 2),
				Event::LeftTopCandidates(5),
				Event::CollatorScheduledExit(2, 5, 4),
				Event::NewRound(15, 3),
				Event::CandidateLeft(6, 50),
				Event::LeftTopCandidates(4),
				Event::CollatorScheduledExit(3, 4, 5),
				Event::NewRound(20, 4),
				Event::CandidateLeft(5, 60),
				Event::NewRound(25, 5),
				Event::CandidateLeft(4, 70),
				Event::NewRound(30, 6),
			];
			expected.append(&mut new_events);
			assert_eq!(events(), expected);
		});
}

#[test]
fn reach_max_top_candidates() {
	ExtBuilder::default()
		.with_balances(vec![
			(1, 11),
			(2, 20),
			(3, 11),
			(4, 11),
			(5, 11),
			(6, 11),
			(7, 11),
			(8, 11),
			(9, 11),
			(10, 11),
			(11, 11),
			(12, 12),
			(13, 13),
		])
		.with_collators(vec![
			(1, 10),
			(2, 20),
			(3, 10),
			(4, 10),
			(5, 10),
			(6, 10),
			(7, 10),
			(8, 10),
			(9, 10),
			(10, 10),
		])
		.build_and_execute_with_sanity_tests(|| {
			assert_eq!(
				StakePallet::top_candidates().len().saturated_into::<u32>(),
				<Test as Config>::MaxTopCandidates::get()
			);
			// should not be possible to join candidate pool, even with more stake
			assert_ok!(StakePallet::join_candidates(RuntimeOrigin::signed(11), 11));
			assert_eq!(
				StakePallet::top_candidates()
					.into_iter()
					.map(|s| s.owner)
					.collect::<Vec<u64>>(),
				vec![2, 11, 1, 3, 4, 5, 6, 7, 8, 9]
			);
			// last come, last one in the list
			assert_ok!(StakePallet::join_candidates(RuntimeOrigin::signed(12), 11));
			assert_eq!(
				StakePallet::top_candidates()
					.into_iter()
					.map(|s| s.owner)
					.collect::<Vec<u64>>(),
				vec![2, 11, 12, 1, 3, 4, 5, 6, 7, 8]
			);
			assert_ok!(StakePallet::candidate_stake_more(RuntimeOrigin::signed(1), 1));
			assert_ok!(StakePallet::candidate_stake_more(RuntimeOrigin::signed(3), 1));
			assert_ok!(StakePallet::candidate_stake_more(RuntimeOrigin::signed(4), 1));
			assert_ok!(StakePallet::candidate_stake_more(RuntimeOrigin::signed(5), 1));
			assert_ok!(StakePallet::candidate_stake_more(RuntimeOrigin::signed(6), 1));
			assert_ok!(StakePallet::candidate_stake_more(RuntimeOrigin::signed(7), 1));
			assert_ok!(StakePallet::candidate_stake_more(RuntimeOrigin::signed(8), 1));
			assert_eq!(
				StakePallet::top_candidates()
					.into_iter()
					.map(|s| s.owner)
					.collect::<Vec<u64>>(),
				vec![2, 11, 12, 1, 3, 4, 5, 6, 7, 8]
			);
		});
}

#[test]
fn set_max_selected_candidates_safe_guards() {
	ExtBuilder::default()
		.with_balances(vec![(1, 10), (2, 100)])
		.with_collators(vec![(1, 10), (2, 10)])
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				StakePallet::set_max_selected_candidates(
					RuntimeOrigin::root(),
					<Test as Config>::MinCollators::get() - 1
				),
				Error::<Test>::CannotSetBelowMin
			);
			assert_noop!(
				StakePallet::set_max_selected_candidates(
					RuntimeOrigin::root(),
					<Test as Config>::MaxTopCandidates::get() + 1
				),
				Error::<Test>::CannotSetAboveMax
			);
			assert_ok!(StakePallet::set_max_selected_candidates(
				RuntimeOrigin::root(),
				<Test as Config>::MinCollators::get() + 1
			));
		});
}
