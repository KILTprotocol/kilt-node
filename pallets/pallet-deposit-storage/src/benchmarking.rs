// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2023 BOTLabs GmbH

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

#[benchmarks(
	where
        T: Config + pallet_balances::Config,
)]
mod benchmarks {
	use frame_support::traits::fungible::Mutate;
	use frame_system::RawOrigin;
	use kilt_support::Deposit;
	use sp_runtime::SaturatedConversion;

	use crate::{traits::BenchmarkHooks, Call, Config, DepositEntryOf, Deposits, HoldReason, Pallet};

	use super::*;

	const KILT: u128 = 10u128.pow(15);

	#[benchmark]
	fn reclaim_deposit() {
		let (submitter, namespace, key) = T::BenchmarkHooks::pre_reclaim_deposit();

		assert!(Deposits::<T>::get(&namespace, &key).is_none());

		let entry = DepositEntryOf::<T> {
			deposit: Deposit {
				amount: KILT.saturated_into(),
				owner: submitter.clone(),
			},
			reason: <T as Config>::RuntimeHoldReason::from(HoldReason::Deposit),
		};

		let amount = KILT * 100;

		<pallet_balances::Pallet<T> as Mutate<<T as frame_system::Config>::AccountId>>::set_balance(
			&submitter,
			amount.saturated_into(),
		);

		Pallet::<T>::add_deposit(namespace.clone(), key.clone(), entry).expect("Creating Deposit should not fail.");

		assert!(Deposits::<T>::get(&namespace, &key).is_some());

		let origin = RawOrigin::Signed(submitter);
		let cloned_namespace = namespace.clone();
		let cloned_key = key.clone();

		#[extrinsic_call]
		Pallet::<T>::reclaim_deposit(origin, cloned_namespace, cloned_key);

		assert!(Deposits::<T>::get(&namespace, &key).is_none());

		T::BenchmarkHooks::post_reclaim_deposit();
	}

	#[cfg(test)]
	mod benchmarks_tests {
		use crate::Pallet;
		use frame_benchmarking::impl_benchmark_test_suite;

		impl_benchmark_test_suite!(
			Pallet,
			crate::mock::ExtBuilder::default().build_with_keystore(),
			crate::mock::TestRuntime,
		);
	}
}
