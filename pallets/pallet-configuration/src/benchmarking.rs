// KILT Blockchain – <https://kilt.io>
// Copyright (C) 2025, KILT Foundation

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

// If you feel like getting in touch with us, you can do so at <hello@kilt.io>

// Old benchmarking macros are a mess.
#![allow(clippy::tests_outside_test_module)]

use frame_benchmarking::benchmarks;
use frame_support::traits::EnsureOriginWithArg;

use crate::*;

benchmarks! {

	set_configuration {
		let new_config = Configuration { relay_block_strictly_increasing: true };
		let origin = T::EnsureOrigin::try_successful_origin(&new_config).expect("Should build successful origin");

	}: _<T::RuntimeOrigin>(origin, new_config)
	verify {
		assert_eq!(ConfigurationStore::<T>::get(), Configuration { relay_block_strictly_increasing: true });
	}

	impl_benchmark_test_suite!(
		Pallet,
		crate::mock::runtime::ExtBuilder.build_with_keystore(),
		crate::mock::runtime::Test
	)
}
