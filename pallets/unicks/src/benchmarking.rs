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

use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite};
use frame_support::{
	sp_runtime::SaturatedConversion,
	traits::{Currency, Get},
	BoundedVec,
};
use frame_system::RawOrigin;
use sp_runtime::app_crypto::sr25519;

use kilt_support::traits::GenerateBenchmarkOrigin;

use crate::*;

const CALLER_SEED: u32 = 0;
const OWNER_SEED: u32 = 1;

fn make_free_for_did<T: Config>(account: &AccountIdOf<T>) {
	let balance = <CurrencyOf<T> as Currency<AccountIdOf<T>>>::minimum_balance() + <T as Config>::Deposit::get();
	<CurrencyOf<T> as Currency<AccountIdOf<T>>>::make_free_balance_be(account, balance);
}

benchmarks! {
	where_clause {
		where
		T::AccountId: From<sr25519::Public>,
		T::UnickOwner: From<T::AccountId>,
		T::RegularOrigin: GenerateBenchmarkOrigin<T::Origin, T::AccountId, T::UnickOwner>,
		T::BlacklistOrigin: GenerateBenchmarkOrigin<T::Origin, T::AccountId, T::UnickOwner>,
	}

	claim {
		let caller: AccountIdOf<T> = account("caller", 0, CALLER_SEED);
		let owner: UnickOwnerOf<T> = account("owner", 0, OWNER_SEED);
		let unick_input = BoundedVec::<u8,T::MaxUnickLength>::try_from(vec![b'1'; T::MaxUnickLength::get().saturated_into()]).unwrap();
		let unick_input_clone = unick_input.clone();
		let origin = T::RegularOrigin::generate_origin(caller.clone(), owner.clone());

		make_free_for_did::<T>(&caller);
	}: _<T::Origin>(origin, unick_input_clone)
	verify {
		assert!(Unicks::<T>::get(&owner).is_some());
		assert!(Owner::<T>::get(&UnickOf::<T>::try_from(unick_input.to_vec()).unwrap()).is_some());
	}

	release_by_owner {
		let caller: AccountIdOf<T> = account("caller", 0, CALLER_SEED);
		let owner: UnickOwnerOf<T> = account("owner", 0, OWNER_SEED);
		let unick_input = BoundedVec::<u8,T::MaxUnickLength>::try_from(vec![b'1'; T::MaxUnickLength::get().saturated_into()]).unwrap();
		let unick_input_clone = unick_input.clone();
		let origin = T::RegularOrigin::generate_origin(caller.clone(), owner.clone());

		make_free_for_did::<T>(&caller);
		Pallet::<T>::claim(origin.clone(), unick_input.clone()).expect("Should register the claimed unick.");
	}: _<T::Origin>(origin, unick_input_clone)
	verify {
		assert!(Unicks::<T>::get(&owner).is_none());
		assert!(Owner::<T>::get(&UnickOf::<T>::try_from(unick_input.to_vec()).unwrap()).is_none());
	}

	release_by_payer {
		let caller: AccountIdOf<T> = account("caller", 0, CALLER_SEED);
		let owner: UnickOwnerOf<T> = account("owner", 0, OWNER_SEED);
		let unick_input = BoundedVec::<u8,T::MaxUnickLength>::try_from(vec![b'1'; T::MaxUnickLength::get().saturated_into()]).unwrap();
		let unick_input_clone = unick_input.clone();
		let did_origin = T::RegularOrigin::generate_origin(caller.clone(), owner.clone());
		let signed_origin = RawOrigin::Signed(caller.clone());

		make_free_for_did::<T>(&caller);
		Pallet::<T>::claim(did_origin, unick_input.clone()).expect("Should register the claimed unick.");
	}: _(signed_origin, unick_input_clone)
	verify {
		assert!(Unicks::<T>::get(&owner).is_none());
		assert!(Owner::<T>::get(&UnickOf::<T>::try_from(unick_input.to_vec()).unwrap()).is_none());
	}

	blacklist {
		let caller: AccountIdOf<T> = account("caller", 0, CALLER_SEED);
		let owner: UnickOwnerOf<T> = account("owner", 0, OWNER_SEED);
		let unick_input = BoundedVec::<u8,T::MaxUnickLength>::try_from(vec![b'1'; T::MaxUnickLength::get().saturated_into()]).unwrap();
		let unick_input_clone = unick_input.clone();
		let did_origin = T::RegularOrigin::generate_origin(caller.clone(), owner.clone());
		let root_origin = RawOrigin::Root;

		make_free_for_did::<T>(&caller);
		Pallet::<T>::claim(did_origin, unick_input.clone()).expect("Should register the claimed unick.");
	}: _(root_origin, unick_input_clone)
	verify {
		let unick = UnickOf::<T>::try_from(unick_input.to_vec()).unwrap();
		assert!(Unicks::<T>::get(&owner).is_none());
		assert!(Owner::<T>::get(&unick).is_none());
		assert!(Blacklist::<T>::get(&unick).is_some());
	}

	unblacklist {
		let caller: AccountIdOf<T> = account("caller", 0, CALLER_SEED);
		let owner: UnickOwnerOf<T> = account("owner", 0, OWNER_SEED);
		let unick_input = BoundedVec::<u8,T::MaxUnickLength>::try_from(vec![b'1'; T::MaxUnickLength::get().saturated_into()]).unwrap();
		let unick_input_clone = unick_input.clone();
		let root_origin = T::RegularOrigin::generate_origin(caller.clone(), owner.clone());

		make_free_for_did::<T>(&caller);
		Pallet::<T>::blacklist(root_origin.clone(), unick_input.clone()).expect("Should blacklist the unick.");
	}: _<T::Origin>(root_origin, unick_input_clone)
	verify {
		let unick = UnickOf::<T>::try_from(unick_input.to_vec()).unwrap();
		assert!(Unicks::<T>::get(&owner).is_none());
		assert!(Owner::<T>::get(&unick).is_none());
		assert!(Blacklist::<T>::get(&unick).is_some());
	}
}

impl_benchmark_test_suite! {
	Pallet,
	crate::mock::ExtBuilder::default().build_with_keystore(),
	crate::mock::Test
}
