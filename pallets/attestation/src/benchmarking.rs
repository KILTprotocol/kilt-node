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

use super::*;

use crate::Pallet as AttestationModule;
use delegation::{benchmarking::setup_delegations, Permissions};
use frame_benchmarking::benchmarks;
use frame_support::storage::StorageMap;
use frame_system::RawOrigin;
use sp_core::sr25519;
use sp_runtime::traits::Hash;
use sp_std::num::NonZeroU32;

const MAX_DEPTH: u32 = 10;
const ONE_CHILD_PER_LEVEL: Option<NonZeroU32> = NonZeroU32::new(1);

benchmarks! {
	where_clause { where T: core::fmt::Debug, T::Signature: From<sr25519::Signature>, T::AccountId: From<sr25519::Public>, 	T::DelegationNodeId: From<T::Hash> }

	add {
		let claim_hash: T::Hash = T::Hashing::hash(b"claim");
		let ctype_hash: T::Hash = T::Hash::default();
		let (_, _, delegate_public, delegation_id) = setup_delegations::<T>(1, ONE_CHILD_PER_LEVEL.expect(">0"), Permissions::ATTEST)?;
		let delegate_acc: T::AccountId = delegate_public.into();
	}: _(RawOrigin::Signed(delegate_acc.clone()), claim_hash, ctype_hash, Some(delegation_id))
	verify {
		assert!(Attestations::<T>::contains_key(claim_hash));
		assert_eq!(AttestationModule::<T>::attestations(claim_hash), Some(Attestation::<T> {
			ctype_hash,
			attester: delegate_acc,
			delegation_id: Some(delegation_id),
			revoked: false,
		}));
	}

	revoke {
		let d in 1 .. MAX_DEPTH;

		let claim_hash: T::Hash = T::Hashing::hash(b"claim");
		let ctype_hash: T::Hash = T::Hash::default();

		let (root_public, _, delegate_public, delegation_id) = setup_delegations::<T>(d, ONE_CHILD_PER_LEVEL.expect(">0"), Permissions::ATTEST | Permissions::DELEGATE)?;
		let root_acc: T::AccountId = root_public.into();
		let delegate_acc: T::AccountId = delegate_public.into();

		// attest with leaf account
		AttestationModule::<T>::add(RawOrigin::Signed(delegate_acc.clone()).into(), claim_hash, ctype_hash, Some(delegation_id))?;
		// revoke with root account, s.t. delegation tree needs to be traversed
	}: _(RawOrigin::Signed(root_acc.clone()), claim_hash, d + 1)
	verify {
		assert!(Attestations::<T>::contains_key(claim_hash));
		assert_eq!(Attestations::<T>::get(claim_hash), Some(Attestation::<T> {
			ctype_hash,
			attester: delegate_acc,
			delegation_id: Some(delegation_id),
			revoked: true,
		}));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::tests::{ExtBuilder, Test};
	use frame_support::assert_ok;

	#[test]
	fn test_benchmarks() {
		ExtBuilder::build_with_keystore().execute_with(|| {
			assert_ok!(test_benchmark_add::<Test>());
			assert_ok!(test_benchmark_revoke::<Test>());
		});
	}
}
