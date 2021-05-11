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

use codec::Encode;
use did::DidSignature;
use frame_benchmarking::{account, benchmarks};
use frame_support::dispatch::UnfilteredDispatchable;
use frame_system::RawOrigin;
use sp_core::{offchain::KeyTypeId, sr25519};
use sp_io::crypto::sr25519_generate;
use sp_std::num::NonZeroU32;

const SEED: u32 = 0;
const MAX_REVOCATIONS: u32 = 5;
const ONE_CHILD_PER_LEVEL: Option<NonZeroU32> = NonZeroU32::new(1);
const DID_KEY_IDENTIFIER: [u8; 4] = *b"did ";

struct DelegationTriplet<T: Config> {
	public: sr25519::Public,
	acc: T::AccountId,
	delegation_id: T::DelegationNodeId,
}

/// generats a delegation id from a given number
fn generate_delegation_id<T: Config>(number: u32) -> T::DelegationNodeId
where
	T::DelegationNodeId: From<T::Hash>,
{
	let hash: T::Hash = T::Hashing::hash(&number.to_ne_bytes());
	hash.into()
}

/// sets parent to `None` if it is the root
fn parent_id_check<T: Config>(
	root_id: T::DelegationNodeId,
	parent_id: T::DelegationNodeId,
) -> Option<T::DelegationNodeId> {
	if parent_id == root_id {
		None
	} else {
		Some(parent_id)
	}
}

/// add ctype to storage and root delegation
fn add_root_delegation<T: Config>(number: u32) -> Result<(DelegationTriplet<T>, T::Hash), DispatchError>
where
	T::AccountId: From<sr25519::Public>,
	T::DelegationNodeId: From<T::Hash>,
{
	let root_public = sr25519_generate(KeyTypeId(DID_KEY_IDENTIFIER), None);
	let root_acc: T::AccountId = root_public.into();
	let ctype_hash = <T::Hash as Default>::default();
	let root_id = generate_delegation_id::<T>(number);

	ctype::Pallet::<T>::add(RawOrigin::Signed(root_acc.clone()).into(), ctype_hash).map_err(|e| e.error)?;
	Pallet::<T>::create_root(RawOrigin::Signed(root_acc.clone()).into(), root_id, ctype_hash).map_err(|e| e.error)?;

	Ok((
		DelegationTriplet::<T> {
			public: root_public,
			acc: root_acc,
			delegation_id: root_id,
		},
		ctype_hash,
	))
}

/// recursively adds children delegations to a parent delegation for each level
/// until reaching leaf level
fn add_children<T: Config>(
	root_id: T::DelegationNodeId,
	parent_id: T::DelegationNodeId,
	parent_acc_public: sr25519::Public,
	parent_acc_id: T::AccountId,
	permissions: Permissions,
	level: u32,
	children_per_level: NonZeroU32,
) -> Result<(sr25519::Public, T::AccountId, T::DelegationNodeId), DispatchError>
where
	T::AccountId: From<sr25519::Public>,
	<T as did::Config>::DidIdentifier: From<T::AccountId>,
	T::DelegationNodeId: From<T::Hash>,
{
	if level == 0 {
		return Ok((parent_acc_public, parent_acc_id, parent_id));
	};

	let mut first_leaf = None;
	for c in 0..children_per_level.get() {
		// setup delegation account and id
		let delegation_acc_public = sr25519_generate(KeyTypeId(DID_KEY_IDENTIFIER), None);
		let delegation_acc_id: T::AccountId = delegation_acc_public.into();
		let delegation_id = generate_delegation_id::<T>(level * children_per_level.get() + c);

		// only set parent if not root
		let parent = parent_id_check::<T>(root_id, parent_id);

		// delegate signs delegation to parent
		let hash: Vec<u8> = Pallet::<T>::calculate_hash(&delegation_id, &root_id, &parent, &permissions).encode();
		let sig: DidSignature =
			sp_io::crypto::sr25519_sign(KeyTypeId(DID_KEY_IDENTIFIER), &delegation_acc_public, hash.as_ref())
				.ok_or("Error while building signature of delegation.")?
				.into();

		// add delegation from delegate to parent
		let _ = Pallet::<T>::add_delegation(
			RawOrigin::Signed(parent_acc_id.clone()).into(),
			delegation_id,
			root_id,
			parent,
			delegation_acc_id.clone().into(),
			permissions,
			sig,
		)
		.map_err(|e| e.error)?;

		// only return first leaf
		first_leaf = first_leaf.or(Some((delegation_acc_public, delegation_acc_id, delegation_id)));
	}

	let (leaf_acc_public, leaf_acc_id, leaf_id) =
		first_leaf.expect("Should not be None due to restricting children_per_level to NonZeroU32");

	// go to next level until we reach level 0
	add_children::<T>(
		root_id,
		leaf_id,
		leaf_acc_public,
		leaf_acc_id,
		permissions,
		level - 1,
		children_per_level,
	)
}

// setup delegations for an arbitrary depth and children per level
// 1. create ctype and root delegation
// 2. create and append children delegations to prior child for each level
pub fn setup_delegations<T: Config>(
	levels: u32,
	children_per_level: NonZeroU32,
	permissions: Permissions,
) -> Result<
	(
		sr25519::Public,
		T::DelegationNodeId,
		sr25519::Public,
		T::DelegationNodeId,
	),
	DispatchError,
>
where
	T::AccountId: From<sr25519::Public>,
	<T as did::Config>::DidIdentifier: From<T::AccountId>,
	T::DelegationNodeId: From<T::Hash>,
{
	let (
		DelegationTriplet::<T> {
			public: root_public,
			acc: root_acc,
			delegation_id: root_id,
		},
		_,
	) = add_root_delegation::<T>(0)?;

	// iterate levels and start with parent == root
	let (leaf_acc_public, _, leaf_id) = add_children::<T>(
		root_id,
		root_id,
		root_public,
		root_acc,
		permissions,
		levels,
		children_per_level,
	)?;
	Ok((root_public, root_id, leaf_acc_public, leaf_id))
}

benchmarks! {
	where_clause {
	where
		T: core::fmt::Debug,
		T::DelegationNodeId: From<T::Hash>,
		T::AccountId: From<sr25519::Public>,
		<T as did::Config>::DidIdentifier: From<T::AccountId>,
		<T as frame_system::Config>::Origin: From<did::Origin<T>>,
	}

	// TODO: Might want to add variant iterating over children instead of depth at some later point

	// worst case #2: revoke leaf node as root
	// because `is_delegating` has to traverse up to the root
	// complexitiy: O(h) with h = height of the delegation tree
	revoke_delegation_leaf {
		let r in 1 .. MAX_REVOCATIONS;
		let (root_acc, _, _, leaf_id) = setup_delegations::<T>(r, ONE_CHILD_PER_LEVEL.expect(">0"), Permissions::DELEGATE)?;

		let root_acc_id: T::AccountId = root_acc.into();
		let origin = did::Origin::<T> { id: root_acc_id.into() };

		let call = Call::<T>::revoke_delegation(leaf_id, r, r);
	}: { call.dispatch_bypass_filter(origin.into())? }
	verify {
		assert!(Delegations::<T>::contains_key(leaf_id));
		let DelegationNode::<T> { revoked, .. } = Delegations::<T>::get(leaf_id).ok_or("Child of root should have delegation id")?;
		assert!(revoked);
	}
	// TODO: Might want to add variant iterating over children instead of depth at some later point
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::tests::{ExtBuilder, Test};
	use ctype::CTYPEs;
	use frame_support::{assert_ok, StorageMap};
	use sp_std::num::NonZeroU32;

	#[test]
	fn test_benchmark_utils_generate_id() {
		ExtBuilder::build_with_keystore().execute_with(|| {
			assert_eq!(generate_delegation_id::<Test>(1), generate_delegation_id::<Test>(1));
			assert_ne!(generate_delegation_id::<Test>(1), generate_delegation_id::<Test>(2));
			let root = generate_delegation_id::<Test>(1);
			let parent = generate_delegation_id::<Test>(2);
			assert_eq!(parent_id_check::<Test>(root, root), None);
			assert_eq!(parent_id_check::<Test>(root, parent), Some(parent));
		});
	}
	#[test]
	fn test_benchmark_utils_manual_setup() {
		ExtBuilder::build_with_keystore().execute_with(|| {
			let (
				DelegationTriplet::<Test> {
					public: root_acc_public,
					acc: root_acc_id,
					delegation_id: root_id,
				},
				ctype_hash,
			) = add_root_delegation::<Test>(0).expect("failed to add root delegation");
			assert_eq!(root_id, generate_delegation_id::<Test>(0));
			assert!(Root::<Test>::contains_key(root_id));
			assert!(CTYPEs::<Test>::contains_key(ctype_hash));

			// add "parent" as child delegation of root
			let (parent_acc_public, parent_acc_id, parent_id) = add_children::<Test>(
				root_id,
				root_id,
				root_acc_public,
				root_acc_id,
				Permissions::DELEGATE,
				1,
				NonZeroU32::new(1).expect(">0"),
			)
			.expect("failed to add children to root delegation");
			assert_eq!(
				Delegations::<Test>::get(parent_id),
				Some(DelegationNode::<Test> {
					root_id,
					parent: None,
					owner: parent_acc_id.clone(),
					permissions: Permissions::DELEGATE,
					revoked: false
				})
			);

			// add "leaf" as child delegation of "parent"
			let (_, leaf_acc_id, leaf_id) = add_children::<Test>(
				root_id,
				parent_id,
				parent_acc_public,
				parent_acc_id,
				Permissions::DELEGATE,
				1,
				NonZeroU32::new(2).expect(">0"),
			)
			.expect("failed to add children to child of root delegation");
			assert_eq!(
				Delegations::<Test>::get(leaf_id),
				Some(DelegationNode::<Test> {
					root_id,
					parent: Some(parent_id),
					owner: leaf_acc_id,
					permissions: Permissions::DELEGATE,
					revoked: false
				})
			);
		});
	}
	#[test]
	fn test_benchmark_utils_auto_setup() {
		ExtBuilder::build_with_keystore().execute_with(|| {
			let (_, root_id, _, leaf_id) =
				setup_delegations::<Test>(2, NonZeroU32::new(2).expect(">0"), Permissions::DELEGATE)
					.expect("failed to run delegation setup");
			assert!(Root::<Test>::contains_key(root_id));
			assert!(Delegations::<Test>::contains_key(leaf_id));
		});
	}

	#[test]
	fn test_benchmarks() {
		ExtBuilder::build_with_keystore().execute_with(|| {
			assert_ok!(test_benchmark_create_root::<Test>());
			assert_ok!(test_benchmark_revoke_root::<Test>());
			assert_ok!(test_benchmark_add_delegation::<Test>());
			assert_ok!(test_benchmark_revoke_delegation_root_child::<Test>());
			assert_ok!(test_benchmark_revoke_delegation_leaf::<Test>());
		});
	}
}
