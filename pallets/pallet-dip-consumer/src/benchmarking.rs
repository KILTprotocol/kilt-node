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

use crate::{Config, IdentityProofOf, RuntimeCallOf};
use frame_benchmarking::v2::*;

pub struct WorstCaseOf<T: Config> {
	pub submitter: T::AccountId,
	pub subject: T::Identifier,
	pub proof: IdentityProofOf<T>,
	pub call: RuntimeCallOf<T>,
}

#[benchmarks(
	where
        T::ProofVerifier: GetWorstCase<Output = WorstCaseOf<T>>,
		<T as Config>::RuntimeCall: From<frame_system::Call<T>>,
)]
mod benchmarks {
	use frame_system::RawOrigin;
	use kilt_support::traits::GetWorstCase;
	use sp_std::boxed::Box;

	use crate::{benchmarking::WorstCaseOf, Call, Config, IdentityEntries, Pallet};

	#[benchmark]
	fn dispatch_as() {
		let WorstCaseOf {
			submitter,
			subject,
			proof,
			call,
		} = <T::ProofVerifier as GetWorstCase>::worst_case(());

		assert!(IdentityEntries::<T>::get(&subject).is_none());

		let origin = RawOrigin::Signed(submitter);
		let boxed_call = Box::from(call);

		let origin = <T as frame_system::Config>::RuntimeOrigin::from(origin);

		#[extrinsic_call]
		Pallet::<T>::dispatch_as(
			origin as <T as frame_system::Config>::RuntimeOrigin,
			subject,
			proof,
			boxed_call,
		);
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
