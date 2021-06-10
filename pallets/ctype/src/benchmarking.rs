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

use frame_benchmarking::{benchmarks, impl_benchmark_test_suite};
use frame_system::RawOrigin;

use crate::*;

fn get_ctype_creator<T: Config>() -> T::AccountId {
	T::AccountId::default()
}

fn get_ctype_hash<T: Config>() -> CtypeHashOf<T> {
	CtypeHashOf::<T>::default()
}

benchmarks! {

	add {
		let caller = get_ctype_creator::<T>();
		let hash = get_ctype_hash::<T>();
	}: _(RawOrigin::Signed(caller), hash)
	verify {
		Ctypes::<T>::contains_key(hash);
	}
}

impl_benchmark_test_suite! {
	Pallet,
	crate::mock::ExtBuilder::default().build_with_keystore(None),
	crate::mock::Test
}
