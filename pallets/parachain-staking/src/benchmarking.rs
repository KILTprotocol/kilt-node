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
use crate::{types::Stake, *};
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, vec};
use frame_support::{
	assert_ok,
	traits::{Currency, Get, Hooks, OnFinalize},
};
use frame_system::{pallet_prelude::BlockNumberFor, Pallet as System, RawOrigin};
use sp_runtime::{
	traits::{One, StaticLookup},
	Perquintill,
};
use frame_system::RawOrigin;
use sp_runtime::{traits::StaticLookup, Perquintill};
use sp_std::vec::Vec;

const COLLATOR_ACCOUNT_SEED: u32 = 0;
const DELEGATOR_ACCOUNT_SEED: u32 = 1;

/// Fills the candidate pool up to `num_candidates`.
fn setup_collator_candidates<T: Config>(num_candidates: u32) -> Vec<T::AccountId> {
	let current_collator_count = CandidatePool::<T>::get().len() as u32;
	let collators: Vec<T::AccountId> = (current_collator_count..=num_candidates)
		.map(|i| account("collator", i as u32, COLLATOR_ACCOUNT_SEED))
		.collect();

	log::info!(
		"add {} collators to {} collators",
		collators.len(),
		CandidatePool::<T>::get().len()
	);

	for acc in collators.iter() {
		T::Currency::make_free_balance_be(&acc, T::MinCollatorCandidateStk::get());
		assert_ok!(<Pallet<T>>::join_candidates(
			T::Origin::from(Some(acc.clone()).into()),
			T::MinCollatorCandidateStk::get(),
		));
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

	log::info!("setup {} delegators", delegators.len());

	for acc in delegators.iter() {
		T::Currency::make_free_balance_be(&acc, T::MinDelegatorStk::get());
		assert_ok!(<Pallet<T>>::join_delegators(
			T::Origin::from(Some(acc.clone()).into()),
			T::Lookup::unlookup(collator.clone()),
			T::MinDelegatorStk::get(),
		));
	}

	delegators
}

benchmarks! {
	where_clause { where u64: Into<<T as frame_system::Config>::BlockNumber> }

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

		let candidates = setup_collator_candidates::<T>(n);
		for (i, c) in candidates.iter().enumerate() {
			fill_delegators::<T>(m, c.clone(), i as u32);
		}
		let old_candidate = candidates[0].clone();
		let old_num_selected = <SelectedCandidates<T>>::get().len();
	}: _(RawOrigin::Root, n)
	verify {
		assert_eq!(<TotalSelected<T>>::get(), n);
		assert_eq!(<SelectedCandidates<T>>::get().len() as u32, n);
	}

	set_blocks_per_round {
		let bpr: T::BlockNumber = T::MinBlocksPerRound::get() + T::BlockNumber::one();
	}: _(RawOrigin::Root, bpr)
	verify {
		assert_eq!(<Round<T>>::get().length, bpr);
	}

	join_candidates {
		let n in 0 .. T::MaxCollatorCandidates::get() - 1;
		let m in 0 .. T::MaxDelegatorsPerCollator::get();

		let candidates = setup_collator_candidates::<T>(n);
		for (i, c) in candidates.iter().enumerate() {
			fill_delegators::<T>(m, c.clone(), i as u32);
		}

		let new_candidate = account("new_collator", u32::MAX , COLLATOR_ACCOUNT_SEED);
		T::Currency::make_free_balance_be(&new_candidate, T::MinCollatorCandidateStk::get());

	}: _(RawOrigin::Signed(new_candidate.clone()), T::MinCollatorCandidateStk::get())
	verify {
		let candidates = CandidatePool::<T>::get();
		assert!(candidates.binary_search_by(|other| other.owner.cmp(&new_candidate)).is_ok())
	}

	leave_candidates {
		let n in 1 .. T::MaxCollatorCandidates::get() - 1;
		let m in 0 .. T::MaxDelegatorsPerCollator::get();

		let candidates = setup_collator_candidates::<T>(n);
		for (i, c) in candidates.iter().enumerate() {
			fill_delegators::<T>(m, c.clone(), i as u32);
		}

		let now = <Round<T>>::get().current;
		let old_candidate = candidates[0].clone();

	}: _(RawOrigin::Signed(old_candidate.clone()))
	verify {
		let candidates = CandidatePool::<T>::get();
		assert!(candidates.binary_search_by(|other| other.owner.cmp(&old_candidate)).is_err());
		let unlocking_at = now.saturating_add(T::ExitQueueDelay::get());
		assert_eq!(<ExitQueue<T>>::get().to_vec(), vec![Stake { owner: old_candidate, amount: unlocking_at }]);
	}

	candidate_stake_more_unstaked_empty {
		let n in 0 .. T::MaxCollatorCandidates::get() - 1;
		let m in 0 .. T::MaxDelegatorsPerCollator::get();

		let candidates = setup_collator_candidates::<T>(n);
		for (i, c) in candidates.iter().enumerate() {
			fill_delegators::<T>(m, c.clone(), i as u32);
		}
		let candidate = candidates[0].clone();

		let old_stake = <CollatorState<T>>::get(&candidate).unwrap().stake;
		let more_stake = T::MinCollatorCandidateStk::get();
		T::Currency::make_free_balance_be(&candidate, more_stake + more_stake + more_stake + more_stake);

	}: candidate_stake_more(RawOrigin::Signed(candidate.clone()), more_stake)
	verify {
		let new_stake = <CollatorState<T>>::get(&candidate).unwrap().stake;
		assert_eq!(new_stake, old_stake + more_stake);
	}

	candidate_stake_more_unstaked_full {
		let n in 0 .. T::MaxCollatorCandidates::get() - 1;
		let m in 0 .. T::MaxDelegatorsPerCollator::get();

		let candidates = setup_collator_candidates::<T>(n);
		for (i, c) in candidates.iter().enumerate() {
			fill_delegators::<T>(m, c.clone(), i as u32);
		}
		let candidate = candidates[0].clone();

		let old_stake = <CollatorState<T>>::get(&candidate).unwrap().stake;
		let more_stake = T::MinCollatorCandidateStk::get();
		T::Currency::make_free_balance_be(&candidate, T::CurrencyBalance::from(100_000u64) * T::CurrencyBalance::from(u64::MAX));
		log::info!("free balance {:?}", T::Currency::free_balance(&candidate));
		log::info!("old_Stake {:?}", old_stake);


		// increase stake so we can unstake, because current stake is minimum
		assert_ok!(<Pallet<T>>::candidate_stake_more(RawOrigin::Signed(candidate.clone()).into(), more_stake));

		// fill unstake BTreeMap by unstaked many entries of 1
		let unstaked = T::MaxUnstakeRequests::get() as u64;
		assert!(more_stake + more_stake - T::CurrencyBalance::from(unstaked) > T::MinCollatorCandidateStk::get());
		while System::<T>::block_number() < unstaked.into() {
			assert_ok!(<Pallet<T>>::candidate_stake_less(RawOrigin::Signed(candidate.clone()).into(), T::CurrencyBalance::from(1u64)));
			System::<T>::set_block_number(System::<T>::block_number() + T::BlockNumber::one());
		}
		assert_eq!(<Unstaking<T>>::get(&candidate).len(), T::MaxUnstakeRequests::get());

	}: candidate_stake_more(RawOrigin::Signed(candidate.clone()), more_stake)
	verify {
		let new_stake = <CollatorState<T>>::get(&candidate).unwrap().stake;
		assert!(<Unstaking<T>>::get(candidate).is_empty());
		assert_eq!(new_stake, old_stake + more_stake + more_stake - T::CurrencyBalance::from(unstaked));
	}

	candidate_stake_less {
		let n in 0 .. T::MaxCollatorCandidates::get() - 1;
		let m in 0 .. T::MaxDelegatorsPerCollator::get();

		let candidates = setup_collator_candidates::<T>(n);
		for (i, c) in candidates.iter().enumerate() {
			fill_delegators::<T>(m, c.clone(), i as u32);
		}
		let candidate = candidates[0].clone();

		// increase stake of candidate to later decrease it again
		let old_stake = <CollatorState<T>>::get(&candidate).unwrap().stake;
		let more_stake = T::MinCollatorCandidateStk::get();

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
		let n in 0 .. T::MaxCollatorCandidates::get();
		let m in 0 .. T::MaxDelegatorsPerCollator::get() - 1;

		let candidates = setup_collator_candidates::<T>(n);
		for (i, c) in candidates.iter().enumerate() {
			fill_delegators::<T>(m, c.clone(), i as u32);
		}
		let collator = candidates[0].clone();
		let delegator = account("new-delegator", 0, DELEGATOR_ACCOUNT_SEED);
		let amount = T::MinDelegatorStk::get();
		T::Currency::make_free_balance_be(&delegator, amount + amount + amount + amount);

	}: _(RawOrigin::Signed(delegator.clone()), T::Lookup::unlookup(collator.clone()), amount)
	verify {
		let state = <CollatorState<T>>::get(&collator).unwrap();
		assert!(state.delegators.binary_search_by(|x| x.owner.cmp(&delegator)).is_ok());
	}

	// on_initialize {
	// 	// TODO: implement this benchmark
	// 	let num_of_collators = T::MinSelectedCandidates::get();
	// 	let num_of_candidates = T::MaxCollatorCandidates::get();

	// }: { <Pallet<T> as Hooks<BlockNumberFor<T>>>::on_initialize(T::BlockNumber::one()) }
	// verify {
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
