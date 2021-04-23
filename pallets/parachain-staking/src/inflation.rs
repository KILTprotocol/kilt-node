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
use sp_runtime::{traits::Saturating, Perbill, RuntimeDebug};

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
		let rate = Perbill::from_rational(stake, total_issuance).max(self.max_rate);
		let reward_rate = rate * self.reward_rate;
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
