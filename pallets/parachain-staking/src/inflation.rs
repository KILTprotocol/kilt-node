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

//! Helper methods for computing issuance based on inflation
use crate::pallet::{BalanceOf, Config, Pallet};
use frame_support::traits::Currency;
use kilt_primitives::constants::YEARS;
use parity_scale_codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{Perbill, Perquintill, RuntimeDebug};

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Eq, PartialEq, Clone, Encode, Decode, Default, RuntimeDebug)]
pub struct RewardRate {
	pub annual: Perbill,
	pub round: Perbill,
}

/// Convert annual inflation rate range to round inflation range
pub fn annual_to_round<T: Config>(rate: Perbill) -> Perbill {
	let periods = rounds_per_year::<T>();
	// safe because periods > 0 is ensured in `set_blocks_per_round`
	perbill_annual_to_perbill_round(rate, periods)
}

// Convert blocks per round to rounds per year
fn rounds_per_year<T: Config>() -> u32 {
	let blocks_per_round = <Pallet<T>>::round().length;
	// blocks_per_round > 0 is ensured in `set_blocks_per_round`
	YEARS / blocks_per_round
}

/// Convert an annual inflation to a round inflation
fn perbill_annual_to_perbill_round(annual: Perbill, rounds_per_year: u32) -> Perbill {
	Perbill::from_parts(annual.deconstruct() / rounds_per_year)
}

impl RewardRate {
	pub fn new<T: Config>(rate: Perbill) -> Self {
		RewardRate {
			annual: rate,
			round: annual_to_round::<T>(rate),
		}
	}

	pub fn update_blocks_per_round(&mut self, blocks_per_round: u32) {
		let rounds_per_year = YEARS / blocks_per_round;
		self.round = perbill_annual_to_perbill_round(self.annual, rounds_per_year);
	}
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Eq, PartialEq, Clone, Encode, Decode, Default, RuntimeDebug)]
pub struct StakingInfo {
	/// Maximum staking rate
	pub max_rate: Perbill,
	/// Reward rate
	pub reward_rate: RewardRate,
}

impl StakingInfo {
	pub fn new<T: Config>(max_rate: Perbill, annual_reward_rate: Perbill) -> Self {
		StakingInfo {
			max_rate,
			reward_rate: RewardRate::new::<T>(annual_reward_rate),
		}
	}

	/// Set max staking rate
	pub fn set_max_rate(&mut self, max_rate: Perbill) {
		self.max_rate = max_rate;
	}

	/// Set reward rate
	pub fn set_reward_rate<T: Config>(&mut self, annual_rate: Perbill) {
		self.reward_rate.annual = annual_rate;
		self.reward_rate.round = annual_to_round::<T>(annual_rate)
	}

	pub fn compute_rewards<T: Config>(&self, stake: BalanceOf<T>, total_issuance: BalanceOf<T>) -> BalanceOf<T> {
		let staking_rate = Perbill::from_rational(stake, total_issuance).min(self.max_rate);
		let rewards = self.reward_rate.round * total_issuance;
		staking_rate * rewards
	}

	pub fn compute_block_rewards<T: Config>(&self, stake: BalanceOf<T>, total_issuance: BalanceOf<T>) -> BalanceOf<T> {
		// TODO: Refactor Perbills to be Perquintill
		let max_rate: u64 = (self.max_rate.deconstruct() as u64).saturating_mul(1000000000u64);
		let annual_rate: u64 = (self.reward_rate.annual.deconstruct() as u64).saturating_mul(1000000000u64);

		let staking_rate = Perquintill::from_rational(stake, total_issuance).min(Perquintill::from_parts(max_rate));
		let rewards = Perquintill::from_parts(annual_rate) * total_issuance;
		let rewards = staking_rate * rewards;
		Perquintill::from_rational(1u64, YEARS.into()) * rewards
	}
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Eq, PartialEq, Clone, Encode, Decode, Default, RuntimeDebug)]
pub struct InflationInfo {
	pub collator: StakingInfo,
	pub delegator: StakingInfo,
}

impl InflationInfo {
	pub fn new<T: Config>(
		// TODO: How to solve this more elegantly?
		collator_max_rate_percentage: u32,
		collator_annual_reward_rate_percentage: u32,
		delegator_max_rate_percentage: u32,
		delegator_annual_reward_rate_percentage: u32,
	) -> Self {
		Self {
			collator: StakingInfo::new::<T>(
				Perbill::from_percent(collator_max_rate_percentage),
				Perbill::from_percent(collator_annual_reward_rate_percentage),
			),
			delegator: StakingInfo::new::<T>(
				Perbill::from_percent(delegator_max_rate_percentage),
				Perbill::from_percent(delegator_annual_reward_rate_percentage),
			),
		}
	}

	pub fn update_blocks_per_round(&mut self, blocks_per_round: u32) {
		self.collator.reward_rate.update_blocks_per_round(blocks_per_round);
		self.delegator.reward_rate.update_blocks_per_round(blocks_per_round);
	}

	pub fn round_issuance<T: Config>(
		&self,
		collator_stake: BalanceOf<T>,
		delegator_stake: BalanceOf<T>,
	) -> (BalanceOf<T>, BalanceOf<T>) {
		let circulating = T::Currency::total_issuance();

		let collator_rewards = self.collator.compute_rewards::<T>(collator_stake, circulating);
		let delegator_rewards = self.delegator.compute_rewards::<T>(delegator_stake, circulating);

		(collator_rewards, delegator_rewards)
	}

	pub fn block_issuance<T: Config>(
		&self,
		collator_stake: BalanceOf<T>,
		delegator_stake: BalanceOf<T>,
	) -> (BalanceOf<T>, BalanceOf<T>) {
		let circulating = T::Currency::total_issuance();

		let collator_rewards = self.collator.compute_block_rewards::<T>(collator_stake, circulating);
		let delegator_rewards = self.delegator.compute_block_rewards::<T>(delegator_stake, circulating);

		(collator_rewards, delegator_rewards)
	}

	pub fn is_valid(&self) -> bool {
		self.collator.max_rate >= Perbill::zero()
			&& self.collator.max_rate <= Perbill::one()
			&& self.delegator.max_rate >= Perbill::zero()
			&& self.delegator.max_rate <= Perbill::one()
			&& self.collator.reward_rate.annual >= Perbill::zero()
			&& self.collator.reward_rate.annual <= Perbill::one()
			&& self.delegator.reward_rate.annual >= Perbill::zero()
			&& self.delegator.reward_rate.annual <= Perbill::one()
			&& self.collator.reward_rate.annual >= Perbill::zero()
			&& self.collator.reward_rate.annual <= Perbill::one()
			&& self.collator.reward_rate.annual >= self.collator.reward_rate.round
			&& self.delegator.reward_rate.annual >= self.delegator.reward_rate.round
			&& self.collator.reward_rate.round >= Perbill::zero()
			&& self.collator.reward_rate.round <= Perbill::one()
			&& self.delegator.reward_rate.round >= Perbill::zero()
			&& self.delegator.reward_rate.round <= Perbill::one()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		mock::{ExtBuilder, Stake, Test},
		Round, RoundInfo,
	};

	#[test]
	fn perbill() {
		assert_eq!(
			Perbill::from_percent(100) * Perbill::from_percent(50),
			Perbill::from_percent(50)
		);
	}

	#[test]
	fn simple_rewards() {
		ExtBuilder::default()
			.with_inflation(10, 15, 40, 10, YEARS)
			.build()
			.execute_with(|| {
				// Unrealistic configuration but makes computation simple
				<Round<Test>>::put(RoundInfo {
					current: 1,
					first: 1,
					length: YEARS,
				});
				let rounds_per_year = 1;
				let inflation = InflationInfo::new::<Test>(10, 15, 40, 10);

				// Dummy checks for correct instantiation
				assert!(inflation.is_valid());
				assert_eq!(inflation.collator.max_rate, Perbill::from_percent(10));
				assert_eq!(inflation.collator.reward_rate.annual, Perbill::from_percent(15));
				assert_eq!(
					inflation.collator.reward_rate.round,
					Perbill::from_percent(15) / rounds_per_year
				);
				assert_eq!(inflation.delegator.max_rate, Perbill::from_percent(40));
				assert_eq!(inflation.delegator.reward_rate.annual, Perbill::from_percent(10));
				assert_eq!(
					inflation.delegator.reward_rate.round,
					Perbill::from_percent(10) / rounds_per_year
				);

				// Check collator reward computation
				assert_eq!(inflation.collator.compute_rewards::<Test>(0, 100_000), 0);
				assert_eq!(inflation.collator.compute_rewards::<Test>(5000, 100_000), 750);
				// Check for max_rate which is 10%
				assert_eq!(inflation.collator.compute_rewards::<Test>(10_000, 100_000), 1500);
				// Check exceeding max_rate
				assert_eq!(inflation.collator.compute_rewards::<Test>(100_000, 100_000), 1500);
				// Stake can never be more than what is issued, but let's check whether the cap
				// still applies
				assert_eq!(inflation.collator.compute_rewards::<Test>(100_001, 100_000), 1500);

				// Check delegator reward calculation
				assert_eq!(inflation.delegator.compute_rewards::<Test>(0, 100_000), 0);
				assert_eq!(inflation.delegator.compute_rewards::<Test>(5000, 100_000), 500);
				// Check for max_rate which is 40%
				assert_eq!(inflation.delegator.compute_rewards::<Test>(40_000, 100_000), 4000);
				// Check exceeding max_rate
				assert_eq!(inflation.delegator.compute_rewards::<Test>(100_000, 100_000), 4000);
				// Stake can never be more than what is issued, but let's check whether the cap
				// still applies
				assert_eq!(inflation.delegator.compute_rewards::<Test>(100_001, 100_000), 4000);
			});
	}

	#[test]
	fn more_realistic_rewards() {
		ExtBuilder::default()
			.with_inflation(10, 15, 40, 10, 600)
			.build()
			.execute_with(|| {
				let rounds_per_year = YEARS / 600;
				let inflation = Stake::inflation_config();

				// Dummy checks for correct instantiation
				assert!(inflation.is_valid());
				assert_eq!(inflation.collator.max_rate, Perbill::from_percent(10));
				assert_eq!(inflation.collator.reward_rate.annual, Perbill::from_percent(15));
				assert_eq!(
					inflation.collator.reward_rate.round,
					Perbill::from_percent(15) / rounds_per_year
				);
				assert_eq!(inflation.delegator.max_rate, Perbill::from_percent(40));
				assert_eq!(inflation.delegator.reward_rate.annual, Perbill::from_percent(10));
				assert_eq!(
					inflation.delegator.reward_rate.round,
					Perbill::from_percent(10) / rounds_per_year
				);

				// Check collator reward computation
				assert_eq!(inflation.collator.compute_rewards::<Test>(0, 160_000_000), 0);
				assert_eq!(inflation.collator.compute_rewards::<Test>(100_000, 160_000_000), 2);
				// Check for max_rate which is 10%
				assert_eq!(inflation.collator.compute_rewards::<Test>(16_000_000, 160_000_000), 278);
				// Check exceeding max_rate
				assert_eq!(inflation.collator.compute_rewards::<Test>(32_000_000, 160_000_000), 278);
				// Stake can never be more than what is issued, but let's check whether the cap
				// still applies
				assert_eq!(
					inflation.collator.compute_rewards::<Test>(200_000_000, 160_000_000),
					278
				);

				// Check delegator reward calculation
				assert_eq!(inflation.delegator.compute_rewards::<Test>(0, 160_000_000), 0);
				assert_eq!(inflation.delegator.compute_rewards::<Test>(100_000, 160_000_000), 1);
				assert!(
					inflation.delegator.compute_rewards::<Test>(16_000_000, 160_000_000)
						< inflation.collator.compute_rewards::<Test>(16_000_000, 160_000_000)
				);
				// Check for max_rate which is 40%
				assert_eq!(
					inflation.delegator.compute_rewards::<Test>(64_000_000, 160_000_000),
					741
				);
				// Check exceeding max_rate
				assert_eq!(
					inflation.delegator.compute_rewards::<Test>(100_000_000, 160_000_000),
					741
				);
				// Stake can never be more than what is issued, but let's check whether the cap
				// still applies
				assert_eq!(
					inflation.delegator.compute_rewards::<Test>(200_000_000, 160_000_000),
					741
				);
			});
	}

	#[test]
	fn real_rewards() {
		ExtBuilder::default()
			.set_blocks_per_round(600)
			.build()
			.execute_with(|| {
				let rounds_per_year = YEARS / 600;
				let inflation = Stake::inflation_config();

				// Dummy checks for correct instantiation
				assert!(inflation.is_valid());
				assert_eq!(inflation.collator.max_rate, Perbill::from_percent(10));
				assert_eq!(inflation.collator.reward_rate.annual, Perbill::from_percent(15));
				assert_eq!(
					inflation.collator.reward_rate.round,
					Perbill::from_percent(15) / rounds_per_year
				);
				assert_eq!(inflation.delegator.max_rate, Perbill::from_percent(40));
				assert_eq!(inflation.delegator.reward_rate.annual, Perbill::from_percent(10));
				assert_eq!(
					inflation.delegator.reward_rate.round,
					Perbill::from_percent(10) / rounds_per_year
				);

				let decimals = 10u128.pow(15);
				let total_issuance: u128 = 160_000_000u128 * decimals;

				// Check collator reward computation
				assert_eq!(inflation.collator.compute_rewards::<Test>(0, total_issuance), 0);
				assert_eq!(
					inflation
						.collator
						.compute_rewards::<Test>(100_000 * decimals, total_issuance),
					1736100000000000
				);
				// Check for max_rate which is 10%
				assert_eq!(
					inflation
						.collator
						.compute_rewards::<Test>(16_000_000 * decimals, total_issuance),
					277776000000000000
				);
				// Check exceeding max_rate
				assert_eq!(
					inflation
						.collator
						.compute_rewards::<Test>(32_000_000 * decimals, total_issuance),
					277776000000000000
				);
				// Stake can never be more than what is issued, but let's check whether the cap
				// still applies
				assert_eq!(
					inflation
						.collator
						.compute_rewards::<Test>(200_000_000 * decimals, total_issuance),
					277776000000000000
				);

				// Check delegator reward calculation
				assert_eq!(inflation.delegator.compute_rewards::<Test>(0, total_issuance), 0);
				assert_eq!(
					inflation
						.delegator
						.compute_rewards::<Test>(100_000 * decimals, total_issuance),
					1157400000000000
				);
				assert!(
					inflation
						.delegator
						.compute_rewards::<Test>(16_000_000 * decimals, total_issuance)
						< inflation
							.collator
							.compute_rewards::<Test>(16_000_000 * decimals, total_issuance)
				);
				// Check for max_rate which is 40%
				assert_eq!(
					inflation
						.delegator
						.compute_rewards::<Test>(64_000_000 * decimals, total_issuance),
					740736000000000000
				);
				// Check exceeding max_rate
				assert_eq!(
					inflation
						.delegator
						.compute_rewards::<Test>(100_000_000 * decimals, total_issuance),
					740736000000000000
				);
				// Stake can never be more than what is issued, but let's check whether the cap
				// still applies
				assert_eq!(
					inflation
						.delegator
						.compute_rewards::<Test>(200_000_000 * decimals, total_issuance),
					740736000000000000
				);
			});
	}
}
