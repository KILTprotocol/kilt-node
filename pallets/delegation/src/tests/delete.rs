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

use frame_support::{assert_err, assert_noop, assert_ok, traits::fungible::InspectHold};
use kilt_support::mock::mock_origin::DoubleOrigin;
use sp_runtime::traits::Zero;

use crate::{mock::*, Config, Error, HoldReason};

#[test]
fn parent_too_far_revoke_and_remove_delegation_error() {
	let owner = ed25519_did_from_seed(&ALICE_SEED);
	let delegate = ed25519_did_from_seed(&BOB_SEED);
	let intermediate = ed25519_did_from_seed(&CHARLIE_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details::<Test>();
	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let parent_node = generate_base_delegation_node::<Test>(
		hierarchy_root_id,
		intermediate.clone(),
		Some(hierarchy_root_id),
		ACCOUNT_01,
	);

	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let delegation_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, delegate.clone(), Some(parent_id), ACCOUNT_02);

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
		.build_and_execute_with_sanity_tests(|| {
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
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				<Test as Config>::Deposit::get()
			);
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_02),
				<Test as Config>::Deposit::get()
			);
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_01),
				<Test as Config>::Deposit::get()
			);

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
fn list_hierarchy_revoke_and_remove_root_successful() {
	let revoker = ed25519_did_from_seed(&ALICE_SEED);
	let delegate = sr25519_did_from_seed(&BOB_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details::<Test>();
	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let parent_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, revoker.clone(), Some(hierarchy_root_id), ACCOUNT_00);

	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let delegation_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, delegate, Some(parent_id), ACCOUNT_01);

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
		.build_and_execute_with_sanity_tests(|| {
			assert!(Delegation::delegation_hierarchies(hierarchy_root_id).is_some());
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				2 * <Test as Config>::Deposit::get()
			);
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_01),
				<Test as Config>::Deposit::get()
			);

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
			assert!(Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00).is_zero());
			assert!(Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_01).is_zero());
		});
}

#[test]
fn tree_hierarchy_revoke_and_remove_root_successful() {
	let revoker = ed25519_did_from_seed(&ALICE_SEED);
	let delegate = sr25519_did_from_seed(&BOB_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details::<Test>();
	let delegation1_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let delegation1_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, revoker.clone(), Some(hierarchy_root_id), ACCOUNT_00);

	let delegation2_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let delegation2_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, delegate, Some(hierarchy_root_id), ACCOUNT_01);

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
		.build_and_execute_with_sanity_tests(|| {
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
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				2 * <Test as Config>::Deposit::get()
			);
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_01),
				<Test as Config>::Deposit::get()
			);

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
			assert!(Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00).is_zero());
			assert!(Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_01).is_zero());
		});
}

#[test]
fn max_max_revocations_revoke_and_remove_successful() {
	let revoker = ed25519_did_from_seed(&ALICE_SEED);
	let delegate = sr25519_did_from_seed(&BOB_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details::<Test>();
	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let parent_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, revoker.clone(), Some(hierarchy_root_id), ACCOUNT_00);

	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let delegation_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, delegate, Some(parent_id), ACCOUNT_01);

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
		.build_and_execute_with_sanity_tests(|| {
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
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				2 * <Test as Config>::Deposit::get()
			);
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_01),
				<Test as Config>::Deposit::get()
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
			assert!(Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00).is_zero());
			assert!(Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_01).is_zero());
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
	let hierarchy_details = generate_base_delegation_hierarchy_details::<Test>();

	let operation = generate_base_delegation_hierarchy_revocation_operation(hierarchy_root_id);

	ExtBuilder::default()
		.with_balances(vec![
			(ACCOUNT_00, <Test as Config>::Deposit::get()),
			(ACCOUNT_01, <Test as Config>::Deposit::get()),
		])
		.with_ctypes(vec![(hierarchy_details.ctype_hash, owner.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, owner, ACCOUNT_00)])
		.build_and_execute_with_sanity_tests(|| {
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
	let hierarchy_details = generate_base_delegation_hierarchy_details::<Test>();

	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let delegation_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, delegate, Some(hierarchy_root_id), ACCOUNT_00);

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
		.build_and_execute_with_sanity_tests(|| {
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
	let hierarchy_details = generate_base_delegation_hierarchy_details::<Test>();
	let delegation1_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let delegation1_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, delegate.clone(), Some(hierarchy_root_id), ACCOUNT_01);

	let delegation2_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let delegation2_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, delegate.clone(), Some(delegation1_id), ACCOUNT_01);

	let delegation3_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_3);
	let delegation3_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, delegate, Some(delegation1_id), ACCOUNT_01);

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
		.build_and_execute_with_sanity_tests(|| {
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				<Test as Config>::Deposit::get()
			);

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
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				<Test as Config>::Deposit::get()
			);
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_01),
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
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				<Test as Config>::Deposit::get()
			);
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_01),
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
			assert!(Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00).is_zero());
			assert!(Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_01).is_zero());
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
	let hierarchy_details = generate_base_delegation_hierarchy_details::<Test>();

	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let parent_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, revoker.clone(), Some(hierarchy_root_id), ACCOUNT_00);

	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let delegation_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, delegate, Some(parent_id), ACCOUNT_01);

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
		.build_and_execute_with_sanity_tests(|| {
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
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				2 * <Test as Config>::Deposit::get()
			);
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_01),
				<Test as Config>::Deposit::get()
			);

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
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				<Test as Config>::Deposit::get()
			);
			assert!(Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_01).is_zero());
		});
}

#[test]
fn remove_single_hierarchy() {
	let creator = ed25519_did_from_seed(&ALICE_SEED);
	let attacker = ed25519_did_from_seed(&CHARLIE_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details::<Test>();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get())])
		.with_ctypes(vec![(hierarchy_details.ctype_hash, creator.clone())])
		.with_delegation_hierarchies(vec![(
			hierarchy_root_id,
			hierarchy_details,
			creator.clone(),
			ACCOUNT_00,
		)])
		.build_and_execute_with_sanity_tests(|| {
			assert!(Delegation::delegation_hierarchies(hierarchy_root_id).is_some());
			assert!(Delegation::delegation_nodes(hierarchy_root_id).is_some());
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				<Test as Config>::Deposit::get()
			);

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
			assert!(Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00).is_zero());
		});
}

#[test]
fn remove_children_gas_runs_out() {
	let revoker = ed25519_did_from_seed(&ALICE_SEED);
	let delegate = ed25519_did_from_seed(&BOB_SEED);
	let child = ed25519_did_from_seed(&CHARLIE_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details::<Test>();
	let delegation1_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let delegation1_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, delegate.clone(), Some(hierarchy_root_id), ACCOUNT_01);

	let delegation2_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let delegation2_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, delegate.clone(), Some(delegation1_id), ACCOUNT_01);

	let delegation3_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_3);
	let delegation3_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, delegate, Some(delegation1_id), ACCOUNT_01);

	let delegation4_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_4);
	let delegation4_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, child, Some(delegation3_id), ACCOUNT_02);

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
		.build_and_execute_with_sanity_tests(|| {
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				<Test as Config>::Deposit::get()
			);
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_01),
				3 * <Test as Config>::Deposit::get()
			);
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_02),
				<Test as Config>::Deposit::get()
			);

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
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				<Test as Config>::Deposit::get()
			);
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_01),
				3 * <Test as Config>::Deposit::get()
			);
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_02),
				<Test as Config>::Deposit::get()
			);

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
			assert!(Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00).is_zero());
			assert!(Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_01).is_zero());
			assert!(Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_02).is_zero());
		});
}

#[test]
fn delegation_not_found_revoke_and_remove_delegation_error() {
	let revoker = ed25519_did_from_seed(&ALICE_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details::<Test>();
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
		.build_and_execute_with_sanity_tests(|| {
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
	let hierarchy_details = generate_base_delegation_hierarchy_details::<Test>();

	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let delegation_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, owner.clone(), Some(hierarchy_root_id), ACCOUNT_00);

	let mut operation = generate_base_delegation_revocation_operation(delegation_id);
	operation.max_parent_checks = MaxParentChecks::get();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get() * 2)])
		.with_ctypes(vec![(hierarchy_details.ctype_hash, owner.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, owner, ACCOUNT_00)])
		.with_delegations(vec![(delegation_id, delegation_node)])
		.build_and_execute_with_sanity_tests(|| {
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
fn parent_owner_revoke_delegation_successful() {
	let revoker = ed25519_did_from_seed(&ALICE_SEED);
	let delegate = ed25519_did_from_seed(&BOB_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details::<Test>();

	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let parent_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, revoker.clone(), Some(hierarchy_root_id), ACCOUNT_00);

	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let delegation_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, delegate.clone(), Some(hierarchy_root_id), ACCOUNT_01);

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
		.build_and_execute_with_sanity_tests(|| {
			// Parent should not be able to remove the child delegation directly
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				2 * <Test as Config>::Deposit::get()
			);
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_01),
				<Test as Config>::Deposit::get()
			);
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
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				2 * <Test as Config>::Deposit::get()
			);
			assert!(Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_01).is_zero());
		});
}
