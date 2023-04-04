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

//! Benchmarking

use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite};
use frame_support::{
	crypto::ecdsa::ECDSAExt,
	traits::{Currency, Get},
};
use frame_system::RawOrigin;
use sha3::{Digest, Keccak256};
use sp_io::crypto::{ecdsa_generate, ed25519_generate, sr25519_generate};
use sp_runtime::{
	app_crypto::{ed25519, sr25519},
	traits::IdentifyAccount,
	AccountId32, KeyTypeId,
};

use kilt_support::{deposit::Deposit, traits::GenerateBenchmarkOrigin};

use crate::{
	account::AccountId20,
	associate_account_request::{get_challenge, AssociateAccountRequest},
	linkable_account::LinkableAccountId,
	signature::get_wrapped_payload,
	AccountIdOf, Call, Config, ConnectedAccounts, ConnectedDids, CurrencyOf, Pallet,
};

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
		T::AccountId: From<sr25519::Public> + From<ed25519::Public> + Into<LinkableAccountId> + Into<AccountId32> + From<sp_runtime::AccountId32>,
		T::DidIdentifier: From<T::AccountId>,
		T::EnsureOrigin: GenerateBenchmarkOrigin<T::RuntimeOrigin, T::AccountId, T::DidIdentifier>,
	}

	associate_account_multisig_sr25519 {
		let caller: T::AccountId = account("caller", 0, SEED);
		let did: T::DidIdentifier = account("did", 0, SEED);
		let previous_did: T::DidIdentifier = account("prev", 0, SEED + 1);
		let connected_acc = sr25519_generate(KeyTypeId(*b"aura"), None);
		let connected_acc_id: T::AccountId = connected_acc.into();
		let linkable_id: LinkableAccountId = connected_acc_id.clone().into();
		let expire_at: <T as frame_system::Config>::BlockNumber = 500_u32.into();

		let sig = sp_io::crypto::sr25519_sign(
			KeyTypeId(*b"aura"),
			&connected_acc,
			&get_wrapped_payload(
				get_challenge(&did, expire_at).as_bytes(),
				crate::signature::WrapType::Substrate,
			))
			.ok_or("Error while building signature.")?;

		make_free_for_did::<T>(&caller);

		// Add existing connected_acc -> previous_did connection that will be replaced
		Pallet::<T>::add_association(caller.clone(), previous_did.clone(), linkable_id.clone()).expect("should create previous association");
		assert!(ConnectedAccounts::<T>::get(&previous_did, linkable_id.clone()).is_some());
		let origin = T::EnsureOrigin::generate_origin(caller, did.clone());
		let id_arg = linkable_id.clone();
		let req = AssociateAccountRequest::Polkadot(connected_acc_id.into(), sig.into());
	}: associate_account<T::RuntimeOrigin>(origin, req, expire_at)
	verify {
		assert!(ConnectedDids::<T>::get(linkable_id.clone()).is_some());
		assert!(ConnectedAccounts::<T>::get(&previous_did, linkable_id.clone()).is_none());
		assert!(ConnectedAccounts::<T>::get(did, linkable_id).is_some());
	}

	associate_account_multisig_ed25519 {
		let caller: T::AccountId = account("caller", 0, SEED);
		let did: T::DidIdentifier = account("did", 0, SEED);
		let previous_did: T::DidIdentifier = account("prev", 0, SEED + 1);
		let connected_acc = ed25519_generate(KeyTypeId(*b"aura"), None);
		let connected_acc_id: T::AccountId = connected_acc.into();
		let linkable_id: LinkableAccountId = connected_acc_id.clone().into();
		let expire_at: <T as frame_system::Config>::BlockNumber = 500_u32.into();

		let sig = sp_io::crypto::ed25519_sign(
			KeyTypeId(*b"aura"),
			&connected_acc,
			&get_wrapped_payload(
				get_challenge(&did, expire_at).as_bytes(),
				crate::signature::WrapType::Substrate,
			))
			.ok_or("Error while building signature.")?;

		make_free_for_did::<T>(&caller);

		// Add existing connected_acc -> previous_did connection that will be replaced
		Pallet::<T>::add_association(caller.clone(), previous_did.clone(), linkable_id.clone()).expect("should create previous association");
		assert!(ConnectedAccounts::<T>::get(&previous_did, linkable_id.clone()).is_some());
		let origin = T::EnsureOrigin::generate_origin(caller, did.clone());
		let id_arg = linkable_id.clone();
		let req = AssociateAccountRequest::Polkadot(connected_acc_id.into(), sig.into());
	}: associate_account<T::RuntimeOrigin>(origin, req, expire_at)
	verify {
		assert!(ConnectedDids::<T>::get(linkable_id.clone()).is_some());
		assert!(ConnectedAccounts::<T>::get(&previous_did, linkable_id.clone()).is_none());
		assert!(ConnectedAccounts::<T>::get(did, linkable_id).is_some());
	}

	associate_account_multisig_ecdsa {
		let caller: T::AccountId = account("caller", 0, SEED);
		let did: T::DidIdentifier = account("did", 0, SEED);
		let previous_did: T::DidIdentifier = account("prev", 0, SEED + 1);
		let connected_acc = ecdsa_generate(KeyTypeId(*b"aura"), None);
		let connected_acc_id = sp_runtime::MultiSigner::from(connected_acc).into_account();
		let linkable_id: LinkableAccountId = connected_acc_id.clone().into();
		let expire_at: <T as frame_system::Config>::BlockNumber = 500_u32.into();

		let sig = sp_io::crypto::ecdsa_sign(
			KeyTypeId(*b"aura"),
			&connected_acc,
			&get_wrapped_payload(
				get_challenge(&did, expire_at).as_bytes(),
				crate::signature::WrapType::Substrate,
			))
			.ok_or("Error while building signature.")?;

		make_free_for_did::<T>(&caller);

		// Add existing connected_acc -> previous_did connection that will be replaced
		Pallet::<T>::add_association(caller.clone(), previous_did.clone(), linkable_id.clone()).expect("should create previous association");
		assert!(ConnectedAccounts::<T>::get(&previous_did, linkable_id.clone()).is_some());
		let origin = T::EnsureOrigin::generate_origin(caller, did.clone());
		let id_arg = linkable_id.clone();
		let req = AssociateAccountRequest::Polkadot(connected_acc_id, sig.into());
	}: associate_account<T::RuntimeOrigin>(origin, req, expire_at)
	verify {
		assert!(ConnectedDids::<T>::get(linkable_id.clone()).is_some());
		assert!(ConnectedAccounts::<T>::get(&previous_did, linkable_id.clone()).is_none());
		assert!(ConnectedAccounts::<T>::get(did, linkable_id).is_some());
	}

	associate_eth_account {
		let caller: T::AccountId = account("caller", 0, SEED);
		let did: T::DidIdentifier = account("did", 0, SEED);
		let previous_did: T::DidIdentifier = account("prev", 0, SEED + 1);
		let expire_at: <T as frame_system::Config>::BlockNumber = 500_u32.into();

		let eth_public_key = ecdsa_generate(KeyTypeId(*b"aura"), None);
		let eth_account = AccountId20(eth_public_key.to_eth_address().unwrap());

		let wrapped_payload = get_wrapped_payload(
			get_challenge(&did, expire_at).as_bytes(),
			crate::signature::WrapType::Ethereum,
		);

		let sig = sp_io::crypto::ecdsa_sign_prehashed(
			KeyTypeId(*b"aura"),
			&eth_public_key,
			&Keccak256::digest(wrapped_payload).try_into().unwrap(),
		).ok_or("Error while building signature.")?;

		make_free_for_did::<T>(&caller);

		// Add existing connected_acc -> previous_did connection that will be replaced
		Pallet::<T>::add_association(caller.clone(), previous_did.clone(), eth_account.into()).expect("should create previous association");
		assert!(ConnectedAccounts::<T>::get(&previous_did, LinkableAccountId::from(eth_account)).is_some());
		let origin = T::EnsureOrigin::generate_origin(caller, did.clone());
		let req = AssociateAccountRequest::Ethereum(eth_account, sig.into());
	}: associate_account<T::RuntimeOrigin>(origin, req, expire_at)
	verify {
		assert!(ConnectedDids::<T>::get(LinkableAccountId::from(eth_account)).is_some());
		assert!(ConnectedAccounts::<T>::get(&previous_did, LinkableAccountId::from(eth_account)).is_none());
		assert!(ConnectedAccounts::<T>::get(did, LinkableAccountId::from(eth_account)).is_some());
	}

	associate_sender {
		let caller: T::AccountId = account("caller", 0, SEED);
		let linkable_id: LinkableAccountId = caller.clone().into();
		let did: T::DidIdentifier = account("did", 0, SEED);
		let previous_did: T::DidIdentifier = account("prev", 0, SEED + 1);

		make_free_for_did::<T>(&caller);

		// Add existing sender -> previous_did connection that will be replaced
		Pallet::<T>::add_association(caller.clone(), previous_did.clone(), caller.clone().into()).expect("should create previous association");
		assert!(ConnectedAccounts::<T>::get(&previous_did, &linkable_id).is_some());
		let origin = T::EnsureOrigin::generate_origin(caller, did.clone());
	}: _<T::RuntimeOrigin>(origin)
	verify {
		assert!(ConnectedDids::<T>::get(&linkable_id).is_some());
		assert!(ConnectedAccounts::<T>::get(previous_did, &linkable_id).is_none());
		assert!(ConnectedAccounts::<T>::get(did, linkable_id).is_some());
	}

	remove_sender_association {
		let caller: T::AccountId = account("caller", 0, SEED);
		let linkable_id: LinkableAccountId = caller.clone().into();
		let did: T::DidIdentifier = account("did", 0, SEED);

		make_free_for_did::<T>(&caller);
		Pallet::<T>::add_association(caller.clone(), did.clone(), linkable_id.clone()).expect("should create association");

		let origin = RawOrigin::Signed(caller);
	}: _(origin)
	verify {
		assert!(ConnectedDids::<T>::get(&linkable_id).is_none());
		assert!(ConnectedAccounts::<T>::get(did, linkable_id).is_none());
	}

	remove_account_association {
		let caller: T::AccountId = account("caller", 0, SEED);
		let linkable_id: LinkableAccountId = caller.clone().into();
		let did: T::DidIdentifier = account("did", 0, SEED);
		make_free_for_did::<T>(&caller);

		Pallet::<T>::add_association(caller.clone(), did.clone(), linkable_id.clone()).expect("should create association");

		let origin = T::EnsureOrigin::generate_origin(caller, did.clone());
		let id_arg = linkable_id.clone();
	}: _<T::RuntimeOrigin>(origin, id_arg)
	verify {
		assert!(ConnectedDids::<T>::get(&linkable_id).is_none());
		assert!(ConnectedAccounts::<T>::get(did, linkable_id).is_none());
	}

	change_deposit_owner {
		let deposit_owner_old: T::AccountId = account("caller", 0, SEED);
		let deposit_owner_new: T::AccountId = account("caller", 1, SEED);
		let linkable_id: LinkableAccountId = deposit_owner_old.clone().into();
		let did: T::DidIdentifier = account("did", 0, SEED);
		make_free_for_did::<T>(&deposit_owner_old);
		make_free_for_did::<T>(&deposit_owner_new);

		Pallet::<T>::add_association(deposit_owner_old, did.clone(), linkable_id.clone()).expect("should create association");

		let origin = T::EnsureOrigin::generate_origin(deposit_owner_new.clone(), did);
		let id_arg = linkable_id.clone();
	}: _<T::RuntimeOrigin>(origin, id_arg)
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
		let linkable_id: LinkableAccountId = deposit_owner.clone().into();
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
