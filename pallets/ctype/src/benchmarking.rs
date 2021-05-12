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

use super::pallet::*;

use frame_benchmarking::benchmarks;
use frame_support::traits::{EnsureOrigin, UnfilteredDispatchable};

benchmarks! {
	add {
		let hash = <T::Hash as Default>::default();
		let origin = T::EnsureOrigin::successful_origin();
		let call = Call::<T>::add(hash);

	}: { call.dispatch_bypass_filter(origin)? }
	verify {
		Ctypes::<T>::contains_key(hash)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::mock::{ExtBuilder, Test};
	use frame_support::assert_ok;

	#[test]
	fn test_benchmarks() {
		ExtBuilder::default().build(None).execute_with(|| {
			assert_ok!(test_benchmark_add::<Test>());
		});
	}
}
