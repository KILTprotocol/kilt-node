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

use frame_support::assert_ok;
use pallet_authorship::EventHandler;
use sp_runtime::{traits::Zero, Perquintill};

use crate::{
	mock::{roll_to_claim_rewards, ExtBuilder, RuntimeOrigin, StakePallet, System, Test, DECIMALS},
	Config, InflationInfo, RewardRate, StakingInfo,
};

#[test]
fn rewards_set_inflation() {
	let hundred = Perquintill::from_percent(100);
	ExtBuilder::default()
		.with_balances(vec![
			(1, DECIMALS),
			(2, DECIMALS),
			(3, DECIMALS),
			(4, DECIMALS),
			(5, DECIMALS),
		])
		.with_collators(vec![(1, DECIMALS), (2, DECIMALS)])
		.with_delegators(vec![(3, 1, DECIMALS), (4, 1, DECIMALS), (5, 2, DECIMALS)])
		.build_and_execute_with_sanity_tests(|| {
			// note collators
			StakePallet::note_author(1);
			StakePallet::note_author(1);
			StakePallet::note_author(2);

			// set inflation to trigger reward setting
			assert_ok!(StakePallet::set_inflation(
				RuntimeOrigin::root(),
				hundred,
				hundred,
				hundred,
				hundred
			));
			// rewards and counters should be set
			(1..=5).for_each(|id| {
				assert!(!StakePallet::blocks_rewarded(id).is_zero(), "acc_id {:?}", id);
				assert!(!StakePallet::rewards(id).is_zero(), "acc_id {:?}", id);
			});
		});
}

#[test]
fn rewards_yearly_inflation_adjustment() {
	ExtBuilder::default()
		.with_balances(vec![
			(1, DECIMALS),
			(2, DECIMALS),
			(3, DECIMALS),
			(4, DECIMALS),
			(5, DECIMALS),
		])
		.with_collators(vec![(1, DECIMALS), (2, DECIMALS)])
		.with_delegators(vec![(3, 1, DECIMALS), (4, 1, DECIMALS), (5, 2, DECIMALS)])
		.build_and_execute_with_sanity_tests(|| {
			// init counter and go to next year
			StakePallet::note_author(1);
			StakePallet::note_author(2);
			System::set_block_number(<Test as Config>::BLOCKS_PER_YEAR - 1);
			roll_to_claim_rewards(<Test as Config>::BLOCKS_PER_YEAR + 1, vec![]);
			assert!(!StakePallet::blocks_authored(1).is_zero());
			assert!(!StakePallet::blocks_authored(2).is_zero());

			// rewards should not be triggered before executing pending adjustment
			(1..=5).for_each(|id| {
				assert!(StakePallet::rewards(id).is_zero(), "acc_id {:?}", id);
			});

			// execute to trigger reward increment
			assert_ok!(StakePallet::execute_scheduled_reward_change(RuntimeOrigin::signed(1)));
			(1..=5).for_each(|id| {
				assert!(
					!StakePallet::blocks_rewarded(id).is_zero(),
					"Zero rewarded blocks for acc_id {:?}",
					id
				);
				assert!(!StakePallet::rewards(id).is_zero(), "Zero rewards for acc_id {:?}", id);
			});
		});
}

#[test]
fn update_inflation() {
	ExtBuilder::default()
		.with_balances(vec![(1, 10), (2, 100)])
		.with_collators(vec![(1, 10), (2, 10)])
		.build_and_execute_with_sanity_tests(|| {
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
			assert!(!invalid_inflation.is_valid(<Test as Config>::BLOCKS_PER_YEAR));
			invalid_inflation.collator.reward_rate.per_block = Perquintill::zero();
			assert!(!invalid_inflation.is_valid(<Test as Config>::BLOCKS_PER_YEAR));

			assert_ok!(StakePallet::set_inflation(
				RuntimeOrigin::root(),
				Perquintill::from_percent(0),
				Perquintill::from_percent(100),
				Perquintill::from_percent(100),
				Perquintill::from_percent(100),
			));
			assert_ok!(StakePallet::set_inflation(
				RuntimeOrigin::root(),
				Perquintill::from_percent(100),
				Perquintill::from_percent(0),
				Perquintill::from_percent(100),
				Perquintill::from_percent(100),
			));
			assert_ok!(StakePallet::set_inflation(
				RuntimeOrigin::root(),
				Perquintill::from_percent(100),
				Perquintill::from_percent(100),
				Perquintill::from_percent(0),
				Perquintill::from_percent(100),
			));
			assert_ok!(StakePallet::set_inflation(
				RuntimeOrigin::root(),
				Perquintill::from_percent(100),
				Perquintill::from_percent(100),
				Perquintill::from_percent(100),
				Perquintill::from_percent(0),
			));
		});
}
