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

use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::{storage::StorageMap, traits::Currency};
use frame_system::RawOrigin;
use kilt_primitives::{AccountId, Balance};
use sp_core::{sr25519, Pair, H256};
use sp_runtime::{
	traits::{Bounded, Hash, IdentifyAccount, IdentityLookup, Verify},
	MultiSignature, MultiSigner,
};
use crate::Module as AttestationModule;
use delegation::Module as Delegation;
use frame_system::Module as System;
use pallet_balances::Module as Balances;
use sp_core::Public;

const SEED: u32 = 0;

fn hash_to_u8<T: Encode>(hash: T) -> Vec<u8> {
	hash.encode()
}

benchmarks! {
	where_clause { where T::Signature: From<sr25519::Signature>, <T as frame_system::Config>::AccountId: From<sr25519::Public> }

	add {
		// create root delegation
		let claim_hash: T::Hash = T::Hashing::hash(b"claim");
		let ctype_hash: T::Hash = T::Hashing::hash(b"ctype");

		let (root_acc, _, _, delegation_leaf) = delegation::benchmarking::setup_delegations::<T>(1, 1)?;
		let root_acc_id: <T as frame_system::Config>::AccountId = root_acc.public().into();
	}: _(RawOrigin::Signed(root_acc_id), claim_hash, ctype_hash, Some(delegation_leaf))
	verify {
		assert!(Attestations::<T>::contains_key(claim_hash));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::tests_composite::{ExtBuilder, Test};
	use frame_support::assert_ok;
}
