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
use sp_core::*;
use sp_std::{collections::btree_set::BTreeSet, convert::TryFrom};

use crate::{self as did, mock::*};
use ctype::mock as ctype_mock;

// submit_did_create_operation

#[test]
fn check_successful_simple_ed25519_creation() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let auth_did_key = did::DidVerificationKey::from(auth_key.public());
	let operation = generate_base_did_creation_operation(alice_did.clone());

	let signature = auth_key.sign(operation.encode().as_ref());

	let mut ext = ExtBuilder::default().build(None);

	ext.execute_with(|| {
		assert_ok!(Did::submit_did_create_operation(
			Origin::signed(DEFAULT_ACCOUNT),
			operation.clone(),
			did::DidSignature::from(signature),
		));
	});

	let stored_did = ext.execute_with(|| Did::get_did(&alice_did).expect("ALICE_DID should be present on chain."));
	assert_eq!(
		stored_did.get_authentication_key_id(),
		generate_key_id(&auth_did_key.clone().into())
	);
	assert_eq!(stored_did.get_key_agreement_keys_ids().len(), 0);
	assert_eq!(stored_did.get_delegation_key_id(), &None);
	assert_eq!(stored_did.get_attestation_key_id(), &None);
	assert_eq!(stored_did.get_public_keys().len(), 1);
	assert!(stored_did
		.get_public_keys()
		.contains_key(&generate_key_id(&auth_did_key.into())));
	assert_eq!(stored_did.endpoint_url, None);
	assert_eq!(stored_did.last_tx_counter, 0u64);
}

#[test]
fn check_successful_simple_sr25519_creation() {
	let auth_key = get_sr25519_authentication_key(true);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	let auth_did_key = did::DidVerificationKey::from(auth_key.public());
	let operation = generate_base_did_creation_operation(alice_did.clone());

	let signature = auth_key.sign(operation.encode().as_ref());

	let mut ext = ExtBuilder::default().build(None);

	ext.execute_with(|| {
		assert_ok!(Did::submit_did_create_operation(
			Origin::signed(DEFAULT_ACCOUNT),
			operation.clone(),
			did::DidSignature::from(signature),
		));
	});

	let stored_did = ext.execute_with(|| Did::get_did(&alice_did).expect("ALICE_DID should be present on chain."));
	assert_eq!(
		stored_did.get_authentication_key_id(),
		generate_key_id(&auth_did_key.clone().into())
	);
	assert_eq!(stored_did.get_key_agreement_keys_ids().len(), 0);
	assert_eq!(stored_did.get_delegation_key_id(), &None);
	assert_eq!(stored_did.get_attestation_key_id(), &None);
	assert_eq!(stored_did.get_public_keys().len(), 1);
	assert!(stored_did
		.get_public_keys()
		.contains_key(&generate_key_id(&auth_did_key.into())));
	assert_eq!(stored_did.endpoint_url, None);
	assert_eq!(stored_did.last_tx_counter, 0u64);
}

#[test]
fn check_successful_simple_ecdsa_creation() {
	let auth_key = get_ecdsa_authentication_key(true);
	let alice_did = get_did_identifier_from_ecdsa_key(auth_key.public());
	let auth_did_key = did::DidVerificationKey::from(auth_key.public());
	let operation = generate_base_did_creation_operation(alice_did.clone());

	let signature = auth_key.sign(operation.encode().as_ref());

	let mut ext = ExtBuilder::default().build(None);

	ext.execute_with(|| {
		assert_ok!(Did::submit_did_create_operation(
			Origin::signed(DEFAULT_ACCOUNT),
			operation.clone(),
			did::DidSignature::from(signature),
		));
	});

	let stored_did = ext.execute_with(|| Did::get_did(&alice_did).expect("ALICE_DID should be present on chain."));
	assert_eq!(
		stored_did.get_authentication_key_id(),
		generate_key_id(&auth_did_key.clone().into())
	);
	assert_eq!(stored_did.get_key_agreement_keys_ids().len(), 0);
	assert_eq!(stored_did.get_delegation_key_id(), &None);
	assert_eq!(stored_did.get_attestation_key_id(), &None);
	assert_eq!(stored_did.get_public_keys().len(), 1);
	assert!(stored_did
		.get_public_keys()
		.contains_key(&generate_key_id(&auth_did_key.into())));
	assert_eq!(stored_did.endpoint_url, None);
	assert_eq!(stored_did.last_tx_counter, 0u64);
}

#[test]
fn check_successful_complete_creation() {
	let auth_key = get_sr25519_authentication_key(true);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	let auth_did_key = did::DidVerificationKey::from(auth_key.public());
	let enc_keys: BTreeSet<did::DidEncryptionKey> =
		vec![get_x25519_encryption_key(true), get_x25519_encryption_key(false)]
			.iter()
			.copied()
			.collect();
	let del_key = get_sr25519_delegation_key(true);
	let att_key = get_ecdsa_attestation_key(true);
	let new_url = did::Url::from(
		did::HttpUrl::try_from("https://new_kilt.io".as_bytes())
			.expect("https://new_kilt.io should not be considered an invalid HTTP URL."),
	);
	let mut operation = generate_base_did_creation_operation(alice_did.clone());
	operation.new_key_agreement_keys = enc_keys.clone();
	operation.new_attestation_key = Some(did::DidVerificationKey::from(att_key.public()));
	operation.new_delegation_key = Some(did::DidVerificationKey::from(del_key.public()));
	operation.new_endpoint_url = Some(new_url);

	let signature = auth_key.sign(operation.encode().as_ref());

	let mut ext = ExtBuilder::default().build(None);

	ext.execute_with(|| {
		assert_ok!(Did::submit_did_create_operation(
			Origin::signed(DEFAULT_ACCOUNT),
			operation.clone(),
			did::DidSignature::from(signature),
		));
	});

	let stored_did = ext.execute_with(|| Did::get_did(&alice_did).expect("ALICE_DID should be present on chain."));
	assert_eq!(
		stored_did.get_authentication_key_id(),
		generate_key_id(&auth_did_key.clone().into())
	);
	assert_eq!(stored_did.get_key_agreement_keys_ids().len(), 2);
	for key in enc_keys.iter().copied() {
		assert!(stored_did
			.get_key_agreement_keys_ids()
			.contains(&generate_key_id(&key.into())))
	}
	assert_eq!(
		stored_did.get_delegation_key_id(),
		&Some(generate_key_id(&operation.new_delegation_key.clone().unwrap().into()))
	);
	assert_eq!(
		stored_did.get_attestation_key_id(),
		&Some(generate_key_id(&operation.new_attestation_key.clone().unwrap().into()))
	);
	// Authentication key + 2 * Encryption key + Delegation key + Attestation key =
	// 5
	assert_eq!(stored_did.get_public_keys().len(), 5);
	assert!(stored_did
		.get_public_keys()
		.contains_key(&generate_key_id(&auth_did_key.into())));
	let mut key_agreement_keys_iterator = operation.new_key_agreement_keys.iter().copied();
	assert!(stored_did
		.get_public_keys()
		.contains_key(&generate_key_id(&key_agreement_keys_iterator.next().unwrap().into())));
	assert!(stored_did
		.get_public_keys()
		.contains_key(&generate_key_id(&key_agreement_keys_iterator.next().unwrap().into())));
	assert!(stored_did
		.get_public_keys()
		.contains_key(&generate_key_id(&operation.new_attestation_key.clone().unwrap().into())));
	assert!(stored_did
		.get_public_keys()
		.contains_key(&generate_key_id(&operation.new_delegation_key.clone().unwrap().into())));
}

#[test]
fn check_duplicate_did_creation() {
	let auth_key = get_sr25519_authentication_key(true);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	let auth_did_key = did::DidVerificationKey::from(auth_key.public());
	let mock_did = generate_base_did_details(auth_did_key);
	let operation = generate_base_did_creation_operation(alice_did.clone());

	let signature = auth_key.sign(operation.encode().as_ref());

	let mut ext = ExtBuilder::default().with_dids(vec![(alice_did, mock_did)]).build(None);

	ext.execute_with(|| {
		assert_noop!(
			Did::submit_did_create_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature),
			),
			did::Error::<Test>::DidAlreadyPresent
		);
	});
}

#[test]
fn check_invalid_signature_format_did_creation() {
	let auth_key = get_sr25519_authentication_key(true);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	// Using an Ed25519 key where an Sr25519 is expected
	let invalid_key = get_ed25519_authentication_key(true);
	// DID creation contains auth_key, but signature is generated using invalid_key
	let operation = generate_base_did_creation_operation(alice_did);

	let signature = invalid_key.sign(operation.encode().as_ref());

	let mut ext = ExtBuilder::default().build(None);

	ext.execute_with(|| {
		assert_noop!(
			Did::submit_did_create_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature),
			),
			did::Error::<Test>::InvalidSignature
		);
	});
}

#[test]
fn check_invalid_signature_did_creation() {
	let auth_key = get_sr25519_authentication_key(true);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	let alternative_key = get_sr25519_authentication_key(false);
	let operation = generate_base_did_creation_operation(alice_did);

	let signature = alternative_key.sign(operation.encode().as_ref());

	let mut ext = ExtBuilder::default().build(None);

	ext.execute_with(|| {
		assert_noop!(
			Did::submit_did_create_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature),
			),
			did::Error::<Test>::InvalidSignature
		);
	});
}

#[test]
fn check_max_limit_key_agreement_keys_did_creation() {
	let auth_key = get_sr25519_authentication_key(true);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	// Max keys allowed + 1
	let enc_keys = get_key_agreement_keys(<Test as did::Config>::MaxNewKeyAgreementKeys::get().saturating_add(1));
	let mut operation = generate_base_did_creation_operation(alice_did);
	operation.new_key_agreement_keys = enc_keys;

	let signature = auth_key.sign(operation.encode().as_ref());

	let mut ext = ExtBuilder::default().build(None);

	ext.execute_with(|| {
		assert_noop!(
			Did::submit_did_create_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature),
			),
			did::Error::<Test>::MaxKeyAgreementKeysLimitExceeded
		);
	});
}

#[test]
fn check_url_too_long_did_creation() {
	let auth_key = get_sr25519_authentication_key(true);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	let url_endpoint = get_url_endpoint(<Test as did::Config>::MaxUrlLength::get().saturating_add(1));
	let mut operation = generate_base_did_creation_operation(alice_did);
	// Max length allowed + 1
	operation.new_endpoint_url = Some(url_endpoint);

	let signature = auth_key.sign(operation.encode().as_ref());

	let mut ext = ExtBuilder::default().build(None);

	ext.execute_with(|| {
		assert_noop!(
			Did::submit_did_create_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature),
			),
			did::Error::<Test>::MaxUrlLengthExceeded
		);
	});
}

// submit_did_update_operation

#[test]
fn check_successful_complete_update() {
	let old_auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(old_auth_key.public());
	let new_auth_key = get_ed25519_authentication_key(false);
	let old_enc_key = get_x25519_encryption_key(true);
	let new_enc_key = get_x25519_encryption_key(false);
	let old_att_key = get_ed25519_attestation_key(true);
	let new_att_key = get_ed25519_attestation_key(false);
	let new_del_key = get_sr25519_delegation_key(true);
	let new_url = did::Url::from(
		did::HttpUrl::try_from("https://new_kilt.io".as_bytes())
			.expect("https://new_kilt.io should not be considered an invalid HTTP URL."),
	);

	let mut old_did_details = generate_base_did_details(did::DidVerificationKey::from(old_auth_key.public()));
	old_did_details.add_key_agreement_keys(
		vec![old_enc_key]
			.iter()
			.copied()
			.collect::<BTreeSet<did::DidEncryptionKey>>(),
		0u64,
	);
	old_did_details.update_attestation_key(did::DidVerificationKey::from(old_att_key.public()), 0u64);

	// Update all keys, URL endpoint and tx counter. The old key agreement key is
	// removed.
	let mut operation = generate_base_did_update_operation(alice_did.clone());
	operation.new_authentication_key = Some(did::DidVerificationKey::from(new_auth_key.public()));
	operation.new_key_agreement_keys = vec![new_enc_key]
		.iter()
		.copied()
		.collect::<BTreeSet<did::DidEncryptionKey>>();
	operation.attestation_key_update =
		did::DidVerificationKeyUpdateAction::Change(did::DidVerificationKey::from(new_att_key.public()));
	operation.delegation_key_update =
		did::DidVerificationKeyUpdateAction::Change(did::DidVerificationKey::from(new_del_key.public()));
	operation.public_keys_to_remove = vec![generate_key_id(&old_enc_key.into())]
		.iter()
		.copied()
		.collect::<BTreeSet<TestKeyId>>();
	operation.new_endpoint_url = Some(new_url);
	operation.tx_counter = old_did_details.last_tx_counter + 1u64;

	// Generate signature using the old authentication key
	let signature = old_auth_key.sign(operation.encode().as_ref());

	let mut ext = ExtBuilder::default()
		.with_dids(vec![(alice_did.clone(), old_did_details.clone())])
		.build(None);

	let new_block_number: TestBlockNumber = 1;

	ext.execute_with(|| {
		System::set_block_number(new_block_number);
		assert_ok!(Did::submit_did_update_operation(
			Origin::signed(DEFAULT_ACCOUNT),
			operation.clone(),
			did::DidSignature::from(signature),
		));
	});

	let new_did_details = ext.execute_with(|| Did::get_did(&alice_did).expect("ALICE_DID should be present on chain."));
	assert_eq!(
		new_did_details.get_authentication_key_id(),
		generate_key_id(&did::DidVerificationKey::from(new_auth_key.public()).into())
	);
	// Old one deleted, new one added -> Total keys = 1
	assert_eq!(new_did_details.get_key_agreement_keys_ids().len(), 1);
	assert_eq!(
		new_did_details.get_key_agreement_keys_ids().iter().next().unwrap(),
		&generate_key_id(&new_enc_key.into())
	);
	assert_eq!(
		new_did_details.get_attestation_key_id(),
		&Some(generate_key_id(
			&did::DidVerificationKey::from(new_att_key.public()).into()
		))
	);
	assert_eq!(
		new_did_details.get_delegation_key_id(),
		&Some(generate_key_id(
			&did::DidVerificationKey::from(new_del_key.public()).into()
		))
	);

	// Total is +1 for the new auth key, -1 for the old auth key (replaced), +1 for
	// the new key agreement key, -1 for the old key agreement key (deleted), +1 for
	// the old attestation key, +1 for the new attestation key, +1 for the new
	// delegation key = 5
	let public_keys = new_did_details.get_public_keys();
	assert_eq!(public_keys.len(), 5);
	// Check for new authentication key
	assert!(public_keys.contains_key(&generate_key_id(
		&did::DidVerificationKey::from(new_att_key.public()).into()
	)));
	// Check for new key agreement key
	assert!(public_keys.contains_key(&generate_key_id(&new_enc_key.into())));
	// Check for old attestation key
	assert!(public_keys.contains_key(&generate_key_id(
		&did::DidVerificationKey::from(old_att_key.public()).into()
	)));
	// Check for new attestation key
	assert!(public_keys.contains_key(&generate_key_id(
		&did::DidVerificationKey::from(new_att_key.public()).into()
	)));
	// Check for new delegation key
	assert!(public_keys.contains_key(&generate_key_id(
		&did::DidVerificationKey::from(new_del_key.public()).into()
	)));
	assert_eq!(new_did_details.last_tx_counter, old_did_details.last_tx_counter + 1u64);
}

#[test]
fn check_successful_keys_deletion_update() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let att_key = get_ed25519_attestation_key(true);
	let del_key = get_sr25519_delegation_key(true);

	let mut old_did_details = generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));
	old_did_details.update_attestation_key(did::DidVerificationKey::from(att_key.public()), 0u64);
	old_did_details.update_delegation_key(did::DidVerificationKey::from(del_key.public()), 0u64);

	// Remove both attestation and delegation key
	let mut operation = generate_base_did_update_operation(alice_did.clone());
	operation.attestation_key_update = did::DidVerificationKeyUpdateAction::Delete;
	operation.delegation_key_update = did::DidVerificationKeyUpdateAction::Delete;
	operation.tx_counter = old_did_details.last_tx_counter + 1u64;

	let signature = auth_key.sign(operation.encode().as_ref());

	let mut ext = ExtBuilder::default()
		.with_dids(vec![(alice_did.clone(), old_did_details.clone())])
		.build(None);

	let new_block_number: TestBlockNumber = 1;

	ext.execute_with(|| {
		System::set_block_number(new_block_number);
		assert_ok!(Did::submit_did_update_operation(
			Origin::signed(DEFAULT_ACCOUNT),
			operation.clone(),
			did::DidSignature::from(signature),
		));
	});

	// Auth key and key agreement key unchanged
	let new_did_details = ext.execute_with(|| Did::get_did(&alice_did).expect("ALICE_DID should be present on chain."));
	assert_eq!(
		new_did_details.get_authentication_key_id(),
		old_did_details.get_authentication_key_id()
	);
	assert_eq!(
		new_did_details.get_key_agreement_keys_ids(),
		old_did_details.get_key_agreement_keys_ids()
	);
	assert_eq!(new_did_details.get_attestation_key_id(), &None);
	assert_eq!(new_did_details.get_delegation_key_id(), &None);

	// Public keys should now contain only the authentication key and the revoked
	// attestation key
	let stored_public_keys = new_did_details.get_public_keys();
	assert_eq!(stored_public_keys.len(), 2);
	assert!(stored_public_keys.contains_key(&generate_key_id(
		&did::DidVerificationKey::from(auth_key.public()).into()
	)));
	assert!(stored_public_keys.contains_key(&generate_key_id(
		&did::DidVerificationKey::from(att_key.public()).into()
	)));
	assert_eq!(new_did_details.last_tx_counter, old_did_details.last_tx_counter + 1u64);
}

#[test]
fn check_successful_keys_overwrite_update() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	// Same as the authentication key -> leads to two keys having the same ID
	let new_att_key = auth_key.clone();

	let old_did_details = generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));

	// Remove both attestation and delegation key
	let mut operation = generate_base_did_update_operation(alice_did.clone());
	operation.attestation_key_update =
		did::DidVerificationKeyUpdateAction::Change(did::DidVerificationKey::from(new_att_key.public()));

	let signature = auth_key.sign(operation.encode().as_ref());

	let mut ext = ExtBuilder::default()
		.with_dids(vec![(alice_did.clone(), old_did_details.clone())])
		.build(None);

	let new_block_number: TestBlockNumber = 1;

	ext.execute_with(|| {
		System::set_block_number(new_block_number);
		assert_ok!(Did::submit_did_update_operation(
			Origin::signed(DEFAULT_ACCOUNT),
			operation.clone(),
			did::DidSignature::from(signature),
		));
	});

	// Auth key unchanged
	let new_did_details = ext.execute_with(|| Did::get_did(&alice_did).expect("ALICE_DID should be present on chain."));
	assert_eq!(
		new_did_details.get_authentication_key_id(),
		old_did_details.get_authentication_key_id()
	);
	// New attestation key and authentication key should now have the same ID
	assert_eq!(
		new_did_details.get_attestation_key_id(),
		&Some(old_did_details.get_authentication_key_id())
	);

	// As the two keys have the same ID, public keys still contain only one element
	let stored_public_keys = new_did_details.get_public_keys();
	assert_eq!(stored_public_keys.len(), 1);
	assert!(stored_public_keys.contains_key(&generate_key_id(
		&did::DidVerificationKey::from(auth_key.public()).into()
	)));
	assert!(stored_public_keys.contains_key(&generate_key_id(
		&did::DidVerificationKey::from(new_att_key.public()).into()
	)));
	// The block number should be the updated to the latest one, even if the ID was
	// also present before.
	let stored_key_details = stored_public_keys
		.get(&old_did_details.get_authentication_key_id())
		.expect("There should be a key with the given ID stored on chain.");
	assert_eq!(stored_key_details.block_number, new_block_number);
	assert_eq!(new_did_details.last_tx_counter, old_did_details.last_tx_counter + 1u64);
}

#[test]
fn check_successful_keys_multiuse_update() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	// Same as the authentication key -> leads to two keys having the same ID
	let old_att_key = auth_key.clone();

	let mut old_did_details = generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));
	old_did_details.update_attestation_key(did::DidVerificationKey::from(old_att_key.public()), 0u64);

	// Remove attestation key
	let mut operation = generate_base_did_update_operation(alice_did.clone());
	operation.attestation_key_update = did::DidVerificationKeyUpdateAction::Delete;

	let signature = auth_key.sign(operation.encode().as_ref());

	let mut ext = ExtBuilder::default()
		.with_dids(vec![(alice_did.clone(), old_did_details.clone())])
		.build(None);

	let new_block_number: TestBlockNumber = 1;

	ext.execute_with(|| {
		System::set_block_number(new_block_number);
		assert_ok!(Did::submit_did_update_operation(
			Origin::signed(DEFAULT_ACCOUNT),
			operation.clone(),
			did::DidSignature::from(signature),
		));
	});

	// Auth key unchanged
	let new_did_details = ext.execute_with(|| Did::get_did(&alice_did).expect("ALICE_DID should be present on chain."));
	assert_eq!(
		new_did_details.get_authentication_key_id(),
		old_did_details.get_authentication_key_id()
	);
	// Attestation key should now be set to None
	assert_eq!(new_did_details.get_attestation_key_id(), &None);

	// As the two keys have the same ID, public keys still contain only one element
	let stored_public_keys = new_did_details.get_public_keys();
	assert_eq!(stored_public_keys.len(), 1);
	assert!(stored_public_keys.contains_key(&generate_key_id(
		&did::DidVerificationKey::from(auth_key.public()).into()
	)));
}

#[test]
fn check_did_not_present_update() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let bob_auth_key = get_ed25519_authentication_key(false);
	let bob_did = get_did_identifier_from_ed25519_key(bob_auth_key.public());
	let mock_did = generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));
	let operation = generate_base_did_update_operation(alice_did);

	let signature = auth_key.sign(operation.encode().as_ref());

	let mut ext = ExtBuilder::default().with_dids(vec![(bob_did, mock_did)]).build(None);

	ext.execute_with(|| {
		assert_noop!(
			Did::submit_did_update_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature),
			),
			did::Error::<Test>::DidNotPresent
		);
	});
}

#[test]
fn check_did_max_counter_update() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let mut mock_did = generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));
	mock_did.last_tx_counter = u64::MAX;
	let operation = generate_base_did_update_operation(alice_did.clone());

	let signature = auth_key.sign(operation.encode().as_ref());

	let mut ext = ExtBuilder::default().with_dids(vec![(alice_did, mock_did)]).build(None);

	ext.execute_with(|| {
		assert_noop!(
			Did::submit_did_update_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature),
			),
			did::Error::<Test>::MaxTxCounterValue
		);
	});
}

#[test]
fn check_smaller_tx_counter_did_update() {
	let auth_key = get_sr25519_authentication_key(true);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	let mut mock_did = generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));
	mock_did.last_tx_counter = 1;
	let mut operation = generate_base_did_update_operation(alice_did.clone());
	operation.tx_counter = mock_did.last_tx_counter - 1;

	let signature = auth_key.sign(operation.encode().as_ref());

	let mut ext = ExtBuilder::default().with_dids(vec![(alice_did, mock_did)]).build(None);

	ext.execute_with(|| {
		assert_noop!(
			Did::submit_did_update_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature),
			),
			did::Error::<Test>::InvalidNonce
		);
	});
}

#[test]
fn check_equal_tx_counter_did_update() {
	let auth_key = get_sr25519_authentication_key(true);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	let mock_did = generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));
	let mut operation = generate_base_did_update_operation(alice_did.clone());
	operation.tx_counter = mock_did.last_tx_counter;

	let signature = auth_key.sign(operation.encode().as_ref());

	let mut ext = ExtBuilder::default().with_dids(vec![(alice_did, mock_did)]).build(None);

	ext.execute_with(|| {
		assert_noop!(
			Did::submit_did_update_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature),
			),
			did::Error::<Test>::InvalidNonce
		);
	});
}

#[test]
fn check_too_large_tx_counter_did_update() {
	let auth_key = get_sr25519_authentication_key(true);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	let mock_did = generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));
	let mut operation = generate_base_did_update_operation(alice_did.clone());
	operation.tx_counter = mock_did.last_tx_counter + 2;

	let signature = auth_key.sign(operation.encode().as_ref());

	let mut ext = ExtBuilder::default().with_dids(vec![(alice_did, mock_did)]).build(None);

	ext.execute_with(|| {
		assert_noop!(
			Did::submit_did_update_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature),
			),
			did::Error::<Test>::InvalidNonce
		);
	});
}

#[test]
fn check_invalid_signature_format_did_update() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	// Using an Sr25519 key where an Ed25519 is expected
	let invalid_key = get_sr25519_authentication_key(true);
	// DID update contains auth_key, but signature is generated using invalid_key
	let mock_did = generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));
	let operation = generate_base_did_update_operation(alice_did.clone());

	let signature = invalid_key.sign(operation.encode().as_ref());

	let mut ext = ExtBuilder::default().with_dids(vec![(alice_did, mock_did)]).build(None);

	ext.execute_with(|| {
		assert_noop!(
			Did::submit_did_update_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature),
			),
			did::Error::<Test>::InvalidSignatureFormat
		);
	});
}

#[test]
fn check_invalid_signature_did_update() {
	let auth_key = get_sr25519_authentication_key(true);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	// Using an Sr25519 key as expected, but from a different seed (default = false)
	let alternative_key = get_sr25519_authentication_key(false);
	let mock_did = generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));
	let operation = generate_base_did_update_operation(alice_did.clone());

	let signature = alternative_key.sign(operation.encode().as_ref());

	let mut ext = ExtBuilder::default().with_dids(vec![(alice_did, mock_did)]).build(None);

	ext.execute_with(|| {
		assert_noop!(
			Did::submit_did_update_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature),
			),
			did::Error::<Test>::InvalidSignature
		);
	});
}

#[test]
fn check_max_limit_key_agreement_keys_did_update() {
	let auth_key = get_sr25519_authentication_key(true);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());

	let old_did_details = generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));

	// Max keys allowed + 1
	let new_enc_keys = get_key_agreement_keys(<Test as did::Config>::MaxNewKeyAgreementKeys::get().saturating_add(1));

	let mut operation = generate_base_did_update_operation(alice_did.clone());
	operation.new_key_agreement_keys = new_enc_keys;

	let signature = auth_key.sign(operation.encode().as_ref());

	let mut ext = ExtBuilder::default()
		.with_dids(vec![(alice_did, old_did_details)])
		.build(None);

	let new_block_number: TestBlockNumber = 1;

	ext.execute_with(|| {
		System::set_block_number(new_block_number);
		assert_noop!(
			Did::submit_did_update_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature),
			),
			did::Error::<Test>::MaxKeyAgreementKeysLimitExceeded
		);
	});
}

#[test]
fn check_max_limit_public_keys_to_remove_did_update() {
	let auth_key = get_sr25519_authentication_key(true);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());

	let old_did_details = generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));

	// Max keys allowed + 1
	let keys_ids_to_remove =
		get_public_keys_to_remove(<Test as did::Config>::MaxNewKeyAgreementKeys::get().saturating_add(1));

	let mut operation = generate_base_did_update_operation(alice_did.clone());
	operation.public_keys_to_remove = keys_ids_to_remove;

	let signature = auth_key.sign(operation.encode().as_ref());

	let mut ext = ExtBuilder::default()
		.with_dids(vec![(alice_did, old_did_details)])
		.build(None);

	let new_block_number: TestBlockNumber = 1;

	ext.execute_with(|| {
		System::set_block_number(new_block_number);
		assert_noop!(
			Did::submit_did_update_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature),
			),
			did::Error::<Test>::MaxVerificationKeysToRemoveLimitExceeded
		);
	});
}

#[test]
fn check_url_too_long_did_update() {
	let auth_key = get_sr25519_authentication_key(true);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());

	let old_did_details = generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));

	// Max URL length allowed + 1
	let new_endpoint_url = get_url_endpoint(<Test as did::Config>::MaxUrlLength::get().saturating_add(1));

	let mut operation = generate_base_did_update_operation(alice_did.clone());
	operation.new_endpoint_url = Some(new_endpoint_url);

	let signature = auth_key.sign(operation.encode().as_ref());

	let mut ext = ExtBuilder::default()
		.with_dids(vec![(alice_did, old_did_details)])
		.build(None);

	let new_block_number: TestBlockNumber = 1;

	ext.execute_with(|| {
		System::set_block_number(new_block_number);
		assert_noop!(
			Did::submit_did_update_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature),
			),
			did::Error::<Test>::MaxUrlLengthExceeded
		);
	});
}

#[test]
fn check_currently_active_authentication_key_update() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());

	let old_did_details = generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));

	// Remove both attestation and delegation key
	let mut operation = generate_base_did_update_operation(alice_did.clone());
	// Trying to remove the currently active authentication key
	operation.public_keys_to_remove = vec![generate_key_id(
		&did::DidVerificationKey::from(auth_key.public()).into(),
	)]
	.iter()
	.copied()
	.collect::<BTreeSet<TestKeyId>>();

	let signature = auth_key.sign(operation.encode().as_ref());

	let mut ext = ExtBuilder::default()
		.with_dids(vec![(alice_did, old_did_details)])
		.build(None);

	let new_block_number: TestBlockNumber = 1;

	ext.execute_with(|| {
		System::set_block_number(new_block_number);
		assert_noop!(
			Did::submit_did_update_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature),
			),
			did::Error::<Test>::CurrentlyActiveKey
		);
	});
}

#[test]
fn check_currently_active_delegation_key_update() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let del_key = get_ecdsa_delegation_key(true);

	let mut old_did_details = generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));
	old_did_details.update_delegation_key(did::DidVerificationKey::from(del_key.public()), 0u64);

	// Remove both attestation and delegation key
	let mut operation = generate_base_did_update_operation(alice_did.clone());
	// Trying to remove the currently active delegation key
	operation.public_keys_to_remove = vec![generate_key_id(&did::DidVerificationKey::from(del_key.public()).into())]
		.iter()
		.copied()
		.collect::<BTreeSet<TestKeyId>>();

	let signature = auth_key.sign(operation.encode().as_ref());

	let mut ext = ExtBuilder::default()
		.with_dids(vec![(alice_did, old_did_details)])
		.build(None);

	let new_block_number: TestBlockNumber = 1;

	ext.execute_with(|| {
		System::set_block_number(new_block_number);
		assert_noop!(
			Did::submit_did_update_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature),
			),
			did::Error::<Test>::CurrentlyActiveKey
		);
	});
}

#[test]
fn check_currently_active_attestation_key_update() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let att_key = get_sr25519_attestation_key(true);

	let mut old_did_details = generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));
	old_did_details.update_attestation_key(did::DidVerificationKey::from(att_key.public()), 0u64);

	// Remove both attestation and delegation key
	let mut operation = generate_base_did_update_operation(alice_did.clone());
	// Trying to remove the currently active attestation key
	operation.public_keys_to_remove = vec![generate_key_id(&did::DidVerificationKey::from(att_key.public()).into())]
		.iter()
		.copied()
		.collect::<BTreeSet<TestKeyId>>();

	let signature = auth_key.sign(operation.encode().as_ref());

	let mut ext = ExtBuilder::default()
		.with_dids(vec![(alice_did, old_did_details)])
		.build(None);

	let new_block_number: TestBlockNumber = 1;

	ext.execute_with(|| {
		System::set_block_number(new_block_number);
		assert_noop!(
			Did::submit_did_update_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature),
			),
			did::Error::<Test>::CurrentlyActiveKey
		);
	});
}

#[test]
fn check_verification_key_not_present_update() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let key_to_delete = get_sr25519_authentication_key(true);

	let old_did_details = generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));

	// Remove both attestation and delegation key
	let mut operation = generate_base_did_update_operation(alice_did.clone());
	// Trying to remove the currently active authentication key
	operation.public_keys_to_remove = vec![generate_key_id(
		&did::DidVerificationKey::from(key_to_delete.public()).into(),
	)]
	.iter()
	.copied()
	.collect::<BTreeSet<TestKeyId>>();

	let signature = auth_key.sign(operation.encode().as_ref());

	let mut ext = ExtBuilder::default()
		.with_dids(vec![(alice_did, old_did_details)])
		.build(None);

	let new_block_number: TestBlockNumber = 1;

	ext.execute_with(|| {
		System::set_block_number(new_block_number);
		assert_noop!(
			Did::submit_did_update_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature),
			),
			did::Error::<Test>::VerificationKeyNotPresent
		);
	});
}

// submit_did_delete_operation

#[test]
fn check_successful_deletion() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let did_details = generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));

	// Update all keys, URL endpoint and tx counter. No keys are removed in this
	// test
	let operation = generate_base_did_delete_operation(alice_did.clone());

	// Generate signature using the old authentication key
	let signature = auth_key.sign(operation.encode().as_ref());

	let mut ext = ExtBuilder::default()
		.with_dids(vec![(alice_did.clone(), did_details)])
		.build(None);

	ext.execute_with(|| {
		assert_ok!(Did::submit_did_delete_operation(
			Origin::signed(DEFAULT_ACCOUNT),
			operation.clone(),
			did::DidSignature::from(signature),
		));
	});

	assert_eq!(ext.execute_with(|| Did::get_did(alice_did.clone())), None);

	// Re-adding the same DID identifier, which should not fail.
	let operation = generate_base_did_creation_operation(alice_did);

	let signature = auth_key.sign(operation.encode().as_ref());

	let mut ext = ExtBuilder::default().build(None);

	ext.execute_with(|| {
		assert_ok!(Did::submit_did_create_operation(
			Origin::signed(DEFAULT_ACCOUNT),
			operation.clone(),
			did::DidSignature::from(signature),
		));
	});
}

#[test]
fn check_did_not_present_deletion() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());

	// Update all keys, URL endpoint and tx counter. No keys are removed in this
	// test
	let operation = generate_base_did_delete_operation(alice_did);

	// Generate signature using the old authentication key
	let signature = auth_key.sign(operation.encode().as_ref());

	let mut ext = ExtBuilder::default().build(None);

	ext.execute_with(|| {
		assert_noop!(
			Did::submit_did_delete_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature),
			),
			did::Error::<Test>::DidNotPresent
		);
	});
}

#[test]
fn check_max_tx_counter_did_deletion() {
	let auth_key = get_sr25519_authentication_key(true);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	let mut mock_did = generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));
	mock_did.last_tx_counter = u64::MAX;
	let operation = generate_base_did_delete_operation(alice_did.clone());

	let signature = auth_key.sign(operation.encode().as_ref());

	let mut ext = ExtBuilder::default().with_dids(vec![(alice_did, mock_did)]).build(None);

	ext.execute_with(|| {
		assert_noop!(
			Did::submit_did_delete_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature),
			),
			did::Error::<Test>::MaxTxCounterValue
		);
	});
}

#[test]
fn check_smaller_tx_counter_did_deletion() {
	let auth_key = get_sr25519_authentication_key(true);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	let mut mock_did = generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));
	mock_did.last_tx_counter = 1;
	let mut operation = generate_base_did_delete_operation(alice_did.clone());
	operation.tx_counter = mock_did.last_tx_counter - 1;

	let signature = auth_key.sign(operation.encode().as_ref());

	let mut ext = ExtBuilder::default().with_dids(vec![(alice_did, mock_did)]).build(None);

	ext.execute_with(|| {
		assert_noop!(
			Did::submit_did_delete_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature),
			),
			did::Error::<Test>::InvalidNonce
		);
	});
}

#[test]
fn check_equal_tx_counter_did_deletion() {
	let auth_key = get_sr25519_authentication_key(true);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	let mut mock_did = generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));
	mock_did.last_tx_counter = 1;
	let mut operation = generate_base_did_delete_operation(alice_did.clone());
	operation.tx_counter = mock_did.last_tx_counter;

	let signature = auth_key.sign(operation.encode().as_ref());

	let mut ext = ExtBuilder::default().with_dids(vec![(alice_did, mock_did)]).build(None);

	ext.execute_with(|| {
		assert_noop!(
			Did::submit_did_delete_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature),
			),
			did::Error::<Test>::InvalidNonce
		);
	});
}

#[test]
fn check_too_large_tx_counter_did_deletion() {
	let auth_key = get_sr25519_authentication_key(true);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	let mock_did = generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));
	let mut operation = generate_base_did_delete_operation(alice_did.clone());
	operation.tx_counter = mock_did.last_tx_counter + 2;

	let signature = auth_key.sign(operation.encode().as_ref());

	let mut ext = ExtBuilder::default().with_dids(vec![(alice_did, mock_did)]).build(None);

	ext.execute_with(|| {
		assert_noop!(
			Did::submit_did_delete_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature),
			),
			did::Error::<Test>::InvalidNonce
		);
	});
}

#[test]
fn check_invalid_signature_format_did_deletion() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	// Using an Sr25519 key where an Ed25519 is expected
	let invalid_key = get_sr25519_authentication_key(true);
	let mock_did = generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));
	let operation = generate_base_did_delete_operation(alice_did.clone());

	let signature = invalid_key.sign(operation.encode().as_ref());

	let mut ext = ExtBuilder::default().with_dids(vec![(alice_did, mock_did)]).build(None);

	ext.execute_with(|| {
		assert_noop!(
			Did::submit_did_delete_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature),
			),
			did::Error::<Test>::InvalidSignatureFormat
		);
	});
}

#[test]
fn check_invalid_signature_did_deletion() {
	let auth_key = get_sr25519_authentication_key(true);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	// Using an Sr25519 key as expected, but from a different seed (default = false)
	let alternative_key = get_sr25519_authentication_key(false);
	let mock_did = generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));
	let operation = generate_base_did_delete_operation(alice_did.clone());

	let signature = alternative_key.sign(operation.encode().as_ref());

	let mut ext = ExtBuilder::default().with_dids(vec![(alice_did, mock_did)]).build(None);

	ext.execute_with(|| {
		assert_noop!(
			Did::submit_did_delete_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature),
			),
			did::Error::<Test>::InvalidSignature
		);
	});
}

// submit_did_call

#[test]
fn check_no_key_error() {
	let auth_key = get_sr25519_authentication_key(true);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = DEFAULT_ACCOUNT;

	let mut ext = ExtBuilder::default().build(None);

	// CapabilityInvocation is not supported at the moment, so it should return no
	// key and hence the operation fail.
	let call_operation = generate_test_did_call(did::DidVerificationKeyRelationship::CapabilityInvocation, did);
	let signature = did::DidSignature::from(ed25519::Signature::default());

	ext.execute_with(|| {
		assert_noop!(
			Did::submit_did_call(Origin::signed(caller), Box::new(call_operation), signature),
			did::Error::<Test>::UnsupportedDidAuthorizationCall
		);
	});
}

#[test]
fn check_did_not_found_call_error() {
	let auth_key = get_sr25519_authentication_key(true);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = DEFAULT_ACCOUNT;

	// No DID added
	let mut ext = ExtBuilder::default().build(None);

	let call_operation = generate_test_did_call(did::DidVerificationKeyRelationship::Authentication, did);
	let signature = auth_key.sign(call_operation.encode().as_ref());

	ext.execute_with(|| {
		assert_noop!(
			Did::submit_did_call(
				Origin::signed(caller),
				Box::new(call_operation),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::DidNotPresent
		);
	});
}

#[test]
fn check_max_counter_call_error() {
	let auth_key = get_sr25519_authentication_key(true);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = DEFAULT_ACCOUNT;
	let mut mock_did = generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));
	mock_did.last_tx_counter = u64::MAX;

	let mut ext = ExtBuilder::default()
		.with_dids(vec![(did.clone(), mock_did)])
		.build(None);

	let call_operation = generate_test_did_call(did::DidVerificationKeyRelationship::Authentication, did);
	let signature = auth_key.sign(call_operation.encode().as_ref());

	ext.execute_with(|| {
		assert_noop!(
			Did::submit_did_call(
				Origin::signed(caller),
				Box::new(call_operation),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::MaxTxCounterValue
		);
	});
}

#[test]
fn check_too_small_tx_counter_call_error() {
	let auth_key = get_sr25519_authentication_key(true);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = DEFAULT_ACCOUNT;
	let mut mock_did = generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));
	mock_did.last_tx_counter = 1u64;

	let mut ext = ExtBuilder::default()
		.with_dids(vec![(did.clone(), mock_did)])
		.build(None);

	let mut call_operation = generate_test_did_call(did::DidVerificationKeyRelationship::Authentication, did);
	call_operation.tx_counter = 0u64;
	let signature = auth_key.sign(call_operation.encode().as_ref());

	ext.execute_with(|| {
		assert_noop!(
			Did::submit_did_call(
				Origin::signed(caller),
				Box::new(call_operation),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidNonce
		);
	});
}

#[test]
fn check_equal_tx_counter_call_error() {
	let auth_key = get_sr25519_authentication_key(true);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = DEFAULT_ACCOUNT;
	let mock_did = generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));

	let mut ext = ExtBuilder::default()
		.with_dids(vec![(did.clone(), mock_did.clone())])
		.build(None);

	let mut call_operation = generate_test_did_call(did::DidVerificationKeyRelationship::Authentication, did);
	call_operation.tx_counter = mock_did.last_tx_counter;
	let signature = auth_key.sign(call_operation.encode().as_ref());

	ext.execute_with(|| {
		assert_noop!(
			Did::submit_did_call(
				Origin::signed(caller),
				Box::new(call_operation),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidNonce
		);
	});
}

#[test]
fn check_too_large_tx_counter_call_error() {
	let auth_key = get_sr25519_authentication_key(true);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = DEFAULT_ACCOUNT;
	let mock_did = generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));

	let mut ext = ExtBuilder::default()
		.with_dids(vec![(did.clone(), mock_did.clone())])
		.build(None);

	let mut call_operation = generate_test_did_call(did::DidVerificationKeyRelationship::Authentication, did);
	call_operation.tx_counter = mock_did.last_tx_counter + 2u64;
	let signature = auth_key.sign(call_operation.encode().as_ref());

	ext.execute_with(|| {
		assert_noop!(
			Did::submit_did_call(
				Origin::signed(caller),
				Box::new(call_operation),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidNonce
		);
	});
}

#[test]
fn check_verification_key_not_present_call_error() {
	let auth_key = get_sr25519_authentication_key(true);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = DEFAULT_ACCOUNT;
	let mock_did = generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));

	let mut ext = ExtBuilder::default()
		.with_dids(vec![(did.clone(), mock_did)])
		.build(None);

	// The operation requires the delegation key that is currently not stored for
	// the given DID.
	let call_operation = generate_test_did_call(did::DidVerificationKeyRelationship::CapabilityDelegation, did);
	let signature = auth_key.sign(call_operation.encode().as_ref());

	ext.execute_with(|| {
		assert_noop!(
			Did::submit_did_call(
				Origin::signed(caller),
				Box::new(call_operation),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::VerificationKeyNotPresent
		);
	});
}

#[test]
fn check_invalid_signature_format_call_error() {
	let auth_key = get_sr25519_authentication_key(true);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = DEFAULT_ACCOUNT;
	let alternative_auth_key = get_ed25519_authentication_key(true);
	let mock_did = generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));

	let mut ext = ExtBuilder::default()
		.with_dids(vec![(did.clone(), mock_did)])
		.build(None);

	let call_operation = generate_test_did_call(did::DidVerificationKeyRelationship::Authentication, did);
	let signature = alternative_auth_key.sign(call_operation.encode().as_ref());

	ext.execute_with(|| {
		assert_noop!(
			Did::submit_did_call(
				Origin::signed(caller),
				Box::new(call_operation),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidSignatureFormat
		);
	});
}

#[test]
fn check_invalid_signature_call_error() {
	let auth_key = get_sr25519_authentication_key(true);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = DEFAULT_ACCOUNT;
	let alternative_auth_key = get_sr25519_authentication_key(false);
	let mock_did = generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));

	let mut ext = ExtBuilder::default()
		.with_dids(vec![(did.clone(), mock_did)])
		.build(None);

	let call_operation = generate_test_did_call(did::DidVerificationKeyRelationship::Authentication, did);
	let signature = alternative_auth_key.sign(call_operation.encode().as_ref());

	ext.execute_with(|| {
		assert_noop!(
			Did::submit_did_call(
				Origin::signed(caller),
				Box::new(call_operation),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidSignature
		);
	});
}

#[test]
fn check_call_attestation_key_successful() {
	let auth_key = get_sr25519_authentication_key(true);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = DEFAULT_ACCOUNT;
	let attestation_key = get_ed25519_attestation_key(true);

	let mut mock_did = generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));
	mock_did.update_attestation_key(did::DidVerificationKey::from(attestation_key.public()), 0);

	let mut ext = ExtBuilder::default()
		.with_dids(vec![(did.clone(), mock_did)])
		.build(None);

	let call_operation = generate_test_did_call(did::DidVerificationKeyRelationship::AssertionMethod, did);
	let signature = attestation_key.sign(call_operation.encode().as_ref());

	ext.execute_with(|| {
		assert_ok!(Did::submit_did_call(
			Origin::signed(caller),
			Box::new(call_operation),
			did::DidSignature::from(signature)
		));
	});
}

#[test]
fn check_call_attestation_key_error() {
	let auth_key = get_sr25519_authentication_key(true);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = DEFAULT_ACCOUNT;
	let attestation_key = get_ed25519_attestation_key(true);

	let mut mock_did = generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));
	mock_did.update_attestation_key(did::DidVerificationKey::from(attestation_key.public()), 0);

	let ext = ExtBuilder::default()
		.with_dids(vec![(did.clone(), mock_did)])
		.build(None);
	// CTYPE already added to storage
	let mut ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(get_attestation_key_test_input(), did.clone())])
		.build(Some(ext));

	let call_operation = generate_test_did_call(did::DidVerificationKeyRelationship::AssertionMethod, did);
	let signature = attestation_key.sign(call_operation.encode().as_ref());

	ext.execute_with(|| {
		assert_err!(
			Did::submit_did_call(
				Origin::signed(caller),
				Box::new(call_operation),
				did::DidSignature::from(signature)
			),
			ctype::Error::<Test>::CTypeAlreadyExists
		);
	});
}

#[test]
fn check_call_delegation_key_successful() {
	let auth_key = get_sr25519_authentication_key(true);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = DEFAULT_ACCOUNT;
	let delegation_key = get_ed25519_delegation_key(true);

	let mut mock_did = generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));
	mock_did.update_delegation_key(did::DidVerificationKey::from(delegation_key.public()), 0);

	let mut ext = ExtBuilder::default()
		.with_dids(vec![(did.clone(), mock_did)])
		.build(None);

	let call_operation = generate_test_did_call(did::DidVerificationKeyRelationship::CapabilityDelegation, did);
	let signature = delegation_key.sign(call_operation.encode().as_ref());

	ext.execute_with(|| {
		assert_ok!(Did::submit_did_call(
			Origin::signed(caller),
			Box::new(call_operation),
			did::DidSignature::from(signature)
		));
	});
}

#[test]
fn check_call_delegation_key_error() {
	let auth_key = get_sr25519_authentication_key(true);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = DEFAULT_ACCOUNT;
	let delegation_key = get_ed25519_delegation_key(true);

	let mut mock_did = generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));
	mock_did.update_delegation_key(did::DidVerificationKey::from(delegation_key.public()), 0);

	let ext = ExtBuilder::default()
		.with_dids(vec![(did.clone(), mock_did)])
		.build(None);
	// CTYPE already added to storage
	let mut ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(get_delegation_key_test_input(), did.clone())])
		.build(Some(ext));

	let call_operation = generate_test_did_call(did::DidVerificationKeyRelationship::CapabilityDelegation, did);
	let signature = delegation_key.sign(call_operation.encode().as_ref());

	ext.execute_with(|| {
		assert_err!(
			Did::submit_did_call(
				Origin::signed(caller),
				Box::new(call_operation),
				did::DidSignature::from(signature)
			),
			ctype::Error::<Test>::CTypeAlreadyExists
		);
	});
}

#[test]
fn check_call_authentication_key_successful() {
	let auth_key = get_sr25519_authentication_key(true);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = DEFAULT_ACCOUNT;

	let mock_did = generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));

	let mut ext = ExtBuilder::default()
		.with_dids(vec![(did.clone(), mock_did)])
		.build(None);

	let call_operation = generate_test_did_call(did::DidVerificationKeyRelationship::Authentication, did);
	let signature = auth_key.sign(call_operation.encode().as_ref());

	ext.execute_with(|| {
		assert_ok!(Did::submit_did_call(
			Origin::signed(caller),
			Box::new(call_operation),
			did::DidSignature::from(signature)
		));
	});
}

#[test]
fn check_call_authentication_key_error() {
	let auth_key = get_sr25519_authentication_key(true);
	let did = get_did_identifier_from_sr25519_key(auth_key.public());
	let caller = DEFAULT_ACCOUNT;

	let mock_did = generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));

	let ext = ExtBuilder::default()
		.with_dids(vec![(did.clone(), mock_did)])
		.build(None);
	// CTYPE already added to storage
	let mut ext = ctype_mock::ExtBuilder::default()
		.with_ctypes(vec![(get_authentication_key_test_input(), did.clone())])
		.build(Some(ext));

	let call_operation = generate_test_did_call(did::DidVerificationKeyRelationship::Authentication, did);
	let signature = auth_key.sign(call_operation.encode().as_ref());

	ext.execute_with(|| {
		assert_err!(
			Did::submit_did_call(
				Origin::signed(caller),
				Box::new(call_operation),
				did::DidSignature::from(signature)
			),
			ctype::Error::<Test>::CTypeAlreadyExists
		);
	});
}

// Internal function: verify_operation_validity_and_increase_did_nonce

#[test]
fn check_authentication_successful_operation_verification() {
	let auth_key = get_sr25519_authentication_key(true);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	let mock_did = generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));
	let operation = TestDidOperation {
		did: alice_did.clone(),
		verification_key_type: did::DidVerificationKeyRelationship::Authentication,
		tx_counter: mock_did.last_tx_counter + 1,
	};

	let signature = auth_key.sign(&operation.encode());

	let mut ext = ExtBuilder::default()
		.with_dids(vec![(alice_did, mock_did.clone())])
		.build(None);

	ext.execute_with(|| {
		assert_ok!(
			Did::verify_operation_validity_and_increase_did_nonce::<TestDidOperation>(
				&operation,
				&did::DidSignature::from(signature)
			)
		);
	});

	// Verify that the DID tx counter has increased
	let did_details = ext.execute_with(|| Did::get_did(&operation.did).expect("DID should be present on chain."));
	assert_eq!(
		did_details.get_tx_counter_value(),
		mock_did.get_tx_counter_value() + 1u64
	);
}

#[test]
fn check_attestation_successful_operation_verification() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let att_key = get_ecdsa_attestation_key(true);
	let mut mock_did = generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));
	mock_did.update_attestation_key(did::DidVerificationKey::from(att_key.public()), 0u64);
	let operation = TestDidOperation {
		did: alice_did.clone(),
		verification_key_type: did::DidVerificationKeyRelationship::AssertionMethod,
		tx_counter: mock_did.last_tx_counter + 1,
	};

	let signature = att_key.sign(&operation.encode());

	let mut ext = ExtBuilder::default()
		.with_dids(vec![(alice_did, mock_did.clone())])
		.build(None);

	ext.execute_with(|| {
		assert_ok!(
			Did::verify_operation_validity_and_increase_did_nonce::<TestDidOperation>(
				&operation,
				&did::DidSignature::from(signature)
			)
		);
	});

	// Verify that the DID tx counter has increased
	let did_details = ext.execute_with(|| Did::get_did(&operation.did).expect("DID should be present on chain."));
	assert_eq!(
		did_details.get_tx_counter_value(),
		mock_did.get_tx_counter_value() + 1u64
	);
}

#[test]
fn check_delegation_successful_operation_verification() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let del_key = get_ed25519_delegation_key(true);
	let mut mock_did = generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));
	mock_did.update_delegation_key(did::DidVerificationKey::from(del_key.public()), 0u64);
	let operation = TestDidOperation {
		did: alice_did.clone(),
		verification_key_type: did::DidVerificationKeyRelationship::CapabilityDelegation,
		tx_counter: mock_did.last_tx_counter + 1,
	};
	let signature = del_key.sign(&operation.encode());

	let mut ext = ExtBuilder::default()
		.with_dids(vec![(alice_did, mock_did.clone())])
		.build(None);

	ext.execute_with(|| {
		assert_ok!(
			Did::verify_operation_validity_and_increase_did_nonce::<TestDidOperation>(
				&operation,
				&did::DidSignature::from(signature)
			)
		);
	});

	// Verify that the DID tx counter has increased
	let did_details = ext.execute_with(|| Did::get_did(&operation.did).expect("DID should be present on chain."));
	assert_eq!(
		did_details.get_tx_counter_value(),
		mock_did.get_tx_counter_value() + 1u64
	);
}

#[test]
fn check_did_not_present_operation_verification() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let del_key = get_ed25519_delegation_key(true);
	let mock_did = generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));
	let operation = TestDidOperation {
		did: alice_did,
		verification_key_type: did::DidVerificationKeyRelationship::CapabilityDelegation,
		tx_counter: mock_did.last_tx_counter + 1,
	};
	let signature = del_key.sign(&operation.encode());

	let mut ext = ExtBuilder::default().build(None);

	ext.execute_with(|| {
		assert_noop!(
			Did::verify_operation_validity_and_increase_did_nonce::<TestDidOperation>(
				&operation,
				&did::DidSignature::from(signature)
			),
			did::DidError::StorageError(did::StorageError::DidNotPresent)
		);
	});
}

#[test]
fn check_max_tx_counter_operation_verification() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let mut mock_did = generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));
	mock_did.last_tx_counter = u64::MAX;
	let operation = TestDidOperation {
		did: alice_did.clone(),
		verification_key_type: did::DidVerificationKeyRelationship::Authentication,
		tx_counter: mock_did.last_tx_counter,
	};
	let signature = auth_key.sign(&operation.encode());

	let mut ext = ExtBuilder::default()
		.with_dids(vec![(alice_did, mock_did.clone())])
		.build(None);

	ext.execute_with(|| {
		assert_noop!(
			Did::verify_operation_validity_and_increase_did_nonce::<TestDidOperation>(
				&operation,
				&did::DidSignature::from(signature)
			),
			did::DidError::StorageError(did::StorageError::MaxTxCounterValue)
		);
	});

	// Verify that the DID tx counter has not increased
	let did_details = ext.execute_with(|| Did::get_did(&operation.did).expect("DID should be present on chain."));
	assert_eq!(did_details.get_tx_counter_value(), mock_did.get_tx_counter_value());
}

#[test]
fn check_smaller_counter_operation_verification() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let mut mock_did = generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));
	mock_did.last_tx_counter = 1;
	let operation = TestDidOperation {
		did: alice_did.clone(),
		verification_key_type: did::DidVerificationKeyRelationship::Authentication,
		tx_counter: mock_did.last_tx_counter - 1,
	};
	let signature = auth_key.sign(&operation.encode());

	let mut ext = ExtBuilder::default()
		.with_dids(vec![(alice_did, mock_did.clone())])
		.build(None);

	ext.execute_with(|| {
		assert_noop!(
			Did::verify_operation_validity_and_increase_did_nonce::<TestDidOperation>(
				&operation,
				&did::DidSignature::from(signature)
			),
			did::DidError::SignatureError(did::SignatureError::InvalidNonce)
		);
	});

	// Verify that the DID tx counter has not increased
	let did_details = ext.execute_with(|| Did::get_did(&operation.did).expect("DID should be present on chain."));
	assert_eq!(did_details.get_tx_counter_value(), mock_did.get_tx_counter_value());
}

#[test]
fn check_equal_counter_operation_verification() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let mock_did = generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));
	let operation = TestDidOperation {
		did: alice_did.clone(),
		verification_key_type: did::DidVerificationKeyRelationship::Authentication,
		tx_counter: mock_did.last_tx_counter,
	};
	let signature = auth_key.sign(&operation.encode());

	let mut ext = ExtBuilder::default()
		.with_dids(vec![(alice_did, mock_did.clone())])
		.build(None);

	ext.execute_with(|| {
		assert_noop!(
			Did::verify_operation_validity_and_increase_did_nonce::<TestDidOperation>(
				&operation,
				&did::DidSignature::from(signature)
			),
			did::DidError::SignatureError(did::SignatureError::InvalidNonce)
		);
	});

	// Verify that the DID tx counter has not increased
	let did_details = ext.execute_with(|| Did::get_did(&operation.did).expect("DID should be present on chain."));
	assert_eq!(did_details.get_tx_counter_value(), mock_did.get_tx_counter_value());
}

#[test]
fn check_too_large_counter_operation_verification() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let mock_did = generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));
	let operation = TestDidOperation {
		did: alice_did.clone(),
		verification_key_type: did::DidVerificationKeyRelationship::Authentication,
		tx_counter: mock_did.last_tx_counter + 2,
	};
	let signature = auth_key.sign(&operation.encode());

	let mut ext = ExtBuilder::default()
		.with_dids(vec![(alice_did, mock_did.clone())])
		.build(None);

	ext.execute_with(|| {
		assert_noop!(
			Did::verify_operation_validity_and_increase_did_nonce::<TestDidOperation>(
				&operation,
				&did::DidSignature::from(signature)
			),
			did::DidError::SignatureError(did::SignatureError::InvalidNonce)
		);
	});

	// Verify that the DID tx counter has not increased
	let did_details = ext.execute_with(|| Did::get_did(&operation.did).expect("DID should be present on chain."));
	assert_eq!(did_details.get_tx_counter_value(), mock_did.get_tx_counter_value());
}

#[test]
fn check_verification_key_not_present_operation_verification() {
	let auth_key = get_ed25519_authentication_key(true);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let mock_did = generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));
	let verification_key_required = did::DidVerificationKeyRelationship::CapabilityInvocation;
	let operation = TestDidOperation {
		did: alice_did.clone(),
		verification_key_type: verification_key_required,
		tx_counter: mock_did.last_tx_counter + 1,
	};

	let signature = auth_key.sign(&operation.encode());

	let mut ext = ExtBuilder::default()
		.with_dids(vec![(alice_did, mock_did.clone())])
		.build(None);

	ext.execute_with(|| {
		assert_noop!(
			Did::verify_operation_validity_and_increase_did_nonce::<TestDidOperation>(
				&operation,
				&did::DidSignature::from(signature)
			),
			did::DidError::StorageError(did::StorageError::DidKeyNotPresent(verification_key_required))
		);
	});

	// Verify that the DID tx counter has not increased
	let did_details = ext.execute_with(|| Did::get_did(&operation.did).expect("DID should be present on chain."));
	assert_eq!(did_details.get_tx_counter_value(), mock_did.get_tx_counter_value());
}

#[test]
fn check_invalid_signature_format_operation_verification() {
	let auth_key = get_sr25519_authentication_key(true);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	// Expected an Sr25519, given an Ed25519
	let invalid_key = get_ed25519_authentication_key(true);
	let mock_did = generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));
	let operation = TestDidOperation {
		did: alice_did.clone(),
		verification_key_type: did::DidVerificationKeyRelationship::Authentication,
		tx_counter: mock_did.last_tx_counter + 1,
	};

	let signature = invalid_key.sign(&operation.encode());

	let mut ext = ExtBuilder::default()
		.with_dids(vec![(alice_did, mock_did.clone())])
		.build(None);

	ext.execute_with(|| {
		assert_noop!(
			Did::verify_operation_validity_and_increase_did_nonce::<TestDidOperation>(
				&operation,
				&did::DidSignature::from(signature)
			),
			did::DidError::SignatureError(did::SignatureError::InvalidSignatureFormat)
		);
	});

	// Verify that the DID tx counter has not increased
	let did_details = ext.execute_with(|| Did::get_did(&operation.did).expect("DID should be present on chain."));
	assert_eq!(did_details.get_tx_counter_value(), mock_did.get_tx_counter_value());
}

#[test]
fn check_invalid_signature_operation_verification() {
	let auth_key = get_sr25519_authentication_key(true);
	let alice_did = get_did_identifier_from_sr25519_key(auth_key.public());
	// Using same key type but different seed (default = false)
	let alternative_key = get_sr25519_authentication_key(false);
	let mock_did = generate_base_did_details(did::DidVerificationKey::from(auth_key.public()));
	let operation = TestDidOperation {
		did: alice_did.clone(),
		verification_key_type: did::DidVerificationKeyRelationship::Authentication,
		tx_counter: mock_did.last_tx_counter + 1,
	};

	let signature = alternative_key.sign(&operation.encode());

	let mut ext = ExtBuilder::default()
		.with_dids(vec![(alice_did, mock_did.clone())])
		.build(None);

	ext.execute_with(|| {
		assert_noop!(
			Did::verify_operation_validity_and_increase_did_nonce::<TestDidOperation>(
				&operation,
				&did::DidSignature::from(signature)
			),
			did::DidError::SignatureError(did::SignatureError::InvalidSignature)
		);
	});

	// Verify that the DID tx counter has not increased
	let did_details = ext.execute_with(|| Did::get_did(&operation.did).expect("DID should be present on chain."));
	assert_eq!(did_details.get_tx_counter_value(), mock_did.get_tx_counter_value());
}

// Internal function: HttpUrl try_from

#[test]
fn check_http_url() {
	assert_ok!(did::HttpUrl::try_from("http://kilt.io".as_bytes()));

	assert_ok!(did::HttpUrl::try_from("https://kilt.io".as_bytes()));

	assert_ok!(did::HttpUrl::try_from(
		"https://super.long.domain.kilt.io:12345/public/files/test.txt".as_bytes()
	));

	// All other valid ASCII characters
	assert_ok!(did::HttpUrl::try_from("http://:/?#[]@!$&'()*+,;=-._~".as_bytes()));

	assert_eq!(
		did::HttpUrl::try_from("".as_bytes()),
		Err(did::UrlError::InvalidUrlScheme)
	);

	// Non-printable ASCII characters
	assert_eq!(
		did::HttpUrl::try_from("http://kilt.io/\x00".as_bytes()),
		Err(did::UrlError::InvalidUrlEncoding)
	);

	// Some invalid ASCII characters
	assert_eq!(
		did::HttpUrl::try_from("http://kilt.io/<tag>".as_bytes()),
		Err(did::UrlError::InvalidUrlEncoding)
	);

	// Non-ASCII characters
	assert_eq!(
		did::HttpUrl::try_from("http://¶.com".as_bytes()),
		Err(did::UrlError::InvalidUrlEncoding)
	);

	assert_eq!(
		did::HttpUrl::try_from("htt://kilt.io".as_bytes()),
		Err(did::UrlError::InvalidUrlScheme)
	);

	assert_eq!(
		did::HttpUrl::try_from("httpss://kilt.io".as_bytes()),
		Err(did::UrlError::InvalidUrlScheme)
	);
}

// Internal function: FtpUrl try_from

#[test]
fn check_ftp_url() {
	assert_ok!(did::FtpUrl::try_from("ftp://kilt.io".as_bytes()));

	assert_ok!(did::FtpUrl::try_from("ftps://kilt.io".as_bytes()));

	assert_ok!(did::FtpUrl::try_from(
		"ftps://user@super.long.domain.kilt.io:12345/public/files/test.txt".as_bytes()
	));

	// All other valid ASCII characters
	assert_ok!(did::FtpUrl::try_from("ftps://:/?#[]@%!$&'()*+,;=-._~".as_bytes()));

	assert_eq!(
		did::FtpUrl::try_from("".as_bytes()),
		Err(did::UrlError::InvalidUrlScheme)
	);

	// Non-printable ASCII characters
	assert_eq!(
		did::HttpUrl::try_from("http://kilt.io/\x00".as_bytes()),
		Err(did::UrlError::InvalidUrlEncoding)
	);

	// Some invalid ASCII characters
	assert_eq!(
		did::FtpUrl::try_from("ftp://kilt.io/<tag>".as_bytes()),
		Err(did::UrlError::InvalidUrlEncoding)
	);

	// Non-ASCII characters
	assert_eq!(
		did::FtpUrl::try_from("ftps://¶.com".as_bytes()),
		Err(did::UrlError::InvalidUrlEncoding)
	);

	assert_eq!(
		did::FtpUrl::try_from("ft://kilt.io".as_bytes()),
		Err(did::UrlError::InvalidUrlScheme)
	);

	assert_eq!(
		did::HttpUrl::try_from("ftpss://kilt.io".as_bytes()),
		Err(did::UrlError::InvalidUrlScheme)
	);
}

// Internal function: IpfsUrl try_from

#[test]
fn check_ipfs_url() {
	// Base58 address
	assert_ok!(did::IpfsUrl::try_from(
		"ipfs://QmdQ1rHHHTbgbGorfuMMYDQQ36q4sxvYcB4GDEHREuJQkL".as_bytes()
	));

	// Base32 address (at the moment, padding characters can appear anywhere in the
	// string)
	assert_ok!(did::IpfsUrl::try_from(
		"ipfs://OQQHHHTGMMYDQQ364YB4GDE=HREJQL==".as_bytes()
	));

	assert_eq!(
		did::IpfsUrl::try_from("".as_bytes()),
		Err(did::UrlError::InvalidUrlScheme)
	);

	assert_eq!(
		did::IpfsUrl::try_from(
			"ipfs://¶
QmdQ1rHHHTbgbGorfuMMYDQQ36q4sxvYcB4GDEHREuJQkL"
				.as_bytes()
		),
		Err(did::UrlError::InvalidUrlEncoding)
	);

	assert_eq!(
		did::IpfsUrl::try_from("ipf://QmdQ1rHHHTbgbGorfuMMYDQQ36q4sxvYcB4GDEHREuJQkL".as_bytes()),
		Err(did::UrlError::InvalidUrlScheme)
	);

	assert_eq!(
		did::IpfsUrl::try_from(
			"ipfss://
QmdQ1rHHHTbgbGorfuMMYDQQ36q4sxvYcB4GDEHREuJQkL"
				.as_bytes()
		),
		Err(did::UrlError::InvalidUrlScheme)
	);
}
