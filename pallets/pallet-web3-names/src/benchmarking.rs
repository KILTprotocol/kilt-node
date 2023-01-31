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
#![cfg(feature = "runtime-benchmarks")]

use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, vec, Vec, Zero};
use frame_support::{
	pallet_prelude::EnsureOrigin,
	sp_runtime::SaturatedConversion,
	traits::{Currency, Get},
	BoundedVec,
};
use frame_system::RawOrigin;
use sp_runtime::app_crypto::sr25519;

use kilt_support::{deposit::Deposit, traits::GenerateBenchmarkOrigin};

use crate::{
	mock::insert_raw_w3n, AccountIdOf, Banned, Call, Config, CurrencyOf, Names, Owner, Pallet, Web3NameOf,
	Web3NameOwnerOf,
};

const CALLER_SEED: u32 = 0;
const OWNER_SEED: u32 = 1;

fn make_free_for_did<T: Config>(account: &AccountIdOf<T>) {
	let balance = <CurrencyOf<T> as Currency<AccountIdOf<T>>>::minimum_balance()
		+ <T as Config>::Deposit::get()
		+ <T as Config>::Deposit::get();
	<CurrencyOf<T> as Currency<AccountIdOf<T>>>::make_free_balance_be(account, balance);
}

fn generate_web3_name_input(length: usize) -> Vec<u8> {
	vec![b'1'; length]
}

benchmarks! {
	where_clause {
		where
		T::AccountId: From<sr25519::Public>,
		T::Web3NameOwner: From<T::AccountId>,
		T::OwnerOrigin: GenerateBenchmarkOrigin<T::RuntimeOrigin, T::AccountId, T::Web3NameOwner>,
		T::BanOrigin: EnsureOrigin<T::RuntimeOrigin>,
	}

	claim {
		let n in (T::MinNameLength::get()) .. (T::MaxNameLength::get());
		let caller: AccountIdOf<T> = account("caller", 0, CALLER_SEED);
		let owner: Web3NameOwnerOf<T> = account("owner", 0, OWNER_SEED);
		let web3_name_input: BoundedVec<u8, T::MaxNameLength> = BoundedVec::try_from(generate_web3_name_input(n.saturated_into())).expect("BoundedVec creation should not fail.");
		let web3_name_input_clone = web3_name_input.clone();
		let origin = T::OwnerOrigin::generate_origin(caller.clone(), owner.clone());

		make_free_for_did::<T>(&caller);
	}: _<T::RuntimeOrigin>(origin, web3_name_input_clone)
	verify {
		let web3_name = Web3NameOf::<T>::try_from(web3_name_input.to_vec()).unwrap();
		assert!(Names::<T>::get(&owner).is_some());
		assert!(Owner::<T>::get(&web3_name).is_some());
	}

	release_by_owner {
		let caller: AccountIdOf<T> = account("caller", 0, CALLER_SEED);
		let owner: Web3NameOwnerOf<T> = account("owner", 0, OWNER_SEED);
		let web3_name_input: BoundedVec<u8, T::MaxNameLength> = BoundedVec::try_from(generate_web3_name_input(T::MaxNameLength::get().saturated_into())).expect("BoundedVec creation should not fail.");
		let origin = T::OwnerOrigin::generate_origin(caller.clone(), owner.clone());

		make_free_for_did::<T>(&caller);
		Pallet::<T>::claim(origin.clone(), web3_name_input.clone()).expect("Should register the claimed web3 name.");
	}: _<T::RuntimeOrigin>(origin)
	verify {
		let web3_name = Web3NameOf::<T>::try_from(web3_name_input.to_vec()).unwrap();
		assert!(Names::<T>::get(&owner).is_none());
		assert!(Owner::<T>::get(&web3_name).is_none());
	}

	reclaim_deposit {
		let n in (T::MinNameLength::get()) .. (T::MaxNameLength::get());
		let caller: AccountIdOf<T> = account("caller", 0, CALLER_SEED);
		let owner: Web3NameOwnerOf<T> = account("owner", 0, OWNER_SEED);
		let web3_name_input: BoundedVec<u8, T::MaxNameLength> = BoundedVec::try_from(generate_web3_name_input(n.saturated_into())).expect("BoundedVec creation should not fail.");
		let web3_name_input_clone = web3_name_input.clone();
		let did_origin = T::OwnerOrigin::generate_origin(caller.clone(), owner.clone());
		let signed_origin = RawOrigin::Signed(caller.clone());

		make_free_for_did::<T>(&caller);
		Pallet::<T>::claim(did_origin, web3_name_input.clone()).expect("Should register the claimed web3 name.");
	}: _(signed_origin, web3_name_input_clone)
	verify {
		let web3_name = Web3NameOf::<T>::try_from(web3_name_input.to_vec()).unwrap();
		assert!(Names::<T>::get(&owner).is_none());
		assert!(Owner::<T>::get(&web3_name).is_none());
	}

	ban {
		let n in (T::MinNameLength::get()) .. (T::MaxNameLength::get());
		let caller: AccountIdOf<T> = account("caller", 0, CALLER_SEED);
		let owner: Web3NameOwnerOf<T> = account("owner", 0, OWNER_SEED);
		let web3_name_input: BoundedVec<u8, T::MaxNameLength> = BoundedVec::try_from(generate_web3_name_input(n.saturated_into())).expect("BoundedVec creation should not fail.");
		let web3_name_input_clone = web3_name_input.clone();
		let did_origin = T::OwnerOrigin::generate_origin(caller.clone(), owner.clone());
		let ban_origin = RawOrigin::Root;

		make_free_for_did::<T>(&caller);
		Pallet::<T>::claim(did_origin, web3_name_input.clone()).expect("Should register the claimed web3 name.");
	}: _(ban_origin, web3_name_input_clone)
	verify {
		let web3_name = Web3NameOf::<T>::try_from(web3_name_input.to_vec()).unwrap();
		assert!(Names::<T>::get(&owner).is_none());
		assert!(Owner::<T>::get(&web3_name).is_none());
		assert!(Banned::<T>::get(&web3_name).is_some());
	}

	unban {
		let n in (T::MinNameLength::get()) .. (T::MaxNameLength::get());
		let caller: AccountIdOf<T> = account("caller", 0, CALLER_SEED);
		let owner: Web3NameOwnerOf<T> = account("owner", 0, OWNER_SEED);
		let web3_name_input: BoundedVec<u8, T::MaxNameLength> = BoundedVec::try_from(generate_web3_name_input(n.saturated_into())).expect("BoundedVec creation should not fail.");
		let web3_name_input_clone = web3_name_input.clone();
		let ban_origin = RawOrigin::Root;

		make_free_for_did::<T>(&caller);
		Pallet::<T>::ban(ban_origin.clone().into(), web3_name_input.clone()).expect("Should ban the web3 name.");
	}: _(ban_origin, web3_name_input_clone)
	verify {
		let web3_name = Web3NameOf::<T>::try_from(web3_name_input.to_vec()).unwrap();
		assert!(Names::<T>::get(&owner).is_none());
		assert!(Owner::<T>::get(&web3_name).is_none());
		assert!(Banned::<T>::get(&web3_name).is_none());
	}

	change_deposit_owner {
		let deposit_owner_old: AccountIdOf<T> = account("caller", 0, CALLER_SEED);
		let deposit_owner_new: AccountIdOf<T> = account("caller", 1, CALLER_SEED);
		let owner: Web3NameOwnerOf<T> = account("owner", 0, OWNER_SEED);
		let web3_name_input: BoundedVec<u8, T::MaxNameLength> = BoundedVec::try_from(
			generate_web3_name_input(T::MaxNameLength::get().saturated_into())
		).expect("BoundedVec creation should not fail.");
		let web3_name_input_clone = web3_name_input.clone();
		let origin_create = T::OwnerOrigin::generate_origin(deposit_owner_old.clone(), owner.clone());

		make_free_for_did::<T>(&deposit_owner_old);
		make_free_for_did::<T>(&deposit_owner_new);
		Pallet::<T>::claim(origin_create, web3_name_input.clone()).expect("Should register the claimed web3 name.");

		let origin = T::OwnerOrigin::generate_origin(deposit_owner_new.clone(), owner);
	}: _<T::RuntimeOrigin>(origin)
	verify {
		let web3_name = Web3NameOf::<T>::try_from(web3_name_input.to_vec()).unwrap();
		assert_eq!(Owner::<T>::get(&web3_name).expect("w3n should exists").deposit, Deposit {
			owner: deposit_owner_new,
			amount: <T as Config>::Deposit::get(),
		});
	}

	update_deposit {
		let deposit_owner: AccountIdOf<T> = account("caller", 0, CALLER_SEED);
		let owner: Web3NameOwnerOf<T> = account("owner", 0, OWNER_SEED);
		let web3_name_input: BoundedVec<u8, T::MaxNameLength> = BoundedVec::try_from(
			generate_web3_name_input(T::MaxNameLength::get().saturated_into())
		).expect("BoundedVec creation should not fail.");
		let web3_name = Web3NameOf::<T>::try_from(web3_name_input.to_vec()).unwrap();

		make_free_for_did::<T>(&deposit_owner);
		insert_raw_w3n::<T>(
			deposit_owner.clone(),
			owner,
			web3_name.clone(),
			T::BlockNumber::zero(),
			<T as Config>::Deposit::get() + <T as Config>::Deposit::get()
		);

		let origin = RawOrigin::Signed(deposit_owner.clone());
	}: _(origin, web3_name_input)
	verify {
		assert_eq!(Owner::<T>::get(&web3_name).expect("w3n should exists").deposit, Deposit {
			owner: deposit_owner,
			amount: <T as Config>::Deposit::get(),
		});
	}
}

impl_benchmark_test_suite! {
	Pallet,
	crate::mock::ExtBuilder::default().build_with_keystore(),
	crate::mock::Test
}
