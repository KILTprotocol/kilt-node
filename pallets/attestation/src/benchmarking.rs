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

use frame_benchmarking::{account, benchmarks};
use frame_support::traits::{fungible::Mutate, Get};
use frame_system::{pallet_prelude::BlockNumberFor, RawOrigin};
use sp_runtime::traits::Hash;

use ctype::CtypeEntryOf;
use kilt_support::traits::GenerateBenchmarkOrigin;

use crate::*;

const SEED: u32 = 0;

benchmarks! {
	where_clause {
		where
		T: core::fmt::Debug,
		<T as Config>::EnsureOrigin: GenerateBenchmarkOrigin<T::RuntimeOrigin, T::AccountId, T::AttesterId>,
		T: ctype::Config<CtypeCreatorId = T::AttesterId>,
		BlockNumberFor<T>: From<u64>,
		<T as Config>::Currency: Mutate<T::AccountId>
	}

	add {
		let sender: T::AccountId = account("sender", 0, SEED);
		let attester: T::AttesterId = account("attester", 0, SEED);
		let claim_hash: T::Hash = T::Hashing::hash(b"claim");
		let ctype_hash: T::Hash = T::Hash::default();

		ctype::Ctypes::<T>::insert(ctype_hash, CtypeEntryOf::<T> {
			creator: attester.clone(),
			created_at: 0u64.into()
		});
		<T as Config>::Currency::set_balance(&sender, <T as Config>::Deposit::get() + <T as Config>::Deposit::get());

		let origin = <T as Config>::EnsureOrigin::generate_origin(sender.clone(), attester.clone());
	}: _<T::RuntimeOrigin>(origin, claim_hash, ctype_hash, None)
	verify {
		assert!(Attestations::<T>::contains_key(claim_hash));
		assert_eq!(Pallet::<T>::attestations(claim_hash), Some(AttestationDetails {
			ctype_hash,
			attester,
			authorization_id: None,
			revoked: false,
			deposit: kilt_support::Deposit {
				owner: sender,
				amount: <T as Config>::Deposit::get(),
			}
		}));
	}

	revoke {
		let sender: T::AccountId = account("sender", 0, SEED);
		let attester: T::AttesterId = account("attester", 0, SEED);
		let claim_hash: T::Hash = T::Hashing::hash(b"claim");
		let ctype_hash: T::Hash = T::Hash::default();

		ctype::Ctypes::<T>::insert(ctype_hash, CtypeEntryOf::<T> {
			creator: attester.clone(),
			created_at: 0u64.into()
		});
		<T as Config>::Currency::set_balance(&sender, <T as Config>::Deposit::get() + <T as Config>::Deposit::get());

		let origin = <T as Config>::EnsureOrigin::generate_origin(sender.clone(), attester.clone());
		Pallet::<T>::add(origin.clone(), claim_hash, ctype_hash, None)?;
	}: _<T::RuntimeOrigin>(origin, claim_hash, None)
	verify {
		assert!(Attestations::<T>::contains_key(claim_hash));
		assert_eq!(Attestations::<T>::get(claim_hash), Some(AttestationDetails {
			ctype_hash,
			attester,
			authorization_id: None,
			revoked: true,
			deposit: kilt_support::Deposit {
				owner: sender,
				amount: <T as Config>::Deposit::get(),
			}
		}));
	}

	remove {
		let attester: T::AttesterId = account("attester", 0, SEED);
		let sender: T::AccountId = account("sender", 0, SEED);
		let claim_hash: T::Hash = T::Hashing::hash(b"claim");
		let ctype_hash: T::Hash = T::Hash::default();

		ctype::Ctypes::<T>::insert(ctype_hash, CtypeEntryOf::<T> {
			creator: attester.clone(),
			created_at: 0u64.into()
		});
		<T as Config>::Currency::set_balance(&sender, <T as Config>::Deposit::get() + <T as Config>::Deposit::get());

		let origin = <T as Config>::EnsureOrigin::generate_origin(sender.clone(), attester.clone());
		Pallet::<T>::add(origin, claim_hash, ctype_hash, None)?;
		let origin = <T as Config>::EnsureOrigin::generate_origin(sender, attester);
	}: _<T::RuntimeOrigin>(origin, claim_hash, None)
	verify {
		assert!(!Attestations::<T>::contains_key(claim_hash));
	}

	reclaim_deposit {
		let attester: T::AttesterId = account("attester", 0, SEED);
		let sender: T::AccountId = account("sender", 0, SEED);
		let claim_hash: T::Hash = T::Hashing::hash(b"claim");
		let ctype_hash: T::Hash = T::Hash::default();

		ctype::Ctypes::<T>::insert(ctype_hash, CtypeEntryOf::<T> {
			creator: attester.clone(),
			created_at: 0u64.into()
		});
		<T as Config>::Currency::set_balance(&sender, <T as Config>::Deposit::get() + <T as Config>::Deposit::get());

		let origin = <T as Config>::EnsureOrigin::generate_origin(sender.clone(), attester);
		Pallet::<T>::add(origin, claim_hash, ctype_hash, None)?;
		let origin = RawOrigin::Signed(sender);
	}: _(origin, claim_hash)
	verify {
		assert!(!Attestations::<T>::contains_key(claim_hash));
	}

	change_deposit_owner {
		let attester: T::AttesterId = account("attester", 0, SEED);
		let deposit_owner_old: T::AccountId = account("sender", 0, SEED);
		let deposit_owner_new: T::AccountId = account("sender", 1, SEED);
		let claim_hash: T::Hash = T::Hashing::hash(b"claim");
		let ctype_hash: T::Hash = T::Hash::default();

		ctype::Ctypes::<T>::insert(ctype_hash, CtypeEntryOf::<T> {
			creator: attester.clone(),
			created_at: 0u64.into()
		});
		<T as Config>::Currency::set_balance(&deposit_owner_old, <T as Config>::Deposit::get() + <T as Config>::Deposit::get());
		<T as Config>::Currency::set_balance(&deposit_owner_new, <T as Config>::Deposit::get() + <T as Config>::Deposit::get());

		let origin = <T as Config>::EnsureOrigin::generate_origin(deposit_owner_old, attester.clone());
		Pallet::<T>::add(origin, claim_hash, ctype_hash, None)?;
		let origin = <T as Config>::EnsureOrigin::generate_origin(deposit_owner_new.clone(), attester.clone());
	}: _<T::RuntimeOrigin>(origin, claim_hash)
	verify {
		assert_eq!(Attestations::<T>::get(claim_hash), Some(AttestationDetails {
			ctype_hash,
			attester,
			authorization_id: None,
			revoked: false,
			deposit: kilt_support::Deposit {
				owner: deposit_owner_new,
				amount: <T as Config>::Deposit::get(),
			}
		}));
	}

	update_deposit {
		let attester: T::AttesterId = account("attester", 0, SEED);
		let deposit_owner: T::AccountId = account("sender", 0, SEED);
		let claim_hash: T::Hash = T::Hashing::hash(b"claim");
		let ctype_hash: T::Hash = T::Hash::default();

		ctype::Ctypes::<T>::insert(ctype_hash, CtypeEntryOf::<T> {
			creator: attester.clone(),
			created_at: 0u64.into()
		});
		<T as Config>::Currency::set_balance(&deposit_owner, <T as Config>::Deposit::get() + <T as Config>::Deposit::get());

		let origin = <T as Config>::EnsureOrigin::generate_origin(deposit_owner.clone(), attester.clone());
		Pallet::<T>::add(origin, claim_hash, ctype_hash, None).expect("claim should be added");

		let origin = RawOrigin::Signed(deposit_owner.clone());
	}: _(origin, claim_hash)
	verify {
		assert_eq!(Attestations::<T>::get(claim_hash), Some(AttestationDetails {
			ctype_hash,
			attester,
			authorization_id: None,
			revoked: false,
			deposit: kilt_support::Deposit {
				owner: deposit_owner,
				amount: <T as Config>::Deposit::get(),
			}
		}));
	}

	impl_benchmark_test_suite!(
		Pallet,
		crate::mock::ExtBuilder::default().build_with_keystore(),
		crate::mock::Test
	)
}
