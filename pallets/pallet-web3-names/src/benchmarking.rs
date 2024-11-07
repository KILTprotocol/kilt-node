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

// Old benchmarking macros are a mess.
#![allow(clippy::tests_outside_test_module)]

use frame_benchmarking::{account, benchmarks_instance_pallet};
use frame_support::{
	pallet_prelude::EnsureOrigin,
	sp_runtime::{traits::Zero, SaturatedConversion},
	traits::{
		fungible::{Inspect, Mutate},
		Get,
	},
	BoundedVec,
};
use frame_system::{pallet_prelude::BlockNumberFor, RawOrigin};
use sp_runtime::app_crypto::sr25519;
use sp_std::vec::Vec;

use kilt_support::{traits::GenerateBenchmarkOrigin, Deposit};

use crate::{
	mock::insert_raw_w3n, AccountIdOf, Banned, Call, Config, CurrencyOf, Error, Names, Owner, Pallet, Web3NameOf,
	Web3NameOwnerOf,
};

pub trait BenchmarkHelper {
	fn generate_name_input_with_length(length: usize) -> Vec<u8>;
}

impl BenchmarkHelper for () {
	fn generate_name_input_with_length(length: usize) -> Vec<u8> {
		sp_std::vec![b'a'; length]
	}
}

const CALLER_SEED: u32 = 0;
const OWNER_SEED: u32 = 1;

fn make_free_for_did<T, I>(account: &AccountIdOf<T>)
where
	T: Config<I>,
	I: 'static,
	<T as Config<I>>::Currency: Mutate<T::AccountId>,
{
	let balance = <CurrencyOf<T, I> as Inspect<AccountIdOf<T>>>::minimum_balance()
		+ <T as Config<I>>::Deposit::get()
		+ <T as Config<I>>::Deposit::get();
	CurrencyOf::<T, I>::set_balance(account, balance);
}

benchmarks_instance_pallet! {
	where_clause {
		where
		T::AccountId: From<sr25519::Public>,
		<T as Config<I>>::Web3NameOwner: From<T::AccountId>,
		<T as Config<I>>::OwnerOrigin: GenerateBenchmarkOrigin<T::RuntimeOrigin, T::AccountId, <T as Config<I>>::Web3NameOwner>,
		<T as Config<I>>::BanOrigin: EnsureOrigin<T::RuntimeOrigin>,
		<<T as Config<I>>::Web3Name as TryFrom<Vec<u8>>>::Error: Into<Error<T, I>>,
		<T as Config<I>>::Currency: Mutate<T::AccountId>,
	}

	claim {
		let n in (<T as Config<I>>::MinNameLength::get()) .. (<T as Config<I>>::MaxNameLength::get());
		let caller: AccountIdOf<T> = account("caller", 0, CALLER_SEED);
		let owner: Web3NameOwnerOf<T, I> = account("owner", 0, OWNER_SEED);
		let web3_name_input: BoundedVec<u8, <T as Config<I>>::MaxNameLength> = BoundedVec::try_from(<T as Config<I>>::BenchmarkHelper::generate_name_input_with_length(n.saturated_into())).expect("BoundedVec creation should not fail.");
		let web3_name_input_clone = web3_name_input.clone();
		let origin = <T as Config<I>>::OwnerOrigin::generate_origin(caller.clone(), owner.clone());

		make_free_for_did::<T, I>(&caller);
	}: _<T::RuntimeOrigin>(origin, web3_name_input_clone)
	verify {
		let Ok(web3_name) = Web3NameOf::<T, I>::try_from(web3_name_input.to_vec()) else {
			panic!();
		};
		assert!(Names::<T, I>::get(&owner).is_some());
		assert!(Owner::<T, I>::get(&web3_name).is_some());
	}

	release_by_owner {
		let caller: AccountIdOf<T> = account("caller", 0, CALLER_SEED);
		let owner: Web3NameOwnerOf<T, I> = account("owner", 0, OWNER_SEED);
		let web3_name_input: BoundedVec<u8, <T as Config<I>>::MaxNameLength> = BoundedVec::try_from(<T as Config<I>>::BenchmarkHelper::generate_name_input_with_length(<T as Config<I>>::MaxNameLength::get().saturated_into())).expect("BoundedVec creation should not fail.");
		let origin = <T as Config<I>>::OwnerOrigin::generate_origin(caller.clone(), owner.clone());

		make_free_for_did::<T, I>(&caller);
		Pallet::<T, I>::claim(origin.clone(), web3_name_input.clone()).expect("Should register the claimed web3 name.");
	}: _<T::RuntimeOrigin>(origin)
	verify {
		let Ok(web3_name) = Web3NameOf::<T, I>::try_from(web3_name_input.to_vec()) else {
			panic!();
		};
		assert!(Names::<T, I>::get(&owner).is_none());
		assert!(Owner::<T, I>::get(&web3_name).is_none());
	}

	reclaim_deposit {
		let n in (<T as Config<I>>::MinNameLength::get()) .. (<T as Config<I>>::MaxNameLength::get());
		let caller: AccountIdOf<T> = account("caller", 0, CALLER_SEED);
		let owner: Web3NameOwnerOf<T, I> = account("owner", 0, OWNER_SEED);
		let web3_name_input: BoundedVec<u8, <T as Config<I>>::MaxNameLength> = BoundedVec::try_from(<T as Config<I>>::BenchmarkHelper::generate_name_input_with_length(n.saturated_into())).expect("BoundedVec creation should not fail.");
		let web3_name_input_clone = web3_name_input.clone();
		let did_origin = <T as Config<I>>::OwnerOrigin::generate_origin(caller.clone(), owner.clone());
		let signed_origin = RawOrigin::Signed(caller.clone());

		make_free_for_did::<T, I>(&caller);
		Pallet::<T, I>::claim(did_origin, web3_name_input.clone()).expect("Should register the claimed web3 name.");
	}: _(signed_origin, web3_name_input_clone)
	verify {
		let Ok(web3_name) = Web3NameOf::<T, I>::try_from(web3_name_input.to_vec()) else {
			panic!();
		};
		assert!(Names::<T, I>::get(&owner).is_none());
		assert!(Owner::<T, I>::get(&web3_name).is_none());
	}

	ban {
		let n in (<T as Config<I>>::MinNameLength::get()) .. (<T as Config<I>>::MaxNameLength::get());
		let caller: AccountIdOf<T> = account("caller", 0, CALLER_SEED);
		let owner: Web3NameOwnerOf<T, I> = account("owner", 0, OWNER_SEED);
		let web3_name_input: BoundedVec<u8, <T as Config<I>>::MaxNameLength> = BoundedVec::try_from(<T as Config<I>>::BenchmarkHelper::generate_name_input_with_length(n.saturated_into())).expect("BoundedVec creation should not fail.");
		let web3_name_input_clone = web3_name_input.clone();
		let did_origin = <T as Config<I>>::OwnerOrigin::generate_origin(caller.clone(), owner.clone());
		let ban_origin = RawOrigin::Root;

		make_free_for_did::<T, I>(&caller);
		Pallet::<T, I>::claim(did_origin, web3_name_input.clone()).expect("Should register the claimed web3 name.");
	}: _(ban_origin, web3_name_input_clone)
	verify {
		let Ok(web3_name) = Web3NameOf::<T, I>::try_from(web3_name_input.to_vec()) else {
			panic!();
		};
		assert!(Names::<T, I>::get(&owner).is_none());
		assert!(Owner::<T, I>::get(&web3_name).is_none());
		assert!(Banned::<T, I>::get(&web3_name).is_some());
	}

	unban {
		let n in (<T as Config<I>>::MinNameLength::get()) .. (<T as Config<I>>::MaxNameLength::get());
		let caller: AccountIdOf<T> = account("caller", 0, CALLER_SEED);
		let owner: Web3NameOwnerOf<T, I> = account("owner", 0, OWNER_SEED);
		let web3_name_input: BoundedVec<u8, <T as Config<I>>::MaxNameLength> = BoundedVec::try_from(<T as Config<I>>::BenchmarkHelper::generate_name_input_with_length(n.saturated_into())).expect("BoundedVec creation should not fail.");
		let web3_name_input_clone = web3_name_input.clone();
		let ban_origin = RawOrigin::Root;

		make_free_for_did::<T, I>(&caller);
		Pallet::<T, I>::ban(ban_origin.clone().into(), web3_name_input.clone()).expect("Should ban the web3 name.");
	}: _(ban_origin, web3_name_input_clone)
	verify {
		let Ok(web3_name) = Web3NameOf::<T, I>::try_from(web3_name_input.to_vec()) else {
			panic!();
		};
		assert!(Names::<T, I>::get(&owner).is_none());
		assert!(Owner::<T, I>::get(&web3_name).is_none());
		assert!(Banned::<T, I>::get(&web3_name).is_none());
	}

	change_deposit_owner {
		let deposit_owner_old: AccountIdOf<T> = account("caller", 0, CALLER_SEED);
		let deposit_owner_new: AccountIdOf<T> = account("caller", 1, CALLER_SEED);
		let owner: Web3NameOwnerOf<T, I> = account("owner", 0, OWNER_SEED);
		let web3_name_input: BoundedVec<u8, <T as Config<I>>::MaxNameLength> = BoundedVec::try_from(
			<T as Config<I>>::BenchmarkHelper::generate_name_input_with_length(<T as Config<I>>::MaxNameLength::get().saturated_into())
		).expect("BoundedVec creation should not fail.");
		let web3_name_input_clone = web3_name_input.clone();
		let origin_create = <T as Config<I>>::OwnerOrigin::generate_origin(deposit_owner_old.clone(), owner.clone());

		make_free_for_did::<T, I>(&deposit_owner_old);
		make_free_for_did::<T, I>(&deposit_owner_new);
		Pallet::<T, I>::claim(origin_create, web3_name_input.clone()).expect("Should register the claimed web3 name.");

		let origin = <T as Config<I>>::OwnerOrigin::generate_origin(deposit_owner_new.clone(), owner);
	}: _<T::RuntimeOrigin>(origin)
	verify {
		let Ok(web3_name) = Web3NameOf::<T, I>::try_from(web3_name_input.to_vec()) else {
			panic!();
		};
		assert_eq!(Owner::<T, I>::get(&web3_name).expect("w3n should exists").deposit, Deposit {
			owner: deposit_owner_new,
			amount: <T as Config<I>>::Deposit::get(),
		});
	}

	update_deposit {
		let deposit_owner: AccountIdOf<T> = account("caller", 0, CALLER_SEED);
		let owner: Web3NameOwnerOf<T, I> = account("owner", 0, OWNER_SEED);
		let web3_name_input: BoundedVec<u8, <T as Config<I>>::MaxNameLength> = BoundedVec::try_from(
			<T as Config<I>>::BenchmarkHelper::generate_name_input_with_length(<T as Config<I>>::MaxNameLength::get().saturated_into())
		).expect("BoundedVec creation should not fail.");
		let Ok(web3_name) = Web3NameOf::<T, I>::try_from(web3_name_input.to_vec()) else {
			panic!();
		};

		make_free_for_did::<T, I>(&deposit_owner);
		insert_raw_w3n::<T, I>(
			deposit_owner.clone(),
			owner,
			web3_name.clone(),
			BlockNumberFor::<T>::zero(),
			<T as Config<I>>::Deposit::get() + <T as Config<I>>::Deposit::get()
		);

		let origin = RawOrigin::Signed(deposit_owner.clone());
	}: _(origin, web3_name_input)
	verify {
		assert_eq!(Owner::<T, I>::get(&web3_name).expect("w3n should exists").deposit, Deposit {
			owner: deposit_owner,
			amount: <T as Config<I>>::Deposit::get(),
		});
	}

	impl_benchmark_test_suite!(
		Pallet,
		crate::mock::ExtBuilder::default().build_with_keystore(),
		crate::mock::Test
	)
}
