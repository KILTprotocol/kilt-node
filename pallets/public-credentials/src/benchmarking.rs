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
	dispatch::RawOrigin,
	traits::{Currency, Get},
	BoundedVec,
};
use sp_std::{boxed::Box, vec, vec::Vec};

use kilt_support::traits::{GenerateBenchmarkOrigin, GetWorstCase};

use crate::{
	mock::{generate_base_public_credential_creation_op, generate_credential_id},
	*,
};

const SEED: u32 = 0;

fn reserve_balance<T: Config>(acc: &T::AccountId) {
	// Has to be more than the deposit, we do 2x just to be safe
	CurrencyOf::<T>::make_free_balance_be(acc, <T as Config>::Deposit::get() + <T as Config>::Deposit::get());
}

benchmarks! {
	where_clause {
		where
		T: core::fmt::Debug,
		T: Config,
		T: ctype::Config<CtypeCreatorId = T::AttesterId>,
		<T as Config>::EnsureOrigin: GenerateBenchmarkOrigin<T::Origin, T::AccountId, T::AttesterId>,
		<T as Config>::SubjectId: GetWorstCase + Into<Vec<u8>> + sp_std::fmt::Debug,
	}

	add {
		let c in 1 .. T::MaxEncodedClaimsLength::get();
		let sender: T::AccountId = account("sender", 0, SEED);
		let attester: T::AttesterId = account("attester", 0, SEED);
		let ctype_hash: T::Hash = T::Hash::default();
		let subject_id = <T as Config>::SubjectId::worst_case();
		let contents = BoundedVec::try_from(vec![0; c as usize]).expect("Contents should not fail.");

		let creation_op = Box::new(generate_base_public_credential_creation_op::<T>(
			subject_id.clone().into().try_into().expect("Input conversion should not fail."),
			ctype_hash,
			contents,
		));
		let credential_id = generate_credential_id::<T>(&creation_op, &attester);

		ctype::Ctypes::<T>::insert(&ctype_hash, attester.clone());
		reserve_balance::<T>(&sender);
		let origin = <T as Config>::EnsureOrigin::generate_origin(sender, attester);
	}: _<T::Origin>(origin, creation_op)
	verify {
		assert!(Credentials::<T>::contains_key(subject_id, &credential_id));
		assert!(CredentialSubjects::<T>::contains_key(&credential_id));
	}

	// Very similar setup as `remove`
	revoke {
		let sender: T::AccountId = account("sender", 0, SEED);
		let attester: T::AttesterId = account("attester", 0, SEED);
		let ctype_hash: T::Hash = T::Hash::default();
		let subject_id = <T as Config>::SubjectId::worst_case();
		let contents = BoundedVec::try_from(vec![0; <T as Config>::MaxEncodedClaimsLength::get() as usize]).expect("Contents should not fail.");
		let origin = <T as Config>::EnsureOrigin::generate_origin(sender.clone(), attester.clone());

		let creation_op = Box::new(generate_base_public_credential_creation_op::<T>(
			subject_id.clone().into().try_into().expect("Input conversion should not fail."),
			ctype_hash,
			contents,
		));
		let credential_id = generate_credential_id::<T>(&creation_op, &attester);

		reserve_balance::<T>(&sender);

		ctype::Ctypes::<T>::insert(&ctype_hash, attester);
		Pallet::<T>::add(origin.clone(), creation_op).expect("Pallet::add should not fail");
		let credential_id_clone = credential_id.clone();
	}: _<T::Origin>(origin, credential_id_clone, None)
	verify {
		assert!(Credentials::<T>::get(subject_id, &credential_id).expect("Credential should be present in storage").revoked);
	}

	// Very similar setup as `remove`
	unrevoke {
		let sender: T::AccountId = account("sender", 0, SEED);
		let attester: T::AttesterId = account("attester", 0, SEED);
		let ctype_hash: T::Hash = T::Hash::default();
		let subject_id = <T as Config>::SubjectId::worst_case();
		let contents = BoundedVec::try_from(vec![0; <T as Config>::MaxEncodedClaimsLength::get() as usize]).expect("Contents should not fail.");
		let origin = <T as Config>::EnsureOrigin::generate_origin(sender.clone(), attester.clone());

		let creation_op = Box::new(generate_base_public_credential_creation_op::<T>(
			subject_id.clone().into().try_into().expect("Input conversion should not fail."),
			ctype_hash,
			contents,
		));
		let credential_id = generate_credential_id::<T>(&creation_op, &attester);

		reserve_balance::<T>(&sender);

		ctype::Ctypes::<T>::insert(&ctype_hash, attester);
		Pallet::<T>::add(origin.clone(), creation_op).expect("Pallet::add should not fail");
		Pallet::<T>::revoke(origin.clone(), credential_id.clone(), None).expect("Pallet::revoke should not fail");
		let credential_id_clone = credential_id.clone();
	}: _<T::Origin>(origin, credential_id_clone, None)
	verify {
		assert!(!Credentials::<T>::get(subject_id, &credential_id).expect("Credential should be present in storage").revoked);
	}

	remove {
		let sender: T::AccountId = account("sender", 0, SEED);
		let attester: T::AttesterId = account("attester", 0, SEED);
		let ctype_hash: T::Hash = T::Hash::default();
		let subject_id = <T as Config>::SubjectId::worst_case();
		let contents = BoundedVec::try_from(vec![0; <T as Config>::MaxEncodedClaimsLength::get() as usize]).expect("Contents should not fail.");
		let origin = <T as Config>::EnsureOrigin::generate_origin(sender.clone(), attester.clone());

		let creation_op = Box::new(generate_base_public_credential_creation_op::<T>(
			subject_id.clone().into().try_into().expect("Input conversion should not fail."),
			ctype_hash,
			contents,
		));
		let credential_id = generate_credential_id::<T>(&creation_op, &attester);

		reserve_balance::<T>(&sender);

		ctype::Ctypes::<T>::insert(&ctype_hash, attester);
		Pallet::<T>::add(origin.clone(), creation_op).expect("Pallet::add should not fail");
		let credential_id_clone = credential_id.clone();
	}: _<T::Origin>(origin, credential_id_clone, None)
	verify {
		assert!(!Credentials::<T>::contains_key(subject_id, &credential_id));
		assert!(!CredentialSubjects::<T>::contains_key(credential_id));
	}

	reclaim_deposit {
		let sender: T::AccountId = account("sender", 0, SEED);
		let attester: T::AttesterId = account("attester", 0, SEED);
		let ctype_hash: T::Hash = T::Hash::default();
		let subject_id = <T as Config>::SubjectId::worst_case();
		let contents = BoundedVec::try_from(vec![0; <T as Config>::MaxEncodedClaimsLength::get() as usize]).expect("Contents should not fail.");
		let origin = <T as Config>::EnsureOrigin::generate_origin(sender.clone(), attester.clone());

		let creation_op = Box::new(generate_base_public_credential_creation_op::<T>(
			subject_id.clone().into().try_into().expect("Input conversion should not fail."),
			ctype_hash,
			contents,
		));
		let credential_id = generate_credential_id::<T>(&creation_op, &attester);

		reserve_balance::<T>(&sender);

		ctype::Ctypes::<T>::insert(&ctype_hash, attester);
		Pallet::<T>::add(origin, creation_op).expect("Pallet::add should not fail");
		let origin = RawOrigin::Signed(sender);
		let credential_id_clone = credential_id.clone();
	}: _(origin, credential_id_clone)
	verify {
		assert!(!Credentials::<T>::contains_key(subject_id, &credential_id));
		assert!(!CredentialSubjects::<T>::contains_key(credential_id));
	}
}

impl_benchmark_test_suite! {
	Pallet,
	crate::mock::ExtBuilder::default().build_with_keystore(),
	crate::mock::Test
}
