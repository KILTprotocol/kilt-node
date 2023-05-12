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

use frame_support::{ensure, traits::Get};
use sp_runtime::{traits::Zero, SaturatedConversion, Saturating};

use crate::{
	types::{BalanceOf, Candidate},
	CandidatePool, Config, DelegatorState, LastDelegation, MaxCollatorCandidateStake, MaxSelectedCandidates, Pallet,
	Round, TopCandidates, TotalCollatorStake,
};

pub(crate) fn do_try_state<T: Config>() -> Result<(), &'static str> {
	validate_candiate_pool::<T>()?;
	validate_top_candidates::<T>()?;
	validate_stake::<T>()
}

fn validate_candiate_pool<T: Config>() -> Result<(), &'static str> {
	// check if enough collators are set.
	ensure!(
		CandidatePool::<T>::count() >= T::MinCollators::get(),
		"Insufficient collators"
	);

	CandidatePool::<T>::iter_values().try_for_each(
		|candidate: Candidate<T::AccountId, BalanceOf<T>, _>| -> Result<(), &'static str> {
			let sum_delegations: BalanceOf<T> = candidate
				.delegators
				.iter()
				.fold(Zero::zero(), |acc, stake| acc.saturating_add(stake.amount));

			// total stake should be the sum of delegators stake + colator stake.
			ensure!(
				sum_delegations.saturating_add(candidate.stake) == candidate.total,
				"Corrupted collator stake"
			);

			// Min required stake should be set
			ensure!(
				candidate.stake >= T::MinCollatorCandidateStake::get(),
				"Insufficient collator stake"
			);

			// delegators should be in delegator pool.
			let are_delegator_present = candidate
				.delegators
				.iter()
				.map(|delegator_stake| DelegatorState::<T>::get(&delegator_stake.owner).is_some())
				.all(|x| x);
			ensure!(are_delegator_present, "Unknown delegator");

			// each delegator should not exceed the [MaxDelegationsPerRound]
			candidate
				.delegators
				.iter()
				.try_for_each(|delegator_stake| -> Result<(), &'static str> {
					let last_delegation = LastDelegation::<T>::get(&delegator_stake.owner);
					let round = Round::<T>::get();
					let counter = if last_delegation.round < round.current {
						0u32
					} else {
						last_delegation.counter
					};

					ensure!(
						counter <= T::MaxDelegationsPerRound::get(),
						"Exceeded delegations per round"
					);

					Ok(())
				})?;

			// check min and max stake for each candidate
			ensure!(
				candidate.stake <= MaxCollatorCandidateStake::<T>::get(),
				"Exceeded collator stake"
			);

			ensure!(
				candidate.stake >= T::MinCollatorStake::get(),
				"Insufficient collator stake"
			);

			// delegators should have the min required stake.
			candidate
				.delegators
				.iter()
				.try_for_each(|delegator_stake| -> Result<(), &'static str> {
					ensure!(
						delegator_stake.amount >= T::MinDelegatorStake::get(),
						"Insufficient delegator stake"
					);
					Ok(())
				})?;

			Ok(())
		},
	)
}

fn validate_top_candidates<T: Config>() -> Result<(), &'static str> {
	let top_candidates = TopCandidates::<T>::get();

	// check if enough top candidates are set.
	ensure!(
		top_candidates.len() >= T::MinRequiredCollators::get().saturated_into(),
		"Insufficient collators"
	);

	top_candidates.iter().try_for_each(|stake| -> Result<(), &'static str> {
		// top candidates should be part of the candidate pool.
		ensure!(CandidatePool::<T>::contains_key(&stake.owner), "Unknown candidate");

		// an account can not be candidate and delegator.
		ensure!(
			DelegatorState::<T>::get(&stake.owner).is_none(),
			"Account is candidate and delegator."
		);

		// a top candidate should be active.
		ensure!(
			Pallet::<T>::is_active_candidate(&stake.owner).unwrap(),
			"Inactive candidate"
		);

		Ok(())
	})
}

fn validate_stake<T: Config>() -> Result<(), &'static str> {
	// the total fund has to be the sum over the first [MaxSelectedCandidates] of
	// [TopCandidates].

	let top_candidates = TopCandidates::<T>::get();
	let top_n = MaxSelectedCandidates::<T>::get().saturated_into::<usize>();

	let total_stake = TotalCollatorStake::<T>::get();

	let collator_delegator_stake = top_candidates
		.iter()
		.take(top_n)
		.fold(Zero::zero(), |acc: BalanceOf<T>, details| {
			acc.saturating_add(details.amount)
		});

	let collator_stake = top_candidates
		.iter()
		.take(top_n)
		.filter_map(|stake| CandidatePool::<T>::get(&stake.owner))
		.fold(Zero::zero(), |acc: BalanceOf<T>, candidate| {
			acc.saturating_add(candidate.stake)
		});

	let delegator_state = collator_delegator_stake.saturating_sub(collator_stake);

	ensure!(
		total_stake.collators == collator_stake,
		"Corrupted total collator stake"
	);

	ensure!(
		total_stake.delegators == delegator_state,
		"Corrupted total delegator stake"
	);

	Ok(())
}
