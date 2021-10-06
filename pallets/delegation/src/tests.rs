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

use frame_support::{assert_err, assert_noop, assert_ok};
use sp_core::Pair;

use crate::{self as delegation, mock::*, Config, Error};
use sp_runtime::traits::Zero;

// submit_delegation_root_creation_operation()

#[test]
fn create_root_delegation_successful() {
	let creator_keypair = get_alice_ed25519();
	let creator = get_ed25519_account(creator_keypair.public());

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let operation = generate_base_delegation_hierarchy_creation_operation::<Test>(hierarchy_root_id);

	ExtBuilder::default()
		.with_ctypes(vec![(operation.ctype_hash, creator.clone())])
		.with_balances(vec![(creator.clone(), <Test as Config>::Deposit::get())])
		.build()
		.execute_with(|| {
			// Create root hierarchy
			assert_ok!(Delegation::create_hierarchy(
				get_origin(creator.clone()),
				operation.id,
				operation.ctype_hash
			));

			// Check reserved balance
			assert_eq!(
				Balances::reserved_balance(creator.clone()),
				<Test as Config>::Deposit::get()
			);

			// Get stored hierarchy
			let stored_hierarchy_details = Delegation::delegation_hierarchies(&hierarchy_root_id)
				.expect("Delegation hierarchy should be present on chain.");
			assert_eq!(stored_hierarchy_details.ctype_hash, operation.ctype_hash);

			// Check root delegation
			let stored_delegation_root =
				Delegation::delegation_nodes(&hierarchy_root_id).expect("Delegation root should be present on chain.");
			assert_eq!(stored_delegation_root.hierarchy_root_id, hierarchy_root_id);
			assert!(stored_delegation_root.parent.is_none());
			assert!(stored_delegation_root.children.len().is_zero());
			assert_eq!(stored_delegation_root.details.owner, creator.clone());
			assert_eq!(stored_delegation_root.deposit.owner, creator);
			assert!(!stored_delegation_root.details.revoked);
		});
}

#[test]
fn duplicate_create_root_delegation_error() {
	let creator_keypair = get_alice_ed25519();
	let creator = get_ed25519_account(creator_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id::<Test>(true),
		generate_base_delegation_hierarchy_details(),
	);

	let operation = generate_base_delegation_hierarchy_creation_operation::<Test>(hierarchy_root_id);

	ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, creator.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, creator.clone())])
		.with_balances(vec![(creator.clone(), <Test as Config>::Deposit::get())])
		.build()
		.execute_with(|| {
			assert_noop!(
				Delegation::create_hierarchy(get_origin(creator.clone()), operation.id, operation.ctype_hash),
				Error::<Test>::HierarchyAlreadyExists
			);
		});
}

#[test]
fn ctype_not_found_create_root_delegation_error() {
	let creator_keypair = get_alice_ed25519();
	let creator = get_ed25519_account(creator_keypair.public());

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);

	let operation = generate_base_delegation_hierarchy_creation_operation::<Test>(hierarchy_root_id);

	// No CType stored
	ExtBuilder::default().build().execute_with(|| {
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
		get_delegation_hierarchy_id::<Test>(true),
		generate_base_delegation_hierarchy_details(),
	);

	ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, creator.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, creator.clone())])
		.with_balances(vec![
			(creator.clone(), <Test as Config>::Deposit::get()),
			(delegate.clone(), <Test as Config>::Deposit::get()),
		])
		.build()
		.execute_with(|| {
			// Create delegation to root
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

			// 1 Deposit should be reserved for hierarchy
			assert_eq!(
				Balances::reserved_balance(creator.clone()),
				<Test as Config>::Deposit::get()
			);

			// Add delegation to root
			assert_ok!(Delegation::add_delegation(
				get_origin(creator.clone()),
				operation.delegation_id,
				operation.parent_id,
				operation.delegate.clone(),
				operation.permissions,
				operation.delegate_signature.clone(),
			));

			// 2 Deposits should be reserved for hierarchy and delegation to root
			assert_eq!(
				Balances::reserved_balance(creator.clone()),
				2 * <Test as Config>::Deposit::get()
			);

			// Check stored delegation against operation
			let stored_delegation =
				Delegation::delegation_nodes(&operation.delegation_id).expect("Delegation should be present on chain.");
			assert_eq!(stored_delegation.hierarchy_root_id, operation.hierarchy_id);
			assert_eq!(stored_delegation.parent, Some(operation.parent_id));
			assert!(stored_delegation.children.is_empty());
			assert_eq!(stored_delegation.details.owner, operation.delegate);
			assert_eq!(stored_delegation.details.permissions, operation.permissions);
			assert!(!stored_delegation.details.revoked);

			// Verify that the root has the new delegation among its children
			let stored_root = Delegation::delegation_nodes(&operation.hierarchy_id)
				.expect("Delegation root should be present on chain.");
			assert!(stored_root.children.contains(&operation.delegation_id));
		});
}

#[test]
fn create_delegation_with_parent_successful() {
	let creator_keypair = get_alice_ed25519();
	let creator = get_ed25519_account(creator_keypair.public());
	let delegate_keypair = get_bob_sr25519();
	let delegate = get_sr25519_account(delegate_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id::<Test>(true),
		generate_base_delegation_hierarchy_details(),
	);
	let (parent_id, parent_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(hierarchy_root_id, creator.clone(), Some(hierarchy_root_id)),
	);

	ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, creator.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, creator.clone())])
		.with_delegations(vec![(parent_id, parent_node)])
		.with_balances(vec![
			(creator.clone(), <Test as Config>::Deposit::get()),
			(delegate.clone(), <Test as Config>::Deposit::get()),
		])
		.build()
		.execute_with(|| {
			// Create sub-delegation
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

			// Should have deposited for hierarchy and parent delegation
			assert_eq!(
				Balances::reserved_balance(&creator),
				2 * <Test as Config>::Deposit::get()
			);

			// Add sub-delegation
			assert_ok!(Delegation::add_delegation(
				get_origin(creator.clone()),
				delegation_id,
				parent_id,
				operation.delegate.clone(),
				operation.permissions,
				operation.delegate_signature.clone(),
			));

			// Should have deposited for hierarchy, parent delegation and sub-delegation
			assert_eq!(
				Balances::reserved_balance(&creator),
				3 * <Test as Config>::Deposit::get()
			);
			assert!(Balances::reserved_balance(delegate).is_zero());

			// Data in stored delegation and operation should match
			let stored_delegation =
				Delegation::delegation_nodes(&operation.delegation_id).expect("Delegation should be present on chain.");
			assert_eq!(stored_delegation.hierarchy_root_id, operation.hierarchy_id);
			assert_eq!(stored_delegation.parent, Some(operation.parent_id));
			assert!(stored_delegation.children.is_empty());
			assert_eq!(stored_delegation.details.owner, operation.delegate);
			assert_eq!(stored_delegation.details.permissions, operation.permissions);
			assert!(!stored_delegation.details.revoked);
			assert_eq!(stored_delegation.deposit.owner, creator);

			// Verify that the parent has the new delegation among its children
			let stored_parent =
				Delegation::delegation_nodes(&operation.parent_id).expect("Delegation parent be present on chain.");

			assert!(stored_parent.children.contains(&operation.delegation_id));
		});
}

#[test]
fn create_delegation_direct_root_revoked_error() {
	let creator_keypair = get_alice_ed25519();
	let creator = get_ed25519_account(creator_keypair.public());
	let delegate_keypair = get_bob_sr25519();
	let delegate = get_sr25519_account(delegate_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id::<Test>(true),
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

	ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, creator.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, creator.clone())])
		.with_balances(vec![(creator.clone(), <Test as Config>::Deposit::get())])
		.build()
		.execute_with(|| {
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
					operation.delegate_signature.clone(),
				),
				Error::<Test>::ParentDelegationRevoked
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
		get_delegation_hierarchy_id::<Test>(true),
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

	ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, creator.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, creator.clone())])
		.with_delegations(vec![(parent_id, parent_node)])
		.with_balances(vec![
			(creator.clone(), <Test as Config>::Deposit::get()),
			(delegate.clone(), <Test as Config>::Deposit::get()),
		])
		.build()
		.execute_with(|| {
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
					operation.delegate_signature.clone(),
				),
				Error::<Test>::ParentDelegationRevoked
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
		get_delegation_hierarchy_id::<Test>(true),
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

	ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, creator.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, creator.clone())])
		.with_balances(vec![(creator.clone(), <Test as Config>::Deposit::get())])
		.build()
		.execute_with(|| {
			assert_noop!(
				Delegation::add_delegation(
					get_origin(creator.clone()),
					operation.delegation_id,
					operation.parent_id,
					delegate.clone(),
					operation.permissions,
					operation.delegate_signature.clone(),
				),
				Error::<Test>::InvalidDelegateSignature
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
		get_delegation_hierarchy_id::<Test>(true),
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

	ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, creator.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, creator.clone())])
		.with_delegations(vec![(delegation_id, delegation_node)])
		.with_balances(vec![
			(creator.clone(), <Test as Config>::Deposit::get()),
			(delegate.clone(), <Test as Config>::Deposit::get()),
		])
		.build()
		.execute_with(|| {
			assert_noop!(
				Delegation::add_delegation(
					get_origin(creator.clone()),
					operation.delegation_id,
					operation.parent_id,
					delegate.clone(),
					operation.permissions,
					operation.delegate_signature.clone(),
				),
				Error::<Test>::DelegationAlreadyExists
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
		get_delegation_hierarchy_id::<Test>(true),
		generate_base_delegation_hierarchy_details::<Test>(),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(true),
		generate_base_delegation_node::<Test>(hierarchy_root_id, delegate.clone(), Some(hierarchy_root_id)),
	);

	let delegate_signature = delegate_keypair.sign(&hash_to_u8(Delegation::calculate_delegation_creation_hash(
		&delegation_id,
		&hierarchy_root_id,
		&hierarchy_root_id,
		&delegation_node.details.permissions,
	)));

	let operation =
		generate_base_delegation_creation_operation(delegation_id, delegate_signature.into(), delegation_node);

	ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, creator.clone())])
		.with_balances(vec![(creator.clone(), <Test as Config>::Deposit::get())])
		.build()
		.execute_with(|| {
			assert_noop!(
				Delegation::add_delegation(
					get_origin(creator.clone()),
					operation.delegation_id,
					operation.parent_id,
					delegate.clone(),
					operation.permissions,
					operation.delegate_signature.clone(),
				),
				Error::<Test>::ParentDelegationNotFound
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
		get_delegation_hierarchy_id::<Test>(true),
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

	ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, creator.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, creator.clone())])
		.with_delegations(vec![(parent_id, parent_node)])
		.with_balances(vec![(creator.clone(), <Test as Config>::Deposit::get())])
		.build()
		.execute_with(|| {
			assert_noop!(
				Delegation::add_delegation(
					get_origin(creator.clone()),
					operation.delegation_id,
					operation.parent_id,
					delegate.clone(),
					operation.permissions,
					operation.delegate_signature.clone(),
				),
				Error::<Test>::NotOwnerOfParentDelegation
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
		get_delegation_hierarchy_id::<Test>(true),
		generate_base_delegation_hierarchy_details(),
	);
	let (parent_id, mut parent_node) = (
		get_delegation_id(true),
		generate_base_delegation_node::<Test>(hierarchy_root_id, creator.clone(), Some(hierarchy_root_id)),
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

	ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, creator.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, creator.clone())])
		.with_balances(vec![(creator.clone(), <Test as Config>::Deposit::get())])
		.with_delegations(vec![(parent_id, parent_node)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Delegation::add_delegation(
					get_origin(creator.clone()),
					operation.delegation_id,
					operation.parent_id,
					delegate.clone(),
					operation.permissions,
					operation.delegate_signature.clone(),
				),
				Error::<Test>::UnauthorizedDelegation
			);
		});
}

// submit_delegation_root_revocation_operation()

#[test]
fn empty_revoke_root_successful() {
	let revoker_keypair = get_alice_ed25519();
	let revoker = get_ed25519_account(revoker_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id::<Test>(true),
		generate_base_delegation_hierarchy_details(),
	);

	let operation = generate_base_delegation_hierarchy_revocation_operation(hierarchy_root_id);

	ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, revoker.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, revoker.clone())])
		.with_balances(vec![(revoker.clone(), <Test as Config>::Deposit::get())])
		.build()
		.execute_with(|| {
			assert_ok!(Delegation::revoke_delegation(
				get_origin(revoker.clone()),
				operation.id,
				0u32,
				operation.max_children
			));

			assert!(
				Delegation::delegation_nodes(&operation.id)
					.expect("Delegation root should be present on chain.")
					.details
					.revoked
			)
		});
}

#[test]
fn list_hierarchy_revoke_and_remove_root_successful() {
	let revoker_keypair = get_alice_ed25519();
	let revoker = get_ed25519_account(revoker_keypair.public());
	let delegate_keypair = get_bob_sr25519();
	let delegate = get_sr25519_account(delegate_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id::<Test>(true),
		generate_base_delegation_hierarchy_details(),
	);
	let (parent_id, parent_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(hierarchy_root_id, revoker.clone(), Some(hierarchy_root_id)),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(hierarchy_root_id, delegate.clone(), Some(parent_id)),
	);

	let mut operation = generate_base_delegation_hierarchy_revocation_operation(hierarchy_root_id);
	operation.max_children = 2u32;

	ExtBuilder::default()
		.with_balances(vec![
			(revoker.clone(), <Test as Config>::Deposit::get() * 2),
			(delegate.clone(), <Test as Config>::Deposit::get()),
		])
		.with_ctypes(vec![(hierarchy_details.ctype_hash, revoker.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, revoker.clone())])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.build()
		.execute_with(|| {
			assert!(Delegation::delegation_hierarchies(&hierarchy_root_id).is_some());
			assert_eq!(
				Balances::reserved_balance(revoker.clone()),
				2 * <Test as Config>::Deposit::get()
			);
			assert_eq!(
				Balances::reserved_balance(delegate.clone()),
				<Test as Config>::Deposit::get()
			);

			// Revoke root
			assert_ok!(Delegation::revoke_delegation(
				get_origin(revoker.clone()),
				operation.id,
				0u32,
				operation.max_children
			));

			// Root and children should still be stored with revoked status
			assert!(
				Delegation::delegation_nodes(&operation.id)
					.expect("Delegation root should be present on chain.")
					.details
					.revoked
			);
			assert!(
				Delegation::delegation_nodes(&parent_id)
					.expect("Parent delegation should be present on chain.")
					.details
					.revoked
			);
			assert!(
				Delegation::delegation_nodes(&delegation_id)
					.expect("Delegation should be present on chain.")
					.details
					.revoked
			);

			// Removing root should also remove children and hierarchy
			assert_ok!(Delegation::remove_delegation(
				get_origin(revoker.clone()),
				operation.id,
				operation.max_children
			));

			assert!(Delegation::delegation_nodes(&operation.id).is_none());
			assert!(Delegation::delegation_hierarchies(&hierarchy_root_id).is_none());
			assert!(Delegation::delegation_nodes(&parent_id).is_none());
			assert!(Delegation::delegation_nodes(&delegation_id).is_none());
			assert!(Balances::reserved_balance(revoker.clone()).is_zero());
			assert!(Balances::reserved_balance(delegate.clone()).is_zero());
		});
}

#[test]
fn tree_hierarchy_revoke_and_remove_root_successful() {
	let revoker_keypair = get_alice_ed25519();
	let revoker = get_ed25519_account(revoker_keypair.public());
	let delegate_keypair = get_bob_sr25519();
	let delegate = get_sr25519_account(delegate_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id::<Test>(true),
		generate_base_delegation_hierarchy_details(),
	);
	let (delegation1_id, delegation1_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(hierarchy_root_id, revoker.clone(), Some(hierarchy_root_id)),
	);
	let (delegation2_id, delegation2_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(hierarchy_root_id, delegate.clone(), Some(hierarchy_root_id)),
	);

	let mut operation = generate_base_delegation_hierarchy_revocation_operation(hierarchy_root_id);
	operation.max_children = 2u32;

	ExtBuilder::default()
		.with_balances(vec![
			(revoker.clone(), <Test as Config>::Deposit::get() * 2),
			(delegate.clone(), <Test as Config>::Deposit::get()),
		])
		.with_ctypes(vec![(hierarchy_details.ctype_hash, revoker.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, revoker.clone())])
		.with_delegations(vec![
			(delegation1_id, delegation1_node),
			(delegation2_id, delegation2_node),
		])
		.build()
		.execute_with(|| {
			// Revoke root
			assert_ok!(Delegation::revoke_delegation(
				get_origin(revoker.clone()),
				operation.id,
				0u32,
				operation.max_children
			));

			// Root and children should still be stored with revoked status
			assert!(
				Delegation::delegation_nodes(&operation.id)
					.expect("Delegation root should be present on chain.")
					.details
					.revoked
			);
			assert!(
				Delegation::delegation_nodes(&delegation1_id)
					.expect("Delegation 1 should be present on chain.")
					.details
					.revoked
			);
			assert!(
				Delegation::delegation_nodes(&delegation2_id)
					.expect("Delegation 2 should be present on chain.")
					.details
					.revoked
			);

			assert_eq!(
				Balances::reserved_balance(revoker.clone()),
				2 * <Test as Config>::Deposit::get()
			);
			assert_eq!(
				Balances::reserved_balance(delegate.clone()),
				<Test as Config>::Deposit::get()
			);

			// Removing root should also remove children and hierarchy
			assert_ok!(Delegation::remove_delegation(
				get_origin(revoker.clone()),
				operation.id,
				operation.max_children
			));

			assert!(Delegation::delegation_nodes(&operation.id).is_none());
			assert!(Delegation::delegation_hierarchies(&hierarchy_root_id).is_none());
			assert!(Delegation::delegation_nodes(&delegation1_id).is_none());
			assert!(Delegation::delegation_nodes(&delegation2_id).is_none());
			assert!(Balances::reserved_balance(revoker.clone()).is_zero());
			assert!(Balances::reserved_balance(delegate.clone()).is_zero());
		});
}

#[test]
fn max_max_revocations_revoke_and_remove_successful() {
	let revoker_keypair = get_alice_ed25519();
	let revoker = get_ed25519_account(revoker_keypair.public());
	let delegate_keypair = get_bob_sr25519();
	let delegate = get_sr25519_account(delegate_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id::<Test>(true),
		generate_base_delegation_hierarchy_details(),
	);
	let (parent_id, parent_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(hierarchy_root_id, revoker.clone(), Some(hierarchy_root_id)),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(hierarchy_root_id, delegate.clone(), Some(parent_id)),
	);

	let mut operation = generate_base_delegation_hierarchy_revocation_operation(hierarchy_root_id);
	operation.max_children = MaxRevocations::get();

	ExtBuilder::default()
		.with_balances(vec![
			(revoker.clone(), <Test as Config>::Deposit::get() * 2),
			(delegate.clone(), <Test as Config>::Deposit::get()),
		])
		.with_ctypes(vec![(hierarchy_details.ctype_hash, revoker.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, revoker.clone())])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.build()
		.execute_with(|| {
			// Revoke root
			assert_ok!(Delegation::revoke_delegation(
				get_origin(revoker.clone()),
				operation.id,
				0u32,
				operation.max_children
			));

			// Root and children should still be stored with revoked status
			assert!(
				Delegation::delegation_nodes(&operation.id)
					.expect("Delegation root should be present on chain.")
					.details
					.revoked
			);
			assert!(
				Delegation::delegation_nodes(&parent_id)
					.expect("Parent delegation should be present on chain.")
					.details
					.revoked
			);
			assert!(
				Delegation::delegation_nodes(&delegation_id)
					.expect("Delegation should be present on chain.")
					.details
					.revoked
			);

			assert_eq!(
				Balances::reserved_balance(revoker.clone()),
				2 * <Test as Config>::Deposit::get()
			);
			assert_eq!(
				Balances::reserved_balance(delegate.clone()),
				<Test as Config>::Deposit::get()
			);

			// Removing root should also remove children and hierarchy
			assert_ok!(Delegation::remove_delegation(
				get_origin(revoker.clone()),
				operation.id,
				operation.max_children
			));

			assert!(Delegation::delegation_nodes(&operation.id).is_none());
			assert!(Delegation::delegation_hierarchies(&hierarchy_root_id).is_none());
			assert!(Delegation::delegation_nodes(&parent_id).is_none());
			assert!(Delegation::delegation_nodes(&delegation_id).is_none());
			assert!(Balances::reserved_balance(revoker.clone()).is_zero());
			assert!(Balances::reserved_balance(delegate.clone()).is_zero());
		});
}

#[test]
fn root_not_found_revoke_and_remove_root_error() {
	let revoker_keypair = get_alice_ed25519();
	let revoker = get_ed25519_account(revoker_keypair.public());

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);

	let operation = generate_base_delegation_hierarchy_revocation_operation(hierarchy_root_id);

	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Delegation::revoke_delegation(get_origin(revoker.clone()), operation.id, 0u32, operation.max_children),
			Error::<Test>::DelegationNotFound
		);
		assert_noop!(
			Delegation::remove_delegation(get_origin(revoker.clone()), operation.id, operation.max_children),
			Error::<Test>::DelegationNotFound
		);
	});
}

#[test]
fn different_root_creator_revoke_and_remove_root_error() {
	let owner_keypair = get_alice_ed25519();
	let owner = get_ed25519_account(owner_keypair.public());
	let unauthorized_keypair = get_charlie_ed25519();
	let unauthorized = get_ed25519_account(unauthorized_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id::<Test>(true),
		generate_base_delegation_hierarchy_details(),
	);

	let operation = generate_base_delegation_hierarchy_revocation_operation(hierarchy_root_id);

	ExtBuilder::default()
		.with_balances(vec![
			(owner.clone(), <Test as Config>::Deposit::get()),
			(unauthorized.clone(), <Test as Config>::Deposit::get()),
		])
		.with_ctypes(vec![(hierarchy_details.ctype_hash, owner.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, owner)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Delegation::revoke_delegation(
					get_origin(unauthorized.clone()),
					operation.id,
					0u32,
					operation.max_children
				),
				Error::<Test>::UnauthorizedRevocation
			);
			assert_noop!(
				Delegation::remove_delegation(get_origin(unauthorized.clone()), operation.id, operation.max_children),
				Error::<Test>::UnauthorizedRemoval
			);
		});
}

#[test]
fn too_small_max_revocations_revoke_and_remove_root_error() {
	let revoker_keypair = get_alice_ed25519();
	let revoker = get_ed25519_account(revoker_keypair.public());
	let delegate_keypair = get_bob_ed25519();
	let delegate = get_ed25519_account(delegate_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id::<Test>(true),
		generate_base_delegation_hierarchy_details(),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(hierarchy_root_id, delegate.clone(), Some(hierarchy_root_id)),
	);

	let mut operation = generate_base_delegation_hierarchy_revocation_operation(hierarchy_root_id);
	operation.max_children = 0u32;

	ExtBuilder::default()
		.with_balances(vec![
			(revoker.clone(), <Test as Config>::Deposit::get() * 2),
			(delegate, <Test as Config>::Deposit::get()),
		])
		.with_ctypes(vec![(hierarchy_details.ctype_hash, revoker.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, revoker.clone())])
		.with_delegations(vec![(delegation_id, delegation_node)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Delegation::revoke_delegation(get_origin(revoker.clone()), operation.id, 0u32, operation.max_children),
				Error::<Test>::ExceededRevocationBounds
			);
			assert_noop!(
				Delegation::remove_delegation(get_origin(revoker.clone()), operation.id, operation.max_children),
				Error::<Test>::ExceededRemovalBounds
			);
		});
}

#[test]
fn exact_children_max_revocations_revoke_and_remove_root_error() {
	let revoker_keypair = get_alice_ed25519();
	let revoker = get_ed25519_account(revoker_keypair.public());
	let delegate_keypair = get_bob_ed25519();
	let delegate = get_ed25519_account(delegate_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id::<Test>(true),
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
		generate_base_delegation_node(hierarchy_root_id, delegate.clone(), Some(delegation1_id)),
	);

	let mut operation = generate_base_delegation_hierarchy_revocation_operation(hierarchy_root_id);
	// set max children below required minimum of 3 to revoke/remove entire tree
	operation.max_children = 2u32;

	ExtBuilder::default()
		.with_balances(vec![
			(revoker.clone(), <Test as Config>::Deposit::get()),
			(delegate.clone(), <Test as Config>::Deposit::get() * 3),
		])
		.with_ctypes(vec![(hierarchy_details.ctype_hash, revoker.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, revoker.clone())])
		.with_delegations(vec![
			(delegation1_id, delegation1_node),
			(delegation2_id, delegation2_node),
			(delegation3_id, delegation3_node),
		])
		.build()
		.execute_with(|| {
			// Should not revoke root because tree traversal steps are insufficient
			// assert_err and not assert_noop because the storage is indeed changed, but
			// only partially (only #2 is revoked)
			assert_err!(
				Delegation::revoke_delegation(get_origin(revoker.clone()), operation.id, 0u32, operation.max_children),
				Error::<Test>::ExceededRevocationBounds
			);

			// Only Delegation 2 should have been revoked
			assert!(
				!Delegation::delegation_nodes(&operation.id)
					.expect("Delegation root should be present on chain.")
					.details
					.revoked
			);
			assert!(
				!Delegation::delegation_nodes(&delegation1_id)
					.expect("Delegation 1 should be present on chain.")
					.details
					.revoked
			);
			assert!(
				Delegation::delegation_nodes(&delegation2_id)
					.expect("Delegation 2 should be present on chain.")
					.details
					.revoked
			);
			assert!(
				!Delegation::delegation_nodes(&delegation3_id)
					.expect("Delegation 3 should be present on chain.")
					.details
					.revoked
			);

			// Should not remove root because tree traversal steps are insufficient
			assert_eq!(
				Balances::reserved_balance(revoker.clone()),
				<Test as Config>::Deposit::get()
			);
			assert_eq!(
				Balances::reserved_balance(delegate.clone()),
				3 * <Test as Config>::Deposit::get()
			);
			// assert_err and not assert_noop because the storage is indeed changed, but
			// only partially (only #2 is removed)
			assert_err!(
				Delegation::remove_delegation(get_origin(revoker.clone()), operation.id, operation.max_children),
				Error::<Test>::ExceededRemovalBounds
			);
			assert!(Delegation::delegation_nodes(&operation.id).is_some());
			assert!(Delegation::delegation_nodes(&delegation1_id).is_some());
			assert!(Delegation::delegation_nodes(&delegation2_id).is_none());
			assert!(Delegation::delegation_nodes(&delegation3_id).is_some());
			assert_eq!(
				Balances::reserved_balance(revoker.clone()),
				<Test as Config>::Deposit::get()
			);
			assert_eq!(
				Balances::reserved_balance(delegate.clone()),
				2 * <Test as Config>::Deposit::get()
			);

			// Should be able to remove root now (because # of remaining children = 2)
			assert_ok!(Delegation::remove_delegation(
				get_origin(revoker.clone()),
				operation.id,
				operation.max_children
			),);
			assert!(Delegation::delegation_nodes(&operation.id).is_none());
			assert!(Delegation::delegation_nodes(&delegation1_id).is_none());
			assert!(Delegation::delegation_nodes(&delegation3_id).is_none());
			assert!(Balances::reserved_balance(revoker.clone()).is_zero());
			assert!(Balances::reserved_balance(delegate.clone()).is_zero());
		});
}

// submit_delegation_revocation_operation()

// difference to `max_max_revocations_revoke_and_remove_successful`: doesn't
// revoke hierarchy but direct child of hierarchy root
#[test]
fn direct_owner_revoke_and_remove_delegation_successful() {
	let revoker_keypair = get_alice_ed25519();
	let revoker = get_ed25519_account(revoker_keypair.public());
	let delegate_keypair = get_bob_ed25519();
	let delegate = get_ed25519_account(delegate_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id::<Test>(true),
		generate_base_delegation_hierarchy_details(),
	);
	let (parent_id, parent_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(hierarchy_root_id, revoker.clone(), Some(hierarchy_root_id)),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(hierarchy_root_id, delegate.clone(), Some(parent_id)),
	);

	let mut operation = generate_base_delegation_revocation_operation(parent_id);
	operation.max_revocations = 2u32;

	ExtBuilder::default()
		.with_balances(vec![
			(revoker.clone(), <Test as Config>::Deposit::get() * 2),
			(delegate.clone(), <Test as Config>::Deposit::get()),
		])
		.with_ctypes(vec![(hierarchy_details.ctype_hash, revoker.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, revoker.clone())])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.build()
		.execute_with(|| {
			// Revoke direct child of hierarchy root
			assert_ok!(Delegation::revoke_delegation(
				get_origin(revoker.clone()),
				operation.delegation_id,
				operation.max_parent_checks,
				operation.max_revocations
			));

			// Root hierarchy and children should still be stored with revoked status
			assert!(
				Delegation::delegation_nodes(&operation.delegation_id)
					.expect("Delegation root should be present on chain.")
					.details
					.revoked
			);
			assert!(
				Delegation::delegation_nodes(&parent_id)
					.expect("Parent delegation should be present on chain.")
					.details
					.revoked
			);
			assert!(
				Delegation::delegation_nodes(&delegation_id)
					.expect("Delegation should be present on chain.")
					.details
					.revoked
			);

			assert_eq!(
				Balances::reserved_balance(revoker.clone()),
				2 * <Test as Config>::Deposit::get()
			);
			assert_eq!(
				Balances::reserved_balance(delegate.clone()),
				<Test as Config>::Deposit::get()
			);

			// Removing root delegation should also remove its child but not hierarchy root
			assert_ok!(Delegation::remove_delegation(
				get_origin(revoker.clone()),
				operation.delegation_id,
				operation.max_revocations
			));

			assert!(Delegation::delegation_hierarchies(&hierarchy_root_id).is_some());
			assert!(Delegation::delegation_nodes(&operation.delegation_id).is_none());
			assert!(Delegation::delegation_nodes(&parent_id).is_none());
			assert!(Delegation::delegation_nodes(&delegation_id).is_none());
			assert_eq!(
				Balances::reserved_balance(revoker.clone()),
				<Test as Config>::Deposit::get()
			);
			assert!(Balances::reserved_balance(delegate.clone()).is_zero());
		});
}

#[test]
fn parent_owner_revoke_delegation_successful() {
	let revoker_keypair = get_alice_ed25519();
	let revoker = get_ed25519_account(revoker_keypair.public());
	let delegate_keypair = get_bob_ed25519();
	let delegate = get_ed25519_account(delegate_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id::<Test>(true),
		generate_base_delegation_hierarchy_details(),
	);
	let (parent_id, parent_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(hierarchy_root_id, revoker.clone(), Some(hierarchy_root_id)),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(hierarchy_root_id, delegate.clone(), Some(parent_id)),
	);

	let mut operation = generate_base_delegation_revocation_operation(delegation_id);
	operation.max_parent_checks = 1u32;
	operation.max_revocations = 1u32;

	ExtBuilder::default()
		.with_balances(vec![
			(revoker.clone(), <Test as Config>::Deposit::get() * 2),
			(delegate.clone(), <Test as Config>::Deposit::get()),
		])
		.with_ctypes(vec![(hierarchy_details.ctype_hash, revoker.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, revoker.clone())])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.build()
		.execute_with(|| {
			// Parent should not be able to remove the child delegation directly
			assert_eq!(
				Balances::reserved_balance(revoker.clone()),
				2 * <Test as Config>::Deposit::get()
			);
			assert_eq!(
				Balances::reserved_balance(delegate.clone()),
				<Test as Config>::Deposit::get()
			);
			assert_noop!(
				Delegation::remove_delegation(
					get_origin(revoker.clone()),
					operation.delegation_id,
					operation.max_revocations
				),
				Error::<Test>::UnauthorizedRemoval
			);

			// Revoke direct child of hierarchy root
			assert_ok!(Delegation::revoke_delegation(
				get_origin(revoker.clone()),
				operation.delegation_id,
				operation.max_parent_checks,
				operation.max_revocations
			));

			// Only child should be revoked
			assert!(
				!Delegation::delegation_nodes(&parent_id)
					.expect("Parent delegation should be present on chain.")
					.details
					.revoked
			);
			assert!(
				Delegation::delegation_nodes(&delegation_id)
					.expect("Delegation should be present on chain.")
					.details
					.revoked
			);

			// Only owner can still remove the delegation to claim back deposit
			assert_ok!(Delegation::remove_delegation(
				get_origin(delegate.clone()),
				operation.delegation_id,
				operation.max_revocations
			));

			assert!(Delegation::delegation_nodes(&parent_id).is_some());
			assert!(Delegation::delegation_nodes(&delegation_id).is_none());
			assert_eq!(
				Balances::reserved_balance(revoker.clone()),
				2 * <Test as Config>::Deposit::get()
			);
			assert!(Balances::reserved_balance(delegate.clone()).is_zero());
		});
}

#[test]
fn delegation_not_found_revoke_and_remove_delegation_error() {
	let revoker_keypair = get_alice_ed25519();
	let revoker = get_ed25519_account(revoker_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id::<Test>(true),
		generate_base_delegation_hierarchy_details(),
	);
	let delegation_id = get_delegation_id(false);

	let operation = generate_base_delegation_revocation_operation(delegation_id);

	ExtBuilder::default()
		.with_balances(vec![(revoker.clone(), <Test as Config>::Deposit::get())])
		.with_ctypes(vec![(hierarchy_details.ctype_hash, revoker.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, revoker.clone())])
		.build()
		.execute_with(|| {
			assert_noop!(
				Delegation::revoke_delegation(
					get_origin(revoker.clone()),
					operation.delegation_id,
					operation.max_parent_checks,
					operation.max_revocations
				),
				Error::<Test>::DelegationNotFound
			);
			assert_noop!(
				Delegation::remove_delegation(
					get_origin(revoker.clone()),
					operation.delegation_id,
					operation.max_revocations
				),
				Error::<Test>::DelegationNotFound
			);
		});
}

#[test]
fn not_delegating_revoke_and_remove_delegation_error() {
	let owner_keypair = get_alice_ed25519();
	let owner = get_ed25519_account(owner_keypair.public());
	let revoker_keypair = get_bob_ed25519();
	let revoker = get_ed25519_account(revoker_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id::<Test>(true),
		generate_base_delegation_hierarchy_details(),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(hierarchy_root_id, owner.clone(), Some(hierarchy_root_id)),
	);

	let mut operation = generate_base_delegation_revocation_operation(delegation_id);
	operation.max_parent_checks = MaxParentChecks::get();

	ExtBuilder::default()
		.with_balances(vec![(owner.clone(), <Test as Config>::Deposit::get() * 2)])
		.with_ctypes(vec![(hierarchy_details.ctype_hash, owner.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, owner)])
		.with_delegations(vec![(delegation_id, delegation_node)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Delegation::revoke_delegation(
					get_origin(revoker.clone()),
					operation.delegation_id,
					operation.max_parent_checks,
					operation.max_revocations
				),
				Error::<Test>::UnauthorizedRevocation
			);
			assert_noop!(
				Delegation::remove_delegation(
					get_origin(revoker.clone()),
					operation.delegation_id,
					operation.max_revocations
				),
				Error::<Test>::UnauthorizedRemoval
			);
		});
}

#[test]
fn parent_too_far_revoke_and_remove_delegation_error() {
	let owner_keypair = get_alice_ed25519();
	let owner = get_ed25519_account(owner_keypair.public());
	let intermediate_keypair = get_charlie_ed25519();
	let intermediate = get_ed25519_account(intermediate_keypair.public());
	let delegate_keypair = get_bob_ed25519();
	let delegate = get_ed25519_account(delegate_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id::<Test>(true),
		generate_base_delegation_hierarchy_details(),
	);
	let (parent_id, parent_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(hierarchy_root_id, intermediate.clone(), Some(hierarchy_root_id)),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(hierarchy_root_id, delegate.clone(), Some(parent_id)),
	);

	let mut operation = generate_base_delegation_revocation_operation(delegation_id);
	operation.max_revocations = 2u32;
	operation.max_parent_checks = 0u32;

	ExtBuilder::default()
		.with_balances(vec![
			(owner.clone(), <Test as Config>::Deposit::get()),
			(intermediate.clone(), <Test as Config>::Deposit::get()),
			(delegate.clone(), <Test as Config>::Deposit::get()),
		])
		.with_ctypes(vec![(hierarchy_details.ctype_hash, owner.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, owner.clone())])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Delegation::revoke_delegation(
					get_origin(intermediate.clone()),
					operation.delegation_id,
					operation.max_parent_checks,
					operation.max_revocations
				),
				Error::<Test>::MaxSearchDepthReached
			);

			// removal
			assert_eq!(
				Balances::reserved_balance(owner.clone()),
				<Test as Config>::Deposit::get()
			);
			assert_eq!(
				Balances::reserved_balance(intermediate.clone()),
				<Test as Config>::Deposit::get()
			);
			assert_eq!(
				Balances::reserved_balance(delegate.clone()),
				<Test as Config>::Deposit::get()
			);

			assert_noop!(
				Delegation::remove_delegation(
					get_origin(owner.clone()),
					operation.delegation_id,
					operation.max_revocations
				),
				Error::<Test>::UnauthorizedRemoval
			);
			assert_noop!(
				Delegation::remove_delegation(
					get_origin(intermediate.clone()),
					operation.delegation_id,
					operation.max_revocations
				),
				Error::<Test>::UnauthorizedRemoval
			);
			assert_ok!(Delegation::remove_delegation(
				get_origin(delegate.clone()),
				operation.delegation_id,
				0u32
			));
		});
}

#[test]
fn too_many_revocations_revoke_delegation_error() {
	let revoker_keypair = get_alice_ed25519();
	let revoker = get_ed25519_account(revoker_keypair.public());
	let delegate_keypair = get_bob_ed25519();
	let delegate = get_ed25519_account(delegate_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id::<Test>(true),
		generate_base_delegation_hierarchy_details(),
	);
	let (parent_id, parent_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(hierarchy_root_id, revoker.clone(), Some(hierarchy_root_id)),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(hierarchy_root_id, delegate.clone(), Some(parent_id)),
	);

	let operation = generate_base_delegation_revocation_operation(parent_id);

	ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, revoker.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, revoker.clone())])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.with_balances(vec![
			(revoker.clone(), <Test as Config>::Deposit::get()),
			(delegate, <Test as Config>::Deposit::get()),
		])
		.build()
		.execute_with(|| {
			assert_noop!(
				Delegation::revoke_delegation(
					get_origin(revoker.clone()),
					operation.delegation_id,
					operation.max_parent_checks,
					operation.max_revocations
				),
				Error::<Test>::ExceededRevocationBounds
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
		get_delegation_hierarchy_id::<Test>(true),
		generate_base_delegation_hierarchy_details(),
	);
	let (parent_id, parent_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(hierarchy_root_id, user_2.clone(), Some(hierarchy_root_id)),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(hierarchy_root_id, user_3.clone(), Some(parent_id)),
	);

	let max_parent_checks = 0u32;

	// Root -> Parent -> Delegation
	// Root -> Parent -> Delegation
	ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, user_1.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, user_1.clone())])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.with_balances(vec![
			(user_1, <Test as Config>::Deposit::get()),
			(user_2, <Test as Config>::Deposit::get()),
			(user_3.clone(), <Test as Config>::Deposit::get()),
		])
		.build()
		.execute_with(|| {
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
		get_delegation_hierarchy_id::<Test>(true),
		generate_base_delegation_hierarchy_details(),
	);
	let (parent_id, parent_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(hierarchy_root_id, user_2.clone(), Some(hierarchy_root_id)),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(hierarchy_root_id, user_3.clone(), Some(parent_id)),
	);

	let max_parent_checks = u32::MAX;

	// Root -> Parent -> Delegation
	ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, user_1.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, user_1.clone())])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.with_balances(vec![
			(user_1, <Test as Config>::Deposit::get()),
			(user_2, <Test as Config>::Deposit::get()),
			(user_3.clone(), <Test as Config>::Deposit::get()),
		])
		.build()
		.execute_with(|| {
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
		get_delegation_hierarchy_id::<Test>(true),
		generate_base_delegation_hierarchy_details(),
	);
	let (parent_id, parent_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(hierarchy_root_id, user_2.clone(), Some(hierarchy_root_id)),
	);
	let (delegation_id, mut delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(hierarchy_root_id, user_3.clone(), Some(parent_id)),
	);
	delegation_node.details.revoked = true;

	let max_parent_checks = 0u32;

	// Root -> Parent -> Delegation
	// Root -> Parent -> Delegation
	ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, user_1.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, user_1.clone())])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.with_balances(vec![
			(user_1, <Test as Config>::Deposit::get()),
			(user_2, <Test as Config>::Deposit::get()),
			(user_3.clone(), <Test as Config>::Deposit::get()),
		])
		.build()
		.execute_with(|| {
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
		get_delegation_hierarchy_id::<Test>(true),
		generate_base_delegation_hierarchy_details(),
	);
	let (parent_id, parent_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(hierarchy_root_id, user_2.clone(), Some(hierarchy_root_id)),
	);
	let (delegation_id, mut delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(hierarchy_root_id, user_3.clone(), Some(parent_id)),
	);
	delegation_node.details.revoked = true;

	let max_parent_checks = u32::MAX;

	// Root -> Parent -> Delegation
	// Root -> Parent -> Delegation
	ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, user_1.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, user_1.clone())])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.with_balances(vec![
			(user_1, <Test as Config>::Deposit::get()),
			(user_2, <Test as Config>::Deposit::get()),
			(user_3.clone(), <Test as Config>::Deposit::get()),
		])
		.build()
		.execute_with(|| {
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
		get_delegation_hierarchy_id::<Test>(true),
		generate_base_delegation_hierarchy_details(),
	);
	let (parent_id, parent_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(hierarchy_root_id, user_2.clone(), Some(hierarchy_root_id)),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(hierarchy_root_id, user_3.clone(), Some(parent_id)),
	);

	let max_parent_checks = 1u32;

	// Root -> Parent -> Delegation
	// Root -> Parent -> Delegation
	ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, user_1.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, user_1.clone())])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.with_balances(vec![
			(user_1, <Test as Config>::Deposit::get()),
			(user_2.clone(), <Test as Config>::Deposit::get()),
			(user_3, <Test as Config>::Deposit::get()),
		])
		.build()
		.execute_with(|| {
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
		get_delegation_hierarchy_id::<Test>(true),
		generate_base_delegation_hierarchy_details(),
	);
	let (parent_id, mut parent_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(hierarchy_root_id, user_2.clone(), Some(hierarchy_root_id)),
	);
	parent_node.details.revoked = true;
	let (delegation_id, delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(hierarchy_root_id, user_3.clone(), Some(parent_id)),
	);

	let max_parent_checks = 2u32;

	// Root -> Parent -> Delegation
	// Root -> Parent -> Delegation
	ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, user_1.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, user_1.clone())])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.with_balances(vec![
			(user_1, <Test as Config>::Deposit::get()),
			(user_2.clone(), <Test as Config>::Deposit::get()),
			(user_3, <Test as Config>::Deposit::get()),
		])
		.build()
		.execute_with(|| {
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
		get_delegation_hierarchy_id::<Test>(true),
		generate_base_delegation_hierarchy_details(),
	);
	let (parent_id, parent_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(hierarchy_root_id, user_2.clone(), Some(hierarchy_root_id)),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(hierarchy_root_id, user_3.clone(), Some(parent_id)),
	);

	let max_parent_checks = 2u32;

	// Root -> Parent -> Delegation
	// Root -> Parent -> Delegation
	ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, user_1.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, user_1.clone())])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.with_balances(vec![
			(user_1.clone(), <Test as Config>::Deposit::get()),
			(user_2, <Test as Config>::Deposit::get()),
			(user_3, <Test as Config>::Deposit::get()),
		])
		.build()
		.execute_with(|| {
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
		get_delegation_hierarchy_id::<Test>(true),
		generate_base_delegation_hierarchy_details(),
	);
	let (parent_id, parent_node) = (
		get_delegation_id(true),
		generate_base_delegation_node(hierarchy_root_id, user_2.clone(), Some(hierarchy_root_id)),
	);
	let (delegation_id, delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(hierarchy_root_id, user_3.clone(), Some(parent_id)),
	);

	let max_parent_checks = 2u32;

	// Root -> Parent -> Delegation
	// Root -> Parent -> Delegation
	ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, user_1.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, user_1.clone())])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.with_balances(vec![
			(user_1.clone(), <Test as Config>::Deposit::get()),
			(user_2, <Test as Config>::Deposit::get()),
			(user_3, <Test as Config>::Deposit::get()),
		])
		.build()
		.execute_with(|| {
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
		get_delegation_hierarchy_id::<Test>(true),
		generate_base_delegation_hierarchy_details(),
	);
	let delegation_id = get_delegation_id(true);

	let max_parent_checks = 2u32;

	// Root -> Delegation 1
	let mut ext = ExtBuilder::default()
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, user_1.clone())])
		.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::is_delegating(&user_1, &delegation_id, max_parent_checks),
			Error::<Test>::DelegationNotFound
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

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details();
	let parent_id = get_delegation_id(true);
	let parent_node = generate_base_delegation_node(hierarchy_root_id, user_2.clone(), Some(hierarchy_root_id));
	let (delegation_id, delegation_node) = (
		get_delegation_id(false),
		generate_base_delegation_node(hierarchy_root_id, user_3.clone(), Some(parent_id)),
	);

	// 1 less than needed
	let max_parent_checks = 1u32;

	// Root -> Parent -> Delegation
	// Root -> Parent -> Delegation
	ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, user_1.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, user_1.clone())])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.with_balances(vec![
			(user_1.clone(), <Test as Config>::Deposit::get()),
			(user_2, <Test as Config>::Deposit::get()),
			(user_3, <Test as Config>::Deposit::get()),
		])
		.build()
		.execute_with(|| {
			assert_noop!(
				Delegation::is_delegating(&user_1, &delegation_id, max_parent_checks),
				Error::<Test>::MaxSearchDepthReached
			);
		});
}

#[test]
fn remove_single_hierarchy() {
	let creator_keypair = get_alice_ed25519();
	let creator = get_ed25519_account(creator_keypair.public());
	let attacker_keypair = get_bob_sr25519();
	let attacker = get_sr25519_account(attacker_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id::<Test>(true),
		generate_base_delegation_hierarchy_details(),
	);

	ExtBuilder::default()
		.with_balances(vec![(creator.clone(), <Test as Config>::Deposit::get())])
		.with_ctypes(vec![(hierarchy_details.ctype_hash, creator.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, creator.clone())])
		.build()
		.execute_with(|| {
			assert!(Delegation::delegation_hierarchies(hierarchy_root_id).is_some());
			assert!(Delegation::delegation_nodes(hierarchy_root_id).is_some());
			assert_eq!(
				Balances::reserved_balance(creator.clone()),
				<Test as Config>::Deposit::get()
			);

			// Remove
			assert_noop!(
				Delegation::remove_delegation(get_origin(attacker), hierarchy_root_id, 0u32),
				Error::<Test>::UnauthorizedRemoval
			);
			assert_ok!(Delegation::remove_delegation(
				get_origin(creator.clone()),
				hierarchy_root_id,
				0
			),);
			assert!(Delegation::delegation_hierarchies(hierarchy_root_id).is_none());
			assert!(Delegation::delegation_nodes(hierarchy_root_id).is_none());
			assert!(Balances::reserved_balance(creator).is_zero());
		});
}

#[test]
fn remove_children_gas_runs_out() {
	let revoker_keypair = get_alice_ed25519();
	let revoker = get_ed25519_account(revoker_keypair.public());
	let delegate_keypair = get_bob_ed25519();
	let delegate = get_ed25519_account(delegate_keypair.public());
	let child_keypair = get_charlie_ed25519();
	let child = get_ed25519_account(child_keypair.public());

	let (hierarchy_root_id, hierarchy_details) = (
		get_delegation_hierarchy_id::<Test>(true),
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
		generate_base_delegation_node(hierarchy_root_id, delegate.clone(), Some(delegation1_id)),
	);
	let (delegation4_id, delegation4_node) = (
		get_delegation_id_2(false),
		generate_base_delegation_node(hierarchy_root_id, child.clone(), Some(delegation3_id)),
	);

	let mut operation = generate_base_delegation_hierarchy_revocation_operation(hierarchy_root_id);
	// set max children below required minimum of 4 to revoke/remove entire tree
	operation.max_children = 3u32;

	ExtBuilder::default()
		.with_balances(vec![
			(revoker.clone(), <Test as Config>::Deposit::get()),
			(delegate.clone(), <Test as Config>::Deposit::get() * 3),
			(child.clone(), <Test as Config>::Deposit::get()),
		])
		.with_ctypes(vec![(hierarchy_details.ctype_hash, revoker.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, revoker.clone())])
		.with_delegations(vec![
			(delegation1_id, delegation1_node),
			(delegation2_id, delegation2_node),
			(delegation3_id, delegation3_node),
			(delegation4_id, delegation4_node),
		])
		.build()
		.execute_with(|| {
			assert_eq!(
				Balances::reserved_balance(revoker.clone()),
				<Test as Config>::Deposit::get()
			);
			assert_eq!(
				Balances::reserved_balance(delegate.clone()),
				3 * <Test as Config>::Deposit::get()
			);
			assert_eq!(
				Balances::reserved_balance(child.clone()),
				<Test as Config>::Deposit::get()
			);

			// Should not be able to remove root because tree traversal steps are
			// insufficient
			// assert_err and not assert_noop because the storage is indeed changed, but
			// only partially (only #4 is removed)
			assert_err!(
				Delegation::remove_delegation(get_origin(revoker.clone()), operation.id, operation.max_children),
				Error::<Test>::ExceededRemovalBounds
			);
			assert!(Delegation::delegation_nodes(&operation.id).is_some());
			assert!(Delegation::delegation_nodes(&delegation1_id).is_some());
			assert!(Delegation::delegation_nodes(&delegation2_id).is_none());
			assert!(Delegation::delegation_nodes(&delegation3_id).is_some());
			assert!(Delegation::delegation_nodes(&delegation4_id).is_some());
			assert_eq!(
				Balances::reserved_balance(revoker.clone()),
				<Test as Config>::Deposit::get()
			);
			assert_eq!(
				Balances::reserved_balance(delegate.clone()),
				2 * <Test as Config>::Deposit::get()
			);
			assert_eq!(
				Balances::reserved_balance(child.clone()),
				<Test as Config>::Deposit::get()
			);

			// Should be able to remove root now because #_of_children = 3
			assert_ok!(Delegation::remove_delegation(
				get_origin(revoker.clone()),
				operation.id,
				operation.max_children
			),);
			assert!(Delegation::delegation_nodes(&operation.id).is_none());
			assert!(Delegation::delegation_nodes(&delegation1_id).is_none());
			assert!(Delegation::delegation_nodes(&delegation2_id).is_none());
			assert!(Delegation::delegation_nodes(&delegation3_id).is_none());
			assert!(Balances::reserved_balance(revoker.clone()).is_zero());
			assert!(Balances::reserved_balance(delegate.clone()).is_zero());
			assert!(Balances::reserved_balance(child.clone()).is_zero());
		});
}

// ⚠️ This test is matched to a unit test in the SDK. Both must be updated in
// sync ⚠️
#[test]
fn calculate_reference_root_hash() {
	let delegation_id = sp_core::H256::from_slice(&[
		185, 126, 188, 188, 245, 142, 61, 132, 79, 33, 135, 134, 152, 6, 203, 109, 120, 38, 111, 103, 61, 241, 239,
		138, 46, 66, 215, 123, 126, 94, 77, 66,
	]);

	let hierarchy_root_id = sp_core::H256::from_slice(&[
		70, 167, 51, 210, 165, 28, 28, 139, 12, 153, 171, 25, 102, 201, 169, 88, 65, 125, 169, 191, 42, 153, 90, 138,
		192, 111, 144, 8, 231, 196, 167, 51,
	]);

	let parent_id = sp_core::H256::from_slice(&[
		41, 25, 184, 103, 76, 11, 115, 50, 39, 65, 32, 13, 205, 136, 227, 114, 253, 200, 50, 199, 71, 206, 216, 234,
		99, 37, 221, 21, 199, 111, 91, 209,
	]);

	let permissions = delegation::Permissions::ATTEST;

	let delegation_hash =
		Delegation::calculate_delegation_creation_hash(&delegation_id, &hierarchy_root_id, &parent_id, &permissions);

	assert_eq!(
		delegation_hash,
		sp_core::H256::from_slice(&[
			163, 68, 221, 218, 225, 105, 180, 154, 248, 52, 210, 46, 111, 20, 142, 1, 154, 18, 189, 126, 217, 41, 151,
			135, 19, 250, 243, 130, 33, 174, 133, 4,
		])
	)
}
