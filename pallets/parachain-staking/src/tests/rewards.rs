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
use pallet_authorship::EventHandler;
use sp_runtime::{traits::Zero, Perbill, Perquintill};

use crate::{
	mock::{
		almost_equal, roll_to, roll_to_claim_rewards, AccountId, Balance, Balances, BlockNumber, ExtBuilder,
		RuntimeOrigin, StakePallet, System, Test, DECIMALS, TREASURY_ACC,
	},
	types::{BalanceOf, StakeOf},
	Config, Error, InflationInfo,
};

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
		.build_and_execute_with_sanity_tests(|| {
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
			let user_1 = Balances::usable_balance(1);
			let user_2 = Balances::usable_balance(2);
			let user_3 = Balances::usable_balance(3);
			let user_4 = Balances::usable_balance(4);
			let user_5 = Balances::usable_balance(5);

			assert_eq!(Balances::usable_balance(1), user_1);
			assert_eq!(Balances::usable_balance(2), user_2);
			assert_eq!(Balances::usable_balance(3), user_3);
			assert_eq!(Balances::usable_balance(4), user_4);
			assert_eq!(Balances::usable_balance(5), user_5);

			// 1 is block author for 1st block
			roll_to_claim_rewards(2, authors.clone());
			assert_eq!(Balances::usable_balance(1), user_1 + c_rewards);
			assert_eq!(Balances::usable_balance(2), user_2);
			assert_eq!(Balances::usable_balance(3), user_3 + d_rewards / 2);
			assert_eq!(Balances::usable_balance(4), user_4 + d_rewards / 4);
			assert_eq!(Balances::usable_balance(5), user_5);

			// 1 is block author for 2nd block
			roll_to_claim_rewards(3, authors.clone());
			assert_eq!(Balances::usable_balance(1), user_1 + 2 * c_rewards);
			assert_eq!(Balances::usable_balance(2), user_2);
			assert_eq!(Balances::usable_balance(3), user_3 + d_rewards);
			assert_eq!(Balances::usable_balance(4), user_4 + d_rewards / 2);
			assert_eq!(Balances::usable_balance(5), user_5);

			// 1 is block author for 3rd block
			roll_to_claim_rewards(4, authors.clone());
			assert_eq!(Balances::usable_balance(1), user_1 + 3 * c_rewards);
			assert_eq!(Balances::usable_balance(2), user_2);
			assert_eq!(Balances::usable_balance(3), user_3 + d_rewards / 2 * 3);
			assert_eq!(Balances::usable_balance(4), user_4 + d_rewards / 4 * 3);
			assert_eq!(Balances::usable_balance(5), user_5);

			// 2 is block author for 4th block
			roll_to_claim_rewards(5, authors.clone());
			assert_eq!(Balances::usable_balance(1), user_1 + 3 * c_rewards);
			assert_eq!(Balances::usable_balance(2), user_2 + c_rewards);
			assert_eq!(Balances::usable_balance(3), user_3 + d_rewards / 2 * 3);
			assert_eq!(Balances::usable_balance(4), user_4 + d_rewards / 4 * 3);
			assert_eq!(Balances::usable_balance(5), user_5 + d_rewards / 4);
			assert_ok!(StakePallet::leave_delegators(RuntimeOrigin::signed(5)));

			// 2 is block author for 5th block
			roll_to_claim_rewards(6, authors);
			assert_eq!(Balances::usable_balance(1), user_1 + 3 * c_rewards);
			assert_eq!(Balances::usable_balance(2), user_2 + 2 * c_rewards);
			assert_eq!(Balances::usable_balance(3), user_3 + d_rewards / 2 * 3);
			assert_eq!(Balances::usable_balance(4), user_4 + d_rewards / 4 * 3);
			// should not receive rewards due to revoked delegation
			assert_eq!(Balances::usable_balance(5), user_5 + d_rewards / 4);
		});
}

#[test]
fn delegator_should_not_receive_rewards_after_revoking() {
	// test edge case of 1 delegator
	ExtBuilder::default()
		.with_balances(vec![(1, 10_000_000 * DECIMALS), (2, 10_000_000 * DECIMALS), (3, 100)])
		.with_collators(vec![(1, 10_000_000 * DECIMALS), (3, 10)])
		.with_delegators(vec![(2, 1, 10_000_000 * DECIMALS)])
		.with_inflation(10, 15, 40, 15, 5)
		.build_and_execute_with_sanity_tests(|| {
			assert_ok!(StakePallet::leave_delegators(RuntimeOrigin::signed(2)));
			let authors: Vec<Option<AccountId>> = (1u64..100u64).map(|_| Some(1u64)).collect();
			assert_eq!(Balances::usable_balance(1), Balance::zero());
			assert_eq!(Balances::usable_balance(2), Balance::zero());
			roll_to_claim_rewards(100, authors);
			assert!(Balances::usable_balance(1) > Balance::zero());
			assert_ok!(StakePallet::unlock_unstaked(RuntimeOrigin::signed(2), 2));
			assert_eq!(Balances::usable_balance(2), 10_000_000 * DECIMALS);
		});

	ExtBuilder::default()
		.with_balances(vec![
			(1, 10_000_000 * DECIMALS),
			(2, 10_000_000 * DECIMALS),
			(3, 10_000_000 * DECIMALS),
			(4, 100),
		])
		.with_collators(vec![(1, 10_000_000 * DECIMALS), (4, 10)])
		.with_delegators(vec![(2, 1, 10_000_000 * DECIMALS), (3, 1, 10_000_000 * DECIMALS)])
		.with_inflation(10, 15, 40, 15, 5)
		.build_and_execute_with_sanity_tests(|| {
			assert_ok!(StakePallet::leave_delegators(RuntimeOrigin::signed(3)));
			let authors: Vec<Option<AccountId>> = (1u64..100u64).map(|_| Some(1u64)).collect();
			assert_eq!(Balances::usable_balance(1), Balance::zero());
			assert_eq!(Balances::usable_balance(2), Balance::zero());
			assert_eq!(Balances::usable_balance(3), Balance::zero());
			roll_to_claim_rewards(100, authors);
			assert!(Balances::usable_balance(1) > Balance::zero());
			assert!(Balances::usable_balance(2) > Balance::zero());
			assert_ok!(StakePallet::unlock_unstaked(RuntimeOrigin::signed(3), 3));
			assert_eq!(Balances::usable_balance(3), 10_000_000 * DECIMALS);
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
		.build_and_execute_with_sanity_tests(|| {
			let inflation = StakePallet::inflation_config();
			let total_issuance = <Test as Config>::Currency::total_issuance();
			assert_eq!(total_issuance, 160_000_000 * DECIMALS);
			let end_block: BlockNumber = num_of_years * Test::BLOCKS_PER_YEAR as BlockNumber;
			// set round robin authoring
			let authors: Vec<Option<AccountId>> = (0u64..=end_block).map(|i| Some(i % 2 + 1)).collect();
			roll_to_claim_rewards(end_block, authors);

			let rewards_1 = Balances::balance(&1).saturating_sub(40_000_000 * DECIMALS);
			let rewards_2 = Balances::balance(&2).saturating_sub(40_000_000 * DECIMALS);
			let rewards_3 = Balances::balance(&3).saturating_sub(40_000_000 * DECIMALS);
			let rewards_4 = Balances::balance(&4).saturating_sub(20_000_000 * DECIMALS);
			let rewards_5 = Balances::balance(&5).saturating_sub(20_000_000 * DECIMALS);
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

// Could only occur if we increase MinDelegatorStakeOf::<Test>via runtime
// upgrade and don't migrate delegators which fall below minimum
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
			let mut state = StakePallet::candidate_pool(1).expect("CollatorState cannot be missing");
			let delegator_stake_below_min = <Test as Config>::MinDelegatorStake::get() - 1;
			state.stake += delegator_stake_below_min;
			state.total += delegator_stake_below_min;
			let impossible_bond = StakeOf::<Test> {
				owner: 4u64,
				amount: delegator_stake_below_min,
			};
			assert_eq!(state.delegators.try_insert(impossible_bond), Ok(true));
			<crate::CandidatePool<Test>>::insert(1u64, state);

			let authors: Vec<Option<AccountId>> = vec![Some(1u64), Some(1u64), Some(1u64), Some(1u64)];
			assert_eq!(Balances::usable_balance(1), Balance::zero());
			assert_eq!(Balances::usable_balance(2), Balance::zero());
			assert_eq!(Balances::usable_balance(3), Balance::zero());
			assert_eq!(Balances::usable_balance(4), 5);

			// should only reward 1
			roll_to_claim_rewards(4, authors);
			assert!(Balances::usable_balance(1) > Balance::zero());
			assert_eq!(Balances::usable_balance(4), 5);
			assert_eq!(Balances::usable_balance(2), Balance::zero());
			assert_eq!(Balances::usable_balance(3), Balance::zero());
		});
}

#[test]
fn adjust_reward_rates() {
	ExtBuilder::default()
		.with_balances(vec![(1, 10_000_000 * DECIMALS), (2, 90_000_000 * DECIMALS), (3, 100)])
		.with_collators(vec![(1, 10_000_000 * DECIMALS), (3, 10)])
		.with_delegators(vec![(2, 1, 40_000_000 * DECIMALS)])
		.with_inflation(10, 10, 40, 8, 5)
		.build_and_execute_with_sanity_tests(|| {
			let inflation_0 = StakePallet::inflation_config();
			let num_of_years = 3 * <Test as Config>::BLOCKS_PER_YEAR;
			// 1 authors every block
			let authors: Vec<Option<AccountId>> = (0u64..=num_of_years).map(|_| Some(1u64)).collect();

			// reward once in first year
			roll_to_claim_rewards(2, authors.clone());
			let c_rewards_0 = Balances::balance(&1).saturating_sub(10_000_000 * DECIMALS);
			let d_rewards_0 = Balances::balance(&2).saturating_sub(90_000_000 * DECIMALS);
			assert!(!c_rewards_0.is_zero());
			assert!(!d_rewards_0.is_zero());

			// finish first year
			System::set_block_number(<Test as Config>::BLOCKS_PER_YEAR);
			roll_to_claim_rewards(<Test as Config>::BLOCKS_PER_YEAR + 1, vec![]);
			// reward reduction should not happen automatically anymore
			assert_eq!(StakePallet::last_reward_reduction(), 0u64);
			assert_ok!(StakePallet::execute_scheduled_reward_change(RuntimeOrigin::signed(1)));
			assert_eq!(StakePallet::last_reward_reduction(), 1u64);
			let inflation_1 = InflationInfo::new(
				<Test as Config>::BLOCKS_PER_YEAR,
				inflation_0.collator.max_rate,
				Perquintill::from_parts(98000000000000000),
				inflation_0.delegator.max_rate,
				Perquintill::from_percent(6),
			);
			assert_eq!(StakePallet::inflation_config(), inflation_1);
			// reward once in 2nd year
			roll_to_claim_rewards(<Test as Config>::BLOCKS_PER_YEAR + 2, authors.clone());
			let c_rewards_1 = Balances::balance(&1)
				.saturating_sub(10_000_000 * DECIMALS)
				.saturating_sub(c_rewards_0);
			let d_rewards_1 = Balances::balance(&2)
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
			System::set_block_number(2 * <Test as Config>::BLOCKS_PER_YEAR);
			roll_to_claim_rewards(2 * <Test as Config>::BLOCKS_PER_YEAR + 1, vec![]);
			// reward reduction should not happen automatically anymore
			assert_eq!(StakePallet::last_reward_reduction(), 1u64);
			assert_ok!(StakePallet::execute_scheduled_reward_change(RuntimeOrigin::signed(1)));
			assert_eq!(StakePallet::last_reward_reduction(), 2u64);
			let inflation_2 = InflationInfo::new(
				<Test as Config>::BLOCKS_PER_YEAR,
				inflation_0.collator.max_rate,
				Perquintill::from_parts(96040000000000000),
				inflation_0.delegator.max_rate,
				Perquintill::from_float(0.051),
			);
			assert_eq!(StakePallet::inflation_config(), inflation_2);
			// reward once in 3rd year
			roll_to_claim_rewards(2 * <Test as Config>::BLOCKS_PER_YEAR + 2, authors.clone());
			let c_rewards_2 = Balances::balance(&1)
				.saturating_sub(10_000_000 * DECIMALS)
				.saturating_sub(c_rewards_0)
				.saturating_sub(c_rewards_1);
			assert!(c_rewards_1 > c_rewards_2);
			// should be zero because we set reward rate to zero
			let d_rewards_2 = Balances::balance(&2)
				.saturating_sub(90_000_000 * DECIMALS)
				.saturating_sub(d_rewards_0)
				.saturating_sub(d_rewards_1);
			assert!(!d_rewards_2.is_zero());
			assert!(d_rewards_2 < d_rewards_1);

			// finish 3rd year
			System::set_block_number(3 * <Test as Config>::BLOCKS_PER_YEAR);
			roll_to_claim_rewards(3 * <Test as Config>::BLOCKS_PER_YEAR + 1, vec![]);
			// reward reduction should not happen automatically anymore
			assert_eq!(StakePallet::last_reward_reduction(), 2u64);
			assert_ok!(StakePallet::execute_scheduled_reward_change(RuntimeOrigin::signed(1)));
			assert_eq!(StakePallet::last_reward_reduction(), 3u64);
			let inflation_3 = InflationInfo::new(
				<Test as Config>::BLOCKS_PER_YEAR,
				inflation_0.collator.max_rate,
				Perquintill::from_parts(94119200000000000),
				inflation_0.delegator.max_rate,
				Perquintill::zero(),
			);
			assert_eq!(StakePallet::inflation_config(), inflation_3);
			// reward once in 4th year
			roll_to_claim_rewards(3 * <Test as Config>::BLOCKS_PER_YEAR + 2, authors);
			let c_rewards_3 = Balances::free_balance(1)
				.saturating_sub(10_000_000 * DECIMALS)
				.saturating_sub(c_rewards_0)
				.saturating_sub(c_rewards_1)
				.saturating_sub(c_rewards_2);
			// collator and delegator should not receive any rewards at all
			assert!(c_rewards_3.is_zero());

			let d_rewards_3 = Balances::free_balance(2)
				.saturating_sub(90_000_000 * DECIMALS)
				.saturating_sub(d_rewards_0)
				.saturating_sub(d_rewards_1)
				.saturating_sub(d_rewards_2);
			assert!(d_rewards_3.is_zero());
		});
}

#[test]
fn network_reward_multiple_blocks() {
	let max_stake: Balance = 160_000_000 * DECIMALS;
	let collators: Vec<(AccountId, Balance)> = (1u64..=<Test as Config>::MinCollators::get().saturating_add(1).into())
		.map(|acc_id| (acc_id, max_stake))
		.collect();

	ExtBuilder::default()
		.with_balances(collators.clone())
		.with_collators(collators)
		.build_and_execute_with_sanity_tests(|| {
			assert_eq!(max_stake, StakePallet::max_candidate_stake());
			let total_collator_stake = max_stake.saturating_mul(<Test as Config>::MinCollators::get().into());
			assert_eq!(total_collator_stake, StakePallet::total_collator_stake().collators);
			assert!(Balances::balance(&TREASURY_ACC).is_zero());
			let total_issuance = <Test as Config>::Currency::total_issuance();

			// total issuance should not increase when not noting authors because we haven't
			// reached NetworkRewardStart yet
			roll_to(10, vec![None]);
			assert!(Balances::balance(&TREASURY_ACC).is_zero());
			assert_eq!(total_issuance, <Test as Config>::Currency::total_issuance());

			// set current block to one block before NetworkRewardStart
			let network_reward_start = <Test as Config>::NetworkRewardStart::get();
			System::set_block_number(network_reward_start.saturating_sub(1));

			// network rewards should only appear 1 block after start
			roll_to(network_reward_start, vec![None]);
			assert!(Balances::balance(&TREASURY_ACC).is_zero());
			assert_eq!(total_issuance, <Test as Config>::Currency::total_issuance());

			// should mint to treasury now
			roll_to(network_reward_start + 1, vec![None]);
			let network_reward = Balances::balance(&TREASURY_ACC);
			assert!(!network_reward.is_zero());
			assert_eq!(
				total_issuance + network_reward,
				<Test as Config>::Currency::total_issuance()
			);
			let inflation_config = StakePallet::inflation_config();
			let col_rewards = inflation_config.collator.reward_rate.per_block * total_collator_stake;
			assert_eq!(network_reward, <Test as Config>::NetworkRewardRate::get() * col_rewards);

			// should mint exactly the same amount
			roll_to(network_reward_start + 2, vec![None]);
			assert_eq!(2 * network_reward, Balances::balance(&TREASURY_ACC));
			assert_eq!(
				total_issuance + 2 * network_reward,
				<Test as Config>::Currency::total_issuance()
			);

			// should mint exactly the same amount in each block
			roll_to(network_reward_start + 100, vec![None]);
			assert_eq!(100 * network_reward, Balances::balance(&TREASURY_ACC));
			assert_eq!(
				total_issuance + 100 * network_reward,
				<Test as Config>::Currency::total_issuance()
			);

			// should mint the same amount even if a collator exits because reward is only
			// based on MaxCollatorCandidateStake and MaxSelectedCandidates
			assert_ok!(StakePallet::init_leave_candidates(RuntimeOrigin::signed(1)));
			roll_to(network_reward_start + 101, vec![None]);
			assert_eq!(101 * network_reward, Balances::balance(&TREASURY_ACC));
			assert_eq!(
				total_issuance + 101 * network_reward,
				<Test as Config>::Currency::total_issuance()
			);
		});
}

#[test]
fn network_reward_increase_max_candidate_stake() {
	let max_stake: Balance = 160_000_000 * DECIMALS;
	let collators: Vec<(AccountId, Balance)> = (1u64..=<Test as Config>::MinCollators::get().into())
		.map(|acc_id| (acc_id, max_stake))
		.collect();

	ExtBuilder::default()
		.with_balances(collators.clone())
		.with_collators(collators)
		.build_and_execute_with_sanity_tests(|| {
			let network_reward_start = <Test as Config>::NetworkRewardStart::get();
			let total_issuance = <Test as Config>::Currency::total_issuance();
			System::set_block_number(network_reward_start);

			// should mint to treasury now
			roll_to(network_reward_start + 1, vec![None]);
			let reward_before = Balances::balance(&TREASURY_ACC);
			assert!(!reward_before.is_zero());
			assert_eq!(
				total_issuance + reward_before,
				<Test as Config>::Currency::total_issuance()
			);

			// double max stake
			let max_stake_doubled = 320_000_000 * DECIMALS;
			let reward_after = 2 * reward_before;
			assert_ok!(StakePallet::set_max_candidate_stake(
				RuntimeOrigin::root(),
				max_stake_doubled
			));
			roll_to(network_reward_start + 2, vec![None]);
			assert_eq!(reward_before + reward_after, Balances::balance(&TREASURY_ACC));
			assert_eq!(
				reward_before + reward_after + total_issuance,
				<Test as Config>::Currency::total_issuance()
			);
		});
}

#[test]
fn network_reward_increase_max_collator_count() {
	let max_stake: Balance = 160_000_000 * DECIMALS;
	let collators: Vec<(AccountId, Balance)> = (1u64..=<Test as Config>::MinCollators::get().into())
		.map(|acc_id| (acc_id, max_stake))
		.collect();

	ExtBuilder::default()
		.with_balances(collators.clone())
		.with_collators(collators)
		.build_and_execute_with_sanity_tests(|| {
			let network_reward_start = <Test as Config>::NetworkRewardStart::get();
			let total_issuance = <Test as Config>::Currency::total_issuance();
			System::set_block_number(network_reward_start);

			// should mint to treasury now
			roll_to(network_reward_start + 1, vec![None]);
			let reward_before = Balances::balance(&TREASURY_ACC);
			assert!(!reward_before.is_zero());
			assert_eq!(
				total_issuance + reward_before,
				<Test as Config>::Currency::total_issuance()
			);

			// tripple number of max collators
			let reward_after = 3 * reward_before;
			assert_ok!(StakePallet::set_max_selected_candidates(
				RuntimeOrigin::root(),
				<Test as Config>::MinCollators::get() * 3
			));
			roll_to(network_reward_start + 2, vec![None]);
			assert_eq!(reward_before + reward_after, Balances::balance(&TREASURY_ACC));
			assert_eq!(
				reward_before + reward_after + total_issuance,
				<Test as Config>::Currency::total_issuance()
			);
		});
}

#[test]
fn rewards_candidate_stake_less() {
	ExtBuilder::default()
		.with_balances(vec![(1, 2 * DECIMALS), (2, DECIMALS), (3, DECIMALS), (4, 100)])
		.with_collators(vec![(1, 2 * DECIMALS), (4, 10)])
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
			assert_ok!(StakePallet::candidate_stake_less(RuntimeOrigin::signed(1), DECIMALS));
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
fn rewards_candidate_leave_network() {
	ExtBuilder::default()
		.with_balances(vec![
			(1, 2 * DECIMALS),
			(2, DECIMALS),
			(3, DECIMALS),
			(4, DECIMALS),
			(5, DECIMALS),
		])
		.with_collators(vec![(1, 2 * DECIMALS), (4, DECIMALS), (5, DECIMALS)])
		.with_delegators(vec![(2, 1, DECIMALS), (3, 1, DECIMALS)])
		.build_and_execute_with_sanity_tests(|| {
			// init does not increment rewards
			assert_ok!(StakePallet::init_leave_candidates(RuntimeOrigin::signed(1)));

			// advance two rounds to enable leaving
			roll_to(
				10,
				vec![
					// we're already in block 1, so cant note_author for block 1
					None,
					Some(1),
					Some(2),
					Some(1),
					Some(2),
					Some(1),
					Some(2),
					Some(1),
					Some(2),
				],
			);
			// Only authored should be bumped for collator, not rewarded
			assert_eq!(StakePallet::blocks_authored(1), 4 * 2);
			assert!(StakePallet::blocks_rewarded(1).is_zero());

			// count for delegators should not be incremented
			assert!(StakePallet::blocks_rewarded(2).is_zero());
			assert!(StakePallet::blocks_rewarded(3).is_zero());

			// rewards should not be incremented
			(1..=3).for_each(|id| {
				assert!(StakePallet::rewards(id).is_zero());
			});

			// execute leave intent to trigger reward incrementing for collator and
			// delegators
			assert_ok!(StakePallet::execute_leave_candidates(RuntimeOrigin::signed(1), 1));

			// reward counting storages should be killed for collator
			assert!(StakePallet::blocks_authored(1).is_zero());
			assert!(StakePallet::blocks_rewarded(1).is_zero());
			assert!(!StakePallet::rewards(1).is_zero());

			// reward counting storages should NOT be killed for delegators
			(2..=3).for_each(|id| {
				assert!(!StakePallet::rewards(id).is_zero(), "Zero rewards acc_id {:?}", id);
				assert_eq!(
					StakePallet::blocks_rewarded(id),
					4 * 2,
					"Rewarded blocks Delegator {:?} do not match up with exited collator",
					id
				);
			});
		});
}

#[test]
fn rewards_force_remove_candidate() {
	ExtBuilder::default()
		.with_balances(vec![
			(1, DECIMALS),
			(2, DECIMALS),
			(3, DECIMALS),
			(4, DECIMALS),
			(5, DECIMALS),
		])
		.with_collators(vec![(1, DECIMALS), (4, DECIMALS), (5, DECIMALS)])
		.with_delegators(vec![(2, 1, DECIMALS), (3, 1, DECIMALS)])
		.build_and_execute_with_sanity_tests(|| {
			// init does not increment rewards
			StakePallet::note_author(1);
			StakePallet::note_author(2);

			// removing triggers reward increment for collator 1 and delegators 4, 5
			assert_ok!(StakePallet::force_remove_candidate(RuntimeOrigin::root(), 1));
			// rewarded counter storage should be killed for collator
			assert!(StakePallet::blocks_authored(1).is_zero());
			assert!(StakePallet::blocks_rewarded(1).is_zero());
			// rewards should be set
			assert!(!StakePallet::rewards(1).is_zero());

			(1..=3).for_each(|id| {
				// rewards should be non zero
				assert!(!StakePallet::rewards(id).is_zero(), "Zero rewards for acc_id {:?}", id);
				// rewards should equal API call
				assert_eq!(
					StakePallet::get_unclaimed_staking_rewards(&id),
					StakePallet::rewards(id)
				);
				if id > 1 {
					assert_eq!(
						StakePallet::blocks_rewarded(id),
						2,
						"Rewarded counter does not match for delegator {:?}",
						id
					);
				}
			});
			assert_eq!(StakePallet::get_unclaimed_staking_rewards(&1), StakePallet::rewards(1));

			(4..=5).for_each(|id| {
				assert!(StakePallet::rewards(id).is_zero(), "acc_id {:?}", id);
				assert!(StakePallet::blocks_rewarded(id).is_zero(), "acc_id {:?}", id);
			});
		});
}

#[test]
fn blocks_rewarded_join_delegators() {
	ExtBuilder::default()
		.with_balances(vec![(1, 1000), (2, 100), (3, 100)])
		.with_collators(vec![(1, 1000), (3, 10)])
		.build_and_execute_with_sanity_tests(|| {
			// note once to set counter to 1
			StakePallet::note_author(1);
			assert_eq!(StakePallet::blocks_authored(1), 2);
			assert!(StakePallet::blocks_rewarded(1).is_zero());
			assert_ok!(StakePallet::join_delegators(RuntimeOrigin::signed(2), 1, 100));
			// delegator's rewarded counter should equal of collator's authored counter upon
			// joining
			assert_eq!(StakePallet::blocks_rewarded(2), StakePallet::blocks_authored(1));
		});
}

#[test]
fn rewards_delegator_stake_more() {
	ExtBuilder::default()
		.with_balances(vec![(1, DECIMALS), (2, DECIMALS), (3, 2 * DECIMALS), (4, 100)])
		.with_collators(vec![(1, DECIMALS), (4, 10)])
		.with_delegators(vec![(2, 1, DECIMALS), (3, 1, DECIMALS)])
		.build_and_execute_with_sanity_tests(|| {
			// note once to set counter to 1
			StakePallet::note_author(1);
			assert_eq!(StakePallet::blocks_authored(1), 2);
			assert!(StakePallet::blocks_rewarded(2).is_zero());
			assert!(StakePallet::blocks_rewarded(3).is_zero());
			(1..=3).for_each(|id| {
				assert!(StakePallet::rewards(id).is_zero(), "acc_id {:?}", id);
			});

			// stake less to trigger reward incrementing just for 3
			assert_ok!(StakePallet::delegator_stake_more(RuntimeOrigin::signed(3), DECIMALS));
			// 1 should still have counter 1 but no rewards
			assert_eq!(StakePallet::blocks_authored(1), 2);
			assert!(StakePallet::blocks_rewarded(1).is_zero());
			assert!(StakePallet::rewards(1).is_zero());
			// 2 should still have neither rewards nor counter
			assert!(StakePallet::blocks_rewarded(2).is_zero());
			assert!(StakePallet::rewards(2).is_zero());
			// 3 should have rewards and the same counter as 1
			assert_eq!(StakePallet::blocks_rewarded(3), 2);
			assert!(!StakePallet::rewards(3).is_zero());
		});
}

#[test]
fn rewards_delegator_stake_less() {
	ExtBuilder::default()
		.with_balances(vec![(1, DECIMALS), (2, DECIMALS), (3, 2 * DECIMALS), (7, 100)])
		.with_collators(vec![(1, DECIMALS), (7, 10)])
		.with_delegators(vec![(2, 1, DECIMALS), (3, 1, 2 * DECIMALS)])
		.build_and_execute_with_sanity_tests(|| {
			// note once to set counter to 1
			StakePallet::note_author(1);
			assert_eq!(StakePallet::blocks_authored(1), 2);
			assert!(StakePallet::blocks_rewarded(2).is_zero());
			assert!(StakePallet::blocks_rewarded(3).is_zero());
			(1..=3).for_each(|id| {
				assert!(StakePallet::rewards(id).is_zero(), "acc_id {:?}", id);
			});

			// stake less to trigger reward incrementing just for 3
			assert_ok!(StakePallet::delegator_stake_less(RuntimeOrigin::signed(3), DECIMALS));
			// 1 should still have counter 1 but no rewards
			assert_eq!(StakePallet::blocks_authored(1), 2);
			assert!(StakePallet::blocks_rewarded(1).is_zero());
			assert!(StakePallet::rewards(1).is_zero());
			// 2 should still have neither rewards nor counter
			assert!(StakePallet::blocks_rewarded(2).is_zero());
			assert!(StakePallet::rewards(2).is_zero());
			// 3 should have rewards and the same counter as 1
			assert_eq!(StakePallet::blocks_rewarded(3), 2);
			assert!(!StakePallet::rewards(3).is_zero());
		});
}

#[test]
fn rewards_delegator_replaced() {
	ExtBuilder::default()
		.with_balances(vec![
			(1, 2 * DECIMALS),
			(2, 2 * DECIMALS),
			(3, 2 * DECIMALS),
			(4, 2 * DECIMALS),
			(5, 2 * DECIMALS),
			(6, 2 * DECIMALS),
			(7, 100),
		])
		.with_collators(vec![(1, 2 * DECIMALS), (7, 10)])
		.with_delegators(vec![
			(2, 1, 2 * DECIMALS),
			(3, 1, 2 * DECIMALS),
			(4, 1, 2 * DECIMALS),
			(5, 1, DECIMALS),
		])
		.build_and_execute_with_sanity_tests(|| {
			// note once to set counter to 1
			StakePallet::note_author(1);
			assert_eq!(StakePallet::blocks_authored(1), 2);

			// 6 kicks 5
			assert_ok!(StakePallet::join_delegators(RuntimeOrigin::signed(6), 1, 2 * DECIMALS));
			// 5 should have rewards and counter updated
			assert!(!StakePallet::rewards(5).is_zero());
			assert_eq!(StakePallet::blocks_rewarded(5), 2);
			// 6 should not have rewards but same counter as former collator
			assert!(StakePallet::rewards(6).is_zero());
			assert_eq!(StakePallet::blocks_rewarded(6), 2);
		});
}

#[test]
fn rewards_delegator_leaves() {
	ExtBuilder::default()
		.with_balances(vec![(1, DECIMALS), (2, DECIMALS), (3, DECIMALS), (4, 100)])
		.with_collators(vec![(1, DECIMALS), (4, 10)])
		.with_delegators(vec![(2, 1, DECIMALS), (3, 1, DECIMALS)])
		.build_and_execute_with_sanity_tests(|| {
			// note collator once to set their counter to 1
			StakePallet::note_author(1);
			assert_eq!(StakePallet::blocks_authored(1), 2);
			assert!(StakePallet::blocks_rewarded(2).is_zero());
			assert!(StakePallet::blocks_rewarded(3).is_zero());
			(1..=3).for_each(|id| {
				assert!(StakePallet::rewards(id).is_zero(), "acc_id {:?}", id);
			});

			// only 3 should have non-zero rewards
			assert_ok!(StakePallet::leave_delegators(RuntimeOrigin::signed(3)));
			assert!(StakePallet::blocks_rewarded(1).is_zero());
			assert!(StakePallet::rewards(1).is_zero());
			assert!(StakePallet::blocks_rewarded(2).is_zero());
			assert!(StakePallet::rewards(2).is_zero());
			assert!(!StakePallet::rewards(3).is_zero());
			assert_eq!(StakePallet::get_unclaimed_staking_rewards(&3), StakePallet::rewards(3));
			// counter should be reset due to leaving
			assert!(StakePallet::blocks_rewarded(3).is_zero());
		});
}

#[test]
fn rewards_incrementing_and_claiming() {
	ExtBuilder::default()
		.with_balances(vec![(1, DECIMALS), (2, DECIMALS), (3, DECIMALS), (4, 100)])
		.with_collators(vec![(1, DECIMALS), (4, 10)])
		.with_delegators(vec![(2, 1, DECIMALS), (3, 1, DECIMALS)])
		.build_and_execute_with_sanity_tests(|| {
			// claiming should not be possible with zero counters
			(1..=4).for_each(|id| {
				assert_noop!(
					StakePallet::claim_rewards(RuntimeOrigin::signed(id)),
					Error::<Test>::RewardsNotFound,
				);
			});

			// note once to set counter to 1
			StakePallet::note_author(1);
			assert_eq!(StakePallet::blocks_authored(1), 2);
			assert!(StakePallet::blocks_rewarded(2).is_zero());

			// claiming should not be possible before incrementing rewards
			(1..=4).for_each(|id| {
				assert_noop!(
					StakePallet::claim_rewards(RuntimeOrigin::signed(id)),
					Error::<Test>::RewardsNotFound
				);
			});

			// increment rewards for 2 and match counter to collator
			assert_ok!(StakePallet::increment_delegator_rewards(RuntimeOrigin::signed(2)));
			assert_eq!(StakePallet::blocks_rewarded(2), 2);
			let rewards_2 = StakePallet::rewards(2);
			assert!(!rewards_2.is_zero());
			assert!(StakePallet::blocks_rewarded(3).is_zero());
			assert!(StakePallet::rewards(3).is_zero());

			// should only update rewards for collator as well
			assert_ok!(StakePallet::increment_collator_rewards(RuntimeOrigin::signed(1)));
			assert_eq!(StakePallet::blocks_rewarded(1), StakePallet::blocks_authored(1));
			assert!(!StakePallet::rewards(1).is_zero());
			// rewards of 2 should not be changed
			assert_eq!(StakePallet::rewards(2), rewards_2);
			// 3 should still not have blocks rewarded bumped
			assert!(StakePallet::blocks_rewarded(3).is_zero());

			// claim for 1 to move rewards into balance
			assert_ok!(StakePallet::claim_rewards(RuntimeOrigin::signed(1)));
			assert!(StakePallet::rewards(1).is_zero());
			// delegator situation should be unchanged
			assert!(Balances::balance(&1) > DECIMALS);
			assert_eq!(Balances::balance(&2), DECIMALS);
			assert_eq!(Balances::balance(&3), DECIMALS);

			// incrementing again should not change anything because collator has not
			// authored blocks since last inc
			assert_ok!(StakePallet::increment_delegator_rewards(RuntimeOrigin::signed(2)));
			assert_eq!(StakePallet::blocks_rewarded(2), 2);
			// claim for 2 to move rewards into balance
			assert_ok!(StakePallet::claim_rewards(RuntimeOrigin::signed(2)));
			assert!(Balances::balance(&2) > DECIMALS);
			assert!(StakePallet::rewards(2).is_zero());
			assert_eq!(Balances::balance(&3), DECIMALS);

			// should not be able to claim for incorrect role
			assert_noop!(
				StakePallet::increment_collator_rewards(RuntimeOrigin::signed(2)),
				Error::<Test>::CandidateNotFound
			);
			assert_noop!(
				StakePallet::increment_delegator_rewards(RuntimeOrigin::signed(1)),
				Error::<Test>::DelegatorNotFound
			);
		});
}

#[test]
fn api_get_unclaimed_staking_rewards() {
	let stake = 100_000 * DECIMALS;
	ExtBuilder::default()
		.with_balances(vec![(1, stake), (2, stake), (3, 100 * stake)])
		.with_collators(vec![(1, stake), (3, 2 * stake)])
		.with_delegators(vec![(2, 1, stake)])
		.build_and_execute_with_sanity_tests(|| {
			let inflation_config = StakePallet::inflation_config();

			// Increment rewards of 1 and 2
			roll_to(2, vec![None, Some(1)]);
			assert_eq!(
				StakePallet::get_unclaimed_staking_rewards(&1),
				// Multiplying with 2 because there are two authors
				inflation_config.collator.reward_rate.per_block * stake * 2
			);
			assert_eq!(
				StakePallet::get_unclaimed_staking_rewards(&2),
				inflation_config.delegator.reward_rate.per_block * stake * 2
			);
			assert!(StakePallet::get_unclaimed_staking_rewards(&3).is_zero());

			// Should only increment rewards of 3
			roll_to(3, vec![None, None, Some(3)]);
			let rewards_1 = StakePallet::get_unclaimed_staking_rewards(&1);
			let rewards_2 = StakePallet::get_unclaimed_staking_rewards(&2);
			let rewards_3 = StakePallet::get_unclaimed_staking_rewards(&3);
			assert_eq!(2 * rewards_1, rewards_3,);
			assert_eq!(rewards_2, inflation_config.delegator.reward_rate.per_block * stake * 2);

			// API and actual claiming should match
			assert_ok!(StakePallet::increment_collator_rewards(RuntimeOrigin::signed(1)));
			assert_ok!(StakePallet::claim_rewards(RuntimeOrigin::signed(1)));
			assert_eq!(rewards_1, Balances::usable_balance(1));
			assert!(StakePallet::get_unclaimed_staking_rewards(&1).is_zero());

			assert_ok!(StakePallet::increment_delegator_rewards(RuntimeOrigin::signed(2)));
			assert_ok!(StakePallet::claim_rewards(RuntimeOrigin::signed(2)));
			assert_eq!(rewards_2, Balances::usable_balance(2));
			assert!(StakePallet::get_unclaimed_staking_rewards(&2).is_zero());

			assert_ok!(StakePallet::increment_collator_rewards(RuntimeOrigin::signed(3)));
			assert_ok!(StakePallet::claim_rewards(RuntimeOrigin::signed(3)));
			assert_eq!(rewards_3 + 98 * stake, Balances::usable_balance(3));
			assert!(StakePallet::get_unclaimed_staking_rewards(&3).is_zero());
		});
}
