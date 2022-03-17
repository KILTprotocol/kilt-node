// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2022 BOTLabs GmbH

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

use super::*;

use codec::Encode;
use frame_benchmarking::{account, benchmarks, impl_benchmark_test_suite};
use frame_support::{
	dispatch::DispatchErrorWithPostInfo,
	storage::bounded_btree_set::BoundedBTreeSet,
	traits::{Currency, Get},
};
use frame_system::RawOrigin;
use sp_core::{offchain::KeyTypeId, sr25519};
use sp_io::crypto::sr25519_generate;
use sp_runtime::traits::Zero;
use sp_std::{num::NonZeroU32, vec::Vec};

use attestation::AttestationAccessControl;
use kilt_support::{signature::VerifySignature, traits::GenerateBenchmarkOrigin};

const SEED: u32 = 0;
const ONE_CHILD_PER_LEVEL: Option<NonZeroU32> = NonZeroU32::new(1);

struct DelegationTriplet<T: Config> {
	public: sr25519::Public,
	acc: T::DelegationEntityId,
	delegation_id: T::DelegationNodeId,
}

/// generats a delegation id from a given number
fn generate_delegation_id<T>(number: u32) -> T::DelegationNodeId
where
	T: Config,
	T::DelegationNodeId: From<T::Hash>,
{
	let hash: T::Hash = T::Hashing::hash(&number.to_ne_bytes());
	hash.into()
}

/// add ctype to storage and root delegation
fn add_delegation_hierarchy<T>(number: u32) -> Result<(DelegationTriplet<T>, T::Hash), DispatchErrorWithPostInfo>
where
	T: Config,
	T::DelegationEntityId: From<sr25519::Public>,
	T::DelegationNodeId: From<T::Hash>,
	T::CtypeCreatorId: From<T::DelegationEntityId>,
	<T as Config>::EnsureOrigin: GenerateBenchmarkOrigin<T::Origin, T::AccountId, T::DelegationEntityId>,
{
	log::info!("create delegation root");
	let root_public = sr25519_generate(KeyTypeId(*b"aura"), None);
	let root_acc: T::DelegationEntityId = root_public.into();
	let ctype_hash = <T::Hash as Default>::default();
	let hierarchy_root_id = generate_delegation_id::<T>(number);

	let sender: T::AccountId = account("sender", 0, SEED);
	<T as Config>::Currency::make_free_balance_be(
		&sender,
		<T as Config>::Currency::minimum_balance() + <T as Config>::Deposit::get() + <T as Config>::Deposit::get(),
	);

	ctype::Ctypes::<T>::insert(&ctype_hash, T::CtypeCreatorId::from(root_acc.clone()));

	Pallet::<T>::create_hierarchy(
		<T as Config>::EnsureOrigin::generate_origin(sender, root_acc.clone()),
		hierarchy_root_id,
		ctype_hash,
	)?;

	Ok((
		DelegationTriplet::<T> {
			public: root_public,
			acc: root_acc,
			delegation_id: hierarchy_root_id,
		},
		ctype_hash,
	))
}

/// recursively adds children delegations to a parent delegation for each level
/// until reaching leaf level
fn add_children<T>(
	root_id: T::DelegationNodeId,
	parent_id: T::DelegationNodeId,
	parent_acc_public: sr25519::Public,
	parent_acc_id: T::DelegationEntityId,
	permissions: Permissions,
	level: u32,
	children_per_level: NonZeroU32,
) -> Result<(sr25519::Public, T::DelegationEntityId, T::DelegationNodeId), DispatchErrorWithPostInfo>
where
	T: Config,
	T::DelegationEntityId: From<sr25519::Public>,
	T::DelegationNodeId: From<T::Hash>,
	<<T as Config>::DelegationSignatureVerification as VerifySignature>::Signature: From<(
		T::DelegationEntityId,
		<<T as Config>::DelegationSignatureVerification as VerifySignature>::Payload,
	)>,
	<T as Config>::EnsureOrigin: GenerateBenchmarkOrigin<T::Origin, T::AccountId, T::DelegationEntityId>,
{
	if level == 0 {
		return Ok((parent_acc_public, parent_acc_id, parent_id));
	};
	let sender: T::AccountId = account("sender", 0, SEED);

	let mut first_leaf = None;
	for c in 0..children_per_level.get() {
		// setup delegation account and id
		let delegation_acc_public = sr25519_generate(KeyTypeId(*b"aura"), None);
		let delegation_acc_id: T::DelegationEntityId = delegation_acc_public.into();
		let delegation_id = generate_delegation_id::<T>(level * children_per_level.get() + c);

		// delegate signs delegation to parent
		let hash: Vec<u8> =
			Pallet::<T>::calculate_delegation_creation_hash(&delegation_id, &root_id, &parent_id, &permissions)
				.encode();
		// Either EqualVerify or AlwaysVerify should be used for benchmarks. Therefore
		// we build a signature that can be verified by both.
		let sig = (delegation_acc_id.clone(), hash.clone());

		// add delegation from delegate to parent
		<T as Config>::Currency::make_free_balance_be(
			&sender,
			<T as Config>::Currency::minimum_balance() + <T as Config>::Deposit::get() + <T as Config>::Deposit::get(),
		);
		let _ = Pallet::<T>::add_delegation(
			<T as Config>::EnsureOrigin::generate_origin(sender.clone(), parent_acc_id.clone()),
			delegation_id,
			parent_id,
			delegation_acc_id.clone(),
			permissions,
			sig.into(),
		)?;

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
pub fn setup_delegations<T>(
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
	DispatchErrorWithPostInfo,
>
where
	T: Config,
	T::DelegationEntityId: From<sr25519::Public>,
	T::CtypeCreatorId: From<T::DelegationEntityId>,
	T::DelegationNodeId: From<T::Hash>,
	<<T as Config>::DelegationSignatureVerification as VerifySignature>::Signature: From<(
		T::DelegationEntityId,
		<<T as Config>::DelegationSignatureVerification as VerifySignature>::Payload,
	)>,
	<T as Config>::EnsureOrigin: GenerateBenchmarkOrigin<T::Origin, T::AccountId, T::DelegationEntityId>,
{
	let (
		DelegationTriplet::<T> {
			public: root_public,
			acc: root_acc,
			delegation_id: hierarchy_id,
		},
		_,
	) = add_delegation_hierarchy::<T>(0)?;

	// iterate levels and start with parent == root
	let (leaf_acc_public, _, leaf_id) = add_children::<T>(
		hierarchy_id,
		hierarchy_id,
		root_public,
		root_acc,
		permissions,
		levels,
		children_per_level,
	)?;
	Ok((root_public, hierarchy_id, leaf_acc_public, leaf_id))
}

benchmarks! {
	where_clause {
		where
		T: core::fmt::Debug,
		T::DelegationNodeId: From<T::Hash>,
		T::DelegationEntityId: From<sr25519::Public>,
		<T as ctype::Config>::CtypeCreatorId: From<T::DelegationEntityId>,
		<<T as Config>::DelegationSignatureVerification as VerifySignature>::Signature: From<(
			T::DelegationEntityId,
			<<T as Config>::DelegationSignatureVerification as VerifySignature>::Payload,
		)>,
		<T as Config>::EnsureOrigin: GenerateBenchmarkOrigin<T::Origin, T::AccountId, T::DelegationEntityId>,
	}

	create_hierarchy {
		let sender: T::AccountId = account("sender", 0, SEED);
		let creator: T::DelegationEntityId = account("creator", 0, SEED);
		let ctype = <T::Hash as Default>::default();
		let delegation = generate_delegation_id::<T>(0);
		ctype::Ctypes::<T>::insert(&ctype, <T as ctype::Config>::CtypeCreatorId::from(creator.clone()));
		<T as Config>::Currency::make_free_balance_be(
			&sender,
			<T as Config>::Currency::minimum_balance() + <T as Config>::Deposit::get(),
		);

		let origin = <T as Config>::EnsureOrigin::generate_origin(sender, creator);
	}: _<T::Origin>(origin, delegation, ctype)
	verify {
		assert!(DelegationHierarchies::<T>::contains_key(delegation));
	}

	add_delegation {
		let sender: T::AccountId = account("sender", 0, SEED);

		let (
			DelegationTriplet::<T> {
				public: root_public,
				acc: root_acc,
				delegation_id: hierarchy_id,
			},
			_,
		) = add_delegation_hierarchy::<T>(0)?;

		// add one more delegation
		let delegate_acc_public = sr25519_generate(
			KeyTypeId(*b"aura"),
			None
		);
		let delegate_acc_id: T::DelegationEntityId = delegate_acc_public.into();

		let delegation_id = generate_delegation_id::<T>(u32::MAX);
		let parent_id = hierarchy_id;

		let perm: Permissions = Permissions::ATTEST | Permissions::DELEGATE;
		let hash_root = Pallet::<T>::calculate_delegation_creation_hash(&delegation_id, &hierarchy_id, &parent_id, &perm);

		// Either EqualVerify or AlwaysVerify should be used for benchmarks. Therefore we build a
		// signature that can be verified by both.
		let sig = (delegate_acc_id.clone(), AsRef::<[u8]>::as_ref(&hash_root).to_vec()).into();

		let leaf_acc_id: T::DelegationEntityId = root_public.into();
		<T as Config>::Currency::make_free_balance_be(
			&sender,
			<T as Config>::Currency::minimum_balance() + <T as Config>::Deposit::get(),
		);
		let origin = <T as Config>::EnsureOrigin::generate_origin(sender, leaf_acc_id);
	}: _<T::Origin>(origin, delegation_id, hierarchy_id, delegate_acc_id, perm, sig)
	verify {
		assert!(DelegationNodes::<T>::contains_key(delegation_id));
	}

	// worst case #1: revoke a child of the root delegation
	// because all of its children have to be revoked
	// complexitiy: O(h * c) with h = height of the delegation tree, c = max number of children in a level
	revoke_delegation_root_child {
		let r in 1 .. T::MaxRevocations::get();
		let c in 1 .. T::MaxParentChecks::get();

		let sender: T::AccountId = account("sender", 0, SEED);
		let (_, hierarchy_id, leaf_acc, leaf_id) = setup_delegations::<T>(r, ONE_CHILD_PER_LEVEL.expect(">0"), Permissions::DELEGATE)?;

		let root_node = DelegationNodes::<T>::get(hierarchy_id).expect("Root hierarchy node should be present on chain.");
		let children: BoundedBTreeSet<T::DelegationNodeId, T::MaxChildren> = root_node.children;
		let child_id: T::DelegationNodeId = *children.iter().next().ok_or("Root should have children")?;
		let child_delegation = DelegationNodes::<T>::get(child_id).ok_or("Child of root should have delegation id")?;
		<T as Config>::Currency::make_free_balance_be(
			&child_delegation.deposit.owner,
			<T as Config>::Currency::minimum_balance() + <T as Config>::Deposit::get(),
		);
		let origin = <T as Config>::EnsureOrigin::generate_origin(sender, child_delegation.details.owner);
	}: revoke_delegation<T::Origin>(origin, child_id, c, r)
	verify {
		assert!(DelegationNodes::<T>::contains_key(child_id));
		let DelegationNode::<T> { details, .. } = DelegationNodes::<T>::get(leaf_id).ok_or("Child of root should have delegation id")?;
		assert!(details.revoked);

		assert!(DelegationNodes::<T>::contains_key(leaf_id));
		let leaf_delegation = DelegationNodes::<T>::get(leaf_id).ok_or("Missing leaf delegation")?;
		assert_eq!(leaf_delegation.hierarchy_root_id, hierarchy_id);
		assert_eq!(leaf_delegation.details.owner, leaf_acc.into());
		assert!(leaf_delegation.details.revoked);
	}
	// TODO: Might want to add variant iterating over children instead of depth at some later point

	// worst case #2: revoke leaf node as root
	// because `is_delegating` has to traverse up to the root
	// complexitiy: O(h) with h = height of the delegation tree
	revoke_delegation_leaf {
		let r in 1 .. T::MaxRevocations::get();
		let c in 1 .. T::MaxParentChecks::get();

		let sender: T::AccountId = account("sender", 0, SEED);
		let (root_acc, _, _, leaf_id) = setup_delegations::<T>(c, ONE_CHILD_PER_LEVEL.expect(">0"), Permissions::DELEGATE)?;
		let origin = <T as Config>::EnsureOrigin::generate_origin(sender, root_acc.into());
	}: revoke_delegation<T::Origin>(origin, leaf_id, c, r)
	verify {
		assert!(DelegationNodes::<T>::contains_key(leaf_id));
		let DelegationNode::<T> { details, .. } = DelegationNodes::<T>::get(leaf_id).ok_or("Child of root should have delegation id")?;
		assert!(details.revoked);
	}
	// TODO: Might want to add variant iterating over children instead of depth at some later point

	// worst case is achieved by removing the root node, since `is_delegating` is not called in remove extrinsic
	remove_delegation {
		let r in 1 .. T::MaxRemovals::get();

		let sender: T::AccountId = account("sender", 0, SEED);

		let (root_acc, hierarchy_id, _, leaf_id) = setup_delegations::<T>(r, ONE_CHILD_PER_LEVEL.expect(">0"), Permissions::DELEGATE)?;
		let root_node = DelegationNodes::<T>::get(hierarchy_id).expect("Root hierarchy node should be present on chain.");
		let children: BoundedBTreeSet<T::DelegationNodeId, T::MaxChildren> = root_node.children;
		let child_id: T::DelegationNodeId = *children.iter().next().ok_or("Root should have children")?;
		let child_delegation = DelegationNodes::<T>::get(child_id).ok_or("Child of root should have delegation id")?;
		assert!(!<T as Config>::Currency::reserved_balance(&sender).is_zero());
		let origin = <T as Config>::EnsureOrigin::generate_origin(sender.clone(), root_acc.into());
	}: _<T::Origin>(origin, hierarchy_id, r)
	verify {
		assert!(!DelegationNodes::<T>::contains_key(hierarchy_id));
		assert!(!DelegationNodes::<T>::contains_key(child_id));
		assert!(!DelegationNodes::<T>::contains_key(leaf_id));
		assert!(<T as Config>::Currency::reserved_balance(&sender).is_zero());
	}

	// worst case is achieved by removing the root node, since `is_delegating` is not called in remove extrinsic
	reclaim_deposit {
		let r in 1 .. T::MaxRemovals::get();

		let sender: T::AccountId = account("sender", 0, SEED);
		let (root_acc, hierarchy_id, _, leaf_id) = setup_delegations::<T>(r, ONE_CHILD_PER_LEVEL.expect(">0"), Permissions::DELEGATE)?;
		let root_node = DelegationNodes::<T>::get(hierarchy_id).expect("Root hierarchy node should be present on chain.");
		let children: BoundedBTreeSet<T::DelegationNodeId, T::MaxChildren> = root_node.children;
		let child_id: T::DelegationNodeId = *children.iter().next().ok_or("Root should have children")?;
		let child_delegation = DelegationNodes::<T>::get(child_id).ok_or("Child of root should have delegation id")?;
		assert!(!<T as Config>::Currency::reserved_balance(&sender).is_zero());

		let origin = RawOrigin::Signed(sender.clone());
	}: _(origin, hierarchy_id, r)
	verify {
		assert!(!DelegationNodes::<T>::contains_key(hierarchy_id));
		assert!(!DelegationNodes::<T>::contains_key(child_id));
		assert!(!DelegationNodes::<T>::contains_key(leaf_id));
		assert!(<T as Config>::Currency::reserved_balance(&sender).is_zero());
	}

	can_attest {
		let c = T::MaxParentChecks::get();

		let ctype = Default::default();
		let claim = Default::default();

		let sender: T::AccountId = account("sender", 0, SEED);
		let (root_acc, _, leaf_acc, leaf_id) = setup_delegations::<T>(c, ONE_CHILD_PER_LEVEL.expect(">0"), Permissions::DELEGATE | Permissions::ATTEST)?;
		let root_acc: T::DelegationEntityId = root_acc.into();
		let leaf_acc: T::DelegationEntityId = leaf_acc.into();

		let ac = DelegationAc::<T>{
			subject_node_id: leaf_id,
			max_checks: c
		};

	}: { ac.can_attest(&leaf_acc, &ctype, &claim).expect("Should be allowed") }
	verify {
	}

	can_revoke {
		let c in 1 .. T::MaxParentChecks::get();

		let ctype = Default::default();
		let claim = Default::default();

		let sender: T::AccountId = account("sender", 0, SEED);
		let (root_acc, root_id, _, leaf_id) = setup_delegations::<T>(c, ONE_CHILD_PER_LEVEL.expect(">0"), Permissions::DELEGATE)?;
		let root_acc: T::DelegationEntityId = root_acc.into();

		let ac = DelegationAc::<T>{
			subject_node_id: leaf_id,
			max_checks: c
		};

	}: { ac.can_revoke(&root_acc, &ctype, &claim, &leaf_id).expect("Should be allowed") }
	verify {
	}

	can_remove {
		let c in 1 .. T::MaxParentChecks::get();

		let ctype = Default::default();
		let claim = Default::default();

		let sender: T::AccountId = account("sender", 0, SEED);
		let (root_acc, root_id, _, leaf_id) = setup_delegations::<T>(c, ONE_CHILD_PER_LEVEL.expect(">0"), Permissions::DELEGATE)?;
		let root_acc: T::DelegationEntityId = root_acc.into();

		let ac = DelegationAc::<T>{
			subject_node_id: leaf_id,
			max_checks: c
		};

	}: { ac.can_remove(&root_acc, &ctype, &claim, &leaf_id).expect("Should be allowed") }
	verify {
	}
}

impl_benchmark_test_suite! {
	Pallet,
	crate::mock::runtime::ExtBuilder::default().build_with_keystore(),
	crate::mock::runtime::Test
}
