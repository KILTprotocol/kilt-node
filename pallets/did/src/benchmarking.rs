// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019  BOTLabs GmbH

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

use super::*;

use frame_benchmarking::{account, benchmarks};
use frame_system::RawOrigin;
use sp_runtime::traits::Hash;
use sp_std::{boxed::Box, vec, vec::Vec};

const SEED: u32 = 0;

benchmarks! {
	where_clause { where T::PublicSigningKey: From<T::Hash>, T::PublicBoxKey: From<T::Hash>}

	add {
		let caller: T::AccountId = account("caller", 0, SEED);
		let sign_key: T::PublicSigningKey = T::Hashing::hash(b"sign_key").into();
		let box_key: T::PublicBoxKey = T::Hashing::hash(b"box_key").into();
	}: _(RawOrigin::Signed(caller), sign_key, box_key, Some(b"http://kilt.org/submit".to_vec()))

	remove {
		let caller: T::AccountId = account("caller", 0, SEED);
		let sign_key: T::PublicSigningKey = T::Hashing::hash(b"sign_key").into();
		let box_key: T::PublicBoxKey = T::Hashing::hash(b"box_key").into();
		let _ = Module::<T>::add(RawOrigin::Signed(caller.clone()).into(), sign_key, box_key, Some(b"http://kilt.org/submit".to_vec()));
	}: _(RawOrigin::Signed(caller))
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::tests::{new_test_ext, Test};
	use frame_support::assert_ok;

	#[test]
	fn test_benchmarks() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_add::<Test>());
			assert_ok!(test_benchmark_remove::<Test>());
		});
	}
}
