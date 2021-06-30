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

//! Unit testing

use std::{collections::BTreeMap, iter};

use frame_support::{assert_noop, assert_ok, traits::EstimateNextSessionRotation};
use pallet_balances::{BalanceLock, Error as BalancesError, Reasons};
use pallet_session::{SessionManager, ShouldEndSession};
use sp_runtime::{traits::Zero, Perbill, Permill, Perquintill, SaturatedConversion};

use kilt_primitives::constants::BLOCKS_PER_YEAR;

use crate::{
	mock::{
		almost_equal, events, last_event, roll_to, AccountId, Balance, Balances, BlockNumber, Event as MetaEvent,
		ExtBuilder, Origin, StakePallet, System, Test, BLOCKS_PER_ROUND, DECIMALS,
	},
	set::OrderedSet,
	types::{BalanceOf, Collator, CollatorStatus, DelegationCounter, Delegator, RoundInfo, Stake, TotalStake},
	Config, Error, Event, InflationInfo, RewardRate, StakingInfo, STAKING_ID,
};

#[test]
fn should_select_collators_genesis_session() {
	ExtBuilder::default()
		.with_balances(vec![
			(1, 20),
			(2, 20),
			(3, 20),
			(4, 20),
			(5, 20),
			(6, 20),
			(7, 20),
			(8, 20),
			(9, 20),
			(10, 20),
			(11, 20),
		])
		.with_collators(vec![(1, 20), (2, 20)])
		.build()
		.execute_with(|| {
			assert_eq!(
				StakePallet::new_session(0)
					.expect("first session must return new collators")
					.len(),
				2
			);
			assert_eq!(
				StakePallet::new_session(1)
					.expect("second session must return new collators")
					.len(),
				2
			);
		});
}

#[test]
fn genesis() {
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
		])
		.with_collators(vec![(1, 500), (2, 200)])
		.with_delegators(vec![(3, 1, 100), (4, 1, 100), (5, 2, 100), (6, 2, 100)])
		.build()
		.execute_with(|| {
			assert!(System::events().is_empty());

			// Collators
			assert_eq!(
				StakePallet::total(),
				TotalStake {
					collators: 700,
					delegators: 400
				}
			);
			assert_eq!(
				vec![Stake { owner: 1, amount: 700 }, Stake { owner: 2, amount: 400 }],
				StakePallet::candidate_pool().into_vec()
			);
			// 1
			assert_eq!(Balances::usable_balance(&1), 500);
			assert_eq!(Balances::free_balance(&1), 1000);
			assert!(StakePallet::is_candidate(&1));
			assert_eq!(
				StakePallet::collator_state(&1),
				Some(Collator {
					id: 1,
					stake: 500,
					delegators: OrderedSet::from_sorted_set(vec![
						Stake { owner: 3, amount: 100 },
						Stake { owner: 4, amount: 100 }
					]),
					total: 700,
					state: CollatorStatus::Active,
				})
			);
			// 2
			assert_eq!(Balances::usable_balance(&2), 100);
			assert_eq!(Balances::free_balance(&2), 300);
			assert!(StakePallet::is_candidate(&2));
			assert_eq!(
				StakePallet::collator_state(&2),
				Some(Collator {
					id: 2,
					stake: 200,
					delegators: OrderedSet::from_sorted_set(vec![
						Stake { owner: 5, amount: 100 },
						Stake { owner: 6, amount: 100 }
					]),
					total: 400,
					state: CollatorStatus::Active,
				})
			);
			// Delegators
			assert_eq!(
				StakePallet::total(),
				TotalStake {
					collators: 700,
					delegators: 400,
				}
			);
			for x in 3..7 {
				assert!(StakePallet::is_delegator(&x));
				assert_eq!(Balances::usable_balance(&x), 0);
				assert_eq!(Balances::free_balance(&x), 100);
			}
			// Uninvolved
			for x in 7..10 {
				assert!(!StakePallet::is_delegator(&x));
			}
			assert_eq!(Balances::free_balance(&7), 100);
			assert_eq!(Balances::usable_balance(&7), 100);
			assert_eq!(Balances::free_balance(&8), 9);
			assert_eq!(Balances::usable_balance(&8), 9);
			assert_eq!(Balances::free_balance(&9), 4);
			assert_eq!(Balances::usable_balance(&9), 4);

			// Safety first checks
			assert_eq!(
				StakePallet::max_selected_candidates(),
				<Test as Config>::MinSelectedCandidates::get()
			);
			assert_eq!(
				StakePallet::round(),
				RoundInfo::new(0u32, 0u32.into(), <Test as Config>::DefaultBlocksPerRound::get())
			);
		});
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
		.with_delegators(vec![(6, 1, 10), (7, 1, 10), (8, 2, 10), (9, 2, 10), (10, 1, 10)])
		.build()
		.execute_with(|| {
			assert!(System::events().is_empty());

			// Collators
			assert_eq!(
				StakePallet::total(),
				TotalStake {
					collators: 40,
					delegators: 50
				}
			);
			assert_eq!(
				StakePallet::candidate_pool().into_vec(),
				vec![
					Stake { owner: 1, amount: 50 },
					Stake { owner: 2, amount: 40 },
					Stake { owner: 3, amount: 20 },
					Stake { owner: 4, amount: 20 },
					Stake { owner: 5, amount: 10 }
				]
			);
			for x in 1..5 {
				assert!(StakePallet::is_candidate(&x));
				assert_eq!(Balances::free_balance(&x), 100);
				assert_eq!(Balances::usable_balance(&x), 80);
			}
			assert!(StakePallet::is_candidate(&5));
			assert_eq!(Balances::free_balance(&5), 100);
			assert_eq!(Balances::usable_balance(&5), 90);
			// Delegators
			for x in 6..11 {
				assert!(StakePallet::is_delegator(&x));
				assert_eq!(Balances::free_balance(&x), 100);
				assert_eq!(Balances::usable_balance(&x), 90);
			}

			// Safety first checks
			assert_eq!(
				StakePallet::max_selected_candidates(),
				<Test as Config>::MinSelectedCandidates::get()
			);
			assert_eq!(
				StakePallet::round(),
				RoundInfo::new(0, 0, <Test as Config>::DefaultBlocksPerRound::get())
			);
		});
}

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
		.build()
		.execute_with(|| {
			assert_eq!(
				StakePallet::total(),
				TotalStake {
					collators: 700,
					delegators: 400
				}
			);
			assert_noop!(
				StakePallet::join_candidates(Origin::signed(1), 11u128,),
				Error::<Test>::CandidateExists
			);
			assert_noop!(
				StakePallet::join_delegators(Origin::signed(1), 1, 11u128,),
				Error::<Test>::CandidateExists
			);
			assert_noop!(
				StakePallet::join_candidates(Origin::signed(3), 11u128,),
				Error::<Test>::DelegatorExists
			);
			assert_noop!(
				StakePallet::join_candidates(Origin::signed(7), 9u128,),
				Error::<Test>::ValStakeBelowMin
			);
			assert_noop!(
				StakePallet::join_candidates(Origin::signed(8), 10u128,),
				BalancesError::<Test>::InsufficientBalance
			);
			assert!(System::events().is_empty());
			assert_ok!(StakePallet::set_max_selected_candidates(Origin::root(), 5));
			assert_ok!(StakePallet::join_candidates(Origin::signed(7), 10u128,));
			assert_eq!(
				last_event(),
				MetaEvent::StakePallet(Event::JoinedCollatorCandidates(7, 10u128, 710u128))
			);

			// MaxCollatorCandidateStake
			assert_noop!(
				StakePallet::join_candidates(Origin::signed(10), 161_000_000 * DECIMALS),
				Error::<Test>::ValStakeAboveMax
			);
			assert_ok!(StakePallet::join_candidates(
				Origin::signed(10),
				StakePallet::max_candidate_stake()
			));
			assert_eq!(
				last_event(),
				MetaEvent::StakePallet(Event::JoinedCollatorCandidates(
					10,
					StakePallet::max_candidate_stake(),
					StakePallet::max_candidate_stake() + 710u128
				))
			);
		});
}

#[test]
fn collator_exit_executes_after_delay() {
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
		])
		.with_collators(vec![(1, 500), (2, 200), (7, 100)])
		.with_delegators(vec![(3, 1, 100), (4, 1, 100), (5, 2, 100), (6, 2, 100)])
		.build()
		.execute_with(|| {
			assert_ok!(StakePallet::set_max_selected_candidates(Origin::root(), 5));
			assert_eq!(StakePallet::selected_candidates(), vec![1, 2, 7]);
			roll_to(4, vec![]);
			assert_noop!(
				StakePallet::init_leave_candidates(Origin::signed(3)),
				Error::<Test>::CandidateNotFound
			);

			roll_to(11, vec![]);
			assert_ok!(StakePallet::init_leave_candidates(Origin::signed(2)));
			assert_noop!(
				StakePallet::delegate_another_candidate(Origin::signed(3), 2, 10),
				Error::<Test>::CannotDelegateIfLeaving
			);
			assert_eq!(StakePallet::selected_candidates(), vec![1, 7]);
			assert_eq!(
				last_event(),
				MetaEvent::StakePallet(Event::CollatorScheduledExit(2, 2, 4))
			);
			let info = StakePallet::collator_state(&2).unwrap();
			assert_eq!(info.state, CollatorStatus::Leaving(4));

			roll_to(21, vec![]);
			assert_ok!(StakePallet::execute_leave_candidates(Origin::signed(2), 2));
			// we must exclude leaving collators from rewards while
			// holding them retroactively accountable for previous faults
			// (within the last T::StakeDuration blocks)
			roll_to(25, vec![]);
			let expected = vec![
				Event::CollatorChosen(1, 500, 200),
				Event::CollatorChosen(2, 200, 200),
				Event::CollatorChosen(7, 100, 0),
				Event::MaxSelectedCandidatesSet(2, 5),
				Event::NewRound(5, 1),
				Event::NewRound(10, 2),
				Event::CollatorChosen(1, 500, 200),
				Event::CollatorChosen(7, 100, 0),
				Event::CollatorScheduledExit(2, 2, 4),
				Event::NewRound(15, 3),
				Event::NewRound(20, 4),
				Event::CollatorLeft(2, 400),
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
		.build()
		.execute_with(|| {
			assert_ok!(StakePallet::set_max_selected_candidates(Origin::root(), 5));
			roll_to(8, vec![]);
			// should choose top MaxSelectedCandidates (5), in order
			assert_eq!(StakePallet::selected_candidates(), vec![1, 2, 3, 4, 5]);
			let expected = vec![
				Event::CollatorChosen(1, 100, 0),
				Event::CollatorChosen(2, 90, 0),
				Event::CollatorChosen(3, 80, 0),
				Event::CollatorChosen(4, 70, 0),
				Event::CollatorChosen(5, 60, 0),
				Event::MaxSelectedCandidatesSet(2, 5),
				Event::NewRound(5, 1),
			];
			assert_eq!(events(), expected);
			assert_ok!(StakePallet::init_leave_candidates(Origin::signed(6)));
			assert_eq!(StakePallet::selected_candidates(), vec![1, 2, 3, 4, 5]);
			assert_eq!(
				last_event(),
				MetaEvent::StakePallet(Event::CollatorScheduledExit(1, 6, 3))
			);

			roll_to(15, vec![]);
			assert_ok!(StakePallet::execute_leave_candidates(Origin::signed(6), 6));

			roll_to(21, vec![]);
			assert_ok!(StakePallet::join_candidates(Origin::signed(6), 69u128));
			assert_eq!(StakePallet::selected_candidates(), vec![1, 2, 3, 4, 6]);
			assert_eq!(
				last_event(),
				MetaEvent::StakePallet(Event::JoinedCollatorCandidates(6, 69u128, 409u128))
			);

			roll_to(27, vec![]);
			// should choose top MaxSelectedCandidates (5), in order
			let expected = vec![
				Event::CollatorChosen(1, 100, 0),
				Event::CollatorChosen(2, 90, 0),
				Event::CollatorChosen(3, 80, 0),
				Event::CollatorChosen(4, 70, 0),
				Event::CollatorChosen(5, 60, 0),
				Event::MaxSelectedCandidatesSet(2, 5),
				Event::NewRound(5, 1),
				Event::CollatorChosen(1, 100, 0),
				Event::CollatorChosen(2, 90, 0),
				Event::CollatorChosen(3, 80, 0),
				Event::CollatorChosen(4, 70, 0),
				Event::CollatorChosen(5, 60, 0),
				Event::CollatorScheduledExit(1, 6, 3),
				// TotalStake is updated once candidate 6 left in `execute_delayed_collator_exits`
				Event::NewRound(10, 2),
				Event::NewRound(15, 3),
				Event::CollatorLeft(6, 50),
				Event::NewRound(20, 4),
				Event::CollatorChosen(1, 100, 0),
				Event::CollatorChosen(2, 90, 0),
				Event::CollatorChosen(3, 80, 0),
				Event::CollatorChosen(4, 70, 0),
				Event::CollatorChosen(6, 69, 0),
				Event::JoinedCollatorCandidates(6, 69, 409),
				Event::NewRound(25, 5),
			];
			assert_eq!(events(), expected);
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
		.build()
		.execute_with(|| {
			assert_ok!(StakePallet::set_max_selected_candidates(Origin::root(), 5));
			roll_to(8, vec![]);
			// should choose top MaxSelectedCandidates (5), in order
			assert_eq!(StakePallet::selected_candidates(), vec![1, 2, 3, 4, 5]);
			let mut expected = vec![
				Event::CollatorChosen(1, 100, 0),
				Event::CollatorChosen(2, 90, 0),
				Event::CollatorChosen(3, 80, 0),
				Event::CollatorChosen(4, 70, 0),
				Event::CollatorChosen(5, 60, 0),
				Event::MaxSelectedCandidatesSet(2, 5),
				Event::NewRound(5, 1),
			];
			assert_eq!(events(), expected);
			assert_ok!(StakePallet::init_leave_candidates(Origin::signed(6)));
			assert_eq!(StakePallet::selected_candidates(), vec![1, 2, 3, 4, 5]);
			assert_eq!(
				last_event(),
				MetaEvent::StakePallet(Event::CollatorScheduledExit(1, 6, 3))
			);

			roll_to(11, vec![]);
			assert_ok!(StakePallet::init_leave_candidates(Origin::signed(5)));
			assert_eq!(StakePallet::selected_candidates(), vec![1, 2, 3, 4]);
			assert_eq!(
				last_event(),
				MetaEvent::StakePallet(Event::CollatorScheduledExit(2, 5, 4))
			);

			roll_to(16, vec![]);
			assert_ok!(StakePallet::execute_leave_candidates(Origin::signed(6), 6));
			assert_ok!(StakePallet::init_leave_candidates(Origin::signed(4)));
			assert_eq!(StakePallet::selected_candidates(), vec![1, 2, 3]);
			assert_eq!(
				last_event(),
				MetaEvent::StakePallet(Event::CollatorScheduledExit(3, 4, 5))
			);
			assert_noop!(
				StakePallet::init_leave_candidates(Origin::signed(4)),
				Error::<Test>::AlreadyLeaving
			);
			roll_to(20, vec![]);
			assert_ok!(StakePallet::execute_leave_candidates(Origin::signed(5), 5));

			roll_to(26, vec![]);
			assert_ok!(StakePallet::execute_leave_candidates(Origin::signed(4), 4));
			roll_to(30, vec![]);
			let mut new_events = vec![
				Event::CollatorChosen(1, 100, 0),
				Event::CollatorChosen(2, 90, 0),
				Event::CollatorChosen(3, 80, 0),
				Event::CollatorChosen(4, 70, 0),
				Event::CollatorChosen(5, 60, 0),
				Event::CollatorScheduledExit(1, 6, 3),
				Event::NewRound(10, 2),
				Event::CollatorChosen(1, 100, 0),
				Event::CollatorChosen(2, 90, 0),
				Event::CollatorChosen(3, 80, 0),
				Event::CollatorChosen(4, 70, 0),
				Event::CollatorScheduledExit(2, 5, 4),
				Event::NewRound(15, 3),
				Event::CollatorLeft(6, 50),
				Event::CollatorChosen(1, 100, 0),
				Event::CollatorChosen(2, 90, 0),
				Event::CollatorChosen(3, 80, 0),
				Event::CollatorScheduledExit(3, 4, 5),
				Event::NewRound(20, 4),
				Event::CollatorLeft(5, 60),
				Event::NewRound(25, 5),
				Event::CollatorLeft(4, 70),
				Event::NewRound(30, 6),
			];
			expected.append(&mut new_events);
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
		.build()
		.execute_with(|| {
			assert_ok!(StakePallet::set_max_selected_candidates(Origin::root(), 5));
			roll_to(5, vec![]);
			// should choose top MaxSelectedCandidates (5), in order
			let mut old_stake = StakePallet::total();
			assert_eq!(
				old_stake,
				TotalStake {
					collators: 300,
					delegators: 500,
				}
			);
			assert_eq!(StakePallet::selected_candidates(), vec![2, 1, 10, 9, 8]);
			assert_ok!(StakePallet::init_leave_candidates(Origin::signed(10)));
			assert_ok!(StakePallet::init_leave_candidates(Origin::signed(9)));
			assert_ok!(StakePallet::init_leave_candidates(Origin::signed(1)));
			assert_ok!(StakePallet::init_leave_candidates(Origin::signed(7)));
			assert_ok!(StakePallet::init_leave_candidates(Origin::signed(6)));
			assert_ok!(StakePallet::init_leave_candidates(Origin::signed(5)));
			assert_ok!(StakePallet::init_leave_candidates(Origin::signed(8)));
			assert_ok!(StakePallet::init_leave_candidates(Origin::signed(2)));
			assert_eq!(StakePallet::selected_candidates(), vec![4, 3]);
			for owner in vec![1, 2, 5, 6, 7, 8, 9, 10].iter() {
				assert!(StakePallet::collator_state(owner)
					.unwrap()
					.can_exit(1 + <Test as Config>::ExitQueueDelay::get()));
			}
			old_stake = TotalStake {
				collators: 70,
				delegators: 0,
			};
			assert_eq!(StakePallet::total(), old_stake);
			assert_eq!(
				StakePallet::collator_state(1),
				Some(Collator::<AccountId, Balance> {
					id: 1,
					stake: 10,
					delegators: OrderedSet::from(vec![
						Stake { owner: 11, amount: 110 },
						Stake { owner: 12, amount: 120 }
					]),
					total: 240,
					state: CollatorStatus::Leaving(3)
				})
			);
			assert_eq!(
				StakePallet::collator_state(2),
				Some(Collator::<AccountId, Balance> {
					id: 2,
					stake: 20,
					delegators: OrderedSet::from(vec![
						Stake { owner: 13, amount: 130 },
						Stake { owner: 14, amount: 140 }
					]),
					total: 290,
					state: CollatorStatus::Leaving(3)
				})
			);
			for collator in 5u64..=10u64 {
				assert_eq!(
					StakePallet::collator_state(collator),
					Some(Collator::<AccountId, Balance> {
						id: collator,
						stake: collator as u128 * 10u128,
						delegators: OrderedSet::from(vec![]),
						total: collator as u128 * 10u128,
						state: CollatorStatus::Leaving(3)
					})
				);
				assert!(StakePallet::is_candidate(&collator));
				assert!(StakePallet::unstaking(collator).is_empty());
			}
			assert_eq!(
				StakePallet::delegator_state(11),
				Some(Delegator::<AccountId, Balance> {
					delegations: OrderedSet::from(vec![Stake { owner: 1, amount: 110 }]),
					total: 110
				})
			);
			assert_eq!(
				StakePallet::delegator_state(12),
				Some(Delegator::<AccountId, Balance> {
					delegations: OrderedSet::from(vec![Stake { owner: 1, amount: 120 }]),
					total: 120
				})
			);
			assert_eq!(
				StakePallet::delegator_state(13),
				Some(Delegator::<AccountId, Balance> {
					delegations: OrderedSet::from(vec![Stake { owner: 2, amount: 130 }]),
					total: 130
				})
			);
			assert_eq!(
				StakePallet::delegator_state(14),
				Some(Delegator::<AccountId, Balance> {
					delegations: OrderedSet::from(vec![Stake { owner: 2, amount: 140 }]),
					total: 140
				})
			);
			for delegator in 11u64..=14u64 {
				assert!(StakePallet::is_delegator(&delegator));
				assert!(StakePallet::unstaking(delegator).is_empty());
			}

			// exits cannot be executed yet but in the next round
			roll_to(10, vec![]);
			assert_eq!(StakePallet::total(), old_stake);
			assert_eq!(StakePallet::selected_candidates(), vec![4, 3]);
			for owner in vec![1, 2, 5, 6, 7, 8, 9, 10].iter() {
				assert!(StakePallet::collator_state(owner)
					.unwrap()
					.can_exit(1 + <Test as Config>::ExitQueueDelay::get()));
				assert_noop!(
					StakePallet::execute_leave_candidates(Origin::signed(*owner), *owner),
					Error::<Test>::CannotLeaveYet
				);
			}
			assert_eq!(StakePallet::total(), old_stake);
			assert_eq!(
				StakePallet::collator_state(1),
				Some(Collator::<AccountId, Balance> {
					id: 1,
					stake: 10,
					delegators: OrderedSet::from(vec![
						Stake { owner: 11, amount: 110 },
						Stake { owner: 12, amount: 120 }
					]),
					total: 240,
					state: CollatorStatus::Leaving(3)
				})
			);
			assert_eq!(
				StakePallet::collator_state(2),
				Some(Collator::<AccountId, Balance> {
					id: 2,
					stake: 20,
					delegators: OrderedSet::from(vec![
						Stake { owner: 13, amount: 130 },
						Stake { owner: 14, amount: 140 }
					]),
					total: 290,
					state: CollatorStatus::Leaving(3)
				})
			);
			for collator in 5u64..=10u64 {
				assert_eq!(
					StakePallet::collator_state(collator),
					Some(Collator::<AccountId, Balance> {
						id: collator,
						stake: collator as u128 * 10u128,
						delegators: OrderedSet::from(vec![]),
						total: collator as u128 * 10u128,
						state: CollatorStatus::Leaving(3)
					})
				);
				assert!(StakePallet::is_candidate(&collator));
				assert!(StakePallet::unstaking(collator).is_empty());
			}
			assert_eq!(
				StakePallet::delegator_state(11),
				Some(Delegator::<AccountId, Balance> {
					delegations: OrderedSet::from(vec![Stake { owner: 1, amount: 110 }]),
					total: 110
				})
			);
			assert_eq!(
				StakePallet::delegator_state(12),
				Some(Delegator::<AccountId, Balance> {
					delegations: OrderedSet::from(vec![Stake { owner: 1, amount: 120 }]),
					total: 120
				})
			);
			assert_eq!(
				StakePallet::delegator_state(13),
				Some(Delegator::<AccountId, Balance> {
					delegations: OrderedSet::from(vec![Stake { owner: 2, amount: 130 }]),
					total: 130
				})
			);
			assert_eq!(
				StakePallet::delegator_state(14),
				Some(Delegator::<AccountId, Balance> {
					delegations: OrderedSet::from(vec![Stake { owner: 2, amount: 140 }]),
					total: 140
				})
			);
			for delegator in 11u64..=14u64 {
				assert!(StakePallet::is_delegator(&delegator));
				assert!(StakePallet::unstaking(delegator).is_empty());
			}

			// execute first five exits are executed
			roll_to(15, vec![]);
			assert_eq!(StakePallet::total(), old_stake);
			assert_eq!(StakePallet::selected_candidates(), vec![4, 3]);
			for collator in vec![1u64, 2u64, 5u64, 6u64, 7u64].iter() {
				assert_ok!(StakePallet::execute_leave_candidates(
					Origin::signed(*collator),
					*collator
				));
				assert!(StakePallet::collator_state(&collator).is_none());
				assert!(!StakePallet::is_candidate(collator));
				assert_eq!(StakePallet::unstaking(collator).len(), 1);
			}
			assert_eq!(StakePallet::total(), old_stake);
			for delegator in 11u64..=14u64 {
				assert!(!StakePallet::is_delegator(&delegator));
				assert_eq!(StakePallet::unstaking(delegator).len(), 1);
			}

			// last 3 exits are executed
			roll_to(20, vec![]);
			for collator in 8u64..=10u64 {
				assert_ok!(StakePallet::execute_leave_candidates(
					Origin::signed(collator),
					collator
				));
				assert!(StakePallet::collator_state(&collator).is_none());
				assert!(!StakePallet::is_candidate(&collator));
				assert_eq!(StakePallet::unstaking(collator).len(), 1);
			}
		});
}

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
		])
		.with_collators(vec![(1, 20), (2, 20), (3, 20), (4, 20), (5, 10)])
		.with_delegators(vec![(6, 1, 10), (7, 1, 10), (8, 2, 10), (9, 2, 10), (10, 1, 10)])
		.set_blocks_per_round(5)
		.build()
		.execute_with(|| {
			assert_ok!(StakePallet::set_max_selected_candidates(Origin::root(), 5));
			roll_to(8, vec![]);
			// chooses top MaxSelectedCandidates (5), in order
			assert_eq!(StakePallet::selected_candidates(), vec![1, 2, 3, 4, 5]);
			let mut expected = vec![
				Event::CollatorChosen(1, 20, 30),
				Event::CollatorChosen(2, 20, 20),
				Event::CollatorChosen(3, 20, 0),
				Event::CollatorChosen(4, 20, 0),
				Event::CollatorChosen(5, 10, 0),
				Event::MaxSelectedCandidatesSet(2, 5),
				Event::NewRound(5, 1),
			];
			assert_eq!(events(), expected);
			assert_noop!(
				StakePallet::delegate_another_candidate(Origin::signed(6), 1, 10),
				Error::<Test>::AlreadyDelegatedCollator,
			);
			assert_noop!(
				StakePallet::delegate_another_candidate(Origin::signed(6), 2, 2),
				Error::<Test>::DelegationBelowMin,
			);
			assert_ok!(StakePallet::delegate_another_candidate(Origin::signed(6), 2, 10));
			assert_ok!(StakePallet::delegate_another_candidate(Origin::signed(6), 4, 10));
			assert_ok!(StakePallet::delegate_another_candidate(Origin::signed(6), 3, 10));
			assert_eq!(StakePallet::selected_candidates(), vec![1, 2, 3, 4, 5]);
			assert_noop!(
				StakePallet::delegate_another_candidate(Origin::signed(6), 5, 10),
				Error::<Test>::ExceedMaxCollatorsPerDelegator,
			);

			roll_to(16, vec![]);
			let mut new = vec![
				Event::CollatorChosen(1, 20, 30),
				Event::CollatorChosen(2, 20, 30),
				Event::CollatorChosen(3, 20, 0),
				Event::CollatorChosen(4, 20, 0),
				Event::CollatorChosen(5, 10, 0),
				Event::Delegation(6, 10, 2, 50),
				Event::CollatorChosen(1, 20, 30),
				Event::CollatorChosen(2, 20, 30),
				Event::CollatorChosen(4, 20, 10),
				Event::CollatorChosen(3, 20, 0),
				Event::CollatorChosen(5, 10, 0),
				Event::Delegation(6, 10, 4, 30),
				Event::CollatorChosen(1, 20, 30),
				Event::CollatorChosen(2, 20, 30),
				Event::CollatorChosen(3, 20, 10),
				Event::CollatorChosen(4, 20, 10),
				Event::CollatorChosen(5, 10, 0),
				Event::Delegation(6, 10, 3, 30),
				Event::NewRound(10, 2),
				Event::NewRound(15, 3),
			];
			expected.append(&mut new);
			assert_eq!(events(), expected);

			roll_to(21, vec![]);
			assert_ok!(StakePallet::delegate_another_candidate(Origin::signed(7), 2, 80));
			assert_noop!(
				StakePallet::delegate_another_candidate(Origin::signed(7), 3, 11),
				BalancesError::<Test>::InsufficientBalance
			);
			assert_noop!(
				StakePallet::delegate_another_candidate(Origin::signed(10), 2, 10),
				Error::<Test>::TooManyDelegators
			);
			assert!(StakePallet::unstaking(9).is_empty());
			assert_ok!(StakePallet::delegate_another_candidate(Origin::signed(10), 2, 11));
			assert_eq!(StakePallet::unstaking(9).get(&23), Some(&10u128));
			assert!(StakePallet::delegator_state(9).is_none());
			assert!(!StakePallet::collator_state(2)
				.unwrap()
				.delegators
				.contains(&Stake { owner: 9, amount: 10 }));

			roll_to(26, vec![]);
			let mut new2 = vec![
				Event::NewRound(20, 4),
				Event::CollatorChosen(2, 20, 110),
				Event::CollatorChosen(1, 20, 30),
				Event::CollatorChosen(3, 20, 10),
				Event::CollatorChosen(4, 20, 10),
				Event::CollatorChosen(5, 10, 0),
				Event::Delegation(7, 80, 2, 130),
				Event::DelegationReplaced(10, 11, 9, 10, 2, 131),
				Event::CollatorChosen(2, 20, 111),
				Event::CollatorChosen(1, 20, 30),
				Event::CollatorChosen(3, 20, 10),
				Event::CollatorChosen(4, 20, 10),
				Event::CollatorChosen(5, 10, 0),
				Event::Delegation(10, 11, 2, 131),
				Event::NewRound(25, 5),
			];
			expected.append(&mut new2);
			assert_eq!(events(), expected);
			assert_ok!(StakePallet::init_leave_candidates(Origin::signed(2)));
			assert_eq!(
				last_event(),
				MetaEvent::StakePallet(Event::CollatorScheduledExit(5, 2, 7))
			);

			roll_to(31, vec![]);
			let mut new3 = vec![
				Event::CollatorChosen(1, 20, 30),
				Event::CollatorChosen(3, 20, 10),
				Event::CollatorChosen(4, 20, 10),
				Event::CollatorChosen(5, 10, 0),
				Event::CollatorScheduledExit(5, 2, 7),
				Event::NewRound(30, 6),
			];
			expected.append(&mut new3);
			assert_eq!(events(), expected);

			// test join_delegator errors
			assert_ok!(StakePallet::delegate_another_candidate(Origin::signed(8), 1, 10));
			assert_noop!(
				StakePallet::join_delegators(Origin::signed(11), 1, 10),
				Error::<Test>::TooManyDelegators
			);
			assert_noop!(
				StakePallet::delegate_another_candidate(Origin::signed(11), 1, 10),
				Error::<Test>::NotYetDelegating
			);
			assert_ok!(StakePallet::join_delegators(Origin::signed(11), 1, 11));

			// verify that delegations are removed after collator leaves, not before
			assert_eq!(StakePallet::delegator_state(6).unwrap().total, 40);
			assert_eq!(StakePallet::delegator_state(6).unwrap().delegations.len(), 4usize);
			assert_eq!(StakePallet::delegator_state(7).unwrap().total, 90);
			assert_eq!(StakePallet::delegator_state(7).unwrap().delegations.len(), 2usize);
			assert_eq!(Balances::usable_balance(&6), 60);
			assert_eq!(Balances::usable_balance(&7), 10);
			assert_eq!(Balances::free_balance(&6), 100);
			assert_eq!(Balances::free_balance(&7), 100);

			roll_to(35, vec![]);
			assert_ok!(StakePallet::execute_leave_candidates(Origin::signed(2), 2));
			let mut unbonding_6 = BTreeMap::new();
			unbonding_6.insert(35u64 + <Test as Config>::StakeDuration::get() as u64, 10);
			assert_eq!(StakePallet::unstaking(6), unbonding_6);
			let mut unbonding_7 = BTreeMap::new();
			unbonding_7.insert(35u64 + <Test as Config>::StakeDuration::get() as u64, 80);
			assert_eq!(StakePallet::unstaking(7), unbonding_7);

			roll_to(37, vec![]);
			assert_eq!(StakePallet::delegator_state(6).unwrap().total, 30);
			assert_eq!(StakePallet::delegator_state(7).unwrap().total, 10);
			assert_eq!(StakePallet::delegator_state(6).unwrap().delegations.len(), 3usize);
			assert_eq!(StakePallet::delegator_state(7).unwrap().delegations.len(), 1usize);
			assert_ok!(StakePallet::unlock_unstaked(Origin::signed(6), 6));
			assert_noop!(
				StakePallet::unlock_unstaked(Origin::signed(6), 6),
				Error::<Test>::UnstakingIsEmpty
			);
			assert_ok!(StakePallet::unlock_unstaked(Origin::signed(7), 7));
			assert_eq!(Balances::usable_balance(&6), 70);
			assert_eq!(Balances::usable_balance(&7), 90);
			assert_eq!(Balances::free_balance(&6), 100);
			assert_eq!(Balances::free_balance(&7), 100);
		});
}

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
		.build()
		.execute_with(|| {
			let mut old_stake = StakePallet::total();
			assert_ok!(StakePallet::candidate_stake_more(Origin::signed(1), 50));
			assert_eq!(
				StakePallet::total(),
				TotalStake {
					collators: old_stake.collators + 50,
					..old_stake
				}
			);

			old_stake = StakePallet::total();
			assert_ok!(StakePallet::candidate_stake_less(Origin::signed(1), 50));
			assert_eq!(
				StakePallet::total(),
				TotalStake {
					collators: old_stake.collators - 50,
					..old_stake
				}
			);

			old_stake = StakePallet::total();
			assert_ok!(StakePallet::delegator_stake_more(Origin::signed(7), 1, 50));
			assert_noop!(
				StakePallet::delegator_stake_more(Origin::signed(7), 1, 0),
				Error::<Test>::ValStakeZero
			);
			assert_noop!(
				StakePallet::delegator_stake_less(Origin::signed(7), 1, 0),
				Error::<Test>::ValStakeZero
			);
			assert_eq!(
				StakePallet::total(),
				TotalStake {
					delegators: old_stake.delegators + 50,
					..old_stake
				}
			);

			old_stake = StakePallet::total();
			assert_ok!(StakePallet::delegator_stake_less(Origin::signed(7), 1, 50));
			assert_eq!(
				StakePallet::total(),
				TotalStake {
					delegators: old_stake.delegators - 50,
					..old_stake
				}
			);

			old_stake = StakePallet::total();
			assert_ok!(StakePallet::join_delegators(Origin::signed(11), 1, 200));
			assert_eq!(
				StakePallet::total(),
				TotalStake {
					delegators: old_stake.delegators + 200,
					..old_stake
				}
			);

			old_stake = StakePallet::total();
			assert_ok!(StakePallet::delegate_another_candidate(Origin::signed(11), 2, 150));
			assert_eq!(
				StakePallet::total(),
				TotalStake {
					delegators: old_stake.delegators + 150,
					..old_stake
				}
			);

			old_stake = StakePallet::total();
			assert_ok!(StakePallet::leave_delegators(Origin::signed(11)));
			assert_eq!(
				StakePallet::total(),
				TotalStake {
					delegators: old_stake.delegators - 350,
					..old_stake
				}
			);

			let old_stake = StakePallet::total();
			assert_ok!(StakePallet::revoke_delegation(Origin::signed(8), 2));
			assert_eq!(
				StakePallet::total(),
				TotalStake {
					delegators: old_stake.delegators - 10,
					..old_stake
				}
			);

			// should immediately affect total stake because collator can't be chosen in
			// active set from now on, thus delegated stake is reduced
			let old_stake = StakePallet::total();
			assert_ok!(StakePallet::init_leave_candidates(Origin::signed(2)));
			let old_stake = TotalStake {
				delegators: old_stake.delegators - 10,
				..old_stake
			};
			assert_eq!(StakePallet::total(), old_stake);

			// shouldn't
			roll_to(10, vec![]);
			assert_eq!(StakePallet::total(), old_stake);
			assert_ok!(StakePallet::execute_leave_candidates(Origin::signed(2), 2));
			assert_eq!(StakePallet::total(), old_stake);
		})
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
		.build()
		.execute_with(|| {
			roll_to(4, vec![]);
			assert_noop!(
				StakePallet::candidate_stake_more(Origin::signed(6), 50),
				Error::<Test>::CandidateNotFound
			);
			assert_noop!(
				StakePallet::candidate_stake_less(Origin::signed(6), 50),
				Error::<Test>::CandidateNotFound
			);
			assert_noop!(
				StakePallet::delegate_another_candidate(Origin::signed(6), 50, 10),
				Error::<Test>::CandidateNotFound
			);
			assert_ok!(StakePallet::candidate_stake_more(Origin::signed(1), 50));
			assert_noop!(
				StakePallet::candidate_stake_more(Origin::signed(1), 40),
				BalancesError::<Test>::InsufficientBalance
			);
			assert_ok!(StakePallet::init_leave_candidates(Origin::signed(1)));
			assert!(StakePallet::collator_state(1)
				.unwrap()
				.can_exit(<Test as Config>::ExitQueueDelay::get()));

			assert_noop!(
				StakePallet::candidate_stake_more(Origin::signed(1), 30),
				Error::<Test>::CannotStakeIfLeaving
			);
			assert_noop!(
				StakePallet::candidate_stake_less(Origin::signed(1), 10),
				Error::<Test>::CannotStakeIfLeaving
			);

			roll_to(30, vec![]);
			assert_ok!(StakePallet::execute_leave_candidates(Origin::signed(1), 1));
			assert_noop!(
				StakePallet::candidate_stake_more(Origin::signed(1), 40),
				Error::<Test>::CandidateNotFound
			);
			assert_ok!(StakePallet::candidate_stake_more(Origin::signed(2), 80));
			assert_ok!(StakePallet::candidate_stake_less(Origin::signed(2), 90));
			assert_ok!(StakePallet::candidate_stake_less(Origin::signed(3), 10));
			assert_noop!(
				StakePallet::candidate_stake_less(Origin::signed(2), 11),
				Error::<Test>::Underflow
			);
			assert_noop!(
				StakePallet::candidate_stake_less(Origin::signed(2), 1),
				Error::<Test>::ValStakeBelowMin
			);
			assert_noop!(
				StakePallet::candidate_stake_less(Origin::signed(3), 1),
				Error::<Test>::ValStakeBelowMin
			);
			assert_noop!(
				StakePallet::candidate_stake_less(Origin::signed(4), 11),
				Error::<Test>::ValStakeBelowMin
			);
			assert_ok!(StakePallet::candidate_stake_less(Origin::signed(4), 10));

			// MaxCollatorCandidateStake
			assert_ok!(StakePallet::join_candidates(
				Origin::signed(11),
				StakePallet::max_candidate_stake()
			));
			assert_noop!(
				StakePallet::candidate_stake_more(Origin::signed(11), 1u128),
				Error::<Test>::ValStakeAboveMax,
			);
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
		.build()
		.execute_with(|| {
			roll_to(4, vec![]);
			assert_noop!(
				StakePallet::join_delegators(Origin::signed(6), 2, 50),
				Error::<Test>::AlreadyDelegating
			);
			assert_noop!(
				StakePallet::delegator_stake_more(Origin::signed(1), 2, 50),
				Error::<Test>::DelegatorNotFound
			);
			assert_noop!(
				StakePallet::delegator_stake_less(Origin::signed(1), 2, 50),
				Error::<Test>::DelegatorNotFound
			);
			assert_noop!(
				StakePallet::delegator_stake_more(Origin::signed(6), 2, 50),
				Error::<Test>::DelegationNotFound
			);
			assert_noop!(
				StakePallet::delegator_stake_more(Origin::signed(7), 6, 50),
				Error::<Test>::CandidateNotFound
			);
			assert_noop!(
				StakePallet::delegator_stake_less(Origin::signed(7), 6, 50),
				Error::<Test>::CandidateNotFound
			);
			assert_noop!(
				StakePallet::delegator_stake_less(Origin::signed(6), 1, 11),
				Error::<Test>::Underflow
			);
			assert_noop!(
				StakePallet::delegator_stake_less(Origin::signed(6), 1, 8),
				Error::<Test>::DelegationBelowMin
			);
			assert_noop!(
				StakePallet::delegator_stake_less(Origin::signed(6), 1, 6),
				Error::<Test>::NomStakeBelowMin
			);
			assert_ok!(StakePallet::delegator_stake_more(Origin::signed(6), 1, 10));
			assert_noop!(
				StakePallet::delegator_stake_less(Origin::signed(6), 2, 5),
				Error::<Test>::DelegationNotFound
			);
			assert_noop!(
				StakePallet::delegator_stake_more(Origin::signed(6), 1, 81),
				BalancesError::<Test>::InsufficientBalance
			);
			assert_noop!(
				StakePallet::join_delegators(Origin::signed(10), 1, 4),
				Error::<Test>::NomStakeBelowMin
			);

			roll_to(9, vec![]);
			assert_eq!(Balances::usable_balance(&6), 80);
			assert_ok!(StakePallet::init_leave_candidates(Origin::signed(1)));
			assert!(StakePallet::collator_state(1)
				.unwrap()
				.can_exit(1 + <Test as Config>::ExitQueueDelay::get()));

			roll_to(31, vec![]);
			assert!(StakePallet::is_delegator(&6));
			assert_ok!(StakePallet::execute_leave_candidates(Origin::signed(1), 1));
			assert!(!StakePallet::is_delegator(&6));
			assert_eq!(Balances::usable_balance(&6), 80);
			assert_eq!(Balances::free_balance(&6), 100);
		});
}

#[test]
fn revoke_delegation_or_leave_delegators() {
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
		.with_delegators(vec![(6, 1, 10), (7, 1, 10), (8, 2, 10), (9, 2, 10), (10, 1, 10)])
		.set_blocks_per_round(5)
		.build()
		.execute_with(|| {
			roll_to(4, vec![]);
			assert_noop!(
				StakePallet::revoke_delegation(Origin::signed(1), 2),
				Error::<Test>::DelegatorNotFound
			);
			assert_noop!(
				StakePallet::revoke_delegation(Origin::signed(6), 2),
				Error::<Test>::DelegationNotFound
			);
			assert_noop!(
				StakePallet::leave_delegators(Origin::signed(1)),
				Error::<Test>::DelegatorNotFound
			);
			assert_ok!(StakePallet::delegate_another_candidate(Origin::signed(6), 2, 3));
			assert_ok!(StakePallet::delegate_another_candidate(Origin::signed(6), 3, 3));
			assert_ok!(StakePallet::revoke_delegation(Origin::signed(6), 1));
			// cannot revoke delegation because would leave remaining total below
			// MinDelegatorStake
			assert_noop!(
				StakePallet::revoke_delegation(Origin::signed(6), 2),
				Error::<Test>::NomStakeBelowMin
			);
			assert_noop!(
				StakePallet::revoke_delegation(Origin::signed(6), 3),
				Error::<Test>::NomStakeBelowMin
			);
			// can revoke both remaining by calling leave delegators
			assert_ok!(StakePallet::leave_delegators(Origin::signed(6)));
			// this leads to 8 leaving set of delegators
			assert_ok!(StakePallet::revoke_delegation(Origin::signed(8), 2));
		});
}

#[test]
fn round_transitions() {
	let col_max = 10;
	let col_rewards = 15;
	let d_max = 40;
	let d_rewards = 10;
	let inflation = InflationInfo::new(
		Perquintill::from_percent(col_max),
		Perquintill::from_percent(col_rewards),
		Perquintill::from_percent(d_max),
		Perquintill::from_percent(d_rewards),
	);

	// round_immediately_jumps_if_current_duration_exceeds_new_blocks_per_round
	// change from 5 bpr to 3 in block 5 -> 8 should be new round
	ExtBuilder::default()
		.with_balances(vec![(1, 100), (2, 100), (3, 100), (4, 100), (5, 100), (6, 100)])
		.with_collators(vec![(1, 20)])
		.with_delegators(vec![(2, 1, 10), (3, 1, 10)])
		.with_inflation(col_max, col_rewards, d_max, d_rewards, 5)
		.build()
		.execute_with(|| {
			assert_eq!(inflation, StakePallet::inflation_config());
			roll_to(5, vec![]);
			let init = vec![Event::NewRound(5, 1)];
			assert_eq!(events(), init);
			assert_ok!(StakePallet::set_blocks_per_round(Origin::root(), 3));
			assert_noop!(
				StakePallet::set_blocks_per_round(Origin::root(), 1),
				Error::<Test>::CannotSetBelowMin
			);
			assert_eq!(
				last_event(),
				MetaEvent::StakePallet(Event::BlocksPerRoundSet(1, 5, 5, 3))
			);

			// inflation config should be untouched after per_block update
			assert_eq!(inflation, StakePallet::inflation_config());

			// last round startet at 5 but we are already at 9, so we expect 9 to be the new
			// round
			roll_to(8, vec![]);
			assert_eq!(last_event(), MetaEvent::StakePallet(Event::NewRound(8, 2)));
		});

	// if duration of current round is less than new bpr, round waits until new bpr
	// passes
	// change from 5 bpr to 3 in block 6 -> 8 should be new round
	ExtBuilder::default()
		.with_balances(vec![(1, 100), (2, 100), (3, 100), (4, 100), (5, 100), (6, 100)])
		.with_collators(vec![(1, 20)])
		.with_delegators(vec![(2, 1, 10), (3, 1, 10)])
		.with_inflation(col_max, col_rewards, d_max, d_rewards, 5)
		.build()
		.execute_with(|| {
			assert_eq!(inflation, StakePallet::inflation_config());
			// Default round every 5 blocks, but MinBlocksPerRound is 3 and we set it to min
			// 3 blocks
			roll_to(6, vec![]);
			// chooses top MaxSelectedCandidates (5), in order
			let init = vec![Event::NewRound(5, 1)];
			assert_eq!(events(), init);
			assert_ok!(StakePallet::set_blocks_per_round(Origin::root(), 3));
			assert_eq!(
				last_event(),
				MetaEvent::StakePallet(Event::BlocksPerRoundSet(1, 5, 5, 3))
			);

			// inflation config should be untouched after per_block update
			assert_eq!(inflation, StakePallet::inflation_config());

			// there should not be a new event
			roll_to(7, vec![]);
			assert_eq!(
				last_event(),
				MetaEvent::StakePallet(Event::BlocksPerRoundSet(1, 5, 5, 3))
			);

			roll_to(8, vec![]);
			assert_eq!(last_event(), MetaEvent::StakePallet(Event::NewRound(8, 2)));
		});

	// round_immediately_jumps_if_current_duration_exceeds_new_blocks_per_round
	// change from 5 bpr (blocks_per_round) to 3 in block 7 -> 8 should be new round
	ExtBuilder::default()
		.with_balances(vec![(1, 100), (2, 100), (3, 100), (4, 100), (5, 100), (6, 100)])
		.with_collators(vec![(1, 20)])
		.with_delegators(vec![(2, 1, 10), (3, 1, 10)])
		.with_inflation(col_max, col_rewards, d_max, d_rewards, 5)
		.build()
		.execute_with(|| {
			// Default round every 5 blocks, but MinBlocksPerRound is 3 and we set it to min
			// 3 blocks
			assert_eq!(inflation, StakePallet::inflation_config());
			roll_to(7, vec![]);
			// chooses top MaxSelectedCandidates (5), in order
			let init = vec![Event::NewRound(5, 1)];
			assert_eq!(events(), init);
			assert_ok!(StakePallet::set_blocks_per_round(Origin::root(), 3));

			// inflation config should be untouched after per_block update
			assert_eq!(inflation, StakePallet::inflation_config());

			assert_eq!(
				StakePallet::inflation_config(),
				InflationInfo::new(
					Perquintill::from_percent(col_max),
					Perquintill::from_percent(col_rewards),
					Perquintill::from_percent(d_max),
					Perquintill::from_percent(d_rewards)
				)
			);
			assert_eq!(
				last_event(),
				MetaEvent::StakePallet(Event::BlocksPerRoundSet(1, 5, 5, 3))
			);
			roll_to(8, vec![]);

			// last round startet at 5, so we expect 8 to be the new round
			assert_eq!(last_event(), MetaEvent::StakePallet(Event::NewRound(8, 2)));
		});
}

#[test]
fn coinbase_rewards_few_blocks_detailed_check() {
	ExtBuilder::default()
		.with_balances(vec![
			(1, 40_000_000 * DECIMALS),
			(2, 40_000_000 * DECIMALS),
			(3, 40_000_000 * DECIMALS),
			(4, 20_000_000 * DECIMALS),
			(5, 20_000_000 * DECIMALS),
		])
		.with_collators(vec![(1, 8_000_000 * DECIMALS), (2, 8_000_000 * DECIMALS)])
		.with_delegators(vec![
			(3, 1, 32_000_000 * DECIMALS),
			(4, 1, 16_000_000 * DECIMALS),
			(5, 2, 16_000_000 * DECIMALS),
		])
		.with_inflation(10, 15, 40, 15, 5)
		.build()
		.execute_with(|| {
			let inflation = StakePallet::inflation_config();
			let total_issuance = <Test as Config>::Currency::total_issuance();
			assert_eq!(total_issuance, 160_000_000 * DECIMALS);

			// compute rewards
			let c_staking_rate = Perquintill::from_rational(16_000_000 * DECIMALS, total_issuance);
			let c_rewards: BalanceOf<Test> =
				inflation
					.collator
					.compute_reward::<Test>(16_000_000 * DECIMALS, c_staking_rate, 1u128);
			let d_staking_rate = Perquintill::from_rational(64_000_000 * DECIMALS, total_issuance);
			let d_rewards: BalanceOf<Test> =
				inflation
					.delegator
					.compute_reward::<Test>(64_000_000 * DECIMALS, d_staking_rate, 2u128);

			// set 1 to be author for blocks 1-3, then 2 for blocks 4-5
			let authors: Vec<Option<AccountId>> =
				vec![None, Some(1u64), Some(1u64), Some(1u64), Some(2u64), Some(2u64)];
			// let d_rewards: Balance = 3 * 2469135802453333 / 2;
			let user_1 = Balances::usable_balance(&1);
			let user_2 = Balances::usable_balance(&2);
			let user_3 = Balances::usable_balance(&3);
			let user_4 = Balances::usable_balance(&4);
			let user_5 = Balances::usable_balance(&5);

			assert_eq!(Balances::usable_balance(&1), user_1);
			assert_eq!(Balances::usable_balance(&2), user_2);
			assert_eq!(Balances::usable_balance(&3), user_3);
			assert_eq!(Balances::usable_balance(&4), user_4);
			assert_eq!(Balances::usable_balance(&5), user_5);

			// 1 is block author for 1st block
			roll_to(2, authors.clone());
			assert_eq!(Balances::usable_balance(&1), user_1 + c_rewards);
			assert_eq!(Balances::usable_balance(&2), user_2);
			assert_eq!(Balances::usable_balance(&3), user_3 + d_rewards / 2);
			assert_eq!(Balances::usable_balance(&4), user_4 + d_rewards / 4);
			assert_eq!(Balances::usable_balance(&5), user_5);

			// 1 is block author for 2nd block
			roll_to(3, authors.clone());
			assert_eq!(Balances::usable_balance(&1), user_1 + 2 * c_rewards);
			assert_eq!(Balances::usable_balance(&2), user_2);
			assert_eq!(Balances::usable_balance(&3), user_3 + d_rewards);
			assert_eq!(Balances::usable_balance(&4), user_4 + d_rewards / 2);
			assert_eq!(Balances::usable_balance(&5), user_5);

			// 1 is block author for 3rd block
			roll_to(4, authors.clone());
			assert_eq!(Balances::usable_balance(&1), user_1 + 3 * c_rewards);
			assert_eq!(Balances::usable_balance(&2), user_2);
			assert_eq!(Balances::usable_balance(&3), user_3 + d_rewards / 2 * 3);
			assert_eq!(Balances::usable_balance(&4), user_4 + d_rewards / 4 * 3);
			assert_eq!(Balances::usable_balance(&5), user_5);

			// 2 is block author for 4th block
			roll_to(5, authors.clone());
			assert_eq!(Balances::usable_balance(&1), user_1 + 3 * c_rewards);
			assert_eq!(Balances::usable_balance(&2), user_2 + c_rewards);
			assert_eq!(Balances::usable_balance(&3), user_3 + d_rewards / 2 * 3);
			assert_eq!(Balances::usable_balance(&4), user_4 + d_rewards / 4 * 3);
			assert_eq!(Balances::usable_balance(&5), user_5 + d_rewards / 4);
			assert_ok!(StakePallet::revoke_delegation(Origin::signed(5), 2));

			// 2 is block author for 5th block
			roll_to(6, authors);
			assert_eq!(Balances::usable_balance(&1), user_1 + 3 * c_rewards);
			assert_eq!(Balances::usable_balance(&2), user_2 + 2 * c_rewards);
			assert_eq!(Balances::usable_balance(&3), user_3 + d_rewards / 2 * 3);
			assert_eq!(Balances::usable_balance(&4), user_4 + d_rewards / 4 * 3);
			// should not receive rewards due to revoked delegation
			assert_eq!(Balances::usable_balance(&5), user_5 + d_rewards / 4);
		});
}

#[test]
fn delegator_should_not_receive_rewards_after_revoking() {
	// test edge case of 1 delegator
	ExtBuilder::default()
		.with_balances(vec![(1, 10_000_000 * DECIMALS), (2, 10_000_000 * DECIMALS)])
		.with_collators(vec![(1, 10_000_000 * DECIMALS)])
		.with_delegators(vec![(2, 1, 10_000_000 * DECIMALS)])
		.with_inflation(10, 15, 40, 15, 5)
		.build()
		.execute_with(|| {
			assert_ok!(StakePallet::revoke_delegation(Origin::signed(2), 1));
			let authors: Vec<Option<AccountId>> = (1u64..100u64).map(|_| Some(1u64)).collect();
			assert_eq!(Balances::usable_balance(&1), Balance::zero());
			assert_eq!(Balances::usable_balance(&2), Balance::zero());
			roll_to(100, authors);
			assert!(Balances::usable_balance(&1) > Balance::zero());
			assert_ok!(StakePallet::unlock_unstaked(Origin::signed(2), 2));
			assert_eq!(Balances::usable_balance(&2), 10_000_000 * DECIMALS);
		});

	ExtBuilder::default()
		.with_balances(vec![
			(1, 10_000_000 * DECIMALS),
			(2, 10_000_000 * DECIMALS),
			(3, 10_000_000 * DECIMALS),
		])
		.with_collators(vec![(1, 10_000_000 * DECIMALS)])
		.with_delegators(vec![(2, 1, 10_000_000 * DECIMALS), (3, 1, 10_000_000 * DECIMALS)])
		.with_inflation(10, 15, 40, 15, 5)
		.build()
		.execute_with(|| {
			assert_ok!(StakePallet::revoke_delegation(Origin::signed(3), 1));
			let authors: Vec<Option<AccountId>> = (1u64..100u64).map(|_| Some(1u64)).collect();
			assert_eq!(Balances::usable_balance(&1), Balance::zero());
			assert_eq!(Balances::usable_balance(&2), Balance::zero());
			assert_eq!(Balances::usable_balance(&3), Balance::zero());
			roll_to(100, authors);
			assert!(Balances::usable_balance(&1) > Balance::zero());
			assert!(Balances::usable_balance(&2) > Balance::zero());
			assert_ok!(StakePallet::unlock_unstaked(Origin::signed(3), 3));
			assert_eq!(Balances::usable_balance(&3), 10_000_000 * DECIMALS);
		});
}
#[test]
fn coinbase_rewards_many_blocks_simple_check() {
	let num_of_years: Perquintill = Perquintill::from_perthousand(2);
	ExtBuilder::default()
		.with_balances(vec![
			(1, 40_000_000 * DECIMALS),
			(2, 40_000_000 * DECIMALS),
			(3, 40_000_000 * DECIMALS),
			(4, 20_000_000 * DECIMALS),
			(5, 20_000_000 * DECIMALS),
		])
		.with_collators(vec![(1, 8_000_000 * DECIMALS), (2, 8_000_000 * DECIMALS)])
		.with_delegators(vec![
			(3, 1, 32_000_000 * DECIMALS),
			(4, 1, 16_000_000 * DECIMALS),
			(5, 2, 16_000_000 * DECIMALS),
		])
		.with_inflation(10, 15, 40, 15, 5)
		.build()
		.execute_with(|| {
			let inflation = StakePallet::inflation_config();
			let total_issuance = <Test as Config>::Currency::total_issuance();
			assert_eq!(total_issuance, 160_000_000 * DECIMALS);
			let end_block: BlockNumber = num_of_years * BLOCKS_PER_YEAR as BlockNumber;
			// set round robin authoring
			let authors: Vec<Option<AccountId>> = (0u64..=end_block).map(|i| Some(i % 2 + 1)).collect();
			roll_to(end_block, authors);

			let rewards_1 = Balances::free_balance(&1).saturating_sub(40_000_000 * DECIMALS);
			let rewards_2 = Balances::free_balance(&2).saturating_sub(40_000_000 * DECIMALS);
			let rewards_3 = Balances::free_balance(&3).saturating_sub(40_000_000 * DECIMALS);
			let rewards_4 = Balances::free_balance(&4).saturating_sub(20_000_000 * DECIMALS);
			let rewards_5 = Balances::free_balance(&5).saturating_sub(20_000_000 * DECIMALS);
			let expected_collator_rewards =
				num_of_years * inflation.collator.reward_rate.annual * 16_000_000 * DECIMALS;
			let expected_delegator_rewards =
				num_of_years * inflation.delegator.reward_rate.annual * 64_000_000 * DECIMALS;

			// 1200000000000000000000
			// 2399074074058720000

			// collator rewards should be about the same
			assert!(almost_equal(rewards_1, rewards_2, Perbill::from_perthousand(1)));
			assert!(
				almost_equal(
					rewards_1,
					num_of_years * inflation.collator.reward_rate.annual * 8_000_000 * DECIMALS,
					Perbill::from_perthousand(1)
				),
				"left {:?}, right {:?}",
				rewards_1,
				inflation.collator.reward_rate.annual * 8_000_000 * DECIMALS,
			);

			// delegator rewards should be about the same
			assert!(
				almost_equal(rewards_3, rewards_4 + rewards_5, Perbill::from_perthousand(1)),
				"left {:?}, right {:?}",
				rewards_3,
				rewards_4 + rewards_5
			);
			assert!(almost_equal(
				rewards_3,
				num_of_years * inflation.delegator.reward_rate.annual * 32_000_000 * DECIMALS,
				Perbill::from_perthousand(1)
			));

			// check rewards in total
			assert!(
				almost_equal(
					rewards_1 + rewards_2,
					expected_collator_rewards,
					Perbill::from_perthousand(1),
				),
				"left {:?}, right {:?}",
				rewards_1 + rewards_2,
				expected_collator_rewards,
			);
			assert!(
				almost_equal(
					rewards_3 + rewards_4 + rewards_5,
					expected_delegator_rewards,
					Perbill::from_perthousand(1),
				),
				"left {:?}, right {:?}",
				rewards_3 + rewards_4 + rewards_5,
				expected_delegator_rewards,
			);

			// old issuance + rewards should equal new issuance
			assert!(
				almost_equal(
					total_issuance + expected_collator_rewards + expected_delegator_rewards,
					<Test as Config>::Currency::total_issuance(),
					Perbill::from_perthousand(1),
				),
				"left {:?}, right {:?}",
				total_issuance + expected_collator_rewards + expected_delegator_rewards,
				<Test as Config>::Currency::total_issuance(),
			);
		});
}

// Could only occur if we increase MinDelegatorStake via runtime upgrade and
// don't migrate delegators which fall below minimum
#[test]
fn should_not_reward_delegators_below_min_stake() {
	ExtBuilder::default()
		.with_balances(vec![(1, 10 * DECIMALS), (2, 10 * DECIMALS), (3, 10 * DECIMALS), (4, 5)])
		.with_collators(vec![(1, 10 * DECIMALS), (2, 10 * DECIMALS)])
		.with_delegators(vec![(3, 2, 10 * DECIMALS)])
		.with_inflation(10, 15, 40, 15, 5)
		.build()
		.execute_with(|| {
			// impossible but lets assume it happened
			let mut state = StakePallet::collator_state(&1).expect("CollatorState cannot be missing");
			let delegator_stake_below_min = <Test as Config>::MinDelegatorStake::get() - 1;
			state.stake += delegator_stake_below_min;
			state.total += delegator_stake_below_min;
			let impossible_bond = Stake {
				owner: 4u64,
				amount: delegator_stake_below_min,
			};
			state.delegators.insert(impossible_bond);
			<crate::CollatorState<Test>>::insert(&1u64, state);

			let authors: Vec<Option<AccountId>> = vec![Some(1u64), Some(1u64), Some(1u64), Some(1u64)];
			assert_eq!(Balances::usable_balance(&1), Balance::zero());
			assert_eq!(Balances::usable_balance(&2), Balance::zero());
			assert_eq!(Balances::usable_balance(&3), Balance::zero());
			assert_eq!(Balances::usable_balance(&4), 5);

			// should only reward 1
			roll_to(4, authors);
			assert!(Balances::usable_balance(&1) > Balance::zero());
			assert_eq!(Balances::usable_balance(&4), 5);
			assert_eq!(Balances::usable_balance(&2), Balance::zero());
			assert_eq!(Balances::usable_balance(&3), Balance::zero());
		});
}

#[test]
#[should_panic]
fn should_deny_low_delegator_stake() {
	ExtBuilder::default()
		.with_balances(vec![(1, 10 * DECIMALS), (2, 10 * DECIMALS), (3, 10 * DECIMALS), (4, 1)])
		.with_collators(vec![(1, 10 * DECIMALS), (2, 10 * DECIMALS)])
		.with_delegators(vec![(4, 2, 1)])
		.build()
		.execute_with(|| {});
}

#[test]
#[should_panic]
fn should_deny_low_collator_stake() {
	ExtBuilder::default()
		.with_balances(vec![(1, 10 * DECIMALS), (2, 5)])
		.with_collators(vec![(1, 10 * DECIMALS), (2, 5)])
		.build()
		.execute_with(|| {});
}

#[test]
#[should_panic]
fn should_deny_duplicate_collators() {
	ExtBuilder::default()
		.with_balances(vec![(1, 10 * DECIMALS)])
		.with_collators(vec![(1, 10 * DECIMALS), (1, 10 * DECIMALS)])
		.build()
		.execute_with(|| {});
}

#[test]
fn reach_max_collator_candidates() {
	ExtBuilder::default()
		.with_balances(vec![
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
			(11, 10),
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
		.build()
		.execute_with(|| {
			assert_eq!(
				StakePallet::candidate_pool().len().saturated_into::<u32>(),
				<Test as Config>::MaxCollatorCandidates::get()
			);
			assert_noop!(
				StakePallet::join_candidates(Origin::signed(11), 10),
				Error::<Test>::TooManyCollatorCandidates
			);
		});
}

#[test]
fn should_estimate_current_session_progress() {
	ExtBuilder::default()
		.set_blocks_per_round(100)
		.with_balances(vec![
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
			(11, 10),
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
		.build()
		.execute_with(|| {
			assert_eq!(
				StakePallet::estimate_current_session_progress(10).0.unwrap(),
				Permill::from_percent(10)
			);
			assert_eq!(
				StakePallet::estimate_current_session_progress(20).0.unwrap(),
				Permill::from_percent(20)
			);
			assert_eq!(
				StakePallet::estimate_current_session_progress(30).0.unwrap(),
				Permill::from_percent(30)
			);
			assert_eq!(
				StakePallet::estimate_current_session_progress(60).0.unwrap(),
				Permill::from_percent(60)
			);
			assert_eq!(
				StakePallet::estimate_current_session_progress(100).0.unwrap(),
				Permill::from_percent(100)
			);
		});
}

#[test]
fn should_estimate_next_session_rotation() {
	ExtBuilder::default()
		.set_blocks_per_round(100)
		.with_balances(vec![
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
			(11, 10),
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
		.build()
		.execute_with(|| {
			assert_eq!(StakePallet::estimate_next_session_rotation(10).0.unwrap(), 100);
			assert_eq!(StakePallet::estimate_next_session_rotation(20).0.unwrap(), 100);
			assert_eq!(StakePallet::estimate_next_session_rotation(30).0.unwrap(), 100);
			assert_eq!(StakePallet::estimate_next_session_rotation(60).0.unwrap(), 100);
			assert_eq!(StakePallet::estimate_next_session_rotation(100).0.unwrap(), 100);
		});
}

#[test]
fn should_end_session_when_appropriate() {
	ExtBuilder::default()
		.set_blocks_per_round(100)
		.with_balances(vec![
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
			(11, 10),
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
		.build()
		.execute_with(|| {
			assert!(!StakePallet::should_end_session(10));
			assert!(!StakePallet::should_end_session(20));
			assert!(!StakePallet::should_end_session(30));
			assert!(!StakePallet::should_end_session(60));
			assert!(StakePallet::should_end_session(100));
		});
}

#[test]
fn set_max_selected_candidates() {
	ExtBuilder::default()
		.with_balances(vec![(1, 10)])
		.with_collators(vec![(1, 10)])
		.build()
		.execute_with(|| {
			assert_noop!(
				StakePallet::set_max_selected_candidates(
					Origin::root(),
					<Test as Config>::MinSelectedCandidates::get() - 1
				),
				Error::<Test>::CannotSetBelowMin
			);
			assert_ok!(StakePallet::set_max_selected_candidates(
				Origin::root(),
				<Test as Config>::MinSelectedCandidates::get() + 1
			));
		});
}

#[test]
fn update_inflation() {
	ExtBuilder::default()
		.with_balances(vec![(1, 10)])
		.with_collators(vec![(1, 10)])
		.build()
		.execute_with(|| {
			let mut invalid_inflation = InflationInfo {
				collator: StakingInfo {
					max_rate: Perquintill::one(),
					reward_rate: RewardRate {
						annual: Perquintill::from_percent(99),
						per_block: Perquintill::from_percent(1),
					},
				},
				delegator: StakingInfo {
					max_rate: Perquintill::one(),
					reward_rate: RewardRate {
						annual: Perquintill::from_percent(99),
						per_block: Perquintill::from_percent(1),
					},
				},
			};
			assert!(!invalid_inflation.is_valid());
			invalid_inflation.collator.reward_rate.per_block = Perquintill::zero();
			assert!(!invalid_inflation.is_valid());

			assert_ok!(StakePallet::set_inflation(
				Origin::root(),
				Perquintill::from_percent(0),
				Perquintill::from_percent(100),
				Perquintill::from_percent(100),
				Perquintill::from_percent(100),
			));
			assert_ok!(StakePallet::set_inflation(
				Origin::root(),
				Perquintill::from_percent(100),
				Perquintill::from_percent(0),
				Perquintill::from_percent(100),
				Perquintill::from_percent(100),
			));
			assert_ok!(StakePallet::set_inflation(
				Origin::root(),
				Perquintill::from_percent(100),
				Perquintill::from_percent(100),
				Perquintill::from_percent(0),
				Perquintill::from_percent(100),
			));
			assert_ok!(StakePallet::set_inflation(
				Origin::root(),
				Perquintill::from_percent(100),
				Perquintill::from_percent(100),
				Perquintill::from_percent(100),
				Perquintill::from_percent(0),
			));
		});
}

#[test]
fn unlock_unstaked() {
	// same_unstaked_as_restaked
	// block 1: stake & unstake for 100
	// block 2: stake & unstake for 100
	// should remove first entry in unstaking BTreeMap when staking in block 2
	// should still have 100 locked until unlocking
	ExtBuilder::default()
		.with_balances(vec![(1, 10), (2, 100)])
		.with_collators(vec![(1, 10)])
		.with_delegators(vec![(2, 1, 100)])
		.build()
		.execute_with(|| {
			assert_ok!(StakePallet::revoke_delegation(Origin::signed(2), 1));
			let mut unstaking = BTreeMap::new();
			unstaking.insert(3, 100);
			let lock = BalanceLock {
				id: STAKING_ID,
				amount: 100,
				reasons: Reasons::All,
			};
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Balances::locks(2), vec![lock.clone()]);
			// shouldn't be able to unlock anything
			assert_ok!(StakePallet::unlock_unstaked(Origin::signed(2), 2));
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Balances::locks(2), vec![lock.clone()]);

			// join delegators and revoke again --> consume unstaking at block 3
			roll_to(2, vec![]);
			assert_ok!(StakePallet::join_delegators(Origin::signed(2), 1, 100));
			assert_ok!(StakePallet::revoke_delegation(Origin::signed(2), 1));
			unstaking.remove(&3);
			unstaking.insert(4, 100);
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Balances::locks(2), vec![lock.clone()]);
			// shouldn't be able to unlock anything
			assert_ok!(StakePallet::unlock_unstaked(Origin::signed(2), 2));
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Balances::locks(2), vec![lock.clone()]);

			// should reduce unlocking but not unlock anything
			roll_to(3, vec![]);
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Balances::locks(2), vec![lock.clone()]);
			// shouldn't be able to unlock anything
			assert_ok!(StakePallet::unlock_unstaked(Origin::signed(2), 2));
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Balances::locks(2), vec![lock.clone()]);

			roll_to(4, vec![]);
			unstaking.remove(&4);
			assert_eq!(Balances::locks(2), vec![lock]);
			// shouldn't be able to unlock anything
			assert_ok!(StakePallet::unlock_unstaked(Origin::signed(2), 2));
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Balances::locks(2), vec![]);
		});

	// less_unstaked_than_restaked
	// block 1: stake & unstake for 10
	// block 2: stake & unstake for 100
	// should remove first entry in unstaking BTreeMap when staking in block 2
	// should still have 90 locked until unlocking in block 4
	ExtBuilder::default()
		.with_balances(vec![(1, 10), (2, 100)])
		.with_collators(vec![(1, 10)])
		.with_delegators(vec![(2, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(StakePallet::revoke_delegation(Origin::signed(2), 1));
			let mut unstaking = BTreeMap::new();
			unstaking.insert(3, 10);
			let mut lock = BalanceLock {
				id: STAKING_ID,
				amount: 10,
				reasons: Reasons::All,
			};
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Balances::locks(2), vec![lock.clone()]);
			// shouldn't be able to unlock anything
			assert_ok!(StakePallet::unlock_unstaked(Origin::signed(2), 2));
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Balances::locks(2), vec![lock.clone()]);

			// join delegators and revoke again
			roll_to(2, vec![]);
			assert_ok!(StakePallet::join_delegators(Origin::signed(2), 1, 100));
			assert_ok!(StakePallet::revoke_delegation(Origin::signed(2), 1));
			unstaking.remove(&3);
			unstaking.insert(4, 100);
			lock.amount = 100;
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Balances::locks(2), vec![lock.clone()]);
			// shouldn't be able to unlock anything
			assert_ok!(StakePallet::unlock_unstaked(Origin::signed(2), 2));
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Balances::locks(2), vec![lock.clone()]);

			roll_to(3, vec![]);
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Balances::locks(2), vec![lock.clone()]);
			// shouldn't be able to unlock anything
			assert_ok!(StakePallet::unlock_unstaked(Origin::signed(2), 2));
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Balances::locks(2), vec![lock.clone()]);

			// unlock unstaked, remove lock, empty unlocking
			roll_to(4, vec![]);
			unstaking.remove(&4);
			assert_eq!(Balances::locks(2), vec![lock]);
			assert_ok!(StakePallet::unlock_unstaked(Origin::signed(2), 2));
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Balances::locks(2), vec![]);
		});

	// more_unstaked_than_restaked
	// block 1: stake & unstake for 100
	// block 2: stake & unstake for 10
	// should reduce first entry from amount 100 to 90 in unstaking BTreeMap when
	// staking in block 2
	// should have 100 locked until unlocking in block 3, then 10
	// should have 10 locked until further unlocking in block 4
	ExtBuilder::default()
		.with_balances(vec![(1, 10), (2, 100)])
		.with_collators(vec![(1, 10)])
		.with_delegators(vec![(2, 1, 100)])
		.build()
		.execute_with(|| {
			assert_ok!(StakePallet::revoke_delegation(Origin::signed(2), 1));
			let mut unstaking = BTreeMap::new();
			unstaking.insert(3, 100);
			let mut lock = BalanceLock {
				id: STAKING_ID,
				amount: 100,
				reasons: Reasons::All,
			};
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Balances::locks(2), vec![lock.clone()]);
			// shouldn't be able to unlock anything
			assert_ok!(StakePallet::unlock_unstaked(Origin::signed(2), 2));
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Balances::locks(2), vec![lock.clone()]);

			// join delegators and revoke again
			roll_to(2, vec![]);
			assert_ok!(StakePallet::join_delegators(Origin::signed(2), 1, 10));
			assert_ok!(StakePallet::revoke_delegation(Origin::signed(2), 1));
			unstaking.insert(3, 90);
			unstaking.insert(4, 10);
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Balances::locks(2), vec![lock.clone()]);
			// shouldn't be able to unlock anything
			assert_ok!(StakePallet::unlock_unstaked(Origin::signed(2), 2));
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Balances::locks(2), vec![lock.clone()]);

			// should reduce unlocking but not unlock anything
			roll_to(3, vec![]);
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Balances::locks(2), vec![lock.clone()]);
			// should be able to unlock 90 of 100 from unstaking
			assert_ok!(StakePallet::unlock_unstaked(Origin::signed(2), 2));
			unstaking.remove(&3);
			lock.amount = 10;
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Balances::locks(2), vec![lock.clone()]);

			roll_to(4, vec![]);
			assert_eq!(Balances::locks(2), vec![lock]);
			// should be able to unlock 10 of remaining 10
			assert_ok!(StakePallet::unlock_unstaked(Origin::signed(2), 2));
			unstaking.remove(&4);
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Balances::locks(2), vec![]);
		});

	// test_stake_less
	// block 1: stake & unstake for 100
	// block 2: stake & unstake for 10
	// should reduce first entry from amount 100 to 90 in unstaking BTreeMap when
	// staking in block 2
	// should have 100 locked until unlocking in block 3, then 10
	// should have 10 locked until further unlocking in block 4
	ExtBuilder::default()
		.with_balances(vec![(1, 200), (2, 200)])
		.with_collators(vec![(1, 200)])
		.with_delegators(vec![(2, 1, 200)])
		.build()
		.execute_with(|| {
			// should be able to decrease more often than MaxUnstakeRequests because it's
			// the same block and thus unstaking is increased at block 3 instead of having
			// multiple entries for the same block
			assert_ok!(StakePallet::candidate_stake_less(Origin::signed(1), 10));
			assert_ok!(StakePallet::candidate_stake_less(Origin::signed(1), 10));
			assert_ok!(StakePallet::candidate_stake_less(Origin::signed(1), 10));
			assert_ok!(StakePallet::candidate_stake_less(Origin::signed(1), 10));
			assert_ok!(StakePallet::candidate_stake_less(Origin::signed(1), 10));
			assert_ok!(StakePallet::candidate_stake_less(Origin::signed(1), 10),);
			assert_ok!(StakePallet::delegator_stake_less(Origin::signed(2), 1, 10));
			assert_ok!(StakePallet::delegator_stake_less(Origin::signed(2), 1, 10));
			assert_ok!(StakePallet::delegator_stake_less(Origin::signed(2), 1, 10));
			assert_ok!(StakePallet::delegator_stake_less(Origin::signed(2), 1, 10));
			assert_ok!(StakePallet::delegator_stake_less(Origin::signed(2), 1, 10));
			assert_ok!(StakePallet::delegator_stake_less(Origin::signed(2), 1, 10),);
			let mut unstaking = BTreeMap::new();
			unstaking.insert(3, 60);
			let mut lock = BalanceLock {
				id: STAKING_ID,
				amount: 200,
				reasons: Reasons::All,
			};
			assert_eq!(Balances::locks(1), vec![lock.clone()]);
			assert_eq!(Balances::locks(2), vec![lock.clone()]);
			assert_eq!(StakePallet::unstaking(1), unstaking);
			assert_eq!(StakePallet::unstaking(2), unstaking);
			// shouldn't be able to unlock anything
			assert_ok!(StakePallet::unlock_unstaked(Origin::signed(1), 1));
			assert_ok!(StakePallet::unlock_unstaked(Origin::signed(2), 2));
			assert_eq!(StakePallet::unstaking(1), unstaking);
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Balances::locks(1), vec![lock.clone()]);
			assert_eq!(Balances::locks(2), vec![lock.clone()]);

			roll_to(2, vec![]);
			assert_ok!(StakePallet::candidate_stake_less(Origin::signed(1), 10),);
			assert_ok!(StakePallet::delegator_stake_less(Origin::signed(2), 1, 10),);
			unstaking.insert(4, 10);
			assert_eq!(Balances::locks(1), vec![lock.clone()]);
			assert_eq!(Balances::locks(2), vec![lock.clone()]);
			assert_eq!(StakePallet::unstaking(1), unstaking);
			assert_eq!(StakePallet::unstaking(2), unstaking);
			// shouldn't be able to unlock anything
			assert_ok!(StakePallet::unlock_unstaked(Origin::signed(1), 1));
			assert_ok!(StakePallet::unlock_unstaked(Origin::signed(2), 2));
			assert_eq!(StakePallet::unstaking(1), unstaking);
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Balances::locks(1), vec![lock.clone()]);
			assert_eq!(Balances::locks(2), vec![lock.clone()]);

			roll_to(3, vec![]);
			assert_ok!(StakePallet::candidate_stake_less(Origin::signed(1), 10),);
			assert_ok!(StakePallet::delegator_stake_less(Origin::signed(2), 1, 10),);
			unstaking.insert(5, 10);
			unstaking.insert(5, 10);
			assert_eq!(Balances::locks(1), vec![lock.clone()]);
			assert_eq!(Balances::locks(2), vec![lock.clone()]);
			assert_eq!(StakePallet::unstaking(1), unstaking);
			assert_eq!(StakePallet::unstaking(2), unstaking);
			// should unlock 60
			assert_ok!(StakePallet::unlock_unstaked(Origin::signed(1), 1));
			assert_ok!(StakePallet::unlock_unstaked(Origin::signed(2), 2));
			lock.amount = 140;
			unstaking.remove(&3);
			assert_eq!(StakePallet::unstaking(1), unstaking);
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Balances::locks(1), vec![lock.clone()]);
			assert_eq!(Balances::locks(2), vec![lock.clone()]);

			// reach MaxUnstakeRequests
			roll_to(4, vec![]);
			assert_ok!(StakePallet::candidate_stake_less(Origin::signed(1), 10));
			assert_ok!(StakePallet::delegator_stake_less(Origin::signed(2), 1, 10));
			roll_to(5, vec![]);
			assert_ok!(StakePallet::candidate_stake_less(Origin::signed(1), 10));
			assert_ok!(StakePallet::delegator_stake_less(Origin::signed(2), 1, 10));
			roll_to(6, vec![]);
			assert_ok!(StakePallet::candidate_stake_less(Origin::signed(1), 10));
			assert_ok!(StakePallet::delegator_stake_less(Origin::signed(2), 1, 10));
			roll_to(7, vec![]);
			assert_ok!(StakePallet::candidate_stake_less(Origin::signed(1), 10));
			assert_ok!(StakePallet::delegator_stake_less(Origin::signed(2), 1, 10));
			unstaking.insert(6, 10);
			unstaking.insert(7, 10);
			unstaking.insert(8, 10);
			unstaking.insert(9, 10);
			assert_eq!(StakePallet::unstaking(1), unstaking);
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Balances::locks(1), vec![lock.clone()]);
			assert_eq!(Balances::locks(2), vec![lock.clone()]);

			roll_to(8, vec![]);
			assert_noop!(
				StakePallet::candidate_stake_less(Origin::signed(1), 10),
				Error::<Test>::NoMoreUnstaking
			);
			assert_noop!(
				StakePallet::delegator_stake_less(Origin::signed(2), 1, 10),
				Error::<Test>::NoMoreUnstaking
			);
			assert_ok!(StakePallet::unlock_unstaked(Origin::signed(1), 1));
			assert_ok!(StakePallet::unlock_unstaked(Origin::signed(2), 2));
			unstaking.remove(&4);
			unstaking.remove(&5);
			unstaking.remove(&6);
			unstaking.remove(&7);
			unstaking.remove(&8);
			lock.amount = 90;
			assert_eq!(StakePallet::unstaking(1), unstaking);
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Balances::locks(1), vec![lock.clone()]);
			assert_eq!(Balances::locks(2), vec![lock.clone()]);
			assert_ok!(StakePallet::candidate_stake_less(Origin::signed(1), 40));
			assert_ok!(StakePallet::delegator_stake_less(Origin::signed(2), 1, 40));
			unstaking.insert(10, 40);
			assert_ok!(StakePallet::candidate_stake_more(Origin::signed(1), 30));
			assert_ok!(StakePallet::delegator_stake_more(Origin::signed(2), 1, 30));
			unstaking.remove(&9);
			unstaking.insert(10, 20);
			assert_eq!(StakePallet::unstaking(1), unstaking);
			assert_eq!(StakePallet::unstaking(2), unstaking);
			assert_eq!(Balances::locks(1), vec![lock.clone()]);
			assert_eq!(Balances::locks(2), vec![lock]);
		});
}

#[test]
fn candidate_leaves() {
	let balances: Vec<(AccountId, Balance)> = (1u64..15u64).map(|id| (id, 100)).collect();
	ExtBuilder::default()
		.with_balances(balances)
		.with_collators(vec![(1, 100), (2, 100)])
		.with_delegators(vec![(12, 1, 100), (13, 1, 10)])
		.build()
		.execute_with(|| {
			assert_noop!(
				StakePallet::init_leave_candidates(Origin::signed(11)),
				Error::<Test>::CandidateNotFound
			);
			assert_noop!(
				StakePallet::init_leave_candidates(Origin::signed(1)),
				Error::<Test>::TooFewCollatorCandidates
			);
			// add five more collator to max fill CandidatePool
			for candidate in 3u64..11u64 {
				assert_ok!(StakePallet::join_candidates(Origin::signed(candidate), 100));
			}
			assert_ok!(StakePallet::init_leave_candidates(Origin::signed(1)));
			assert_noop!(
				StakePallet::join_delegators(Origin::signed(15), 1, 10),
				Error::<Test>::CannotDelegateIfLeaving
			);
			assert_noop!(
				StakePallet::delegator_stake_more(Origin::signed(12), 1, 1),
				Error::<Test>::CannotDelegateIfLeaving
			);
			assert_noop!(
				StakePallet::delegator_stake_less(Origin::signed(12), 1, 1),
				Error::<Test>::CannotDelegateIfLeaving
			);
			assert_noop!(
				StakePallet::init_leave_candidates(Origin::signed(1)),
				Error::<Test>::AlreadyLeaving
			);
			assert_eq!(
				StakePallet::collator_state(1).unwrap().state,
				CollatorStatus::Leaving(2)
			);
			assert!(StakePallet::collator_state(1).unwrap().can_exit(2));
			assert!(!StakePallet::collator_state(1).unwrap().can_exit(1));
			assert!(StakePallet::collator_state(1).unwrap().can_exit(3));

			// next rounds starts, cannot leave yet
			roll_to(5, vec![]);
			assert_noop!(
				StakePallet::execute_leave_candidates(Origin::signed(2), 2),
				Error::<Test>::NotLeaving
			);
			assert_noop!(
				StakePallet::execute_leave_candidates(Origin::signed(2), 1),
				Error::<Test>::CannotLeaveYet
			);
			// add 11 to max out CandidatePool and then leave again to enable 11 to cancel
			// the exit request
			assert_ok!(StakePallet::join_candidates(Origin::signed(11), 100));
			assert_noop!(
				StakePallet::cancel_leave_candidates(Origin::signed(1)),
				Error::<Test>::TooManyCollatorCandidates
			);
			assert_ok!(StakePallet::init_leave_candidates(Origin::signed(11)));
			// join back
			assert_ok!(StakePallet::cancel_leave_candidates(Origin::signed(1)));

			let stake: Vec<Stake<AccountId, Balance>> = (1u64..11u64)
				.zip(iter::once(210).chain(iter::repeat(100)))
				.map(|(id, amount)| Stake { owner: id, amount })
				.collect();
			assert_eq!(StakePallet::candidate_pool(), OrderedSet::from(stake));
			let state = StakePallet::collator_state(1).unwrap();
			assert_eq!(state.state, CollatorStatus::Active);
			assert_eq!(state.delegators.len(), 2);
			assert_eq!(state.total, 210);
			assert_eq!(
				state.total,
				StakePallet::candidate_pool()
					.binary_search_by(|other| other.owner.cmp(&1))
					.map(|i| StakePallet::candidate_pool()[i].clone())
					.unwrap()
					.amount
			);
			assert_eq!(StakePallet::selected_candidates(), vec![1, 2]);

			assert_ok!(StakePallet::init_leave_candidates(Origin::signed(1)));

			roll_to(15, vec![]);
			assert_ok!(StakePallet::execute_leave_candidates(Origin::signed(13), 1));
			let mut unstaking = BTreeMap::new();
			unstaking.insert(17, 100);
			assert_eq!(StakePallet::unstaking(1), unstaking);
			assert_eq!(StakePallet::unstaking(12), unstaking);

			// cannot unlock yet
			roll_to(16, vec![]);
			assert_ok!(StakePallet::unlock_unstaked(Origin::signed(4), 1));
			assert_ok!(StakePallet::unlock_unstaked(Origin::signed(4), 12));
			assert_eq!(StakePallet::unstaking(1), unstaking);
			assert_eq!(StakePallet::unstaking(12), unstaking);

			// can unlock now
			roll_to(17, vec![]);
			unstaking.remove(&17);
			assert_ok!(StakePallet::unlock_unstaked(Origin::signed(4), 1));
			assert_ok!(StakePallet::unlock_unstaked(Origin::signed(4), 12));
			assert_eq!(StakePallet::unstaking(1), unstaking);
			assert_eq!(StakePallet::unstaking(12), unstaking);
		});
}

#[test]
fn adjust_reward_rates() {
	ExtBuilder::default()
		.with_balances(vec![(1, 10_000_000 * DECIMALS), (2, 90_000_000 * DECIMALS)])
		.with_collators(vec![(1, 10_000_000 * DECIMALS)])
		.with_delegators(vec![(2, 1, 40_000_000 * DECIMALS)])
		.with_inflation(10, 10, 40, 8, 5)
		.build()
		.execute_with(|| {
			let inflation_0 = StakePallet::inflation_config();
			let num_of_years = 3 * BLOCKS_PER_YEAR;
			// 1 authors every block
			let authors: Vec<Option<AccountId>> = (0u64..=num_of_years).map(|_| Some(1u64)).collect();

			// reward once in first year
			roll_to(2, authors.clone());
			let c_rewards_0 = Balances::free_balance(&1).saturating_sub(10_000_000 * DECIMALS);
			let d_rewards_0 = Balances::free_balance(&2).saturating_sub(90_000_000 * DECIMALS);
			assert!(!c_rewards_0.is_zero());
			assert!(!d_rewards_0.is_zero());

			// finish first year
			System::set_block_number(BLOCKS_PER_YEAR);
			roll_to(BLOCKS_PER_YEAR + 1, vec![]);
			assert_eq!(StakePallet::last_reward_reduction(), 1u64);
			let inflation_1 = InflationInfo::new(
				inflation_0.collator.max_rate,
				Perquintill::from_parts(98000000000000000),
				inflation_0.delegator.max_rate,
				Perquintill::from_percent(6),
			);
			assert_eq!(StakePallet::inflation_config(), inflation_1);
			// reward once in 2nd year
			roll_to(BLOCKS_PER_YEAR + 2, authors.clone());
			let c_rewards_1 = Balances::free_balance(&1)
				.saturating_sub(10_000_000 * DECIMALS)
				.saturating_sub(c_rewards_0);
			let d_rewards_1 = Balances::free_balance(&2)
				.saturating_sub(90_000_000 * DECIMALS)
				.saturating_sub(d_rewards_0);
			assert!(
				c_rewards_0 > c_rewards_1,
				"left {:?}, right {:?}",
				c_rewards_0,
				c_rewards_1
			);
			assert!(d_rewards_0 > d_rewards_1);

			// finish 2nd year
			System::set_block_number(2 * BLOCKS_PER_YEAR);
			roll_to(2 * BLOCKS_PER_YEAR + 1, vec![]);
			assert_eq!(StakePallet::last_reward_reduction(), 2u64);
			let inflation_2 = InflationInfo::new(
				inflation_0.collator.max_rate,
				Perquintill::from_parts(96040000000000000),
				inflation_0.delegator.max_rate,
				Perquintill::zero(),
			);
			assert_eq!(StakePallet::inflation_config(), inflation_2);
			// reward once in 3rd year
			roll_to(2 * BLOCKS_PER_YEAR + 2, authors);
			let c_rewards_2 = Balances::free_balance(&1)
				.saturating_sub(10_000_000 * DECIMALS)
				.saturating_sub(c_rewards_0)
				.saturating_sub(c_rewards_1);
			let d_rewards_2 = Balances::free_balance(&2)
				.saturating_sub(90_000_000 * DECIMALS)
				.saturating_sub(d_rewards_0)
				.saturating_sub(d_rewards_1);
			assert!(c_rewards_1 > c_rewards_2);
			assert!(d_rewards_2.is_zero());
		});
}

#[test]
fn increase_max_candidate_stake_by() {
	let max_stake = 160_000_000 * DECIMALS;
	ExtBuilder::default()
		.with_balances(vec![(1, 200_000_000 * DECIMALS)])
		.with_collators(vec![(1, max_stake)])
		.build()
		.execute_with(|| {
			assert_eq!(StakePallet::max_candidate_stake(), max_stake);
			assert_noop!(
				StakePallet::candidate_stake_more(Origin::signed(1), 1),
				Error::<Test>::ValStakeAboveMax
			);

			assert_ok!(StakePallet::increase_max_candidate_stake_by(Origin::root(), 1));
			assert_eq!(
				last_event(),
				MetaEvent::StakePallet(Event::MaxCandidateStakeChanged(max_stake, max_stake + 1))
			);
			assert_eq!(StakePallet::max_candidate_stake(), max_stake + 1);
			assert_ok!(StakePallet::candidate_stake_more(Origin::signed(1), 1));
			assert_noop!(
				StakePallet::candidate_stake_more(Origin::signed(1), 1),
				Error::<Test>::ValStakeAboveMax
			);
		});
}

#[test]
fn decrease_max_candidate_stake_by() {
	ExtBuilder::default()
		.with_balances(vec![(1, 100), (2, 100), (3, 100), (4, 100), (5, 100)])
		.with_collators(vec![(1, 100), (2, 90), (3, 40)])
		.with_delegators(vec![(4, 2, 10), (5, 3, 20)])
		.build()
		.execute_with(|| {
			assert_eq!(StakePallet::selected_candidates(), vec![1, 2]);
			assert_eq!(
				StakePallet::candidate_pool(),
				OrderedSet::from_sorted_set(vec![
					Stake { owner: 1, amount: 100 },
					Stake { owner: 2, amount: 100 },
					Stake { owner: 3, amount: 60 }
				])
			);
			let max_stake = StakePallet::max_candidate_stake();

			assert_ok!(StakePallet::decrease_max_candidate_stake_by(
				Origin::root(),
				max_stake - 50
			));
			assert_eq!(StakePallet::max_candidate_stake(), 50);
			assert_eq!(
				last_event(),
				MetaEvent::StakePallet(Event::MaxCandidateStakeChanged(160_000_000 * DECIMALS, 50))
			);

			// check collator states
			assert_eq!(
				StakePallet::candidate_pool(),
				OrderedSet::from_sorted_set(vec![
					Stake { owner: 1, amount: 50 },
					Stake { owner: 2, amount: 60 },
					Stake { owner: 3, amount: 60 },
				])
			);
			assert_eq!(
				StakePallet::collator_state(1),
				Some(Collator {
					id: 1,
					stake: 50,
					delegators: OrderedSet::from(vec![]),
					total: 50,
					state: CollatorStatus::Active
				})
			);
			assert_eq!(
				StakePallet::collator_state(2),
				Some(Collator {
					id: 2,
					stake: 50,
					delegators: OrderedSet::from(vec![Stake { owner: 4, amount: 10 }]),
					total: 60,
					state: CollatorStatus::Active
				})
			);
			assert_eq!(
				StakePallet::collator_state(3),
				Some(Collator {
					id: 3,
					stake: 40,
					delegators: OrderedSet::from(vec![Stake { owner: 5, amount: 20 }]),
					total: 60,
					state: CollatorStatus::Active
				})
			);
			assert_eq!(StakePallet::selected_candidates(), vec![2, 3]);

			assert_noop!(
				StakePallet::candidate_stake_more(Origin::signed(1), 0),
				Error::<Test>::ValStakeZero
			);
			assert_noop!(
				StakePallet::candidate_stake_less(Origin::signed(1), 0),
				Error::<Test>::ValStakeZero
			);
			assert_noop!(
				StakePallet::candidate_stake_more(Origin::signed(1), 1),
				Error::<Test>::ValStakeAboveMax
			);
			assert_noop!(
				StakePallet::decrease_max_candidate_stake_by(Origin::root(), 41),
				Error::<Test>::CannotSetBelowMin
			);
		});
}

#[test]
fn exceed_delegations_per_round() {
	ExtBuilder::default()
		.with_balances(vec![(1, 100), (2, 100), (3, 100), (4, 100), (5, 100), (6, 100)])
		.with_collators(vec![(1, 100), (2, 100), (3, 100), (4, 100), (5, 100)])
		.with_delegators(vec![(6, 1, 10)])
		.build()
		.execute_with(|| {
			assert_ok!(StakePallet::delegate_another_candidate(Origin::signed(6), 2, 10));
			assert_ok!(StakePallet::delegate_another_candidate(Origin::signed(6), 3, 10));
			assert_ok!(StakePallet::delegate_another_candidate(Origin::signed(6), 4, 10));
			assert_noop!(
				StakePallet::delegate_another_candidate(Origin::signed(6), 5, 10),
				Error::<Test>::ExceedMaxCollatorsPerDelegator
			);

			// revoke delegation to allow one more collator for this delegator
			assert_ok!(StakePallet::revoke_delegation(Origin::signed(6), 4));
			// reached max delegations in this round
			assert_noop!(
				StakePallet::delegate_another_candidate(Origin::signed(6), 5, 10),
				Error::<Test>::ExceededDelegationsPerRound
			);

			// revoke all delegations in the same round
			assert_ok!(StakePallet::leave_delegators(Origin::signed(6)));
			assert_noop!(
				StakePallet::join_delegators(Origin::signed(6), 1, 10),
				Error::<Test>::ExceededDelegationsPerRound
			);

			// roll to next round to clear DelegationCounter
			roll_to(5, vec![]);
			assert_eq!(
				StakePallet::last_delegation(6),
				DelegationCounter { round: 0, counter: 4 }
			);
			assert_ok!(StakePallet::join_delegators(Origin::signed(6), 1, 10),);
			assert_eq!(
				StakePallet::last_delegation(6),
				DelegationCounter { round: 1, counter: 1 }
			);
			assert_ok!(StakePallet::leave_delegators(Origin::signed(6)));
			assert_ok!(StakePallet::join_delegators(Origin::signed(6), 1, 10),);
			assert_ok!(StakePallet::leave_delegators(Origin::signed(6)));
			assert_ok!(StakePallet::join_delegators(Origin::signed(6), 1, 10),);
			assert_ok!(StakePallet::leave_delegators(Origin::signed(6)));
			assert_ok!(StakePallet::join_delegators(Origin::signed(6), 1, 10),);
			assert_ok!(StakePallet::leave_delegators(Origin::signed(6)));
			assert_eq!(
				StakePallet::last_delegation(6),
				DelegationCounter { round: 1, counter: 4 }
			);
			assert_noop!(
				StakePallet::join_delegators(Origin::signed(6), 1, 10),
				Error::<Test>::ExceededDelegationsPerRound
			);
		});
}

#[test]
fn force_remove_candidate() {
	ExtBuilder::default()
		.with_balances(vec![(1, 100), (2, 100), (3, 100), (4, 100), (5, 100), (6, 100)])
		.with_collators(vec![(1, 100), (2, 100), (3, 100)])
		.with_delegators(vec![(4, 1, 50), (5, 1, 50)])
		.build()
		.execute_with(|| {
			assert_ok!(StakePallet::delegate_another_candidate(Origin::signed(4), 2, 50));
			assert_eq!(StakePallet::selected_candidates(), vec![1, 2]);
			assert!(StakePallet::unstaking(1).get(&3).is_none());
			assert!(StakePallet::unstaking(2).get(&3).is_none());
			assert!(StakePallet::unstaking(3).get(&3).is_none());

			assert_ok!(StakePallet::force_remove_candidate(Origin::root(), 1));
			assert_eq!(last_event(), MetaEvent::StakePallet(Event::CollatorRemoved(1, 200)));
			assert!(!StakePallet::candidate_pool().contains(&Stake { owner: 1, amount: 100 }));
			assert_eq!(StakePallet::selected_candidates(), vec![2, 3]);
			assert!(StakePallet::collator_state(1).is_none());
			assert!(StakePallet::delegator_state(5).is_none());
			assert_eq!(
				StakePallet::delegator_state(4),
				Some(Delegator {
					delegations: OrderedSet::from(vec![Stake { owner: 2, amount: 50 }]),
					total: 50
				})
			);
			assert_eq!(StakePallet::unstaking(1).get(&3), Some(&100));
			assert_eq!(StakePallet::unstaking(4).get(&3), Some(&50));
			assert_eq!(StakePallet::unstaking(5).get(&3), Some(&50));

			assert_noop!(
				StakePallet::force_remove_candidate(Origin::root(), 2),
				Error::<Test>::TooFewCollatorCandidates
			);
			assert_noop!(
				StakePallet::force_remove_candidate(Origin::root(), 4),
				Error::<Test>::CandidateNotFound
			);
		});
}

#[test]
fn prioritize_collator() {
	ExtBuilder::default()
		.with_balances(vec![(1, 100), (2, 100), (3, 100)])
		.with_collators(vec![(2, 100), (3, 100)])
		.build()
		.execute_with(|| {
			assert_eq!(StakePallet::selected_candidates(), vec![2, 3]);
			assert_ok!(StakePallet::join_candidates(Origin::signed(1), 100));
			assert_eq!(StakePallet::selected_candidates(), vec![1, 2]);
			assert_ok!(StakePallet::init_leave_candidates(Origin::signed(2)));
			assert_eq!(StakePallet::selected_candidates(), vec![1, 3]);
			assert_ok!(StakePallet::candidate_stake_less(Origin::signed(1), 10));
			assert_eq!(StakePallet::selected_candidates(), vec![3, 1]);
		});
}

#[test]
fn authorities_per_round() {
	let stake = 100 * DECIMALS;
	ExtBuilder::default()
		.with_balances(vec![
			(1, stake),
			(2, stake),
			(3, stake),
			(4, stake),
			(5, stake),
			(6, stake),
			(7, stake),
			(8, stake),
			(9, stake),
			(10, stake),
			(11, 100 * stake),
		])
		.with_collators(vec![(1, stake), (2, stake), (3, stake), (4, stake)])
		.build()
		.execute_with(|| {
			assert_eq!(StakePallet::selected_candidates(), vec![1, 2]);
			// reward 1 once per round
			let authors: Vec<Option<AccountId>> = (0u64..=100)
				.map(|i| if i % 5 == 2 { Some(1u64) } else { None })
				.collect();
			let inflation = StakePallet::inflation_config();

			// roll to last block of round 0
			roll_to(4, authors.clone());
			let reward_0 = inflation.collator.reward_rate.per_block * stake * 2;
			assert_eq!(Balances::free_balance(1), stake + reward_0);
			// increase max selected candidates which will become effective in round 2
			assert_ok!(StakePallet::set_max_selected_candidates(Origin::root(), 20));

			// roll to last block of round 1
			// should still multiply with 2 because the Authority set was chosen at start of
			// round 1
			roll_to(9, authors.clone());
			let reward_1 = inflation.collator.reward_rate.per_block * stake * 2;
			assert_eq!(Balances::free_balance(1), stake + reward_0 + reward_1);

			// roll to last block of round 2
			// should multiply with 4 because there are only 4 candidates
			roll_to(14, authors.clone());
			let reward_2 = inflation.collator.reward_rate.per_block * stake * 4;
			assert_eq!(Balances::free_balance(1), stake + reward_0 + reward_1 + reward_2);

			// roll to last block of round 3
			// should multiply with 4 because there are only 4 candidates
			roll_to(19, authors);
			let reward_3 = inflation.collator.reward_rate.per_block * stake * 4;
			assert_eq!(
				Balances::free_balance(1),
				stake + reward_0 + reward_1 + reward_2 + reward_3
			);
		});
}
