// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2021 BOTLabs GmbH

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

use crate::{Call, Config, ConnectedDids, Pallet};

use codec::Encode;
use frame_benchmarking::{account, benchmarks};
use frame_system::RawOrigin;
use sp_io::crypto::sr25519_generate;
use sp_runtime::{app_crypto::sr25519, KeyTypeId};

const SEED: u32 = 0;

benchmarks! {
	where_clause {
		where
		T::AccountId: From<sr25519::Public>,
		T::DidAccount: From<T::AccountId>,
		T::Signature: From<sr25519::Signature>,
		T::Signer: Default,
	}

	associate_account {
		let caller: T::AccountId = account("caller", 0, SEED);
		let connected_acc = sr25519_generate(KeyTypeId(*b"aura"), None);
		let connected_acc_id: T::AccountId = connected_acc.clone().into();

		let sig: T::Signature = sp_io::crypto::sr25519_sign(KeyTypeId(*b"aura"), &connected_acc, &Encode::encode(&caller)[..])
			.ok_or("Error while building signature.")?
			.into();

		let origin = RawOrigin::Signed(caller.clone());
	}: _(origin, connected_acc_id, sig)
	verify {
		assert!(ConnectedDids::<T>::get(T::AccountId::from(connected_acc)).is_some());
	}

	associate_sender {
		let caller: T::AccountId = account("caller", 0, SEED);

		let origin = RawOrigin::Signed(caller.clone());
	}: _(origin)
	verify {
		assert!(ConnectedDids::<T>::get(caller).is_some());
	}

	remove_sender_association {
		let caller: T::AccountId = account("caller", 0, SEED);
		ConnectedDids::<T>::insert(&caller, T::DidAccount::from(caller.clone()));

		let origin = RawOrigin::Signed(caller.clone());
	}: _(origin)
	verify {
		assert!(ConnectedDids::<T>::get(caller).is_none());
	}

	remove_account_association {
		let caller: T::AccountId = account("caller", 0, SEED);
		ConnectedDids::<T>::insert(&caller, T::DidAccount::from(caller.clone()));

		let origin = RawOrigin::Signed(caller.clone());
	}: _(origin, caller.clone())
	verify {
		assert!(ConnectedDids::<T>::get(caller).is_none());
	}
}

// TODO: add benchmark tests
