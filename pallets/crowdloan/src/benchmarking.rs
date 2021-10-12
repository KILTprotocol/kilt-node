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

use crate::*;

use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite};
use frame_system::RawOrigin;
use sp_runtime::traits::{One, StaticLookup};

const SEED_1: u32 = 1;
const SEED_2: u32 = 2;

benchmarks! {
	set_admin_account {
		let admin: AccountIdOf<T> = account("admin", 0, SEED_1);
		let new_admin: AccountIdOf<T> = account("new_admin", 0, SEED_2);
		AdminAccount::<T>::set(admin.clone());
	}: _(RawOrigin::Signed(admin), T::Lookup::unlookup(new_admin.clone()))
	verify {
		assert_eq!(
			AdminAccount::<T>::get(),
			new_admin,
			"Admin account different than expected"
		);
	}

	set_new_contribution {
		let admin: AccountIdOf<T> = account("admin", 0, SEED_1);
		let contributor: AccountIdOf<T> = account("contributor", 0, SEED_2);
		let contribution: BalanceOf<T> = BalanceOf::<T>::one();
		AdminAccount::<T>::set(admin.clone());
	}: _(RawOrigin::Signed(admin), T::Lookup::unlookup(contributor.clone()), contribution)
	verify {
		assert_eq!(
			Contributions::<T>::get(&contributor),
			Some(contribution),
			"Contribution different than the expected one."
		);
	}

	remove_contribution {
		let admin: AccountIdOf<T> = account("admin", 0, SEED_1);
		let contributor: AccountIdOf<T> = account("contributor", 0, SEED_2);
		let contribution: BalanceOf<T> = BalanceOf::<T>::one();
		AdminAccount::<T>::set(admin.clone());
		Contributions::<T>::insert(&contributor, contribution);
	}: _(RawOrigin::Signed(admin), T::Lookup::unlookup(contributor.clone()))
	verify {
		assert!(
			Contributions::<T>::get(&contributor).is_none(),
			"Contribution should have been removed."
		);
	}
}

impl_benchmark_test_suite! {
	Pallet,
	crate::mock::ExtBuilder::default().build_with_keystore(),
	crate::mock::Test
}
