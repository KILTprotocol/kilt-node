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
use crate::{mock::DECIMALS, Pallet as ParachainStaking};
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite};
use frame_support::{assert_ok, traits::OnFinalize};
use frame_system::RawOrigin;
use sp_runtime::Perbill;

const BLOCK: u32 = 5;
const MAX_COLLATORS: usize = 2;
const MAX_CANDIDATES: usize = 5;

// fn setup<T: Config>() -> Result<(), &'static str> {

// }

benchmarks! {
	set_inflation {
		let inflation = InflationInfo::new::<T>(10, 15, 40, 10);

	}: _(RawOrigin::Root, inflation)
	verify {
	}


	on_finalize {
		let block = BLOCK;
		// TODO: From min to max
		// let num_of_collators = MAX_CANDIDATES;
		// let num_of_candidates = MAX_CANDIDATES;
		// let collator_stake

		// // initialize balance

		// // initialize candidates
		// let candidate_ids: Vec<T as frame_system::Config>::AccountId> = (1..=MAX_CANDIDATES).collect()
		// let candidates: Vec<(<T as frame_system::Config>::AccountId, BalanceOf<T>)> =
		// candidate_ids.clone().into_iter().map(|i| (i, 100_000 * DECIMALS)).collect();

		// // TODO: initialize delegators

		//  // TODO: Set up genesis round

		//  // TODO: Collator should exit
	}: { ParachainStaking::<T>::on_finalize(block.into()) }
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

impl_benchmark_test_suite!(Pallet, crate::tests::new_test_ext(), crate::tests::Test);
