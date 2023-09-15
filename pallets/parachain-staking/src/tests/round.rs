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

use frame_support::{assert_noop, assert_ok, traits::fungible::Inspect};
use sp_runtime::Perquintill;

use crate::{
	mock::{
		events, last_event, roll_to, roll_to_claim_rewards, AccountId, Balances, ExtBuilder, RuntimeOrigin, Session,
		StakePallet, Test, DECIMALS,
	},
	types::RoundInfo,
	Config, Error, Event, Event as StakeEvent, InflationInfo,
};

#[test]
fn round_transitions() {
	let col_max = 10;
	let col_rewards = 15;
	let d_max = 40;
	let d_rewards = 10;
	let inflation = InflationInfo::new(
		<Test as Config>::BLOCKS_PER_YEAR,
		Perquintill::from_percent(col_max),
		Perquintill::from_percent(col_rewards),
		Perquintill::from_percent(d_max),
		Perquintill::from_percent(d_rewards),
	);

	// round_immediately_jumps_if_current_duration_exceeds_new_blocks_per_round
	// change from 5 bpr to 3 in block 5 -> 8 should be new round
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
		.with_collators(vec![(1, 20), (7, 10)])
		.with_delegators(vec![(2, 1, 10), (3, 1, 10)])
		.with_inflation(col_max, col_rewards, d_max, d_rewards, 5)
		.build_and_execute_with_sanity_tests(|| {
			assert_eq!(inflation, StakePallet::inflation_config());
			roll_to(5, vec![]);
			let init = vec![Event::NewRound(5, 1)];
			assert_eq!(events(), init);
			assert_ok!(StakePallet::set_blocks_per_round(RuntimeOrigin::root(), 3));
			assert_noop!(
				StakePallet::set_blocks_per_round(RuntimeOrigin::root(), 1),
				Error::<Test>::CannotSetBelowMin
			);
			assert_eq!(last_event(), StakeEvent::BlocksPerRoundSet(1, 5, 5, 3));

			// inflation config should be untouched after per_block update
			assert_eq!(inflation, StakePallet::inflation_config());

			// last round startet at 5 but we are already at 9, so we expect 9 to be the new
			// round
			roll_to(8, vec![]);
			assert_eq!(last_event(), StakeEvent::NewRound(8, 2))
		});

	// if duration of current round is less than new bpr, round waits until new bpr
	// passes
	// change from 5 bpr to 3 in block 6 -> 8 should be new round
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
		.with_collators(vec![(1, 20), (7, 10)])
		.with_delegators(vec![(2, 1, 10), (3, 1, 10)])
		.with_inflation(col_max, col_rewards, d_max, d_rewards, 5)
		.build_and_execute_with_sanity_tests(|| {
			assert_eq!(inflation, StakePallet::inflation_config());
			// Default round every 5 blocks, but MinBlocksPerRound is 3 and we set it to min
			// 3 blocks
			roll_to(6, vec![]);
			// chooses top MaxSelectedCandidates (5), in order
			let init = vec![Event::NewRound(5, 1)];
			assert_eq!(events(), init);
			assert_ok!(StakePallet::set_blocks_per_round(RuntimeOrigin::root(), 3));
			assert_eq!(last_event(), StakeEvent::BlocksPerRoundSet(1, 5, 5, 3));

			// inflation config should be untouched after per_block update
			assert_eq!(inflation, StakePallet::inflation_config());

			// there should not be a new event
			roll_to(7, vec![]);
			assert_eq!(last_event(), StakeEvent::BlocksPerRoundSet(1, 5, 5, 3));

			roll_to(8, vec![]);
			assert_eq!(last_event(), StakeEvent::NewRound(8, 2))
		});

	// round_immediately_jumps_if_current_duration_exceeds_new_blocks_per_round
	// change from 5 bpr (blocks_per_round) to 3 in block 7 -> 8 should be new round
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
		.with_collators(vec![(1, 20), (7, 10)])
		.with_delegators(vec![(2, 1, 10), (3, 1, 10)])
		.with_inflation(col_max, col_rewards, d_max, d_rewards, 5)
		.build_and_execute_with_sanity_tests(|| {
			// Default round every 5 blocks, but MinBlocksPerRound is 3 and we set it to min
			// 3 blocks
			assert_eq!(inflation, StakePallet::inflation_config());
			roll_to(7, vec![]);
			// chooses top MaxSelectedCandidates (5), in order
			let init = vec![Event::NewRound(5, 1)];
			assert_eq!(events(), init);
			assert_ok!(StakePallet::set_blocks_per_round(RuntimeOrigin::root(), 3));

			// inflation config should be untouched after per_block update
			assert_eq!(inflation, StakePallet::inflation_config());

			assert_eq!(
				StakePallet::inflation_config(),
				InflationInfo::new(
					<Test as Config>::BLOCKS_PER_YEAR,
					Perquintill::from_percent(col_max),
					Perquintill::from_percent(col_rewards),
					Perquintill::from_percent(d_max),
					Perquintill::from_percent(d_rewards)
				)
			);
			assert_eq!(last_event(), StakeEvent::BlocksPerRoundSet(1, 5, 5, 3));
			roll_to(8, vec![]);

			// last round startet at 5, so we expect 8 to be the new round
			assert_eq!(last_event(), StakeEvent::NewRound(8, 2))
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
		.build_and_execute_with_sanity_tests(|| {
			assert_eq!(StakePallet::selected_candidates().into_inner(), vec![1, 2]);
			// reward 1 once per round
			let authors: Vec<Option<AccountId>> = (0u64..=100)
				.map(|i| if i % 5 == 2 { Some(1u64) } else { None })
				.collect();
			let inflation = StakePallet::inflation_config();

			// roll to last block of round 0
			roll_to_claim_rewards(4, authors.clone());
			let reward_0 = inflation.collator.reward_rate.per_block * stake * 2;
			assert_eq!(Balances::balance(&1), stake + reward_0);
			// increase max selected candidates which will become effective in round 2
			assert_ok!(StakePallet::set_max_selected_candidates(RuntimeOrigin::root(), 10));

			// roll to last block of round 1
			// should still multiply with 2 because the Authority set was chosen at start of
			// round 1
			roll_to_claim_rewards(9, authors.clone());
			let reward_1 = inflation.collator.reward_rate.per_block * stake * 2;
			assert_eq!(Balances::balance(&1), stake + reward_0 + reward_1);

			// roll to last block of round 2
			// should multiply with 4 because there are only 4 candidates
			roll_to_claim_rewards(14, authors.clone());
			let reward_2 = inflation.collator.reward_rate.per_block * stake * 4;
			assert_eq!(Balances::balance(&1), stake + reward_0 + reward_1 + reward_2);

			// roll to last block of round 3
			// should multiply with 4 because there are only 4 candidates
			roll_to_claim_rewards(19, authors);
			let reward_3 = inflation.collator.reward_rate.per_block * stake * 4;
			assert_eq!(Balances::balance(&1), stake + reward_0 + reward_1 + reward_2 + reward_3);
		});
}

#[test]
fn force_new_round() {
	ExtBuilder::default()
		.with_balances(vec![(1, 100), (2, 100), (3, 100), (4, 100), (5, 100), (6, 100)])
		.with_collators(vec![(1, 100), (2, 100), (3, 100), (4, 100)])
		.build_and_execute_with_sanity_tests(|| {
			let mut round = RoundInfo {
				current: 0,
				first: 0,
				length: 5,
			};
			assert_eq!(StakePallet::round(), round);
			assert_eq!(Session::validators(), vec![1, 2]);
			assert_eq!(Session::current_index(), 0);
			// 3 should be validator in round 2
			assert_ok!(StakePallet::join_delegators(RuntimeOrigin::signed(5), 3, 100));

			// init force new round from 0 to 1, updating the authorities
			assert_ok!(StakePallet::force_new_round(RuntimeOrigin::root()));
			assert_eq!(StakePallet::round(), round);
			assert_eq!(Session::current_index(), 0);
			assert!(StakePallet::new_round_forced());

			// force new round should become active by starting next block
			roll_to(2, vec![]);
			round = RoundInfo {
				current: 1,
				first: 2,
				length: 5,
			};
			assert_eq!(Session::current_index(), 1);
			assert_eq!(Session::validators(), vec![1, 2]);
			assert!(!StakePallet::new_round_forced());

			// roll to next block in same round 1
			roll_to(3, vec![]);
			assert_eq!(Session::current_index(), 1);
			assert_eq!(StakePallet::round(), round);
			// assert_eq!(Session::validators(), vec![3, 1]);
			assert!(!StakePallet::new_round_forced());
			// 4 should become validator in session 3 if we do not force a new round
			assert_ok!(StakePallet::join_delegators(RuntimeOrigin::signed(6), 4, 100));

			// end session 2 naturally
			roll_to(7, vec![]);
			round = RoundInfo {
				current: 2,
				first: 7,
				length: 5,
			};
			assert_eq!(StakePallet::round(), round);
			assert_eq!(Session::current_index(), 2);
			assert!(!StakePallet::new_round_forced());
			assert_eq!(Session::validators(), vec![3, 1]);

			// force new round 3
			assert_ok!(StakePallet::force_new_round(RuntimeOrigin::root()));
			assert_eq!(StakePallet::round(), round);
			assert_eq!(Session::current_index(), 2);
			// validator set should not change until next round
			assert_eq!(Session::validators(), vec![3, 1]);
			assert!(StakePallet::new_round_forced());

			// force new round should become active by starting next block
			roll_to(8, vec![]);
			round = RoundInfo {
				current: 3,
				first: 8,
				length: 5,
			};
			assert_eq!(Session::current_index(), 3);
			assert_eq!(StakePallet::round(), round);
			assert_eq!(Session::validators(), vec![3, 4]);
			assert!(!StakePallet::new_round_forced());
		});
}
