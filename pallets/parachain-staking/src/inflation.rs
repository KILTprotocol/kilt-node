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
use parity_scale_codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{traits::Saturating, PerThing, Perbill, RuntimeDebug};

// TODO: use constants from kilt_primitives
const SECONDS_PER_YEAR: u32 = 31557600;
const SECONDS_PER_BLOCK: u32 = 6;
const BLOCKS_PER_YEAR: u32 = SECONDS_PER_YEAR / SECONDS_PER_BLOCK;

fn rounds_per_year<T: Config>() -> u32 {
	let blocks_per_round = <Pallet<T>>::round().length;
	BLOCKS_PER_YEAR / blocks_per_round
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Eq, PartialEq, Clone, Encode, Decode, Default, RuntimeDebug)]
pub struct StakingRates {
	/// Maximum staking rate
	pub max_rate: Perbill,
	/// Reward rate
	pub reward_rate: Perbill,
}

impl StakingRates {
	/// Set max staking rate
	pub fn set_max_rate(&mut self, max_rate: Perbill) {
		self.max_rate = max_rate;
	}

	/// Set reward rate
	pub fn set_rewards(&mut self, reward_rate: Perbill) {
		self.reward_rate = reward_rate;
	}

	/// Convert annual inflation rate range to round inflation range
	pub fn annual_to_round<T: Config>(&self) -> StakingRates {
		let periods = rounds_per_year::<T>();
		StakingRates {
			// TODO: Probably want to switch to saturating_div
			max_rate: self.max_rate / periods,
			reward_rate: self.reward_rate / periods,
		}
	}

	pub fn compute_rewards<T: Config>(&self, stake: BalanceOf<T>, total_issuance: BalanceOf<T>) -> BalanceOf<T> {
		// TODO: saturated_div?
		let rate = Perbill::from_rational(stake, total_issuance).min(self.max_rate);
		let reward_rate = Perbill::from_parts(rate.deconstruct() * self.reward_rate.deconstruct());
		println!(
			"compute_rewards: {:?} * {:?} = {:?}",
			reward_rate,
			total_issuance,
			reward_rate * total_issuance,
		);
		reward_rate * total_issuance
	}
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Eq, PartialEq, Clone, Encode, Decode, Default, RuntimeDebug)]
pub struct StakingInfo {
	// Collator staking rates
	pub collator: StakingRates,
	// Delegator staking rates
	pub delegator: StakingRates,
}

impl StakingInfo {
	/// Check whether rates are in the interval [0, 1)
	pub fn is_valid(&self) -> bool {
		self.collator.max_rate >= Perbill::zero()
			&& self.collator.reward_rate >= Perbill::zero()
			&& self.delegator.max_rate >= Perbill::zero()
			&& self.delegator.reward_rate >= Perbill::zero()
			&& self.collator.max_rate < Perbill::one()
			&& self.collator.reward_rate < Perbill::one()
			&& self.delegator.max_rate < Perbill::one()
			&& self.delegator.reward_rate < Perbill::one()
	}
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Eq, PartialEq, Clone, Encode, Decode, Default, RuntimeDebug)]
pub struct InflationInfo {
	pub annual: StakingInfo,
	pub round: StakingInfo,
}

impl InflationInfo {
	pub fn new<T: Config>(inflation: StakingInfo) -> InflationInfo {
		InflationInfo {
			annual: inflation.clone(),
			round: StakingInfo {
				collator: inflation.collator.annual_to_round::<T>(),
				delegator: inflation.delegator.annual_to_round::<T>(),
			},
		}
	}

	pub fn round_issuance<T: Config>(
		&self,
		collator_stake: BalanceOf<T>,
		delegator_stake: BalanceOf<T>,
	) -> (BalanceOf<T>, BalanceOf<T>) {
		let circulating = T::Currency::total_issuance();

		let collator_rewards = self.round.collator.compute_rewards::<T>(collator_stake, circulating);
		let delegator_rewards = self.round.delegator.compute_rewards::<T>(delegator_stake, circulating);

		(collator_rewards, delegator_rewards)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::mock::{ExtBuilder, Test};

	fn mock_annual_to_round(annual: StakingInfo, rounds_per_year: u32) -> StakingInfo {
		StakingInfo {
			collator: StakingRates {
				max_rate: Perbill::from_parts(annual.collator.max_rate.deconstruct() / rounds_per_year),
				reward_rate: Perbill::from_parts(annual.collator.reward_rate.deconstruct() / rounds_per_year),
			},
			delegator: StakingRates {
				max_rate: Perbill::from_parts(annual.delegator.max_rate.deconstruct() / rounds_per_year),
				reward_rate: Perbill::from_parts(annual.delegator.reward_rate.deconstruct() / rounds_per_year),
			},
		}
	}

	#[test]
	fn simple_rewards() {
		let collator = StakingRates {
			max_rate: Perbill::from_percent(10),
			reward_rate: Perbill::from_percent(15),
		};
		let delegator = StakingRates {
			max_rate: Perbill::from_percent(40),
			reward_rate: Perbill::from_percent(10),
		};
		let staking_info = StakingInfo {
			collator: collator.clone(),
			delegator: delegator.clone(),
		};

		ExtBuilder::default()
			// .with_inflation(staking_info.clone())
			.build()
			.execute_with(|| {
				let rounds_per_year = BLOCKS_PER_YEAR / <Test as Config>::DefaultBlocksPerRound::get();
				let expected_inflation = InflationInfo::new::<Test>(staking_info.clone());

				assert_eq!(
					expected_inflation.round,
					mock_annual_to_round(staking_info, rounds_per_year)
				);

				// Check collator reward computation
				assert_eq!(collator.compute_rewards::<Test>(0, 100_000), 0);
				assert_eq!(collator.compute_rewards::<Test>(5000, 100_000), 750);
				// Check for max_rate which is 10%
				assert_eq!(collator.compute_rewards::<Test>(10_000, 100_000), 1500);
				// Check exceeding max_rate
				assert_eq!(collator.compute_rewards::<Test>(100_000, 100_000), 1500);
				// Stake can never be more than what is issued, but let's check whether the cap
				// still applies
				assert_eq!(collator.compute_rewards::<Test>(100_001, 100_000), 1500);

				// Check delegator reward calculation
				assert_eq!(delegator.compute_rewards::<Test>(0, 100_000), 0);
				assert_eq!(delegator.compute_rewards::<Test>(5000, 100_000), 500);
				// Check for max_rate which is 40%
				assert_eq!(delegator.compute_rewards::<Test>(40_000, 100_000), 4000);
				// Check exceeding max_rate
				assert_eq!(delegator.compute_rewards::<Test>(100_000, 100_000), 4000);
				// Stake can never be more than what is issued, but let's check whether the cap
				// still applies
				assert_eq!(delegator.compute_rewards::<Test>(100_001, 100_000), 4000);
			});
	}

	#[test]
	fn rewards_only_collator() {
		ExtBuilder::default()
			.with_inflation(10, 15, 40, 0)
			.build()
			.execute_with(|| {
				let rounds_per_year = BLOCKS_PER_YEAR / <Test as Config>::DefaultBlocksPerRound::get();

				let collator = StakingRates {
					max_rate: Perbill::from_percent(10),
					reward_rate: Perbill::from_percent(15),
				};
				let delegator = StakingRates {
					max_rate: Perbill::from_percent(40),
					reward_rate: Perbill::from_percent(0),
				};
				let staking_info = StakingInfo {
					collator: collator.clone(),
					delegator: delegator.clone(),
				};

				let expected_inflation = InflationInfo::new::<Test>(staking_info.clone());

				assert_eq!(
					expected_inflation.round,
					mock_annual_to_round(staking_info, rounds_per_year)
				);

				// Check collator reward computation
				assert_eq!(collator.compute_rewards::<Test>(0, 100_000), 0);
				assert_eq!(collator.compute_rewards::<Test>(5000, 100_000), 750);
				// Check for max_rate which is 10%
				assert_eq!(collator.compute_rewards::<Test>(10_000, 100_000), 1500);
				// Check exceeding max_rate
				assert_eq!(collator.compute_rewards::<Test>(100_000, 100_000), 1500);
				// Stake can never be more than what is issued, but let's check whether the cap
				// still applies
				assert_eq!(collator.compute_rewards::<Test>(100_001, 100_000), 1500);

				// Check delegator reward calculation
				assert_eq!(delegator.compute_rewards::<Test>(0, 100_000), 0);
				assert_eq!(delegator.compute_rewards::<Test>(5000, 100_000), 0);
				// Check for max_rate which is 40%
				assert_eq!(delegator.compute_rewards::<Test>(40_000, 100_000), 0);
				// Check exceeding max_rate
				assert_eq!(delegator.compute_rewards::<Test>(100_000, 100_000), 0);
				// Stake can never be more than what is issued, but let's check whether the cap
				// still applies
				assert_eq!(delegator.compute_rewards::<Test>(100_001, 100_000), 0);
			});
	}
}
