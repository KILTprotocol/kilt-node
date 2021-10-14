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

use delegation::{benchmarking::setup_delegations, Permissions};
use frame_benchmarking::{benchmarks, impl_benchmark_test_suite};
use frame_support::traits::{Currency, Get};
use frame_system::RawOrigin;
use kilt_support::signature::VerifySignature;
use sp_core::sr25519;
use sp_runtime::traits::Hash;
use sp_std::num::NonZeroU32;

use crate::*;

const ONE_CHILD_PER_LEVEL: Option<NonZeroU32> = NonZeroU32::new(1);

benchmarks! {
	where_clause {
		where
		T: core::fmt::Debug,
		T::AccountId: From<sr25519::Public> + Into<T::DelegationEntityId>,
		T::DelegationNodeId: From<T::Hash>,
		T::CtypeCreatorId: From<T::AccountId>,
		T::DelegationEntityId: From<T::AccountId>,
		<<T as delegation::Config>::DelegationSignatureVerification as VerifySignature>::Signature: From<(
			T::AccountId,
			<<T as delegation::Config>::DelegationSignatureVerification as VerifySignature>::Payload,
		)>,
	}

	add {
		let claim_hash: T::Hash = T::Hashing::hash(b"claim");
		let ctype_hash: T::Hash = T::Hash::default();
		let (_, _, delegate_public, delegation_id) = setup_delegations::<T>(1, ONE_CHILD_PER_LEVEL.expect(">0"), Permissions::ATTEST)?;
		let delegate_acc: T::AccountId = delegate_public.into();
		<T as Config>::Currency::make_free_balance_be(&delegate_acc, <T as Config>::Deposit::get() + <T as Config>::Deposit::get());

	}: _(RawOrigin::Signed(delegate_acc.clone()), claim_hash, ctype_hash, Some(delegation_id))
	verify {
		assert!(Attestations::<T>::contains_key(claim_hash));
		assert_eq!(Pallet::<T>::attestations(claim_hash), Some(AttestationDetails {
			ctype_hash,
			attester: delegate_acc.into(),
			delegation_id: Some(delegation_id),
			revoked: false,
			deposit: kilt_support::deposit::Deposit {
				owner: delegate_public.into(),
				amount: <T as Config>::Deposit::get(),
			}
		}));
	}

	revoke {
		let d in 1 .. T::MaxParentChecks::get();

		let claim_hash: T::Hash = T::Hashing::hash(b"claim");
		let ctype_hash: T::Hash = T::Hash::default();

		let (root_public, _, delegate_public, delegation_id) = setup_delegations::<T>(d, ONE_CHILD_PER_LEVEL.expect(">0"), Permissions::ATTEST | Permissions::DELEGATE)?;
		let root_acc: T::AccountId = root_public.into();
		let delegate_acc: T::AccountId = delegate_public.into();
		<T as Config>::Currency::make_free_balance_be(&delegate_acc, <T as Config>::Deposit::get() + <T as Config>::Deposit::get());

		// attest with leaf account
		Pallet::<T>::add(RawOrigin::Signed(delegate_acc.clone()).into(), claim_hash, ctype_hash, Some(delegation_id))?;
		// revoke with root account, s.t. delegation tree needs to be traversed
	}: _(RawOrigin::Signed(root_acc.clone()), claim_hash, d)
	verify {
		assert!(Attestations::<T>::contains_key(claim_hash));
		assert_eq!(Attestations::<T>::get(claim_hash), Some(AttestationDetails {
			ctype_hash,
			attester: delegate_acc.into(),
			delegation_id: Some(delegation_id),
			revoked: true,
			deposit: kilt_support::deposit::Deposit {
				owner: delegate_public.into(),
				amount: <T as Config>::Deposit::get(),
			}
		}));
	}

	remove {
		let d in 1 .. T::MaxParentChecks::get();

		let claim_hash: T::Hash = T::Hashing::hash(b"claim");
		let ctype_hash: T::Hash = T::Hash::default();

		let (root_public, _, delegate_public, delegation_id) = setup_delegations::<T>(d, ONE_CHILD_PER_LEVEL.expect(">0"), Permissions::ATTEST | Permissions::DELEGATE)?;
		let root_acc: T::AccountId = root_public.into();
		let delegate_acc: T::AccountId = delegate_public.into();
		<T as Config>::Currency::make_free_balance_be(&delegate_acc, <T as Config>::Deposit::get() + <T as Config>::Deposit::get());

		// attest with leaf account
		Pallet::<T>::add(RawOrigin::Signed(delegate_acc).into(), claim_hash, ctype_hash, Some(delegation_id))?;
		// revoke with root account, s.t. delegation tree needs to be traversed
	}: _(RawOrigin::Signed(root_acc.clone()), claim_hash, d)
	verify {
		assert!(!Attestations::<T>::contains_key(claim_hash));
	}

	reclaim_deposit {
		let claim_hash: T::Hash = T::Hashing::hash(b"claim");
		let ctype_hash: T::Hash = T::Hash::default();

		let (root_public, _, delegate_public, delegation_id) = setup_delegations::<T>(1, ONE_CHILD_PER_LEVEL.expect(">0"), Permissions::ATTEST | Permissions::DELEGATE)?;
		let root_acc: T::AccountId = root_public.into();
		let delegate_acc: T::AccountId = delegate_public.into();
		<T as Config>::Currency::make_free_balance_be(&delegate_acc, <T as Config>::Deposit::get() + <T as Config>::Deposit::get());

		// attest with leaf account
		Pallet::<T>::add(RawOrigin::Signed(delegate_acc.clone()).into(), claim_hash, ctype_hash, Some(delegation_id))?;
		// revoke with root account, s.t. delegation tree needs to be traversed
	}: _(RawOrigin::Signed(delegate_acc.clone()), claim_hash)
	verify {
		assert!(!Attestations::<T>::contains_key(claim_hash));
	}
}

impl_benchmark_test_suite! {
	Pallet,
	crate::mock::ExtBuilder::default().build_with_keystore(),
	crate::mock::Test
}
