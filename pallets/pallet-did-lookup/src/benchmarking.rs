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
#![cfg(feature = "runtime-benchmarks")]

//! Benchmarking

use crate::{
	signature::get_wrapped_payload, AccountIdOf, Call, Config, ConnectedAccounts, ConnectedDids, CurrencyOf, Pallet,
};

use codec::Encode;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite};
use frame_support::traits::{Currency, Get};
use frame_system::RawOrigin;
use kilt_support::{deposit::Deposit, traits::GenerateBenchmarkOrigin};
use sp_io::crypto::sr25519_generate;
use sp_runtime::{app_crypto::sr25519, KeyTypeId};

const SEED: u32 = 0;

// Free 2x deposit amount + existential deposit so that we can use this function
// to link an account two times to two different DIDs.
fn make_free_for_did<T: Config>(account: &AccountIdOf<T>) {
	let balance = <CurrencyOf<T> as Currency<AccountIdOf<T>>>::minimum_balance()
		+ <T as Config>::Deposit::get()
		+ <T as Config>::Deposit::get();
	<CurrencyOf<T> as Currency<AccountIdOf<T>>>::make_free_balance_be(account, balance);
}

benchmarks! {
	where_clause {
		where
		T::AccountId: From<sr25519::Public>,
		T::DidIdentifier: From<T::AccountId>,
		T::Signature: From<sr25519::Signature>,
		T::EnsureOrigin: GenerateBenchmarkOrigin<T::Origin, T::AccountId, T::DidIdentifier>,
	}

	associate_account {
		let caller: T::AccountId = account("caller", 0, SEED);
		let did: T::DidIdentifier = account("did", 0, SEED);
		let previous_did: T::DidIdentifier = account("prev", 0, SEED + 1);
		let connected_acc = sr25519_generate(KeyTypeId(*b"aura"), None);
		let connected_acc_id: T::AccountId = connected_acc.into();
		let bn: <T as frame_system::Config>::BlockNumber = 500_u32.into();

		let sig: T::Signature = sp_io::crypto::sr25519_sign(KeyTypeId(*b"aura"), &connected_acc, &get_wrapped_payload(&Encode::encode(&(&did, bn))[..]))
			.ok_or("Error while building signature.")?
			.into();

		make_free_for_did::<T>(&caller);

		// Add existing connected_acc -> previous_did connection that will be replaced
		Pallet::<T>::add_association(caller.clone(), previous_did.clone(), connected_acc_id.clone()).expect("should create previous association");
		assert!(ConnectedAccounts::<T>::get(&previous_did, T::AccountId::from(connected_acc)).is_some());
		let origin = T::EnsureOrigin::generate_origin(caller, did.clone());
	}: _<T::Origin>(origin, connected_acc_id, bn, sig)
	verify {
		assert!(ConnectedDids::<T>::get(T::AccountId::from(connected_acc)).is_some());
		assert!(ConnectedAccounts::<T>::get(&previous_did, T::AccountId::from(connected_acc)).is_none());
		assert!(ConnectedAccounts::<T>::get(did, T::AccountId::from(connected_acc)).is_some());
	}

	associate_sender {
		let caller: T::AccountId = account("caller", 0, SEED);
		let did: T::DidIdentifier = account("did", 0, SEED);
		let previous_did: T::DidIdentifier = account("prev", 0, SEED + 1);

		make_free_for_did::<T>(&caller);

		// Add existing sender -> previous_did connection that will be replaced
		Pallet::<T>::add_association(caller.clone(), previous_did.clone(), caller.clone()).expect("should create previous association");
		assert!(ConnectedAccounts::<T>::get(&previous_did, &caller).is_some());
		let origin = T::EnsureOrigin::generate_origin(caller.clone(), did.clone());
	}: _<T::Origin>(origin)
	verify {
		assert!(ConnectedDids::<T>::get(&caller).is_some());
		assert!(ConnectedAccounts::<T>::get(previous_did, &caller).is_none());
		assert!(ConnectedAccounts::<T>::get(did, caller).is_some());
	}

	remove_sender_association {
		let caller: T::AccountId = account("caller", 0, SEED);
		let did: T::DidIdentifier = account("did", 0, SEED);

		make_free_for_did::<T>(&caller);
		Pallet::<T>::add_association(caller.clone(), did.clone(), caller.clone()).expect("should create association");

		let origin = RawOrigin::Signed(caller.clone());
	}: _(origin)
	verify {
		assert!(ConnectedDids::<T>::get(&caller).is_none());
		assert!(ConnectedAccounts::<T>::get(did, caller).is_none());
	}

	remove_account_association {
		let caller: T::AccountId = account("caller", 0, SEED);
		let did: T::DidIdentifier = account("did", 0, SEED);
		make_free_for_did::<T>(&caller);

		Pallet::<T>::add_association(caller.clone(), did.clone(), caller.clone()).expect("should create association");

		let origin = T::EnsureOrigin::generate_origin(caller.clone(), did.clone());
		let caller_clone = caller.clone();
	}: _<T::Origin>(origin, caller_clone)
	verify {
		assert!(ConnectedDids::<T>::get(&caller).is_none());
		assert!(ConnectedAccounts::<T>::get(did, caller).is_none());
	}

	change_deposit_owner {
		let deposit_owner_old: T::AccountId = account("caller", 0, SEED);
		let deposit_owner_new: T::AccountId = account("caller", 1, SEED);
		let linkable_id: T::AccountId = deposit_owner_old.clone();
		let did: T::DidIdentifier = account("did", 0, SEED);
		make_free_for_did::<T>(&deposit_owner_old);
		make_free_for_did::<T>(&deposit_owner_new);

		Pallet::<T>::add_association(deposit_owner_old, did.clone(), linkable_id.clone()).expect("should create association");

		let origin = T::EnsureOrigin::generate_origin(deposit_owner_new.clone(), did);
		let id_arg = linkable_id.clone();
	}: _<T::Origin>(origin, id_arg)
	verify {
		assert_eq!(
			ConnectedDids::<T>::get(&linkable_id).expect("should retain link").deposit,
			Deposit {
				owner: deposit_owner_new,
				amount: <T as Config>::Deposit::get(),
			},
		);
	}

	update_deposit {
		let deposit_owner: T::AccountId = account("caller", 0, SEED);
		let linkable_id: T::AccountId = deposit_owner.clone();
		let did: T::DidIdentifier = account("did", 0, SEED);
		make_free_for_did::<T>(&deposit_owner);

		Pallet::<T>::add_association(
			deposit_owner.clone(),
			did,
			linkable_id.clone()
		).expect("should create association");

		let origin = RawOrigin::Signed(deposit_owner.clone());
		let id_arg = linkable_id.clone();
	}: _(origin, id_arg)
	verify {
		assert_eq!(
			ConnectedDids::<T>::get(&linkable_id).expect("should retain link").deposit,
			Deposit {
				owner: deposit_owner,
				amount: <T as Config>::Deposit::get(),
			},
		);
	}
}

#[cfg(test)]
use crate::Pallet as DidLookup;

impl_benchmark_test_suite!(
	DidLookup,
	crate::mock::ExtBuilder::default().build_with_keystore(),
	crate::mock::Test
);
