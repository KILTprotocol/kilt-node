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
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite};
use frame_support::{
	assert_ok,
	traits::{Currency, Get},
};
use frame_system::RawOrigin;
use sp_runtime::{traits::StaticLookup, Perquintill};
use sp_std::vec::Vec;

const COLLATOR_ACCOUNT_SEED: u32 = 0;
const DELEGATOR_ACCOUNT_SEED: u32 = 1;

/// Creates collators, must be called with an empty candidate pool
fn setup_collator_candidates<T: Config>(num_collators: u32) -> Vec<T::AccountId> {
	let current_collator_count = CandidatePool::<T>::get().len() as u32;
	let collators: Vec<T::AccountId> = (current_collator_count..num_collators)
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

fn add_delegators<T: Config>(num_delegators: u32, collator: T::AccountId) -> Vec<T::AccountId> {
	let delegators: Vec<T::AccountId> = (0..num_delegators)
		.map(|i| account("delegator", i as u32, DELEGATOR_ACCOUNT_SEED))
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
	set_inflation {
		let inflation = InflationInfo::new(
			Perquintill::from_percent(10),
			Perquintill::from_percent(15),
			Perquintill::from_percent(40),
			Perquintill::from_percent(10)
		);
	}: _(RawOrigin::Root, inflation)
	verify {}

	join_candidates {
		let n in 0 .. T::MaxCollatorCandidates::get() - 1;

		let candidates = setup_collator_candidates::<T>(n);
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
		let old_candidate = candidates[0].clone();

		add_delegators::<T>(m, old_candidate.clone());

	}: _(RawOrigin::Signed(old_candidate.clone()))
	verify {
		let candidates = CandidatePool::<T>::get();
		assert!(candidates.binary_search_by(|other| other.owner.cmp(&old_candidate)).is_err())
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
