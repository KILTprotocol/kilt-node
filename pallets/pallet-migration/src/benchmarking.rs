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
use frame_support::{
	traits::{
		fungible::{Inspect, InspectFreeze, InspectHold, Mutate},
		Get, LockableCurrency, ReservableCurrency, WithdrawReasons,
	},
	BoundedVec,
};
use frame_system::RawOrigin;
use sp_runtime::{traits::IdentifyAccount, AccountId32, MultiSigner, SaturatedConversion};
use sp_std::vec;

use did::{
	benchmarking::get_ed25519_public_authentication_key, did_details::DidVerificationKey,
	mock_utils::generate_base_did_details, DidIdentifierOf,
};
use parachain_staking::migrations::STAKING_ID;
use runtime_common::constants::KILT;

use crate::*;

const SEED: u32 = 0;
type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

benchmarks! {
	where_clause {
		where
		<T as attestation::Config>::Currency: ReservableCurrency<AccountIdOf<T>>,
		<T as delegation::Config>::Currency: ReservableCurrency<AccountIdOf<T>>,
		<T as did::Config>::Currency: ReservableCurrency<AccountIdOf<T>>,
		<T as did::Config>::Currency: Mutate<AccountIdOf<T>>,
		<T as did::Config>::Currency: Inspect<AccountIdOf<T>>,
		<T as pallet_did_lookup::Config>::Currency: ReservableCurrency<AccountIdOf<T>>,
		<T as pallet_web3_names::Config>::Currency: ReservableCurrency<AccountIdOf<T>>,
		<T as parachain_staking::Config>::Currency: ReservableCurrency<AccountIdOf<T>>,
		<T as parachain_staking::Config>::Currency: LockableCurrency<AccountIdOf<T>>,
		<T as public_credentials::Config>::Currency: ReservableCurrency<AccountIdOf<T>>,
		<T as did::Config>::DidIdentifier: From<AccountId32>,
		<T as frame_system::Config>::AccountId: From<AccountId32>,
	}
	general_weight {
		let caller : AccountIdOf<T> = account("caller", 0, SEED);
		//create only one did. The migration for did, delegation, attestation, w3n, public credentials, and did lookup is the same.
		let did_public_auth_key = get_ed25519_public_authentication_key();
		let did_subject: DidIdentifierOf<T> = MultiSigner::from(did_public_auth_key).into_account().into();
		let mut did_details = generate_base_did_details::<T>(DidVerificationKey::from(did_public_auth_key), Some(caller.clone()));
		did_details.deposit.version = None;
		did_details.deposit.amount = <T as did::Config>::BaseDeposit::get();

		did::Did::<T>::insert(did_subject.clone(), did_details.clone());
		let initial_balance =  did_details.deposit.amount.saturated_into::<u128>().saturating_mul(2);
		<<T as did::Config>::Currency as Mutate<AccountIdOf<T>>>::set_balance(&caller, initial_balance.saturated_into());

		<<T as did::Config>::Currency as ReservableCurrency<AccountIdOf<T>>>::reserve(&caller, did_details.deposit.amount.saturated_into::<u128>().saturated_into())
			.expect("User should have enough balance");

		let entries_to_upgrade = EntriesToMigrate {
			attestation: BoundedVec::try_from(vec![]).expect("Vector initialization should not fail."),
			delegation: BoundedVec::try_from(vec![]).expect("Vector initialization should not fail."),
			did: BoundedVec::try_from(vec![did_subject.clone()]).expect("Vector initialization should not fail."),
			lookup: BoundedVec::try_from(vec![]).expect("Vector initialization should not fail."),
			w3n: BoundedVec::try_from(vec![]).expect("Vector initialization should not fail."),
			staking: BoundedVec::try_from(vec![]).expect("Vector initialization should not fail."),
			public_credentials: BoundedVec::try_from(vec![]).expect("Vector initialization should not fail."),
		};

		let origin= RawOrigin::Signed(caller.clone());

	}: update_balance(origin, entries_to_upgrade)
	verify {
		let details = did::Did::<T>::get(did_subject);
		let hold_balance = <<T as did::Config>::Currency as  InspectHold<AccountIdOf<T>>>::balance_on_hold(&did::HoldReason::Deposit.into(), &caller);
		assert_eq!(hold_balance, did_details.deposit.amount);
		assert!(details.is_some());
		assert!(details.unwrap().deposit.version.is_none())
	}

	staking_weight {
		let caller : AccountIdOf<T> = account("caller", 0, SEED);
		let initial_lock = KILT.saturating_div(2);

		<<T as did::Config>::Currency as Mutate<AccountIdOf<T>>>::set_balance(&caller, KILT.saturated_into());
		<<T as parachain_staking::Config>::Currency as LockableCurrency<AccountIdOf<T>>>::set_lock(STAKING_ID, &caller, initial_lock.saturated_into(), WithdrawReasons::all());


		let entries_to_upgrade = EntriesToMigrate {
			attestation: BoundedVec::try_from(vec![]).expect("Vector initialization should not fail."),
			delegation: BoundedVec::try_from(vec![]).expect("Vector initialization should not fail."),
			did: BoundedVec::try_from(vec![]).expect("Vector initialization should not fail."),
			lookup: BoundedVec::try_from(vec![]).expect("Vector initialization should not fail."),
			w3n: BoundedVec::try_from(vec![]).expect("Vector initialization should not fail."),
			staking: BoundedVec::try_from(vec![caller.clone()]).expect("Vector initialization should not fail."),
			public_credentials: BoundedVec::try_from(vec![]).expect("Vector initialization should not fail."),
		};

		let origin= RawOrigin::Signed(caller.clone());
	}: update_balance(origin, entries_to_upgrade)
	verify {
		let freezed_balance = <<T as parachain_staking::Config>::Currency as InspectFreeze<AccountIdOf<T>>>::balance_frozen(&parachain_staking::FreezeReason::Staking.into(), &caller);
		assert_eq!(freezed_balance.saturated_into::<u128>(), initial_lock)
	}
}
