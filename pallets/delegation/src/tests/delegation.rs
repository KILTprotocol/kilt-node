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

use frame_support::{assert_noop, traits::fungible::Inspect};

use crate::{self as delegation, mock::*, Config, Error};

#[test]
fn is_delegating_delegation_not_found() {
	let user_1 = ed25519_did_from_seed(&ALICE_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details::<Test>();
	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);

	let max_parent_checks = 2u32;

	// Root -> Delegation 1
	let mut ext = ExtBuilder::default()
		.with_balances(vec![(
			ACCOUNT_00,
			<<Test as Config>::Currency as Inspect<delegation::AccountIdOf<Test>>>::minimum_balance(),
		)])
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
	let hierarchy_details = generate_base_delegation_hierarchy_details::<Test>();

	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let parent_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, user_2, Some(hierarchy_root_id), ACCOUNT_00);

	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let delegation_node = generate_base_delegation_node::<Test>(hierarchy_root_id, user_3, Some(parent_id), ACCOUNT_01);

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
		.build_and_execute_with_sanity_tests(|| {
			assert_noop!(
				Delegation::is_delegating(&user_1, &delegation_id, max_parent_checks),
				Error::<Test>::MaxSearchDepthReached
			);
		});
}

#[test]
fn is_delegating_direct_not_revoked() {
	let user_1 = ed25519_did_from_seed(&ALICE_SEED);
	let user_2 = ed25519_did_from_seed(&BOB_SEED);
	let user_3 = ed25519_did_from_seed(&CHARLIE_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details::<Test>();
	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let parent_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, user_2, Some(hierarchy_root_id), ACCOUNT_00);

	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let delegation_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, user_3.clone(), Some(parent_id), ACCOUNT_01);

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
		.build_and_execute_with_sanity_tests(|| {
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
	let hierarchy_details = generate_base_delegation_hierarchy_details::<Test>();
	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let parent_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, user_2, Some(hierarchy_root_id), ACCOUNT_00);

	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let delegation_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, user_3.clone(), Some(parent_id), ACCOUNT_01);

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
		.build_and_execute_with_sanity_tests(|| {
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
	let hierarchy_details = generate_base_delegation_hierarchy_details::<Test>();
	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let parent_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, user_2, Some(hierarchy_root_id), ACCOUNT_00);

	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let mut delegation_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, user_3.clone(), Some(parent_id), ACCOUNT_00);
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
		.build_and_execute_with_sanity_tests(|| {
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
	let hierarchy_details = generate_base_delegation_hierarchy_details::<Test>();
	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let parent_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, user_2, Some(hierarchy_root_id), ACCOUNT_00);

	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let mut delegation_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, user_3.clone(), Some(parent_id), ACCOUNT_00);
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
		.build_and_execute_with_sanity_tests(|| {
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
	let hierarchy_details = generate_base_delegation_hierarchy_details::<Test>();
	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let parent_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, user_2.clone(), Some(hierarchy_root_id), ACCOUNT_00);

	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let delegation_node = generate_base_delegation_node::<Test>(hierarchy_root_id, user_3, Some(parent_id), ACCOUNT_01);

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
		.build_and_execute_with_sanity_tests(|| {
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
	let hierarchy_details = generate_base_delegation_hierarchy_details::<Test>();

	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_1);
	let mut parent_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, user_2.clone(), Some(hierarchy_root_id), ACCOUNT_00);

	parent_node.details.revoked = true;
	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_2);
	let mut delegation_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, user_3, Some(parent_id), ACCOUNT_01);
	delegation_node.details.revoked = true;

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
		.build_and_execute_with_sanity_tests(|| {
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
			assert_eq!(
				Delegation::is_delegating(&user_1, &delegation_id, max_parent_checks),
				Ok((true, max_parent_checks - 2))
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
