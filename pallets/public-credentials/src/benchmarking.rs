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
	traits::{Currency, Get},
	BoundedVec,
};

use kilt_support::traits::{GetWorstCase, GenerateBenchmarkOrigin};

use crate::{*, mock::generate_base_public_credential_creation_op};

const SEED: u32 = 0;

benchmarks! {
	where_clause {
		where
		T: core::fmt::Debug,
		T: Config,
		T: attestation::Config,
		T: ctype::Config<CtypeCreatorId = T::AttesterId>,
		<T as Config>::EnsureOrigin: GenerateBenchmarkOrigin<T::Origin, T::AccountId, T::AttesterId>,
		<T as Config>::SubjectId: GetWorstCase + Into<InputSubjectIdOf<T>> + sp_std::fmt::Debug,
	}

	add {
		let c in 1 .. T::MaxEncodedClaimsLength::get();
		let sender: T::AccountId = account("sender", 0, SEED);
		let attester: T::AttesterId = account("attester", 0, SEED);
		let claim_hash: T::Hash = T::Hash::default();
		let ctype_hash: T::Hash = T::Hash::default();
		let subject_id = <T as Config>::SubjectId::worst_case();
		let contents = BoundedVec::try_from(vec![0; c as usize]).expect("Contents should not fail.");

		let creation_op = Box::new(generate_base_public_credential_creation_op::<T>(
			subject_id.into(),
			claim_hash,
			ctype_hash,
			contents,
			None,
		));

		ctype::Ctypes::<T>::insert(&ctype_hash, attester.clone());
		CurrencyOf::<T>::make_free_balance_be(&sender, <T as attestation::Config>::Deposit::get() + <T as attestation::Config>::Deposit::get() + <T as Config>::Deposit::get());
		let origin = <T as Config>::EnsureOrigin::generate_origin(sender, attester);
	}: _<T::Origin>(origin, creation_op)
	verify {}
}

impl_benchmark_test_suite! {
	Pallet,
	crate::mock::ExtBuilder::default().build_with_keystore(),
	crate::mock::Test
}
