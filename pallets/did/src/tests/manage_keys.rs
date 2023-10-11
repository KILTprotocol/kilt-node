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
use frame_system::pallet_prelude::BlockNumberFor;
use sp_core::Pair;
use sp_runtime::SaturatedConversion;

use crate::{self as did, did_details::DidVerificationKey, mock::*, mock_utils::*};

#[test]
fn check_successful_authentication_key_update() {
	let old_auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_ed25519_key(old_auth_key.public());
	let new_auth_key = get_ed25519_authentication_key(&AUTH_SEED_1);

	let old_did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(old_auth_key.public()), Some(alice_did.clone()));

	let new_block_number: BlockNumberFor<Test> = 1;

	let origin = build_test_origin(alice_did.clone(), alice_did.clone());

	// Update authentication key. The old one should be removed.
	ExtBuilder::default()
		.with_dids(vec![(alice_did.clone(), old_did_details)])
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.build_and_execute_with_sanity_tests(None, || {
			System::set_block_number(new_block_number);
			assert_ok!(Did::set_authentication_key(
				origin,
				DidVerificationKey::from(new_auth_key.public())
			));
			let new_did_details = Did::get_did(&alice_did).expect("ALICE_DID should be present on chain.");
			assert_eq!(
				new_did_details.authentication_key,
				generate_key_id(&DidVerificationKey::from(new_auth_key.public()).into())
			);
			let public_keys = new_did_details.public_keys;
			// Total is +1 for the new auth key, -1 for the old auth key (replaced) = 1
			assert_eq!(public_keys.len(), 1);
			// Check for new authentication key
			assert!(public_keys.contains_key(&generate_key_id(
				&DidVerificationKey::from(new_auth_key.public()).into()
			)));
		});
}

#[test]
fn check_successful_authentication_key_max_public_keys_update() {
	let old_auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_ed25519_key(old_auth_key.public());
	let new_auth_key = get_ed25519_authentication_key(&AUTH_SEED_1);
	let key_agreement_keys = get_key_agreement_keys::<Test>(MaxTotalKeyAgreementKeys::get());

	let mut did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(old_auth_key.public()), Some(alice_did.clone()));
	assert_ok!(did_details.add_key_agreement_keys(key_agreement_keys, 0u64,));

	// Fill public key map to its max by adding
	// MaxPublicKeysPerDid - MaxTotalKeyAgreementKeys many keys
	did_details = fill_public_keys(did_details);

	let new_block_number: BlockNumberFor<Test> = 1;

	let origin = build_test_origin(alice_did.clone(), alice_did.clone());

	// Update authentication key. The old one should be removed.
	ExtBuilder::default()
		.with_dids(vec![(alice_did.clone(), did_details)])
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.build_and_execute_with_sanity_tests(None, || {
			System::set_block_number(new_block_number);
			assert_ok!(Did::set_authentication_key(
				origin,
				DidVerificationKey::from(new_auth_key.public())
			));

			let new_did_details = Did::get_did(&alice_did).expect("ALICE_DID should be present on chain.");
			assert_eq!(
				new_did_details.authentication_key,
				generate_key_id(&DidVerificationKey::from(new_auth_key.public()).into())
			);
			let public_keys = new_did_details.public_keys;
			// Total is the maximum allowed
			assert_eq!(public_keys.len(), MaxPublicKeysPerDid::get().saturated_into::<usize>());
			// Check for new authentication key
			assert!(public_keys.contains_key(&generate_key_id(
				&DidVerificationKey::from(new_auth_key.public()).into()
			)));
		});
}

#[test]
fn check_reused_key_authentication_key_update() {
	let old_auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_ed25519_key(old_auth_key.public());
	let old_delegation_key = old_auth_key;
	let new_auth_key = get_ed25519_authentication_key(&AUTH_SEED_1);

	let mut old_did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(old_auth_key.public()), Some(alice_did.clone()));
	// Same key for auth and del key
	assert_ok!(old_did_details.update_delegation_key(DidVerificationKey::from(old_delegation_key.public()), 0u64));

	let new_block_number: BlockNumberFor<Test> = 1;

	let origin = build_test_origin(alice_did.clone(), alice_did.clone());

	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(alice_did.clone(), old_did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			System::set_block_number(new_block_number);
			assert_ok!(Did::set_authentication_key(
				origin,
				DidVerificationKey::from(new_auth_key.public())
			));

			let new_did_details = Did::get_did(&alice_did).expect("ALICE_DID should be present on chain.");
			assert_eq!(
				new_did_details.authentication_key,
				generate_key_id(&DidVerificationKey::from(new_auth_key.public()).into())
			);
			let public_keys = new_did_details.public_keys;
			// Total is +1 for the new auth key (the old key is still used as delegation
			// key, so it is not removed)
			assert_eq!(public_keys.len(), 2);
			// Check for new authentication key
			assert!(public_keys.contains_key(&generate_key_id(
				&DidVerificationKey::from(new_auth_key.public()).into()
			)));
			// Check for old authentication key (delegation key)
			assert!(public_keys.contains_key(&generate_key_id(
				&DidVerificationKey::from(old_auth_key.public()).into()
			)));
		});
}

#[test]
fn check_max_keys_authentication_key_update_error() {
	let old_auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_ed25519_key(old_auth_key.public());
	let delegation_key = old_auth_key;
	let new_auth_key = get_ed25519_authentication_key(&AUTH_SEED_1);
	let key_agreement_keys = get_key_agreement_keys::<Test>(MaxTotalKeyAgreementKeys::get());

	let mut did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(old_auth_key.public()), Some(alice_did.clone()));
	assert_ok!(did_details.add_key_agreement_keys(key_agreement_keys, 0u64,));
	assert_ok!(did_details.update_delegation_key(DidVerificationKey::from(delegation_key.public()), 0u64));

	// Fill public key map to its max by adding
	// MaxPublicKeysPerDid - MaxTotalKeyAgreementKeys many keys
	did_details = fill_public_keys(did_details);

	let new_block_number: BlockNumberFor<Test> = 1;

	let origin = build_test_origin(alice_did.clone(), alice_did.clone());

	// Update authentication key. Since the old one is not removed because it is the
	// same as the delegation key, the update should fail as the max number of
	// public keys is already present.
	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(alice_did, did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			System::set_block_number(new_block_number);
			assert_noop!(
				Did::set_authentication_key(origin, DidVerificationKey::from(new_auth_key.public())),
				did::Error::<Test>::MaxPublicKeysExceeded
			);
		});
}

#[test]
fn check_did_not_present_authentication_key_update_error() {
	let old_auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_ed25519_key(old_auth_key.public());
	let new_auth_key = get_ed25519_authentication_key(&AUTH_SEED_1);

	let new_block_number: BlockNumberFor<Test> = 1;

	let origin = build_test_origin(alice_did.clone(), alice_did);

	// Update authentication key. The old one should be removed.
	ExtBuilder::default().build(None).execute_with(|| {
		System::set_block_number(new_block_number);
		assert_noop!(
			Did::set_authentication_key(origin, DidVerificationKey::from(new_auth_key.public())),
			did::Error::<Test>::NotFound
		);
	});
}

#[test]
fn check_successful_delegation_key_update() {
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let old_del_key = get_sr25519_delegation_key(&DEL_SEED_0);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let new_del_key = get_sr25519_delegation_key(&DEL_SEED_1);

	let mut old_did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(alice_did.clone()));
	assert_ok!(old_did_details.update_delegation_key(DidVerificationKey::from(old_del_key.public()), 0u64));

	let new_block_number: BlockNumberFor<Test> = 1;

	let origin = build_test_origin(alice_did.clone(), alice_did.clone());

	// Update delegation key. The old one should be removed.
	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(alice_did.clone(), old_did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			System::set_block_number(new_block_number);
			assert_ok!(Did::set_delegation_key(
				origin,
				DidVerificationKey::from(new_del_key.public())
			));

			let new_did_details = Did::get_did(&alice_did).expect("ALICE_DID should be present on chain.");
			assert_eq!(
				new_did_details.delegation_key,
				Some(generate_key_id(&DidVerificationKey::from(new_del_key.public()).into()))
			);
			let public_keys = new_did_details.public_keys;
			// Total is +1 for the new del key, -1 for the old del key (replaced) + auth key
			// = 2
			assert_eq!(public_keys.len(), 2);
			// Check for new delegation key
			assert!(public_keys.contains_key(&generate_key_id(&DidVerificationKey::from(new_del_key.public()).into())));
		});
}

#[test]
fn check_successful_delegation_key_max_public_keys_update() {
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let old_del_key = get_sr25519_delegation_key(&DEL_SEED_0);
	let new_del_key = get_sr25519_delegation_key(&DEL_SEED_1);
	let key_agreement_keys = get_key_agreement_keys::<Test>(MaxTotalKeyAgreementKeys::get());

	let mut did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(alice_did.clone()));
	assert_ok!(did_details.add_key_agreement_keys(key_agreement_keys, 0u64,));
	assert_ok!(did_details.update_delegation_key(DidVerificationKey::from(old_del_key.public()), 0u64));

	// Fill public key map to its max by adding
	// MaxPublicKeysPerDid - MaxTotalKeyAgreementKeys many keys
	did_details = fill_public_keys(did_details);

	let new_block_number: BlockNumberFor<Test> = 1;

	let origin = build_test_origin(alice_did.clone(), alice_did.clone());

	// Update delegation key. The old one should be removed.
	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(alice_did.clone(), did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			System::set_block_number(new_block_number);
			assert_ok!(Did::set_delegation_key(
				origin,
				DidVerificationKey::from(new_del_key.public())
			));

			let new_did_details = Did::get_did(&alice_did).expect("ALICE_DID should be present on chain.");
			assert_eq!(
				new_did_details.delegation_key,
				Some(generate_key_id(&DidVerificationKey::from(new_del_key.public()).into()))
			);
			let public_keys = new_did_details.public_keys;
			// Total is the maximum allowed
			assert_eq!(public_keys.len(), MaxPublicKeysPerDid::get().saturated_into::<usize>());
			// Check for new delegation key
			assert!(public_keys.contains_key(&generate_key_id(&DidVerificationKey::from(new_del_key.public()).into())));
		});
}

#[test]
fn check_reused_key_delegation_key_update() {
	let old_auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_ed25519_key(old_auth_key.public());
	let old_del_key = old_auth_key;
	let new_del_key = get_sr25519_delegation_key(&DEL_SEED_0);

	let mut old_did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(old_auth_key.public()), Some(alice_did.clone()));
	// Same key for auth and del key
	assert_ok!(old_did_details.update_delegation_key(DidVerificationKey::from(old_del_key.public()), 0u64));

	let new_block_number: BlockNumberFor<Test> = 1;

	let origin = build_test_origin(alice_did.clone(), alice_did.clone());

	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(alice_did.clone(), old_did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			System::set_block_number(new_block_number);
			assert_ok!(Did::set_delegation_key(
				origin,
				DidVerificationKey::from(new_del_key.public())
			));

			let new_did_details = Did::get_did(&alice_did).expect("ALICE_DID should be present on chain.");
			assert_eq!(
				new_did_details.delegation_key,
				Some(generate_key_id(&DidVerificationKey::from(new_del_key.public()).into()))
			);
			let public_keys = new_did_details.public_keys;
			// Total is +1 for the new del key (the old key is still used as authentication
			// key, so it is not removed)
			assert_eq!(public_keys.len(), 2);
			// Check for new delegation key
			assert!(public_keys.contains_key(&generate_key_id(&DidVerificationKey::from(new_del_key.public()).into())));
			// Check for old delegation key (authentication key)
			assert!(public_keys.contains_key(&generate_key_id(&DidVerificationKey::from(old_del_key.public()).into())));
		});
}

#[test]
fn check_max_public_keys_delegation_key_addition_error() {
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let new_del_key = get_sr25519_delegation_key(&DEL_SEED_1);
	let key_agreement_keys = get_key_agreement_keys::<Test>(MaxTotalKeyAgreementKeys::get());

	let mut did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(alice_did.clone()));
	assert_ok!(did_details.add_key_agreement_keys(key_agreement_keys, 0u64,));

	// Fill public key map to its max by adding
	// MaxPublicKeysPerDid - MaxTotalKeyAgreementKeys many keys
	did_details = fill_public_keys(did_details);

	let new_block_number: BlockNumberFor<Test> = 1;

	let origin = build_test_origin(alice_did.clone(), alice_did.clone());

	// Update delegation key. The old one should be removed.
	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(alice_did, did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			System::set_block_number(new_block_number);
			assert_noop!(
				Did::set_delegation_key(origin, DidVerificationKey::from(new_del_key.public())),
				did::Error::<Test>::MaxPublicKeysExceeded
			);
		});
}

#[test]
fn check_max_public_keys_reused_key_delegation_key_update_error() {
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let old_del_key = auth_key;
	let new_del_key = get_sr25519_delegation_key(&DEL_SEED_0);
	let key_agreement_keys = get_key_agreement_keys::<Test>(MaxTotalKeyAgreementKeys::get());

	let mut did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(alice_did.clone()));
	assert_ok!(did_details.add_key_agreement_keys(key_agreement_keys, 0u64,));
	// Same key for auth and delegation
	assert_ok!(did_details.update_delegation_key(DidVerificationKey::from(old_del_key.public()), 0u64));

	// Fill public key map to its max by adding
	// MaxPublicKeysPerDid - MaxTotalKeyAgreementKeys many keys
	did_details = fill_public_keys(did_details);

	let new_block_number: BlockNumberFor<Test> = 1;

	let origin = build_test_origin(alice_did.clone(), alice_did.clone());

	// Update delegation key. The old one should not be removed as it is still used
	// as authentication key.
	ExtBuilder::default()
		.with_dids(vec![(alice_did.clone(), did_details)])
		.with_balances(vec![(alice_did, DEFAULT_BALANCE)])
		.build_and_execute_with_sanity_tests(None, || {
			System::set_block_number(new_block_number);
			assert_noop!(
				Did::set_delegation_key(origin, DidVerificationKey::from(new_del_key.public())),
				did::Error::<Test>::MaxPublicKeysExceeded
			);
		});
}

#[test]
fn check_did_not_present_delegation_key_update_error() {
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let new_del_key = get_sr25519_delegation_key(&DEL_SEED_1);

	let new_block_number: BlockNumberFor<Test> = 1;

	let origin = build_test_origin(alice_did.clone(), alice_did);

	// Update delegation key. The old one should be removed.
	ExtBuilder::default().build(None).execute_with(|| {
		System::set_block_number(new_block_number);
		assert_noop!(
			Did::set_delegation_key(origin, DidVerificationKey::from(new_del_key.public())),
			did::Error::<Test>::NotFound
		);
	});
}

#[test]
fn check_successful_delegation_key_deletion() {
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let old_del_key = get_sr25519_delegation_key(&DEL_SEED_0);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());

	let mut old_did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(alice_did.clone()));
	assert_ok!(old_did_details.update_delegation_key(DidVerificationKey::from(old_del_key.public()), 0u64));

	let origin = build_test_origin(alice_did.clone(), alice_did.clone());

	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(alice_did.clone(), old_did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_ok!(Did::remove_delegation_key(origin));

			let new_did_details = Did::get_did(&alice_did).expect("ALICE_DID should be present on chain.");
			assert!(new_did_details.delegation_key.is_none());
			let public_keys = new_did_details.public_keys;
			// Total is -1 for the removal + auth key = 1
			assert_eq!(public_keys.len(), 1);
			// Check for new delegation key
			assert!(!public_keys.contains_key(&generate_key_id(&DidVerificationKey::from(old_del_key.public()).into())));
		});
}

#[test]
fn check_successful_reused_delegation_key_deletion() {
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let old_del_key = auth_key;
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());

	let mut old_did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(alice_did.clone()));
	assert_ok!(old_did_details.update_delegation_key(DidVerificationKey::from(old_del_key.public()), 0u64));

	let origin = build_test_origin(alice_did.clone(), alice_did.clone());

	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(alice_did.clone(), old_did_details.clone())])
		.build_and_execute_with_sanity_tests(None, || {
			assert_ok!(Did::remove_delegation_key(origin));

			let new_did_details = Did::get_did(&alice_did).expect("ALICE_DID should be present on chain.");
			assert!(new_did_details.delegation_key.is_none());
			let public_keys = new_did_details.public_keys;
			// Total should be unchanged as the key was re-used so it is not completely
			// deleted
			assert_eq!(public_keys.len(), old_did_details.public_keys.len());
			// Check for presence of old delegation key (authentication key)
			assert!(public_keys.contains_key(&generate_key_id(&DidVerificationKey::from(old_del_key.public()).into())));
		});
}

#[test]
fn check_did_not_present_delegation_key_deletion_error() {
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());

	let origin = build_test_origin(alice_did.clone(), alice_did);

	ExtBuilder::default().build(None).execute_with(|| {
		assert_noop!(Did::remove_delegation_key(origin), did::Error::<Test>::NotFound);
	});
}

#[test]
fn check_key_not_present_delegation_key_deletion_error() {
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());

	let old_did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(alice_did.clone()));

	let origin = build_test_origin(alice_did.clone(), alice_did.clone());

	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(alice_did, old_did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::remove_delegation_key(origin),
				did::Error::<Test>::VerificationKeyNotFound
			);
		});
}

#[test]
fn check_successful_attestation_key_update() {
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let old_att_key = get_sr25519_attestation_key(&ATT_SEED_0);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let new_att_key = get_sr25519_attestation_key(&ATT_SEED_1);

	let mut old_did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(alice_did.clone()));
	assert_ok!(old_did_details.update_attestation_key(DidVerificationKey::from(old_att_key.public()), 0u64));

	let new_block_number: BlockNumberFor<Test> = 1;

	let origin = build_test_origin(alice_did.clone(), alice_did.clone());

	// Update attestation key. The old one should be removed.
	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(alice_did.clone(), old_did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			System::set_block_number(new_block_number);
			assert_ok!(Did::set_attestation_key(
				origin,
				DidVerificationKey::from(new_att_key.public())
			));
			let new_did_details = Did::get_did(&alice_did).expect("ALICE_DID should be present on chain.");
			assert_eq!(
				new_did_details.attestation_key,
				Some(generate_key_id(&DidVerificationKey::from(new_att_key.public()).into()))
			);
			let public_keys = new_did_details.public_keys;
			// Total is +1 for the new att key, -1 for the old att key (replaced) + auth key
			// = 2
			assert_eq!(public_keys.len(), 2);
			// Check for new attestation key
			assert!(public_keys.contains_key(&generate_key_id(&DidVerificationKey::from(new_att_key.public()).into())));
		});
}

#[test]
fn check_successful_attestation_key_max_public_keys_update() {
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let old_att_key = get_sr25519_attestation_key(&ATT_SEED_0);
	let key_agreement_keys = get_key_agreement_keys::<Test>(MaxTotalKeyAgreementKeys::get());
	let new_att_key = get_sr25519_attestation_key(&ATT_SEED_1);

	let mut did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(alice_did.clone()));
	assert_ok!(did_details.add_key_agreement_keys(key_agreement_keys, 0u64,));
	assert_ok!(did_details.update_attestation_key(DidVerificationKey::from(old_att_key.public()), 0u64));

	// Fill public key map to its max by adding
	// MaxPublicKeysPerDid - MaxTotalKeyAgreementKeys many keys
	did_details = fill_public_keys(did_details);

	let new_block_number: BlockNumberFor<Test> = 1;

	let origin = build_test_origin(alice_did.clone(), alice_did.clone());

	// Update attestation key. The old one should be removed.
	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(alice_did.clone(), did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			System::set_block_number(new_block_number);
			assert_ok!(Did::set_attestation_key(
				origin,
				DidVerificationKey::from(new_att_key.public())
			));
			let new_did_details = Did::get_did(&alice_did).expect("ALICE_DID should be present on chain.");
			assert_eq!(
				new_did_details.attestation_key,
				Some(generate_key_id(&DidVerificationKey::from(new_att_key.public()).into()))
			);
			let public_keys = new_did_details.public_keys;
			// Total is the maximum allowed
			assert_eq!(public_keys.len(), MaxPublicKeysPerDid::get().saturated_into::<usize>());
			// Check for new attestation key
			assert!(public_keys.contains_key(&generate_key_id(&DidVerificationKey::from(new_att_key.public()).into())));
		});
}

#[test]
fn check_reused_key_attestation_key_update() {
	let old_auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_ed25519_key(old_auth_key.public());
	let old_att_key = old_auth_key;
	let new_att_key = get_sr25519_attestation_key(&ATT_SEED_0);

	let mut old_did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(old_auth_key.public()), Some(alice_did.clone()));
	// Same key for auth and att key
	assert_ok!(old_did_details.update_attestation_key(DidVerificationKey::from(old_att_key.public()), 0u64));

	let new_block_number: BlockNumberFor<Test> = 1;

	let origin = build_test_origin(alice_did.clone(), alice_did.clone());

	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(alice_did.clone(), old_did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			System::set_block_number(new_block_number);
			assert_ok!(Did::set_attestation_key(
				origin,
				DidVerificationKey::from(new_att_key.public())
			));

			let new_did_details = Did::get_did(&alice_did).expect("ALICE_DID should be present on chain.");
			assert_eq!(
				new_did_details.attestation_key,
				Some(generate_key_id(&DidVerificationKey::from(new_att_key.public()).into()))
			);
			let public_keys = new_did_details.public_keys;
			// Total is +1 for the new att key (the old key is still used as authentication
			// key, so it is not removed)
			assert_eq!(public_keys.len(), 2);
			// Check for new attestation key
			assert!(public_keys.contains_key(&generate_key_id(&DidVerificationKey::from(new_att_key.public()).into())));
			// Check for old attestation key (authentication key)
			assert!(public_keys.contains_key(&generate_key_id(&DidVerificationKey::from(old_att_key.public()).into())));
		});
}

#[test]
fn check_max_public_keys_attestation_key_addition_error() {
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let new_att_key = get_sr25519_attestation_key(&ATT_SEED_1);
	let key_agreement_keys = get_key_agreement_keys::<Test>(MaxTotalKeyAgreementKeys::get());

	let mut did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(alice_did.clone()));
	assert_ok!(did_details.add_key_agreement_keys(key_agreement_keys, 0u64,));

	// Fill public key map to its max by adding
	// MaxPublicKeysPerDid - MaxTotalKeyAgreementKeys many keys
	did_details = fill_public_keys(did_details);

	let new_block_number: BlockNumberFor<Test> = 1;

	let origin = build_test_origin(alice_did.clone(), alice_did.clone());

	// Update attestation key. The old one should be removed.
	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(alice_did, did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			System::set_block_number(new_block_number);
			assert_noop!(
				Did::set_attestation_key(origin, DidVerificationKey::from(new_att_key.public())),
				did::Error::<Test>::MaxPublicKeysExceeded
			);
		});
}

#[test]
fn check_max_public_keys_reused_key_attestation_key_update_error() {
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let old_att_key = auth_key;
	let new_att_key = get_sr25519_delegation_key(&DEL_SEED_0);
	let key_agreement_keys = get_key_agreement_keys::<Test>(MaxTotalKeyAgreementKeys::get());

	let mut did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(alice_did.clone()));
	assert_ok!(did_details.add_key_agreement_keys(key_agreement_keys, 0u64,));
	// Same key for auth and attestation
	assert_ok!(did_details.update_attestation_key(DidVerificationKey::from(old_att_key.public()), 0u64));

	// Fill public key map to its max by adding
	// MaxPublicKeysPerDid - MaxTotalKeyAgreementKeys many keys
	did_details = fill_public_keys(did_details);

	let new_block_number: BlockNumberFor<Test> = 1;

	let origin = build_test_origin(alice_did.clone(), alice_did.clone());

	// Update attestation key. The old one should not be removed as it is still used
	// as authentication key.
	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(alice_did, did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			System::set_block_number(new_block_number);
			assert_noop!(
				Did::set_attestation_key(origin, DidVerificationKey::from(new_att_key.public())),
				did::Error::<Test>::MaxPublicKeysExceeded
			);
		});
}

#[test]
fn check_did_not_present_attestation_key_update_error() {
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let new_att_key = get_sr25519_delegation_key(&DEL_SEED_1);

	let new_block_number: BlockNumberFor<Test> = 1;

	let origin = build_test_origin(alice_did.clone(), alice_did);

	// Update delegation key. The old one should be removed.
	ExtBuilder::default().build(None).execute_with(|| {
		System::set_block_number(new_block_number);
		assert_noop!(
			Did::set_delegation_key(origin, DidVerificationKey::from(new_att_key.public())),
			did::Error::<Test>::NotFound
		);
	});
}

#[test]
fn check_successful_attestation_key_deletion() {
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let old_att_key = get_sr25519_attestation_key(&ATT_SEED_0);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());

	let mut old_did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(alice_did.clone()));
	assert_ok!(old_did_details.update_attestation_key(DidVerificationKey::from(old_att_key.public()), 0u64));

	let origin = build_test_origin(alice_did.clone(), alice_did.clone());

	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(alice_did.clone(), old_did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_ok!(Did::remove_attestation_key(origin));

			let new_did_details = Did::get_did(&alice_did).expect("ALICE_DID should be present on chain.");
			assert!(new_did_details.attestation_key.is_none());
			let public_keys = new_did_details.public_keys;
			// Total is -1 for the removal + auth key = 1
			assert_eq!(public_keys.len(), 1);
			// Check for new attestation key
			assert!(!public_keys.contains_key(&generate_key_id(&DidVerificationKey::from(old_att_key.public()).into())));
		});
}

#[test]
fn check_successful_reused_attestation_key_deletion() {
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let old_att_key = auth_key;
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());

	let mut old_did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(alice_did.clone()));
	assert_ok!(old_did_details.update_attestation_key(DidVerificationKey::from(old_att_key.public()), 0u64));

	let origin = build_test_origin(alice_did.clone(), alice_did.clone());

	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(alice_did.clone(), old_did_details.clone())])
		.build_and_execute_with_sanity_tests(None, || {
			assert_ok!(Did::remove_attestation_key(origin));
			let new_did_details = Did::get_did(&alice_did).expect("ALICE_DID should be present on chain.");
			assert!(new_did_details.attestation_key.is_none());
			let public_keys = new_did_details.public_keys;
			// Total should be unchanged as the key was re-used so it is not completely
			// deleted
			assert_eq!(public_keys.len(), old_did_details.public_keys.len());
			// Check for presence of old delegation key (authentication key)
			assert!(public_keys.contains_key(&generate_key_id(&DidVerificationKey::from(old_att_key.public()).into())));
		});
}

#[test]
fn check_did_not_present_attestation_key_deletion_error() {
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());

	let origin = build_test_origin(alice_did.clone(), alice_did);

	ExtBuilder::default().build(None).execute_with(|| {
		assert_noop!(Did::remove_attestation_key(origin), did::Error::<Test>::NotFound);
	});
}

#[test]
fn check_key_not_present_attestation_key_deletion_error() {
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());

	let old_did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(alice_did.clone()));

	let origin = build_test_origin(alice_did.clone(), alice_did.clone());

	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(alice_did, old_did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::remove_attestation_key(origin),
				did::Error::<Test>::VerificationKeyNotFound
			);
		});
}

#[test]
fn check_successful_key_agreement_key_addition() {
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let new_key_agreement_key = get_x25519_encryption_key(&ENC_SEED_0);

	let old_did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(alice_did.clone()));

	let new_block_number: BlockNumberFor<Test> = 1;

	let origin = build_test_origin(alice_did.clone(), alice_did.clone());

	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(alice_did.clone(), old_did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			System::set_block_number(new_block_number);
			assert_ok!(Did::add_key_agreement_key(origin, new_key_agreement_key,));
			let new_did_details = Did::get_did(&alice_did).expect("ALICE_DID should be present on chain.");
			assert_eq!(new_did_details.key_agreement_keys.len(), 1);
			assert_eq!(
				new_did_details.key_agreement_keys.iter().next().unwrap(),
				&generate_key_id(&new_key_agreement_key.into())
			);
			let public_keys = new_did_details.public_keys;
			// Total is +1 for the new enc key + auth key = 2
			assert_eq!(public_keys.len(), 2);
			// Check for new key agreement key
			assert!(public_keys.contains_key(&generate_key_id(&new_key_agreement_key.into())));
		});
}

#[test]
fn check_max_public_keys_key_agreement_key_addition_error() {
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());

	let key_agreement_keys = get_key_agreement_keys::<Test>(MaxTotalKeyAgreementKeys::get());
	let new_key_agreement_key = get_x25519_encryption_key(&ENC_SEED_0);

	let mut did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(alice_did.clone()));
	assert_ok!(did_details.add_key_agreement_keys(key_agreement_keys, 0u64,));

	// Fill public key map to its max by adding
	// MaxPublicKeysPerDid - MaxTotalKeyAgreementKeys many keys
	did_details = fill_public_keys(did_details);

	let new_block_number: BlockNumberFor<Test> = 1;

	let origin = build_test_origin(alice_did.clone(), alice_did.clone());

	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(alice_did, did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			System::set_block_number(new_block_number);
			assert_noop!(
				Did::add_key_agreement_key(origin, new_key_agreement_key,),
				did::Error::<Test>::MaxPublicKeysExceeded
			);
		});
}

#[test]
fn check_did_not_present_key_agreement_key_addition_error() {
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let new_enc_key = get_x25519_encryption_key(&ENC_SEED_0);

	let new_block_number: BlockNumberFor<Test> = 1;

	let origin = build_test_origin(alice_did.clone(), alice_did);

	// Update delegation key. The old one should be removed.
	ExtBuilder::default().build(None).execute_with(|| {
		System::set_block_number(new_block_number);
		assert_noop!(
			Did::add_key_agreement_key(origin, new_enc_key),
			did::Error::<Test>::NotFound
		);
	});
}

#[test]
fn check_successful_key_agreement_key_deletion() {
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let old_enc_key = get_x25519_encryption_key(&ENC_SEED_0);

	let mut old_did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(alice_did.clone()));
	assert_ok!(old_did_details.add_key_agreement_key(old_enc_key, 0u64));

	let origin = build_test_origin(alice_did.clone(), alice_did.clone());

	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(alice_did.clone(), old_did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_ok!(Did::remove_key_agreement_key(
				origin,
				generate_key_id(&old_enc_key.into()),
			));
			let new_did_details = Did::get_did(&alice_did).expect("ALICE_DID should be present on chain.");
			assert!(new_did_details.key_agreement_keys.is_empty());
			let public_keys = new_did_details.public_keys;
			// Total is -1 for the enc key removal + auth key = 1
			assert_eq!(public_keys.len(), 1);
			// Check for new key agreement key
			assert!(!public_keys.contains_key(&generate_key_id(&old_enc_key.into())));
		});
}

#[test]
fn check_did_not_found_key_agreement_key_deletion_error() {
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let test_enc_key = get_x25519_encryption_key(&ENC_SEED_0);

	let origin = build_test_origin(alice_did.clone(), alice_did);

	ExtBuilder::default().build(None).execute_with(|| {
		assert_noop!(
			Did::remove_key_agreement_key(origin, generate_key_id(&test_enc_key.into())),
			did::Error::<Test>::NotFound
		);
	});
}

#[test]
fn check_key_not_found_key_agreement_key_deletion_error() {
	let auth_key = get_ed25519_authentication_key(&AUTH_SEED_0);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let test_enc_key = get_x25519_encryption_key(&ENC_SEED_0);

	// No enc key added
	let old_did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(alice_did.clone()));

	let origin = build_test_origin(alice_did.clone(), alice_did.clone());

	ExtBuilder::default()
		.with_balances(vec![(alice_did.clone(), DEFAULT_BALANCE)])
		.with_dids(vec![(alice_did, old_did_details)])
		.build_and_execute_with_sanity_tests(None, || {
			assert_noop!(
				Did::remove_key_agreement_key(origin, generate_key_id(&test_enc_key.into())),
				did::Error::<Test>::VerificationKeyNotFound
			);
		});
}
