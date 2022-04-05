// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2022 BOTLabs GmbH

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
#![cfg(feature = "runtime-benchmarks")]

//! Benchmarking
use crate::{types::RoundInfo, *};
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, Zero};
use frame_support::{
	assert_ok,
	traits::{Currency, Get, OnInitialize},
};
use frame_system::{Pallet as System, RawOrigin};
use pallet_session::Pallet as Session;
use sp_runtime::{
	traits::{One, SaturatedConversion, StaticLookup},
	Perquintill,
};
use sp_std::{convert::TryInto, vec::Vec};

const COLLATOR_ACCOUNT_SEED: u32 = 0;
const DELEGATOR_ACCOUNT_SEED: u32 = 1;

/// Fills the candidate pool up to `num_candidates`.
fn setup_collator_candidates<T: Config>(
	num_candidates: u32,
	default_amount: Option<T::CurrencyBalance>,
) -> Vec<T::AccountId> {
	let current_collator_count = TopCandidates::<T>::get().len().saturated_into::<u32>();
	let collators: Vec<T::AccountId> = (current_collator_count..num_candidates)
		.map(|i| account("collator", i.saturated_into::<u32>(), COLLATOR_ACCOUNT_SEED))
		.collect();
	let amount: T::CurrencyBalance = default_amount.unwrap_or_else(T::MinCollatorCandidateStake::get);

	for acc in collators.iter() {
		T::Currency::make_free_balance_be(acc, amount);
		assert_ok!(<Pallet<T>>::join_candidates(
			T::Origin::from(Some(acc.clone()).into()),
			amount,
		));
		assert_eq!(<CandidatePool<T>>::get(acc).unwrap().stake, amount);
	}

	TopCandidates::<T>::get()
		.into_bounded_vec()
		.into_inner()
		.drain(..)
		.map(|c| c.owner)
		.collect()
}

fn fill_delegators<T: Config>(num_delegators: u32, collator: T::AccountId, collator_seed: u32) -> Vec<T::AccountId> {
	let state = <CandidatePool<T>>::get(&collator).unwrap();
	let current_delegators = state.delegators.len().saturated_into::<u32>();

	let delegators: Vec<T::AccountId> = (current_delegators..num_delegators)
		.map(|i| {
			account(
				"delegator",
				i.saturated_into::<u32>(),
				DELEGATOR_ACCOUNT_SEED * 1000 + collator_seed,
			)
		})
		.collect();

	for acc in delegators.iter() {
		T::Currency::make_free_balance_be(acc, T::MinDelegatorStake::get());
		assert_ok!(<Pallet<T>>::join_delegators(
			T::Origin::from(Some(acc.clone()).into()),
			T::Lookup::unlookup(collator.clone()),
			T::MinDelegatorStake::get(),
		));
	}

	delegators
}

// fills unstake BTreeMap by unstaked many entries of 1
fn fill_unstaking<T: Config>(collator: &T::AccountId, delegator: Option<&T::AccountId>, unstaked: u64)
where
	u64: Into<<T as frame_system::Config>::BlockNumber>,
{
	let who = delegator.unwrap_or(collator);
	assert_eq!(<Unstaking<T>>::get(who).len(), 0);
	while System::<T>::block_number() < unstaked.into() {
		if let Some(delegator) = delegator {
			assert_ok!(<Pallet<T>>::delegator_stake_less(
				RawOrigin::Signed(delegator.clone()).into(),
				T::Lookup::unlookup(collator.clone()),
				T::CurrencyBalance::one()
			));
		} else {
			assert_ok!(<Pallet<T>>::candidate_stake_less(
				RawOrigin::Signed(collator.clone()).into(),
				T::CurrencyBalance::one()
			));
		}
		System::<T>::set_block_number(System::<T>::block_number() + T::BlockNumber::one());
	}
	assert_eq!(<Unstaking<T>>::get(who).len() as u64, unstaked);
	assert!(<Unstaking<T>>::get(who).len() <= T::MaxUnstakeRequests::get().try_into().unwrap());
}

benchmarks! {
	where_clause { where u64: Into<<T as frame_system::Config>::BlockNumber> }

	on_initialize_no_action {
		assert_eq!(<Round<T>>::get().current, 0u32);
		let block = T::BlockNumber::one();
	}: { Pallet::<T>::on_initialize(block) }
	verify {
		assert_eq!(<Round<T>>::get().current, 0u32);
	}

	on_initialize_round_update {
		let round = <Round<T>>::get();
		assert_eq!(round.current, 0u32);
	}: { Pallet::<T>::on_initialize(round.length) }
	verify {
		assert_eq!(<Round<T>>::get().current, 1u32);
	}

	on_initialize_new_year {
		let old = <InflationConfig<T>>::get();
		assert_eq!(<LastRewardReduction<T>>::get(), T::BlockNumber::zero());
		let block = (T::BLOCKS_PER_YEAR + 1u32.into()).saturated_into::<T::BlockNumber>();
	}: { Pallet::<T>::on_initialize(block) }
	verify {
		let new = <InflationConfig<T>>::get();
		assert_eq!(<LastRewardReduction<T>>::get(), T::BlockNumber::one());
		assert_eq!(new.collator.max_rate, old.collator.max_rate);
		assert_eq!(new.delegator.max_rate, old.delegator.max_rate);
		assert!(new.collator.reward_rate.annual < old.collator.reward_rate.annual);
	}

	on_initialize_network_rewards {
		let issuance = T::Currency::total_issuance();
		// if we only add by one, we also initialize a new year
		let block = T::NetworkRewardStart::get() + T::BlockNumber::one() * 2_u64.into();
	}: { Pallet::<T>::on_initialize(block) }
	verify {
		let new_issuance = T::Currency::total_issuance();
		let max_col_reward = InflationConfig::<T>::get().collator.reward_rate.per_block * MaxCollatorCandidateStake::<T>::get() * MaxSelectedCandidates::<T>::get().into();
		let network_block_reward = T::NetworkRewardRate::get() * max_col_reward;
		assert!(new_issuance > issuance);
		assert_eq!(new_issuance - issuance, network_block_reward)
	}

	force_new_round {
		let round = <Round<T>>::get();
		let now = System::<T>::block_number();
		assert_eq!(round.current, 0);
		assert_eq!(Session::<T>::current_index(), 0);
		assert!(!<ForceNewRound<T>>::get());
	}: _(RawOrigin::Root)
	verify {
		assert!(<ForceNewRound<T>>::get());
		assert_eq!(Session::<T>::current_index(), 0);

		// jump to next block to trigger new round
		let now = now + T::BlockNumber::one();
		System::<T>::set_block_number(now);
		Session::<T>::on_initialize(now);
		assert_eq!(Session::<T>::current_index(), 1);
		assert_eq!(<Round<T>>::get(), RoundInfo {
			current: 1,
			first: now,
			length: round.length,
		});
		assert!(!<ForceNewRound<T>>::get());
	}

	set_inflation {
		let inflation = InflationInfo::new(
			T::BLOCKS_PER_YEAR.saturated_into(),
			Perquintill::from_percent(10),
			Perquintill::from_percent(15),
			Perquintill::from_percent(40),
			Perquintill::from_percent(10)
		);
	}: _(RawOrigin::Root, inflation.collator.max_rate, inflation.collator.reward_rate.annual, inflation.delegator.max_rate, inflation.delegator.reward_rate.annual)
	verify {
		assert_eq!(<InflationConfig<T>>::get(), inflation);
	}

	set_max_selected_candidates {
		let n in (T::MinCollators::get()) .. T::MaxTopCandidates::get();
		let m in 0 .. T::MaxDelegatorsPerCollator::get();

		let candidates = setup_collator_candidates::<T>(n, None);
		for (i, c) in candidates.iter().enumerate() {
			fill_delegators::<T>(m, c.clone(), i.saturated_into::<u32>());
		}
		let old_candidate = candidates[0].clone();
	}: _(RawOrigin::Root, n)
	verify {
		assert_eq!(<MaxSelectedCandidates<T>>::get(), n);
	}

	set_blocks_per_round {
		let bpr: T::BlockNumber = T::MinBlocksPerRound::get() + T::BlockNumber::one();
	}: _(RawOrigin::Root, bpr)
	verify {
		assert_eq!(<Round<T>>::get().length, bpr);
	}

	force_remove_candidate {
		let n in (T::MinCollators::get() + 1) .. T::MaxTopCandidates::get();
		let m in 0 .. T::MaxDelegatorsPerCollator::get();

		let candidates = setup_collator_candidates::<T>(n, None);
		for (i, c) in candidates.iter().enumerate() {
			fill_delegators::<T>(m, c.clone(), i.saturated_into::<u32>());
		}
		let candidate = candidates[0].clone();
		let unlookup_candidate = T::Lookup::unlookup(candidate.clone());
	}: _(RawOrigin::Root, unlookup_candidate)
	verify {
		let candidates = TopCandidates::<T>::get();
		assert!(!candidates.into_iter().any(|other| other.owner == candidate));
	}

	join_candidates {
		let n in 1 .. T::MaxTopCandidates::get() - 1;
		let m in 0 .. T::MaxDelegatorsPerCollator::get();

		let min_candidate_stake = T::MinCollatorCandidateStake::get();
		let candidates = setup_collator_candidates::<T>(n, None);
		for (i, c) in candidates.iter().enumerate() {
			fill_delegators::<T>(m, c.clone(), i.saturated_into::<u32>());
		}

		let new_candidate = account("new_collator", u32::MAX , COLLATOR_ACCOUNT_SEED);
		T::Currency::make_free_balance_be(&new_candidate, min_candidate_stake);

	}: _(RawOrigin::Signed(new_candidate.clone()), min_candidate_stake)
	verify {
		let candidates = TopCandidates::<T>::get();
		assert!(candidates.into_iter().any(|other| other.owner == new_candidate));
	}

	init_leave_candidates {
		let n in (T::MinCollators::get() + 1) .. T::MaxTopCandidates::get() - 1;
		let m in 0 .. T::MaxDelegatorsPerCollator::get();

		let candidates = setup_collator_candidates::<T>(n, None);
		for (i, c) in candidates.iter().enumerate() {
			fill_delegators::<T>(m, c.clone(), i.saturated_into::<u32>());
		}

		let now = <Round<T>>::get().current;
		let candidate = candidates[0].clone();

	}: _(RawOrigin::Signed(candidate.clone()))
	verify {
		let candidates = TopCandidates::<T>::get();
		assert!(!candidates.into_iter().any(|other| other.owner == candidate));
		let unlocking_at = now.saturating_add(T::ExitQueueDelay::get());
		assert!(<CandidatePool<T>>::get(candidate).unwrap().can_exit(unlocking_at));
	}

	cancel_leave_candidates {
		let n in (T::MinCollators::get() + 1) .. T::MaxTopCandidates::get() - 1;
		let m in 0 .. T::MaxDelegatorsPerCollator::get();

		let candidates = setup_collator_candidates::<T>(n, None);
		for (i, c) in candidates.iter().enumerate() {
			fill_delegators::<T>(m, c.clone(), i.saturated_into::<u32>());
		}

		let candidate = candidates[0].clone();
		assert_ok!(<Pallet<T>>::init_leave_candidates(RawOrigin::Signed(candidate.clone()).into()));

	}: _(RawOrigin::Signed(candidate.clone()))
	verify {
		let candidates = TopCandidates::<T>::get();
		assert!(candidates.into_iter().any(|other| other.owner == candidate));
	}

	execute_leave_candidates {
		let n in (T::MinCollators::get() + 1) .. T::MaxTopCandidates::get() - 1;
		let m in 0 .. T::MaxDelegatorsPerCollator::get();

		let u = T::MaxUnstakeRequests::get() as u32 - 1;
		let candidates = setup_collator_candidates::<T>(n, None);
		for (i, c) in candidates.iter().enumerate() {
			fill_delegators::<T>(m, c.clone(), i.saturated_into::<u32>());
		}
		let candidate = candidates[0].clone();

		// increase stake so we can unstake, because current stake is minimum
		let more_stake = T::MinCollatorCandidateStake::get();
		T::Currency::make_free_balance_be(&candidate, T::CurrencyBalance::from(u128::MAX));
		assert_ok!(<Pallet<T>>::candidate_stake_more(RawOrigin::Signed(candidate.clone()).into(), more_stake));

		// fill unstake BTreeMap by unstaked many entries of 1
		fill_unstaking::<T>(&candidate, None, u as u64);

		// go to block in which we can exit
		assert_ok!(<Pallet<T>>::init_leave_candidates(RawOrigin::Signed(candidate.clone()).into()));

		for i in 1..=T::ExitQueueDelay::get() {
			let round = <Round<T>>::get();
			let now = round.first + round.length;
			System::<T>::set_block_number(now);
			Pallet::<T>::on_initialize(now);
		}
		let unlookup_candidate = T::Lookup::unlookup(candidate.clone());

	}: _(RawOrigin::Signed(candidate.clone()), unlookup_candidate)
	verify {
		// should have one more entry in Unstaking
		assert_eq!(<Unstaking<T>>::get(&candidate).len().saturated_into::<u32>(), u.saturating_add(1u32));
	}

	candidate_stake_more {
		let n in 1 .. T::MaxTopCandidates::get() - 1;
		let m in 0 .. T::MaxDelegatorsPerCollator::get();
		let u in 0 .. (T::MaxUnstakeRequests::get().saturated_into::<u32>() - 1);

		let candidates = setup_collator_candidates::<T>(n, None);
		for (i, c) in candidates.iter().enumerate() {
			fill_delegators::<T>(m, c.clone(), i.saturated_into::<u32>());
		}
		let candidate = candidates[0].clone();

		let old_stake = <CandidatePool<T>>::get(&candidate).unwrap().stake;
		let more_stake = T::MinCollatorCandidateStake::get();

		// increase stake so we can unstake, because current stake is minimum
		T::Currency::make_free_balance_be(&candidate, T::CurrencyBalance::from(u128::MAX));
		assert_ok!(<Pallet<T>>::candidate_stake_more(RawOrigin::Signed(candidate.clone()).into(), more_stake));

		// fill unstake BTreeMap by unstaked many entries of 1
		fill_unstaking::<T>(&candidate, None, u as u64);

	}: _(RawOrigin::Signed(candidate.clone()), more_stake)
	verify {
		let new_stake = <CandidatePool<T>>::get(&candidate).unwrap().stake;
		assert!(<Unstaking<T>>::get(candidate).is_empty());
		assert_eq!(new_stake, old_stake + more_stake + more_stake - T::CurrencyBalance::from(u as u64));
	}

	candidate_stake_less {
		let n in 1 .. T::MaxTopCandidates::get() - 1;
		let m in 0 .. T::MaxDelegatorsPerCollator::get();

		let candidates = setup_collator_candidates::<T>(n, None);
		for (i, c) in candidates.iter().enumerate() {
			fill_delegators::<T>(m, c.clone(), i.saturated_into::<u32>());
		}
		let candidate = candidates[0].clone();

		// increase stake of candidate to later decrease it again
		let old_stake = <CandidatePool<T>>::get(&candidate).unwrap().stake;
		let more_stake = T::MinCollatorCandidateStake::get();

		T::Currency::make_free_balance_be(&candidate, T::CurrencyBalance::from(u128::MAX));
		Pallet::<T>::candidate_stake_more(RawOrigin::Signed(candidate.clone()).into(), more_stake).expect("should increase stake");

		let new_stake = <CandidatePool<T>>::get(&candidate).unwrap().stake;
		assert_eq!(new_stake, old_stake + more_stake);

	}: _(RawOrigin::Signed(candidate.clone()), more_stake)
	verify {
		let new_stake = <CandidatePool<T>>::get(&candidate).unwrap().stake;
		assert_eq!(new_stake, old_stake);
	}

	join_delegators {
		let n in 1 .. T::MaxTopCandidates::get();
		let m in 1 .. T::MaxDelegatorsPerCollator::get() - 1;

		let candidates = setup_collator_candidates::<T>(n, None);
		for (i, c) in candidates.iter().enumerate() {
			fill_delegators::<T>(m, c.clone(), i.saturated_into::<u32>());
		}
		let collator = candidates[0].clone();
		let delegator = account("new-delegator", 0, DELEGATOR_ACCOUNT_SEED);
		let amount = T::MinDelegatorStake::get();
		T::Currency::make_free_balance_be(&delegator, amount + amount + amount + amount);
		let unlookup_collator = T::Lookup::unlookup(collator.clone());

	}: _(RawOrigin::Signed(delegator.clone()), unlookup_collator, amount)
	verify {
		let state = <CandidatePool<T>>::get(&collator).unwrap();
		assert!(state.delegators.into_iter().any(|x| x.owner == delegator));
	}

	delegator_stake_more {
		// we need at least 1 collators
		let n in 1 .. T::MaxTopCandidates::get();
		// we need at least 1 delegator
		let m in 1 .. T::MaxDelegatorsPerCollator::get() - 1;
		let u in 1 .. (T::MaxUnstakeRequests::get().saturated_into::<u32>() - 1);

		let candidates = setup_collator_candidates::<T>(n, None);
		for (i, c) in candidates.iter().enumerate() {
			fill_delegators::<T>(m, c.clone(), i.saturated_into::<u32>());
		}
		let collator = candidates[0].clone();
		let amount = T::MinDelegatorStake::get();

		// make sure delegator collated to collator
		let state = <CandidatePool<T>>::get(&collator).unwrap();
		let delegator = state.delegators.into_bounded_vec()[0].owner.clone();
		assert_eq!(<DelegatorState<T>>::get(&delegator).unwrap().total, amount);

		// increase stake so we can unstake, because current stake is minimum
		T::Currency::make_free_balance_be(&delegator, T::CurrencyBalance::from(u128::MAX));
		assert_ok!(<Pallet<T>>::delegator_stake_more(RawOrigin::Signed(delegator.clone()).into(), T::Lookup::unlookup(collator.clone()), T::CurrencyBalance::from(u as u64)));
		assert_eq!(<DelegatorState<T>>::get(&delegator).unwrap().total, amount + T::CurrencyBalance::from(u as u64));

		// fill unstake BTreeMap by unstaked many entries of 1
		fill_unstaking::<T>(&collator, Some(&delegator), u as u64);
		assert_eq!(<DelegatorState<T>>::get(&delegator).unwrap().total, amount);
		let unlookup_collator = T::Lookup::unlookup(collator.clone());
	}: _(RawOrigin::Signed(delegator.clone()), unlookup_collator, amount)
	verify {
		let state = <CandidatePool<T>>::get(&collator).unwrap();
		assert!(state.delegators.into_iter().any(|x| x.owner == delegator));
		assert_eq!(<DelegatorState<T>>::get(&delegator).unwrap().total, amount + amount);
		assert!(<Unstaking<T>>::get(&delegator).is_empty());
	}

	delegator_stake_less {
		// we need at least 1 collators
		let n in 1 .. T::MaxTopCandidates::get();
		// we need at least 1 delegator
		let m in 1 .. T::MaxDelegatorsPerCollator::get() - 1;

		let candidates = setup_collator_candidates::<T>(n, None);
		for (i, c) in candidates.iter().enumerate() {
			fill_delegators::<T>(m, c.clone(), i.saturated_into::<u32>());
		}
		let collator = candidates[0].clone();
		let amount = T::CurrencyBalance::one();

		// make sure delegator collated to collator
		let state = <CandidatePool<T>>::get(&collator).unwrap();
		let delegator = state.delegators.into_bounded_vec()[0].owner.clone();
		assert_eq!(<DelegatorState<T>>::get(&delegator).unwrap().total, T::MinDelegatorStake::get());

		// increase stake so we can unstake, because current stake is minimum
		T::Currency::make_free_balance_be(&delegator, T::CurrencyBalance::from(u128::MAX));
		assert_ok!(<Pallet<T>>::delegator_stake_more(RawOrigin::Signed(delegator.clone()).into(), T::Lookup::unlookup(collator.clone()), amount + amount));
		assert_eq!(<DelegatorState<T>>::get(&delegator).unwrap().total, T::MinDelegatorStake::get() + amount + amount);

		// decrease stake once so we have an unstaking entry for this block
		assert_ok!(<Pallet<T>>::delegator_stake_less(RawOrigin::Signed(delegator.clone()).into(), T::Lookup::unlookup(collator.clone()), amount));
		assert_eq!(<DelegatorState<T>>::get(&delegator).unwrap().total, T::MinDelegatorStake::get() + amount);
		assert_eq!(<Unstaking<T>>::get(&delegator).len(), 1);
		let unlookup_collator = T::Lookup::unlookup(collator.clone());

	}: _(RawOrigin::Signed(delegator.clone()), unlookup_collator, amount)
	verify {
		let state = <CandidatePool<T>>::get(&collator).unwrap();
		assert!(state.delegators.into_iter().any(|x| x.owner == delegator));
		assert_eq!(<DelegatorState<T>>::get(&delegator).unwrap().total, T::MinDelegatorStake::get());
		assert_eq!(<Unstaking<T>>::get(&delegator).len(), 2);
	}

	revoke_delegation {
		// we need at least 1 collators
		let n in 1 .. T::MaxTopCandidates::get();
		// we need at least 1 delegator
		let m in 1 .. T::MaxDelegatorsPerCollator::get() - 1;

		let candidates = setup_collator_candidates::<T>(n, None);
		for (i, c) in candidates.iter().enumerate() {
			fill_delegators::<T>(m, c.clone(), i.saturated_into::<u32>());
		}
		let collator = candidates[0].clone();
		let amount = T::CurrencyBalance::one();

		// make sure delegator collated to collator
		let state = <CandidatePool<T>>::get(&collator).unwrap();
		let delegator = state.delegators.into_bounded_vec()[0].owner.clone();
		assert_eq!(<DelegatorState<T>>::get(&delegator).unwrap().total, T::MinDelegatorStake::get());

		// increase stake so we can unstake, because current stake is minimum
		T::Currency::make_free_balance_be(&delegator, T::CurrencyBalance::from(u128::MAX));
		assert_ok!(<Pallet<T>>::delegator_stake_more(RawOrigin::Signed(delegator.clone()).into(), T::Lookup::unlookup(collator.clone()), amount + amount));
		assert_eq!(<DelegatorState<T>>::get(&delegator).unwrap().total, T::MinDelegatorStake::get() + amount + amount);

		// decrease stake once so we have an unstaking entry for this block
		assert_ok!(<Pallet<T>>::delegator_stake_less(RawOrigin::Signed(delegator.clone()).into(), T::Lookup::unlookup(collator.clone()), amount));
		assert_eq!(<DelegatorState<T>>::get(&delegator).unwrap().total, T::MinDelegatorStake::get() + amount);
		assert_eq!(<Unstaking<T>>::get(&delegator).len(), 1);
		let unlookup_collator =  T::Lookup::unlookup(collator.clone());

	}: _(RawOrigin::Signed(delegator.clone()), unlookup_collator)
	verify {
		let state = <CandidatePool<T>>::get(&collator).unwrap();
		assert!(!state.delegators.into_iter().any(|x| x.owner == delegator));
		assert!(<DelegatorState<T>>::get(&delegator).is_none());
		assert_eq!(<Unstaking<T>>::get(&delegator).len(), 2);
	}

	leave_delegators {
		// we need at least 1 collators
		let n in 1 .. T::MaxTopCandidates::get();
		// we need at least 1 delegator
		let m in 1 .. T::MaxDelegatorsPerCollator::get() - 1;

		let candidates = setup_collator_candidates::<T>(n, None);
		for (i, c) in candidates.iter().enumerate() {
			fill_delegators::<T>(m, c.clone(), i.saturated_into::<u32>());
		}
		let collator = candidates[0].clone();
		let amount = T::CurrencyBalance::one();

		// make sure delegator collated to collator
		let state = <CandidatePool<T>>::get(&collator).unwrap();
		let delegator = state.delegators.into_bounded_vec()[0].owner.clone();
		assert_eq!(<DelegatorState<T>>::get(&delegator).unwrap().total, T::MinDelegatorStake::get());

		// increase stake so we can unstake, because current stake is minimum
		T::Currency::make_free_balance_be(&delegator, T::CurrencyBalance::from(u128::MAX));
		assert_ok!(<Pallet<T>>::delegator_stake_more(RawOrigin::Signed(delegator.clone()).into(), T::Lookup::unlookup(collator.clone()), amount + amount));
		assert_eq!(<DelegatorState<T>>::get(&delegator).unwrap().total, T::MinDelegatorStake::get() + amount + amount);

		// decrease stake once so we have an unstaking entry for this block
		assert_ok!(<Pallet<T>>::delegator_stake_less(RawOrigin::Signed(delegator.clone()).into(), T::Lookup::unlookup(collator.clone()), amount));
		assert_eq!(<DelegatorState<T>>::get(&delegator).unwrap().total, T::MinDelegatorStake::get() + amount);
		assert_eq!(<Unstaking<T>>::get(&delegator).len(), 1);

	}: _(RawOrigin::Signed(delegator.clone()))
	verify {
		let state = <CandidatePool<T>>::get(&collator).unwrap();
		assert!(!state.delegators.into_iter().any(|x| x.owner == delegator));
		assert!(<DelegatorState<T>>::get(&delegator).is_none());
		assert_eq!(<Unstaking<T>>::get(&delegator).len(), 2);
	}

	unlock_unstaked {
		let u in 1 .. (T::MaxUnstakeRequests::get() as u32 - 1);

		let candidate = account("collator", 0u32, COLLATOR_ACCOUNT_SEED);
		let free_balance = T::CurrencyBalance::from(u128::MAX);
		let stake = T::MinCollatorCandidateStake::get();
		T::Currency::make_free_balance_be(&candidate, free_balance);
		assert_ok!(<Pallet<T>>::join_candidates(
			T::Origin::from(Some(candidate.clone()).into()),
			stake,
		));
		assert_eq!(pallet_balances::Pallet::<T>::usable_balance(&candidate), (free_balance - T::MinCollatorCandidateStake::get()).into());

		// increase stake so we can unstake, because current stake is minimum
		assert_ok!(<Pallet<T>>::candidate_stake_more(RawOrigin::Signed(candidate.clone()).into(), stake));

		// fill unstake BTreeMap by unstaked many entries of 1
		fill_unstaking::<T>(&candidate, None, u as u64);
		assert_eq!(<CandidatePool<T>>::get(&candidate).unwrap().stake, stake + stake -  T::CurrencyBalance::from(u as u64));

		// roll to block in which first unstake can be unlocked
		System::<T>::set_block_number(T::StakeDuration::get());
		assert_eq!(pallet_balances::Pallet::<T>::usable_balance(&candidate), (free_balance - stake - stake).into());
		let unlookup_candidate = T::Lookup::unlookup(candidate.clone());

	}: _(RawOrigin::Signed(candidate.clone()),  unlookup_candidate)
	verify {
		assert_eq!(<Unstaking<T>>::get(&candidate).len().saturated_into::<u32>(), u.saturating_sub(1u32));
		assert_eq!(pallet_balances::Pallet::<T>::usable_balance(&candidate), (free_balance - stake - stake + T::CurrencyBalance::one()).into());
	}

	set_max_candidate_stake {
		let old = <MaxCollatorCandidateStake<T>>::get();
		let new = <MaxCollatorCandidateStake<T>>::get() + T::CurrencyBalance::from(10u128);
	}: _(RawOrigin::Root, new)
	verify {
		assert_eq!(<MaxCollatorCandidateStake<T>>::get(), new);
	}

	// [Post-launch TODO]: Activate after increasing MaxCollatorsPerDelegator to at least 2. Expected to throw otherwise.
	// delegate_another_candidate {
	// 	// we need at least 2 collators
	// 	let n in 2 .. T::MaxTopCandidates::get();
	// 	// we need at least 1 delegator
	// 	let m in 1 .. T::MaxDelegatorsPerCollator::get() - 1;
	// 	let u in 0 .. (T::MaxUnstakeRequests::get().saturated_into::<u32>() - 1);

	// 	let candidates = setup_collator_candidates::<T>(n, None);
	// 	for (i, c) in candidates.iter().enumerate() {
	// 		fill_delegators::<T>(m, c.clone(), i.saturated_into::<u32>());
	// 	}
	// 	let collator_delegated = candidates[0].clone();
	// 	let collator = candidates.last().unwrap().clone();
	// 	let amount = T::MinDelegatorStake::get();

	// 	// make sure delegator collated to collator_delegated
	// 	let state_delegated = <CandidatePool<T>>::get(&collator_delegated).unwrap();
	// 	let delegator = state_delegated.delegators.into_bounded_vec()[0].owner.clone();
	// 	assert!(<DelegatorState<T>>::get(&delegator).is_some());

	// 	// should not have delegated to collator yet
	// 	let state = <CandidatePool<T>>::get(&collator).unwrap();
	// 	assert!(!state.delegators.into_iter().any(|x| x.owner == delegator));

	// 	// increase stake so we can unstake, because current stake is minimum
	// 	T::Currency::make_free_balance_be(&delegator, T::CurrencyBalance::from(u128::MAX));
	// 	assert_ok!(<Pallet<T>>::delegator_stake_more(RawOrigin::Signed(delegator.clone()).into(), T::Lookup::unlookup(collator_delegated.clone()), T::CurrencyBalance::from(u as u64)));

	// 	// fill unstake BTreeMap by unstaked many entries of 1
	// 	fill_unstaking::<T>(&collator_delegated, Some(&delegator), u as u64);

	// }: _(RawOrigin::Signed(delegator.clone()), T::Lookup::unlookup(collator.clone()), amount)
	// verify {
	// 	let state = <CandidatePool<T>>::get(&collator).unwrap();
	// 	assert!(state.delegators.into_iter().any(|x| x.owner == delegator);
	// }
}

impl_benchmark_test_suite!(
	Pallet,
	crate::mock::ExtBuilder::default()
		.with_balances(vec![(u64::MAX, 1000 * crate::mock::MILLI_KILT)])
		.with_collators(vec![(u64::MAX, 1000 * crate::mock::MILLI_KILT)])
		.build(),
	crate::mock::Test,
);
