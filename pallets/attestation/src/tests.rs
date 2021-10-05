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

use frame_support::{assert_noop, assert_ok};
use sp_core::Pair;
use sp_runtime::traits::Zero;

use crate::{
	self as attestation,
	mock::{runtime::Balances, *},
	AccountIdOf, Config, DelegatedAttestations,
};
use ctype::mock as ctype_mock;
use delegation::mock as delegation_mock;

// #############################################################################
// submit_attestation_creation_operation

#[test]
fn attest_no_delegation_successful() {
	let attester: AccountIdOf<Test> = get_alice_ed25519().public().into();
	let claim_hash = get_claim_hash(true);
	let attestation = generate_base_attestation::<Test>(attester.clone(), attester.clone());

	let operation = generate_base_attestation_creation_details(claim_hash, attestation);

	ExtBuilder::default()
		.with_ctypes(vec![(operation.ctype_hash, attester.clone())])
		.with_balances(vec![(attester.clone(), <Test as Config>::Deposit::get() * 100)])
		.build()
		.execute_with(|| {
			assert_ok!(Attestation::add(
				Origin::signed(attester.clone()),
				operation.claim_hash,
				operation.ctype_hash,
				operation.delegation_id
			));
			let stored_attestation =
				Attestation::attestations(&claim_hash).expect("Attestation should be present on chain.");

			assert_eq!(stored_attestation.ctype_hash, operation.ctype_hash);
			assert_eq!(stored_attestation.attester, attester);
			assert_eq!(stored_attestation.delegation_id, operation.delegation_id);
			assert!(!stored_attestation.revoked);
		});
}

#[test]
fn attest_with_delegation_successful() {
	let attester: AccountIdOf<Test> = get_alice_ed25519().public().into();
	let claim_hash = get_claim_hash(true);

	let hierarchy_root_id = delegation_mock::get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = delegation_mock::generate_base_delegation_hierarchy_details();
	let delegation_id = delegation_mock::get_delegation_id(true);
	let mut delegation_node =
		delegation_mock::generate_base_delegation_node(hierarchy_root_id, attester.clone(), Some(hierarchy_root_id));
	delegation_node.details.permissions = delegation::Permissions::ATTEST;
	let mut attestation = generate_base_attestation::<Test>(attester.clone(), attester.clone());
	attestation.delegation_id = Some(delegation_id);

	let operation = generate_base_attestation_creation_details(claim_hash, attestation);

	ExtBuilder::default()
		.with_ctypes(vec![(operation.ctype_hash, attester.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, attester.clone())])
		.with_delegations(vec![(delegation_id, delegation_node)])
		.with_balances(vec![(attester.clone(), <Test as Config>::Deposit::get() * 100)])
		.build()
		.execute_with(|| {
			assert_ok!(Attestation::add(
				Origin::signed(attester.clone()),
				operation.claim_hash,
				operation.ctype_hash,
				operation.delegation_id
			));
			let stored_attestation =
				Attestation::attestations(&claim_hash).expect("Attestation should be present on chain.");

			assert_eq!(stored_attestation.ctype_hash, operation.ctype_hash);
			assert_eq!(stored_attestation.attester, attester);
			assert_eq!(stored_attestation.delegation_id, operation.delegation_id);
			assert!(!stored_attestation.revoked);

			let delegated_attestations = Attestation::delegated_attestations(&delegation_id)
				.expect("Attested delegation should be present on chain.");

			assert_eq!(delegated_attestations, vec![claim_hash]);
		});
}

#[test]
fn ctype_not_present_attest_error() {
	let attester: AccountIdOf<Test> = get_alice_ed25519().public().into();
	let claim_hash = get_claim_hash(true);
	let attestation = generate_base_attestation::<Test>(attester.clone(), attester.clone());

	let operation = generate_base_attestation_creation_details(claim_hash, attestation);

	ExtBuilder::default()
		.with_balances(vec![(attester.clone(), <Test as Config>::Deposit::get() * 100)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Attestation::add(
					Origin::signed(attester),
					operation.claim_hash,
					operation.ctype_hash,
					operation.delegation_id
				),
				ctype::Error::<Test>::CTypeNotFound
			);
		});
}

#[test]
fn duplicate_attest_error() {
	let attester: AccountIdOf<Test> = get_alice_ed25519().public().into();
	let claim_hash = get_claim_hash(true);

	let attestation = generate_base_attestation::<Test>(attester.clone(), attester.clone());
	let operation = generate_base_attestation_creation_details(claim_hash, attestation.clone());

	ExtBuilder::default()
		.with_balances(vec![(attester.clone(), <Test as Config>::Deposit::get() * 100)])
		.with_ctypes(vec![(operation.ctype_hash, attester.clone())])
		.with_attestations(vec![(claim_hash, attestation)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Attestation::add(
					Origin::signed(attester.clone()),
					operation.claim_hash,
					operation.ctype_hash,
					operation.delegation_id
				),
				attestation::Error::<Test>::AlreadyAttested
			);
		});
}

#[test]
fn delegation_not_found_attest_error() {
	let attester: AccountIdOf<Test> = get_alice_ed25519().public().into();
	let claim_hash = get_claim_hash(true);
	let delegation_id = delegation_mock::get_delegation_id(true);
	let mut attestation = generate_base_attestation::<Test>(attester.clone(), attester.clone());
	attestation.delegation_id = Some(delegation_id);

	let operation = generate_base_attestation_creation_details(claim_hash, attestation);

	ExtBuilder::default()
		.with_balances(vec![(attester.clone(), <Test as Config>::Deposit::get() * 100)])
		.with_ctypes(vec![(operation.ctype_hash, attester.clone())])
		.build()
		.execute_with(|| {
			assert_noop!(
				Attestation::add(
					Origin::signed(attester.clone()),
					operation.claim_hash,
					operation.ctype_hash,
					operation.delegation_id
				),
				delegation::Error::<Test>::DelegationNotFound
			);
		});
}

#[test]
fn delegation_revoked_attest_error() {
	let attester: AccountIdOf<Test> = get_alice_ed25519().public().into();
	let claim_hash = get_claim_hash(true);
	let hierarchy_root_id = delegation_mock::get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = delegation_mock::generate_base_delegation_hierarchy_details::<Test>();
	// Delegation node does not have permissions to attest.
	let delegation_id = delegation_mock::get_delegation_id(true);
	let mut delegation_node =
		delegation_mock::generate_base_delegation_node(hierarchy_root_id, attester.clone(), Some(hierarchy_root_id));
	delegation_node.details.permissions = delegation::Permissions::ATTEST;
	delegation_node.details.revoked = true;
	let mut attestation = generate_base_attestation::<Test>(attester.clone(), attester.clone());
	attestation.delegation_id = Some(delegation_id);

	let operation = generate_base_attestation_creation_details(claim_hash, attestation);

	ExtBuilder::default()
		.with_balances(vec![(attester.clone(), <Test as Config>::Deposit::get() * 100)])
		.with_ctypes(vec![(operation.ctype_hash, attester.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, attester.clone())])
		.with_delegations(vec![(delegation_id, delegation_node)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Attestation::add(
					Origin::signed(attester.clone()),
					operation.claim_hash,
					operation.ctype_hash,
					operation.delegation_id
				),
				attestation::Error::<Test>::DelegationRevoked
			);
		});
}

#[test]
fn not_delegation_owner_attest_error() {
	let attester: AccountIdOf<Test> = get_alice_ed25519().public().into();
	let alternative_owner_keypair = get_bob_ed25519();
	let alternative_owner = get_ed25519_account(alternative_owner_keypair.public());
	let claim_hash = get_claim_hash(true);
	let hierarchy_root_id = delegation_mock::get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = delegation_mock::generate_base_delegation_hierarchy_details::<Test>();
	let delegation_id = delegation_mock::get_delegation_id(true);
	let mut delegation_node = delegation_mock::generate_base_delegation_node::<Test>(
		hierarchy_root_id,
		alternative_owner,
		Some(hierarchy_root_id),
	);
	delegation_node.details.permissions = delegation::Permissions::ATTEST;
	let mut attestation = generate_base_attestation::<Test>(attester.clone(), attester.clone());
	attestation.delegation_id = Some(delegation_id);

	let operation = generate_base_attestation_creation_details(claim_hash, attestation);

	ExtBuilder::default()
		.with_balances(vec![(attester.clone(), <Test as Config>::Deposit::get() * 100)])
		.with_ctypes(vec![(operation.ctype_hash, attester.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, attester.clone())])
		.with_delegations(vec![(delegation_id, delegation_node)])
		.with_balances(vec![(attester.clone(), <Test as Config>::Deposit::get() * 100)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Attestation::add(
					Origin::signed(attester.clone()),
					operation.claim_hash,
					operation.ctype_hash,
					operation.delegation_id
				),
				attestation::Error::<Test>::NotDelegatedToAttester
			);
		});
}

#[test]
fn unauthorised_permissions_attest_error() {
	let attester: AccountIdOf<Test> = get_alice_ed25519().public().into();
	let claim_hash = get_claim_hash(true);
	let hierarchy_root_id = delegation_mock::get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = delegation_mock::generate_base_delegation_hierarchy_details::<Test>();
	// Delegation node does not have permissions to attest.
	let delegation_id = delegation_mock::get_delegation_id(true);
	let delegation_node =
		delegation_mock::generate_base_delegation_node(hierarchy_root_id, attester.clone(), Some(hierarchy_root_id));
	let mut attestation = generate_base_attestation::<Test>(attester.clone(), attester.clone());
	attestation.delegation_id = Some(delegation_id);

	let operation = generate_base_attestation_creation_details(claim_hash, attestation);

	ExtBuilder::default()
		.with_balances(vec![(attester.clone(), <Test as Config>::Deposit::get() * 100)])
		.with_ctypes(vec![(operation.ctype_hash, attester.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, attester.clone())])
		.with_delegations(vec![(delegation_id, delegation_node)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Attestation::add(
					Origin::signed(attester.clone()),
					operation.claim_hash,
					operation.ctype_hash,
					operation.delegation_id
				),
				attestation::Error::<Test>::DelegationUnauthorizedToAttest
			);
		});
}

#[test]
fn root_not_present_attest_error() {
	let attester: AccountIdOf<Test> = get_alice_ed25519().public().into();
	let claim_hash = get_claim_hash(true);
	let hierarchy_root_id = delegation_mock::get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = delegation_mock::generate_base_delegation_hierarchy_details::<Test>();
	let alternative_hierarchy_root_id = delegation_mock::get_delegation_hierarchy_id::<Test>(false);
	let delegation_id = delegation_mock::get_delegation_id(true);
	let mut delegation_node = delegation_mock::generate_base_delegation_node(
		hierarchy_root_id,
		attester.clone(),
		Some(alternative_hierarchy_root_id),
	);
	delegation_node.details.permissions = delegation::Permissions::ATTEST;
	let mut attestation = generate_base_attestation::<Test>(attester.clone(), attester.clone());
	attestation.delegation_id = Some(delegation_id);

	let operation = generate_base_attestation_creation_details(claim_hash, attestation);

	ExtBuilder::default()
		.with_balances(vec![(attester.clone(), <Test as Config>::Deposit::get() * 100)])
		.with_ctypes(vec![(operation.ctype_hash, attester.clone())])
		.with_delegation_hierarchies(vec![(
			alternative_hierarchy_root_id,
			hierarchy_details,
			attester.clone(),
		)])
		.with_delegations(vec![(delegation_id, delegation_node)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Attestation::add(
					Origin::signed(attester.clone()),
					operation.claim_hash,
					operation.ctype_hash,
					operation.delegation_id
				),
				delegation::Error::<Test>::HierarchyNotFound
			);
		});
}

#[test]
fn root_ctype_mismatch_attest_error() {
	let attester: AccountIdOf<Test> = get_alice_ed25519().public().into();
	let claim_hash = get_claim_hash(true);
	let alternative_ctype_hash = ctype_mock::get_ctype_hash::<Test>(false);
	let hierarchy_root_id = delegation_mock::get_delegation_hierarchy_id::<Test>(true);
	let mut hierarchy_details = delegation_mock::generate_base_delegation_hierarchy_details::<Test>();
	hierarchy_details.ctype_hash = alternative_ctype_hash;
	let delegation_id = delegation_mock::get_delegation_id(true);
	let mut delegation_node =
		delegation_mock::generate_base_delegation_node(hierarchy_root_id, attester.clone(), Some(hierarchy_root_id));
	delegation_node.details.permissions = delegation::Permissions::ATTEST;
	let mut attestation = generate_base_attestation::<Test>(attester.clone(), attester.clone());
	attestation.delegation_id = Some(delegation_id);

	let operation = generate_base_attestation_creation_details(claim_hash, attestation);

	ExtBuilder::default()
		.with_balances(vec![(attester.clone(), <Test as Config>::Deposit::get() * 100)])
		.with_ctypes(vec![(operation.ctype_hash, attester.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, attester.clone())])
		.with_delegations(vec![(delegation_id, delegation_node)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Attestation::add(
					Origin::signed(attester.clone()),
					operation.claim_hash,
					operation.ctype_hash,
					operation.delegation_id
				),
				attestation::Error::<Test>::CTypeMismatch
			);
		});
}

// #############################################################################
// submit_attestation_revocation_operation

#[test]
fn revoke_and_remove_direct_successful() {
	let revoker: AccountIdOf<Test> = get_alice_ed25519().public().into();
	let claim_hash = get_claim_hash(true);
	let attestation = generate_base_attestation::<Test>(revoker.clone(), revoker.clone());

	let operation = generate_base_attestation_revocation_details::<Test>(claim_hash);

	ExtBuilder::default()
		.with_balances(vec![(revoker.clone(), <Test as Config>::Deposit::get() * 100)])
		.with_ctypes(vec![(attestation.ctype_hash, revoker.clone())])
		.with_attestations(vec![(operation.claim_hash, attestation)])
		.build()
		.execute_with(|| {
			assert_ok!(Attestation::revoke(
				Origin::signed(revoker.clone()),
				operation.claim_hash,
				operation.max_parent_checks
			));
			let stored_attestation =
				Attestation::attestations(claim_hash).expect("Attestation should be present on chain.");

			assert!(stored_attestation.revoked);
			assert_eq!(
				Balances::reserved_balance(revoker.clone()),
				<Test as Config>::Deposit::get()
			);

			assert_ok!(Attestation::remove(
				Origin::signed(revoker.clone()),
				operation.claim_hash,
				operation.max_parent_checks
			));
			assert!(Attestation::attestations(claim_hash).is_none());
			assert!(Balances::reserved_balance(revoker).is_zero());
		});
}

#[test]
fn revoke_with_delegation_successful() {
	let revoker: AccountIdOf<Test> = get_alice_ed25519().public().into();
	let attestation_owner: AccountIdOf<Test> = get_bob_ed25519().public().into();
	let claim_hash = get_claim_hash(true);

	let hierarchy_root_id = delegation_mock::get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = delegation_mock::generate_base_delegation_hierarchy_details::<Test>();
	let delegation_id = delegation_mock::get_delegation_id(true);
	let mut delegation_node =
		delegation_mock::generate_base_delegation_node(hierarchy_root_id, revoker.clone(), Some(hierarchy_root_id));
	delegation_node.details.permissions = delegation::Permissions::ATTEST;
	// Attestation owned by a different user, but delegation owned by the user
	// submitting the operation.
	let mut attestation = generate_base_attestation::<Test>(attestation_owner.clone(), attestation_owner.clone());
	attestation.delegation_id = Some(delegation_id);

	let mut operation = generate_base_attestation_revocation_details::<Test>(claim_hash);
	// Set to 0 as we only need to check the delegation node itself and no parent.
	operation.max_parent_checks = 0u32;

	ExtBuilder::default()
		.with_balances(vec![
			(revoker.clone(), <Test as Config>::Deposit::get() * 100),
			(attestation_owner, <Test as Config>::Deposit::get() * 100),
		])
		.with_ctypes(vec![(attestation.ctype_hash, revoker.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, revoker.clone())])
		.with_delegations(vec![(delegation_id, delegation_node)])
		.with_attestations(vec![(operation.claim_hash, attestation)])
		.build()
		.execute_with(|| {
			assert_ok!(Attestation::revoke(
				Origin::signed(revoker.clone()),
				operation.claim_hash,
				operation.max_parent_checks
			));
			let stored_attestation =
				Attestation::attestations(operation.claim_hash).expect("Attestation should be present on chain.");

			assert!(stored_attestation.revoked);
		});
}

#[test]
fn revoke_with_parent_delegation_successful() {
	let revoker: AccountIdOf<Test> = get_alice_ed25519().public().into();
	let attestation_owner: AccountIdOf<Test> = get_bob_ed25519().public().into();
	let claim_hash = get_claim_hash(true);

	let hierarchy_root_id = delegation_mock::get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = delegation_mock::generate_base_delegation_hierarchy_details::<Test>();
	let parent_id = delegation_mock::get_delegation_id(true);
	let mut parent_node =
		delegation_mock::generate_base_delegation_node(hierarchy_root_id, revoker.clone(), Some(hierarchy_root_id));
	parent_node.details.permissions = delegation::Permissions::ATTEST;
	let delegation_id = delegation_mock::get_delegation_id(false);
	let delegation_node =
		delegation_mock::generate_base_delegation_node(hierarchy_root_id, attestation_owner.clone(), Some(parent_id));
	let mut attestation = generate_base_attestation::<Test>(attestation_owner.clone(), attestation_owner.clone());
	attestation.delegation_id = Some(delegation_id);

	let mut operation = generate_base_attestation_revocation_details::<Test>(claim_hash);
	// Set to 1 as the delegation referenced in the attestation is the child of the
	// node we want to use
	operation.max_parent_checks = 1u32;

	ExtBuilder::default()
		.with_balances(vec![
			(revoker.clone(), <Test as Config>::Deposit::get() * 100),
			(attestation_owner, <Test as Config>::Deposit::get() * 100),
		])
		.with_ctypes(vec![(attestation.ctype_hash, revoker.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, revoker.clone())])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.with_attestations(vec![(operation.claim_hash, attestation)])
		.build()
		.execute_with(|| {
			assert_ok!(Attestation::revoke(
				Origin::signed(revoker.clone()),
				operation.claim_hash,
				operation.max_parent_checks
			));
			let stored_attestation =
				Attestation::attestations(claim_hash).expect("Attestation should be present on chain.");

			assert!(stored_attestation.revoked);
		});
}

#[test]
fn revoke_parent_delegation_no_attestation_permissions_successful() {
	let revoker: AccountIdOf<Test> = get_alice_ed25519().public().into();
	let attestation_owner: AccountIdOf<Test> = get_bob_ed25519().public().into();
	let claim_hash = get_claim_hash(true);

	let hierarchy_root_id = delegation_mock::get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = delegation_mock::generate_base_delegation_hierarchy_details::<Test>();
	let parent_id = delegation_mock::get_delegation_id(true);
	let mut parent_node =
		delegation_mock::generate_base_delegation_node(hierarchy_root_id, revoker.clone(), Some(hierarchy_root_id));
	parent_node.details.permissions = delegation::Permissions::DELEGATE;

	let delegation_id = delegation_mock::get_delegation_id(false);
	let delegation_node =
		delegation_mock::generate_base_delegation_node(hierarchy_root_id, attestation_owner.clone(), Some(parent_id));

	let mut attestation = generate_base_attestation::<Test>(attestation_owner.clone(), attestation_owner.clone());
	attestation.delegation_id = Some(delegation_id);

	let mut operation = generate_base_attestation_revocation_details::<Test>(claim_hash);
	// Set to 1 as the delegation referenced in the attestation is the child of the
	// node we want to use
	operation.max_parent_checks = 1u32;

	ExtBuilder::default()
		.with_balances(vec![
			(revoker.clone(), <Test as Config>::Deposit::get() * 100),
			(attestation_owner, <Test as Config>::Deposit::get() * 100),
		])
		.with_ctypes(vec![(attestation.ctype_hash, revoker.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, revoker.clone())])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.with_attestations(vec![(operation.claim_hash, attestation)])
		.build()
		.execute_with(|| {
			assert_ok!(Attestation::revoke(
				Origin::signed(revoker.clone()),
				operation.claim_hash,
				operation.max_parent_checks
			));
			let stored_attestation =
				Attestation::attestations(claim_hash).expect("Attestation should be present on chain.");

			assert!(stored_attestation.revoked);
		});
}

#[test]
fn revoke_parent_delegation_with_direct_delegation_revoked_successful() {
	let revoker: AccountIdOf<Test> = get_alice_ed25519().public().into();
	let attestation_owner: AccountIdOf<Test> = get_bob_ed25519().public().into();
	let claim_hash = get_claim_hash(true);

	let hierarchy_root_id = delegation_mock::get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = delegation_mock::generate_base_delegation_hierarchy_details::<Test>();
	let parent_id = delegation_mock::get_delegation_id(true);
	let mut parent_node =
		delegation_mock::generate_base_delegation_node(hierarchy_root_id, revoker.clone(), Some(hierarchy_root_id));
	parent_node.details.permissions = delegation::Permissions::ATTEST;

	let delegation_id = delegation_mock::get_delegation_id(false);
	let mut delegation_node =
		delegation_mock::generate_base_delegation_node(hierarchy_root_id, attestation_owner.clone(), Some(parent_id));

	delegation_node.details.revoked = true;
	let mut attestation = generate_base_attestation::<Test>(attestation_owner.clone(), attestation_owner.clone());
	attestation.delegation_id = Some(delegation_id);

	let mut operation = generate_base_attestation_revocation_details::<Test>(claim_hash);
	// Set to 1 as the delegation referenced in the attestation is the child of the
	// node we want to use
	operation.max_parent_checks = 1u32;

	ExtBuilder::default()
		.with_balances(vec![
			(revoker.clone(), <Test as Config>::Deposit::get() * 100),
			(attestation_owner, <Test as Config>::Deposit::get() * 100),
		])
		.with_ctypes(vec![(attestation.ctype_hash, revoker.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, revoker.clone())])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.with_attestations(vec![(operation.claim_hash, attestation)])
		.build()
		.execute_with(|| {
			assert_ok!(Attestation::revoke(
				Origin::signed(revoker.clone()),
				operation.claim_hash,
				operation.max_parent_checks
			));
			let stored_attestation =
				Attestation::attestations(claim_hash).expect("Attestation should be present on chain.");

			assert!(stored_attestation.revoked);
		});
}

#[test]
fn attestation_not_present_revoke_error() {
	let revoker: AccountIdOf<Test> = get_alice_ed25519().public().into();
	let claim_hash = get_claim_hash(true);

	let attestation = generate_base_attestation::<Test>(revoker.clone(), revoker.clone());

	let operation = generate_base_attestation_revocation_details::<Test>(claim_hash);

	ExtBuilder::default()
		.with_balances(vec![(revoker.clone(), <Test as Config>::Deposit::get() * 100)])
		.with_ctypes(vec![(attestation.ctype_hash, revoker.clone())])
		.build()
		.execute_with(|| {
			assert_noop!(
				Attestation::revoke(
					Origin::signed(revoker.clone()),
					operation.claim_hash,
					operation.max_parent_checks
				),
				attestation::Error::<Test>::AttestationNotFound
			);
		});
}

#[test]
fn already_revoked_revoke_error() {
	let revoker: AccountIdOf<Test> = get_alice_ed25519().public().into();
	let claim_hash = get_claim_hash(true);

	// Attestation already revoked
	let mut attestation = generate_base_attestation::<Test>(revoker.clone(), revoker.clone());
	attestation.revoked = true;

	let operation = generate_base_attestation_revocation_details::<Test>(claim_hash);

	ExtBuilder::default()
		.with_balances(vec![(revoker.clone(), <Test as Config>::Deposit::get() * 100)])
		.with_ctypes(vec![(attestation.ctype_hash, revoker.clone())])
		.with_attestations(vec![(operation.claim_hash, attestation)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Attestation::revoke(
					Origin::signed(revoker.clone()),
					operation.claim_hash,
					operation.max_parent_checks
				),
				attestation::Error::<Test>::AlreadyRevoked
			);
		});
}

#[test]
fn unauthorised_attestation_revoke_error() {
	let revoker: AccountIdOf<Test> = get_alice_ed25519().public().into();
	let attestation_owner: AccountIdOf<Test> = get_bob_ed25519().public().into();
	let claim_hash = get_claim_hash(true);

	// Attestation owned by a different user
	let attestation = generate_base_attestation::<Test>(attestation_owner.clone(), attestation_owner.clone());

	let operation = generate_base_attestation_revocation_details::<Test>(claim_hash);

	ExtBuilder::default()
		.with_balances(vec![
			(revoker.clone(), <Test as Config>::Deposit::get() * 100),
			(attestation_owner, <Test as Config>::Deposit::get() * 100),
		])
		.with_ctypes(vec![(attestation.ctype_hash, revoker.clone())])
		.with_attestations(vec![(operation.claim_hash, attestation)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Attestation::revoke(
					Origin::signed(revoker.clone()),
					operation.claim_hash,
					operation.max_parent_checks
				),
				attestation::Error::<Test>::Unauthorized
			);
		});
}

#[test]
fn max_parent_lookups_revoke_error() {
	let revoker: AccountIdOf<Test> = get_alice_ed25519().public().into();
	let attestation_owner: AccountIdOf<Test> = get_bob_ed25519().public().into();
	let claim_hash = get_claim_hash(true);

	let hierarchy_root_id = delegation_mock::get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = delegation_mock::generate_base_delegation_hierarchy_details::<Test>();
	let parent_id = delegation_mock::get_delegation_id(true);
	let parent_node =
		delegation_mock::generate_base_delegation_node(hierarchy_root_id, revoker.clone(), Some(hierarchy_root_id));
	let delegation_id = delegation_mock::get_delegation_id(false);
	let mut delegation_node =
		delegation_mock::generate_base_delegation_node(hierarchy_root_id, attestation_owner.clone(), Some(parent_id));

	delegation_node.details.permissions = delegation::Permissions::ATTEST;
	let mut attestation = generate_base_attestation::<Test>(attestation_owner.clone(), attestation_owner.clone());
	attestation.delegation_id = Some(delegation_id);

	let mut operation = generate_base_attestation_revocation_details::<Test>(claim_hash);
	operation.max_parent_checks = 0u32;

	ExtBuilder::default()
		.with_balances(vec![
			(revoker.clone(), <Test as Config>::Deposit::get() * 100),
			(attestation_owner, <Test as Config>::Deposit::get() * 100),
		])
		.with_ctypes(vec![(attestation.ctype_hash, revoker.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, revoker.clone())])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.with_attestations(vec![(operation.claim_hash, attestation)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Attestation::revoke(
					Origin::signed(revoker.clone()),
					operation.claim_hash,
					operation.max_parent_checks
				),
				delegation::Error::<Test>::MaxSearchDepthReached
			);
		});
}

#[test]
fn revoked_delegation_revoke_error() {
	let revoker: AccountIdOf<Test> = get_alice_ed25519().public().into();
	let attestation_owner: AccountIdOf<Test> = get_bob_ed25519().public().into();
	let claim_hash = get_claim_hash(true);

	let hierarchy_root_id = delegation_mock::get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = delegation_mock::generate_base_delegation_hierarchy_details::<Test>();
	let delegation_id = delegation_mock::get_delegation_id(true);
	let mut delegation_node =
		delegation_mock::generate_base_delegation_node(hierarchy_root_id, revoker.clone(), Some(hierarchy_root_id));
	delegation_node.details.permissions = delegation::Permissions::ATTEST;
	delegation_node.details.revoked = true;
	let mut attestation = generate_base_attestation::<Test>(attestation_owner.clone(), attestation_owner.clone());
	attestation.delegation_id = Some(delegation_id);

	let operation = generate_base_attestation_revocation_details::<Test>(claim_hash);

	ExtBuilder::default()
		.with_balances(vec![
			(revoker.clone(), <Test as Config>::Deposit::get() * 100),
			(attestation_owner, <Test as Config>::Deposit::get() * 100),
		])
		.with_ctypes(vec![(attestation.ctype_hash, revoker.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, revoker.clone())])
		.with_delegations(vec![(delegation_id, delegation_node)])
		.with_attestations(vec![(operation.claim_hash, attestation)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Attestation::revoke(
					Origin::signed(revoker.clone()),
					operation.claim_hash,
					operation.max_parent_checks
				),
				attestation::Error::<Test>::Unauthorized
			);
		});
}

// #############################################################################
// remove attestation

#[test]
fn subject_remove_direct_successful() {
	let revoker: AccountIdOf<Test> = get_alice_ed25519().public().into();
	let claim_hash = get_claim_hash(true);
	let attestation = generate_base_attestation::<Test>(revoker.clone(), revoker.clone());

	let operation = generate_base_attestation_revocation_details::<Test>(claim_hash);

	ExtBuilder::default()
		.with_balances(vec![(revoker.clone(), <Test as Config>::Deposit::get() * 100)])
		.with_ctypes(vec![(attestation.ctype_hash, revoker.clone())])
		.with_attestations(vec![(operation.claim_hash, attestation)])
		.build()
		.execute_with(|| {
			assert_ok!(Attestation::remove(
				Origin::signed(revoker.clone()),
				operation.claim_hash,
				operation.max_parent_checks
			));
			assert!(Attestation::attestations(claim_hash).is_none())
		});
}

#[test]
fn reclaim_deposit() {
	let deposit_owner: AccountIdOf<Test> = get_alice_ed25519().public().into();
	let attester: AccountIdOf<Test> = get_bob_ed25519().public().into();
	let claim_hash = get_claim_hash(true);
	let attestation = generate_base_attestation::<Test>(attester.clone(), deposit_owner.clone());

	let operation = generate_base_attestation_revocation_details::<Test>(claim_hash);

	ExtBuilder::default()
		.with_balances(vec![(deposit_owner.clone(), <Test as Config>::Deposit::get() * 100)])
		.with_ctypes(vec![(attestation.ctype_hash, deposit_owner.clone())])
		.with_attestations(vec![(operation.claim_hash, attestation)])
		.build()
		.execute_with(|| {
			assert_noop!(
				Attestation::reclaim_deposit(Origin::signed(attester), operation.claim_hash),
				attestation::Error::<Test>::Unauthorized,
			);
			assert_ok!(Attestation::reclaim_deposit(
				Origin::signed(deposit_owner.clone()),
				operation.claim_hash,
			));
			assert!(Attestation::attestations(claim_hash).is_none())
		});
}

#[test]
fn remove_with_delegation_successful() {
	let attester: AccountIdOf<Test> = get_alice_ed25519().public().into();
	let attestation_owner: AccountIdOf<Test> = get_bob_ed25519().public().into();
	let claim_hash = get_claim_hash(true);

	let hierarchy_root_id = delegation_mock::get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = delegation_mock::generate_base_delegation_hierarchy_details::<Test>();
	let delegation_id = delegation_mock::get_delegation_id(true);
	let mut delegation_node =
		delegation_mock::generate_base_delegation_node(hierarchy_root_id, attester.clone(), Some(hierarchy_root_id));
	delegation_node.details.permissions = delegation::Permissions::ATTEST;
	// Attestation owned by a different user, but delegation owned by the user
	// submitting the operation.
	let mut attestation = generate_base_attestation::<Test>(attestation_owner.clone(), attestation_owner.clone());
	attestation.delegation_id = Some(delegation_id);

	let mut operation = generate_base_attestation_revocation_details::<Test>(claim_hash);
	// Set to 0 as we only need to check the delegation node itself and no parent.
	operation.max_parent_checks = 0u32;

	ExtBuilder::default()
		.with_balances(vec![
			(attester.clone(), <Test as Config>::Deposit::get() * 100),
			(attestation_owner.clone(), <Test as Config>::Deposit::get() * 100),
		])
		.with_ctypes(vec![(attestation.ctype_hash, attester.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, attester.clone())])
		.with_delegations(vec![(delegation_id, delegation_node)])
		.with_attestations(vec![(operation.claim_hash, attestation)])
		.build()
		.execute_with(|| {
			assert_eq!(
				Balances::reserved_balance(attestation_owner.clone()),
				<Test as Config>::Deposit::get()
			);
			assert_ok!(Attestation::remove(
				Origin::signed(attester.clone()),
				operation.claim_hash,
				operation.max_parent_checks
			));
			assert!(Attestation::attestations(operation.claim_hash).is_none());
			assert!(Balances::reserved_balance(attestation_owner).is_zero());
		});
}

#[test]
fn attestation_not_present_remove_error() {
	let attester: AccountIdOf<Test> = get_alice_ed25519().public().into();
	let claim_hash = get_claim_hash(true);

	let attestation = generate_base_attestation::<Test>(attester.clone(), attester.clone());

	let operation = generate_base_attestation_revocation_details::<Test>(claim_hash);

	ExtBuilder::default()
		.with_balances(vec![(attester.clone(), <Test as Config>::Deposit::get() * 100)])
		.with_ctypes(vec![(attestation.ctype_hash, attester.clone())])
		.build()
		.execute_with(|| {
			assert_eq!(Balances::reserved_balance(attester.clone()), 0,);

			assert_noop!(
				Attestation::remove(
					Origin::signed(attester.clone()),
					operation.claim_hash,
					operation.max_parent_checks
				),
				attestation::Error::<Test>::AttestationNotFound
			);
			assert!(Balances::reserved_balance(attester.clone()).is_zero());
		});
}

#[test]
fn unauthorised_attestation_remove_error() {
	let remover: AccountIdOf<Test> = get_alice_ed25519().public().into();
	let attestation_owner: AccountIdOf<Test> = get_bob_ed25519().public().into();
	let claim_hash = get_claim_hash(true);

	// Attestation owned by a different user
	let attestation = generate_base_attestation::<Test>(attestation_owner.clone(), attestation_owner.clone());

	let operation = generate_base_attestation_revocation_details::<Test>(claim_hash);

	ExtBuilder::default()
		.with_balances(vec![
			(remover.clone(), <Test as Config>::Deposit::get() * 100),
			(attestation_owner.clone(), <Test as Config>::Deposit::get() * 100),
		])
		.with_ctypes(vec![(attestation.ctype_hash, remover.clone())])
		.with_attestations(vec![(operation.claim_hash, attestation)])
		.build()
		.execute_with(|| {
			assert_eq!(
				Balances::reserved_balance(attestation_owner.clone()),
				<Test as Config>::Deposit::get()
			);
			assert_noop!(
				Attestation::remove(
					Origin::signed(remover.clone()),
					operation.claim_hash,
					operation.max_parent_checks
				),
				attestation::Error::<Test>::Unauthorized
			);
			assert_eq!(
				Balances::reserved_balance(attestation_owner),
				<Test as Config>::Deposit::get()
			);
		});
}

#[test]
fn revoked_delegation_remove_error() {
	let attester: AccountIdOf<Test> = get_alice_ed25519().public().into();
	let attestation_owner: AccountIdOf<Test> = get_bob_ed25519().public().into();
	let claim_hash = get_claim_hash(true);

	let hierarchy_root_id = delegation_mock::get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = delegation_mock::generate_base_delegation_hierarchy_details();
	let (delegation_id, mut delegation_node) = (
		delegation_mock::get_delegation_id(true),
		delegation_mock::generate_base_delegation_node(hierarchy_root_id, attester.clone(), Some(hierarchy_root_id)),
	);
	delegation_node.details.permissions = delegation::Permissions::ATTEST;
	delegation_node.details.revoked = true;
	let mut attestation = generate_base_attestation::<Test>(attestation_owner.clone(), attestation_owner.clone());
	attestation.delegation_id = Some(delegation_id);

	let operation = generate_base_attestation_revocation_details::<Test>(claim_hash);

	ExtBuilder::default()
		.with_balances(vec![
			(attester.clone(), <Test as Config>::Deposit::get() * 100),
			(attestation_owner.clone(), <Test as Config>::Deposit::get() * 100),
		])
		.with_ctypes(vec![(attestation.ctype_hash, attester.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, attester.clone())])
		.with_delegations(vec![(delegation_id, delegation_node)])
		.with_attestations(vec![(operation.claim_hash, attestation)])
		.build()
		.execute_with(|| {
			assert_eq!(
				Balances::reserved_balance(attestation_owner.clone()),
				<Test as Config>::Deposit::get()
			);
			assert_noop!(
				Attestation::remove(
					Origin::signed(attester.clone()),
					operation.claim_hash,
					operation.max_parent_checks
				),
				attestation::Error::<Test>::Unauthorized
			);
			assert_eq!(
				Balances::reserved_balance(attestation_owner),
				<Test as Config>::Deposit::get()
			);
		});
}

#[test]
fn remove_delegated_attestation() {
	let attester: AccountIdOf<Test> = get_alice_ed25519().public().into();
	let attestation_owner: AccountIdOf<Test> = get_bob_ed25519().public().into();
	let claim_hash = get_claim_hash(true);

	let hierarchy_root_id = delegation_mock::get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = delegation_mock::generate_base_delegation_hierarchy_details();
	let (delegation_id, mut delegation_node) = (
		delegation_mock::get_delegation_id(true),
		delegation_mock::generate_base_delegation_node(hierarchy_root_id, attester.clone(), Some(hierarchy_root_id)),
	);
	delegation_node.details.permissions = delegation::Permissions::ATTEST;
	let mut attestation = generate_base_attestation::<Test>(attestation_owner.clone(), attestation_owner.clone());
	attestation.delegation_id = Some(delegation_id);

	let operation = generate_base_attestation_revocation_details::<Test>(claim_hash);

	ExtBuilder::default()
		.with_balances(vec![
			(attester.clone(), <Test as Config>::Deposit::get() * 100),
			(attestation_owner.clone(), <Test as Config>::Deposit::get() * 100),
		])
		.with_ctypes(vec![(attestation.ctype_hash, attester.clone())])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, attester.clone())])
		.with_delegations(vec![(delegation_id, delegation_node)])
		.with_attestations(vec![(operation.claim_hash, attestation)])
		.build()
		.execute_with(|| {
			assert_eq!(
				Balances::reserved_balance(attestation_owner.clone()),
				<Test as Config>::Deposit::get()
			);
			assert!(
				DelegatedAttestations::<Test>::get(delegation_id)
					.unwrap_or_default()
					.iter()
					.any(|&ch| ch == operation.claim_hash),
				"delegated attestation entry should be present before removal"
			);

			assert_ok!(Attestation::remove(
				Origin::signed(attester.clone()),
				operation.claim_hash,
				operation.max_parent_checks
			));
			assert!(Balances::reserved_balance(attestation_owner).is_zero());
			assert!(
				!DelegatedAttestations::<Test>::get(delegation_id)
					.unwrap_or_default()
					.iter()
					.any(|&ch| ch == operation.claim_hash),
				"delegated attestation entry should be removed"
			);
		});
}
