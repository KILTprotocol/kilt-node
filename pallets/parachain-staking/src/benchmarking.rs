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
use super::*;
use crate::Pallet as ParachainStaking;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite};
use frame_support::{
	assert_ok,
	traits::{Currency, Get, Hooks, OnFinalize},
};
use frame_system::{pallet_prelude::BlockNumberFor, RawOrigin};
use sp_runtime::{traits::One, Perquintill};
use sp_std::vec::Vec;

const COLLATOR_ACCOUNT_SEED: u32 = 0;

fn setup_collator_candidates<T: Config>(num_collators: u32) -> Vec<T::AccountId> {
	let collators: Vec<T::AccountId> = (0..num_collators)
		.map(|i| account("collator", i as u32, COLLATOR_ACCOUNT_SEED))
		.collect();

	for acc in collators.iter() {
		T::Currency::make_free_balance_be(&acc, T::MinCollatorCandidateStk::get());
		assert_ok!(<Pallet<T>>::join_candidates(
			T::Origin::from(Some(acc.clone()).into()),
			T::MinCollatorCandidateStk::get(),
		));
	}

	collators
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
		let n in 1 .. T::MaxCollatorCandidates::get() - 1;

		let candidates = setup_collator_candidates::<T>(n);
		let new_candidate = account("new_collator", u32::MAX , COLLATOR_ACCOUNT_SEED);
		T::Currency::make_free_balance_be(&new_candidate, T::MinCollatorCandidateStk::get());

	}: _(RawOrigin::Signed(new_candidate), T::MinCollatorCandidateStk::get())
	verify {
	}


	on_initialize {
		// TODO: implement this benchmark
		let num_of_collators = T::MinSelectedCandidates::get();
		let num_of_candidates = T::MaxCollatorCandidates::get();

	}: { <ParachainStaking<T> as Hooks<BlockNumberFor<T>>>::on_initialize(T::BlockNumber::one()) }
	verify {
	}
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

impl_benchmark_test_suite!(
	ParachainStaking,
	crate::mock::ExtBuilder::default().build(),
	crate::mock::Test,
);
