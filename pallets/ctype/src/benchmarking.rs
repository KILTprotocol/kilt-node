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

use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite};
use frame_support::{
	sp_runtime::traits::Hash,
	traits::{Currency, Get},
};
use sp_std::{
	convert::{TryFrom, TryInto},
	fmt::Debug,
	vec::Vec,
};

use kilt_support::traits::GenerateBenchmarkOrigin;

use crate::*;

const SEED: u32 = 0;
const MAX_CTYPE_SIZE: u32 = 5 * 1024 * 1024;

benchmarks! {
	where_clause {
		where
		<<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance: TryFrom<usize>,
		<<<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance as TryFrom<usize>>::Error: Debug,
		T::EnsureOrigin: GenerateBenchmarkOrigin<T::RuntimeOrigin, T::AccountId, T::CtypeCreatorId>,
	}

	add {
		let l in 1 .. MAX_CTYPE_SIZE;

		let caller = account("caller", 0, SEED);
		let did: T::CtypeCreatorId = account("did", 0, SEED);

		let ctype: Vec<u8> = (0u8..u8::MAX).cycle().take(l.try_into().unwrap()).collect();
		let ctype_hash = <T as frame_system::Config>::Hashing::hash(&ctype[..]);

		let initial_balance = <T as Config>::Fee::get() * ctype.len().try_into().unwrap() + <T as Config>::Currency::minimum_balance();
		<T as Config>::Currency::make_free_balance_be(&caller, initial_balance);
		let origin = T::EnsureOrigin::generate_origin(caller, did.clone());

	}: _<T::RuntimeOrigin>(origin, ctype)
	verify {
		let stored_ctype_creator: T::CtypeCreatorId = Ctypes::<T>::get(ctype_hash).expect("CType hash should be present on chain.");

		// Verify the CType has the right owner
		assert_eq!(stored_ctype_creator, did);
	}
}

impl_benchmark_test_suite! {
	Pallet,
	crate::mock::runtime::ExtBuilder::default().build_with_keystore(),
	crate::mock::runtime::Test
}
