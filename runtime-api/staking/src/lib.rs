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

#![cfg_attr(not(feature = "std"), no_std)]

use parity_scale_codec::{Codec, Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::Perquintill;

#[derive(Decode, Encode, TypeInfo, MaxEncodedLen, PartialEq, Eq, Debug)]
pub struct StakingRates {
	pub collator_staking_rate: Perquintill,
	pub collator_reward_rate: Perquintill,
	pub delegator_staking_rate: Perquintill,
	pub delegator_reward_rate: Perquintill,
}

sp_api::decl_runtime_apis! {
	/// The API to query staking and reward rates.
	pub trait Staking<AccountId, Balance>
	where
		AccountId: Codec,
		Balance: Codec
	{
		/// Returns the current staking rewards for a given account address.
		fn get_unclaimed_staking_rewards(account: &AccountId) -> Balance;
		/// Returns the current staking and reward rates for collators and
		/// delegators.
		fn get_staking_rates() -> StakingRates;
	}
}
