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

use frame_benchmarking::{account, benchmarks};
use frame_system::RawOrigin;
use sp_core::{offchain::KeyTypeId, sr25519};
use sp_io::crypto::sr25519_generate;
use sp_std::{boxed::Box, vec, vec::Vec};

const SEED: u32 = 0;

/// generats a delegation id from a given number
fn generate_delegation_id<T: Config>(number: u64) -> T::DelegationNodeId
where
	T::DelegationNodeId: From<<T as frame_system::Config>::Hash>,
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
fn add_root_delegation<T: Config>(
	number: u64,
) -> Result<
	(
		sr25519::Public,
		<T as frame_system::Config>::AccountId,
		T::DelegationNodeId,
		T::Hash,
	),
	DispatchError,
>
where
	<T as frame_system::Config>::AccountId: From<sr25519::Public>,
	T::DelegationNodeId: From<<T as frame_system::Config>::Hash>,
{
	let root_public = sr25519_generate(KeyTypeId(*b"aura"), None);
	let root_acc: <T as frame_system::Config>::AccountId = root_public.into();
	let ctype_hash = <<T as frame_system::Config>::Hash as Default>::default();
	let root_id = generate_delegation_id::<T>(number);

	ctype::Module::<T>::add(RawOrigin::Signed(root_acc.clone()).into(), ctype_hash)?;
	Module::<T>::create_root(
		RawOrigin::Signed(root_acc.clone()).into(),
		root_id,
		ctype_hash,
	)?;

	Ok((root_public, root_acc, root_id, ctype_hash))
}

/// recursively adds children delegations to a parent delegation for each level until reaching leaf level
fn add_children<T: Config>(
	root_id: <T as Config>::DelegationNodeId,
	parent_id: <T as Config>::DelegationNodeId,
	parent_acc_public: sr25519::Public,
	parent_acc_id: <T as frame_system::Config>::AccountId,
	permissions: Permissions,
	level: u64,
	children_per_level: u64,
) -> Result<
	(
		sr25519::Public,
		<T as frame_system::Config>::AccountId,
		T::DelegationNodeId,
	),
	DispatchError,
>
where
	<T as frame_system::Config>::AccountId: From<sr25519::Public>,
	<T as Config>::Signature: From<sr25519::Signature>,
	T::DelegationNodeId: From<<T as frame_system::Config>::Hash>,
{
	if level == 0 {
		return Ok((parent_acc_public, parent_acc_id, parent_id));
	};

	let mut leaf = None;
	for c in 0..children_per_level {
		// setup delegation account and id
		let delegation_acc_public = sr25519_generate(KeyTypeId(*b"aura"), None);
		let delegation_acc_id: <T as frame_system::Config>::AccountId =
			delegation_acc_public.into();
		let delegation_id = generate_delegation_id::<T>(level * children_per_level + c);

		// only set parent if not root
		let parent = parent_id_check::<T>(root_id, parent_id);

		// delegate signs delegation to parent
		let hash: Vec<u8> =
			Module::<T>::calculate_hash(delegation_id, root_id, parent, permissions).encode();
		let sig: <T as Config>::Signature =
			sp_io::crypto::sr25519_sign(KeyTypeId(*b"aura"), &delegation_acc_public, hash.as_ref())
				.ok_or("Error while building signature of delegation.")?
				.into();

		// add delegation from delegate to parent
		let _ = Module::<T>::add_delegation(
			RawOrigin::Signed(parent_acc_id.clone()).into(),
			delegation_id,
			root_id,
			parent,
			delegation_acc_id.clone(),
			permissions,
			sig,
		)?;

		// only put in a leaf in the the first iteration
		leaf = leaf.or(Some((
			delegation_acc_public,
			delegation_acc_id,
			delegation_id,
		)));
	}
	// if we didn't add children, return the parent
	let (leaf_acc_public, leaf_acc_id, leaf_id) =
		leaf.unwrap_or((parent_acc_public, parent_acc_id, parent_id));

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
	levels: u64,
	children_per_level: u64,
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
	<T as frame_system::Config>::AccountId: From<sr25519::Public>,
	<T as Config>::Signature: From<sr25519::Signature>,
	T::DelegationNodeId: From<<T as frame_system::Config>::Hash>,
{
	let (root_public, root_acc, root_id, _) = add_root_delegation::<T>(0)?;

	// iterate levels and start with parent == root
	let (leaf_acc_public, _, leaf_id) = add_children::<T>(
		root_id,
		root_id,
		root_public,
		root_acc.clone(),
		permissions,
		levels,
		children_per_level,
	)?;
	return Ok((root_public, root_id, leaf_acc_public, leaf_id));
}

benchmarks! {
	where_clause { where T: core::fmt::Debug, T::Signature: From<sr25519::Signature>, <T as frame_system::Config>::AccountId: From<sr25519::Public>, 	T::DelegationNodeId: From<<T as frame_system::Config>::Hash> }

	create_root {
		let caller: <T as frame_system::Config>::AccountId = account("caller", 0, SEED);
		let ctype = <<T as frame_system::Config>::Hash as Default>::default();
		let delegation = generate_delegation_id::<T>(0);
		ctype::Module::<T>::add(RawOrigin::Signed(caller.clone()).into(), ctype)?;
	}: _(RawOrigin::Signed(caller), delegation, ctype)
	verify {
		assert!(Root::<T>::contains_key(delegation));
	}

	revoke_root {
		// TODO: Switch to variable depth & children
		let depth = 1;
		let (root_acc, root_id, leaf_acc, leaf_id) = setup_delegations::<T>(5, 1, Permissions::DELEGATE)?;
		let root_acc_id: <T as frame_system::Config>::AccountId = root_acc.into();
	}: _(RawOrigin::Signed(root_acc_id.clone()), root_id)
	verify {
		assert!(Root::<T>::contains_key(root_id));
		let root_delegation = Root::<T>::get(root_id).ok_or("Missing root delegation")?;
		assert_eq!(root_delegation.owner, root_acc_id);
		assert_eq!(root_delegation.revoked, true);

		assert!(Delegations::<T>::contains_key(leaf_id));
		let leaf_delegation = Delegations::<T>::get(leaf_id).ok_or("Missing leaf delegation")?;
		assert_eq!(leaf_delegation.root_id, root_id);
		assert_eq!(leaf_delegation.owner, leaf_acc.into());
		assert_eq!(leaf_delegation.revoked, true);
	}

	add_delegation {
		// TODO: Switch to variable depth & children
		// do setup
		let (_, root_id, leaf_acc, leaf_id) = setup_delegations::<T>(1, 1, Permissions::DELEGATE)?;

		// add one more delegation
		let delegate_acc_public = sr25519_generate(
			KeyTypeId(*b"aura"),
			None
		);
		let delegation_id = generate_delegation_id::<T>(u64::MAX);
		let parent_id = parent_id_check::<T>(root_id, leaf_id);

		let perm: Permissions = Permissions::ATTEST | Permissions::DELEGATE;
		let hash_root = Module::<T>::calculate_hash(delegation_id, root_id, parent_id, perm);
		let sig: <T as Config>::Signature = sp_io::crypto::sr25519_sign(KeyTypeId(*b"aura"), &delegate_acc_public, hash_root.as_ref()).ok_or("Error while building signature of delegation.")?.into();

		let delegate_acc_id: <T as frame_system::Config>::AccountId = delegate_acc_public.into();
		let leaf_acc_id: <T as frame_system::Config>::AccountId = leaf_acc.into();
	}: _(RawOrigin::Signed(leaf_acc_id), delegation_id, root_id, parent_id, delegate_acc_id, perm, sig)
	verify {
		assert!(Delegations::<T>::contains_key(delegation_id));
	}

	// worst case is to revoke child of root delegation
	revoke_delegation {
		// TODO: Switch to variable depth & children
		let depth = 1;
		let (_, root_id, leaf_acc, leaf_id) = setup_delegations::<T>(depth, 1, Permissions::DELEGATE)?;
		let children: Vec<T::DelegationNodeId> = Children::<T>::get(root_id);
		let child_id: T::DelegationNodeId = *children.get(0).ok_or("Root should have children")?;
		let child_delegation = Delegations::<T>::get(child_id).ok_or("Child of root should have delegation id")?;
	}: _(RawOrigin::Signed(child_delegation.owner.clone()), child_id, depth)
	verify {
		assert!(Delegations::<T>::contains_key(child_id));
		let DelegationNode::<T> { revoked, .. } = Delegations::<T>::get(leaf_id).ok_or("Child of root should have delegation id")?;
		assert_eq!(revoked, true);

		assert!(Delegations::<T>::contains_key(leaf_id));
		let leaf_delegation = Delegations::<T>::get(leaf_id).ok_or("Missing leaf delegation")?;
		assert_eq!(leaf_delegation.root_id, root_id);
		assert_eq!(leaf_delegation.owner, leaf_acc.into());
		assert_eq!(leaf_delegation.revoked, true);
	}
}

// TODO: Add tests
