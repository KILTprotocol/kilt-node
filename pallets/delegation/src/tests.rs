// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2023 BOTLabs GmbH

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
use kilt_support::mock::mock_origin::DoubleOrigin;

use crate::{
	self as delegation,
	mock::{runtime::*, *},
	Config, Error,
};
use sp_runtime::traits::Zero;

// submit_delegation_root_creation_operation()

#[test]
fn create_root_delegation_successful() {
	let creator = ed25519_did_from_seed(&ALICE_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let operation = generate_base_delegation_hierarchy_creation_operation::<Test>(hierarchy_root_id);

	ExtBuilder::default()
		.with_ctypes(vec![(operation.ctype_hash, creator.clone())])
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get())])
		.build()
		.execute_with(|| {
			// Create root hierarchy
			assert_ok!(Delegation::create_hierarchy(
				DoubleOrigin(ACCOUNT_00, creator.clone()).into(),
				operation.id,
				operation.ctype_hash
			));

			// Check reserved balance
			assert_eq!(Balances::reserved_balance(ACCOUNT_00), <Test as Config>::Deposit::get());

			// Get stored hierarchy
			let stored_hierarchy_details = Delegation::delegation_hierarchies(hierarchy_root_id)
				.expect("Delegation hierarchy should be present on chain.");
			assert_eq!(stored_hierarchy_details.ctype_hash, operation.ctype_hash);

			// Check root delegation
			let stored_delegation_root =
				Delegation::delegation_nodes(hierarchy_root_id).expect("Delegation root should be present on chain.");
			assert_eq!(stored_delegation_root.hierarchy_root_id, hierarchy_root_id);
			assert!(stored_delegation_root.parent.is_none());
			assert!(stored_delegation_root.children.len().is_zero());
			assert_eq!(stored_delegation_root.details.owner, creator.clone());
			assert_eq!(stored_delegation_root.deposit.owner, ACCOUNT_00);
			assert!(!stored_delegation_root.details.revoked);
		});
}

#[test]
fn duplicate_create_root_delegation_error() {
	let creator = ed25519_did_from_seed(&ALICE_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details();

	let operation = generate_base_delegation_hierarchy_creation_operation::<Test>(hierarchy_root_id);

	ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, creator.clone())])
		.with_delegation_hierarchies(vec![(
			hierarchy_root_id,
			hierarchy_details,
			creator.clone(),
			ACCOUNT_00,
		)])
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get())])
		.build()
		.execute_with(|| {
			assert_noop!(
				Delegation::create_hierarchy(
					DoubleOrigin(ACCOUNT_00, creator.clone()).into(),
					operation.id,
					operation.ctype_hash
				),
				Error::<Test>::HierarchyAlreadyExists
			);
		});
}

#[test]
fn ctype_not_found_create_root_delegation_error() {
	let creator = ed25519_did_from_seed(&ALICE_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);

	let operation = generate_base_delegation_hierarchy_creation_operation::<Test>(hierarchy_root_id);

	// No CType stored
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Delegation::create_hierarchy(
				DoubleOrigin(ACCOUNT_00, creator.clone()).into(),
				operation.id,
				operation.ctype_hash
			),
			ctype::Error::<Test>::NotFound
		);
	});
}

// submit_delegation_creation_operation()

#[test]
fn create_delegation_direct_root_successful() {
	let creator = ed25519_did_from_seed(&ALICE_SEED);
	let delegate = sr25519_did_from_seed(&BOB_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details();

	ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, creator.clone())])
		.with_delegation_hierarchies(vec![(
			hierarchy_root_id,
			hierarchy_details,
			creator.clone(),
			ACCOUNT_00,
		)])
		.with_balances(vec![
			(ACCOUNT_00, <Test as Config>::Deposit::get()),
			(ACCOUNT_01, <Test as Config>::Deposit::get()),
		])
		.build()
		.execute_with(|| {
			// Create delegation to root
			let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
			let delegation_node =
				generate_base_delegation_node(hierarchy_root_id, delegate.clone(), Some(hierarchy_root_id), ACCOUNT_01);
			let delegation_info = Delegation::calculate_delegation_creation_hash(
				&delegation_id,
				&hierarchy_root_id,
				&hierarchy_root_id,
				&delegation_node.details.permissions,
			);
			let delegate_signature = (delegate.clone(), hash_to_u8(delegation_info));
			let operation =
				generate_base_delegation_creation_operation(delegation_id, delegate_signature, delegation_node);

			// 1 Deposit should be reserved for hierarchy
			assert_eq!(Balances::reserved_balance(ACCOUNT_00), <Test as Config>::Deposit::get());

			// Add delegation to root
			assert_ok!(Delegation::add_delegation(
				DoubleOrigin(ACCOUNT_00, creator.clone()).into(),
				operation.delegation_id,
				operation.parent_id,
				operation.delegate.clone(),
				operation.permissions,
				operation.delegate_signature.clone(),
			));

			// 2 Deposits should be reserved for hierarchy and delegation to root
			assert_eq!(
				Balances::reserved_balance(ACCOUNT_00),
				2 * <Test as Config>::Deposit::get()
			);

			// Check stored delegation against operation
			let stored_delegation =
				Delegation::delegation_nodes(operation.delegation_id).expect("Delegation should be present on chain.");
			assert_eq!(stored_delegation.hierarchy_root_id, operation.hierarchy_id);
			assert_eq!(stored_delegation.parent, Some(operation.parent_id));
			assert!(stored_delegation.children.is_empty());
			assert_eq!(stored_delegation.details.owner, operation.delegate);
			assert_eq!(stored_delegation.details.permissions, operation.permissions);
			assert!(!stored_delegation.details.revoked);

			// Verify that the root has the new delegation among its children
			let stored_root = Delegation::delegation_nodes(operation.hierarchy_id)
				.expect("Delegation root should be present on chain.");
			assert!(stored_root.children.contains(&operation.delegation_id));
		});
}

#[test]
fn create_delegation_with_parent_successful() {
	let creator = ed25519_did_from_seed(&ALICE_SEED);
	let delegate = sr25519_did_from_seed(&BOB_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details();
	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let parent_node =
		generate_base_delegation_node(hierarchy_root_id, creator.clone(), Some(hierarchy_root_id), ACCOUNT_00);

	ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, creator.clone())])
		.with_delegation_hierarchies(vec![(
			hierarchy_root_id,
			hierarchy_details,
			creator.clone(),
			ACCOUNT_00,
		)])
		.with_delegations(vec![(parent_id, parent_node)])
		.with_balances(vec![
			(ACCOUNT_00, <Test as Config>::Deposit::get()),
			(ACCOUNT_01, <Test as Config>::Deposit::get()),
		])
		.build()
		.execute_with(|| {
			// Create sub-delegation
			let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
			let delegation_node =
				generate_base_delegation_node(hierarchy_root_id, delegate.clone(), Some(parent_id), ACCOUNT_00);
			let delegation_info = Delegation::calculate_delegation_creation_hash(
				&delegation_id,
				&hierarchy_root_id,
				&parent_id,
				&delegation_node.details.permissions,
			);
			let delegate_signature = (delegate.clone(), hash_to_u8(delegation_info));
			let operation =
				generate_base_delegation_creation_operation(delegation_id, delegate_signature, delegation_node);

			// Should have deposited for hierarchy and parent delegation
			assert_eq!(
				Balances::reserved_balance(ACCOUNT_00),
				2 * <Test as Config>::Deposit::get()
			);

			// Add sub-delegation
			assert_ok!(Delegation::add_delegation(
				DoubleOrigin(ACCOUNT_00, creator.clone()).into(),
				delegation_id,
				parent_id,
				operation.delegate.clone(),
				operation.permissions,
				operation.delegate_signature.clone(),
			));

			// Should have deposited for hierarchy, parent delegation and sub-delegation
			assert_eq!(
				Balances::reserved_balance(ACCOUNT_00),
				3 * <Test as Config>::Deposit::get()
			);
			assert!(Balances::reserved_balance(ACCOUNT_01).is_zero());

			// Data in stored delegation and operation should match
			let stored_delegation =
				Delegation::delegation_nodes(operation.delegation_id).expect("Delegation should be present on chain.");
			assert_eq!(stored_delegation.hierarchy_root_id, operation.hierarchy_id);
			assert_eq!(stored_delegation.parent, Some(operation.parent_id));
			assert!(stored_delegation.children.is_empty());
			assert_eq!(stored_delegation.details.owner, operation.delegate);
			assert_eq!(stored_delegation.details.permissions, operation.permissions);
			assert!(!stored_delegation.details.revoked);
			assert_eq!(stored_delegation.deposit.owner, ACCOUNT_00);

			// Verify that the parent has the new delegation among its children
			let stored_parent =
				Delegation::delegation_nodes(operation.parent_id).expect("Delegation parent be present on chain.");

			assert!(stored_parent.children.contains(&operation.delegation_id));
		});
}

#[test]
fn create_delegation_direct_root_revoked_error() {
	let creator = ed25519_did_from_seed(&ALICE_SEED);
	let delegate = sr25519_did_from_seed(&BOB_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details();
	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let delegation_node =
		generate_base_delegation_node(hierarchy_root_id, delegate.clone(), Some(hierarchy_root_id), ACCOUNT_00);

	let delegation_hash = &hash_to_u8(Delegation::calculate_delegation_creation_hash(
		&delegation_id,
		&hierarchy_root_id,
		&hierarchy_root_id,
		&delegation_node.details.permissions,
	));
	let delegate_signature = (delegate.clone(), delegation_hash.clone());

	let operation = generate_base_delegation_creation_operation(delegation_id, delegate_signature, delegation_node);

	ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, creator.clone())])
		.with_delegation_hierarchies(vec![(
			hierarchy_root_id,
			hierarchy_details,
			creator.clone(),
			ACCOUNT_00,
		)])
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get())])
		.build()
		.execute_with(|| {
			let _ = Delegation::revoke_delegation(
				DoubleOrigin(ACCOUNT_00, creator.clone()).into(),
				operation.hierarchy_id,
				0u32,
				MaxRevocations::get(),
			);
			assert_noop!(
				Delegation::add_delegation(
					DoubleOrigin(ACCOUNT_00, creator.clone()).into(),
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
	let creator = ed25519_did_from_seed(&ALICE_SEED);
	let delegate = sr25519_did_from_seed(&BOB_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details();

	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let parent_node =
		generate_base_delegation_node(hierarchy_root_id, creator.clone(), Some(hierarchy_root_id), ACCOUNT_00);

	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let delegation_node =
		generate_base_delegation_node(hierarchy_root_id, delegate.clone(), Some(parent_id), ACCOUNT_00);

	let delegation_info = Delegation::calculate_delegation_creation_hash(
		&delegation_id,
		&hierarchy_root_id,
		&parent_id,
		&delegation_node.details.permissions,
	);

	let delegate_signature = (delegate.clone(), hash_to_u8(delegation_info));

	let operation = generate_base_delegation_creation_operation(delegation_id, delegate_signature, delegation_node);

	ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, creator.clone())])
		.with_delegation_hierarchies(vec![(
			hierarchy_root_id,
			hierarchy_details,
			creator.clone(),
			ACCOUNT_00,
		)])
		.with_delegations(vec![(parent_id, parent_node)])
		.with_balances(vec![
			(ACCOUNT_00, <Test as Config>::Deposit::get()),
			(ACCOUNT_01, <Test as Config>::Deposit::get()),
		])
		.build()
		.execute_with(|| {
			let _ = Delegation::revoke_delegation(
				DoubleOrigin(ACCOUNT_00, creator.clone()).into(),
				operation.parent_id,
				MaxRevocations::get(),
				MaxParentChecks::get(),
			);
			assert_noop!(
				Delegation::add_delegation(
					DoubleOrigin(ACCOUNT_00, creator.clone()).into(),
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
	let creator = ed25519_did_from_seed(&ALICE_SEED);
	let delegate = sr25519_did_from_seed(&BOB_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details();
	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let delegation_node =
		generate_base_delegation_node(hierarchy_root_id, delegate.clone(), Some(hierarchy_root_id), ACCOUNT_00);

	let delegate_signature = (delegate.clone(), vec![]);

	let operation = generate_base_delegation_creation_operation(delegation_id, delegate_signature, delegation_node);

	ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, creator.clone())])
		.with_delegation_hierarchies(vec![(
			hierarchy_root_id,
			hierarchy_details,
			creator.clone(),
			ACCOUNT_00,
		)])
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get())])
		.build()
		.execute_with(|| {
			assert_noop!(
				Delegation::add_delegation(
					DoubleOrigin(ACCOUNT_00, creator.clone()).into(),
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
	let creator = ed25519_did_from_seed(&ALICE_SEED);
	let delegate = sr25519_did_from_seed(&BOB_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details();
	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let delegation_node =
		generate_base_delegation_node(hierarchy_root_id, delegate.clone(), Some(hierarchy_root_id), ACCOUNT_00);

	let delegation_hash = &hash_to_u8(Delegation::calculate_delegation_creation_hash(
		&delegation_id,
		&hierarchy_root_id,
		&hierarchy_root_id,
		&delegation_node.details.permissions,
	));
	let delegate_signature = (delegate.clone(), delegation_hash.clone());

	let operation =
		generate_base_delegation_creation_operation(delegation_id, delegate_signature, delegation_node.clone());

	ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, creator.clone())])
		.with_delegation_hierarchies(vec![(
			hierarchy_root_id,
			hierarchy_details,
			creator.clone(),
			ACCOUNT_00,
		)])
		.with_delegations(vec![(delegation_id, delegation_node)])
		.with_balances(vec![
			(ACCOUNT_00, <Test as Config>::Deposit::get()),
			(ACCOUNT_01, <Test as Config>::Deposit::get()),
		])
		.build()
		.execute_with(|| {
			assert_noop!(
				Delegation::add_delegation(
					DoubleOrigin(ACCOUNT_00, creator.clone()).into(),
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
	let creator = ed25519_did_from_seed(&ALICE_SEED);
	let delegate = sr25519_did_from_seed(&BOB_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details::<Test>();

	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let delegation_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, delegate.clone(), Some(hierarchy_root_id), ACCOUNT_00);

	let delegation_hash = &hash_to_u8(Delegation::calculate_delegation_creation_hash(
		&delegation_id,
		&hierarchy_root_id,
		&hierarchy_root_id,
		&delegation_node.details.permissions,
	));
	let delegate_signature = (delegate.clone(), delegation_hash.clone());

	let operation = generate_base_delegation_creation_operation(delegation_id, delegate_signature, delegation_node);

	ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, creator.clone())])
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get())])
		.build()
		.execute_with(|| {
			assert_noop!(
				Delegation::add_delegation(
					DoubleOrigin(ACCOUNT_00, creator.clone()).into(),
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
	let creator = ed25519_did_from_seed(&ALICE_SEED);
	let delegate = sr25519_did_from_seed(&BOB_SEED);
	let alternative_owner = ed25519_did_from_seed(&CHARLIE_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details();
	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let parent_node = generate_base_delegation_node(
		hierarchy_root_id,
		alternative_owner,
		Some(hierarchy_root_id),
		ACCOUNT_00,
	);

	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let delegation_node =
		generate_base_delegation_node(hierarchy_root_id, delegate.clone(), Some(parent_id), ACCOUNT_00);

	let delegation_hash = &hash_to_u8(Delegation::calculate_delegation_creation_hash(
		&delegation_id,
		&hierarchy_root_id,
		&parent_id,
		&delegation_node.details.permissions,
	));
	let delegate_signature = (delegate.clone(), delegation_hash.clone());

	let operation = generate_base_delegation_creation_operation(delegation_id, delegate_signature, delegation_node);

	ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, creator.clone())])
		.with_delegation_hierarchies(vec![(
			hierarchy_root_id,
			hierarchy_details,
			creator.clone(),
			ACCOUNT_00,
		)])
		.with_delegations(vec![(parent_id, parent_node)])
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get())])
		.build()
		.execute_with(|| {
			assert_noop!(
				Delegation::add_delegation(
					DoubleOrigin(ACCOUNT_00, creator.clone()).into(),
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
	let creator = ed25519_did_from_seed(&ALICE_SEED);
	let delegate = sr25519_did_from_seed(&BOB_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details();
	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let mut parent_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, creator.clone(), Some(hierarchy_root_id), ACCOUNT_00);
	parent_node.details.permissions = delegation::Permissions::ATTEST;

	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let delegation_node =
		generate_base_delegation_node(hierarchy_root_id, delegate.clone(), Some(parent_id), ACCOUNT_00);

	let delegation_hash = &hash_to_u8(Delegation::calculate_delegation_creation_hash(
		&delegation_id,
		&hierarchy_root_id,
		&parent_id,
		&delegation_node.details.permissions,
	));
	let delegate_signature = (delegate.clone(), delegation_hash.clone());

	let operation = generate_base_delegation_creation_operation(delegation_id, delegate_signature, delegation_node);

	ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, creator.clone())])
		.with_delegation_hierarchies(vec![(
			hierarchy_root_id,
			hierarchy_details,
			creator.clone(),
			ACCOUNT_00,
		)])
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get())])
		.with_delegations(vec![(parent_id, parent_node)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Delegation::add_delegation(
					DoubleOrigin(ACCOUNT_00, creator.clone()).into(),
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
	let revoker = ed25519_did_from_seed(&ALICE_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details();

	let operation = generate_base_delegation_hierarchy_revocation_operation(hierarchy_root_id);

	ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, revoker.clone())])
		.with_delegation_hierarchies(vec![(
			hierarchy_root_id,
			hierarchy_details,
			revoker.clone(),
			ACCOUNT_00,
		)])
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get())])
		.build()
		.execute_with(|| {
			assert_ok!(Delegation::revoke_delegation(
				DoubleOrigin(ACCOUNT_00, revoker.clone()).into(),
				operation.id,
				0u32,
				operation.max_children
			));

			assert!(
				Delegation::delegation_nodes(operation.id)
					.expect("Delegation root should be present on chain.")
					.details
					.revoked
			)
		});
}

#[test]
fn list_hierarchy_revoke_and_remove_root_successful() {
	let revoker = ed25519_did_from_seed(&ALICE_SEED);
	let delegate = sr25519_did_from_seed(&BOB_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details();
	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let parent_node =
		generate_base_delegation_node(hierarchy_root_id, revoker.clone(), Some(hierarchy_root_id), ACCOUNT_00);

	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let delegation_node = generate_base_delegation_node(hierarchy_root_id, delegate, Some(parent_id), ACCOUNT_01);

	let mut operation = generate_base_delegation_hierarchy_revocation_operation(hierarchy_root_id);
	operation.max_children = 2u32;

	ExtBuilder::default()
		.with_balances(vec![
			(ACCOUNT_00, <Test as Config>::Deposit::get() * 2),
			(ACCOUNT_01, <Test as Config>::Deposit::get()),
		])
		.with_ctypes(vec![(hierarchy_details.ctype_hash, revoker.clone())])
		.with_delegation_hierarchies(vec![(
			hierarchy_root_id,
			hierarchy_details,
			revoker.clone(),
			ACCOUNT_00,
		)])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.build()
		.execute_with(|| {
			assert!(Delegation::delegation_hierarchies(hierarchy_root_id).is_some());
			assert_eq!(
				Balances::reserved_balance(ACCOUNT_00),
				2 * <Test as Config>::Deposit::get()
			);
			assert_eq!(Balances::reserved_balance(ACCOUNT_01), <Test as Config>::Deposit::get());

			// Revoke root
			assert_ok!(Delegation::revoke_delegation(
				DoubleOrigin(ACCOUNT_00, revoker.clone()).into(),
				operation.id,
				0u32,
				operation.max_children
			));

			// Root and children should still be stored with revoked status
			assert!(
				Delegation::delegation_nodes(operation.id)
					.expect("Delegation root should be present on chain.")
					.details
					.revoked
			);
			assert!(
				Delegation::delegation_nodes(parent_id)
					.expect("Parent delegation should be present on chain.")
					.details
					.revoked
			);
			assert!(
				Delegation::delegation_nodes(delegation_id)
					.expect("Delegation should be present on chain.")
					.details
					.revoked
			);

			// Removing root should also remove children and hierarchy
			assert_ok!(Delegation::remove_delegation(
				DoubleOrigin(ACCOUNT_00, revoker.clone()).into(),
				operation.id,
				operation.max_children
			));

			assert!(Delegation::delegation_nodes(operation.id).is_none());
			assert!(Delegation::delegation_hierarchies(hierarchy_root_id).is_none());
			assert!(Delegation::delegation_nodes(parent_id).is_none());
			assert!(Delegation::delegation_nodes(delegation_id).is_none());
			assert!(Balances::reserved_balance(ACCOUNT_00).is_zero());
			assert!(Balances::reserved_balance(ACCOUNT_01).is_zero());
		});
}

#[test]
fn tree_hierarchy_revoke_and_remove_root_successful() {
	let revoker = ed25519_did_from_seed(&ALICE_SEED);
	let delegate = sr25519_did_from_seed(&BOB_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details();
	let delegation1_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let delegation1_node =
		generate_base_delegation_node(hierarchy_root_id, revoker.clone(), Some(hierarchy_root_id), ACCOUNT_00);

	let delegation2_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let delegation2_node =
		generate_base_delegation_node(hierarchy_root_id, delegate, Some(hierarchy_root_id), ACCOUNT_01);

	let mut operation = generate_base_delegation_hierarchy_revocation_operation(hierarchy_root_id);
	operation.max_children = 2u32;

	ExtBuilder::default()
		.with_balances(vec![
			(ACCOUNT_00, <Test as Config>::Deposit::get() * 2),
			(ACCOUNT_01, <Test as Config>::Deposit::get()),
		])
		.with_ctypes(vec![(hierarchy_details.ctype_hash, revoker.clone())])
		.with_delegation_hierarchies(vec![(
			hierarchy_root_id,
			hierarchy_details,
			revoker.clone(),
			ACCOUNT_00,
		)])
		.with_delegations(vec![
			(delegation1_id, delegation1_node),
			(delegation2_id, delegation2_node),
		])
		.build()
		.execute_with(|| {
			// Revoke root
			assert_ok!(Delegation::revoke_delegation(
				DoubleOrigin(ACCOUNT_00, revoker.clone()).into(),
				operation.id,
				0u32,
				operation.max_children
			));

			// Root and children should still be stored with revoked status
			assert!(
				Delegation::delegation_nodes(operation.id)
					.expect("Delegation root should be present on chain.")
					.details
					.revoked
			);
			assert!(
				Delegation::delegation_nodes(delegation1_id)
					.expect("Delegation 1 should be present on chain.")
					.details
					.revoked
			);
			assert!(
				Delegation::delegation_nodes(delegation2_id)
					.expect("Delegation 2 should be present on chain.")
					.details
					.revoked
			);

			assert_eq!(
				Balances::reserved_balance(ACCOUNT_00),
				2 * <Test as Config>::Deposit::get()
			);
			assert_eq!(Balances::reserved_balance(ACCOUNT_01), <Test as Config>::Deposit::get());

			// Removing root should also remove children and hierarchy
			assert_ok!(Delegation::remove_delegation(
				DoubleOrigin(ACCOUNT_00, revoker.clone()).into(),
				operation.id,
				operation.max_children
			));

			assert!(Delegation::delegation_nodes(operation.id).is_none());
			assert!(Delegation::delegation_hierarchies(hierarchy_root_id).is_none());
			assert!(Delegation::delegation_nodes(delegation1_id).is_none());
			assert!(Delegation::delegation_nodes(delegation2_id).is_none());
			assert!(Balances::reserved_balance(ACCOUNT_00).is_zero());
			assert!(Balances::reserved_balance(ACCOUNT_01).is_zero());
		});
}

#[test]
fn max_max_revocations_revoke_and_remove_successful() {
	let revoker = ed25519_did_from_seed(&ALICE_SEED);
	let delegate = sr25519_did_from_seed(&BOB_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details();
	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let parent_node =
		generate_base_delegation_node(hierarchy_root_id, revoker.clone(), Some(hierarchy_root_id), ACCOUNT_00);

	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let delegation_node = generate_base_delegation_node(hierarchy_root_id, delegate, Some(parent_id), ACCOUNT_01);

	let mut operation = generate_base_delegation_hierarchy_revocation_operation(hierarchy_root_id);
	operation.max_children = MaxRevocations::get();

	ExtBuilder::default()
		.with_balances(vec![
			(ACCOUNT_00, <Test as Config>::Deposit::get() * 2),
			(ACCOUNT_01, <Test as Config>::Deposit::get()),
		])
		.with_ctypes(vec![(hierarchy_details.ctype_hash, revoker.clone())])
		.with_delegation_hierarchies(vec![(
			hierarchy_root_id,
			hierarchy_details,
			revoker.clone(),
			ACCOUNT_00,
		)])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.build()
		.execute_with(|| {
			// Revoke root
			assert_ok!(Delegation::revoke_delegation(
				DoubleOrigin(ACCOUNT_00, revoker.clone()).into(),
				operation.id,
				0u32,
				operation.max_children
			));

			// Root and children should still be stored with revoked status
			assert!(
				Delegation::delegation_nodes(operation.id)
					.expect("Delegation root should be present on chain.")
					.details
					.revoked
			);
			assert!(
				Delegation::delegation_nodes(parent_id)
					.expect("Parent delegation should be present on chain.")
					.details
					.revoked
			);
			assert!(
				Delegation::delegation_nodes(delegation_id)
					.expect("Delegation should be present on chain.")
					.details
					.revoked
			);

			assert_eq!(
				Balances::reserved_balance(ACCOUNT_00),
				2 * <Test as Config>::Deposit::get()
			);
			assert_eq!(Balances::reserved_balance(ACCOUNT_01), <Test as Config>::Deposit::get());

			// Removing root should also remove children and hierarchy
			assert_ok!(Delegation::remove_delegation(
				DoubleOrigin(ACCOUNT_00, revoker.clone()).into(),
				operation.id,
				operation.max_children
			));

			assert!(Delegation::delegation_nodes(operation.id).is_none());
			assert!(Delegation::delegation_hierarchies(hierarchy_root_id).is_none());
			assert!(Delegation::delegation_nodes(parent_id).is_none());
			assert!(Delegation::delegation_nodes(delegation_id).is_none());
			assert!(Balances::reserved_balance(ACCOUNT_00).is_zero());
			assert!(Balances::reserved_balance(ACCOUNT_01).is_zero());
		});
}

#[test]
fn root_not_found_revoke_and_remove_root_error() {
	let revoker = ed25519_did_from_seed(&ALICE_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);

	let operation = generate_base_delegation_hierarchy_revocation_operation(hierarchy_root_id);

	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Delegation::revoke_delegation(
				DoubleOrigin(ACCOUNT_00, revoker.clone()).into(),
				operation.id,
				0u32,
				operation.max_children
			),
			Error::<Test>::DelegationNotFound
		);
		assert_noop!(
			Delegation::remove_delegation(
				DoubleOrigin(ACCOUNT_00, revoker.clone()).into(),
				operation.id,
				operation.max_children
			),
			Error::<Test>::DelegationNotFound
		);
	});
}

#[test]
fn different_root_creator_revoke_and_remove_root_error() {
	let owner = ed25519_did_from_seed(&ALICE_SEED);
	let unauthorized = ed25519_did_from_seed(&CHARLIE_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details();

	let operation = generate_base_delegation_hierarchy_revocation_operation(hierarchy_root_id);

	ExtBuilder::default()
		.with_balances(vec![
			(ACCOUNT_00, <Test as Config>::Deposit::get()),
			(ACCOUNT_01, <Test as Config>::Deposit::get()),
		])
		.with_ctypes(vec![(hierarchy_details.ctype_hash, owner.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, owner, ACCOUNT_00)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Delegation::revoke_delegation(
					DoubleOrigin(ACCOUNT_00, unauthorized.clone()).into(),
					operation.id,
					0u32,
					operation.max_children
				),
				Error::<Test>::UnauthorizedRevocation
			);
			assert_noop!(
				Delegation::remove_delegation(
					DoubleOrigin(ACCOUNT_00, unauthorized.clone()).into(),
					operation.id,
					operation.max_children
				),
				Error::<Test>::UnauthorizedRemoval
			);
		});
}

#[test]
fn too_small_max_revocations_revoke_and_remove_root_error() {
	let revoker = ed25519_did_from_seed(&ALICE_SEED);
	let delegate = ed25519_did_from_seed(&BOB_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details();

	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let delegation_node =
		generate_base_delegation_node(hierarchy_root_id, delegate, Some(hierarchy_root_id), ACCOUNT_00);

	let mut operation = generate_base_delegation_hierarchy_revocation_operation(hierarchy_root_id);
	operation.max_children = 0u32;

	ExtBuilder::default()
		.with_balances(vec![
			(ACCOUNT_00, <Test as Config>::Deposit::get() * 2),
			(ACCOUNT_01, <Test as Config>::Deposit::get()),
		])
		.with_ctypes(vec![(hierarchy_details.ctype_hash, revoker.clone())])
		.with_delegation_hierarchies(vec![(
			hierarchy_root_id,
			hierarchy_details,
			revoker.clone(),
			ACCOUNT_00,
		)])
		.with_delegations(vec![(delegation_id, delegation_node)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Delegation::revoke_delegation(
					DoubleOrigin(ACCOUNT_00, revoker.clone()).into(),
					operation.id,
					0u32,
					operation.max_children
				),
				Error::<Test>::ExceededRevocationBounds
			);
			assert_noop!(
				Delegation::remove_delegation(
					DoubleOrigin(ACCOUNT_00, revoker.clone()).into(),
					operation.id,
					operation.max_children
				),
				Error::<Test>::ExceededRemovalBounds
			);
		});
}

#[test]
fn exact_children_max_revocations_revoke_and_remove_root_error() {
	let revoker = ed25519_did_from_seed(&ALICE_SEED);
	let delegate = ed25519_did_from_seed(&BOB_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details();
	let delegation1_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let delegation1_node =
		generate_base_delegation_node(hierarchy_root_id, delegate.clone(), Some(hierarchy_root_id), ACCOUNT_01);

	let delegation2_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let delegation2_node =
		generate_base_delegation_node(hierarchy_root_id, delegate.clone(), Some(delegation1_id), ACCOUNT_01);

	let delegation3_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_3);
	let delegation3_node = generate_base_delegation_node(hierarchy_root_id, delegate, Some(delegation1_id), ACCOUNT_01);

	let mut operation = generate_base_delegation_hierarchy_revocation_operation(hierarchy_root_id);
	// set max children below required minimum of 3 to revoke/remove entire tree
	operation.max_children = 2u32;

	ExtBuilder::default()
		.with_balances(vec![
			(ACCOUNT_00, <Test as Config>::Deposit::get()),
			(ACCOUNT_01, <Test as Config>::Deposit::get() * 3),
		])
		.with_ctypes(vec![(hierarchy_details.ctype_hash, revoker.clone())])
		.with_delegation_hierarchies(vec![(
			hierarchy_root_id,
			hierarchy_details,
			revoker.clone(),
			ACCOUNT_00,
		)])
		.with_delegations(vec![
			(delegation1_id, delegation1_node),
			(delegation2_id, delegation2_node),
			(delegation3_id, delegation3_node),
		])
		.build()
		.execute_with(|| {
			assert_eq!(Balances::reserved_balance(ACCOUNT_00), <Test as Config>::Deposit::get());

			// Should not revoke root because tree traversal steps are insufficient
			// assert_err and not assert_noop because the storage is indeed changed, but
			// only partially (only #2 is revoked)
			assert_err!(
				Delegation::revoke_delegation(
					DoubleOrigin(ACCOUNT_00, revoker.clone()).into(),
					operation.id,
					0u32,
					operation.max_children
				),
				Error::<Test>::ExceededRevocationBounds
			);

			// No delegation should have been revoked because of transactional storage layer
			assert!(
				!Delegation::delegation_nodes(operation.id)
					.expect("Delegation root should be present on chain.")
					.details
					.revoked
			);
			assert!(
				!Delegation::delegation_nodes(delegation1_id)
					.expect("Delegation 1 should be present on chain.")
					.details
					.revoked
			);
			assert!(
				!Delegation::delegation_nodes(delegation2_id)
					.expect("Delegation 2 should be present on chain.")
					.details
					.revoked
			);
			assert!(
				!Delegation::delegation_nodes(delegation3_id)
					.expect("Delegation 3 should be present on chain.")
					.details
					.revoked
			);

			// Should not remove root because tree traversal steps are insufficient
			assert_eq!(Balances::reserved_balance(ACCOUNT_00), <Test as Config>::Deposit::get());
			assert_eq!(
				Balances::reserved_balance(ACCOUNT_01),
				3 * <Test as Config>::Deposit::get()
			);
			// assert_err and not assert_noop because the storage is indeed changed, but
			// only partially (only #2 is removed)
			assert_err!(
				Delegation::remove_delegation(
					DoubleOrigin(ACCOUNT_00, revoker.clone()).into(),
					operation.id,
					operation.max_children
				),
				Error::<Test>::ExceededRemovalBounds
			);
			// Should not remove any delegation because of transactional storage layer
			assert!(Delegation::delegation_nodes(operation.id).is_some());
			assert!(Delegation::delegation_nodes(delegation1_id).is_some());
			assert!(Delegation::delegation_nodes(delegation2_id).is_some());
			assert!(Delegation::delegation_nodes(delegation3_id).is_some());
			assert_eq!(Balances::reserved_balance(ACCOUNT_00), <Test as Config>::Deposit::get());
			assert_eq!(
				Balances::reserved_balance(ACCOUNT_01),
				3 * <Test as Config>::Deposit::get()
			);

			// Should be able to remove root only with depth = #_of_children + 1
			assert_ok!(Delegation::remove_delegation(
				DoubleOrigin(ACCOUNT_00, revoker.clone()).into(),
				operation.id,
				operation.max_children + 1
			));
			assert!(Delegation::delegation_nodes(operation.id).is_none());
			assert!(Delegation::delegation_nodes(delegation1_id).is_none());
			assert!(Delegation::delegation_nodes(delegation3_id).is_none());
			assert!(Balances::reserved_balance(ACCOUNT_00).is_zero());
			assert!(Balances::reserved_balance(ACCOUNT_01).is_zero());
		});
}

// submit_delegation_revocation_operation()

// difference to `max_max_revocations_revoke_and_remove_successful`: doesn't
// revoke hierarchy but direct child of hierarchy root
#[test]
fn direct_owner_revoke_and_remove_delegation_successful() {
	let revoker = ed25519_did_from_seed(&ALICE_SEED);
	let delegate = ed25519_did_from_seed(&BOB_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details();

	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let parent_node =
		generate_base_delegation_node(hierarchy_root_id, revoker.clone(), Some(hierarchy_root_id), ACCOUNT_00);

	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let delegation_node = generate_base_delegation_node(hierarchy_root_id, delegate, Some(parent_id), ACCOUNT_01);

	let mut operation = generate_base_delegation_revocation_operation(parent_id);
	operation.max_revocations = 2u32;

	ExtBuilder::default()
		.with_balances(vec![
			(ACCOUNT_00, <Test as Config>::Deposit::get() * 2),
			(ACCOUNT_01, <Test as Config>::Deposit::get()),
		])
		.with_ctypes(vec![(hierarchy_details.ctype_hash, revoker.clone())])
		.with_delegation_hierarchies(vec![(
			hierarchy_root_id,
			hierarchy_details,
			revoker.clone(),
			ACCOUNT_00,
		)])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.build()
		.execute_with(|| {
			// Revoke direct child of hierarchy root
			assert_ok!(Delegation::revoke_delegation(
				DoubleOrigin(ACCOUNT_00, revoker.clone()).into(),
				operation.delegation_id,
				operation.max_parent_checks,
				operation.max_revocations
			));

			// Root hierarchy and children should still be stored with revoked status
			assert!(
				Delegation::delegation_nodes(operation.delegation_id)
					.expect("Delegation root should be present on chain.")
					.details
					.revoked
			);
			assert!(
				Delegation::delegation_nodes(parent_id)
					.expect("Parent delegation should be present on chain.")
					.details
					.revoked
			);
			assert!(
				Delegation::delegation_nodes(delegation_id)
					.expect("Delegation should be present on chain.")
					.details
					.revoked
			);

			assert_eq!(
				Balances::reserved_balance(ACCOUNT_00),
				2 * <Test as Config>::Deposit::get()
			);
			assert_eq!(Balances::reserved_balance(ACCOUNT_01), <Test as Config>::Deposit::get());

			// Removing root delegation should also remove its child but not hierarchy root
			assert_ok!(Delegation::remove_delegation(
				DoubleOrigin(ACCOUNT_00, revoker.clone()).into(),
				operation.delegation_id,
				operation.max_revocations
			));

			assert!(Delegation::delegation_hierarchies(hierarchy_root_id).is_some());
			assert!(Delegation::delegation_nodes(operation.delegation_id).is_none());
			assert!(Delegation::delegation_nodes(parent_id).is_none());
			assert!(Delegation::delegation_nodes(delegation_id).is_none());
			assert_eq!(Balances::reserved_balance(ACCOUNT_00), <Test as Config>::Deposit::get());
			assert!(Balances::reserved_balance(ACCOUNT_01).is_zero());
		});
}

#[test]
fn parent_owner_revoke_delegation_successful() {
	let revoker = ed25519_did_from_seed(&ALICE_SEED);
	let delegate = ed25519_did_from_seed(&BOB_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details();

	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let parent_node =
		generate_base_delegation_node(hierarchy_root_id, revoker.clone(), Some(hierarchy_root_id), ACCOUNT_00);

	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let delegation_node =
		generate_base_delegation_node(hierarchy_root_id, delegate.clone(), Some(parent_id), ACCOUNT_01);

	let mut operation = generate_base_delegation_revocation_operation(delegation_id);
	operation.max_parent_checks = 1u32;
	operation.max_revocations = 1u32;

	ExtBuilder::default()
		.with_balances(vec![
			(ACCOUNT_00, <Test as Config>::Deposit::get() * 2),
			(ACCOUNT_01, <Test as Config>::Deposit::get()),
		])
		.with_ctypes(vec![(hierarchy_details.ctype_hash, revoker.clone())])
		.with_delegation_hierarchies(vec![(
			hierarchy_root_id,
			hierarchy_details,
			revoker.clone(),
			ACCOUNT_00,
		)])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.build()
		.execute_with(|| {
			// Parent should not be able to remove the child delegation directly
			assert_eq!(
				Balances::reserved_balance(ACCOUNT_00),
				2 * <Test as Config>::Deposit::get()
			);
			assert_eq!(Balances::reserved_balance(ACCOUNT_01), <Test as Config>::Deposit::get());
			assert_noop!(
				Delegation::remove_delegation(
					DoubleOrigin(ACCOUNT_00, revoker.clone()).into(),
					operation.delegation_id,
					operation.max_revocations
				),
				Error::<Test>::UnauthorizedRemoval
			);

			// Revoke direct child of hierarchy root
			assert_ok!(Delegation::revoke_delegation(
				DoubleOrigin(ACCOUNT_00, revoker.clone()).into(),
				operation.delegation_id,
				operation.max_parent_checks,
				operation.max_revocations
			));

			// Only child should be revoked
			assert!(
				!Delegation::delegation_nodes(parent_id)
					.expect("Parent delegation should be present on chain.")
					.details
					.revoked
			);
			assert!(
				Delegation::delegation_nodes(delegation_id)
					.expect("Delegation should be present on chain.")
					.details
					.revoked
			);

			// Only owner can still remove the delegation to claim back deposit
			assert_ok!(Delegation::remove_delegation(
				DoubleOrigin(ACCOUNT_01, delegate.clone()).into(),
				operation.delegation_id,
				operation.max_revocations
			));

			assert!(Delegation::delegation_nodes(parent_id).is_some());
			assert!(Delegation::delegation_nodes(delegation_id).is_none());
			assert_eq!(
				Balances::reserved_balance(ACCOUNT_00),
				2 * <Test as Config>::Deposit::get()
			);
			assert!(Balances::reserved_balance(ACCOUNT_01).is_zero());
		});
}

#[test]
fn delegation_not_found_revoke_and_remove_delegation_error() {
	let revoker = ed25519_did_from_seed(&ALICE_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details();
	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);

	let operation = generate_base_delegation_revocation_operation(delegation_id);

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get())])
		.with_ctypes(vec![(hierarchy_details.ctype_hash, revoker.clone())])
		.with_delegation_hierarchies(vec![(
			hierarchy_root_id,
			hierarchy_details,
			revoker.clone(),
			ACCOUNT_00,
		)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Delegation::revoke_delegation(
					DoubleOrigin(ACCOUNT_00, revoker.clone()).into(),
					operation.delegation_id,
					operation.max_parent_checks,
					operation.max_revocations
				),
				Error::<Test>::DelegationNotFound
			);
			assert_noop!(
				Delegation::remove_delegation(
					DoubleOrigin(ACCOUNT_00, revoker.clone()).into(),
					operation.delegation_id,
					operation.max_revocations
				),
				Error::<Test>::DelegationNotFound
			);
		});
}

#[test]
fn not_delegating_revoke_and_remove_delegation_error() {
	let owner = ed25519_did_from_seed(&ALICE_SEED);
	let revoker = ed25519_did_from_seed(&BOB_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details();

	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let delegation_node =
		generate_base_delegation_node(hierarchy_root_id, owner.clone(), Some(hierarchy_root_id), ACCOUNT_00);

	let mut operation = generate_base_delegation_revocation_operation(delegation_id);
	operation.max_parent_checks = MaxParentChecks::get();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get() * 2)])
		.with_ctypes(vec![(hierarchy_details.ctype_hash, owner.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, owner, ACCOUNT_00)])
		.with_delegations(vec![(delegation_id, delegation_node)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Delegation::revoke_delegation(
					DoubleOrigin(ACCOUNT_01, revoker.clone()).into(),
					operation.delegation_id,
					operation.max_parent_checks,
					operation.max_revocations
				),
				Error::<Test>::UnauthorizedRevocation
			);
			assert_noop!(
				Delegation::remove_delegation(
					DoubleOrigin(ACCOUNT_01, revoker.clone()).into(),
					operation.delegation_id,
					operation.max_revocations
				),
				Error::<Test>::UnauthorizedRemoval
			);
		});
}

#[test]
fn parent_too_far_revoke_and_remove_delegation_error() {
	let owner = ed25519_did_from_seed(&ALICE_SEED);
	let delegate = ed25519_did_from_seed(&BOB_SEED);
	let intermediate = ed25519_did_from_seed(&CHARLIE_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details();
	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let parent_node = generate_base_delegation_node(
		hierarchy_root_id,
		intermediate.clone(),
		Some(hierarchy_root_id),
		ACCOUNT_01,
	);

	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let delegation_node =
		generate_base_delegation_node(hierarchy_root_id, delegate.clone(), Some(parent_id), ACCOUNT_02);

	let mut operation = generate_base_delegation_revocation_operation(delegation_id);
	operation.max_revocations = 2u32;
	operation.max_parent_checks = 0u32;

	ExtBuilder::default()
		.with_balances(vec![
			(ACCOUNT_00, <Test as Config>::Deposit::get()),
			(ACCOUNT_01, <Test as Config>::Deposit::get()),
			(ACCOUNT_02, <Test as Config>::Deposit::get()),
		])
		.with_ctypes(vec![(hierarchy_details.ctype_hash, owner.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, owner, ACCOUNT_00)])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Delegation::revoke_delegation(
					DoubleOrigin(ACCOUNT_01, intermediate.clone()).into(),
					operation.delegation_id,
					operation.max_parent_checks,
					operation.max_revocations
				),
				Error::<Test>::MaxSearchDepthReached
			);

			// removal
			assert_eq!(Balances::reserved_balance(ACCOUNT_00), <Test as Config>::Deposit::get());
			assert_eq!(Balances::reserved_balance(ACCOUNT_02), <Test as Config>::Deposit::get());
			assert_eq!(Balances::reserved_balance(ACCOUNT_01), <Test as Config>::Deposit::get());

			assert_noop!(
				Delegation::remove_delegation(
					DoubleOrigin(ACCOUNT_01, intermediate.clone()).into(),
					operation.delegation_id,
					operation.max_revocations
				),
				Error::<Test>::UnauthorizedRemoval
			);
			assert_noop!(
				Delegation::remove_delegation(
					DoubleOrigin(ACCOUNT_01, intermediate.clone()).into(),
					operation.delegation_id,
					operation.max_revocations
				),
				Error::<Test>::UnauthorizedRemoval
			);
			assert_ok!(Delegation::remove_delegation(
				DoubleOrigin(ACCOUNT_02, delegate.clone()).into(),
				operation.delegation_id,
				0u32
			));
		});
}

#[test]
fn too_many_revocations_revoke_delegation_error() {
	let revoker = ed25519_did_from_seed(&ALICE_SEED);
	let delegate = ed25519_did_from_seed(&BOB_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details();
	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let parent_node =
		generate_base_delegation_node(hierarchy_root_id, revoker.clone(), Some(hierarchy_root_id), ACCOUNT_00);
	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let delegation_node = generate_base_delegation_node(hierarchy_root_id, delegate, Some(parent_id), ACCOUNT_00);

	let operation = generate_base_delegation_revocation_operation(parent_id);

	ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, revoker.clone())])
		.with_delegation_hierarchies(vec![(
			hierarchy_root_id,
			hierarchy_details,
			revoker.clone(),
			ACCOUNT_00,
		)])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.with_balances(vec![
			(ACCOUNT_00, <Test as Config>::Deposit::get()),
			(ACCOUNT_01, <Test as Config>::Deposit::get()),
		])
		.build()
		.execute_with(|| {
			assert_noop!(
				Delegation::revoke_delegation(
					DoubleOrigin(ACCOUNT_00, revoker.clone()).into(),
					operation.delegation_id,
					operation.max_parent_checks,
					operation.max_revocations
				),
				Error::<Test>::ExceededRevocationBounds
			);
		});
}

// reclaim_deposit

#[test]
fn direct_owner_reclaim_deposit_delegation_successful() {
	let revoker = ed25519_did_from_seed(&ALICE_SEED);
	let delegate = ed25519_did_from_seed(&BOB_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details();
	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let parent_node =
		generate_base_delegation_node(hierarchy_root_id, revoker.clone(), Some(hierarchy_root_id), ACCOUNT_00);
	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let delegation_node = generate_base_delegation_node(hierarchy_root_id, delegate, Some(parent_id), ACCOUNT_00);

	let mut operation = generate_base_delegation_deposit_claim_operation(hierarchy_root_id);
	operation.max_removals = 2u32;

	ExtBuilder::default()
		.with_balances(vec![
			(ACCOUNT_00, <Test as Config>::Deposit::get() * 2),
			(ACCOUNT_01, <Test as Config>::Deposit::get()),
		])
		.with_ctypes(vec![(hierarchy_details.ctype_hash, revoker.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, revoker, ACCOUNT_00)])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.build()
		.execute_with(|| {
			// Revoke direct child of hierarchy root
			assert_ok!(Delegation::reclaim_deposit(
				RuntimeOrigin::signed(ACCOUNT_00),
				operation.delegation_id,
				operation.max_removals
			));

			// Root hierarchy and children should not be stored anymore
			assert!(Delegation::delegation_nodes(operation.delegation_id).is_none());
			assert!(Delegation::delegation_nodes(parent_id).is_none());
			assert!(Delegation::delegation_nodes(delegation_id).is_none());

			// We have released all the deposits by deleting the root node.
			assert!(Balances::reserved_balance(ACCOUNT_00).is_zero());
			assert!(Balances::reserved_balance(ACCOUNT_01).is_zero());
		});
}

// Implicitely checks the case where deposit owner != signed origin
#[test]
fn parent_owner_reclaim_deposit_error() {
	let revoker = ed25519_did_from_seed(&ALICE_SEED);
	let delegate = ed25519_did_from_seed(&BOB_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details();
	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let parent_node =
		generate_base_delegation_node(hierarchy_root_id, revoker.clone(), Some(hierarchy_root_id), ACCOUNT_00);
	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let delegation_node = generate_base_delegation_node(hierarchy_root_id, delegate, Some(parent_id), ACCOUNT_01);

	let mut operation = generate_base_delegation_deposit_claim_operation(delegation_id);
	operation.max_removals = 1u32;

	ExtBuilder::default()
		.with_balances(vec![
			(ACCOUNT_00, <Test as Config>::Deposit::get() * 2),
			(ACCOUNT_01, <Test as Config>::Deposit::get()),
		])
		.with_ctypes(vec![(hierarchy_details.ctype_hash, revoker.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, revoker, ACCOUNT_00)])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.build()
		.execute_with(|| {
			// Parent should not be able to claim the deposit for the child delegation
			// directly
			assert_eq!(
				Balances::reserved_balance(ACCOUNT_00),
				2 * <Test as Config>::Deposit::get()
			);
			assert_eq!(Balances::reserved_balance(ACCOUNT_01), <Test as Config>::Deposit::get());
			assert_noop!(
				Delegation::reclaim_deposit(
					RuntimeOrigin::signed(ACCOUNT_00),
					operation.delegation_id,
					operation.max_removals
				),
				Error::<Test>::UnauthorizedRemoval
			);
		});
}

#[test]
fn delegation_not_found_reclaim_deposit_error() {
	let revoker = ed25519_did_from_seed(&ALICE_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details();
	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);

	let operation = generate_base_delegation_deposit_claim_operation(delegation_id);

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get())])
		.with_ctypes(vec![(hierarchy_details.ctype_hash, revoker.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, revoker, ACCOUNT_00)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Delegation::reclaim_deposit(
					RuntimeOrigin::signed(ACCOUNT_00),
					operation.delegation_id,
					operation.max_removals
				),
				Error::<Test>::DelegationNotFound
			);
		});
}

#[test]
fn max_removals_too_large_reclaim_deposit_error() {
	let revoker = ed25519_did_from_seed(&ALICE_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details();

	let mut operation = generate_base_delegation_deposit_claim_operation(hierarchy_root_id);
	operation.max_removals = <Test as Config>::MaxRemovals::get() + 1;

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get())])
		.with_ctypes(vec![(hierarchy_details.ctype_hash, revoker.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, revoker, ACCOUNT_00)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Delegation::reclaim_deposit(
					RuntimeOrigin::signed(ACCOUNT_00),
					operation.delegation_id,
					operation.max_removals
				),
				Error::<Test>::MaxRemovalsTooLarge
			);
		});
}

// Internal function: is_delegating()

#[test]
fn is_delegating_direct_not_revoked() {
	let user_1 = ed25519_did_from_seed(&ALICE_SEED);
	let user_2 = ed25519_did_from_seed(&BOB_SEED);
	let user_3 = ed25519_did_from_seed(&CHARLIE_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details();
	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let parent_node = generate_base_delegation_node(hierarchy_root_id, user_2, Some(hierarchy_root_id), ACCOUNT_00);

	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let delegation_node = generate_base_delegation_node(hierarchy_root_id, user_3.clone(), Some(parent_id), ACCOUNT_01);

	let max_parent_checks = 0u32;

	// Root -> Parent -> Delegation
	ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, user_1.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, user_1, ACCOUNT_00)])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.with_balances(vec![
			(ACCOUNT_00, <Test as Config>::Deposit::get()),
			(ACCOUNT_01, <Test as Config>::Deposit::get()),
			(ACCOUNT_02, <Test as Config>::Deposit::get()),
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
	let user_1 = ed25519_did_from_seed(&ALICE_SEED);
	let user_2 = ed25519_did_from_seed(&BOB_SEED);
	let user_3 = ed25519_did_from_seed(&CHARLIE_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details();
	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let parent_node = generate_base_delegation_node(hierarchy_root_id, user_2, Some(hierarchy_root_id), ACCOUNT_00);

	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let delegation_node = generate_base_delegation_node(hierarchy_root_id, user_3.clone(), Some(parent_id), ACCOUNT_01);

	let max_parent_checks = u32::MAX;

	// Root -> Parent -> Delegation
	ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, user_1.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, user_1, ACCOUNT_00)])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.with_balances(vec![
			(ACCOUNT_00, <Test as Config>::Deposit::get()),
			(ACCOUNT_01, <Test as Config>::Deposit::get()),
			(ACCOUNT_02, <Test as Config>::Deposit::get()),
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
	let user_1 = ed25519_did_from_seed(&ALICE_SEED);
	let user_2 = ed25519_did_from_seed(&BOB_SEED);
	let user_3 = ed25519_did_from_seed(&CHARLIE_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details();
	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let parent_node = generate_base_delegation_node(hierarchy_root_id, user_2, Some(hierarchy_root_id), ACCOUNT_00);

	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let mut delegation_node =
		generate_base_delegation_node(hierarchy_root_id, user_3.clone(), Some(parent_id), ACCOUNT_00);
	delegation_node.details.revoked = true;

	let max_parent_checks = 0u32;

	// Root -> Parent -> Delegation
	ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, user_1.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, user_1, ACCOUNT_00)])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.with_balances(vec![
			(ACCOUNT_00, <Test as Config>::Deposit::get()),
			(ACCOUNT_01, <Test as Config>::Deposit::get()),
			(ACCOUNT_02, <Test as Config>::Deposit::get()),
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
	let user_1 = ed25519_did_from_seed(&ALICE_SEED);
	let user_2 = ed25519_did_from_seed(&BOB_SEED);
	let user_3 = ed25519_did_from_seed(&CHARLIE_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details();
	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let parent_node = generate_base_delegation_node(hierarchy_root_id, user_2, Some(hierarchy_root_id), ACCOUNT_00);

	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let mut delegation_node =
		generate_base_delegation_node(hierarchy_root_id, user_3.clone(), Some(parent_id), ACCOUNT_00);
	delegation_node.details.revoked = true;

	let max_parent_checks = u32::MAX;

	// Root -> Parent -> Delegation
	ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, user_1.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, user_1, ACCOUNT_00)])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.with_balances(vec![
			(ACCOUNT_00, <Test as Config>::Deposit::get()),
			(ACCOUNT_01, <Test as Config>::Deposit::get()),
			(ACCOUNT_02, <Test as Config>::Deposit::get()),
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
	let user_1 = ed25519_did_from_seed(&ALICE_SEED);
	let user_2 = ed25519_did_from_seed(&BOB_SEED);
	let user_3 = ed25519_did_from_seed(&CHARLIE_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details();
	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let parent_node =
		generate_base_delegation_node(hierarchy_root_id, user_2.clone(), Some(hierarchy_root_id), ACCOUNT_00);

	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let delegation_node = generate_base_delegation_node(hierarchy_root_id, user_3, Some(parent_id), ACCOUNT_01);

	let max_parent_checks = 1u32;

	// Root -> Parent -> Delegation
	ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, user_1.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, user_1, ACCOUNT_00)])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.with_balances(vec![
			(ACCOUNT_00, <Test as Config>::Deposit::get()),
			(ACCOUNT_01, <Test as Config>::Deposit::get()),
			(ACCOUNT_02, <Test as Config>::Deposit::get()),
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
	let user_1 = ed25519_did_from_seed(&ALICE_SEED);
	let user_2 = ed25519_did_from_seed(&BOB_SEED);
	let user_3 = ed25519_did_from_seed(&CHARLIE_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details();

	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let mut parent_node =
		generate_base_delegation_node(hierarchy_root_id, user_2.clone(), Some(hierarchy_root_id), ACCOUNT_00);

	parent_node.details.revoked = true;
	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let delegation_node = generate_base_delegation_node(hierarchy_root_id, user_3, Some(parent_id), ACCOUNT_01);

	let max_parent_checks = 2u32;

	// Root -> Parent -> Delegation
	ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, user_1.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, user_1, ACCOUNT_00)])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.with_balances(vec![
			(ACCOUNT_00, <Test as Config>::Deposit::get()),
			(ACCOUNT_01, <Test as Config>::Deposit::get()),
			(ACCOUNT_02, <Test as Config>::Deposit::get()),
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
	let user_1 = ed25519_did_from_seed(&ALICE_SEED);
	let user_2 = ed25519_did_from_seed(&BOB_SEED);
	let user_3 = ed25519_did_from_seed(&CHARLIE_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details();

	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let parent_node = generate_base_delegation_node(hierarchy_root_id, user_2, Some(hierarchy_root_id), ACCOUNT_00);

	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let delegation_node = generate_base_delegation_node(hierarchy_root_id, user_3, Some(parent_id), ACCOUNT_01);

	let max_parent_checks = 2u32;

	// Root -> Parent -> Delegation
	ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, user_1.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, user_1.clone(), ACCOUNT_00)])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.with_balances(vec![
			(ACCOUNT_00, <Test as Config>::Deposit::get()),
			(ACCOUNT_01, <Test as Config>::Deposit::get()),
			(ACCOUNT_02, <Test as Config>::Deposit::get()),
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
	let user_1 = ed25519_did_from_seed(&ALICE_SEED);
	let user_2 = ed25519_did_from_seed(&BOB_SEED);
	let user_3 = ed25519_did_from_seed(&CHARLIE_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details();
	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let parent_node = generate_base_delegation_node(hierarchy_root_id, user_2, Some(hierarchy_root_id), ACCOUNT_00);

	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let delegation_node = generate_base_delegation_node(hierarchy_root_id, user_3, Some(parent_id), ACCOUNT_01);

	let max_parent_checks = 2u32;

	// Root -> Parent -> Delegation
	ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, user_1.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, user_1.clone(), ACCOUNT_00)])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.with_balances(vec![
			(ACCOUNT_00, <Test as Config>::Deposit::get()),
			(ACCOUNT_01, <Test as Config>::Deposit::get()),
			(ACCOUNT_02, <Test as Config>::Deposit::get()),
		])
		.build()
		.execute_with(|| {
			// First revoke the hierarchy, then test is_delegating.
			let _ = Delegation::revoke_delegation(
				DoubleOrigin(ACCOUNT_00, user_1.clone()).into(),
				hierarchy_root_id,
				0u32,
				2,
			);

			assert_eq!(
				Delegation::is_delegating(&user_1, &delegation_id, max_parent_checks),
				Ok((false, 0u32))
			);
		});
}

#[test]
fn is_delegating_delegation_not_found() {
	let user_1 = ed25519_did_from_seed(&ALICE_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details();
	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);

	let max_parent_checks = 2u32;

	// Root -> Delegation 1
	let mut ext = ExtBuilder::default()
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, user_1.clone(), ACCOUNT_00)])
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
	let user_1 = ed25519_did_from_seed(&ALICE_SEED);
	let user_2 = ed25519_did_from_seed(&BOB_SEED);
	let user_3 = ed25519_did_from_seed(&CHARLIE_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details();

	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let parent_node = generate_base_delegation_node(hierarchy_root_id, user_2, Some(hierarchy_root_id), ACCOUNT_00);

	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let delegation_node = generate_base_delegation_node(hierarchy_root_id, user_3, Some(parent_id), ACCOUNT_01);

	// 1 less than needed
	let max_parent_checks = 1u32;

	// Root -> Parent -> Delegation
	ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, user_1.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, user_1.clone(), ACCOUNT_00)])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.with_balances(vec![
			(ACCOUNT_00, <Test as Config>::Deposit::get()),
			(ACCOUNT_01, <Test as Config>::Deposit::get()),
			(ACCOUNT_02, <Test as Config>::Deposit::get()),
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
	let creator = ed25519_did_from_seed(&ALICE_SEED);
	let attacker = ed25519_did_from_seed(&CHARLIE_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get())])
		.with_ctypes(vec![(hierarchy_details.ctype_hash, creator.clone())])
		.with_delegation_hierarchies(vec![(
			hierarchy_root_id,
			hierarchy_details,
			creator.clone(),
			ACCOUNT_00,
		)])
		.build()
		.execute_with(|| {
			assert!(Delegation::delegation_hierarchies(hierarchy_root_id).is_some());
			assert!(Delegation::delegation_nodes(hierarchy_root_id).is_some());
			assert_eq!(Balances::reserved_balance(ACCOUNT_00), <Test as Config>::Deposit::get());

			// Remove
			assert_noop!(
				Delegation::remove_delegation(
					DoubleOrigin(ACCOUNT_00, attacker.clone()).into(),
					hierarchy_root_id,
					0u32
				),
				Error::<Test>::UnauthorizedRemoval
			);
			assert_ok!(Delegation::remove_delegation(
				DoubleOrigin(ACCOUNT_00, creator.clone()).into(),
				hierarchy_root_id,
				0
			));
			assert!(Delegation::delegation_hierarchies(hierarchy_root_id).is_none());
			assert!(Delegation::delegation_nodes(hierarchy_root_id).is_none());
			assert!(Balances::reserved_balance(ACCOUNT_00).is_zero());
		});
}

#[test]
fn remove_children_gas_runs_out() {
	let revoker = ed25519_did_from_seed(&ALICE_SEED);
	let delegate = ed25519_did_from_seed(&BOB_SEED);
	let child = ed25519_did_from_seed(&CHARLIE_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details();
	let delegation1_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let delegation1_node =
		generate_base_delegation_node(hierarchy_root_id, delegate.clone(), Some(hierarchy_root_id), ACCOUNT_01);

	let delegation2_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let delegation2_node =
		generate_base_delegation_node(hierarchy_root_id, delegate.clone(), Some(delegation1_id), ACCOUNT_01);

	let delegation3_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_3);
	let delegation3_node = generate_base_delegation_node(hierarchy_root_id, delegate, Some(delegation1_id), ACCOUNT_01);

	let delegation4_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_4);
	let delegation4_node = generate_base_delegation_node(hierarchy_root_id, child, Some(delegation3_id), ACCOUNT_02);

	let mut operation = generate_base_delegation_hierarchy_revocation_operation(hierarchy_root_id);
	// set max children below required minimum of 4 to revoke/remove entire tree
	operation.max_children = 3u32;

	ExtBuilder::default()
		.with_balances(vec![
			(ACCOUNT_00, <Test as Config>::Deposit::get()),
			(ACCOUNT_01, <Test as Config>::Deposit::get() * 3),
			(ACCOUNT_02, <Test as Config>::Deposit::get()),
		])
		.with_ctypes(vec![(hierarchy_details.ctype_hash, revoker.clone())])
		.with_delegation_hierarchies(vec![(
			hierarchy_root_id,
			hierarchy_details,
			revoker.clone(),
			ACCOUNT_00,
		)])
		.with_delegations(vec![
			(delegation1_id, delegation1_node),
			(delegation2_id, delegation2_node),
			(delegation3_id, delegation3_node),
			(delegation4_id, delegation4_node),
		])
		.build()
		.execute_with(|| {
			assert_eq!(Balances::reserved_balance(ACCOUNT_00), <Test as Config>::Deposit::get());
			assert_eq!(
				Balances::reserved_balance(ACCOUNT_01),
				3 * <Test as Config>::Deposit::get()
			);
			assert_eq!(Balances::reserved_balance(ACCOUNT_02), <Test as Config>::Deposit::get());

			// Should not be able to remove root because tree traversal steps are
			// insufficient
			// assert_err and not assert_noop because the storage is indeed changed, but
			// only partially (only #4 is removed)
			assert_err!(
				Delegation::remove_delegation(
					DoubleOrigin(ACCOUNT_00, revoker.clone()).into(),
					operation.id,
					operation.max_children
				),
				Error::<Test>::ExceededRemovalBounds
			);
			assert!(Delegation::delegation_nodes(operation.id).is_some());
			assert!(Delegation::delegation_nodes(delegation1_id).is_some());
			// Should still be existing because of transactional storage
			assert!(Delegation::delegation_nodes(delegation2_id).is_some());
			assert!(Delegation::delegation_nodes(delegation3_id).is_some());
			assert!(Delegation::delegation_nodes(delegation4_id).is_some());
			assert_eq!(Balances::reserved_balance(ACCOUNT_00), <Test as Config>::Deposit::get());
			assert_eq!(
				Balances::reserved_balance(ACCOUNT_01),
				3 * <Test as Config>::Deposit::get()
			);
			assert_eq!(Balances::reserved_balance(ACCOUNT_02), <Test as Config>::Deposit::get());

			// Should be able to remove root only with depth = #_of_children + 1
			assert_ok!(Delegation::remove_delegation(
				DoubleOrigin(ACCOUNT_00, revoker.clone()).into(),
				operation.id,
				operation.max_children + 1
			));
			assert!(Delegation::delegation_nodes(operation.id).is_none());
			assert!(Delegation::delegation_nodes(delegation1_id).is_none());
			assert!(Delegation::delegation_nodes(delegation2_id).is_none());
			assert!(Delegation::delegation_nodes(delegation3_id).is_none());
			assert!(Balances::reserved_balance(ACCOUNT_00).is_zero());
			assert!(Balances::reserved_balance(ACCOUNT_01).is_zero());
			assert!(Balances::reserved_balance(ACCOUNT_02).is_zero());
		});
}

// #############################################################################
// transfer deposit

#[test]
fn test_change_deposit_owner() {
	let root_owner = ed25519_did_from_seed(&ALICE_SEED);
	let delegate = ed25519_did_from_seed(&BOB_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details();
	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let parent_node = generate_base_delegation_node(
		hierarchy_root_id,
		root_owner.clone(),
		Some(hierarchy_root_id),
		ACCOUNT_00,
	);
	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let delegation_node =
		generate_base_delegation_node(hierarchy_root_id, delegate.clone(), Some(parent_id), ACCOUNT_00);

	ExtBuilder::default()
		.with_balances(vec![
			(ACCOUNT_00, <Test as Config>::Deposit::get() * 2),
			(ACCOUNT_01, <Test as Config>::Deposit::get()),
		])
		.with_ctypes(vec![(hierarchy_details.ctype_hash, root_owner.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, root_owner, ACCOUNT_00)])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.build()
		.execute_with(|| {
			assert_eq!(
				Balances::reserved_balance(ACCOUNT_00),
				<Test as Config>::Deposit::get() * 3
			);
			assert!(Balances::reserved_balance(ACCOUNT_01).is_zero());
			assert_ok!(Delegation::change_deposit_owner(
				DoubleOrigin(ACCOUNT_01, delegate).into(),
				delegation_id
			));

			// ACCOUNT_00 has still one deposit (there are two nodes)
			assert_eq!(
				Balances::reserved_balance(ACCOUNT_00),
				<Test as Config>::Deposit::get() * 2
			);
			assert_eq!(Balances::reserved_balance(ACCOUNT_01), <Test as Config>::Deposit::get());
		});
}

#[test]
fn test_change_deposit_owner_insufficient_balance() {
	let root_owner = ed25519_did_from_seed(&ALICE_SEED);
	let delegate = ed25519_did_from_seed(&BOB_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details();
	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let parent_node = generate_base_delegation_node(
		hierarchy_root_id,
		root_owner.clone(),
		Some(hierarchy_root_id),
		ACCOUNT_00,
	);
	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let delegation_node =
		generate_base_delegation_node(hierarchy_root_id, delegate.clone(), Some(parent_id), ACCOUNT_00);

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get() * 2)])
		.with_ctypes(vec![(hierarchy_details.ctype_hash, root_owner.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, root_owner, ACCOUNT_00)])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.build()
		.execute_with(|| {
			assert_eq!(
				Balances::reserved_balance(ACCOUNT_00),
				<Test as Config>::Deposit::get() * 3
			);
			assert!(Balances::reserved_balance(ACCOUNT_01).is_zero());
			assert_noop!(
				Delegation::change_deposit_owner(DoubleOrigin(ACCOUNT_01, delegate).into(), delegation_id),
				pallet_balances::Error::<Test>::InsufficientBalance
			);
		});
}

#[test]
fn test_change_deposit_owner_unauthorized() {
	let root_owner = ed25519_did_from_seed(&ALICE_SEED);
	let delegate = ed25519_did_from_seed(&BOB_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details();
	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let parent_node = generate_base_delegation_node(
		hierarchy_root_id,
		root_owner.clone(),
		Some(hierarchy_root_id),
		ACCOUNT_00,
	);
	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let delegation_node = generate_base_delegation_node(hierarchy_root_id, delegate, Some(parent_id), ACCOUNT_00);

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get() * 2)])
		.with_ctypes(vec![(hierarchy_details.ctype_hash, root_owner.clone())])
		.with_delegation_hierarchies(vec![(
			hierarchy_root_id,
			hierarchy_details,
			root_owner.clone(),
			ACCOUNT_00,
		)])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Delegation::change_deposit_owner(DoubleOrigin(ACCOUNT_01, root_owner).into(), delegation_id),
				Error::<Test>::AccessDenied
			);
		});
}

#[test]
fn test_change_deposit_owner_not_found() {
	let root_owner = ed25519_did_from_seed(&ALICE_SEED);
	let delegate = ed25519_did_from_seed(&BOB_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details();
	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let parent_node = generate_base_delegation_node(
		hierarchy_root_id,
		root_owner.clone(),
		Some(hierarchy_root_id),
		ACCOUNT_00,
	);
	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get() * 2)])
		.with_ctypes(vec![(hierarchy_details.ctype_hash, root_owner.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, root_owner, ACCOUNT_00)])
		.with_delegations(vec![(parent_id, parent_node)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Delegation::change_deposit_owner(DoubleOrigin(ACCOUNT_01, delegate).into(), delegation_id),
				Error::<Test>::DelegationNotFound
			);
		});
}

/// Update the deposit amount
#[test]
fn test_update_deposit() {
	let root_owner = ed25519_did_from_seed(&ALICE_SEED);
	let delegate = ed25519_did_from_seed(&BOB_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details();
	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let parent_node = generate_base_delegation_node(
		hierarchy_root_id,
		root_owner.clone(),
		Some(hierarchy_root_id),
		ACCOUNT_00,
	);
	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let mut delegation_node = generate_base_delegation_node(hierarchy_root_id, delegate, Some(parent_id), ACCOUNT_00);
	delegation_node.deposit.amount = <Test as Config>::Deposit::get() * 2;

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get() * 4)])
		.with_ctypes(vec![(hierarchy_details.ctype_hash, root_owner.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, root_owner, ACCOUNT_00)])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.build()
		.execute_with(|| {
			assert_eq!(
				Balances::reserved_balance(ACCOUNT_00),
				<Test as Config>::Deposit::get() * 3
			);
			assert_ok!(Delegation::update_deposit(
				RuntimeOrigin::signed(ACCOUNT_00),
				delegation_id
			));

			// ACCOUNT_00 has still one deposit (there are two nodes)
			assert_eq!(
				Balances::reserved_balance(ACCOUNT_00),
				<Test as Config>::Deposit::get() * 2
			);
		});
}

#[test]
fn test_update_deposit_unauthorized() {
	let root_owner = ed25519_did_from_seed(&ALICE_SEED);
	let delegate = ed25519_did_from_seed(&BOB_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details();
	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let parent_node = generate_base_delegation_node(
		hierarchy_root_id,
		root_owner.clone(),
		Some(hierarchy_root_id),
		ACCOUNT_00,
	);
	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let mut delegation_node = generate_base_delegation_node(hierarchy_root_id, delegate, Some(parent_id), ACCOUNT_00);
	delegation_node.deposit.amount = <Test as Config>::Deposit::get() * 2;

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get() * 4)])
		.with_ctypes(vec![(hierarchy_details.ctype_hash, root_owner.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, root_owner, ACCOUNT_00)])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.build()
		.execute_with(|| {
			assert_eq!(
				Balances::reserved_balance(ACCOUNT_00),
				<Test as Config>::Deposit::get() * 3
			);
			assert_noop!(
				Delegation::update_deposit(RuntimeOrigin::signed(ACCOUNT_01), delegation_id),
				Error::<Test>::AccessDenied
			);
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
