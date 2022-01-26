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

use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite, vec};
use frame_support::{
	pallet_prelude::EnsureOrigin,
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

fn generate_unick_input(length: usize) -> Vec<u8> {
	vec![b'1'; length]
}

benchmarks! {
	where_clause {
		where
		T::AccountId: From<sr25519::Public>,
		T::UnickOwner: From<T::AccountId>,
		T::RegularOrigin: GenerateBenchmarkOrigin<T::Origin, T::AccountId, T::UnickOwner>,
		T::BlacklistOrigin: EnsureOrigin<T::Origin>,
	}

	claim {
		let n in (T::MinUnickLength::get()) .. (T::MaxUnickLength::get());
		let caller: AccountIdOf<T> = account("caller", 0, CALLER_SEED);
		let owner: UnickOwnerOf<T> = account("owner", 0, OWNER_SEED);
		let unick_input: BoundedVec<u8, T::MaxUnickLength> = BoundedVec::try_from(generate_unick_input(n.saturated_into())).expect("BoundedVec creation should not fail.");
		let unick_input_clone = unick_input.clone();
		let origin = T::RegularOrigin::generate_origin(caller.clone(), owner.clone());

		make_free_for_did::<T>(&caller);
	}: _<T::Origin>(origin, unick_input_clone)
	verify {
		let unick = UnickOf::<T>::try_from(unick_input.to_vec()).unwrap();
		assert!(Unicks::<T>::get(&owner).is_some());
		assert!(Owner::<T>::get(&unick).is_some());
	}

	release_by_owner {
		let n in (T::MinUnickLength::get()) .. (T::MaxUnickLength::get());
		let caller: AccountIdOf<T> = account("caller", 0, CALLER_SEED);
		let owner: UnickOwnerOf<T> = account("owner", 0, OWNER_SEED);
		let unick_input: BoundedVec<u8, T::MaxUnickLength> = BoundedVec::try_from(generate_unick_input(n.saturated_into())).expect("BoundedVec creation should not fail.");
		let unick_input_clone = unick_input.clone();
		let origin = T::RegularOrigin::generate_origin(caller.clone(), owner.clone());

		make_free_for_did::<T>(&caller);
		Pallet::<T>::claim(origin.clone(), unick_input.clone()).expect("Should register the claimed unick.");
	}: _<T::Origin>(origin, unick_input_clone)
	verify {
		let unick = UnickOf::<T>::try_from(unick_input.to_vec()).unwrap();
		assert!(Unicks::<T>::get(&owner).is_none());
		assert!(Owner::<T>::get(&unick).is_none());
	}

	release_by_payer {
		let n in (T::MinUnickLength::get()) .. (T::MaxUnickLength::get());
		let caller: AccountIdOf<T> = account("caller", 0, CALLER_SEED);
		let owner: UnickOwnerOf<T> = account("owner", 0, OWNER_SEED);
		let unick_input: BoundedVec<u8, T::MaxUnickLength> = BoundedVec::try_from(generate_unick_input(n.saturated_into())).expect("BoundedVec creation should not fail.");
		let unick_input_clone = unick_input.clone();
		let did_origin = T::RegularOrigin::generate_origin(caller.clone(), owner.clone());
		let signed_origin = RawOrigin::Signed(caller.clone());

		make_free_for_did::<T>(&caller);
		Pallet::<T>::claim(did_origin, unick_input.clone()).expect("Should register the claimed unick.");
	}: _(signed_origin, unick_input_clone)
	verify {
		let unick = UnickOf::<T>::try_from(unick_input.to_vec()).unwrap();
		assert!(Unicks::<T>::get(&owner).is_none());
		assert!(Owner::<T>::get(&unick).is_none());
	}

	blacklist {
		let n in (T::MinUnickLength::get()) .. (T::MaxUnickLength::get());
		let caller: AccountIdOf<T> = account("caller", 0, CALLER_SEED);
		let owner: UnickOwnerOf<T> = account("owner", 0, OWNER_SEED);
		let unick_input: BoundedVec<u8, T::MaxUnickLength> = BoundedVec::try_from(generate_unick_input(n.saturated_into())).expect("BoundedVec creation should not fail.");
		let unick_input_clone = unick_input.clone();
		let did_origin = T::RegularOrigin::generate_origin(caller.clone(), owner.clone());
		let blacklist_origin = RawOrigin::Root;

		make_free_for_did::<T>(&caller);
		Pallet::<T>::claim(did_origin, unick_input.clone()).expect("Should register the claimed unick.");
	}: _(blacklist_origin, unick_input_clone)
	verify {
		let unick = UnickOf::<T>::try_from(unick_input.to_vec()).unwrap();
		assert!(Unicks::<T>::get(&owner).is_none());
		assert!(Owner::<T>::get(&unick).is_none());
		assert!(Blacklist::<T>::get(&unick).is_some());
	}

	unblacklist {
		let n in (T::MinUnickLength::get()) .. (T::MaxUnickLength::get());
		let caller: AccountIdOf<T> = account("caller", 0, CALLER_SEED);
		let owner: UnickOwnerOf<T> = account("owner", 0, OWNER_SEED);
		let unick_input: BoundedVec<u8, T::MaxUnickLength> = BoundedVec::try_from(generate_unick_input(n.saturated_into())).expect("BoundedVec creation should not fail.");
		let unick_input_clone = unick_input.clone();
		let blacklist_origin = RawOrigin::Root;

		make_free_for_did::<T>(&caller);
		Pallet::<T>::blacklist(blacklist_origin.clone().into(), unick_input.clone()).expect("Should blacklist the unick.");
	}: _(blacklist_origin, unick_input_clone)
	verify {
		let unick = UnickOf::<T>::try_from(unick_input.to_vec()).unwrap();
		assert!(Unicks::<T>::get(&owner).is_none());
		assert!(Owner::<T>::get(&unick).is_none());
		assert!(Blacklist::<T>::get(&unick).is_none());
	}
}

impl_benchmark_test_suite! {
	Pallet,
	crate::mock::ExtBuilder::default().build_with_keystore(),
	crate::mock::Test
}
