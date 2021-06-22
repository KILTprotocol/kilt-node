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
#![cfg(feature = "runtime-benchmarks")]

//! Benchmarking
use crate::*;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, Zero};
use frame_support::{
	assert_ok,
	traits::{Currency, Get, OnInitialize},
};
use frame_system::{Pallet as System, RawOrigin};
use kilt_primitives::constants::YEARS;
use sp_runtime::{
	traits::{One, SaturatedConversion, Saturating, StaticLookup},
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
	let current_collator_count = CandidatePool::<T>::get().len() as u32;
	let collators: Vec<T::AccountId> = (current_collator_count..num_candidates)
		.map(|i| account("collator", i as u32, COLLATOR_ACCOUNT_SEED))
		.collect();
	let amount: T::CurrencyBalance = default_amount.unwrap_or_else(T::MinCollatorCandidateStake::get);

	for acc in collators.iter() {
		T::Currency::make_free_balance_be(acc, amount);
		assert_ok!(<Pallet<T>>::join_candidates(
			T::Origin::from(Some(acc.clone()).into()),
			amount,
		));
		assert_eq!(<CollatorState<T>>::get(acc).unwrap().stake, amount);
	}

	CandidatePool::<T>::get()
		.into_vec()
		.drain(..)
		.map(|c| c.owner)
		.collect()
}

fn fill_delegators<T: Config>(num_delegators: u32, collator: T::AccountId, collator_seed: u32) -> Vec<T::AccountId> {
	let state = <CollatorState<T>>::get(&collator).unwrap();
	let current_delegators = state.delegators.len() as u32;

	let delegators: Vec<T::AccountId> = (current_delegators..num_delegators)
		.map(|i| account("delegator", i as u32, DELEGATOR_ACCOUNT_SEED * 1000 + collator_seed))
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
	while System::<T>::block_number() < unstaked.into() {
		if let Some(delegator) = delegator {
			assert_ok!(<Pallet<T>>::delegator_stake_less(
				RawOrigin::Signed(delegator.clone()).into(),
				T::Lookup::unlookup(collator.clone()),
				T::CurrencyBalance::from(1u64)
			));
		} else {
			assert_ok!(<Pallet<T>>::candidate_stake_less(
				RawOrigin::Signed(collator.clone()).into(),
				T::CurrencyBalance::from(1u64)
			));
		}
		System::<T>::set_block_number(System::<T>::block_number() + T::BlockNumber::one());
	}
	let who = delegator.unwrap_or(collator);
	assert_eq!(<Unstaking<T>>::get(who).len() as u64, unstaked);
	assert!(<Unstaking<T>>::get(who).len() <= T::MaxUnstakeRequests::get().try_into().unwrap());
}

benchmarks! {
	where_clause { where u64: Into<<T as frame_system::Config>::BlockNumber> }

	on_initialize_no_action {
		assert_eq!(<Round<T>>::get().current, 0u32);
	}: { Pallet::<T>::on_initialize(T::BlockNumber::one()) }
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
	}: { Pallet::<T>::on_initialize((YEARS + 1u64).saturated_into::<T::BlockNumber>()) }
	verify {
		let new = <InflationConfig<T>>::get();
		assert_eq!(<LastRewardReduction<T>>::get(), T::BlockNumber::one());
		assert_eq!(new.collator.max_rate, old.collator.max_rate);
		assert_eq!(new.delegator.max_rate, old.delegator.max_rate);
		assert!(new.collator.reward_rate.annual < old.collator.reward_rate.annual);
	}

	set_inflation {
		let inflation = InflationInfo::new(
			Perquintill::from_percent(10),
			Perquintill::from_percent(15),
			Perquintill::from_percent(40),
			Perquintill::from_percent(10)
		);
	}: _(RawOrigin::Root, inflation.clone())
	verify {
		assert_eq!(<InflationConfig<T>>::get(), inflation);
	}

	set_max_selected_candidates {
		let n in (T::MinSelectedCandidates::get()) .. T::MaxCollatorCandidates::get();
		let m in 0 .. T::MaxDelegatorsPerCollator::get();

		let candidates = setup_collator_candidates::<T>(n, None);
		for (i, c) in candidates.iter().enumerate() {
			fill_delegators::<T>(m, c.clone(), i as u32);
		}
		let old_candidate = candidates[0].clone();
		let old_num_selected = <SelectedCandidates<T>>::get().len();
	}: _(RawOrigin::Root, n)
	verify {
		assert_eq!(<MaxSelectedCandidates<T>>::get(), n);
		assert_eq!(<SelectedCandidates<T>>::get().len() as u32, n);
		assert_eq!(<SelectedCandidates<T>>::get().len(), old_num_selected + ((n - T::MinSelectedCandidates::get()) as usize));
	}

	set_blocks_per_round {
		let bpr: T::BlockNumber = T::MinBlocksPerRound::get() + T::BlockNumber::one();
	}: _(RawOrigin::Root, bpr)
	verify {
		assert_eq!(<Round<T>>::get().length, bpr);
	}

	force_remove_candidate {
		let n in (T::MinSelectedCandidates::get() + 1) .. T::MaxCollatorCandidates::get();
		let m in 0 .. T::MaxDelegatorsPerCollator::get();

		let candidates = setup_collator_candidates::<T>(n, None);
		for (i, c) in candidates.iter().enumerate() {
			fill_delegators::<T>(m, c.clone(), i as u32);
		}
		let candidate = candidates[0].clone();
	}: _(RawOrigin::Root,  T::Lookup::unlookup(candidate.clone()))
	verify {
		let candidates = CandidatePool::<T>::get();
		assert!(candidates.binary_search_by(|other| other.owner.cmp(&candidate)).is_err())
	}

	join_candidates {
		let n in 1 .. T::MaxCollatorCandidates::get() - 1;
		let m in 0 .. T::MaxDelegatorsPerCollator::get();

		let candidates = setup_collator_candidates::<T>(n, None);
		for (i, c) in candidates.iter().enumerate() {
			fill_delegators::<T>(m, c.clone(), i as u32);
		}

		let new_candidate = account("new_collator", u32::MAX , COLLATOR_ACCOUNT_SEED);
		T::Currency::make_free_balance_be(&new_candidate, T::MinCollatorCandidateStake::get());

	}: _(RawOrigin::Signed(new_candidate.clone()), T::MinCollatorCandidateStake::get())
	verify {
		let candidates = CandidatePool::<T>::get();
		assert!(candidates.binary_search_by(|other| other.owner.cmp(&new_candidate)).is_ok())
	}

	init_leave_candidates {
		let n in (T::MinSelectedCandidates::get() + 1) .. T::MaxCollatorCandidates::get() - 1;
		let m in 0 .. T::MaxDelegatorsPerCollator::get();

		let candidates = setup_collator_candidates::<T>(n, None);
		for (i, c) in candidates.iter().enumerate() {
			fill_delegators::<T>(m, c.clone(), i as u32);
		}

		let now = <Round<T>>::get().current;
		let candidate = candidates[0].clone();

	}: _(RawOrigin::Signed(candidate.clone()))
	verify {
		let candidates = CandidatePool::<T>::get();
		assert!(candidates.binary_search_by(|other| other.owner.cmp(&candidate)).is_err());
		let unlocking_at = now.saturating_add(T::ExitQueueDelay::get());
		assert!(<CollatorState<T>>::get(candidate).unwrap().can_exit(unlocking_at));
	}

	cancel_leave_candidates {
		let n in (T::MinSelectedCandidates::get() + 1) .. T::MaxCollatorCandidates::get() - 1;
		let m in 0 .. T::MaxDelegatorsPerCollator::get();

		let candidates = setup_collator_candidates::<T>(n, None);
		for (i, c) in candidates.iter().enumerate() {
			fill_delegators::<T>(m, c.clone(), i as u32);
		}

		let candidate = candidates[0].clone();
		assert_ok!(<Pallet<T>>::init_leave_candidates(RawOrigin::Signed(candidate.clone()).into()));

	}: _(RawOrigin::Signed(candidate.clone()))
	verify {
		let candidates = CandidatePool::<T>::get();
		assert!(candidates.binary_search_by(|other| other.owner.cmp(&candidate)).is_ok());
	}

	execute_leave_candidates {
		let n in (T::MinSelectedCandidates::get() + 1) .. T::MaxCollatorCandidates::get() - 1;
		let m in 0 .. T::MaxDelegatorsPerCollator::get();
		let u in 0 .. (T::MaxUnstakeRequests::get() as u32 - 1);

		let candidates = setup_collator_candidates::<T>(n, None);
		for (i, c) in candidates.iter().enumerate() {
			fill_delegators::<T>(m, c.clone(), i as u32);
		}
		let candidate = candidates[0].clone();

		// increase stake so we can unstake, because current stake is minimum
		let more_stake = T::MinCollatorCandidateStake::get();
		T::Currency::make_free_balance_be(&candidate, T::CurrencyBalance::from(u64::MAX) * T::CurrencyBalance::from(u64::MAX));
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

	}: _(RawOrigin::Signed(candidate.clone()), T::Lookup::unlookup(candidate.clone()))
	verify {
		// should have one more entry in Unstaking
		assert_eq!(<Unstaking<T>>::get(&candidate).len() as u32, u.saturating_add(1u32));
	}

	candidate_stake_more {
		let n in 1 .. T::MaxCollatorCandidates::get() - 1;
		let m in 0 .. T::MaxDelegatorsPerCollator::get();
		let u in 0 .. (T::MaxUnstakeRequests::get() as u32);

		let candidates = setup_collator_candidates::<T>(n, None);
		for (i, c) in candidates.iter().enumerate() {
			fill_delegators::<T>(m, c.clone(), i as u32);
		}
		let candidate = candidates[0].clone();

		let old_stake = <CollatorState<T>>::get(&candidate).unwrap().stake;
		let more_stake = T::MinCollatorCandidateStake::get();

		// increase stake so we can unstake, because current stake is minimum
		T::Currency::make_free_balance_be(&candidate, T::CurrencyBalance::from(u64::MAX) * T::CurrencyBalance::from(u64::MAX));
		assert_ok!(<Pallet<T>>::candidate_stake_more(RawOrigin::Signed(candidate.clone()).into(), more_stake));

		// fill unstake BTreeMap by unstaked many entries of 1
		fill_unstaking::<T>(&candidate, None, u as u64);

	}: _(RawOrigin::Signed(candidate.clone()), more_stake)
	verify {
		let new_stake = <CollatorState<T>>::get(&candidate).unwrap().stake;
		assert!(<Unstaking<T>>::get(candidate).is_empty());
		assert_eq!(new_stake, old_stake + more_stake + more_stake - T::CurrencyBalance::from(u as u64));
	}

	candidate_stake_less {
		let n in 1 .. T::MaxCollatorCandidates::get() - 1;
		let m in 0 .. T::MaxDelegatorsPerCollator::get();

		let candidates = setup_collator_candidates::<T>(n, None);
		for (i, c) in candidates.iter().enumerate() {
			fill_delegators::<T>(m, c.clone(), i as u32);
		}
		let candidate = candidates[0].clone();

		// increase stake of candidate to later decrease it again
		let old_stake = <CollatorState<T>>::get(&candidate).unwrap().stake;
		let more_stake = T::MinCollatorCandidateStake::get();

		T::Currency::make_free_balance_be(&candidate, more_stake + more_stake + more_stake + more_stake);
		Pallet::<T>::candidate_stake_more(RawOrigin::Signed(candidate.clone()).into(), more_stake).expect("should increase stake");

		let new_stake = <CollatorState<T>>::get(&candidate).unwrap().stake;
		assert_eq!(new_stake, old_stake + more_stake);

	}: _(RawOrigin::Signed(candidate.clone()), more_stake)
	verify {
		let new_stake = <CollatorState<T>>::get(&candidate).unwrap().stake;
		assert_eq!(new_stake, old_stake);
	}

	join_delegators {
		let n in 1 .. T::MaxCollatorCandidates::get();
		let m in 1 .. T::MaxDelegatorsPerCollator::get() - 1;

		let candidates = setup_collator_candidates::<T>(n, None);
		for (i, c) in candidates.iter().enumerate() {
			fill_delegators::<T>(m, c.clone(), i as u32);
		}
		let collator = candidates[0].clone();
		let delegator = account("new-delegator", 0, DELEGATOR_ACCOUNT_SEED);
		let amount = T::MinDelegatorStake::get();
		T::Currency::make_free_balance_be(&delegator, amount + amount + amount + amount);

	}: _(RawOrigin::Signed(delegator.clone()), T::Lookup::unlookup(collator.clone()), amount)
	verify {
		let state = <CollatorState<T>>::get(&collator).unwrap();
		assert!(state.delegators.binary_search_by(|x| x.owner.cmp(&delegator)).is_ok());
	}

	delegator_stake_more {
		// we need at least 1 collators
		let n in 1 .. T::MaxCollatorCandidates::get();
		// we need at least 1 delegator
		let m in 1 .. T::MaxDelegatorsPerCollator::get() - 1;
		let u in 1 .. (T::MaxUnstakeRequests::get() as u32);

		let candidates = setup_collator_candidates::<T>(n, None);
		for (i, c) in candidates.iter().enumerate() {
			fill_delegators::<T>(m, c.clone(), i as u32);
		}
		let collator = candidates[0].clone();
		let amount = T::MinDelegatorStake::get();

		// make sure delegator collated to collator
		let state = <CollatorState<T>>::get(&collator).unwrap();
		let delegator = state.delegators.into_vec()[0].owner.clone();
		assert_eq!(<DelegatorState<T>>::get(&delegator).unwrap().total, amount);

		// increase stake so we can unstake, because current stake is minimum
		T::Currency::make_free_balance_be(&delegator, T::CurrencyBalance::from(u64::MAX) * T::CurrencyBalance::from(u64::MAX));
		assert_ok!(<Pallet<T>>::delegator_stake_more(RawOrigin::Signed(delegator.clone()).into(), T::Lookup::unlookup(collator.clone()), T::CurrencyBalance::from(u as u64)));
		assert_eq!(<DelegatorState<T>>::get(&delegator).unwrap().total, amount + T::CurrencyBalance::from(u as u64));

		// fill unstake BTreeMap by unstaked many entries of 1
		fill_unstaking::<T>(&collator, Some(&delegator), u as u64);
		assert_eq!(<DelegatorState<T>>::get(&delegator).unwrap().total, amount);
	}: _(RawOrigin::Signed(delegator.clone()), T::Lookup::unlookup(collator.clone()), amount)
	verify {
		let state = <CollatorState<T>>::get(&collator).unwrap();
		assert!(state.delegators.binary_search_by(|x| x.owner.cmp(&delegator)).is_ok());
		assert_eq!(<DelegatorState<T>>::get(&delegator).unwrap().total, amount + amount);
		assert!(<Unstaking<T>>::get(&delegator).is_empty());
	}

	delegator_stake_less {
		// we need at least 1 collators
		let n in 1 .. T::MaxCollatorCandidates::get();
		// we need at least 1 delegator
		let m in 1 .. T::MaxDelegatorsPerCollator::get() - 1;

		let candidates = setup_collator_candidates::<T>(n, None);
		for (i, c) in candidates.iter().enumerate() {
			fill_delegators::<T>(m, c.clone(), i as u32);
		}
		let collator = candidates[0].clone();
		let amount = T::CurrencyBalance::from(1u64);

		// make sure delegator collated to collator
		let state = <CollatorState<T>>::get(&collator).unwrap();
		let delegator = state.delegators.into_vec()[0].owner.clone();
		assert_eq!(<DelegatorState<T>>::get(&delegator).unwrap().total, T::MinDelegatorStake::get());

		// increase stake so we can unstake, because current stake is minimum
		T::Currency::make_free_balance_be(&delegator, T::CurrencyBalance::from(u64::MAX) * T::CurrencyBalance::from(u64::MAX));
		assert_ok!(<Pallet<T>>::delegator_stake_more(RawOrigin::Signed(delegator.clone()).into(), T::Lookup::unlookup(collator.clone()), amount + amount));
		assert_eq!(<DelegatorState<T>>::get(&delegator).unwrap().total, T::MinDelegatorStake::get() + amount + amount);

		// decrease stake once so we have an unstaking entry for this block
		assert_ok!(<Pallet<T>>::delegator_stake_less(RawOrigin::Signed(delegator.clone()).into(), T::Lookup::unlookup(collator.clone()), amount));
		assert_eq!(<DelegatorState<T>>::get(&delegator).unwrap().total, T::MinDelegatorStake::get() + amount);
		assert_eq!(<Unstaking<T>>::get(&delegator).len(), 1);

	}: _(RawOrigin::Signed(delegator.clone()), T::Lookup::unlookup(collator.clone()), amount)
	verify {
		let state = <CollatorState<T>>::get(&collator).unwrap();
		assert!(state.delegators.binary_search_by(|x| x.owner.cmp(&delegator)).is_ok());
		assert_eq!(<DelegatorState<T>>::get(&delegator).unwrap().total, T::MinDelegatorStake::get());
		assert_eq!(<Unstaking<T>>::get(&delegator).len(), 2);
	}

	revoke_delegation {
		// we need at least 1 collators
		let n in 1 .. T::MaxCollatorCandidates::get();
		// we need at least 1 delegator
		let m in 1 .. T::MaxDelegatorsPerCollator::get() - 1;

		let candidates = setup_collator_candidates::<T>(n, None);
		for (i, c) in candidates.iter().enumerate() {
			fill_delegators::<T>(m, c.clone(), i as u32);
		}
		let collator = candidates[0].clone();
		let amount = T::CurrencyBalance::from(1u64);

		// make sure delegator collated to collator
		let state = <CollatorState<T>>::get(&collator).unwrap();
		let delegator = state.delegators.into_vec()[0].owner.clone();
		assert_eq!(<DelegatorState<T>>::get(&delegator).unwrap().total, T::MinDelegatorStake::get());

		// increase stake so we can unstake, because current stake is minimum
		T::Currency::make_free_balance_be(&delegator, T::CurrencyBalance::from(u64::MAX) * T::CurrencyBalance::from(u64::MAX));
		assert_ok!(<Pallet<T>>::delegator_stake_more(RawOrigin::Signed(delegator.clone()).into(), T::Lookup::unlookup(collator.clone()), amount + amount));
		assert_eq!(<DelegatorState<T>>::get(&delegator).unwrap().total, T::MinDelegatorStake::get() + amount + amount);

		// decrease stake once so we have an unstaking entry for this block
		assert_ok!(<Pallet<T>>::delegator_stake_less(RawOrigin::Signed(delegator.clone()).into(), T::Lookup::unlookup(collator.clone()), amount));
		assert_eq!(<DelegatorState<T>>::get(&delegator).unwrap().total, T::MinDelegatorStake::get() + amount);
		assert_eq!(<Unstaking<T>>::get(&delegator).len(), 1);

	}: _(RawOrigin::Signed(delegator.clone()), T::Lookup::unlookup(collator.clone()))
	verify {
		let state = <CollatorState<T>>::get(&collator).unwrap();
		assert!(state.delegators.binary_search_by(|x| x.owner.cmp(&delegator)).is_err());
		assert!(<DelegatorState<T>>::get(&delegator).is_none());
		assert_eq!(<Unstaking<T>>::get(&delegator).len(), 2);
	}

	leave_delegators {
		// we need at least 1 collators
		let n in 1 .. T::MaxCollatorCandidates::get();
		// we need at least 1 delegator
		let m in 1 .. T::MaxDelegatorsPerCollator::get() - 1;

		let candidates = setup_collator_candidates::<T>(n, None);
		for (i, c) in candidates.iter().enumerate() {
			fill_delegators::<T>(m, c.clone(), i as u32);
		}
		let collator = candidates[0].clone();
		let amount = T::CurrencyBalance::from(1u64);

		// make sure delegator collated to collator
		let state = <CollatorState<T>>::get(&collator).unwrap();
		let delegator = state.delegators.into_vec()[0].owner.clone();
		assert_eq!(<DelegatorState<T>>::get(&delegator).unwrap().total, T::MinDelegatorStake::get());

		// increase stake so we can unstake, because current stake is minimum
		T::Currency::make_free_balance_be(&delegator, T::CurrencyBalance::from(u64::MAX) * T::CurrencyBalance::from(u64::MAX));
		assert_ok!(<Pallet<T>>::delegator_stake_more(RawOrigin::Signed(delegator.clone()).into(), T::Lookup::unlookup(collator.clone()), amount + amount));
		assert_eq!(<DelegatorState<T>>::get(&delegator).unwrap().total, T::MinDelegatorStake::get() + amount + amount);

		// decrease stake once so we have an unstaking entry for this block
		assert_ok!(<Pallet<T>>::delegator_stake_less(RawOrigin::Signed(delegator.clone()).into(), T::Lookup::unlookup(collator.clone()), amount));
		assert_eq!(<DelegatorState<T>>::get(&delegator).unwrap().total, T::MinDelegatorStake::get() + amount);
		assert_eq!(<Unstaking<T>>::get(&delegator).len(), 1);

	}: _(RawOrigin::Signed(delegator.clone()))
	verify {
		let state = <CollatorState<T>>::get(&collator).unwrap();
		assert!(state.delegators.binary_search_by(|x| x.owner.cmp(&delegator)).is_err());
		assert!(<DelegatorState<T>>::get(&delegator).is_none());
		assert_eq!(<Unstaking<T>>::get(&delegator).len(), 2);
	}

	withdraw_unstaked {
		let u in 1 .. (T::MaxUnstakeRequests::get() as u32);

		let candidate = account("collator", 0u32, COLLATOR_ACCOUNT_SEED);
		let free_balance = T::CurrencyBalance::from(u64::MAX) * T::CurrencyBalance::from(u64::MAX);
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
		assert_eq!(<CollatorState<T>>::get(&candidate).unwrap().stake, stake + stake -  T::CurrencyBalance::from(u as u64));

		// roll to block in which first unstake can be withdrawn
		System::<T>::set_block_number(T::StakeDuration::get());
		assert_eq!(pallet_balances::Pallet::<T>::usable_balance(&candidate), (free_balance - stake - stake).into());

	}: _(RawOrigin::Signed(candidate.clone()),  T::Lookup::unlookup(candidate.clone()))
	verify {
		assert_eq!(<Unstaking<T>>::get(&candidate).len() as u32, u.saturating_sub(1u32));
		assert_eq!(pallet_balances::Pallet::<T>::usable_balance(&candidate), (free_balance - stake - stake + T::CurrencyBalance::from(1u64)).into());
	}

	increase_max_candidate_stake_by {
		let old = <MaxCollatorCandidateStake<T>>::get();
	}: _(RawOrigin::Root, T::CurrencyBalance::from(1u64))
	verify {
		assert_eq!(<MaxCollatorCandidateStake<T>>::get(), old + T::CurrencyBalance::from(1u64));
		assert!(old < <MaxCollatorCandidateStake<T>>::get());
	}

	decrease_max_candidate_stake_by {
		let n in 2 .. T::MaxCollatorCandidates::get();
		let m in 0 .. T::MaxDelegatorsPerCollator::get();

		// worst case: all candidates have staked more than new max
		let old = <MaxCollatorCandidateStake<T>>::get();
		let new =  T::MinCollatorCandidateStake::get();
		let stake = new + new;
		let candidates = setup_collator_candidates::<T>(n, Some(stake));
		for (i, c) in candidates.iter().enumerate() {
			fill_delegators::<T>(m, c.clone(), i as u32);
		}
		let candidate = candidates[0].clone();
		assert_eq!(<CollatorState<T>>::get(&candidate).unwrap().stake, stake);
	}: _(RawOrigin::Root, old.saturating_sub(new))
	verify {
		assert_eq!(<MaxCollatorCandidateStake<T>>::get(), new);
		assert_eq!(<CollatorState<T>>::get(candidate).unwrap().stake, new);
	}

	// [Post-launch TODO]: Activate after increasing MaxCollatorsPerDelegator to at least 2. Expected to throw otherwise.
	// delegate_another_candidate {
	// 	// we need at least 2 collators
	// 	let n in 2 .. T::MaxCollatorCandidates::get();
	// 	// we need at least 1 delegator
	// 	let m in 1 .. T::MaxDelegatorsPerCollator::get() - 1;
	// 	let u in 0 .. (T::MaxUnstakeRequests::get() as u32);

	// 	let candidates = setup_collator_candidates::<T>(n, None);
	// 	for (i, c) in candidates.iter().enumerate() {
	// 		fill_delegators::<T>(m, c.clone(), i as u32);
	// 	}
	// 	let collator_delegated = candidates[0].clone();
	// 	let collator = candidates.last().unwrap().clone();
	// 	let amount = T::MinDelegatorStake::get();

	// 	// make sure delegator collated to collator_delegated
	// 	let state_delegated = <CollatorState<T>>::get(&collator_delegated).unwrap();
	// 	let delegator = state_delegated.delegators.into_vec()[0].owner.clone();
	// 	assert!(<DelegatorState<T>>::get(&delegator).is_some());

	// 	// should not have delegated to collator yet
	// 	let state = <CollatorState<T>>::get(&collator).unwrap();
	// 	assert!(!state.delegators.binary_search_by(|x| x.owner.cmp(&delegator)).is_ok());

	// 	// increase stake so we can unstake, because current stake is minimum
	// 	T::Currency::make_free_balance_be(&delegator, T::CurrencyBalance::from(u64::MAX) * T::CurrencyBalance::from(u64::MAX));
	// 	assert_ok!(<Pallet<T>>::delegator_stake_more(RawOrigin::Signed(delegator.clone()).into(), T::Lookup::unlookup(collator_delegated.clone()), T::CurrencyBalance::from(u as u64)));

	// 	// fill unstake BTreeMap by unstaked many entries of 1
	// 	fill_unstaking::<T>(&collator_delegated, Some(&delegator), u as u64);

	// }: _(RawOrigin::Signed(delegator.clone()), T::Lookup::unlookup(collator.clone()), amount)
	// verify {
	// 	let state = <CollatorState<T>>::get(&collator).unwrap();
	// 	assert!(state.delegators.binary_search_by(|x| x.owner.cmp(&delegator)).is_ok());
	// }
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::mock::Test;
	use sp_io::TestExternalities;

	pub fn new_test_ext() -> TestExternalities {
		let t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
		TestExternalities::new(t)
	}

	#[test]
	fn test_benchmarks() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_set_inflation::<Test>());
		});
	}
}

impl_benchmark_test_suite!(Pallet, crate::mock::ExtBuilder::default().build(), crate::mock::Test,);
