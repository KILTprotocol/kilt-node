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
use sp_runtime::{traits::Zero, TokenError};

use crate::{self as delegation, mock::*, Config, Error, HoldReason};

#[test]
fn test_change_deposit_owner() {
	let root_owner = ed25519_did_from_seed(&ALICE_SEED);
	let delegate = ed25519_did_from_seed(&BOB_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details::<Test>();
	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let parent_node = generate_base_delegation_node::<Test>(
		hierarchy_root_id,
		root_owner.clone(),
		Some(hierarchy_root_id),
		ACCOUNT_00,
	);
	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let delegation_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, delegate.clone(), Some(parent_id), ACCOUNT_00);

	ExtBuilder::default()
		.with_balances(vec![
			(ACCOUNT_00, <Test as Config>::Deposit::get() * 2),
			(
				ACCOUNT_01,
				<Test as Config>::Deposit::get()
					+ <<Test as Config>::Currency as Inspect<delegation::AccountIdOf<Test>>>::minimum_balance(),
			),
		])
		.with_ctypes(vec![(hierarchy_details.ctype_hash, root_owner.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, root_owner, ACCOUNT_00)])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.build_and_execute_with_sanity_tests(|| {
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				<Test as Config>::Deposit::get() * 3
			);
			assert!(Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_01).is_zero());
			assert_ok!(Delegation::change_deposit_owner(
				DoubleOrigin(ACCOUNT_01, delegate).into(),
				delegation_id
			));

			// ACCOUNT_00 has still one deposit (there are two nodes)
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				<Test as Config>::Deposit::get() * 2
			);
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_01),
				<Test as Config>::Deposit::get()
			);
		});
}

#[test]
fn test_change_deposit_owner_insufficient_balance() {
	let root_owner = ed25519_did_from_seed(&ALICE_SEED);
	let delegate = ed25519_did_from_seed(&BOB_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details::<Test>();
	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let parent_node = generate_base_delegation_node::<Test>(
		hierarchy_root_id,
		root_owner.clone(),
		Some(hierarchy_root_id),
		ACCOUNT_00,
	);
	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let delegation_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, delegate.clone(), Some(parent_id), ACCOUNT_00);

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get() * 2)])
		.with_ctypes(vec![(hierarchy_details.ctype_hash, root_owner.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, root_owner, ACCOUNT_00)])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.build_and_execute_with_sanity_tests(|| {
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				<Test as Config>::Deposit::get() * 3
			);
			assert!(Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_01).is_zero());
			assert_noop!(
				Delegation::change_deposit_owner(DoubleOrigin(ACCOUNT_01, delegate).into(), delegation_id),
				TokenError::CannotCreateHold
			);
		});
}

#[test]
fn test_change_deposit_owner_unauthorized() {
	let root_owner = ed25519_did_from_seed(&ALICE_SEED);
	let delegate = ed25519_did_from_seed(&BOB_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details::<Test>();
	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let parent_node = generate_base_delegation_node::<Test>(
		hierarchy_root_id,
		root_owner.clone(),
		Some(hierarchy_root_id),
		ACCOUNT_00,
	);
	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let delegation_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, delegate, Some(parent_id), ACCOUNT_00);

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
		.build_and_execute_with_sanity_tests(|| {
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
	let hierarchy_details = generate_base_delegation_hierarchy_details::<Test>();
	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let parent_node = generate_base_delegation_node::<Test>(
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
		.build_and_execute_with_sanity_tests(|| {
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
	let hierarchy_details = generate_base_delegation_hierarchy_details::<Test>();
	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let parent_node = generate_base_delegation_node::<Test>(
		hierarchy_root_id,
		root_owner.clone(),
		Some(hierarchy_root_id),
		ACCOUNT_00,
	);
	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let mut delegation_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, delegate, Some(parent_id), ACCOUNT_00);
	delegation_node.deposit.amount = <Test as Config>::Deposit::get() * 2;

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get() * 4)])
		.with_ctypes(vec![(hierarchy_details.ctype_hash, root_owner.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, root_owner, ACCOUNT_00)])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.build_and_execute_with_sanity_tests(|| {
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				<Test as Config>::Deposit::get() * 3
			);
			assert_ok!(Delegation::update_deposit(
				RuntimeOrigin::signed(ACCOUNT_00),
				delegation_id
			));

			// ACCOUNT_00 has still one deposit (there are two nodes)
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				<Test as Config>::Deposit::get() * 2
			);
		});
}

#[test]
fn test_update_deposit_unauthorized() {
	let root_owner = ed25519_did_from_seed(&ALICE_SEED);
	let delegate = ed25519_did_from_seed(&BOB_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details::<Test>();
	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let parent_node = generate_base_delegation_node::<Test>(
		hierarchy_root_id,
		root_owner.clone(),
		Some(hierarchy_root_id),
		ACCOUNT_00,
	);
	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let mut delegation_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, delegate, Some(parent_id), ACCOUNT_00);
	delegation_node.deposit.amount = <Test as Config>::Deposit::get() * 2;

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get() * 4)])
		.with_ctypes(vec![(hierarchy_details.ctype_hash, root_owner.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, root_owner, ACCOUNT_00)])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.build_and_execute_with_sanity_tests(|| {
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				<Test as Config>::Deposit::get() * 3
			);
			assert_noop!(
				Delegation::update_deposit(RuntimeOrigin::signed(ACCOUNT_01), delegation_id),
				Error::<Test>::AccessDenied
			);
		});
}
#[test]
fn parent_owner_reclaim_deposit_error() {
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
		.build_and_execute_with_sanity_tests(|| {
			// Parent should not be able to claim the deposit for the child delegation
			// directly
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00),
				2 * <Test as Config>::Deposit::get()
			);
			assert_eq!(
				Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_01),
				<Test as Config>::Deposit::get()
			);
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
	let hierarchy_details = generate_base_delegation_hierarchy_details::<Test>();
	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);

	let operation = generate_base_delegation_deposit_claim_operation(delegation_id);

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get())])
		.with_ctypes(vec![(hierarchy_details.ctype_hash, revoker.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, revoker, ACCOUNT_00)])
		.build_and_execute_with_sanity_tests(|| {
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
	let hierarchy_details = generate_base_delegation_hierarchy_details::<Test>();

	let mut operation = generate_base_delegation_deposit_claim_operation(hierarchy_root_id);
	operation.max_removals = <Test as Config>::MaxRemovals::get() + 1;

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get())])
		.with_ctypes(vec![(hierarchy_details.ctype_hash, revoker.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, revoker, ACCOUNT_00)])
		.build_and_execute_with_sanity_tests(|| {
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

#[test]
fn direct_owner_reclaim_deposit_delegation_successful() {
	let revoker = ed25519_did_from_seed(&ALICE_SEED);
	let delegate = ed25519_did_from_seed(&BOB_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details::<Test>();
	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let parent_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, revoker.clone(), Some(hierarchy_root_id), ACCOUNT_00);
	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let delegation_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, delegate, Some(parent_id), ACCOUNT_00);

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
		.build_and_execute_with_sanity_tests(|| {
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
			assert!(Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_00).is_zero());
			assert!(Balances::balance_on_hold(&HoldReason::Deposit.into(), &ACCOUNT_01).is_zero());
		});
}
