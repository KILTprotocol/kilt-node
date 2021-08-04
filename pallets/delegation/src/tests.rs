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

use codec::Encode;
use frame_support::{assert_err, assert_noop, assert_ok};
use sp_core::Pair;

use crate::{self as delegation, mock::*};
use ctype::mock as ctype_mock;

// submit_delegation_root_creation_operation()

#[test]
fn create_root_delegation_successful() {
	let creator_keypair = get_alice_ed25519();
	let creator = get_ed25519_account(creator_keypair.public());

	let hierarchy_root_id = get_delegation_hierarchy_id(true);

	let operation = generate_base_delegation_hierarchy_creation_operation(hierarchy_root_id);

	let mut ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(operation.ctype_hash, creator.clone())])
		.build(None);

	ext.execute_with(|| {
		assert_ok!(Delegation::create_hierarchy(
			get_origin(creator.clone()),
			operation.id,
			operation.ctype_hash
		));
	});

	let stored_hierarchy_details = ext.execute_with(|| {
		Delegation::delegation_hierarchies(&hierarchy_root_id)
			.expect("Delegation hierarchy should be present on chain.")
	});
	assert_eq!(stored_hierarchy_details.ctype_hash, operation.ctype_hash);

	let stored_delegation_root = ext.execute_with(|| {
		Delegation::delegation_nodes(&hierarchy_root_id).expect("Delegation root should be present on chain.")
	});

	assert_eq!(stored_delegation_root.hierarchy_root_id, hierarchy_root_id);
	assert_eq!(stored_delegation_root.parent, None);
	assert_eq!(stored_delegation_root.children.len(), 0);
	assert_eq!(stored_delegation_root.details.owner, creator);
	assert!(!stored_delegation_root.details.revoked);
}

#[test]
fn duplicate_create_root_delegation_error() {
	let creator_keypair = get_alice_ed25519();
	let creator = get_ed25519_account(creator_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id(true),
		generate_base_delegation_hierarchy_details(),
	);

	let operation = generate_base_delegation_hierarchy_creation_operation(hierarchy_root_id);

	let ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(operation.ctype_hash, creator.clone())])
		.build(None);
	let mut ext = ExtBuilder::default()
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, creator.clone())])
		.build(Some(ext));

	ext.execute_with(|| {
		assert_noop!(
			Delegation::create_hierarchy(get_origin(creator.clone()), operation.id, operation.ctype_hash),
			delegation::Error::<Test>::HierarchyAlreadyExists
		);
	});
}

#[test]
fn ctype_not_found_create_root_delegation_error() {
	let creator_keypair = get_alice_ed25519();
	let creator = get_ed25519_account(creator_keypair.public());

	let hierarchy_root_id = get_delegation_hierarchy_id(true);

	let operation = generate_base_delegation_hierarchy_creation_operation(hierarchy_root_id);

	// No CType stored,
	let mut ext = ExtBuilder::default().build(None);

	ext.execute_with(|| {
		assert_noop!(
			Delegation::create_hierarchy(get_origin(creator.clone()), operation.id, operation.ctype_hash),
			ctype::Error::<Test>::CTypeNotFound
		);
	});
}

// submit_delegation_creation_operation()

#[test]
fn create_delegation_direct_root_successful() {
	let creator_keypair = get_alice_ed25519();
	let creator = get_ed25519_account(creator_keypair.public());
	let delegate_keypair = get_bob_sr25519();
	let delegate = get_sr25519_account(delegate_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id(true),
		generate_base_delegation_hierarchy_details(),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(hierarchy_root_id, delegate, Some(hierarchy_root_id)),
	);

	let delegation_info = Delegation::calculate_delegation_creation_hash(
		&delegation_id,
		&hierarchy_root_id,
		&hierarchy_root_id,
		&delegation_node.details.permissions,
	);

	let delegate_signature = delegate_keypair.sign(&hash_to_u8(delegation_info));

	let operation =
		generate_base_delegation_creation_operation(delegation_id, delegate_signature.into(), delegation_node);

	let ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, creator.clone())])
		.build(None);
	let mut ext = ExtBuilder::default()
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, creator.clone())])
		.build(Some(ext));

	ext.execute_with(|| {
		assert_ok!(Delegation::add_delegation(
			get_origin(creator.clone()),
			operation.delegation_id,
			operation.parent_id,
			operation.delegate.clone(),
			operation.permissions,
			operation.delegate_signature.clone().encode(),
		));
	});

	let stored_delegation = ext.execute_with(|| {
		Delegation::delegation_nodes(&operation.delegation_id).expect("Delegation should be present on chain.")
	});

	assert_eq!(stored_delegation.hierarchy_root_id, operation.hierarchy_id);
	assert_eq!(stored_delegation.parent, Some(operation.parent_id));
	assert!(stored_delegation.children.is_empty());
	assert_eq!(stored_delegation.details.owner, operation.delegate);
	assert_eq!(stored_delegation.details.permissions, operation.permissions);
	assert!(!stored_delegation.details.revoked);

	// Verify that the root has the new delegation among its children
	let stored_root = ext.execute_with(|| {
		Delegation::delegation_nodes(&operation.hierarchy_id).expect("Delegation root should be present on chain.")
	});

	assert!(stored_root.children.contains(&operation.delegation_id));
}

#[test]
fn create_delegation_with_parent_successful() {
	let creator_keypair = get_alice_ed25519();
	let creator = get_ed25519_account(creator_keypair.public());
	let delegate_keypair = get_bob_sr25519();
	let delegate = get_sr25519_account(delegate_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id(true),
		generate_base_delegation_hierarchy_details(),
	);
	let (parent_id, parent_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(hierarchy_root_id, creator.clone(), Some(hierarchy_root_id)),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(hierarchy_root_id, delegate, Some(parent_id)),
	);

	let delegation_info = Delegation::calculate_delegation_creation_hash(
		&delegation_id,
		&hierarchy_root_id,
		&parent_id,
		&delegation_node.details.permissions,
	);

	let delegate_signature = delegate_keypair.sign(&hash_to_u8(delegation_info));

	let operation =
		generate_base_delegation_creation_operation(delegation_id, delegate_signature.into(), delegation_node);

	let ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, creator.clone())])
		.build(None);
	let mut ext = ExtBuilder::default()
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, creator.clone())])
		.with_delegations(vec![(parent_id, parent_node)])
		.build(Some(ext));

	ext.execute_with(|| {
		assert_ok!(Delegation::add_delegation(
			get_origin(creator.clone()),
			operation.delegation_id,
			operation.parent_id,
			operation.delegate.clone(),
			operation.permissions,
			operation.delegate_signature.clone().encode(),
		));
	});

	let stored_delegation = ext.execute_with(|| {
		Delegation::delegation_nodes(&operation.delegation_id).expect("Delegation should be present on chain.")
	});

	assert_eq!(stored_delegation.hierarchy_root_id, operation.hierarchy_id);
	assert_eq!(stored_delegation.parent, Some(operation.parent_id));
	assert!(stored_delegation.children.is_empty());
	assert_eq!(stored_delegation.details.owner, operation.delegate);
	assert_eq!(stored_delegation.details.permissions, operation.permissions);
	assert!(!stored_delegation.details.revoked);

	// Verify that the parent has the new delegation among its children
	let stored_parent = ext.execute_with(|| {
		Delegation::delegation_nodes(&operation.parent_id).expect("Delegation parent should be present on chain.")
	});

	assert!(stored_parent.children.contains(&operation.delegation_id));
}

#[test]
fn create_delegation_direct_root_revoked_error() {
	let creator_keypair = get_alice_ed25519();
	let creator = get_ed25519_account(creator_keypair.public());
	let delegate_keypair = get_bob_sr25519();
	let delegate = get_sr25519_account(delegate_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id(true),
		generate_base_delegation_hierarchy_details(),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(hierarchy_root_id, delegate.clone(), Some(hierarchy_root_id)),
	);

	let delegation_info = Delegation::calculate_delegation_creation_hash(
		&delegation_id,
		&hierarchy_root_id,
		&hierarchy_root_id,
		&delegation_node.details.permissions,
	);

	let delegate_signature = delegate_keypair.sign(&hash_to_u8(delegation_info));

	let operation =
		generate_base_delegation_creation_operation(delegation_id, delegate_signature.into(), delegation_node);

	let ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, creator.clone())])
		.build(None);
	let mut ext = ExtBuilder::default()
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, creator.clone())])
		.build(Some(ext));

	ext.execute_with(|| {
		let _ = Delegation::revoke_delegation(
			get_origin(creator.clone()),
			operation.hierarchy_id,
			0u32,
			MaxRevocations::get(),
		);
		assert_noop!(
			Delegation::add_delegation(
				get_origin(creator.clone()),
				operation.delegation_id,
				operation.parent_id,
				delegate,
				operation.permissions,
				operation.delegate_signature.clone().encode(),
			),
			delegation::Error::<Test>::ParentDelegationRevoked
		);
	});
}

#[test]
fn create_delegation_with_parent_revoked_error() {
	let creator_keypair = get_alice_ed25519();
	let creator = get_ed25519_account(creator_keypair.public());
	let delegate_keypair = get_bob_sr25519();
	let delegate = get_sr25519_account(delegate_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id(true),
		generate_base_delegation_hierarchy_details(),
	);
	let (parent_id, parent_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(hierarchy_root_id, creator.clone(), Some(hierarchy_root_id)),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(hierarchy_root_id, delegate.clone(), Some(parent_id)),
	);

	let delegation_info = Delegation::calculate_delegation_creation_hash(
		&delegation_id,
		&hierarchy_root_id,
		&parent_id,
		&delegation_node.details.permissions,
	);

	let delegate_signature = delegate_keypair.sign(&hash_to_u8(delegation_info));

	let operation =
		generate_base_delegation_creation_operation(delegation_id, delegate_signature.into(), delegation_node);

	let ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, creator.clone())])
		.build(None);
	let mut ext = ExtBuilder::default()
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, creator.clone())])
		.with_delegations(vec![(parent_id, parent_node)])
		.build(Some(ext));

	ext.execute_with(|| {
		let _ = Delegation::revoke_delegation(
			get_origin(creator.clone()),
			operation.parent_id,
			MaxRevocations::get(),
			MaxParentChecks::get(),
		);
		assert_noop!(
			Delegation::add_delegation(
				get_origin(creator.clone()),
				operation.delegation_id,
				operation.parent_id,
				delegate,
				operation.permissions,
				operation.delegate_signature.clone().encode(),
			),
			delegation::Error::<Test>::ParentDelegationRevoked
		);
	});
}

#[test]
fn invalid_delegate_signature_create_delegation_error() {
	let creator_keypair = get_alice_ed25519();
	let creator = get_ed25519_account(creator_keypair.public());
	let alternative_keypair = get_alice_sr25519();
	let delegate_keypair = get_bob_sr25519();
	let delegate = get_sr25519_account(delegate_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id(true),
		generate_base_delegation_hierarchy_details(),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(hierarchy_root_id, delegate.clone(), Some(hierarchy_root_id)),
	);

	let delegate_signature = alternative_keypair.sign(&hash_to_u8(Delegation::calculate_delegation_creation_hash(
		&delegation_id,
		&hierarchy_root_id,
		&hierarchy_root_id,
		&delegation_node.details.permissions,
	)));

	let operation =
		generate_base_delegation_creation_operation(delegation_id, delegate_signature.into(), delegation_node);

	let ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, creator.clone())])
		.build(None);
	let mut ext = ExtBuilder::default()
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, creator.clone())])
		.build(Some(ext));

	ext.execute_with(|| {
		assert_noop!(
			Delegation::add_delegation(
				get_origin(creator.clone()),
				operation.delegation_id,
				operation.parent_id,
				delegate.clone(),
				operation.permissions,
				operation.delegate_signature.clone().encode(),
			),
			delegation::Error::<Test>::InvalidDelegateSignature
		);
	});
}

#[test]
fn duplicate_delegation_create_delegation_error() {
	let creator_keypair = get_alice_ed25519();
	let creator = get_ed25519_account(creator_keypair.public());
	let delegate_keypair = get_bob_sr25519();
	let delegate = get_sr25519_account(delegate_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id(true),
		generate_base_delegation_hierarchy_details(),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(hierarchy_root_id, delegate.clone(), Some(hierarchy_root_id)),
	);

	let delegate_signature = delegate_keypair.sign(&hash_to_u8(Delegation::calculate_delegation_creation_hash(
		&delegation_id,
		&hierarchy_root_id,
		&hierarchy_root_id,
		&delegation_node.details.permissions,
	)));

	let operation =
		generate_base_delegation_creation_operation(delegation_id, delegate_signature.into(), delegation_node.clone());

	let ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, creator.clone())])
		.build(None);
	let mut ext = ExtBuilder::default()
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, creator.clone())])
		.with_delegations(vec![(delegation_id, delegation_node)])
		.build(Some(ext));

	ext.execute_with(|| {
		assert_noop!(
			Delegation::add_delegation(
				get_origin(creator.clone()),
				operation.delegation_id,
				operation.parent_id,
				delegate.clone(),
				operation.permissions,
				operation.delegate_signature.clone().encode(),
			),
			delegation::Error::<Test>::DelegationAlreadyExists
		);
	});
}

#[test]
fn parent_not_existing_create_delegation_error() {
	let creator_keypair = get_alice_ed25519();
	let creator = get_ed25519_account(creator_keypair.public());
	let delegate_keypair = get_bob_sr25519();
	let delegate = get_sr25519_account(delegate_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id(true),
		generate_base_delegation_hierarchy_details(),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(hierarchy_root_id, delegate.clone(), Some(hierarchy_root_id)),
	);

	let delegate_signature = delegate_keypair.sign(&hash_to_u8(Delegation::calculate_delegation_creation_hash(
		&delegation_id,
		&hierarchy_root_id,
		&hierarchy_root_id,
		&delegation_node.details.permissions,
	)));

	let operation =
		generate_base_delegation_creation_operation(delegation_id, delegate_signature.into(), delegation_node);

	let ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, creator.clone())])
		.build(None);
	// No delegations added to the pallet storage
	let mut ext = ExtBuilder::default().build(Some(ext));

	ext.execute_with(|| {
		assert_noop!(
			Delegation::add_delegation(
				get_origin(creator.clone()),
				operation.delegation_id,
				operation.parent_id,
				delegate.clone(),
				operation.permissions,
				operation.delegate_signature.clone().encode(),
			),
			delegation::Error::<Test>::ParentDelegationNotFound
		);
	});
}

#[test]
fn not_owner_of_parent_create_delegation_error() {
	let creator_keypair = get_alice_ed25519();
	let creator = get_ed25519_account(creator_keypair.public());
	let alternative_owner_keypair = get_charlie_ed25519();
	let alternative_owner = get_ed25519_account(alternative_owner_keypair.public());
	let delegate_keypair = get_bob_sr25519();
	let delegate = get_sr25519_account(delegate_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id(true),
		generate_base_delegation_hierarchy_details(),
	);
	let (parent_id, parent_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(hierarchy_root_id, alternative_owner, Some(hierarchy_root_id)),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(hierarchy_root_id, delegate.clone(), Some(parent_id)),
	);

	let delegate_signature = delegate_keypair.sign(&hash_to_u8(Delegation::calculate_delegation_creation_hash(
		&delegation_id,
		&hierarchy_root_id,
		&parent_id,
		&delegation_node.details.permissions,
	)));

	let operation =
		generate_base_delegation_creation_operation(delegation_id, delegate_signature.into(), delegation_node);

	let ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, creator.clone())])
		.build(None);
	let mut ext = ExtBuilder::default()
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, creator.clone())])
		.with_delegations(vec![(parent_id, parent_node)])
		.build(Some(ext));

	ext.execute_with(|| {
		assert_noop!(
			Delegation::add_delegation(
				get_origin(creator.clone()),
				operation.delegation_id,
				operation.parent_id,
				delegate.clone(),
				operation.permissions,
				operation.delegate_signature.clone().encode(),
			),
			delegation::Error::<Test>::NotOwnerOfParentDelegation
		);
	});
}

#[test]
fn unauthorised_delegation_create_delegation_error() {
	let creator_keypair = get_alice_ed25519();
	let creator = get_ed25519_account(creator_keypair.public());
	let delegate_keypair = get_bob_sr25519();
	let delegate = get_sr25519_account(delegate_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id(true),
		generate_base_delegation_hierarchy_details(),
	);
	let (parent_id, mut parent_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(hierarchy_root_id, creator.clone(), Some(hierarchy_root_id)),
	);
	parent_node.details.permissions = delegation::Permissions::ATTEST;
	let (delegation_id, delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(hierarchy_root_id, delegate.clone(), Some(parent_id)),
	);

	let delegate_signature = delegate_keypair.sign(&hash_to_u8(Delegation::calculate_delegation_creation_hash(
		&delegation_id,
		&hierarchy_root_id,
		&parent_id,
		&delegation_node.details.permissions,
	)));

	let operation =
		generate_base_delegation_creation_operation(delegation_id, delegate_signature.into(), delegation_node);

	let ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, creator.clone())])
		.build(None);
	let mut ext = ExtBuilder::default()
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, creator.clone())])
		.with_delegations(vec![(parent_id, parent_node)])
		.build(Some(ext));

	ext.execute_with(|| {
		assert_noop!(
			Delegation::add_delegation(
				get_origin(creator.clone()),
				operation.delegation_id,
				operation.parent_id,
				delegate.clone(),
				operation.permissions,
				operation.delegate_signature.clone().encode(),
			),
			delegation::Error::<Test>::UnauthorizedDelegation
		);
	});
}

// submit_delegation_root_revocation_operation()

#[test]
fn empty_revoke_root_successful() {
	let revoker_keypair = get_alice_ed25519();
	let revoker = get_ed25519_account(revoker_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id(true),
		generate_base_delegation_hierarchy_details(),
	);

	let operation = generate_base_delegation_hierarchy_revocation_operation(hierarchy_root_id);

	let ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, revoker.clone())])
		.build(None);
	let mut ext = ExtBuilder::default()
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, revoker.clone())])
		.build(Some(ext));

	ext.execute_with(|| {
		assert_ok!(Delegation::revoke_delegation(
			get_origin(revoker.clone()),
			operation.id,
			0u32,
			operation.max_children
		));
	});

	let stored_delegation_hierarchy_root = ext.execute_with(|| {
		Delegation::delegation_nodes(&operation.id).expect("Delegation root should be present on chain.")
	});

	assert!(stored_delegation_hierarchy_root.details.revoked);
}

#[test]
fn list_hierarchy_revoke_root_successful() {
	let revoker_keypair = get_alice_ed25519();
	let revoker = get_ed25519_account(revoker_keypair.public());
	let delegate_keypair = get_bob_sr25519();
	let delegate = get_sr25519_account(delegate_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id(true),
		generate_base_delegation_hierarchy_details(),
	);
	let (parent_id, parent_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(hierarchy_root_id, revoker.clone(), Some(hierarchy_root_id)),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(hierarchy_root_id, delegate, Some(parent_id)),
	);

	let mut operation = generate_base_delegation_hierarchy_revocation_operation(hierarchy_root_id);
	operation.max_children = 2u32;

	let ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, revoker.clone())])
		.build(None);
	let mut ext = ExtBuilder::default()
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, revoker.clone())])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.build(Some(ext));

	ext.execute_with(|| {
		assert_ok!(Delegation::revoke_delegation(
			get_origin(revoker.clone()),
			operation.id,
			0u32,
			operation.max_children
		));
	});

	let stored_delegation_hierarchy_root = ext.execute_with(|| {
		Delegation::delegation_nodes(&operation.id).expect("Delegation root should be present on chain.")
	});
	assert!(stored_delegation_hierarchy_root.details.revoked);

	let stored_parent_delegation = ext.execute_with(|| {
		Delegation::delegation_nodes(&parent_id).expect("Parent delegation should be present on chain.")
	});
	assert!(stored_parent_delegation.details.revoked);

	let stored_delegation = ext
		.execute_with(|| Delegation::delegation_nodes(&delegation_id).expect("Delegation should be present on chain."));
	assert!(stored_delegation.details.revoked);
}

#[test]
fn tree_hierarchy_revoke_root_successful() {
	let revoker_keypair = get_alice_ed25519();
	let revoker = get_ed25519_account(revoker_keypair.public());
	let delegate_keypair = get_bob_sr25519();
	let delegate = get_sr25519_account(delegate_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id(true),
		generate_base_delegation_hierarchy_details(),
	);
	let (delegation1_id, delegation1_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(hierarchy_root_id, revoker.clone(), Some(hierarchy_root_id)),
	);
	let (delegation2_id, delegation2_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(hierarchy_root_id, delegate, Some(hierarchy_root_id)),
	);

	let mut operation = generate_base_delegation_hierarchy_revocation_operation(hierarchy_root_id);
	operation.max_children = 2u32;

	let ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, revoker.clone())])
		.build(None);
	let mut ext = ExtBuilder::default()
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, revoker.clone())])
		.with_delegations(vec![
			(delegation1_id, delegation1_node),
			(delegation2_id, delegation2_node),
		])
		.build(Some(ext));

	ext.execute_with(|| {
		assert_ok!(Delegation::revoke_delegation(
			get_origin(revoker.clone()),
			operation.id,
			0u32,
			operation.max_children
		));
	});

	let stored_delegation_hierarchy_root = ext.execute_with(|| {
		Delegation::delegation_nodes(&operation.id).expect("Delegation root should be present on chain.")
	});
	assert!(stored_delegation_hierarchy_root.details.revoked);

	let stored_delegation_1 = ext.execute_with(|| {
		Delegation::delegation_nodes(&delegation1_id).expect("Delegation 1 should be present on chain.")
	});
	assert!(stored_delegation_1.details.revoked);

	let stored_delegation_2 = ext.execute_with(|| {
		Delegation::delegation_nodes(&delegation2_id).expect("Delegation 2 should be present on chain.")
	});
	assert!(stored_delegation_2.details.revoked);
}

#[test]
fn max_max_revocations_revoke_successful() {
	let revoker_keypair = get_alice_ed25519();
	let revoker = get_ed25519_account(revoker_keypair.public());
	let delegate_keypair = get_bob_sr25519();
	let delegate = get_sr25519_account(delegate_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id(true),
		generate_base_delegation_hierarchy_details(),
	);
	let (parent_id, parent_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(hierarchy_root_id, revoker.clone(), Some(hierarchy_root_id)),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(hierarchy_root_id, delegate, Some(parent_id)),
	);

	let mut operation = generate_base_delegation_hierarchy_revocation_operation(hierarchy_root_id);
	operation.max_children = MaxRevocations::get();

	let ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, revoker.clone())])
		.build(None);
	let mut ext = ExtBuilder::default()
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, revoker.clone())])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.build(Some(ext));

	ext.execute_with(|| {
		assert_ok!(Delegation::revoke_delegation(
			get_origin(revoker.clone()),
			operation.id,
			0u32,
			operation.max_children
		));
	});

	let stored_delegation_hierarchy_root = ext.execute_with(|| {
		Delegation::delegation_nodes(&operation.id).expect("Delegation root should be present on chain.")
	});
	assert!(stored_delegation_hierarchy_root.details.revoked);

	let stored_parent_delegation = ext.execute_with(|| {
		Delegation::delegation_nodes(&parent_id).expect("Parent delegation should be present on chain.")
	});
	assert!(stored_parent_delegation.details.revoked);

	let stored_delegation = ext
		.execute_with(|| Delegation::delegation_nodes(&delegation_id).expect("Delegation should be present on chain."));
	assert!(stored_delegation.details.revoked);
}

#[test]
fn root_not_found_revoke_root_error() {
	let revoker_keypair = get_alice_ed25519();
	let revoker = get_ed25519_account(revoker_keypair.public());

	let hierarchy_root_id = get_delegation_hierarchy_id(true);

	let operation = generate_base_delegation_hierarchy_revocation_operation(hierarchy_root_id);

	let mut ext = ExtBuilder::default().build(None);

	ext.execute_with(|| {
		assert_noop!(
			Delegation::revoke_delegation(get_origin(revoker.clone()), operation.id, 0u32, operation.max_children),
			delegation::Error::<Test>::DelegationNotFound
		);
	});
}

#[test]
fn different_root_creator_revoke_root_error() {
	let revoker_keypair = get_alice_ed25519();
	let revoker = get_ed25519_account(revoker_keypair.public());
	let alternative_revoker_keypair = get_charlie_ed25519();
	let alternative_revoker = get_ed25519_account(alternative_revoker_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id(true),
		generate_base_delegation_hierarchy_details(),
	);

	let operation = generate_base_delegation_hierarchy_revocation_operation(hierarchy_root_id);

	let ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, revoker.clone())])
		.build(None);
	let mut ext = ExtBuilder::default()
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, alternative_revoker)])
		.build(Some(ext));

	ext.execute_with(|| {
		assert_noop!(
			Delegation::revoke_delegation(get_origin(revoker.clone()), operation.id, 0u32, operation.max_children),
			delegation::Error::<Test>::UnauthorizedRevocation
		);
	});
}

#[test]
fn too_small_max_revocations_revoke_root_error() {
	let revoker_keypair = get_alice_ed25519();
	let revoker = get_ed25519_account(revoker_keypair.public());
	let delegate_keypair = get_alice_ed25519();
	let delegate = get_ed25519_account(delegate_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id(true),
		generate_base_delegation_hierarchy_details(),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(hierarchy_root_id, delegate, Some(hierarchy_root_id)),
	);

	let mut operation = generate_base_delegation_hierarchy_revocation_operation(hierarchy_root_id);
	operation.max_children = 0u32;

	let ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, revoker.clone())])
		.build(None);
	let mut ext = ExtBuilder::default()
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, revoker.clone())])
		.with_delegations(vec![(delegation_id, delegation_node)])
		.build(Some(ext));

	ext.execute_with(|| {
		assert_noop!(
			Delegation::revoke_delegation(get_origin(revoker.clone()), operation.id, 0u32, operation.max_children),
			delegation::Error::<Test>::ExceededRevocationBounds
		);
	});
}

#[test]
fn exact_children_max_revocations_revoke_root_error() {
	let revoker_keypair = get_alice_ed25519();
	let revoker = get_ed25519_account(revoker_keypair.public());
	let delegate_keypair = get_alice_ed25519();
	let delegate = get_ed25519_account(delegate_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id(true),
		generate_base_delegation_hierarchy_details(),
	);
	let (delegation1_id, delegation1_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(hierarchy_root_id, delegate.clone(), Some(hierarchy_root_id)),
	);
	let (delegation2_id, delegation2_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(hierarchy_root_id, delegate.clone(), Some(delegation1_id)),
	);
	let (delegation3_id, delegation3_node) = (
		get_delegation_id_2(true),
		generate_base_delegation_node(hierarchy_root_id, delegate, Some(delegation1_id)),
	);

	let mut operation = generate_base_delegation_hierarchy_revocation_operation(hierarchy_root_id);
	operation.max_children = 2u32;

	let ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, revoker.clone())])
		.build(None);
	let mut ext = ExtBuilder::default()
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, revoker.clone())])
		.with_delegations(vec![
			(delegation1_id, delegation1_node),
			(delegation2_id, delegation2_node),
			(delegation3_id, delegation3_node),
		])
		.build(Some(ext));

	ext.execute_with(|| {
		// assert_err and not asser_noop becase the storage is indeed changed, even tho
		// partially
		assert_err!(
			Delegation::revoke_delegation(get_origin(revoker.clone()), operation.id, 0u32, operation.max_children),
			delegation::Error::<Test>::ExceededRevocationBounds
		);
	});

	let stored_delegation_root = ext.execute_with(|| {
		Delegation::delegation_nodes(&operation.id).expect("Delegation root should be present on chain.")
	});
	assert!(!stored_delegation_root.details.revoked);

	let stored_delegation_1 = ext.execute_with(|| {
		Delegation::delegation_nodes(&delegation1_id).expect("Delegation 1 should be present on chain.")
	});
	assert!(!stored_delegation_1.details.revoked);

	// Only this leaf should have been revoked as it is the first child of
	// delegation_1
	let stored_delegation_2 = ext.execute_with(|| {
		Delegation::delegation_nodes(&delegation2_id).expect("Delegation 2 should be present on chain.")
	});
	assert!(stored_delegation_2.details.revoked);

	let stored_delegation_3 = ext.execute_with(|| {
		Delegation::delegation_nodes(&delegation3_id).expect("Delegation 3 should be present on chain.")
	});
	assert!(!stored_delegation_3.details.revoked);
}

// submit_delegation_revocation_operation()

#[test]
fn direct_owner_revoke_delegation_successful() {
	let revoker_keypair = get_alice_ed25519();
	let revoker = get_ed25519_account(revoker_keypair.public());
	let delegate_keypair = get_alice_ed25519();
	let delegate = get_ed25519_account(delegate_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id(true),
		generate_base_delegation_hierarchy_details(),
	);
	let (parent_id, parent_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(hierarchy_root_id, revoker.clone(), Some(hierarchy_root_id)),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(hierarchy_root_id, delegate, Some(parent_id)),
	);

	let mut operation = generate_base_delegation_revocation_operation(parent_id);
	operation.max_revocations = 2u32;

	let ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, revoker.clone())])
		.build(None);
	let mut ext = ExtBuilder::default()
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, revoker.clone())])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.build(Some(ext));

	ext.execute_with(|| {
		assert_ok!(Delegation::revoke_delegation(
			get_origin(revoker.clone()),
			operation.delegation_id,
			operation.max_parent_checks,
			operation.max_revocations
		));
	});

	let stored_parent_delegation = ext.execute_with(|| {
		Delegation::delegation_nodes(&parent_id).expect("Parent delegation should be present on chain.")
	});
	assert!(stored_parent_delegation.details.revoked);

	let stored_child_delegation = ext.execute_with(|| {
		Delegation::delegation_nodes(&delegation_id).expect("Child delegation should be present on chain.")
	});
	assert!(stored_child_delegation.details.revoked);
}

#[test]
fn parent_owner_revoke_delegation_successful() {
	let revoker_keypair = get_alice_ed25519();
	let revoker = get_ed25519_account(revoker_keypair.public());
	let delegate_keypair = get_alice_ed25519();
	let delegate = get_ed25519_account(delegate_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id(true),
		generate_base_delegation_hierarchy_details(),
	);
	let (parent_id, parent_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(hierarchy_root_id, revoker.clone(), Some(hierarchy_root_id)),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(hierarchy_root_id, delegate, Some(parent_id)),
	);

	let mut operation = generate_base_delegation_revocation_operation(delegation_id);
	operation.max_parent_checks = 1u32;
	operation.max_revocations = 1u32;

	let ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, revoker.clone())])
		.build(None);
	let mut ext = ExtBuilder::default()
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, revoker.clone())])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.build(Some(ext));

	ext.execute_with(|| {
		assert_ok!(Delegation::revoke_delegation(
			get_origin(revoker.clone()),
			operation.delegation_id,
			operation.max_parent_checks,
			operation.max_revocations
		));
	});

	let stored_parent_delegation = ext.execute_with(|| {
		Delegation::delegation_nodes(&parent_id).expect("Parent delegation should be present on chain.")
	});
	assert!(!stored_parent_delegation.details.revoked);

	let stored_child_delegation = ext.execute_with(|| {
		Delegation::delegation_nodes(&delegation_id).expect("Child delegation should be present on chain.")
	});
	assert!(stored_child_delegation.details.revoked);
}

#[test]
fn delegation_not_found_revoke_delegation_error() {
	let revoker_keypair = get_alice_ed25519();
	let revoker = get_ed25519_account(revoker_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id(true),
		generate_base_delegation_hierarchy_details(),
	);
	let delegation_id = get_delegation_id(false);

	let operation = generate_base_delegation_revocation_operation(delegation_id);

	let ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, revoker.clone())])
		.build(None);
	let mut ext = ExtBuilder::default()
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, revoker.clone())])
		.build(Some(ext));

	ext.execute_with(|| {
		assert_noop!(
			Delegation::revoke_delegation(
				get_origin(revoker.clone()),
				operation.delegation_id,
				operation.max_parent_checks,
				operation.max_revocations
			),
			delegation::Error::<Test>::DelegationNotFound
		);
	});
}

#[test]
fn not_delegating_revoke_delegation_error() {
	let owner_keypair = get_alice_ed25519();
	let owner = get_ed25519_account(owner_keypair.public());
	let revoker_keypair = get_bob_ed25519();
	let revoker = get_ed25519_account(revoker_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id(true),
		generate_base_delegation_hierarchy_details(),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(hierarchy_root_id, owner.clone(), Some(hierarchy_root_id)),
	);

	let mut operation = generate_base_delegation_revocation_operation(delegation_id);
	operation.max_parent_checks = MaxParentChecks::get();

	let ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, revoker.clone())])
		.build(None);
	let mut ext = ExtBuilder::default()
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, owner)])
		.with_delegations(vec![(delegation_id, delegation_node)])
		.build(Some(ext));

	ext.execute_with(|| {
		assert_noop!(
			Delegation::revoke_delegation(
				get_origin(revoker.clone()),
				operation.delegation_id,
				operation.max_parent_checks,
				operation.max_revocations
			),
			delegation::Error::<Test>::UnauthorizedRevocation
		);
	});
}

#[test]
fn parent_too_far_revoke_delegation_error() {
	let owner_keypair = get_alice_ed25519();
	let owner = get_ed25519_account(owner_keypair.public());
	let intermediate_keypair = get_charlie_ed25519();
	let intermediate = get_ed25519_account(intermediate_keypair.public());
	let delegate_keypair = get_bob_ed25519();
	let delegate = get_ed25519_account(delegate_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id(true),
		generate_base_delegation_hierarchy_details(),
	);
	let (parent_id, parent_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(hierarchy_root_id, intermediate.clone(), Some(hierarchy_root_id)),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(hierarchy_root_id, delegate, Some(parent_id)),
	);

	let mut operation = generate_base_delegation_revocation_operation(delegation_id);
	operation.max_parent_checks = 0u32;

	let ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, owner.clone())])
		.build(None);
	let mut ext = ExtBuilder::default()
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, owner)])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.build(Some(ext));

	ext.execute_with(|| {
		assert_noop!(
			Delegation::revoke_delegation(
				get_origin(intermediate.clone()),
				operation.delegation_id,
				operation.max_parent_checks,
				operation.max_revocations
			),
			delegation::Error::<Test>::MaxSearchDepthReached
		);
	});
}

#[test]
fn too_many_revocations_revoke_delegation_error() {
	let revoker_keypair = get_alice_ed25519();
	let revoker = get_ed25519_account(revoker_keypair.public());
	let delegate_keypair = get_bob_ed25519();
	let delegate = get_ed25519_account(delegate_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id(true),
		generate_base_delegation_hierarchy_details(),
	);
	let (parent_id, parent_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(hierarchy_root_id, revoker.clone(), Some(hierarchy_root_id)),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(hierarchy_root_id, delegate, Some(parent_id)),
	);

	let operation = generate_base_delegation_revocation_operation(parent_id);

	let ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, revoker.clone())])
		.build(None);
	let mut ext = ExtBuilder::default()
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, revoker.clone())])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.build(Some(ext));

	ext.execute_with(|| {
		assert_noop!(
			Delegation::revoke_delegation(
				get_origin(revoker.clone()),
				operation.delegation_id,
				operation.max_parent_checks,
				operation.max_revocations
			),
			delegation::Error::<Test>::ExceededRevocationBounds
		);
	});
}

// Internal function: is_delegating()

#[test]
fn is_delegating_direct_not_revoked() {
	let user_1_keypair = get_alice_ed25519();
	let user_1 = get_ed25519_account(user_1_keypair.public());
	let user_2_keypair = get_bob_ed25519();
	let user_2 = get_ed25519_account(user_2_keypair.public());
	let user_3_keypair = get_charlie_ed25519();
	let user_3 = get_ed25519_account(user_3_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id(true),
		generate_base_delegation_hierarchy_details(),
	);
	let (parent_id, parent_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(hierarchy_root_id, user_2, Some(hierarchy_root_id)),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(hierarchy_root_id, user_3.clone(), Some(parent_id)),
	);

	let max_parent_checks = 0u32;

	// Root -> Parent -> Delegation
	let ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, user_1.clone())])
		.build(None);
	let mut ext = ExtBuilder::default()
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, user_1)])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.build(Some(ext));

	ext.execute_with(|| {
		assert_eq!(
			Delegation::is_delegating(&user_3, &delegation_id, max_parent_checks),
			Ok((true, max_parent_checks))
		);
	});
}

#[test]
fn is_delegating_direct_not_revoked_max_parent_checks_value() {
	let user_1_keypair = get_alice_ed25519();
	let user_1 = get_ed25519_account(user_1_keypair.public());
	let user_2_keypair = get_bob_ed25519();
	let user_2 = get_ed25519_account(user_2_keypair.public());
	let user_3_keypair = get_charlie_ed25519();
	let user_3 = get_ed25519_account(user_3_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id(true),
		generate_base_delegation_hierarchy_details(),
	);
	let (parent_id, parent_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(hierarchy_root_id, user_2, Some(hierarchy_root_id)),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(hierarchy_root_id, user_3.clone(), Some(parent_id)),
	);

	let max_parent_checks = u32::MAX;

	// Root -> Parent -> Delegation
	let ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, user_1.clone())])
		.build(None);
	let mut ext = ExtBuilder::default()
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, user_1)])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.build(Some(ext));

	ext.execute_with(|| {
		assert_eq!(
			Delegation::is_delegating(&user_3, &delegation_id, max_parent_checks),
			Ok((true, 0u32))
		);
	});
}

#[test]
fn is_delegating_direct_revoked() {
	let user_1_keypair = get_alice_ed25519();
	let user_1 = get_ed25519_account(user_1_keypair.public());
	let user_2_keypair = get_bob_ed25519();
	let user_2 = get_ed25519_account(user_2_keypair.public());
	let user_3_keypair = get_charlie_ed25519();
	let user_3 = get_ed25519_account(user_3_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id(true),
		generate_base_delegation_hierarchy_details(),
	);
	let (parent_id, parent_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(hierarchy_root_id, user_2, Some(hierarchy_root_id)),
	);
	let (delegation_id, mut delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(hierarchy_root_id, user_3.clone(), Some(parent_id)),
	);
	delegation_node.details.revoked = true;

	let max_parent_checks = 0u32;

	// Root -> Parent -> Delegation
	let ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, user_1.clone())])
		.build(None);
	let mut ext = ExtBuilder::default()
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, user_1)])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.build(Some(ext));

	ext.execute_with(|| {
		assert_eq!(
			Delegation::is_delegating(&user_3, &delegation_id, max_parent_checks),
			Ok((false, 0))
		);
	});
}

#[test]
fn is_delegating_direct_revoked_max_parent_checks_value() {
	let user_1_keypair = get_alice_ed25519();
	let user_1 = get_ed25519_account(user_1_keypair.public());
	let user_2_keypair = get_bob_ed25519();
	let user_2 = get_ed25519_account(user_2_keypair.public());
	let user_3_keypair = get_charlie_ed25519();
	let user_3 = get_ed25519_account(user_3_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id(true),
		generate_base_delegation_hierarchy_details(),
	);
	let (parent_id, parent_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(hierarchy_root_id, user_2, Some(hierarchy_root_id)),
	);
	let (delegation_id, mut delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(hierarchy_root_id, user_3.clone(), Some(parent_id)),
	);
	delegation_node.details.revoked = true;

	let max_parent_checks = u32::MAX;

	// Root -> Parent -> Delegation
	let ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, user_1.clone())])
		.build(None);
	let mut ext = ExtBuilder::default()
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, user_1)])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.build(Some(ext));

	ext.execute_with(|| {
		assert_eq!(
			Delegation::is_delegating(&user_3, &delegation_id, max_parent_checks),
			Ok((false, 0))
		);
	});
}

#[test]
fn is_delegating_max_parent_not_revoked() {
	let user_1_keypair = get_alice_ed25519();
	let user_1 = get_ed25519_account(user_1_keypair.public());
	let user_2_keypair = get_bob_ed25519();
	let user_2 = get_ed25519_account(user_2_keypair.public());
	let user_3_keypair = get_charlie_ed25519();
	let user_3 = get_ed25519_account(user_3_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id(true),
		generate_base_delegation_hierarchy_details(),
	);
	let (parent_id, parent_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(hierarchy_root_id, user_2.clone(), Some(hierarchy_root_id)),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(hierarchy_root_id, user_3, Some(parent_id)),
	);

	let max_parent_checks = 1u32;

	// Root -> Parent -> Delegation
	let ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, user_1.clone())])
		.build(None);
	let mut ext = ExtBuilder::default()
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, user_1)])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.build(Some(ext));

	ext.execute_with(|| {
		assert_eq!(
			Delegation::is_delegating(&user_2, &delegation_id, max_parent_checks),
			Ok((true, max_parent_checks - 1))
		);
	});
}

#[test]
fn is_delegating_max_parent_revoked() {
	let user_1_keypair = get_alice_ed25519();
	let user_1 = get_ed25519_account(user_1_keypair.public());
	let user_2_keypair = get_bob_ed25519();
	let user_2 = get_ed25519_account(user_2_keypair.public());
	let user_3_keypair = get_charlie_ed25519();
	let user_3 = get_ed25519_account(user_3_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id(true),
		generate_base_delegation_hierarchy_details(),
	);
	let (parent_id, mut parent_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(hierarchy_root_id, user_2.clone(), Some(hierarchy_root_id)),
	);
	parent_node.details.revoked = true;
	let (delegation_id, delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(hierarchy_root_id, user_3, Some(parent_id)),
	);

	let max_parent_checks = 2u32;

	// Root -> Parent -> Delegation
	let ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, user_1.clone())])
		.build(None);
	let mut ext = ExtBuilder::default()
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, user_1)])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.build(Some(ext));

	ext.execute_with(|| {
		assert_eq!(
			Delegation::is_delegating(&user_2, &delegation_id, max_parent_checks),
			Ok((false, max_parent_checks - 2))
		);
	});
}

#[test]
fn is_delegating_root_owner_not_revoked() {
	let user_1_keypair = get_alice_ed25519();
	let user_1 = get_ed25519_account(user_1_keypair.public());
	let user_2_keypair = get_bob_ed25519();
	let user_2 = get_ed25519_account(user_2_keypair.public());
	let user_3_keypair = get_charlie_ed25519();
	let user_3 = get_ed25519_account(user_3_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id(true),
		generate_base_delegation_hierarchy_details(),
	);
	let (parent_id, parent_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(hierarchy_root_id, user_2, Some(hierarchy_root_id)),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(hierarchy_root_id, user_3, Some(parent_id)),
	);

	let max_parent_checks = 2u32;

	// Root -> Parent -> Delegation
	let ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, user_1.clone())])
		.build(None);
	let mut ext = ExtBuilder::default()
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, user_1.clone())])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.build(Some(ext));

	ext.execute_with(|| {
		assert_eq!(
			Delegation::is_delegating(&user_1, &delegation_id, max_parent_checks),
			Ok((true, max_parent_checks - 2))
		);
	});
}

#[test]
fn is_delegating_root_owner_revoked() {
	let user_1_keypair = get_alice_ed25519();
	let user_1 = get_ed25519_account(user_1_keypair.public());
	let user_2_keypair = get_bob_ed25519();
	let user_2 = get_ed25519_account(user_2_keypair.public());
	let user_3_keypair = get_charlie_ed25519();
	let user_3 = get_ed25519_account(user_3_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id(true),
		generate_base_delegation_hierarchy_details(),
	);
	let (parent_id, parent_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(hierarchy_root_id, user_2, Some(hierarchy_root_id)),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(hierarchy_root_id, user_3, Some(parent_id)),
	);

	let max_parent_checks = 2u32;

	// Root -> Parent -> Delegation
	let ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, user_1.clone())])
		.build(None);
	let mut ext = ExtBuilder::default()
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, user_1.clone())])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.build(Some(ext));

	ext.execute_with(|| {
		// First revoke the hierarchy, then test is_delegating.
		let _ = Delegation::revoke_delegation(get_origin(user_1.clone()), hierarchy_root_id, 0u32, 2);
		assert_eq!(
			Delegation::is_delegating(&user_1, &delegation_id, max_parent_checks),
			Ok((false, 0u32))
		);
	});
}

#[test]
fn is_delegating_delegation_not_found() {
	let user_1_keypair = get_alice_ed25519();
	let user_1 = get_ed25519_account(user_1_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id(true),
		generate_base_delegation_hierarchy_details(),
	);
	let delegation_id = get_delegation_id(true);

	let max_parent_checks = 2u32;

	// Root -> Delegation 1
	let mut ext = ExtBuilder::default()
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, user_1.clone())])
		.build(None);

	ext.execute_with(|| {
		assert_noop!(
			Delegation::is_delegating(&user_1, &delegation_id, max_parent_checks),
			delegation::Error::<Test>::DelegationNotFound
		);
	});
}

#[test]
fn is_delegating_root_after_max_limit() {
	let user_1_keypair = get_alice_ed25519();
	let user_1 = get_ed25519_account(user_1_keypair.public());
	let user_2_keypair = get_bob_ed25519();
	let user_2 = get_ed25519_account(user_2_keypair.public());
	let user_3_keypair = get_charlie_ed25519();
	let user_3 = get_ed25519_account(user_3_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id(true),
		generate_base_delegation_hierarchy_details(),
	);
	let (parent_id, parent_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(hierarchy_root_id, user_2, Some(hierarchy_root_id)),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(hierarchy_root_id, user_3, Some(parent_id)),
	);

	// 1 less than needed
	let max_parent_checks = 1u32;

	// Root -> Parent -> Delegation
	let ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, user_1.clone())])
		.build(None);
	let mut ext = ExtBuilder::default()
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, user_1.clone())])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.build(Some(ext));

	ext.execute_with(|| {
		assert_noop!(
			Delegation::is_delegating(&user_1, &delegation_id, max_parent_checks),
			delegation::Error::<Test>::MaxSearchDepthReached
		);
	});
}
