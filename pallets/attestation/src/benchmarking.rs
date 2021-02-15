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
use frame_benchmarking::{benchmarks};
use frame_support::{storage::StorageMap};
use frame_system::{RawOrigin};
use sp_core::{sr25519, Pair};
use sp_runtime::{
	traits::{Hash},
};

// const MAX_DEPTH: u32 = 100;
// const MAX_CHILDREN: u32 = 4;

benchmarks! {
	where_clause { where T: std::fmt::Debug, T::Signature: From<sr25519::Signature>, <T as frame_system::Config>::AccountId: From<sr25519::Public> }

	add {
		// let depth in 1 .. MAX_DEPTH;
		// let children in 1 .. MAX_CHILDREN;
		let depth: u32 = 1;
		let children: u32 = 1;

		let claim_hash: T::Hash = T::Hashing::hash(b"claim");
		let ctype_hash: T::Hash = T::Hashing::hash(b"ctype");

		let (_, _, delegate, delegation_leaf) = delegation::benchmarking::setup_delegations::<T>(depth.into(), children.into())?;
		let delegate_id: <T as frame_system::Config>::AccountId = delegate.public().into();
	}: _(RawOrigin::Signed(delegate_id.clone()), claim_hash, ctype_hash, Some(delegation_leaf))
	verify {
		assert!(Attestations::<T>::contains_key(claim_hash));
		assert_eq!(Attestations::<T>::get(claim_hash), Some(Attestation::<T> {
			ctype_hash,
			attester: delegate_id,
			delegation_id: Some(delegation_leaf),
			revoked: false,
		}));
	}

	revoke {
		// let depth in 1 .. MAX_DEPTH ;
		// let children in 1 .. MAX_CHILDREN;
		let depth: u32 = 1;
		let children: u32 = 1;

		let claim_hash: T::Hash = T::Hashing::hash(b"claim");
		let ctype_hash: T::Hash = T::Hashing::hash(b"ctype");

		let (_, _, delegate, delegation_leaf) = delegation::benchmarking::setup_delegations::<T>(depth.into(), children.into())?;
		let delegate_id: <T as frame_system::Config>::AccountId = delegate.public().into();
		AttestationModule::<T>::add(RawOrigin::Signed(delegate_id.clone()).into(), claim_hash, ctype_hash, Some(delegation_leaf))?;
	}: _(RawOrigin::Signed(delegate_id.clone()), claim_hash, depth.into())
	verify {
		assert!(Attestations::<T>::contains_key(claim_hash));
		assert_eq!(Attestations::<T>::get(claim_hash), Some(Attestation::<T> {
			ctype_hash,
			attester: delegate_id,
			delegation_id: Some(delegation_leaf),
			revoked: true,
		}));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::tests_composite::{ExtBuilder, Test};
	use frame_support::assert_ok;
}
