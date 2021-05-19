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
use crate::{
	mock::{
		almost_equal, events, last_event, roll_to, AccountId, Authorship, Balance, Balances, BlockNumber,
		Event as MetaEvent, ExtBuilder, Origin, Stake, System, Test, BLOCKS_PER_ROUND, DECIMALS,
	},
	types::{BalanceOf, Bond, CollatorSnapshot, CollatorStatus, RoundInfo, TotalStake},
	Config, Error, Event, InflationInfo, Pallet,
};
use frame_support::{assert_noop, assert_ok, traits::EstimateNextSessionRotation};
use kilt_primitives::constants::YEARS;
use pallet_balances::Error as BalancesError;
use pallet_session::{SessionManager, ShouldEndSession};
use sp_runtime::{traits::Zero, Perbill, Percent, Perquintill};

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
		.with_collators(vec![
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
		])
		.build()
		.execute_with(|| {
			assert_eq!(
				Stake::new_session(0)
					.expect("first session must return new collators")
					.len(),
				Pallet::<Test>::total_selected() as usize
			);
			assert_eq!(
				Stake::new_session(1)
					.expect("second session must return new collators")
					.len(),
				Pallet::<Test>::total_selected() as usize
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
				Stake::total(),
				TotalStake {
					collators: 700,
					delegators: 400
				}
			);
			assert_eq!(
				vec![Bond { owner: 1, amount: 700 }, Bond { owner: 2, amount: 400 }],
				Stake::candidate_pool().to_vec()
			);
			// 1
			assert_eq!(Balances::usable_balance(&1), 500);
			assert_eq!(Balances::free_balance(&1), 1000);
			assert!(Stake::is_candidate(&1));
			assert_eq!(
				Stake::at_stake(0u32, &1),
				CollatorSnapshot {
					bond: 500,
					delegators: vec![Bond { owner: 3, amount: 100 }, Bond { owner: 4, amount: 100 }],
					total: 700
				}
			);
			// 2
			assert_eq!(Balances::usable_balance(&2), 100);
			assert_eq!(Balances::free_balance(&2), 300);
			assert!(Stake::is_candidate(&2));
			assert_eq!(
				Stake::at_stake(0u32, &2),
				CollatorSnapshot {
					bond: 200,
					delegators: vec![Bond { owner: 5, amount: 100 }, Bond { owner: 6, amount: 100 }],
					total: 400
				}
			);
			// Delegators
			assert_eq!(
				Stake::total(),
				TotalStake {
					collators: 700,
					delegators: 400
				}
			);
			for x in 3..7 {
				assert!(Stake::is_delegator(&x));
				assert_eq!(Balances::usable_balance(&x), 0);
				assert_eq!(Balances::free_balance(&x), 100);
			}
			// Uninvolved
			for x in 7..10 {
				assert!(!Stake::is_delegator(&x));
			}
			assert_eq!(Balances::free_balance(&7), 100);
			assert_eq!(Balances::usable_balance(&7), 100);
			assert_eq!(Balances::free_balance(&8), 9);
			assert_eq!(Balances::usable_balance(&8), 9);
			assert_eq!(Balances::free_balance(&9), 4);
			assert_eq!(Balances::usable_balance(&9), 4);

			// Safety first checks
			assert_eq!(Stake::total_selected(), <Test as Config>::MinSelectedCandidates::get());
			assert_eq!(
				Stake::round(),
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
				Stake::total(),
				TotalStake {
					collators: 90,
					delegators: 50
				}
			);
			assert_eq!(
				Stake::candidate_pool().to_vec(),
				vec![
					Bond { owner: 1, amount: 50 },
					Bond { owner: 2, amount: 40 },
					Bond { owner: 3, amount: 20 },
					Bond { owner: 4, amount: 20 },
					Bond { owner: 5, amount: 10 }
				]
			);
			for x in 1..5 {
				assert!(Stake::is_candidate(&x));
				assert_eq!(Balances::free_balance(&x), 100);
				assert_eq!(Balances::usable_balance(&x), 80);
			}
			assert!(Stake::is_candidate(&5));
			assert_eq!(Balances::free_balance(&5), 100);
			assert_eq!(Balances::usable_balance(&5), 90);
			// Delegators
			for x in 6..11 {
				assert!(Stake::is_delegator(&x));
				assert_eq!(Balances::free_balance(&x), 100);
				assert_eq!(Balances::usable_balance(&x), 90);
			}

			// Safety first checks
			assert_eq!(Stake::total_selected(), <Test as Config>::MinSelectedCandidates::get());
			assert_eq!(
				Stake::round(),
				RoundInfo::new(0, 0, <Test as Config>::DefaultBlocksPerRound::get())
			);
		});
}

#[test]
fn online_offline_works() {
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
			roll_to(4, vec![]);
			assert_noop!(Stake::go_offline(Origin::signed(3)), Error::<Test>::CandidateDNE);
			roll_to(11, vec![]);
			assert_noop!(Stake::go_online(Origin::signed(3)), Error::<Test>::CandidateDNE);
			assert_noop!(Stake::go_online(Origin::signed(2)), Error::<Test>::AlreadyActive);
			assert_ok!(Stake::go_offline(Origin::signed(2)));
			assert_eq!(last_event(), MetaEvent::stake(Event::CollatorWentOffline(2, 2)));
			roll_to(21, vec![]);
			let mut expected = vec![
				Event::CollatorChosen(1, 1, 500, 200),
				Event::CollatorChosen(1, 2, 200, 200),
				Event::NewRound(5, 1, 2, 700, 400),
				Event::CollatorChosen(2, 1, 500, 200),
				Event::CollatorChosen(2, 2, 200, 200),
				Event::NewRound(10, 2, 2, 700, 400),
				Event::CollatorWentOffline(2, 2),
				Event::CollatorChosen(3, 1, 500, 200),
				Event::NewRound(15, 3, 1, 500, 200),
				Event::CollatorChosen(4, 1, 500, 200),
				Event::NewRound(20, 4, 1, 500, 200),
			];
			assert_eq!(events(), expected);
			assert_noop!(Stake::go_offline(Origin::signed(2)), Error::<Test>::AlreadyOffline);
			assert_ok!(Stake::go_online(Origin::signed(2)));
			assert_eq!(last_event(), MetaEvent::stake(Event::CollatorBackOnline(4, 2)));
			expected.push(Event::CollatorBackOnline(4, 2));
			roll_to(26, vec![]);
			expected.push(Event::CollatorChosen(5, 1, 500, 200));
			expected.push(Event::CollatorChosen(5, 2, 200, 200));
			expected.push(Event::NewRound(25, 5, 2, 700, 400));
			assert_eq!(events(), expected);
			assert_ok!(Stake::leave_candidates(Origin::signed(1)));
			assert_noop!(
				Stake::go_online(Origin::signed(1)),
				Error::<Test>::CannotActivateIfLeaving
			);
			assert_noop!(
				Stake::go_offline(Origin::signed(1)),
				Error::<Test>::CannotActivateIfLeaving
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
			assert_noop!(
				Stake::join_candidates(Origin::signed(1), 11u128,),
				Error::<Test>::CandidateExists
			);
			assert_noop!(
				Stake::join_candidates(Origin::signed(3), 11u128,),
				Error::<Test>::DelegatorExists
			);
			assert_noop!(
				Stake::join_candidates(Origin::signed(7), 9u128,),
				Error::<Test>::ValBondBelowMin
			);
			assert_noop!(
				Stake::join_candidates(Origin::signed(8), 10u128,),
				BalancesError::<Test>::InsufficientBalance
			);
			assert!(System::events().is_empty());
			assert_ok!(Stake::join_candidates(Origin::signed(7), 10u128,));
			assert_eq!(
				last_event(),
				MetaEvent::stake(Event::JoinedCollatorCandidates(7, 10u128, 710u128))
			);

			// MaxCollatorCandidateStk
			assert_noop!(
				Stake::join_candidates(Origin::signed(10), 161_000_000 * DECIMALS),
				Error::<Test>::ValBondAboveMax
			);
			assert_ok!(Stake::join_candidates(
				Origin::signed(10),
				<Test as Config>::MaxCollatorCandidateStk::get()
			));
			assert_eq!(
				last_event(),
				MetaEvent::stake(Event::JoinedCollatorCandidates(
					10,
					<Test as Config>::MaxCollatorCandidateStk::get(),
					<Test as Config>::MaxCollatorCandidateStk::get() + 710u128
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
		.with_collators(vec![(1, 500), (2, 200)])
		.with_delegators(vec![(3, 1, 100), (4, 1, 100), (5, 2, 100), (6, 2, 100)])
		.build()
		.execute_with(|| {
			roll_to(4, vec![]);
			assert_noop!(Stake::leave_candidates(Origin::signed(3)), Error::<Test>::CandidateDNE);
			roll_to(11, vec![]);
			assert_ok!(Stake::leave_candidates(Origin::signed(2)));
			assert_eq!(last_event(), MetaEvent::stake(Event::CollatorScheduledExit(2, 2, 4)));
			let info = Stake::collator_state(&2).unwrap();
			assert_eq!(info.state, CollatorStatus::Leaving(4));
			roll_to(21, vec![]);
			// we must exclude leaving collators from rewards while
			// holding them retroactively accountable for previous faults
			// (within the last T::SlashingWindow blocks)
			let expected = vec![
				Event::CollatorChosen(1, 1, 500, 200),
				Event::CollatorChosen(1, 2, 200, 200),
				Event::NewRound(5, 1, 2, 700, 400),
				Event::CollatorChosen(2, 1, 500, 200),
				Event::CollatorChosen(2, 2, 200, 200),
				Event::NewRound(10, 2, 2, 700, 400),
				Event::CollatorScheduledExit(2, 2, 4),
				Event::CollatorChosen(3, 1, 500, 200),
				Event::NewRound(15, 3, 1, 500, 200),
				Event::CollatorLeft(2, 400, 500, 200),
				Event::CollatorChosen(4, 1, 500, 200),
				Event::NewRound(20, 4, 1, 500, 200),
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
			roll_to(8, vec![]);
			// should choose top TotalSelectedCandidates (5), in order
			let expected = vec![
				Event::CollatorChosen(1, 1, 100, 0),
				Event::CollatorChosen(1, 2, 90, 0),
				Event::CollatorChosen(1, 3, 80, 0),
				Event::CollatorChosen(1, 4, 70, 0),
				Event::CollatorChosen(1, 5, 60, 0),
				Event::NewRound(5, 1, 5, 400, 0),
			];
			assert_eq!(events(), expected);
			assert_ok!(Stake::leave_candidates(Origin::signed(6)));
			assert_eq!(last_event(), MetaEvent::stake(Event::CollatorScheduledExit(1, 6, 3)));
			roll_to(21, vec![]);
			assert_ok!(Stake::join_candidates(Origin::signed(6), 69u128));
			assert_eq!(
				last_event(),
				MetaEvent::stake(Event::JoinedCollatorCandidates(6, 69u128, 469u128))
			);
			roll_to(27, vec![]);
			// should choose top TotalSelectedCandidates (5), in order
			let expected = vec![
				Event::CollatorChosen(1, 1, 100, 0),
				Event::CollatorChosen(1, 2, 90, 0),
				Event::CollatorChosen(1, 3, 80, 0),
				Event::CollatorChosen(1, 4, 70, 0),
				Event::CollatorChosen(1, 5, 60, 0),
				Event::NewRound(5, 1, 5, 400, 0),
				Event::CollatorScheduledExit(1, 6, 3),
				Event::CollatorChosen(2, 1, 100, 0),
				Event::CollatorChosen(2, 2, 90, 0),
				Event::CollatorChosen(2, 3, 80, 0),
				Event::CollatorChosen(2, 4, 70, 0),
				Event::CollatorChosen(2, 5, 60, 0),
				Event::NewRound(10, 2, 5, 400, 0),
				Event::CollatorLeft(6, 50, 400, 0),
				Event::CollatorChosen(3, 1, 100, 0),
				Event::CollatorChosen(3, 2, 90, 0),
				Event::CollatorChosen(3, 3, 80, 0),
				Event::CollatorChosen(3, 4, 70, 0),
				Event::CollatorChosen(3, 5, 60, 0),
				Event::NewRound(15, 3, 5, 400, 0),
				Event::CollatorChosen(4, 1, 100, 0),
				Event::CollatorChosen(4, 2, 90, 0),
				Event::CollatorChosen(4, 3, 80, 0),
				Event::CollatorChosen(4, 4, 70, 0),
				Event::CollatorChosen(4, 5, 60, 0),
				Event::NewRound(20, 4, 5, 400, 0),
				Event::JoinedCollatorCandidates(6, 69, 469),
				Event::CollatorChosen(5, 1, 100, 0),
				Event::CollatorChosen(5, 2, 90, 0),
				Event::CollatorChosen(5, 3, 80, 0),
				Event::CollatorChosen(5, 4, 70, 0),
				Event::CollatorChosen(5, 6, 69, 0),
				Event::NewRound(25, 5, 5, 409, 0),
			];
			assert_eq!(events(), expected);
		});
}

#[test]
fn exit_queue() {
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
			roll_to(8, vec![]);
			// should choose top TotalSelectedCandidates (5), in order
			let mut expected = vec![
				Event::CollatorChosen(1, 1, 100, 0),
				Event::CollatorChosen(1, 2, 90, 0),
				Event::CollatorChosen(1, 3, 80, 0),
				Event::CollatorChosen(1, 4, 70, 0),
				Event::CollatorChosen(1, 5, 60, 0),
				Event::NewRound(5, 1, 5, 400, 0),
			];
			assert_eq!(events(), expected);
			assert_ok!(Stake::leave_candidates(Origin::signed(6)));
			assert_eq!(last_event(), MetaEvent::stake(Event::CollatorScheduledExit(1, 6, 3)));
			roll_to(11, vec![]);
			assert_ok!(Stake::leave_candidates(Origin::signed(5)));
			assert_eq!(last_event(), MetaEvent::stake(Event::CollatorScheduledExit(2, 5, 4)));
			roll_to(16, vec![]);
			assert_ok!(Stake::leave_candidates(Origin::signed(4)));
			assert_eq!(last_event(), MetaEvent::stake(Event::CollatorScheduledExit(3, 4, 5)));
			assert_noop!(
				Stake::leave_candidates(Origin::signed(4)),
				Error::<Test>::AlreadyLeaving
			);
			roll_to(21, vec![]);
			let mut new_events = vec![
				Event::CollatorScheduledExit(1, 6, 3),
				Event::CollatorChosen(2, 1, 100, 0),
				Event::CollatorChosen(2, 2, 90, 0),
				Event::CollatorChosen(2, 3, 80, 0),
				Event::CollatorChosen(2, 4, 70, 0),
				Event::CollatorChosen(2, 5, 60, 0),
				Event::NewRound(10, 2, 5, 400, 0),
				Event::CollatorScheduledExit(2, 5, 4),
				Event::CollatorLeft(6, 50, 400, 0),
				Event::CollatorChosen(3, 1, 100, 0),
				Event::CollatorChosen(3, 2, 90, 0),
				Event::CollatorChosen(3, 3, 80, 0),
				Event::CollatorChosen(3, 4, 70, 0),
				Event::NewRound(15, 3, 4, 340, 0),
				Event::CollatorScheduledExit(3, 4, 5),
				Event::CollatorLeft(5, 60, 340, 0),
				Event::CollatorChosen(4, 1, 100, 0),
				Event::CollatorChosen(4, 2, 90, 0),
				Event::CollatorChosen(4, 3, 80, 0),
				Event::NewRound(20, 4, 3, 270, 0),
			];
			expected.append(&mut new_events);
			assert_eq!(events(), expected);
		});
}

// Total issuance 6_099_000_000
// At stake: 450_000_000 (7.37% below max rate of 10%)
// At stake of active collators: 400_000_000 (6.55%)
// TODO: Apply coinbase rewards
// #[test]
// fn payout_distribution_to_solo_collators_below_max_rate() {
// 	let blocks_per_round: BlockNumber = 600;
// 	// max_rate not met
// 	ExtBuilder::default()
// 		.with_balances(vec![
// 			(1, 1_000_000_000),
// 			(2, 1_000_000_000),
// 			(3, 1_000_000_000),
// 			(4, 1_000_000_000),
// 			(5, 1_000_000_000),
// 			(6, 1_000_000_000),
// 			(7, 33_000_000),
// 			(8, 33_000_000),
// 			(9, 33_000_000),
// 		])
// 		.with_collators(vec![
// 			(1, 100_000_000),
// 			(2, 90_000_000),
// 			(3, 80_000_000),
// 			(4, 70_000_000),
// 			(5, 60_000_000),
// 			(6, 50_000_000),
// 		])
// 		.with_inflation(10, 15, 40, 10, blocks_per_round as u32)
// 		.build()
// 		.execute_with(|| {
// 			let inflation = Stake::inflation_config();
// 			let total_issuance = <Test as Config>::Currency::total_issuance();
// 			assert_eq!(total_issuance, 6_099_000_000);
// 			let rewards = inflation.collator.compute_rewards::<Test>(450_000_000,
// total_issuance); 			assert_eq!(rewards, 7812);

// 			let mut round: BlockNumber = 1;
// 			roll_to(round * blocks_per_round + 1, vec![]);
// 			// should choose top TotalCandidatesSelected (5), in order
// 			let mut expected = vec![
// 				Event::CollatorChosen(1, 1, 100_000_000, 0),
// 				Event::CollatorChosen(1, 2, 90_000_000, 0),
// 				Event::CollatorChosen(1, 3, 80_000_000, 0),
// 				Event::CollatorChosen(1, 4, 70_000_000, 0),
// 				Event::CollatorChosen(1, 5, 60_000_000, 0),
// 				Event::NewRound(blocks_per_round, 2, 5, 400_000_000, 0),
// 			];
// 			assert_eq!(events(), expected);
// 			// ~ set block author as 1 for all blocks this round
// 			set_author(2, 1, 100);
// 			round = 3;
// 			roll_to(round * blocks_per_round + 1, vec![]);
// 			// pay total issuance to 1
// 			let mut new = vec![
// 				Event::CollatorChosen(2, 1, 100_000_000, 0),
// 				Event::CollatorChosen(2, 2, 90_000_000, 0),
// 				Event::CollatorChosen(2, 3, 80_000_000, 0),
// 				Event::CollatorChosen(2, 4, 70_000_000, 0),
// 				Event::CollatorChosen(2, 5, 60_000_000, 0),
// 				Event::NewRound(2 * blocks_per_round, 3, 5, 400_000_000, 0),
// 				Event::Rewarded(1, rewards),
// 				Event::CollatorChosen(3, 1, 100_000_000, 0),
// 				Event::CollatorChosen(3, 2, 90_000_000, 0),
// 				Event::CollatorChosen(3, 3, 80_000_000, 0),
// 				Event::CollatorChosen(3, 4, 70_000_000, 0),
// 				Event::CollatorChosen(3, 5, 60_000_000, 0),
// 				Event::NewRound(3 * blocks_per_round, 4, 5, 400_000_000, 0),
// 			];
// 			expected.append(&mut new);
// 			assert_eq!(events(), expected);
// 			// ~ set block author as 1 for 3 blocks this round
// 			set_author(4, 1, 60);
// 			// ~ set block author as 2 for 2 blocks this round
// 			set_author(4, 2, 40);
// 			round = 5;
// 			roll_to(round * blocks_per_round + 1, vec![]);

// 			// pay 60% total issuance to 1 and 40% total issuance to 2
// 			let mut new1 = vec![
// 				Event::CollatorChosen(4, 1, 100_000_000, 0),
// 				Event::CollatorChosen(4, 2, 90_000_000, 0),
// 				Event::CollatorChosen(4, 3, 80_000_000, 0),
// 				Event::CollatorChosen(4, 4, 70_000_000, 0),
// 				Event::CollatorChosen(4, 5, 60_000_000, 0),
// 				Event::NewRound(4 * blocks_per_round, 5, 5, 400_000_000, 0),
// 				Event::Rewarded(1, Perbill::from_percent(60) * rewards),
// 				Event::Rewarded(2, Perbill::from_percent(40) * rewards),
// 				Event::CollatorChosen(5, 1, 100_000_000, 0),
// 				Event::CollatorChosen(5, 2, 90_000_000, 0),
// 				Event::CollatorChosen(5, 3, 80_000_000, 0),
// 				Event::CollatorChosen(5, 4, 70_000_000, 0),
// 				Event::CollatorChosen(5, 5, 60_000_000, 0),
// 				Event::NewRound(5 * blocks_per_round, 6, 5, 400_000_000, 0),
// 			];
// 			expected.append(&mut new1);
// 			assert_eq!(events(), expected);
// 			// ~ each collator produces 1 block this round
// 			set_author(6, 1, 20);
// 			set_author(6, 2, 20);
// 			set_author(6, 3, 20);
// 			set_author(6, 4, 20);
// 			set_author(6, 5, 20);
// 			round = 7;
// 			roll_to(round * blocks_per_round + 1, vec![]);

// 			// pay 20% issuance for all collators
// 			let mut new2 = vec![
// 				Event::CollatorChosen(7, 1, 100_000_000, 0),
// 				Event::CollatorChosen(7, 2, 90_000_000, 0),
// 				Event::CollatorChosen(7, 3, 80_000_000, 0),
// 				Event::CollatorChosen(7, 4, 70_000_000, 0),
// 				Event::CollatorChosen(7, 5, 60_000_000, 0),
// 				Event::NewRound(6 * blocks_per_round, 7, 5, 400_000_000, 0),
// 				Event::Rewarded(5, Perbill::from_percent(20) * rewards),
// 				Event::Rewarded(3, Perbill::from_percent(20) * rewards),
// 				Event::Rewarded(4, Perbill::from_percent(20) * rewards),
// 				Event::Rewarded(1, Perbill::from_percent(20) * rewards),
// 				Event::Rewarded(2, Perbill::from_percent(20) * rewards),
// 				Event::CollatorChosen(8, 1, 100_000_000, 0),
// 				Event::CollatorChosen(8, 2, 90_000_000, 0),
// 				Event::CollatorChosen(8, 3, 80_000_000, 0),
// 				Event::CollatorChosen(8, 4, 70_000_000, 0),
// 				Event::CollatorChosen(8, 5, 60_000_000, 0),
// 				Event::NewRound(7 * blocks_per_round, 8, 5, 400_000_000, 0),
// 			];
// 			expected.append(&mut new2);
// 			assert_eq!(events(), expected);
// 			// check that distributing rewards clears awarded pts
// 			assert!(Stake::awarded_pts(1, 1).is_zero());
// 			assert!(Stake::awarded_pts(4, 1).is_zero());
// 			assert!(Stake::awarded_pts(4, 2).is_zero());
// 			assert!(Stake::awarded_pts(6, 1).is_zero());
// 			assert!(Stake::awarded_pts(6, 2).is_zero());
// 			assert!(Stake::awarded_pts(6, 3).is_zero());
// 			assert!(Stake::awarded_pts(6, 4).is_zero());
// 			assert!(Stake::awarded_pts(6, 5).is_zero());
// 		});
// }

// Total issuance 6_099_000_000
// At stake: 850_000_000 (14.10% exceeds max_rate of 10%)
// At stake of active collators: 800_000_000 (13.11%)
// TODO: Apply coinbase rewards
// #[test]
// fn payout_distribution_to_solo_collators_above_max_rate() {
// 	let blocks_per_round: BlockNumber = 600;

// 	ExtBuilder::default()
// 		.with_balances(vec![
// 			(1, 1_000_000_000),
// 			(2, 1_000_000_000),
// 			(3, 1_000_000_000),
// 			(4, 1_000_000_000),
// 			(5, 1_000_000_000),
// 			(6, 1_000_000_000),
// 			(7, 33_000_000),
// 			(8, 33_000_000),
// 			(9, 33_000_000),
// 		])
// 		.with_collators(vec![
// 			(1, 500_000_000),
// 			(2, 90_000_000),
// 			(3, 80_000_000),
// 			(4, 70_000_000),
// 			(5, 60_000_000),
// 			(6, 50_000_000),
// 		])
// 		.with_inflation(10, 15, 40, 10, blocks_per_round as u32)
// 		.build()
// 		.execute_with(|| {
// 			let inflation = Stake::inflation_config();
// 			let total_issuance = <Test as Config>::Currency::total_issuance();
// 			assert_eq!(total_issuance, 6_099_000_000);
// 			let rewards = inflation
// 				.collator
// 				.compute_rewards::<Test>(Perbill::from_percent(10) * total_issuance,
// total_issuance); 			assert_eq!(rewards, 10588);

// 			roll_to(blocks_per_round + 1, vec![]);
// 			// should choose top TotalCandidatesSelected (5), in order
// 			let mut expected = vec![
// 				Event::CollatorChosen(2, 1, 500_000_000, 0),
// 				Event::CollatorChosen(2, 2, 90_000_000, 0),
// 				Event::CollatorChosen(2, 3, 80_000_000, 0),
// 				Event::CollatorChosen(2, 4, 70_000_000, 0),
// 				Event::CollatorChosen(2, 5, 60_000_000, 0),
// 				Event::NewRound(blocks_per_round, 2, 5, 800_000_000, 0),
// 			];
// 			assert_eq!(events(), expected);
// 			// ~ set block author as 1 for all blocks this round
// 			set_author(2, 1, 100);
// 			roll_to(3 * blocks_per_round + 1, vec![]);
// 			// pay total issuance to 1
// 			let mut new = vec![
// 				Event::CollatorChosen(3, 1, 500_000_000, 0),
// 				Event::CollatorChosen(3, 2, 90_000_000, 0),
// 				Event::CollatorChosen(3, 3, 80_000_000, 0),
// 				Event::CollatorChosen(3, 4, 70_000_000, 0),
// 				Event::CollatorChosen(3, 5, 60_000_000, 0),
// 				Event::NewRound(2 * blocks_per_round, 3, 5, 800_000_000, 0),
// 				Event::Rewarded(1, rewards),
// 				Event::CollatorChosen(4, 1, 500_000_000, 0),
// 				Event::CollatorChosen(4, 2, 90_000_000, 0),
// 				Event::CollatorChosen(4, 3, 80_000_000, 0),
// 				Event::CollatorChosen(4, 4, 70_000_000, 0),
// 				Event::CollatorChosen(4, 5, 60_000_000, 0),
// 				Event::NewRound(3 * blocks_per_round, 4, 5, 800_000_000, 0),
// 			];
// 			expected.append(&mut new);
// 			assert_eq!(events(), expected);
// 			// ~ set block author as 1 for 3 blocks this round
// 			set_author(4, 1, 60);
// 			// ~ set block author as 2 for 2 blocks this round
// 			set_author(4, 2, 40);
// 			roll_to(5 * blocks_per_round + 1, vec![]);

// 			// pay 60% total issuance to 1 and 40% total issuance to 2
// 			let mut new1 = vec![
// 				Event::CollatorChosen(4, 1, 500_000_000, 0),
// 				Event::CollatorChosen(4, 2, 90_000_000, 0),
// 				Event::CollatorChosen(4, 3, 80_000_000, 0),
// 				Event::CollatorChosen(4, 4, 70_000_000, 0),
// 				Event::CollatorChosen(4, 5, 60_000_000, 0),
// 				Event::NewRound(4 * blocks_per_round, 5, 5, 800_000_000, 0),
// 				Event::Rewarded(1, Perbill::from_percent(60) * rewards),
// 				Event::Rewarded(2, Perbill::from_percent(40) * rewards),
// 				Event::CollatorChosen(5, 1, 500_000_000, 0),
// 				Event::CollatorChosen(5, 2, 90_000_000, 0),
// 				Event::CollatorChosen(5, 3, 80_000_000, 0),
// 				Event::CollatorChosen(5, 4, 70_000_000, 0),
// 				Event::CollatorChosen(5, 5, 60_000_000, 0),
// 				Event::NewRound(5 * blocks_per_round, 6, 5, 800_000_000, 0),
// 			];
// 			expected.append(&mut new1);
// 			assert_eq!(events(), expected);
// 			// ~ each collator produces 1 block this round
// 			set_author(6, 1, 20);
// 			set_author(6, 2, 20);
// 			set_author(6, 3, 20);
// 			set_author(6, 4, 20);
// 			set_author(6, 5, 20);
// 			roll_to(7 * blocks_per_round + 1, vec![]);
// 			// pay 20% issuance for all collators
// 			let mut new2 = vec![
// 				Event::CollatorChosen(7, 1, 500_000_000, 0),
// 				Event::CollatorChosen(7, 2, 90_000_000, 0),
// 				Event::CollatorChosen(7, 3, 80_000_000, 0),
// 				Event::CollatorChosen(7, 4, 70_000_000, 0),
// 				Event::CollatorChosen(7, 5, 60_000_000, 0),
// 				Event::NewRound(6 * blocks_per_round, 7, 5, 800_000_000, 0),
// 				Event::Rewarded(5, Perbill::from_percent(20) * rewards),
// 				Event::Rewarded(3, Perbill::from_percent(20) * rewards),
// 				Event::Rewarded(4, Perbill::from_percent(20) * rewards),
// 				Event::Rewarded(1, Perbill::from_percent(20) * rewards),
// 				Event::Rewarded(2, Perbill::from_percent(20) * rewards),
// 				Event::CollatorChosen(8, 1, 500_000_000, 0),
// 				Event::CollatorChosen(8, 2, 90_000_000, 0),
// 				Event::CollatorChosen(8, 3, 80_000_000, 0),
// 				Event::CollatorChosen(8, 4, 70_000_000, 0),
// 				Event::CollatorChosen(8, 5, 60_000_000, 0),
// 				Event::NewRound(7 * blocks_per_round, 8, 5, 800_000_000, 0),
// 			];
// 			expected.append(&mut new2);
// 			assert_eq!(events(), expected);
// 			// check that distributing rewards clears awarded pts
// 			assert!(Stake::awarded_pts(1, 1).is_zero());
// 			assert!(Stake::awarded_pts(4, 1).is_zero());
// 			assert!(Stake::awarded_pts(4, 2).is_zero());
// 			assert!(Stake::awarded_pts(6, 1).is_zero());
// 			assert!(Stake::awarded_pts(6, 2).is_zero());
// 			assert!(Stake::awarded_pts(6, 3).is_zero());
// 			assert!(Stake::awarded_pts(6, 4).is_zero());
// 			assert!(Stake::awarded_pts(6, 5).is_zero());
// 		});
// }

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
			roll_to(8, vec![]);
			// chooses top TotalSelectedCandidates (5), in order
			let mut expected = vec![
				Event::CollatorChosen(1, 1, 20, 30),
				Event::CollatorChosen(1, 2, 20, 20),
				Event::CollatorChosen(1, 4, 20, 0),
				Event::CollatorChosen(1, 3, 20, 0),
				Event::CollatorChosen(1, 5, 10, 0),
				Event::NewRound(5, 1, 5, 90, 50),
			];
			assert_eq!(events(), expected);
			assert_noop!(
				Stake::delegate_another_candidate(Origin::signed(6), 1, 10),
				Error::<Test>::AlreadyDelegatedCollator,
			);
			assert_noop!(
				Stake::delegate_another_candidate(Origin::signed(6), 2, 2),
				Error::<Test>::DelegationBelowMin,
			);
			assert_ok!(Stake::delegate_another_candidate(Origin::signed(6), 2, 10));
			assert_ok!(Stake::delegate_another_candidate(Origin::signed(6), 3, 10));
			assert_ok!(Stake::delegate_another_candidate(Origin::signed(6), 4, 10));
			assert_noop!(
				Stake::delegate_another_candidate(Origin::signed(6), 5, 10),
				Error::<Test>::ExceedMaxCollatorsPerDelegator,
			);
			roll_to(16, vec![]);
			let mut new = vec![
				Event::Delegation(6, 10, 2, 50),
				Event::Delegation(6, 10, 3, 30),
				Event::Delegation(6, 10, 4, 30),
				Event::CollatorChosen(2, 2, 20, 30),
				Event::CollatorChosen(2, 1, 20, 30),
				Event::CollatorChosen(2, 4, 20, 10),
				Event::CollatorChosen(2, 3, 20, 10),
				Event::CollatorChosen(2, 5, 10, 0),
				Event::NewRound(10, 2, 5, 90, 80),
				Event::CollatorChosen(3, 2, 20, 30),
				Event::CollatorChosen(3, 1, 20, 30),
				Event::CollatorChosen(3, 4, 20, 10),
				Event::CollatorChosen(3, 3, 20, 10),
				Event::CollatorChosen(3, 5, 10, 0),
				Event::NewRound(15, 3, 5, 90, 80),
			];
			expected.append(&mut new);
			assert_eq!(events(), expected);
			roll_to(21, vec![]);
			assert_ok!(Stake::delegate_another_candidate(Origin::signed(7), 2, 80));
			assert_noop!(
				Stake::delegate_another_candidate(Origin::signed(7), 3, 11),
				BalancesError::<Test>::InsufficientBalance
			);
			assert_noop!(
				Stake::delegate_another_candidate(Origin::signed(10), 2, 10),
				Error::<Test>::TooManyDelegators
			);
			assert_ok!(Stake::delegate_another_candidate(Origin::signed(10), 2, 11),);
			roll_to(26, vec![]);
			let mut new2 = vec![
				Event::CollatorChosen(4, 2, 20, 30),
				Event::CollatorChosen(4, 1, 20, 30),
				Event::CollatorChosen(4, 4, 20, 10),
				Event::CollatorChosen(4, 3, 20, 10),
				Event::CollatorChosen(4, 5, 10, 0),
				Event::NewRound(20, 4, 5, 90, 80),
				Event::Delegation(7, 80, 2, 130),
				Event::DelegationReplaced(10, 11, 9, 10, 2, 131),
				Event::Delegation(10, 11, 2, 131),
				Event::CollatorChosen(5, 2, 20, 111),
				Event::CollatorChosen(5, 1, 20, 30),
				Event::CollatorChosen(5, 4, 20, 10),
				Event::CollatorChosen(5, 3, 20, 10),
				Event::CollatorChosen(5, 5, 10, 0),
				Event::NewRound(25, 5, 5, 90, 161),
			];
			expected.append(&mut new2);
			assert_eq!(events(), expected);
			assert_ok!(Stake::leave_candidates(Origin::signed(2)));
			assert_eq!(last_event(), MetaEvent::stake(Event::CollatorScheduledExit(5, 2, 7)));
			roll_to(31, vec![]);
			let mut new3 = vec![
				Event::CollatorScheduledExit(5, 2, 7),
				Event::CollatorChosen(6, 1, 20, 30),
				Event::CollatorChosen(6, 4, 20, 10),
				Event::CollatorChosen(6, 3, 20, 10),
				Event::CollatorChosen(6, 5, 10, 0),
				Event::NewRound(30, 6, 4, 70, 50),
			];
			expected.append(&mut new3);
			assert_eq!(events(), expected);

			// test join_delegator errors
			assert_ok!(Stake::delegate_another_candidate(Origin::signed(8), 1, 10),);
			assert_noop!(
				Stake::join_delegators(Origin::signed(11), 1, 10),
				Error::<Test>::TooManyDelegators
			);
			assert_noop!(
				Stake::delegate_another_candidate(Origin::signed(11), 1, 10),
				Error::<Test>::NotYetDelegating
			);
			assert_ok!(Stake::join_delegators(Origin::signed(11), 1, 11),);

			// verify that delegations are removed after collator leaves, not before
			assert_eq!(Stake::delegator_state(6).unwrap().total, 40);
			assert_eq!(Stake::delegator_state(6).unwrap().delegations.len(), 4usize);
			assert_eq!(Stake::delegator_state(7).unwrap().total, 90);
			assert_eq!(Stake::delegator_state(7).unwrap().delegations.len(), 2usize);
			assert_eq!(Balances::usable_balance(&6), 60);
			assert_eq!(Balances::usable_balance(&7), 10);
			assert_eq!(Balances::free_balance(&6), 100);
			assert_eq!(Balances::free_balance(&7), 100);
			// TODO: Enable after removing execute_delayed_collator_exits
			// let mut unbonding_6 = BTreeMap::new();
			// unbonding_6.insert(31 + <Test as Config>::BondDuration::get(), 10);
			// assert_eq!(Stake::unbonding(6), unbonding_6);
			// let mut unbonding_7 = BTreeMap::new();
			// unbonding_7.insert(31 + <Test as Config>::BondDuration::get(), 80);
			// assert_eq!(Stake::unbonding(6), unbonding_7);
			// TODO: Switch back to n == 40 after removing execute_delayed_collator_exits
			roll_to(50, vec![]);
			assert_eq!(Stake::delegator_state(6).unwrap().total, 30);
			assert_eq!(Stake::delegator_state(7).unwrap().total, 10);
			assert_eq!(Stake::delegator_state(6).unwrap().delegations.len(), 3usize);
			assert_eq!(Stake::delegator_state(7).unwrap().delegations.len(), 1usize);
			assert_ok!(Stake::withdraw_unbonded(Origin::signed(6), 6));
			assert_ok!(Stake::withdraw_unbonded(Origin::signed(7), 7));
			assert_eq!(Balances::usable_balance(&6), 70);
			assert_eq!(Balances::usable_balance(&7), 90);
			assert_eq!(Balances::free_balance(&6), 100);
			assert_eq!(Balances::free_balance(&7), 100);
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
		.build()
		.execute_with(|| {
			roll_to(4, vec![]);
			assert_noop!(
				Stake::candidate_bond_more(Origin::signed(6), 50),
				Error::<Test>::CandidateDNE
			);
			assert_ok!(Stake::candidate_bond_more(Origin::signed(1), 50));
			assert_noop!(
				Stake::candidate_bond_more(Origin::signed(1), 40),
				BalancesError::<Test>::InsufficientBalance
			);
			assert_ok!(Stake::leave_candidates(Origin::signed(1)));
			assert_noop!(
				Stake::candidate_bond_more(Origin::signed(1), 30),
				Error::<Test>::CannotActivateIfLeaving
			);
			roll_to(30, vec![]);
			assert_noop!(
				Stake::candidate_bond_more(Origin::signed(1), 40),
				Error::<Test>::CandidateDNE
			);
			assert_ok!(Stake::candidate_bond_more(Origin::signed(2), 80));
			assert_ok!(Stake::candidate_bond_less(Origin::signed(2), 90));
			assert_ok!(Stake::candidate_bond_less(Origin::signed(3), 10));
			assert_noop!(
				Stake::candidate_bond_less(Origin::signed(2), 11),
				Error::<Test>::Underflow
			);
			assert_noop!(
				Stake::candidate_bond_less(Origin::signed(2), 1),
				Error::<Test>::ValBondBelowMin
			);
			assert_noop!(
				Stake::candidate_bond_less(Origin::signed(3), 1),
				Error::<Test>::ValBondBelowMin
			);
			assert_noop!(
				Stake::candidate_bond_less(Origin::signed(4), 11),
				Error::<Test>::ValBondBelowMin
			);
			assert_ok!(Stake::candidate_bond_less(Origin::signed(4), 10));

			// MaxCollatorCandidateStk
			assert_ok!(Stake::join_candidates(
				Origin::signed(11),
				<Test as Config>::MaxCollatorCandidateStk::get()
			));
			assert_noop!(
				Stake::candidate_bond_more(Origin::signed(11), 1u128),
				Error::<Test>::ValBondAboveMax,
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
		.with_delegators(vec![(6, 1, 10), (7, 1, 10), (8, 2, 10), (9, 2, 10), (10, 1, 10)])
		.set_blocks_per_round(5)
		.build()
		.execute_with(|| {
			roll_to(4, vec![]);
			assert_noop!(
				Stake::delegator_bond_more(Origin::signed(1), 2, 50),
				Error::<Test>::DelegatorDNE
			);
			assert_noop!(
				Stake::delegator_bond_more(Origin::signed(6), 2, 50),
				Error::<Test>::DelegationDNE
			);
			assert_noop!(
				Stake::delegator_bond_more(Origin::signed(7), 6, 50),
				Error::<Test>::CandidateDNE
			);
			assert_noop!(
				Stake::delegator_bond_less(Origin::signed(6), 1, 11),
				Error::<Test>::Underflow
			);
			assert_noop!(
				Stake::delegator_bond_less(Origin::signed(6), 1, 8),
				Error::<Test>::DelegationBelowMin
			);
			assert_noop!(
				Stake::delegator_bond_less(Origin::signed(6), 1, 6),
				Error::<Test>::NomBondBelowMin
			);
			assert_ok!(Stake::delegator_bond_more(Origin::signed(6), 1, 10));
			assert_noop!(
				Stake::delegator_bond_less(Origin::signed(6), 2, 5),
				Error::<Test>::DelegationDNE
			);
			assert_noop!(
				Stake::delegator_bond_more(Origin::signed(6), 1, 81),
				BalancesError::<Test>::InsufficientBalance
			);
			roll_to(9, vec![]);
			assert_eq!(Balances::usable_balance(&6), 80);
			assert_ok!(Stake::leave_candidates(Origin::signed(1)));
			roll_to(31, vec![]);
			assert!(!Stake::is_delegator(&6));
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
				Stake::revoke_delegation(Origin::signed(1), 2),
				Error::<Test>::DelegatorDNE
			);
			assert_noop!(
				Stake::revoke_delegation(Origin::signed(6), 2),
				Error::<Test>::DelegationDNE
			);
			assert_noop!(Stake::leave_delegators(Origin::signed(1)), Error::<Test>::DelegatorDNE);
			assert_ok!(Stake::delegate_another_candidate(Origin::signed(6), 2, 3));
			assert_ok!(Stake::delegate_another_candidate(Origin::signed(6), 3, 3));
			assert_ok!(Stake::revoke_delegation(Origin::signed(6), 1));
			// cannot revoke delegation because would leave remaining total below
			// MinDelegatorStk
			assert_noop!(
				Stake::revoke_delegation(Origin::signed(6), 2),
				Error::<Test>::NomBondBelowMin
			);
			assert_noop!(
				Stake::revoke_delegation(Origin::signed(6), 3),
				Error::<Test>::NomBondBelowMin
			);
			// can revoke both remaining by calling leave delegators
			assert_ok!(Stake::leave_delegators(Origin::signed(6)));
			// this leads to 8 leaving set of delegators
			assert_ok!(Stake::revoke_delegation(Origin::signed(8), 2));
		});
}

// #[test]
// Total issuance 1_000_000_000_000
// At stake collators: 90_000_000 (0.009%)
// At stake delegators: 60_000_000 (0.006%)
// TODO: Apply coinbase rewards
// fn payouts_follow_delegation_changes() {
// 	let blocks_per_round: BlockNumber = 600;
// 	ExtBuilder::default()
// 		.with_balances(vec![
// 			(1, 100_000_000),
// 			(2, 100_000_000),
// 			(3, 100_000_000),
// 			(4, 100_000_000),
// 			(5, 100_000_000),
// 			(6, 100_000_000),
// 			(7, 100_000_000),
// 			(8, 100_000_000),
// 			(9, 100_000_000),
// 			(10, 50_000_000),
// 			(11, 50_000_000),
// 		])
// 		.with_collators(vec![
// 			(1, 20_000_000),
// 			(2, 20_000_000),
// 			(3, 20_000_000),
// 			(4, 20_000_000),
// 			(5, 10_000_000),
// 		])
// 		.with_delegators(vec![
// 			(6, 1, 20_000_000),
// 			(7, 1, 10_000_000),
// 			(8, 2, 10_000_000),
// 			(9, 2, 10_000_000),
// 			(10, 1, 10_000_000),
// 		])
// 		.with_inflation(10, 15, 40, 10, blocks_per_round as u32)
// 		.build()
// 		.execute_with(|| {
// 			roll_to(blocks_per_round + 1, vec![]);
// 			// choose top TotalSelectedCandidates (5) in order
// 			let mut expected = vec![
// 				// Round 2 initialization
// 				Event::CollatorChosen(2, 1, 20_000_000, 40_000_000),
// 				Event::CollatorChosen(2, 2, 20_000_000, 20_000_000),
// 				Event::CollatorChosen(2, 4, 20_000_000, 0),
// 				Event::CollatorChosen(2, 3, 20_000_000, 0),
// 				Event::CollatorChosen(2, 5, 10_000_000, 0),
// 				Event::NewRound(blocks_per_round, 2, 5, 90_000_000, 60_000_000),
// 			];
// 			assert_eq!(events(), expected);

// 			// Round 2 -> 3
// 			// set block author as 1 for all blocks this round
// 			set_author(2, 1, 100);
// 			roll_to(3 * blocks_per_round + 1, vec![]);
// 			// distribute total issuance to collator 1 and its delegators 6, 7, 10
// 			let mut round_2_to_3 = vec![
// 				Event::CollatorChosen(3, 1, 20_000_000, 40_000_000),
// 				Event::CollatorChosen(3, 2, 20_000_000, 20_000_000),
// 				Event::CollatorChosen(3, 4, 20_000_000, 0),
// 				Event::CollatorChosen(3, 3, 20_000_000, 0),
// 				Event::CollatorChosen(3, 5, 10_000_000, 0),
// 				Event::NewRound(2 * blocks_per_round, 3, 5, 90_000_000, 60_000_000),
// 				// Round 2 rewards
// 				Event::Rewarded(1, 1562),
// 				Event::Rewarded(6, 347),
// 				Event::Rewarded(7, 173),
// 				Event::Rewarded(10, 173),
// 				// Round 3 initialization
// 				Event::CollatorChosen(4, 1, 20_000_000, 40_000_000),
// 				Event::CollatorChosen(4, 2, 20_000_000, 20_000_000),
// 				Event::CollatorChosen(4, 4, 20_000_000, 0),
// 				Event::CollatorChosen(4, 3, 20_000_000, 0),
// 				Event::CollatorChosen(4, 5, 10_000_000, 0),
// 				Event::NewRound(3 * blocks_per_round, 4, 5, 90_000_000, 60_000_000),
// 			];
// 			expected.append(&mut round_2_to_3);
// 			assert_eq!(events(), expected);

// 			// Round 3 -> 4: 6 leaves delegators
// 			// set block author as 1 for all blocks this round
// 			set_author(3, 1, 100);
// 			assert_noop!(Stake::leave_delegators(Origin::signed(66)),
// Error::<Test>::DelegatorDNE); 			assert_ok!(Stake::leave_delegators(Origin::
// signed(6))); 			roll_to(4 * blocks_per_round + 1, vec![]);
// 			// ensure delegators are paid for 2 rounds after they leave, e.g. 6 should
// 			// receive rewards for rounds 3 and 4 after leaving during round 3
// 			let mut round_3_to_4 = vec![
// 				Event::DelegatorLeftCollator(6, 1, 20_000_000, 40_000_000),
// 				Event::DelegatorLeft(6, 20_000_000),
// 				// Round 3 rewards
// 				Event::Rewarded(1, 1562),
// 				Event::Rewarded(6, 231),
// 				Event::Rewarded(7, 116),
// 				Event::Rewarded(10, 116),
// 				// Event::Rewarded(6, 347),
// 				// Event::Rewarded(7, 173),
// 				// Event::Rewarded(10, 173),
// 				// Round 4 initialization
// 				Event::CollatorChosen(4, 2, 20_000_000, 20_000_000),
// 				Event::CollatorChosen(4, 1, 20_000_000, 20_000_000),
// 				Event::CollatorChosen(4, 4, 20_000_000, 0),
// 				Event::CollatorChosen(4, 3, 20_000_000, 0),
// 				Event::CollatorChosen(4, 5, 10_000_000, 0),
// 				Event::NewRound(4 * blocks_per_round, 5, 5, 90_000_000, 40_000_000),
// 			];
// 			expected.append(&mut round_3_to_4);
// 			assert_eq!(events(), expected);

// 			// Round 4 -> 5
// 			set_author(4, 1, 100);
// 			roll_to(5 * blocks_per_round + 1, vec![]);
// 			// last round in which 6 receives rewards after leaving in round 3
// 			let mut round_4_to_5 = vec![
// 				// Round 4 rewards
// 				Event::Rewarded(1, 1562),
// 				// TODO: Check whether it makes sense that the rewards shrink but 6 is still
// rewarded 				Event::Rewarded(6, 231),
// 				Event::Rewarded(7, 116),
// 				Event::Rewarded(10, 116),
// 				// Round 5 initialization
// 				Event::CollatorChosen(5, 2, 20_000_000, 20_000_000),
// 				Event::CollatorChosen(5, 1, 20_000_000, 20_000_000),
// 				Event::CollatorChosen(5, 4, 20_000_000, 0),
// 				Event::CollatorChosen(5, 3, 20_000_000, 0),
// 				Event::CollatorChosen(5, 5, 10_000_000, 0),
// 				Event::NewRound(5 * blocks_per_round, 6, 5, 90_000_000, 40_000_000),
// 			];
// 			expected.append(&mut round_4_to_5);
// 			assert_eq!(events(), expected);

// 			// Round 5 -> 6
// 			set_author(5, 1, 100);
// 			roll_to(6 * blocks_per_round + 1, vec![]);
// 			// 6 should not receive rewards
// 			let mut round_5_to_6 = vec![
// 				// Round 5 rewards
// 				Event::Rewarded(1, 1562),
// 				Event::Rewarded(7, 231),
// 				Event::Rewarded(10, 231),
// 				// Round 6 collators
// 				Event::CollatorChosen(7, 2, 20_000_000, 20_000_000),
// 				Event::CollatorChosen(7, 1, 20_000_000, 20_000_000),
// 				Event::CollatorChosen(7, 4, 20_000_000, 0),
// 				Event::CollatorChosen(7, 3, 20_000_000, 0),
// 				Event::CollatorChosen(7, 5, 10_000_000, 0),
// 				Event::NewRound(6 * blocks_per_round, 7, 5, 90_000_000, 40_000_000),
// 			];
// 			expected.append(&mut round_5_to_6);
// 			assert_eq!(events(), expected);

// 			// Round 6 -> 7: 8 delegates to 1
// 			set_author(6, 1, 100);
// 			assert_ok!(Stake::delegate_another_candidate(Origin::signed(8), 1,
// 30_000_000)); 			roll_to(7 * blocks_per_round + 1, vec![]);
// 			// new delegation should not be rewarded for this round and the next one
// (expect 			// rewards at conclusion of round 8)
// 			let mut round_6_to_7 = vec![
// 				// round 6 finalization
// 				Event::Delegation(8, 30_000_000, 1, 70_000_000),
// 				Event::Rewarded(1, 1562),
// 				Event::Rewarded(7, 405),
// 				Event::Rewarded(10, 405),
// 				// Event::Rewarded(7, 231),
// 				// Event::Rewarded(10, 231),
// 				// Round 7 initialization
// 				Event::CollatorChosen(8, 1, 20_000_000, 50_000_000),
// 				Event::CollatorChosen(8, 2, 20_000_000, 20_000_000),
// 				Event::CollatorChosen(8, 4, 20_000_000, 0),
// 				Event::CollatorChosen(8, 3, 20_000_000, 0),
// 				Event::CollatorChosen(8, 5, 10_000_000, 0),
// 				Event::NewRound(7 * blocks_per_round, 8, 5, 90_000_000, 70_000_000),
// 			];
// 			expected.append(&mut round_6_to_7);
// 			assert_eq!(events(), expected);

// 			// Round 7 -> 8
// 			set_author(7, 1, 100);
// 			roll_to(8 * blocks_per_round + 1, vec![]);
// 			// new delegation is still not rewarded yet, but should be next round
// 			let mut round_7_to_8 = vec![
// 				Event::Rewarded(1, 1562),
// 				// TODO: Check whether it makes sense to apply the stake for the rewards but
// not to the 				// Collator-Delegator-Pool
// 				// round 7 finalization
// 				Event::Rewarded(7, 405),
// 				Event::Rewarded(10, 405),
// 				// Round 8 initialization
// 				Event::CollatorChosen(9, 1, 20_000_000, 50_000_000),
// 				Event::CollatorChosen(9, 2, 20_000_000, 20_000_000),
// 				Event::CollatorChosen(9, 4, 20_000_000, 0),
// 				Event::CollatorChosen(9, 3, 20_000_000, 0),
// 				Event::CollatorChosen(9, 5, 10_000_000, 0),
// 				Event::NewRound(8 * blocks_per_round, 9, 5, 90_000_000, 70_000_000),
// 			];
// 			expected.append(&mut round_7_to_8);
// 			assert_eq!(events(), expected);

// 			// Round 8 -> 9
// 			set_author(8, 1, 100);
// 			roll_to(9 * blocks_per_round + 1, vec![]);
// 			// new delegation is rewarded for first time, 2 rounds after joining
// 			// (`BondDuration` = 2)
// 			let mut round_8_to_9 = vec![
// 				// round 8 finalization
// 				Event::Rewarded(1, 1562),
// 				Event::Rewarded(7, 162),
// 				Event::Rewarded(8, 486),
// 				Event::Rewarded(10, 162),
// 				// round 9 initiation
// 				Event::CollatorChosen(10, 1, 20_000_000, 50_000_000),
// 				Event::CollatorChosen(10, 2, 20_000_000, 20_000_000),
// 				Event::CollatorChosen(10, 4, 20_000_000, 0),
// 				Event::CollatorChosen(10, 3, 20_000_000, 0),
// 				Event::CollatorChosen(10, 5, 10_000_000, 0),
// 				Event::NewRound(9 * blocks_per_round, 10, 5, 90_000_000, 70_000_000),
// 			];
// 			expected.append(&mut round_8_to_9);
// 			assert_eq!(events(), expected);

// 			// Round 9 -> 10: 11 joins collator candidates (6 candidates in total)
// 			set_author(9, 1, 50);
// 			set_author(9, 2, 40);
// 			set_author(9, 3, 5);
// 			set_author(9, 11, 5);
// 			// new collator candidate with higher self bond than anyone else
// 			assert_ok!(Stake::join_candidates(Origin::signed(11), 30_000_000));
// 			roll_to(10 * blocks_per_round + 1, vec![]);
// 			// expect collator candidate 5 not to be chosen because of lowest stake
// 			// new collator should immediately be rewarded because they authored blocks
// 			let mut round_9_to_10 = vec![
// 				Event::JoinedCollatorCandidates(11, 30_000_000, 120_000_000),
// 				Event::Rewarded(3, 87),
// 				// reward 1 and their delegators
// 				Event::Rewarded(1, 868),
// 				Event::Rewarded(7, 81),
// 				Event::Rewarded(8, 243),
// 				Event::Rewarded(10, 81),
// 				// reward 2 and their delegators
// 				Event::Rewarded(2, 694),
// 				Event::Rewarded(8, 162),
// 				Event::Rewarded(9, 162),
// 				// reward 11
// 				Event::Rewarded(11, 87),
// 				// round 10 initiation
// 				Event::CollatorChosen(11, 1, 20_000_000, 50_000_000),
// 				Event::CollatorChosen(11, 2, 20_000_000, 20_000_000),
// 				Event::CollatorChosen(11, 11, 30_000_000, 0),
// 				Event::CollatorChosen(11, 4, 20_000_000, 0),
// 				Event::CollatorChosen(11, 3, 20_000_000, 0),
// 				Event::NewRound(10 * blocks_per_round, 11, 5, 110_000_000, 70_000_000),
// 			];
// 			expected.append(&mut round_9_to_10);
// 			assert_eq!(events(), expected);

// 			// Round 10 -> 11: 8 delegates to 11
// 			set_author(10, 1, 50);
// 			set_author(10, 2, 30);
// 			set_author(10, 3, 10);
// 			set_author(10, 5, 5);
// 			set_author(10, 11, 5);
// 			// 8 adds delegation to 11
// 			assert_ok!(Stake::delegate_another_candidate(Origin::signed(8), 11,
// 20_000_000)); 			roll_to(11 * blocks_per_round + 1, vec![]);
// 			// new delegation of 8 should not be rewarded for this and the following
// round 			let mut round_10_to_11 = vec![
// 				Event::Delegation(8, 20_000_000, 11, 50_000_000),
// 				Event::Rewarded(5, 87),
// 				Event::Rewarded(3, 174),
// 				// reward 1 and their delegators
// 				Event::Rewarded(1, 868),
// 				Event::Rewarded(7, 104),
// 				Event::Rewarded(8, 313),
// 				Event::Rewarded(10, 104),
// 				// reward 2 and their delegators
// 				Event::Rewarded(2, 521),
// 				Event::Rewarded(8, 156),
// 				Event::Rewarded(9, 156),
// 				// reward 11
// 				Event::Rewarded(11, 87),
// 				// round 11 initiation
// 				Event::CollatorChosen(12, 1, 20_000_000, 50_000_000),
// 				Event::CollatorChosen(12, 11, 30_000_000, 20_000_000),
// 				Event::CollatorChosen(12, 2, 20_000_000, 20_000_000),
// 				Event::CollatorChosen(12, 4, 20_000_000, 0),
// 				Event::CollatorChosen(12, 3, 20_000_000, 0),
// 				Event::NewRound(11 * blocks_per_round, 12, 5, 110_000_000, 90_000_000),
// 			];
// 			expected.append(&mut round_10_to_11);
// 			assert_eq!(events(), expected);

// 			// Round 11 -> 12: 9 delegates to 5
// 			set_author(11, 1, 50);
// 			set_author(11, 2, 30);
// 			set_author(11, 4, 10);
// 			set_author(11, 5, 5);
// 			set_author(11, 11, 5);
// 			// 9 adds delegation to 5
// 			assert_ok!(Stake::delegate_another_candidate(Origin::signed(9), 5,
// 30_000_000)); 			roll_to(12 * blocks_per_round + 1, vec![]);
// 			// delegation of 8 should not be rewarded for this round
// 			// new delegation of 9 should not be rewarded for this and the following
// round 			let mut round_11_to_12 = vec![
// 				Event::Delegation(9, 30_000_000, 5, 40_000_000),
// 				Event::Rewarded(5, 87),
// 				Event::Rewarded(4, 174),
// 				// reward 1 and their delegators
// 				Event::Rewarded(1, 868),
// 				Event::Rewarded(7, 139),
// 				Event::Rewarded(8, 416),
// 				Event::Rewarded(10, 139),
// 				// reward 2 and their delegators
// 				Event::Rewarded(2, 521),
// 				Event::Rewarded(8, 208),
// 				Event::Rewarded(9, 208),
// 				// reward 11
// 				Event::Rewarded(11, 87),
// 				// round 12 initiation
// 				Event::CollatorChosen(13, 1, 20_000_000, 50_000_000),
// 				Event::CollatorChosen(13, 11, 30_000_000, 20_000_000),
// 				Event::CollatorChosen(13, 5, 10_000_000, 30_000_000),
// 				Event::CollatorChosen(13, 2, 20_000_000, 20_000_000),
// 				Event::CollatorChosen(13, 4, 20_000_000, 0),
// 				Event::NewRound(12 * blocks_per_round, 13, 5, 100_000_000, 120_000_000),
// 			];
// 			expected.append(&mut round_11_to_12);
// 			assert_eq!(events(), expected);

// 			// Round 12 -> 13
// 			set_author(12, 1, 50);
// 			set_author(12, 2, 30);
// 			set_author(12, 4, 10);
// 			set_author(12, 5, 5);
// 			set_author(12, 11, 5);
// 			roll_to(13 * blocks_per_round + 1, vec![]);
// 			// delegation of 8 should not be rewarded for this round
// 			// delegation of 9 should be rewarded from now on
// 			let mut round_12_to_13 = vec![
// 				Event::Rewarded(5, 87),
// 				Event::Rewarded(4, 174),
// 				// reward 1 and their delegators
// 				Event::Rewarded(1, 868),
// 				Event::Rewarded(7, 139),
// 				Event::Rewarded(8, 416),
// 				Event::Rewarded(10, 139),
// 				// reward 2 and their delegators
// 				Event::Rewarded(2, 521),
// 				Event::Rewarded(8, 208),
// 				Event::Rewarded(9, 208),
// 				// reward 11 and their delegators
// 				Event::Rewarded(11, 87),
// 				Event::Rewarded(8, 69),
// 				// round 14 initiation
// 				Event::CollatorChosen(14, 1, 20_000_000, 50_000_000),
// 				Event::CollatorChosen(14, 11, 30_000_000, 20_000_000),
// 				Event::CollatorChosen(14, 5, 10_000_000, 30_000_000),
// 				Event::CollatorChosen(14, 2, 20_000_000, 20_000_000),
// 				Event::CollatorChosen(14, 4, 20_000_000, 0),
// 				Event::NewRound(13 * blocks_per_round, 14, 5, 100_000_000, 120_000_000),
// 			];
// 			expected.append(&mut round_12_to_13);
// 			assert_eq!(events(), expected);

// 			// Round 13 -> 14
// 			set_author(13, 1, 20);
// 			set_author(13, 2, 20);
// 			set_author(13, 4, 20);
// 			set_author(13, 5, 20);
// 			set_author(13, 11, 20);
// 			roll_to(14 * blocks_per_round + 1, vec![]);
// 			// delegation of 8 should not be rewarded for this round
// 			// delegation of 9 should be rewarded from now on
// 			let mut round_13_to_14 = vec![
// 				// reward 5 and their delegators
// 				Event::Rewarded(5, 347),
// 				Event::Rewarded(9, 278),
// 				// reward 4
// 				Event::Rewarded(4, 347),
// 				// reward 1 and their delegators
// 				Event::Rewarded(1, 347),
// 				Event::Rewarded(7, 56),
// 				Event::Rewarded(8, 167),
// 				Event::Rewarded(10, 56),
// 				// reward 2 and their delegators
// 				Event::Rewarded(2, 347),
// 				Event::Rewarded(8, 139),
// 				Event::Rewarded(9, 139),
// 				// reward 11
// 				Event::Rewarded(11, 347),
// 				Event::Rewarded(8, 278),
// 				// round 15 initiation
// 				Event::CollatorChosen(15, 1, 20_000_000, 50_000_000),
// 				Event::CollatorChosen(15, 11, 30_000_000, 20_000_000),
// 				Event::CollatorChosen(15, 5, 10_000_000, 30_000_000),
// 				Event::CollatorChosen(15, 2, 20_000_000, 20_000_000),
// 				Event::CollatorChosen(15, 4, 20_000_000, 0),
// 				Event::NewRound(14 * blocks_per_round, 15, 5, 100_000_000, 120_000_000),
// 			];
// 			expected.append(&mut round_13_to_14);
// 			assert_eq!(events(), expected);
// 		});
// }

// FIXME:
// #[test]
// fn set_inflation() {
// 	ExtBuilder::default().build().execute_with(|| {
// 		// check that Perbill is limited to 100%
// 		let capped_inflation = InflationInfo {
// 			collator: StakingInfo {
// 				max_rate: Perquintill::from_percent(200),
// 				reward_rate: RewardRate {
// 					annual: Perquintill::from_percent(50),
// 					per_block: Perbill::from_parts(2_000_000_000u32),
// 				},
// 			},
// 			delegator: StakingInfo {
// 				max_rate: Perquintill::from_percent(200),
// 				reward_rate: RewardRate {
// 					annual: Perquintill::from_percent(200),
// 					per_block: Perquintill::from_percent(200),
// 				},
// 			},
// 		};
// 		let mut inflation = InflationInfo {
// 			collator: StakingInfo {
// 				max_rate: Perbill::one(),
// 				reward_rate: RewardRate {
// 					annual: Perquintill::from_percent(50),
// 					per_block: Perbill::one(),
// 				},
// 			},
// 			delegator: StakingInfo {
// 				max_rate: Perbill::one(),
// 				reward_rate: RewardRate {
// 					annual: Perbill::one(),
// 					per_block: Perbill::one(),
// 				},
// 			},
// 		};
// 		assert_eq!(capped_inflation, inflation);

// 		// annual < round
// 		assert_noop!(
// 			Stake::set_inflation(Origin::root(), inflation.clone()),
// 			Error::<Test>::InvalidSchedule
// 		);
// 		inflation.collator.reward_rate.annual = Perbill::one();
// 		assert_ok!(Stake::set_inflation(Origin::root(), inflation.clone()));
// 		assert_eq!(
// 			last_event(),
// 			MetaEvent::stake(Event::RoundInflationSet(
// 				Perbill::one(),
// 				Perbill::one(),
// 				Perbill::one(),
// 				Perbill::one()
// 			))
// 		);

// 		// annual < round
// 		inflation.delegator.reward_rate.annual = Perquintill::from_percent(50);
// 		assert_noop!(
// 			Stake::set_inflation(Origin::root(), inflation.clone()),
// 			Error::<Test>::InvalidSchedule
// 		);
// 		inflation.delegator.reward_rate.per_block = Perquintill::from_percent(15);
// 		assert_ok!(Stake::set_inflation(Origin::root(), inflation));
// 		assert_eq!(
// 			last_event(),
// 			MetaEvent::stake(Event::RoundInflationSet(
// 				Perbill::one(),
// 				Perbill::one(),
// 				Perbill::one(),
// 				Perquintill::from_percent(15)
// 			))
// 		);
// 	});
// }

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
			assert_eq!(inflation, Stake::inflation_config());
			roll_to(5, vec![]);
			let init = vec![Event::CollatorChosen(1, 1, 20, 20), Event::NewRound(5, 1, 1, 20, 20)];
			assert_eq!(events(), init);
			assert_ok!(Stake::set_blocks_per_round(Origin::root(), 3));
			assert_eq!(last_event(), MetaEvent::stake(Event::BlocksPerRoundSet(1, 5, 5, 3)));

			// inflation config should be untouched after per_block update
			assert_eq!(inflation, Stake::inflation_config());

			// last round startet at 5 but we are already at 9, so we expect 9 to be the new
			// round
			roll_to(8, vec![]);
			assert_eq!(last_event(), MetaEvent::stake(Event::NewRound(8, 2, 1, 20, 20)));
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
			assert_eq!(inflation, Stake::inflation_config());
			// Default round every 5 blocks, but MinBlocksPerRound is 3 and we set it to min
			// 3 blocks
			roll_to(6, vec![]);
			// chooses top TotalSelectedCandidates (5), in order
			let init = vec![Event::CollatorChosen(1, 1, 20, 20), Event::NewRound(5, 1, 1, 20, 20)];
			assert_eq!(events(), init);
			assert_ok!(Stake::set_blocks_per_round(Origin::root(), 3));
			assert_eq!(last_event(), MetaEvent::stake(Event::BlocksPerRoundSet(1, 5, 5, 3)));

			// inflation config should be untouched after per_block update
			assert_eq!(inflation, Stake::inflation_config());

			// there should not be a new event
			roll_to(7, vec![]);
			assert_eq!(last_event(), MetaEvent::stake(Event::BlocksPerRoundSet(1, 5, 5, 3)));

			roll_to(8, vec![]);
			assert_eq!(last_event(), MetaEvent::stake(Event::NewRound(8, 2, 1, 20, 20)));
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
			assert_eq!(inflation, Stake::inflation_config());
			roll_to(7, vec![]);
			// chooses top TotalSelectedCandidates (5), in order
			let init = vec![Event::CollatorChosen(1, 1, 20, 20), Event::NewRound(5, 1, 1, 20, 20)];
			assert_eq!(events(), init);
			assert_ok!(Stake::set_blocks_per_round(Origin::root(), 3));

			// inflation config should be untouched after per_block update
			assert_eq!(inflation, Stake::inflation_config());

			assert_eq!(
				Stake::inflation_config(),
				InflationInfo::new(
					Perquintill::from_percent(col_max),
					Perquintill::from_percent(col_rewards),
					Perquintill::from_percent(d_max),
					Perquintill::from_percent(d_rewards)
				)
			);
			assert_eq!(last_event(), MetaEvent::stake(Event::BlocksPerRoundSet(1, 5, 5, 3)));
			roll_to(8, vec![]);

			// last round startet at 5, so we expect 8 to be the new round
			assert_eq!(last_event(), MetaEvent::stake(Event::NewRound(8, 2, 1, 20, 20)));
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
			assert_eq!(Authorship::author(), 1337);
			let inflation = Stake::inflation_config();
			let total_issuance = <Test as Config>::Currency::total_issuance();
			assert_eq!(total_issuance, 160_000_000 * DECIMALS);
			let c_rewards: BalanceOf<Test> = inflation
				.collator
				.compute_block_rewards::<Test>(16_000_000 * DECIMALS, total_issuance);
			let d_rewards: BalanceOf<Test> = inflation
				.delegator
				.compute_block_rewards::<Test>(64_000_000 * DECIMALS, total_issuance);
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
			assert_eq!(Balances::usable_balance(&3), user_3 + d_rewards * 2 / 3);
			assert_eq!(Balances::usable_balance(&4), user_4 + d_rewards / 3 + 1);
			assert_eq!(Balances::usable_balance(&5), user_5);

			// 1 is block author for 2nd block
			roll_to(3, authors.clone());
			assert_eq!(Balances::usable_balance(&1), user_1 + 2 * c_rewards);
			assert_eq!(Balances::usable_balance(&2), user_2);
			assert_eq!(Balances::usable_balance(&3), user_3 + d_rewards * 4 / 3);
			assert_eq!(Balances::usable_balance(&4), user_4 + d_rewards * 2 / 3 + 1);
			assert_eq!(Balances::usable_balance(&5), user_5);

			// 1 is block author for 3rd block
			roll_to(4, authors.clone());
			assert_eq!(Balances::usable_balance(&1), user_1 + 3 * c_rewards);
			assert_eq!(Balances::usable_balance(&2), user_2);
			assert_eq!(Balances::usable_balance(&3), user_3 + d_rewards * 2 - 1);
			assert_eq!(Balances::usable_balance(&4), user_4 + d_rewards + 1);
			assert_eq!(Balances::usable_balance(&5), user_5);

			// 2 is block author for 4th block
			roll_to(5, authors.clone());
			assert_eq!(Balances::usable_balance(&1), user_1 + 3 * c_rewards);
			assert_eq!(Balances::usable_balance(&2), user_2 + c_rewards);
			assert_eq!(Balances::usable_balance(&3), user_3 + d_rewards * 2 - 1);
			assert_eq!(Balances::usable_balance(&4), user_4 + d_rewards + 1);
			assert_eq!(Balances::usable_balance(&5), user_5 + d_rewards);

			// 2 is block author for 5th block
			roll_to(6, authors);
			assert_eq!(Balances::usable_balance(&1), user_1 + 3 * c_rewards);
			assert_eq!(Balances::usable_balance(&2), user_2 + 2 * c_rewards);
			assert_eq!(Balances::usable_balance(&3), user_3 + d_rewards * 2 - 1);
			assert_eq!(Balances::usable_balance(&4), user_4 + d_rewards + 1);
			assert_eq!(Balances::usable_balance(&5), user_5 + 2 * d_rewards);
		});
}

#[test]
fn coinbase_rewards_many_blocks_simple_check() {
	let num_of_years: Perquintill = Perquintill::from_perthousand(1);
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
			assert_eq!(Authorship::author(), 1337);
			let inflation = Stake::inflation_config();
			let total_issuance = <Test as Config>::Currency::total_issuance();
			assert_eq!(total_issuance, 160_000_000 * DECIMALS);
			let end_block: BlockNumber = num_of_years * YEARS as BlockNumber;
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

			// collator rewards should be about the same
			assert!(almost_equal(rewards_1, rewards_2, Perbill::from_perthousand(1)));
			// delegator rewards should be about the same
			assert!(
				almost_equal(rewards_3 + rewards_4, rewards_5, Perbill::from_perthousand(1)),
				"left {:?}, right {:?}",
				rewards_3 + rewards_4,
				rewards_5
			);
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

// Could only occur if we increase MinDelegatorStk via runtime upgrade and don't
// migrate delegators which fall below minimum
#[test]
fn should_not_reward_delegators_below_min_stake() {
	ExtBuilder::default()
		.with_balances(vec![(1, 10 * DECIMALS), (2, 10 * DECIMALS), (3, 10 * DECIMALS), (4, 5)])
		.with_collators(vec![(1, 10 * DECIMALS), (2, 10 * DECIMALS)])
		.with_delegators(vec![(3, 2, 10 * DECIMALS), (4, 1, 4)])
		.with_inflation(10, 15, 40, 15, 5)
		.build()
		.execute_with(|| {
			assert!(Stake::delegator_state(3).is_some());
			assert!(Stake::delegator_state(4).is_none());

			// impossible but lets assume it happened
			let mut state = Stake::collator_state(&1).expect("CollatorState cannot be missing");
			let delegator_stake_below_min = <Test as Config>::MinDelegatorStk::get() - 1;
			state.bond += delegator_stake_below_min;
			state.total += delegator_stake_below_min;
			let impossible_bond = Bond {
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
				Stake::candidate_pool().len() as u32,
				<Test as Config>::MaxCollatorCandidates::get()
			);
			assert_noop!(
				Stake::join_candidates(Origin::signed(11), 10),
				Error::<Test>::TooManyCollatorCandidates
			);
		});
}

// #[test]
// // 1/4 year with 12h rounds, 80 collators and 80 delegators
// // TODO: Remove or fix long execution time
// fn collator_delegator_rewards_quarter_year_12h() {
// 	// unfortunately, collator_stake at 100_000 leads to stack overflow
// 	check_yearly_inflation(
// 		800_000 * DECIMALS,
// 		400_000 * DECIMALS,
// 		800_000 * DECIMALS,
// 		10,
// 		15,
// 		40,
// 		10,
// 		7200,
// 		Perbill::from_percent(25),
// 	);
// }

// #[test]
// // full year with 12h rounds, 4 collators and 26 delegators
// // TODO: Remove or fix long execution time
// fn collator_delegator_rewards_full_year_24h() {
// 	check_yearly_inflation(
// 		16_000_000 * DECIMALS,
// 		16_000_000 * DECIMALS,
// 		16_000_000 * DECIMALS,
// 		10,
// 		15,
// 		40,
// 		10,
// 		5,
// 		Perbill::from_percent(1),
// 	);
// }

#[test]
fn should_estimate_current_session_progress() {
	ExtBuilder::default()
		.set_blocks_per_round(100)
		.build()
		.execute_with(|| {
			assert_eq!(
				Stake::estimate_current_session_progress(10).0.unwrap(),
				Percent::from_percent(10)
			);
			assert_eq!(
				Stake::estimate_current_session_progress(20).0.unwrap(),
				Percent::from_percent(20)
			);
			assert_eq!(
				Stake::estimate_current_session_progress(30).0.unwrap(),
				Percent::from_percent(30)
			);
			assert_eq!(
				Stake::estimate_current_session_progress(60).0.unwrap(),
				Percent::from_percent(60)
			);
			assert_eq!(
				Stake::estimate_current_session_progress(100).0.unwrap(),
				Percent::from_percent(100)
			);
		});
}

#[test]
fn should_estimate_next_session_rotation() {
	ExtBuilder::default()
		.set_blocks_per_round(100)
		.build()
		.execute_with(|| {
			assert_eq!(Stake::estimate_next_session_rotation(10).0.unwrap(), 100);
			assert_eq!(Stake::estimate_next_session_rotation(20).0.unwrap(), 100);
			assert_eq!(Stake::estimate_next_session_rotation(30).0.unwrap(), 100);
			assert_eq!(Stake::estimate_next_session_rotation(60).0.unwrap(), 100);
			assert_eq!(Stake::estimate_next_session_rotation(100).0.unwrap(), 100);
		});
}

#[test]
fn should_end_session_when_appropriate() {
	ExtBuilder::default()
		.set_blocks_per_round(100)
		.build()
		.execute_with(|| {
			assert_eq!(Stake::should_end_session(10), false);
			assert_eq!(Stake::should_end_session(20), false);
			assert_eq!(Stake::should_end_session(30), false);
			assert_eq!(Stake::should_end_session(60), false);
			assert_eq!(Stake::should_end_session(100), true);
		});
}
