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

use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, Zero};
use frame_support::{
	dispatch::RawOrigin,
	traits::{Currency, Get},
	BoundedVec,
};
use sp_std::{boxed::Box, vec, vec::Vec};

use ctype::CtypeEntryOf;
use kilt_support::{
	deposit::Deposit,
	traits::{GenerateBenchmarkOrigin, GetWorstCase},
};

use crate::{
	mock::{
		generate_base_credential_entry, generate_base_public_credential_creation_op, generate_credential_id,
		insert_public_credentials,
	},
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
		<T as Config>::EnsureOrigin: GenerateBenchmarkOrigin<T::RuntimeOrigin, T::AccountId, T::AttesterId>,
		<T as Config>::SubjectId: GetWorstCase + Into<Vec<u8>> + sp_std::fmt::Debug,
		<T as Config>::CredentialId: Default,
		T::BlockNumber: From<u64>
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

		ctype::Ctypes::<T>::insert(ctype_hash, CtypeEntryOf::<T> {
			creator: attester.clone(),
			created_at: 0u64.into()
		});
		reserve_balance::<T>(&sender);
		let origin = <T as Config>::EnsureOrigin::generate_origin(sender, attester);
	}: _<T::RuntimeOrigin>(origin, creation_op)
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

		ctype::Ctypes::<T>::insert(ctype_hash, CtypeEntryOf::<T> {
			creator: attester,
			created_at: 0u64.into()
		});
		Pallet::<T>::add(origin.clone(), creation_op).expect("Pallet::add should not fail");
		let credential_id_clone = credential_id.clone();
	}: _<T::RuntimeOrigin>(origin, credential_id_clone, None)
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

		ctype::Ctypes::<T>::insert(ctype_hash, CtypeEntryOf::<T> {
			creator: attester,
			created_at: 0u64.into()
		});
		Pallet::<T>::add(origin.clone(), creation_op).expect("Pallet::add should not fail");
		Pallet::<T>::revoke(origin.clone(), credential_id.clone(), None).expect("Pallet::revoke should not fail");
		let credential_id_clone = credential_id.clone();
	}: _<T::RuntimeOrigin>(origin, credential_id_clone, None)
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

		ctype::Ctypes::<T>::insert(ctype_hash, CtypeEntryOf::<T> {
			creator: attester,
			created_at: 0u64.into()
		});
		Pallet::<T>::add(origin.clone(), creation_op).expect("Pallet::add should not fail");
		let credential_id_clone = credential_id.clone();
	}: _<T::RuntimeOrigin>(origin, credential_id_clone, None)
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

		ctype::Ctypes::<T>::insert(ctype_hash, CtypeEntryOf::<T> {
			creator: attester,
			created_at: 0u64.into()
		});
		Pallet::<T>::add(origin, creation_op).expect("Pallet::add should not fail");
		let origin = RawOrigin::Signed(sender);
		let credential_id_clone = credential_id.clone();
	}: _(origin, credential_id_clone)
	verify {
		assert!(!Credentials::<T>::contains_key(subject_id, &credential_id));
		assert!(!CredentialSubjects::<T>::contains_key(credential_id));
	}

	change_deposit_owner {
		let deposit_owner_old: AccountIdOf<T> = account("caller", 0, SEED);
		let deposit_owner_new: AccountIdOf<T> = account("caller", 1, SEED);
		let attester: T::AttesterId = account("attester", 0, SEED);
		let ctype_hash: T::Hash = T::Hash::default();
		let subject_id = <T as Config>::SubjectId::worst_case();
		let contents = BoundedVec::try_from(vec![0; <T as Config>::MaxEncodedClaimsLength::get() as usize]).expect("Contents should not fail.");
		let origin = <T as Config>::EnsureOrigin::generate_origin(deposit_owner_old.clone(), attester.clone());

		let creation_op = Box::new(generate_base_public_credential_creation_op::<T>(
			subject_id.clone().into().try_into().expect("Input conversion should not fail."),
			ctype_hash,
			contents,
		));
		let credential_id = generate_credential_id::<T>(&creation_op, &attester);

		reserve_balance::<T>(&deposit_owner_old);
		reserve_balance::<T>(&deposit_owner_new);

		ctype::Ctypes::<T>::insert(ctype_hash, CtypeEntryOf::<T> {
			creator: attester.clone(),
			created_at: 0u64.into()
		});
		Pallet::<T>::add(origin, creation_op).expect("Pallet::add should not fail");
		let credential_id_clone = credential_id.clone();
		let origin = <T as Config>::EnsureOrigin::generate_origin(deposit_owner_new.clone(), attester);
	}: _<T::RuntimeOrigin>(origin, credential_id_clone)
	verify {
		assert_eq!(
			Credentials::<T>::get(subject_id, &credential_id)
				.expect("Credential should be present in storage")
				.deposit
				.owner,
			deposit_owner_new
		);
	}

	update_deposit {
		let deposit_owner: AccountIdOf<T> = account("caller", 0, SEED);
		let attester: T::AttesterId = account("attester", 0, SEED);
		let ctype_hash: T::Hash = T::Hash::default();
		let subject_id = <T as Config>::SubjectId::worst_case();
		let origin = <T as Config>::EnsureOrigin::generate_origin(deposit_owner.clone(), attester.clone());

		reserve_balance::<T>(&deposit_owner);
		ctype::Ctypes::<T>::insert(ctype_hash, CtypeEntryOf::<T> {
			creator: attester.clone(),
			created_at: 0u64.into()
		});

		let credential_entry = generate_base_credential_entry::<T>(
			deposit_owner.clone(),
			T::BlockNumber::zero(),
			attester,
			Some(ctype_hash),
			Some(Deposit::<T::AccountId, BalanceOf<T>> {
				owner: deposit_owner.clone(),
				amount: <T as Config>::Deposit::get() + <T as Config>::Deposit::get(),
			})
		);
		let credential_id: CredentialIdOf<T> = Default::default();
		insert_public_credentials::<T>(
			subject_id.clone(),
			credential_id.clone(),
			credential_entry
		);
		let credential_id_clone = credential_id.clone();

		let origin = RawOrigin::Signed(deposit_owner);
	}: _(origin, credential_id_clone)
	verify {
		assert_eq!(
			Credentials::<T>::get(subject_id, &credential_id)
				.expect("Credential should be present in storage")
				.deposit
				.amount,
			T::Deposit::get()
		);
	}
}

impl_benchmark_test_suite! {
	Pallet,
	crate::mock::ExtBuilder::default().build_with_keystore(),
	crate::mock::Test
}
