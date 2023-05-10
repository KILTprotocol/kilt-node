use frame_support::{ensure, traits::Get};
use sp_runtime::{traits::Zero, SaturatedConversion, Saturating};

use crate::{
	types::{BalanceOf, Candidate},
	CandidatePool, Config, DelegatorState, LastDelegation, MaxCollatorCandidateStake, MaxSelectedCandidates, Pallet,
	Round, TopCandidates, TotalCollatorStake,
};

pub fn validate_candiate_pool<T: Config>() -> Result<(), &'static str> {
	// check if enough collators are set.
	ensure!(
		CandidatePool::<T>::count() >= T::MinCollators::get(),
		"Staking: Not enough collators are present."
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
				"Staking: Total stake of a collator can not be reconstructed."
			);

			// Min required stake should be set
			ensure!(
				candidate.stake >= T::MinCollatorCandidateStake::get(),
				"Staking: Insufficient stake from a collator."
			);

			// delegators should be in delegator pool.
			let are_delegator_present = candidate
				.delegators
				.iter()
				.map(|delegator_stake| DelegatorState::<T>::get(&delegator_stake.owner).is_some())
				.all(|x| x);
			ensure!(are_delegator_present, "Staking: Delegator is not present");

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
						"Staking: Exceeded delegations per round by a collator."
					);

					Ok(())
				})?;

			// check min and max stake for each candidate
			ensure!(
				candidate.stake <= MaxCollatorCandidateStake::<T>::get(),
				"Staking: Exceeded stake by a collator."
			);

			ensure!(
				candidate.stake >= T::MinCollatorStake::get(),
				"Staking: Lag behind stake by a collator."
			);

			// delegators should have the min required stake.
			candidate
				.delegators
				.iter()
				.try_for_each(|delegator_stake| -> Result<(), &'static str> {
					ensure!(
						delegator_stake.amount >= T::MinDelegatorStake::get(),
						"Staking: Lag behind stake of a delegator"
					);
					Ok(())
				})?;

			Ok(())
		},
	)
}

pub fn validate_top_candidates<T: Config>() -> Result<(), &'static str> {
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

pub fn validate_stake<T: Config>() -> Result<(), &'static str> {
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
		"Total collator stake not matching."
	);

	ensure!(
		total_stake.delegators == delegator_state,
		"Total delegator stake is not matching."
	);

	Ok(())
}
