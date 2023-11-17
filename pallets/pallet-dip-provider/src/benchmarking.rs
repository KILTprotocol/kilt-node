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

use crate::{Call, Config, Pallet};
use frame_benchmarking::v2::*;
use kilt_support::traits::{GenerateBenchmarkOrigin, Instanciate};

#[benchmarks(
	where
		T::CommitOriginCheck: GenerateBenchmarkOrigin<T::RuntimeOrigin, T::AccountId, T::Identifier>,
		T::AccountId: Instanciate,
		T::Identifier: Instanciate,
)]
mod benchmarks {
	use super::*;

	#[benchmark]
	fn commit_identity() {
		let submitter = T::AccountId::new(1);
		let subject = T::Identifier::new(1);
		let commitment_version = 0;

		assert!(Pallet::identity_commitments(&subject, commitment_version).is_none());

		let origin: T::RuntimeOrigin = T::CommitOriginCheck::generate_origin(submitter, subject);

		#[extrinsic_call]
		Pallet::commit_identity(origin, subject, Some(commitment_version));

		assert!(Pallet::identity_commitments(&subject, commitment_version).is_some());
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
