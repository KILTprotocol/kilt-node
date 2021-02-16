// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019  BOTLabs GmbH

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

use crate::Module as AttestationModule;
use delegation::{Delegations, Module as Delegation};
use frame_benchmarking::benchmarks;
use frame_support::storage::StorageMap;
use frame_system::RawOrigin;
use sp_core::{offchain::KeyTypeId, sr25519, Pair};
use sp_io::crypto::sr25519_generate;
use sp_runtime::traits::Hash;
use sp_std::{boxed::Box, vec};

// const MAX_DEPTH: u32 = 100;
// const MAX_CHILDREN: u32 = 4;

benchmarks! {
	where_clause { where T: core::fmt::Debug, T::Signature: From<sr25519::Signature>, <T as frame_system::Config>::AccountId: From<sr25519::Public> }

	add {
		// let depth in 1 .. MAX_DEPTH;
		// let children in 1 .. MAX_CHILDREN;
		let depth: u32 = 1;
		let children: u32 = 1;

		let claim_hash: T::Hash = T::Hashing::hash(b"claim");
		let ctype_hash: T::Hash = T::Hashing::hash(b"ctype");
		let delegation_root_id: <T as delegation::Config>::DelegationNodeId = <<T as delegation::Config>::DelegationNodeId as Default>::default();
		let delegation_id = <<T as delegation::Config>::DelegationNodeId as Default>::default();

		let root_public = sr25519_generate(
			KeyTypeId(*b"aura"),
			None
		);
		let delegate_public = sr25519_generate(
			KeyTypeId(*b"aura"),
			None
		);

		let hash: Vec<u8> = Delegation::<T>::calculate_hash(
			delegation_id,
			delegation_root_id,
			None,
			delegation::Permissions::ATTEST
		).encode();
		let signature: T::Signature = sp_io::crypto::sr25519_sign(KeyTypeId(*b"aura"), &delegate_public, &hash).unwrap().into();

		let _ = Delegation::<T>::create_root(RawOrigin::Signed(root_public.clone().into()).into(), delegation_root_id, ctype_hash);
		// let delegation =
		Delegation::<T>::add_delegation(
			RawOrigin::Signed(root_public.clone().into()).into(),
			delegation_root_id,
			delegation_id,
			None,
			delegate_public.into(),
			delegation::Permissions::ATTEST,
			signature
		);

		// let (_, _, delegate, delegation_leaf) = delegation::benchmarking::setup_delegations::<T>(depth.into(), children.into())?;
		// let delegate_id: <T as frame_system::Config>::AccountId = delegate.public().into();
	}: _(RawOrigin::Signed(delegate_public.into()), claim_hash, ctype_hash, Some(delegation_id))
	verify {
		assert!(Attestations::<T>::contains_key(claim_hash));
		assert!(Attestations::<T>::contains_key(ctype_hash));
		assert_eq!(AttestationModule::<T>::attestations(claim_hash), Some(Attestation::<T> {
			ctype_hash,
			// attester: delegate_id,
			attester: delegate_public.into(),
			delegation_id: Some(delegation_id),
			revoked: false,
		}));
	}

	// revoke {
	// 	// let depth in 1 .. MAX_DEPTH ;
	// 	// let children in 1 .. MAX_CHILDREN;
	// 	let depth: u32 = 1;
	// 	let children: u32 = 1;

	// 	let claim_hash: T::Hash = T::Hashing::hash(b"claim");
	// 	let ctype_hash: T::Hash = T::Hashing::hash(b"ctype");

	// 	let (_, _, delegate, delegation_leaf) = delegation::benchmarking::setup_delegations::<T>(depth.into(), children.into())?;
	// 	let delegate_id: <T as frame_system::Config>::AccountId = delegate.public().into();
	// 	AttestationModule::<T>::add(RawOrigin::Signed(delegate_id.clone()).into(), claim_hash, ctype_hash, Some(delegation_leaf))?;
	// }: _(RawOrigin::Signed(delegate_id.clone()), claim_hash, depth.into())
	// verify {
	// 	assert!(Attestations::<T>::contains_key(claim_hash));
	// 	assert_eq!(Attestations::<T>::get(claim_hash), Some(Attestation::<T> {
	// 		ctype_hash,
	// 		attester: delegate_id,
	// 		delegation_id: Some(delegation_leaf),
	// 		revoked: true,
	// 	}));
	// }
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::tests::{new_test_ext, Test};
	use frame_support::assert_ok;

	#[test]
	fn test_benchmarks() {
		new_test_ext().execute_with(|| {
			assert_ok!(test_benchmark_add::<Test>());
		});
	}
}
