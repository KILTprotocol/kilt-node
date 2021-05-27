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
use crate::{pallet::Config, types::BalanceOf};
use frame_support::traits::Currency;
use kilt_primitives::constants::YEARS;
use parity_scale_codec::{Decode, Encode};
use sp_runtime::{Perquintill, RuntimeDebug};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Eq, PartialEq, Clone, Encode, Decode, Default, RuntimeDebug)]
pub struct RewardRate {
	pub annual: Perquintill,
	pub per_block: Perquintill,
}

/// Convert annual reward rate to per_block.
fn annual_to_per_block(rate: Perquintill) -> Perquintill {
	rate / YEARS
}

impl RewardRate {
	pub fn new(rate: Perquintill) -> Self {
		RewardRate {
			annual: rate,
			per_block: annual_to_per_block(rate),
		}
	}
}

/// Staking info (staking rate and reward rate) for collators and delegators.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Eq, PartialEq, Clone, Encode, Decode, Default, RuntimeDebug)]
pub struct StakingInfo {
	/// Maximum staking rate.
	pub max_rate: Perquintill,
	/// Reward rate annually and per_block.
	pub reward_rate: RewardRate,
}

impl StakingInfo {
	pub fn new(max_rate: Perquintill, annual_reward_rate: Perquintill) -> Self {
		StakingInfo {
			max_rate,
			reward_rate: RewardRate::new(annual_reward_rate),
		}
	}

	/// Calculate newly minted rewards on coinbase, e.g.,
	/// reward = rewards_per_block * staking_rate.
	pub fn compute_block_rewards<T: Config>(&self, stake: BalanceOf<T>, total_issuance: BalanceOf<T>) -> BalanceOf<T> {
		let staking_rate = Perquintill::from_rational(stake, total_issuance).min(self.max_rate);
		// multiplication with perbill cannot overflow
		let rewards = self.reward_rate.per_block * total_issuance;
		// multiplication with perbill cannot overflow
		staking_rate * rewards
	}
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Eq, PartialEq, Clone, Encode, Decode, Default, RuntimeDebug)]
pub struct InflationInfo {
	pub collator: StakingInfo,
	pub delegator: StakingInfo,
}

impl InflationInfo {
	/// Create a new inflation info from the max staking rates and annual reward
	/// rates for collators and delegators.
	///
	/// Example: InflationInfo::new(Perquintill_from_percent(10), ...)
	pub fn new(
		collator_max_rate_percentage: Perquintill,
		collator_annual_reward_rate_percentage: Perquintill,
		delegator_max_rate_percentage: Perquintill,
		delegator_annual_reward_rate_percentage: Perquintill,
	) -> Self {
		Self {
			collator: StakingInfo::new(collator_max_rate_percentage, collator_annual_reward_rate_percentage),
			delegator: StakingInfo::new(delegator_max_rate_percentage, delegator_annual_reward_rate_percentage),
		}
	}

	/// Compute coinbase rewards for collators and delegators based on the
	/// current staking rates and the InflationInfo.
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

	/// Check whether the annual reward rate is approx. the per_block reward
	/// rate multiplied with the number of blocks per year
	pub fn is_valid(&self) -> bool {
		self.collator.reward_rate.annual
			>= Perquintill::from_parts(self.collator.reward_rate.per_block.deconstruct().saturating_mul(YEARS))
			&& self.delegator.reward_rate.annual
				>= Perquintill::from_parts(self.delegator.reward_rate.per_block.deconstruct().saturating_mul(YEARS))
	}
}

#[cfg(test)]
mod tests {
	use sp_runtime::Perbill;

	use super::*;
	use crate::mock::{almost_equal, ExtBuilder, Test, DECIMALS};

	#[test]
	fn perquintill() {
		assert_eq!(
			Perquintill::from_percent(100) * Perquintill::from_percent(50),
			Perquintill::from_percent(50)
		);
	}

	#[test]
	fn annual_to_block_rate() {
		let rate = Perquintill::one();
		assert!(almost_equal(
			rate * 10_000_000_000u128,
			Perquintill::from_parts(annual_to_per_block(rate).deconstruct() * YEARS) * 10_000_000_000u128,
			Perbill::from_perthousand(1)
		));
	}

	#[test]
	fn simple_block_reward_check() {
		let precision = Perbill::from_perthousand(1);
		ExtBuilder::default()
			.with_inflation(10, 15, 40, 10, 5)
			.with_balances(vec![(1, 10)])
			.with_collators(vec![(1, 10)])
			.build()
			.execute_with(|| {
				let inflation = InflationInfo::new(
					Perquintill::from_percent(10),
					Perquintill::from_percent(15),
					Perquintill::from_percent(40),
					Perquintill::from_percent(10),
				);
				let years_u128: BalanceOf<Test> = YEARS as u128;

				// Dummy checks for correct instantiation
				assert!(inflation.is_valid());
				assert_eq!(inflation.collator.max_rate, Perquintill::from_percent(10));
				assert_eq!(inflation.collator.reward_rate.annual, Perquintill::from_percent(15));
				assert!(
					almost_equal(
						inflation.collator.reward_rate.per_block * DECIMALS * 10_000,
						Perquintill::from_percent(15) * 10_000 * DECIMALS / years_u128,
						precision
					),
					"left = {:?}, right = {:?}",
					inflation.collator.reward_rate.per_block * 10_000 * DECIMALS,
					Perquintill::from_percent(15) * 10_000 * DECIMALS / years_u128,
				);
				assert_eq!(inflation.delegator.max_rate, Perquintill::from_percent(40));
				assert_eq!(inflation.delegator.reward_rate.annual, Perquintill::from_percent(10));
				assert!(
					almost_equal(
						inflation.delegator.reward_rate.per_block * DECIMALS * 10_000,
						Perquintill::from_percent(10) * 10_000 * DECIMALS / years_u128,
						precision
					),
					"left = {:?}, right = {:?}",
					inflation.delegator.reward_rate.per_block * DECIMALS * 10_000,
					Perquintill::from_percent(10) * 10_000 * DECIMALS / years_u128,
				);

				// Check collator reward computation
				assert_eq!(inflation.collator.compute_block_rewards::<Test>(0, 100_000), 0);
				assert!(
					almost_equal(
						inflation
							.collator
							.compute_block_rewards::<Test>(5000 * DECIMALS, 100_000 * DECIMALS)
							* years_u128,
						Perquintill::from_percent(15) * 5000 * DECIMALS,
						Perbill::from_percent(1)
					),
					"left = {:?}, right = {:?}",
					inflation
						.collator
						.compute_block_rewards::<Test>(5000 * DECIMALS, 100_000 * DECIMALS)
						* years_u128,
					Perquintill::from_percent(15) * 5000 * DECIMALS,
				);
				// Check for max_rate which is 10%
				assert_eq!(
					inflation.collator.compute_block_rewards::<Test>(10_000, 100_000),
					inflation.collator.compute_block_rewards::<Test>(100_000, 100_000)
				);
				// Stake can never be more than what is issued, but let's check whether the cap
				// still applies
				assert_eq!(
					inflation.collator.compute_block_rewards::<Test>(10_000, 100_000),
					inflation.collator.compute_block_rewards::<Test>(100_001, 100_000)
				);

				// Check delegator reward computation
				assert_eq!(inflation.delegator.compute_block_rewards::<Test>(0, 100_000), 0);
				assert!(
					almost_equal(
						inflation
							.delegator
							.compute_block_rewards::<Test>(5000 * DECIMALS, 100_000 * DECIMALS)
							* years_u128,
						Perquintill::from_percent(10) * 5000 * DECIMALS,
						Perbill::from_percent(1)
					),
					"left = {:?}, right = {:?}",
					inflation
						.delegator
						.compute_block_rewards::<Test>(5000 * DECIMALS, 100_000 * DECIMALS)
						* years_u128,
					Perquintill::from_percent(10) * 5000 * DECIMALS,
				);
				// Check for max_rate which is 40%
				assert_eq!(
					inflation.delegator.compute_block_rewards::<Test>(40_000, 100_000),
					inflation.delegator.compute_block_rewards::<Test>(100_000, 100_000)
				);
				// Stake can never be more than what is issued, but let's check whether the cap
				// still applies
				assert_eq!(
					inflation.delegator.compute_block_rewards::<Test>(40_000, 100_000),
					inflation.delegator.compute_block_rewards::<Test>(100_001, 100_000)
				);
			});
	}
}
