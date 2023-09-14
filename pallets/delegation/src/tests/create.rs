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

use frame_support::{
	assert_noop, assert_ok,
	traits::fungible::{Inspect, InspectHold},
};
use kilt_support::mock::mock_origin::DoubleOrigin;
use sp_runtime::traits::Zero;

use crate::{self as delegation, mock::*, Config, Error, HoldReason};

#[test]
fn create_root_delegation_successful() {
	let creator = ed25519_did_from_seed(&ALICE_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let operation = generate_base_delegation_hierarchy_creation_operation::<Test>(hierarchy_root_id);

	ExtBuilder::default()
		.with_ctypes(vec![(operation.ctype_hash, creator.clone())])
		.with_balances(vec![(
			ACCOUNT_00,
			<Test as Config>::Deposit::get()
				+ <<Test as Config>::Currency as Inspect<delegation::AccountIdOf<Test>>>::minimum_balance(),
		)])
		.build_and_execute_with_sanity_tests(|| {
			// Create root hierarchy
			assert_ok!(Delegation::create_hierarchy(
				DoubleOrigin(ACCOUNT_00, creator.clone()).into(),
				operation.id,
				operation.ctype_hash
			));

			// Check reserved balance
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				<Test as Config>::Deposit::get()
			);

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
	let hierarchy_details = generate_base_delegation_hierarchy_details::<Test>();

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
		.build_and_execute_with_sanity_tests(|| {
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

#[test]
fn create_delegation_direct_root_successful() {
	let creator = ed25519_did_from_seed(&ALICE_SEED);
	let delegate = sr25519_did_from_seed(&BOB_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details::<Test>();

	ExtBuilder::default()
		.with_ctypes(vec![(hierarchy_details.ctype_hash, creator.clone())])
		.with_delegation_hierarchies(vec![(
			hierarchy_root_id,
			hierarchy_details,
			creator.clone(),
			ACCOUNT_00,
		)])
		.with_balances(vec![
			(
				ACCOUNT_00,
				<Test as Config>::Deposit::get()
					+ <<Test as Config>::Currency as Inspect<delegation::AccountIdOf<Test>>>::minimum_balance(),
			),
			(
				ACCOUNT_01,
				<Test as Config>::Deposit::get()
					+ <<Test as Config>::Currency as Inspect<delegation::AccountIdOf<Test>>>::minimum_balance(),
			),
		])
		.build_and_execute_with_sanity_tests(|| {
			// Create delegation to root
			let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
			let delegation_node = generate_base_delegation_node::<Test>(
				hierarchy_root_id,
				delegate.clone(),
				Some(hierarchy_root_id),
				ACCOUNT_01,
			);
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
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				<Test as Config>::Deposit::get()
			);

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
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
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
	let hierarchy_details = generate_base_delegation_hierarchy_details::<Test>();
	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let parent_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, creator.clone(), Some(hierarchy_root_id), ACCOUNT_00);

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
			(
				ACCOUNT_00,
				<Test as Config>::Deposit::get()
					+ <<Test as Config>::Currency as Inspect<delegation::AccountIdOf<Test>>>::minimum_balance(),
			),
			(
				ACCOUNT_01,
				<Test as Config>::Deposit::get()
					+ <<Test as Config>::Currency as Inspect<delegation::AccountIdOf<Test>>>::minimum_balance(),
			),
		])
		.build_and_execute_with_sanity_tests(|| {
			// Create sub-delegation
			let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
			let delegation_node =
				generate_base_delegation_node::<Test>(hierarchy_root_id, delegate.clone(), Some(parent_id), ACCOUNT_00);
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
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
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
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				3 * <Test as Config>::Deposit::get()
			);
			assert!(Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_01).is_zero());

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
fn invalid_delegate_signature_create_delegation_error() {
	let creator = ed25519_did_from_seed(&ALICE_SEED);
	let delegate = sr25519_did_from_seed(&BOB_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details::<Test>();
	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let delegation_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, delegate.clone(), Some(hierarchy_root_id), ACCOUNT_00);

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
		.build_and_execute_with_sanity_tests(|| {
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
		.build_and_execute_with_sanity_tests(|| {
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
		.build_and_execute_with_sanity_tests(|| {
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
	let hierarchy_details = generate_base_delegation_hierarchy_details::<Test>();
	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let parent_node = generate_base_delegation_node::<Test>(
		hierarchy_root_id,
		alternative_owner,
		Some(hierarchy_root_id),
		ACCOUNT_00,
	);

	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let delegation_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, delegate.clone(), Some(parent_id), ACCOUNT_00);

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
		.build_and_execute_with_sanity_tests(|| {
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
	let hierarchy_details = generate_base_delegation_hierarchy_details::<Test>();
	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let mut parent_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, creator.clone(), Some(hierarchy_root_id), ACCOUNT_00);
	parent_node.details.permissions = delegation::Permissions::ATTEST;

	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let delegation_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, delegate.clone(), Some(parent_id), ACCOUNT_00);

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
		.build_and_execute_with_sanity_tests(|| {
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
