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

use crate::{self as attestation, mock::*};
use ctype::mock as ctype_mock;
use delegation::mock as delegation_mock;

// submit_attestation_creation_operation

#[test]
fn attest_no_delegation_successful() {
	let attester_keypair = get_alice_ed25519();
	let attester = get_ed25519_account(attester_keypair.public());
	let claim_hash = get_claim_hash(true);
	let attestation = generate_base_attestation(attester.clone());

	let operation = generate_base_attestation_creation_details(claim_hash, attestation);

	let mut ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(operation.ctype_hash, attester.clone())])
		.build(None);

	ext.execute_with(|| {
		assert_ok!(Attestation::add(
			get_origin(attester.clone()),
			operation.claim_hash,
			operation.ctype_hash,
			operation.delegation_id
		));
	});

	let stored_attestation =
		ext.execute_with(|| Attestation::attestations(&claim_hash).expect("Attestation should be present on chain."));

	assert_eq!(stored_attestation.ctype_hash, operation.ctype_hash);
	assert_eq!(stored_attestation.attester, attester);
	assert_eq!(stored_attestation.delegation_id, operation.delegation_id);
	assert!(!stored_attestation.revoked);
}

#[test]
fn attest_with_delegation_successful() {
	let attester_keypair = get_alice_ed25519();
	let attester = get_ed25519_account(attester_keypair.public());
	let claim_hash = get_claim_hash(true);
	let (root_id, root_node) = (
		delegation_mock::get_delegation_root_id(true),
		delegation_mock::generate_base_delegation_root(attester.clone()),
	);
	let (delegation_id, mut delegation_node) = (
		delegation_mock::get_delegation_id(true),
		delegation_mock::generate_base_delegation_node(root_id, attester.clone()),
	);
	delegation_node.permissions = delegation::Permissions::ATTEST;
	let mut attestation = generate_base_attestation(attester.clone());
	attestation.delegation_id = Some(delegation_id);

	let operation = generate_base_attestation_creation_details(claim_hash, attestation);

	let ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(operation.ctype_hash, attester.clone())])
		.build(None);
	let mut ext = delegation_mock::ExtBuilder::default()
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![(delegation_id, delegation_node)])
		.with_children(vec![(root_id, vec![delegation_id])])
		.build(Some(ext));

	ext.execute_with(|| {
		assert_ok!(Attestation::add(
			get_origin(attester.clone()),
			operation.claim_hash,
			operation.ctype_hash,
			operation.delegation_id
		));
	});

	let stored_attestation =
		ext.execute_with(|| Attestation::attestations(&claim_hash).expect("Attestation should be present on chain."));

	assert_eq!(stored_attestation.ctype_hash, operation.ctype_hash);
	assert_eq!(stored_attestation.attester, attester);
	assert_eq!(stored_attestation.delegation_id, operation.delegation_id);
	assert!(!stored_attestation.revoked);

	let delegated_attestations = ext.execute_with(|| {
		Attestation::delegated_attestations(&delegation_id).expect("Attested delegation should be present on chain.")
	});

	assert_eq!(delegated_attestations, vec![claim_hash]);
}

#[test]
fn ctype_not_present_attest_error() {
	let attester_keypair = get_alice_ed25519();
	let attester = get_ed25519_account(attester_keypair.public());
	let claim_hash = get_claim_hash(true);
	let attestation = generate_base_attestation(attester.clone());

	let operation = generate_base_attestation_creation_details(claim_hash, attestation);

	// No CTYPE stored
	let mut ext = ExtBuilder::default().build(None);

	ext.execute_with(|| {
		assert_noop!(
			Attestation::add(
				get_origin(attester.clone()),
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
	let attester_keypair = get_alice_ed25519();
	let attester = get_ed25519_account(attester_keypair.public());
	let claim_hash = get_claim_hash(true);
	let attestation = generate_base_attestation(attester.clone());

	let operation = generate_base_attestation_creation_details(claim_hash, attestation.clone());

	let ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(operation.ctype_hash, attester.clone())])
		.build(None);
	let ext = delegation_mock::ExtBuilder::default().build(Some(ext));
	let mut ext = ExtBuilder::default()
		.with_attestations(vec![(claim_hash, attestation)])
		.build(Some(ext));

	ext.execute_with(|| {
		assert_noop!(
			Attestation::add(
				get_origin(attester.clone()),
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
	let attester_keypair = get_alice_ed25519();
	let attester = get_ed25519_account(attester_keypair.public());
	let claim_hash = get_claim_hash(true);
	let delegation_id = delegation_mock::get_delegation_id(true);
	let mut attestation = generate_base_attestation(attester.clone());
	attestation.delegation_id = Some(delegation_id);

	let operation = generate_base_attestation_creation_details(claim_hash, attestation);

	let mut ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(operation.ctype_hash, attester.clone())])
		.build(None);

	ext.execute_with(|| {
		assert_noop!(
			Attestation::add(
				get_origin(attester.clone()),
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
	let attester_keypair = get_alice_ed25519();
	let attester = get_ed25519_account(attester_keypair.public());
	let claim_hash = get_claim_hash(true);
	let (root_id, root_node) = (
		delegation_mock::get_delegation_root_id(true),
		delegation_mock::generate_base_delegation_root(attester.clone()),
	);
	let (delegation_id, mut delegation_node) = (
		delegation_mock::get_delegation_id(true),
		delegation_mock::generate_base_delegation_node(root_id, attester.clone()),
	);
	delegation_node.permissions = delegation::Permissions::ATTEST;
	delegation_node.revoked = true;
	let mut attestation = generate_base_attestation(attester.clone());
	attestation.delegation_id = Some(delegation_id);

	let operation = generate_base_attestation_creation_details(claim_hash, attestation);

	let ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(operation.ctype_hash, attester.clone())])
		.build(None);
	let mut ext = delegation_mock::ExtBuilder::default()
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![(delegation_id, delegation_node)])
		.with_children(vec![(root_id, vec![delegation_id])])
		.build(Some(ext));

	ext.execute_with(|| {
		assert_noop!(
			Attestation::add(
				get_origin(attester.clone()),
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
	let attester_keypair = get_alice_ed25519();
	let attester = get_ed25519_account(attester_keypair.public());
	let alternative_owner_keypair = get_bob_ed25519();
	let alternative_owner = get_ed25519_account(alternative_owner_keypair.public());
	let claim_hash = get_claim_hash(true);
	let (root_id, root_node) = (
		delegation_mock::get_delegation_root_id(true),
		delegation_mock::generate_base_delegation_root(alternative_owner.clone()),
	);
	let (delegation_id, mut delegation_node) = (
		delegation_mock::get_delegation_id(true),
		delegation_mock::generate_base_delegation_node(root_id, alternative_owner),
	);
	delegation_node.permissions = delegation::Permissions::ATTEST;
	let mut attestation = generate_base_attestation(attester.clone());
	attestation.delegation_id = Some(delegation_id);

	let operation = generate_base_attestation_creation_details(claim_hash, attestation);

	let ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(operation.ctype_hash, attester.clone())])
		.build(None);
	let mut ext = delegation_mock::ExtBuilder::default()
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![(delegation_id, delegation_node)])
		.with_children(vec![(root_id, vec![delegation_id])])
		.build(Some(ext));

	ext.execute_with(|| {
		assert_noop!(
			Attestation::add(
				get_origin(attester.clone()),
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
	let attester_keypair = get_alice_ed25519();
	let attester = get_ed25519_account(attester_keypair.public());
	let claim_hash = get_claim_hash(true);
	let (root_id, root_node) = (
		delegation_mock::get_delegation_root_id(true),
		delegation_mock::generate_base_delegation_root(attester.clone()),
	);
	let (delegation_id, delegation_node) = (
		delegation_mock::get_delegation_id(true),
		delegation_mock::generate_base_delegation_node(root_id, attester.clone()),
	);
	let mut attestation = generate_base_attestation(attester.clone());
	attestation.delegation_id = Some(delegation_id);

	let operation = generate_base_attestation_creation_details(claim_hash, attestation);

	let ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(operation.ctype_hash, attester.clone())])
		.build(None);
	let mut ext = delegation_mock::ExtBuilder::default()
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![(delegation_id, delegation_node)])
		.with_children(vec![(root_id, vec![delegation_id])])
		.build(Some(ext));

	ext.execute_with(|| {
		assert_noop!(
			Attestation::add(
				get_origin(attester.clone()),
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
	let attester_keypair = get_alice_ed25519();
	let attester = get_ed25519_account(attester_keypair.public());
	let claim_hash = get_claim_hash(true);
	let (root_id, root_node) = (
		delegation_mock::get_delegation_root_id(true),
		delegation_mock::generate_base_delegation_root(attester.clone()),
	);
	let alternative_root_id = delegation_mock::get_delegation_root_id(false);
	let (delegation_id, mut delegation_node) = (
		delegation_mock::get_delegation_id(true),
		delegation_mock::generate_base_delegation_node(root_id, attester.clone()),
	);
	delegation_node.permissions = delegation::Permissions::ATTEST;
	let mut attestation = generate_base_attestation(attester.clone());
	attestation.delegation_id = Some(delegation_id);

	let operation = generate_base_attestation_creation_details(claim_hash, attestation);

	let ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(operation.ctype_hash, attester.clone())])
		.build(None);
	let mut ext = delegation_mock::ExtBuilder::default()
		.with_root_delegations(vec![(alternative_root_id, root_node)])
		.with_delegations(vec![(delegation_id, delegation_node)])
		.with_children(vec![(alternative_root_id, vec![delegation_id])])
		.build(Some(ext));

	ext.execute_with(|| {
		assert_noop!(
			Attestation::add(
				get_origin(attester.clone()),
				operation.claim_hash,
				operation.ctype_hash,
				operation.delegation_id
			),
			delegation::Error::<Test>::RootNotFound
		);
	});
}

#[test]
fn root_ctype_mismatch_attest_error() {
	let attester_keypair = get_alice_ed25519();
	let attester = get_ed25519_account(attester_keypair.public());
	let claim_hash = get_claim_hash(true);
	let alternative_ctype_hash = ctype_mock::get_ctype_hash(false);
	let (root_id, mut root_node) = (
		delegation_mock::get_delegation_root_id(true),
		delegation_mock::generate_base_delegation_root(attester.clone()),
	);
	root_node.ctype_hash = alternative_ctype_hash;
	let (delegation_id, mut delegation_node) = (
		delegation_mock::get_delegation_id(true),
		delegation_mock::generate_base_delegation_node(root_id, attester.clone()),
	);
	delegation_node.permissions = delegation::Permissions::ATTEST;
	let mut attestation = generate_base_attestation(attester.clone());
	attestation.delegation_id = Some(delegation_id);

	let operation = generate_base_attestation_creation_details(claim_hash, attestation);

	let ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(operation.ctype_hash, attester.clone())])
		.build(None);
	let mut ext = delegation_mock::ExtBuilder::default()
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![(delegation_id, delegation_node)])
		.with_children(vec![(root_id, vec![delegation_id])])
		.build(Some(ext));

	ext.execute_with(|| {
		assert_noop!(
			Attestation::add(
				get_origin(attester.clone()),
				operation.claim_hash,
				operation.ctype_hash,
				operation.delegation_id
			),
			attestation::Error::<Test>::CTypeMismatch
		);
	});
}

// submit_attestation_revocation_operation

#[test]
fn revoke_direct_successful() {
	let revoker_keypair = get_alice_ed25519();
	let revoker = get_ed25519_account(revoker_keypair.public());
	let claim_hash = get_claim_hash(true);
	let attestation = generate_base_attestation(revoker.clone());

	let operation = generate_base_attestation_revocation_details(claim_hash);

	let ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(attestation.ctype_hash, revoker.clone())])
		.build(None);
	let ext = delegation_mock::ExtBuilder::default().build(Some(ext));
	let mut ext = ExtBuilder::default()
		.with_attestations(vec![(operation.claim_hash, attestation)])
		.build(Some(ext));

	ext.execute_with(|| {
		assert_ok!(Attestation::revoke(
			get_origin(revoker.clone()),
			operation.claim_hash,
			operation.max_parent_checks
		));
	});

	let stored_attestation =
		ext.execute_with(|| Attestation::attestations(claim_hash).expect("Attestation should be present on chain."));

	assert!(stored_attestation.revoked);
}

#[test]
fn revoke_with_delegation_successful() {
	let revoker_keypair = get_alice_ed25519();
	let revoker = get_ed25519_account(revoker_keypair.public());
	let attestation_owner_keypair = get_bob_ed25519();
	let attestation_owner = get_ed25519_account(attestation_owner_keypair.public());
	let claim_hash = get_claim_hash(true);

	let (root_id, root_node) = (
		delegation_mock::get_delegation_root_id(true),
		delegation_mock::generate_base_delegation_root(revoker.clone()),
	);
	let (delegation_id, mut delegation_node) = (
		delegation_mock::get_delegation_id(true),
		delegation_mock::generate_base_delegation_node(root_id, revoker.clone()),
	);
	delegation_node.permissions = delegation::Permissions::ATTEST;
	// Attestation owned by a different user, but delegation owned by the user
	// submitting the operation.
	let mut attestation = generate_base_attestation(attestation_owner);
	attestation.delegation_id = Some(delegation_id);

	let mut operation = generate_base_attestation_revocation_details(claim_hash);
	// Set to 0 as we only need to check the delegation node itself and no parent.
	operation.max_parent_checks = 0u32;

	let ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(attestation.ctype_hash, revoker.clone())])
		.build(None);
	let ext = delegation_mock::ExtBuilder::default()
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![(delegation_id, delegation_node)])
		.with_children(vec![(root_id, vec![delegation_id])])
		.build(Some(ext));
	let mut ext = ExtBuilder::default()
		.with_attestations(vec![(operation.claim_hash, attestation)])
		.build(Some(ext));

	ext.execute_with(|| {
		assert_ok!(Attestation::revoke(
			get_origin(revoker.clone()),
			operation.claim_hash,
			operation.max_parent_checks
		));
	});

	let stored_attestation = ext.execute_with(|| {
		Attestation::attestations(operation.claim_hash).expect("Attestation should be present on chain.")
	});

	assert!(stored_attestation.revoked);
}

#[test]
fn revoke_with_parent_delegation_successful() {
	let revoker_keypair = get_alice_ed25519();
	let revoker = get_ed25519_account(revoker_keypair.public());
	let attestation_owner_keypair = get_bob_ed25519();
	let attestation_owner = get_ed25519_account(attestation_owner_keypair.public());
	let claim_hash = get_claim_hash(true);

	let (root_id, root_node) = (
		delegation_mock::get_delegation_root_id(true),
		delegation_mock::generate_base_delegation_root(revoker.clone()),
	);
	let (parent_id, mut parent_node) = (
		delegation_mock::get_delegation_id(true),
		delegation_mock::generate_base_delegation_node(root_id, revoker.clone()),
	);
	parent_node.permissions = delegation::Permissions::ATTEST;
	let (delegation_id, delegation_node) = (
		delegation_mock::get_delegation_id(false),
		delegation_mock::generate_base_delegation_node(root_id, attestation_owner.clone()),
	);
	let mut attestation = generate_base_attestation(attestation_owner);
	attestation.delegation_id = Some(delegation_id);

	let mut operation = generate_base_attestation_revocation_details(claim_hash);
	// Set to 1 as the delegation referenced in the attestation is the child of the
	// node we want to use
	operation.max_parent_checks = 1u32;

	let ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(attestation.ctype_hash, revoker.clone())])
		.build(None);
	let ext = delegation_mock::ExtBuilder::default()
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.with_children(vec![(root_id, vec![parent_id]), (parent_id, vec![delegation_id])])
		.build(Some(ext));
	let mut ext = ExtBuilder::default()
		.with_attestations(vec![(operation.claim_hash, attestation)])
		.build(Some(ext));

	ext.execute_with(|| {
		assert_ok!(Attestation::revoke(
			get_origin(revoker.clone()),
			operation.claim_hash,
			operation.max_parent_checks
		));
	});

	let stored_attestation =
		ext.execute_with(|| Attestation::attestations(claim_hash).expect("Attestation should be present on chain."));

	assert!(stored_attestation.revoked);
}

#[test]
fn revoke_parent_delegation_no_attestation_permissions_successful() {
	let revoker_keypair = get_alice_ed25519();
	let revoker = get_ed25519_account(revoker_keypair.public());
	let attestation_owner_keypair = get_bob_ed25519();
	let attestation_owner = get_ed25519_account(attestation_owner_keypair.public());
	let claim_hash = get_claim_hash(true);

	let (root_id, root_node) = (
		delegation_mock::get_delegation_root_id(true),
		delegation_mock::generate_base_delegation_root(revoker.clone()),
	);
	let (parent_id, mut parent_node) = (
		delegation_mock::get_delegation_id(true),
		delegation_mock::generate_base_delegation_node(root_id, revoker.clone()),
	);
	parent_node.permissions = delegation::Permissions::DELEGATE;
	let (delegation_id, delegation_node) = (
		delegation_mock::get_delegation_id(false),
		delegation_mock::generate_base_delegation_node(root_id, attestation_owner.clone()),
	);
	let mut attestation = generate_base_attestation(attestation_owner);
	attestation.delegation_id = Some(delegation_id);

	let mut operation = generate_base_attestation_revocation_details(claim_hash);
	// Set to 1 as the delegation referenced in the attestation is the child of the
	// node we want to use
	operation.max_parent_checks = 1u32;

	let ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(attestation.ctype_hash, revoker.clone())])
		.build(None);
	let ext = delegation_mock::ExtBuilder::default()
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.with_children(vec![(root_id, vec![parent_id]), (parent_id, vec![delegation_id])])
		.build(Some(ext));
	let mut ext = ExtBuilder::default()
		.with_attestations(vec![(operation.claim_hash, attestation)])
		.build(Some(ext));

	ext.execute_with(|| {
		assert_ok!(Attestation::revoke(
			get_origin(revoker.clone()),
			operation.claim_hash,
			operation.max_parent_checks
		));
	});

	let stored_attestation =
		ext.execute_with(|| Attestation::attestations(claim_hash).expect("Attestation should be present on chain."));

	assert!(stored_attestation.revoked);
}

#[test]
fn revoke_parent_delegation_with_direct_delegation_revoked_successful() {
	let revoker_keypair = get_alice_ed25519();
	let revoker = get_ed25519_account(revoker_keypair.public());
	let attestation_owner_keypair = get_bob_ed25519();
	let attestation_owner = get_ed25519_account(attestation_owner_keypair.public());
	let claim_hash = get_claim_hash(true);

	let (root_id, root_node) = (
		delegation_mock::get_delegation_root_id(true),
		delegation_mock::generate_base_delegation_root(revoker.clone()),
	);
	let (parent_id, mut parent_node) = (
		delegation_mock::get_delegation_id(true),
		delegation_mock::generate_base_delegation_node(root_id, revoker.clone()),
	);
	parent_node.permissions = delegation::Permissions::ATTEST;
	let (delegation_id, mut delegation_node) = (
		delegation_mock::get_delegation_id(false),
		delegation_mock::generate_base_delegation_node(root_id, attestation_owner.clone()),
	);
	delegation_node.revoked = true;
	let mut attestation = generate_base_attestation(attestation_owner);
	attestation.delegation_id = Some(delegation_id);

	let mut operation = generate_base_attestation_revocation_details(claim_hash);
	// Set to 1 as the delegation referenced in the attestation is the child of the
	// node we want to use
	operation.max_parent_checks = 1u32;

	let ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(attestation.ctype_hash, revoker.clone())])
		.build(None);
	let ext = delegation_mock::ExtBuilder::default()
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![(parent_id, parent_node), (delegation_id, delegation_node)])
		.with_children(vec![(root_id, vec![parent_id]), (parent_id, vec![delegation_id])])
		.build(Some(ext));
	let mut ext = ExtBuilder::default()
		.with_attestations(vec![(operation.claim_hash, attestation)])
		.build(Some(ext));

	ext.execute_with(|| {
		assert_ok!(Attestation::revoke(
			get_origin(revoker.clone()),
			operation.claim_hash,
			operation.max_parent_checks
		));
	});

	let stored_attestation =
		ext.execute_with(|| Attestation::attestations(claim_hash).expect("Attestation should be present on chain."));

	assert!(stored_attestation.revoked);
}

#[test]
fn attestation_not_present_revoke_error() {
	let revoker_keypair = get_alice_ed25519();
	let revoker = get_ed25519_account(revoker_keypair.public());
	let claim_hash = get_claim_hash(true);

	let attestation = generate_base_attestation(revoker.clone());

	let operation = generate_base_attestation_revocation_details(claim_hash);

	let mut ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(attestation.ctype_hash, revoker.clone())])
		.build(None);

	ext.execute_with(|| {
		assert_noop!(
			Attestation::revoke(
				get_origin(revoker.clone()),
				operation.claim_hash,
				operation.max_parent_checks
			),
			attestation::Error::<Test>::AttestationNotFound
		);
	});
}

#[test]
fn already_revoked_revoke_error() {
	let revoker_keypair = get_alice_ed25519();
	let revoker = get_ed25519_account(revoker_keypair.public());
	let claim_hash = get_claim_hash(true);

	// Attestation already revoked
	let mut attestation = generate_base_attestation(revoker.clone());
	attestation.revoked = true;

	let operation = generate_base_attestation_revocation_details(claim_hash);

	let ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(attestation.ctype_hash, revoker.clone())])
		.build(None);
	let ext = delegation_mock::ExtBuilder::default().build(Some(ext));
	let mut ext = ExtBuilder::default()
		.with_attestations(vec![(operation.claim_hash, attestation)])
		.build(Some(ext));

	ext.execute_with(|| {
		assert_noop!(
			Attestation::revoke(
				get_origin(revoker.clone()),
				operation.claim_hash,
				operation.max_parent_checks
			),
			attestation::Error::<Test>::AlreadyRevoked
		);
	});
}

#[test]
fn unauthorised_attestation_revoke_error() {
	let revoker_keypair = get_alice_ed25519();
	let revoker = get_ed25519_account(revoker_keypair.public());
	let attestation_owner_keypair = get_bob_ed25519();
	let attestation_owner = get_ed25519_account(attestation_owner_keypair.public());
	let claim_hash = get_claim_hash(true);

	// Attestation owned by a different user
	let attestation = generate_base_attestation(attestation_owner);

	let operation = generate_base_attestation_revocation_details(claim_hash);

	let ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(attestation.ctype_hash, revoker.clone())])
		.build(None);
	let ext = delegation_mock::ExtBuilder::default().build(Some(ext));
	let mut ext = ExtBuilder::default()
		.with_attestations(vec![(operation.claim_hash, attestation)])
		.build(Some(ext));

	ext.execute_with(|| {
		assert_noop!(
			Attestation::revoke(
				get_origin(revoker.clone()),
				operation.claim_hash,
				operation.max_parent_checks
			),
			attestation::Error::<Test>::UnauthorizedRevocation
		);
	});
}

#[test]
fn max_parent_lookups_revoke_error() {
	let revoker_keypair = get_alice_ed25519();
	let revoker = get_ed25519_account(revoker_keypair.public());
	let attestation_owner_keypair = get_bob_ed25519();
	let attestation_owner = get_ed25519_account(attestation_owner_keypair.public());
	let claim_hash = get_claim_hash(true);

	let (root_id, root_node) = (
		delegation_mock::get_delegation_root_id(true),
		delegation_mock::generate_base_delegation_root(revoker.clone()),
	);
	let (parent_delegation_id, parent_delegation_node) = (
		delegation_mock::get_delegation_id(true),
		delegation_mock::generate_base_delegation_node(root_id, revoker.clone()),
	);
	let (delegation_id, mut delegation_node) = (
		delegation_mock::get_delegation_id(true),
		delegation_mock::generate_base_delegation_node(root_id, attestation_owner.clone()),
	);
	delegation_node.permissions = delegation::Permissions::ATTEST;
	delegation_node.parent = Some(parent_delegation_id);
	let mut attestation = generate_base_attestation(attestation_owner);
	attestation.delegation_id = Some(delegation_id);

	let mut operation = generate_base_attestation_revocation_details(claim_hash);
	operation.max_parent_checks = 0u32;

	let ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(attestation.ctype_hash, revoker.clone())])
		.build(None);
	let ext = delegation_mock::ExtBuilder::default()
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![
			(parent_delegation_id, parent_delegation_node),
			(delegation_id, delegation_node),
		])
		.with_children(vec![
			(root_id, vec![parent_delegation_id]),
			(parent_delegation_id, vec![delegation_id]),
		])
		.build(Some(ext));
	let mut ext = ExtBuilder::default()
		.with_attestations(vec![(operation.claim_hash, attestation)])
		.build(Some(ext));

	ext.execute_with(|| {
		assert_noop!(
			Attestation::revoke(
				get_origin(revoker.clone()),
				operation.claim_hash,
				operation.max_parent_checks
			),
			delegation::Error::<Test>::MaxSearchDepthReached
		);
	});
}

#[test]
fn revoked_delegation_revoke_error() {
	let revoker_keypair = get_alice_ed25519();
	let revoker = get_ed25519_account(revoker_keypair.public());
	let attestation_owner_keypair = get_bob_ed25519();
	let attestation_owner = get_ed25519_account(attestation_owner_keypair.public());
	let claim_hash = get_claim_hash(true);

	let (root_id, root_node) = (
		delegation_mock::get_delegation_root_id(true),
		delegation_mock::generate_base_delegation_root(revoker.clone()),
	);
	let (delegation_id, mut delegation_node) = (
		delegation_mock::get_delegation_id(true),
		delegation_mock::generate_base_delegation_node(root_id, revoker.clone()),
	);
	delegation_node.permissions = delegation::Permissions::ATTEST;
	delegation_node.revoked = true;
	let mut attestation = generate_base_attestation(attestation_owner);
	attestation.delegation_id = Some(delegation_id);

	let operation = generate_base_attestation_revocation_details(claim_hash);

	let ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(attestation.ctype_hash, revoker.clone())])
		.build(None);
	let ext = delegation_mock::ExtBuilder::default()
		.with_root_delegations(vec![(root_id, root_node)])
		.with_delegations(vec![(delegation_id, delegation_node)])
		.with_children(vec![(root_id, vec![delegation_id])])
		.build(Some(ext));
	let mut ext = ExtBuilder::default()
		.with_attestations(vec![(operation.claim_hash, attestation)])
		.build(Some(ext));

	ext.execute_with(|| {
		assert_noop!(
			Attestation::revoke(
				get_origin(revoker.clone()),
				operation.claim_hash,
				operation.max_parent_checks
			),
			attestation::Error::<Test>::UnauthorizedRevocation
		);
	});
}
