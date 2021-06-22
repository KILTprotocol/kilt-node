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
use kilt_primitives::constants::YEARS;
use parity_scale_codec::{Decode, Encode};
use sp_runtime::{traits::Saturating, Perquintill, RuntimeDebug};

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
	rate / YEARS.max(1)
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
	///
	/// NOTE: If we exceed the max staking rate, the reward will be reduced by
	/// max_rate / current_rate.
	pub fn compute_reward<T: Config>(
		&self,
		stake: BalanceOf<T>,
		current_staking_rate: Perquintill,
		authors_per_round: BalanceOf<T>,
	) -> BalanceOf<T> {
		// Perquintill automatically bounds to [0, 100]% in case staking_rate is greater
		// than self.max_rate
		let reduction = Perquintill::from_rational(self.max_rate.deconstruct(), current_staking_rate.deconstruct());
		// multiplication with perbill cannot overflow
		let reward = (self.reward_rate.per_block * stake).saturating_mul(authors_per_round);
		reduction * reward
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

	/// Check whether the annual reward rate is approx. the per_block reward
	/// rate multiplied with the number of blocks per year
	pub fn is_valid(&self) -> bool {
		let years = YEARS;
		self.collator.reward_rate.annual
			>= Perquintill::from_parts(self.collator.reward_rate.per_block.deconstruct().saturating_mul(years))
			&& self.delegator.reward_rate.annual
				>= Perquintill::from_parts(self.delegator.reward_rate.per_block.deconstruct().saturating_mul(years))
	}
}

#[cfg(test)]
mod tests {
	use sp_runtime::Perbill;

	use super::*;
	use crate::mock::{almost_equal, ExtBuilder, Test, DECIMALS};
	use kilt_primitives::constants::YEARS;

	#[test]
	fn perquintill() {
		assert_eq!(
			Perquintill::from_percent(100) * Perquintill::from_percent(50),
			Perquintill::from_percent(50)
		);
	}

	#[allow(clippy::assertions_on_constants)]
	#[test]
	fn blocks_per_year_saturation() {
		assert!(YEARS < u64::MAX);
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
				let authors_per_round = 1u128;
				let mut current_staking_rate: Perquintill = inflation.collator.max_rate;
				assert_eq!(
					inflation
						.collator
						.compute_reward::<Test>(0, current_staking_rate, authors_per_round),
					0
				);
				current_staking_rate = Perquintill::from_rational(5000u64, 100_000u64);
				assert!(
					almost_equal(
						inflation.collator.compute_reward::<Test>(
							5000 * DECIMALS,
							current_staking_rate,
							authors_per_round
						) * years_u128,
						Perquintill::from_percent(15) * 5000 * DECIMALS,
						Perbill::from_percent(1)
					),
					"left = {:?}, right = {:?}",
					inflation
						.collator
						.compute_reward::<Test>(5000 * DECIMALS, current_staking_rate, authors_per_round)
						* years_u128,
					Perquintill::from_percent(15) * 5000 * DECIMALS,
				);
				// Check for max_rate which is 10%
				current_staking_rate = Perquintill::from_rational(10_000u64, 100_000u64);
				assert!(
					almost_equal(
						inflation.collator.compute_reward::<Test>(
							10_000 * DECIMALS,
							current_staking_rate,
							authors_per_round
						) * years_u128,
						Perquintill::from_percent(15) * 10_000 * DECIMALS,
						Perbill::from_percent(1)
					),
					"left = {:?}, right = {:?}",
					inflation.collator.compute_reward::<Test>(
						10_000 * DECIMALS,
						current_staking_rate,
						authors_per_round
					) * years_u128,
					Perquintill::from_percent(15) * 10_000 * DECIMALS,
				);

				// Check for exceeding max_rate: 50% instead of 10%
				current_staking_rate = Perquintill::from_rational(50_000u64, 100_000u64);
				assert!(
					almost_equal(
						inflation.collator.compute_reward::<Test>(
							50_000 * DECIMALS,
							current_staking_rate,
							authors_per_round
						) * years_u128,
						Perquintill::from_percent(15) * 10_000 * DECIMALS,
						Perbill::from_percent(1)
					),
					"left = {:?}, right = {:?}",
					inflation.collator.compute_reward::<Test>(
						50_000 * DECIMALS,
						current_staking_rate,
						authors_per_round
					) * years_u128,
					Perquintill::from_percent(15) * 10_000 * DECIMALS,
				);

				// Check delegator reward computation
				current_staking_rate = inflation.delegator.max_rate;
				assert_eq!(
					inflation
						.delegator
						.compute_reward::<Test>(0, current_staking_rate, authors_per_round),
					0
				);
				current_staking_rate = Perquintill::from_rational(5000u64, 100_000u64);
				assert!(
					almost_equal(
						inflation.delegator.compute_reward::<Test>(
							5000 * DECIMALS,
							current_staking_rate,
							authors_per_round
						) * years_u128,
						Perquintill::from_percent(10) * 5000 * DECIMALS,
						Perbill::from_percent(1)
					),
					"left = {:?}, right = {:?}",
					inflation.delegator.compute_reward::<Test>(
						5000 * DECIMALS,
						current_staking_rate,
						authors_per_round
					) * years_u128,
					Perquintill::from_percent(10) * 5000 * DECIMALS,
				);
				// Check for max_rate which is 40%
				current_staking_rate = Perquintill::from_rational(40_000u64, 100_000u64);
				assert!(
					almost_equal(
						inflation.delegator.compute_reward::<Test>(
							40_000 * DECIMALS,
							current_staking_rate,
							authors_per_round
						) * years_u128,
						Perquintill::from_percent(10) * 40_000 * DECIMALS,
						Perbill::from_percent(1)
					),
					"left = {:?}, right = {:?}",
					inflation.delegator.compute_reward::<Test>(
						40_000 * DECIMALS,
						current_staking_rate,
						authors_per_round
					) * years_u128,
					Perquintill::from_percent(10) * 40_000 * DECIMALS,
				);

				// Check for exceeding max_rate: 50% instead of 40%
				current_staking_rate = Perquintill::from_rational(50_000u64, 100_000u64);
				assert!(
					almost_equal(
						inflation.delegator.compute_reward::<Test>(
							50_000 * DECIMALS,
							current_staking_rate,
							authors_per_round
						) * years_u128,
						Perquintill::from_percent(8) * 50_000 * DECIMALS,
						Perbill::from_percent(1)
					),
					"left = {:?}, right = {:?}",
					inflation.delegator.compute_reward::<Test>(
						50_000 * DECIMALS,
						current_staking_rate,
						authors_per_round
					) * years_u128,
					Perquintill::from_percent(8) * 50_000 * DECIMALS,
				);
			});
	}
}
