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
use kilt_support::test_utils::log_and_return_error_message;
use scale_info::prelude::format;
use sp_runtime::{
	traits::{CheckedAdd, Zero},
	SaturatedConversion, Saturating,
};

use crate::{
	set::OrderedSet,
	types::{BalanceOf, Candidate, Stake},
	CandidatePool, Config, DelegatorState, LastDelegation, MaxCollatorCandidateStake, MaxSelectedCandidates, Pallet,
	Round, TopCandidates, TotalCollatorStake,
};

pub(crate) fn do_try_state<T: Config>() -> Result<(), &'static str> {
	validate_candiate_pool::<T>()?;
	validate_delegators::<T>()?;
	validate_top_candidates::<T>()?;
	validate_stake::<T>()
}

fn validate_candiate_pool<T: Config>() -> Result<(), &'static str> {
	// check if enough collators are set.
	ensure!(
		CandidatePool::<T>::count() >= T::MinCollators::get(),
		log_and_return_error_message(format!(
			"Insufficient collators. Collators count: {:?}. Min required collators: {:?}",
			CandidatePool::<T>::count(),
			T::MinCollators::get()
		))
	);

	CandidatePool::<T>::iter_values().try_for_each(
		|candidate: Candidate<T::AccountId, BalanceOf<T>, _>| -> Result<(), &'static str> {
			let sum_delegations: BalanceOf<T> = candidate
				.delegators
				.iter()
				.fold(Zero::zero(), |acc, stake| acc.saturating_add(stake.amount));

			// total stake should be the sum of delegators stake + colator stake.
			let stake_total = sum_delegations.checked_add(&candidate.stake);
			ensure!(
				stake_total == Some(candidate.total),
				log_and_return_error_message(format!(
					"Total stake of collator {:?} does not match. Saved stake: {:?}. Calculated stake: {:?}",
					candidate.id, candidate.stake, stake_total
				))
			);

			// Min required stake should be set
			ensure!(
				candidate.stake >= T::MinCollatorCandidateStake::get(),
				log_and_return_error_message(format!(
					"Stake of collator {:?} insufficient. Required stake: {:?}. Owned Stake: {:?} ",
					candidate.id,
					T::MinCollatorCandidateStake::get(),
					candidate.stake
				))
			);

			validate_delegators_from_collator::<T>(candidate.delegators)?;

			// check min and max stake for each candidate
			ensure!(
				candidate.stake <= MaxCollatorCandidateStake::<T>::get(),
				log_and_return_error_message(format!(
					"Candidate {:?} exceeded stake. Allowed stake: {:?}. Owned Stake: {:?}",
					candidate.id,
					MaxCollatorCandidateStake::<T>::get(),
					candidate.stake
				))
			);

			Ok(())
		},
	)
}

fn validate_top_candidates<T: Config>() -> Result<(), &'static str> {
	let top_candidates = TopCandidates::<T>::get();

	// check if enough top candidates are set.
	ensure!(
		top_candidates.len() >= T::MinRequiredCollators::get().saturated_into(),
		log_and_return_error_message(format!(
			"Not enough candidates are set. Candidate count: {:?}. Required: {:?}",
			top_candidates.len(),
			T::MinRequiredCollators::get()
		))
	);

	top_candidates.iter().try_for_each(|stake| -> Result<(), &'static str> {
		// top candidates should be part of the candidate pool.
		ensure!(
			CandidatePool::<T>::contains_key(&stake.owner),
			log_and_return_error_message(format!("Unknown candidate {:?} in top candidates.", stake.owner))
		);

		// an account can not be candidate and delegator.
		ensure!(
			DelegatorState::<T>::get(&stake.owner).is_none(),
			log_and_return_error_message(format!("Account {:?} is delegator and candidate.", stake.owner))
		);

		// a top candidate should be active.
		ensure!(
			Pallet::<T>::is_active_candidate(&stake.owner).unwrap(),
			log_and_return_error_message(format!("Top candidate {:?} is inactive", stake.owner))
		);

		Ok(())
	})
}

fn validate_delegators_from_collator<T: Config>(
	delegators: OrderedSet<Stake<T::AccountId, BalanceOf<T>>, T::MaxDelegatorsPerCollator>,
) -> Result<(), &'static str> {
	delegators
		.iter()
		.try_for_each(|delegator_stake| -> Result<(), &'static str> {
			let last_delegation = LastDelegation::<T>::get(&delegator_stake.owner);
			let round = Round::<T>::get();
			let counter = if last_delegation.round < round.current {
				0u32
			} else {
				last_delegation.counter
			};

			// each delegator should not exceed the [MaxDelegationsPerRound]
			ensure!(
				counter <= T::MaxDelegationsPerRound::get(),
				log_and_return_error_message(format!(
					"Delegator {:?} exceeded delegations per round. Allowed delegations {:?}. Confirmed delegations {:?}",
					delegator_stake.owner, T::MaxDelegationsPerRound::get(), counter
				))
			);

			// each delegator should have the min required stake
			ensure!(
				delegator_stake.amount >= T::MinDelegatorStake::get(),
				log_and_return_error_message(format!(
					"Delegator {:?} insufficient stake. Required stake: {:?}. Owned stake: {:?}",
					delegator_stake.owner,
					T::MinDelegatorStake::get(),
					delegator_stake.amount
				))
			);

			ensure!(
				DelegatorState::<T>::get(&delegator_stake.owner).is_some(),
				log_and_return_error_message(format!("Unknown delegator {:?}", delegator_stake.owner))
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
		log_and_return_error_message(format!(
			"Corrupted total collator stake. Saved total stake: {:?}. Calculated stake: {:?}",
			total_stake.collators, collator_stake
		))
	);

	ensure!(
		total_stake.delegators == delegator_state,
		log_and_return_error_message(format!(
			"Corrupted total delegator stake. Saved total stake: {:?}. Calculated stake: {:?}",
			total_stake.delegators, delegator_state
		))
	);

	Ok(())
}

fn validate_delegators<T: Config>() -> Result<(), &'static str> {
	DelegatorState::<T>::iter_values().try_for_each(|delegator_details| -> Result<(), &'static str> {
		ensure!(
			CandidatePool::<T>::contains_key(&delegator_details.owner),
			log_and_return_error_message(format!("Collator {:?} not found", delegator_details.owner))
		);
		Ok(())
	})
}
