// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2024 BOTLabs GmbH

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

use frame_benchmarking::v2::*;
use frame_support::traits::fungible::{Inspect, Mutate};
use frame_system::pallet_prelude::BlockNumberFor;
use kilt_support::traits::GenerateBenchmarkOrigin;
use sp_std::fmt::Debug;

use crate::{AccountIdOf, Config, Pallet};

#[benchmarks(
	where <<T as Config>::Currency as Inspect<AccountIdOf<T>>>::Balance: TryFrom<usize>,
	<T as Config>::Currency: Mutate<T::AccountId>,
	<<<T as Config>::Currency as Inspect<<T as frame_system::Config>::AccountId>>::Balance as TryFrom<usize>>::Error: Debug,
	T::EnsureOrigin: GenerateBenchmarkOrigin<T::RuntimeOrigin, T::AccountId, T::CtypeCreatorId>,
	BlockNumberFor<T>: From<u64>,
)]
mod benchmarks {
	const SEED: u32 = 0;
	const MAX_CTYPE_SIZE: u32 = 5 * 1024 * 1024;

	use frame_support::{
		assert_ok,
		traits::{EnsureOrigin, Get},
	};
	use sp_runtime::traits::Hash;

	use crate::Ctypes;

	use super::*;

	#[benchmark]
	fn add(l: Linear<1, MAX_CTYPE_SIZE>) {
		let caller = account("caller", 0, SEED);
		let did: T::CtypeCreatorId = account("did", 0, SEED);

		let ctype: Vec<u8> = (0u8..u8::MAX).cycle().take(l.try_into().unwrap()).collect();
		let ctype_hash = <T as frame_system::Config>::Hashing::hash(&ctype[..]);

		let initial_balance =
			<T as Config>::Fee::get() * ctype.len().try_into().unwrap() + <T as Config>::Currency::minimum_balance();
		<T as Config>::Currency::set_balance(&caller, initial_balance);
		let origin = T::EnsureOrigin::generate_origin(caller, did.clone());

		#[block]
		{
			assert_ok!(Pallet::<T>::add(origin, ctype));
		}

		let stored_ctype_entry = Ctypes::<T>::get(ctype_hash).expect("CType hash should be present on chain.");
		// Verify the CType has the right owner
		assert_eq!(stored_ctype_entry.creator, did);
	}

	#[benchmark]
	fn set_block_number() {
		let caller = account("caller", 0, SEED);
		let did: T::CtypeCreatorId = account("did", 0, SEED);

		let ctype: Vec<u8> = (0u8..u8::MAX)
			.cycle()
			.take(MAX_CTYPE_SIZE.try_into().unwrap())
			.collect();
		let ctype_hash = <T as frame_system::Config>::Hashing::hash(&ctype[..]);
		let new_block_number = 500u64.into();

		let initial_balance =
			<T as Config>::Fee::get() * ctype.len().try_into().unwrap() + <T as Config>::Currency::minimum_balance();
		<T as Config>::Currency::set_balance(&caller, initial_balance);
		let origin = T::EnsureOrigin::generate_origin(caller, did);
		Pallet::<T>::add(origin, ctype).expect("CType creation should not fail.");
		let overarching_origin =
			T::OverarchingOrigin::try_successful_origin().expect("Successful origin creation should not fail.");

		#[block]
		{
			assert_ok!(Pallet::<T>::set_block_number(
				overarching_origin,
				ctype_hash,
				new_block_number
			));
		}

		let stored_ctype_entry = Ctypes::<T>::get(ctype_hash).expect("CType hash should be present on chain.");

		// Verify the CType has the right block number
		assert_eq!(stored_ctype_entry.created_at, new_block_number);
	}

	#[cfg(test)]
	mod benchmarks_tests {
		use crate::Pallet;
		use frame_benchmarking::impl_benchmark_test_suite;

		impl_benchmark_test_suite!(
			Pallet,
			crate::mock::runtime::ExtBuilder::default().build_with_keystore(),
			crate::mock::runtime::Test,
		);
	}
}
