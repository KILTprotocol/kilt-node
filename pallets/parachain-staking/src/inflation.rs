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
use crate::pallet::{BalanceOf, Config, Pallet, Round, RoundInfo};
use frame_support::traits::Currency;
use parity_scale_codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{traits::Saturating, PerThing, Perbill, RuntimeDebug};
use substrate_fixed::{
	transcendental::pow as floatpow,
	types::{I32F32, I64F64},
};

// TODO: use constants from kilt_primitives
const SECONDS_PER_YEAR: u32 = 31557600;
const SECONDS_PER_BLOCK: u32 = 6;
// = 5.259.600
const BLOCKS_PER_YEAR: u32 = SECONDS_PER_YEAR / SECONDS_PER_BLOCK;

fn rounds_per_year<T: Config>() -> u32 {
	let blocks_per_round = <Pallet<T>>::round().length;
	BLOCKS_PER_YEAR / blocks_per_round
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Eq, PartialEq, Clone, Encode, Decode, Default, RuntimeDebug)]
pub struct RewardRate {
	pub annual: Perbill,
	pub round: Perbill,
}

/// Convert annual inflation rate range to round inflation range
pub fn annual_to_round<T: Config>(rate: Perbill) -> Perbill {
	let periods = rounds_per_year::<T>();
	// TODO: Add check if periods are 0
	perbill_annual_to_perbill_round(rate, periods)
}

// TODO: Check if we need this. See here https://github.com/PureStake/moonbeam/pull/366
/// Convert an annual inflation to a round inflation
/// round = 1 - (1+annual)^(1/rounds_per_year)
fn perbill_annual_to_perbill_round(annual: Perbill, rounds_per_year: u32) -> Perbill {
	// let exponent = I32F32::from_num(1) / I32F32::from_num(rounds_per_year);

	// let x = I32F32::from_num(annual.deconstruct()) /
	// I32F32::from_num(Perbill::ACCURACY); let y: I64F64 =
	// floatpow(I32F32::from_num(1) + x, exponent).expect( 	"Cannot overflow since
	// rounds_per_year is u32 so worst case 0; QED",
	// );
	// Perbill::from_parts(
	// 	((y - I64F64::from_num(1)) * I64F64::from_num(Perbill::ACCURACY))
	// 		.ceil()
	// 		.to_num::<u32>(),
	// )
	Perbill::from_parts(annual.deconstruct() / rounds_per_year)
}

impl RewardRate {
	pub fn new<T: Config>(rate: Perbill) -> Self {
		RewardRate {
			annual: rate,
			round: annual_to_round::<T>(rate),
		}
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
		let reward_rate = staking_rate * self.reward_rate.round;
		println!(
			"compute_rewards: {:?} {:?} | {:?} * {:?} = {:?}",
			staking_rate,
			self.reward_rate.round,
			reward_rate,
			total_issuance,
			reward_rate * total_issuance,
		);
		reward_rate * total_issuance
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
	) -> InflationInfo {
		InflationInfo {
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
	use crate::mock::{ExtBuilder, Test};

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
			// .with_inflation(staking_info.clone())
			.build()
			.execute_with(|| {
				// Unrealistic configuration but makes computation simple
				<Round<Test>>::put(RoundInfo {
					current: 1,
					first: 1,
					length: BLOCKS_PER_YEAR,
				});
				// let rounds_per_year = BLOCKS_PER_YEAR / <Test as
				// Config>::DefaultBlocksPerRound::get();
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
		ExtBuilder::default().build().execute_with(|| {
			let rounds_per_year = BLOCKS_PER_YEAR / <Test as Config>::DefaultBlocksPerRound::get();
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
			assert_eq!(inflation.collator.compute_rewards::<Test>(0, 160_000_000), 0);
			assert_eq!(inflation.collator.compute_rewards::<Test>(100_000, 160_000_000), 2);
			// Check for max_rate which is 10%
			assert_eq!(inflation.collator.compute_rewards::<Test>(16_000_000, 160_000_000), 274);
			// Check exceeding max_rate
			assert_eq!(inflation.collator.compute_rewards::<Test>(32_000_000, 160_000_000), 274);
			// Stake can never be more than what is issued, but let's check whether the cap
			// still applies
			assert_eq!(
				inflation.collator.compute_rewards::<Test>(200_000_000, 160_000_000),
				274
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
				730
			);
			// Check exceeding max_rate
			assert_eq!(
				inflation.delegator.compute_rewards::<Test>(100_000_000, 160_000_000),
				730
			);
			// Stake can never be more than what is issued, but let's check whether the cap
			// still applies
			assert_eq!(
				inflation.delegator.compute_rewards::<Test>(200_000_000, 160_000_000),
				730
			);
		});
	}

	#[test]
	fn real_rewards() {
		ExtBuilder::default().build().execute_with(|| {
			let rounds_per_year = BLOCKS_PER_YEAR / <Test as Config>::DefaultBlocksPerRound::get();
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

			let decimals = 10u128.pow(15);
			let total_issuance: u128 = 160_000_000u128 * decimals;

			// Check collator reward computation
			assert_eq!(inflation.collator.compute_rewards::<Test>(0, total_issuance), 0);
			assert_eq!(
				inflation
					.collator
					.compute_rewards::<Test>(100_000 * decimals, total_issuance),
				1600 * 10u128.pow(12)
			);
			// Check for max_rate which is 10%
			assert_eq!(
				inflation
					.collator
					.compute_rewards::<Test>(16_000_000 * decimals, total_issuance),
				273760 * 10u128.pow(12)
			);
			// Check exceeding max_rate
			assert_eq!(
				inflation
					.collator
					.compute_rewards::<Test>(32_000_000 * decimals, total_issuance),
				273760 * 10u128.pow(12)
			);
			// Stake can never be more than what is issued, but let's check whether the cap
			// still applies
			assert_eq!(
				inflation
					.collator
					.compute_rewards::<Test>(200_000_000 * decimals, total_issuance),
				273760 * 10u128.pow(12)
			);

			// Check delegator reward calculation
			assert_eq!(inflation.delegator.compute_rewards::<Test>(0, total_issuance), 0);
			assert_eq!(
				inflation
					.delegator
					.compute_rewards::<Test>(100_000 * decimals, total_issuance),
				1120 * 10u128.pow(12)
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
				729_920 * 10u128.pow(12)
			);
			// Check exceeding max_rate
			assert_eq!(
				inflation
					.delegator
					.compute_rewards::<Test>(100_000_000 * decimals, total_issuance),
				729_920 * 10u128.pow(12)
			);
			// Stake can never be more than what is issued, but let's check whether the cap
			// still applies
			assert_eq!(
				inflation
					.delegator
					.compute_rewards::<Test>(200_000_000 * decimals, total_issuance),
				729_920 * 10u128.pow(12)
			);
		});
	}
}
