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

use crate::{traits::IdentityProofVerifier, Call, Config, IdentityEntries, Pallet};
use frame_benchmarking::v2::*;
use frame_system::RawOrigin;
use kilt_support::{
	benchmark::IdentityContext,
	traits::{GetWorstCase, Instanciate},
};

#[benchmarks(
	where
		T::AccountId: Instanciate,
		T::Identifier: Instanciate,
        <<T as Config>::ProofVerifier as IdentityProofVerifier<T>>::Proof: GetWorstCase<IdentityContext<T::Identifier, T::AccountId>>,
        <T as Config>::RuntimeCall: From<frame_system::Call<T>>,
)]
mod benchmarks {

	use super::*;

	type IdentityContextOf<Runtime> =
		IdentityContext<<Runtime as Config>::Identifier, <Runtime as frame_system::Config>::AccountId>;

	#[benchmark]
	fn dispatch_as() {
		let submitter = T::AccountId::new(1);
		let subject = T::Identifier::new(1);

		let context = IdentityContext::<T::Identifier, T::AccountId> {
			did: subject.clone(),
			submitter: submitter.clone(),
		};

		assert!(IdentityEntries::<T>::get(&subject).is_none());

		let origin = RawOrigin::Signed(submitter);

		let call: <T as Config>::RuntimeCall = frame_system::Call::<T>::remark { remark: vec![] }.into();

		let boxed_call = Box::from(call);

		let proof = <<<T as Config>::ProofVerifier as IdentityProofVerifier<T>>::Proof as GetWorstCase<
			IdentityContextOf<T>,
		>>::worst_case(context);

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
