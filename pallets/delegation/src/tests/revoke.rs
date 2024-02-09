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

use frame_support::{assert_noop, assert_ok};
use kilt_support::mock::mock_origin::DoubleOrigin;

use crate::{mock::*, Config, Error};

#[test]
fn create_delegation_direct_root_revoked_error() {
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
		.with_delegation_hierarchies(vec![(
			hierarchy_root_id,
			hierarchy_details,
			creator.clone(),
			ACCOUNT_00,
		)])
		.with_balances(vec![(ACCOUNT_00, <Test as Config>::Deposit::get())])
		.build_and_execute_with_sanity_tests(|| {
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
	let hierarchy_details = generate_base_delegation_hierarchy_details::<Test>();

	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let parent_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, creator.clone(), Some(hierarchy_root_id), ACCOUNT_00);

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
		.build_and_execute_with_sanity_tests(|| {
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
fn empty_revoke_root_successful() {
	let revoker = ed25519_did_from_seed(&ALICE_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details::<Test>();

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
		.build_and_execute_with_sanity_tests(|| {
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
fn too_many_revocations_revoke_delegation_error() {
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
		.build_and_execute_with_sanity_tests(|| {
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

#[test]
fn is_delegating_root_owner_revoked() {
	let user_1 = ed25519_did_from_seed(&ALICE_SEED);
	let user_2 = ed25519_did_from_seed(&BOB_SEED);
	let user_3 = ed25519_did_from_seed(&CHARLIE_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details::<Test>();
	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let parent_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, user_2, Some(hierarchy_root_id), ACCOUNT_00);

	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let delegation_node = generate_base_delegation_node::<Test>(hierarchy_root_id, user_3, Some(parent_id), ACCOUNT_01);

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
		.build_and_execute_with_sanity_tests(|| {
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
