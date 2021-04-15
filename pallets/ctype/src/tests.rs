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

use crate::{self as ctype, mock::*};
use did::mock as did_mock;
use did_mock::{ALICE_DID, BOB_DID};
use frame_support::{assert_ok, assert_noop};
use sp_core::{Pair, H256};

use codec::Encode;

#[test]
fn check_successful_ctype_creation() {
	let did_auth_key = did_mock::get_ed25519_authentication_key(true);
	let did_enc_key = did_mock::get_x25519_encryption_key(true);
	let did_att_key = did_mock::get_sr25519_attestation_key(true);
	let mut mock_did_details = did_mock::generate_mock_did_details(did::PublicVerificationKey::from(did_auth_key.public()), did_enc_key);
	mock_did_details.attestation_key = Some(did::PublicVerificationKey::from(did_att_key.public()));

	let ctype_creation_operation = ctype::CtypeCreationOperation {
		creator_did: did_mock::ALICE_DID,
		hash: H256::from_low_u64_be(1),
		tx_counter: mock_did_details.get_tx_counter_value() + 1u64
	};

	let signature = did_att_key.sign(&ctype_creation_operation.encode());

	let did_builder = did_mock::ExtBuilder::default().with_dids(vec![(ALICE_DID, mock_did_details.clone())]);
	let ctype_builder = ExtBuilder::from(did_builder);

	let mut ext = ctype_builder.build();

	// Write CTYPE on chain
	ext.execute_with(|| {
		assert_ok!(Ctype::submit_ctype_creation_operation(Origin::signed(DEFAULT_ACCOUNT), ctype_creation_operation.clone(), did::DidSignature::from(signature)));
	});

	// Verify the CTYPE has the right owner DID
	let stored_ctype_creator = ext.execute_with(|| {
		Ctype::ctypes(&ctype_creation_operation.hash).expect("CTYPE hash should be present on chain.")
	});
	assert_eq!(stored_ctype_creator, ctype_creation_operation.creator_did);

	// Verify that the DID tx counter has increased
	let ctype_creator_details = ext.execute_with(|| {
		Did::get_did(&ctype_creation_operation.creator_did).expect("CTYPE creator should be present on chain.")
	});
	assert_eq!(ctype_creator_details.get_tx_counter_value(), mock_did_details.get_tx_counter_value() + 1u64);
}

#[test]
fn check_duplicate_ctype_creation() {
	let did_auth_key = did_mock::get_ed25519_authentication_key(true);
	let did_enc_key = did_mock::get_x25519_encryption_key(true);
	let did_att_key = did_mock::get_sr25519_attestation_key(true);
	let mut mock_did_details = did_mock::generate_mock_did_details(did::PublicVerificationKey::from(did_auth_key.public()), did_enc_key);
	mock_did_details.attestation_key = Some(did::PublicVerificationKey::from(did_att_key.public()));

	let ctype_creation_operation = ctype::CtypeCreationOperation {
		creator_did: did_mock::ALICE_DID,
		hash: H256::from_low_u64_be(1),
		tx_counter: mock_did_details.get_tx_counter_value() + 1u64
	};

	let signature = did_att_key.sign(&ctype_creation_operation.encode());

	let did_builder = did_mock::ExtBuilder::default().with_dids(vec![(ALICE_DID, mock_did_details.clone())]);
	let ctype_builder = ExtBuilder::from(did_builder).with_ctypes(vec![(ctype_creation_operation.hash, ALICE_DID)]);

	let mut ext = ctype_builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Ctype::submit_ctype_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				ctype_creation_operation.clone(),
				did::DidSignature::from(signature)
			),
			ctype::Error::<Test>::AlreadyExists
		);
	});

	// Verify that the DID tx counter has NOT increased
	let ctype_creator_details = ext.execute_with(|| {
		Did::get_did(&ctype_creation_operation.creator_did).expect("CTYPE creator should be present on chain.")
	});
	assert_eq!(ctype_creator_details.get_tx_counter_value(), mock_did_details.get_tx_counter_value());
}

#[test]
fn check_did_not_present_ctype_creation() {
	let did_auth_key = did_mock::get_ed25519_authentication_key(true);
	let did_enc_key = did_mock::get_x25519_encryption_key(true);
	let did_att_key = did_mock::get_sr25519_attestation_key(true);
	let mut mock_did_details = did_mock::generate_mock_did_details(did::PublicVerificationKey::from(did_auth_key.public()), did_enc_key);
	mock_did_details.attestation_key = Some(did::PublicVerificationKey::from(did_att_key.public()));

	let ctype_creation_operation = ctype::CtypeCreationOperation {
		creator_did: did_mock::ALICE_DID,
		hash: H256::from_low_u64_be(1),
		tx_counter: mock_did_details.get_tx_counter_value() + 1u64
	};

	let signature = did_att_key.sign(&ctype_creation_operation.encode());

	let did_builder = did_mock::ExtBuilder::default().with_dids(vec![(BOB_DID, mock_did_details)]);
	let ctype_builder = ExtBuilder::from(did_builder).with_ctypes(vec![(ctype_creation_operation.hash, ALICE_DID)]);

	let mut ext = ctype_builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Ctype::submit_ctype_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				ctype_creation_operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::DidNotPresent
		);
	});
}

#[test]
fn check_max_did_counter_ctype_creation() {
	let did_auth_key = did_mock::get_ed25519_authentication_key(true);
	let did_enc_key = did_mock::get_x25519_encryption_key(true);
	let did_att_key = did_mock::get_sr25519_attestation_key(true);
	let mut mock_did_details = did_mock::generate_mock_did_details(did::PublicVerificationKey::from(did_auth_key.public()), did_enc_key);
	mock_did_details.attestation_key = Some(did::PublicVerificationKey::from(did_att_key.public()));
	mock_did_details.set_tx_counter(u64::MAX);

	let ctype_creation_operation = ctype::CtypeCreationOperation {
		creator_did: did_mock::ALICE_DID,
		hash: H256::from_low_u64_be(1),
		tx_counter: mock_did_details.get_tx_counter_value()
	};

	let signature = did_att_key.sign(&ctype_creation_operation.encode());

	let did_builder = did_mock::ExtBuilder::default().with_dids(vec![(ALICE_DID, mock_did_details)]);
	let ctype_builder = ExtBuilder::from(did_builder);

	let mut ext = ctype_builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Ctype::submit_ctype_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				ctype_creation_operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::MaxTxCounterValue
		);
	});
}

#[test]
fn check_smaller_did_counter_ctype_creation() {
	let did_auth_key = did_mock::get_ed25519_authentication_key(true);
	let did_enc_key = did_mock::get_x25519_encryption_key(true);
	let did_att_key = did_mock::get_sr25519_attestation_key(true);
	let mut mock_did_details = did_mock::generate_mock_did_details(did::PublicVerificationKey::from(did_auth_key.public()), did_enc_key);
	mock_did_details.attestation_key = Some(did::PublicVerificationKey::from(did_att_key.public()));
	mock_did_details.set_tx_counter(1u64);

	let ctype_creation_operation = ctype::CtypeCreationOperation {
		creator_did: did_mock::ALICE_DID,
		hash: H256::from_low_u64_be(1),
		tx_counter: mock_did_details.get_tx_counter_value() - 1u64
	};

	let signature = did_att_key.sign(&ctype_creation_operation.encode());

	let did_builder = did_mock::ExtBuilder::default().with_dids(vec![(ALICE_DID, mock_did_details)]);
	let ctype_builder = ExtBuilder::from(did_builder);

	let mut ext = ctype_builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Ctype::submit_ctype_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				ctype_creation_operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidNonce
		);
	});
}

#[test]
fn check_equal_did_counter_ctype_creation() {
	let did_auth_key = did_mock::get_ed25519_authentication_key(true);
	let did_enc_key = did_mock::get_x25519_encryption_key(true);
	let did_att_key = did_mock::get_sr25519_attestation_key(true);
	let mut mock_did_details = did_mock::generate_mock_did_details(did::PublicVerificationKey::from(did_auth_key.public()), did_enc_key);
	mock_did_details.attestation_key = Some(did::PublicVerificationKey::from(did_att_key.public()));

	let ctype_creation_operation = ctype::CtypeCreationOperation {
		creator_did: did_mock::ALICE_DID,
		hash: H256::from_low_u64_be(1),
		tx_counter: mock_did_details.get_tx_counter_value()
	};

	let signature = did_att_key.sign(&ctype_creation_operation.encode());

	let did_builder = did_mock::ExtBuilder::default().with_dids(vec![(ALICE_DID, mock_did_details)]);
	let ctype_builder = ExtBuilder::from(did_builder);

	let mut ext = ctype_builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Ctype::submit_ctype_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				ctype_creation_operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidNonce
		);
	});
}

#[test]
fn check_too_large_did_counter_ctype_creation() {
	let did_auth_key = did_mock::get_ed25519_authentication_key(true);
	let did_enc_key = did_mock::get_x25519_encryption_key(true);
	let did_att_key = did_mock::get_sr25519_attestation_key(true);
	let mut mock_did_details = did_mock::generate_mock_did_details(did::PublicVerificationKey::from(did_auth_key.public()), did_enc_key);
	mock_did_details.attestation_key = Some(did::PublicVerificationKey::from(did_att_key.public()));

	let ctype_creation_operation = ctype::CtypeCreationOperation {
		creator_did: did_mock::ALICE_DID,
		hash: H256::from_low_u64_be(1),
		tx_counter: mock_did_details.get_tx_counter_value() + 2u64
	};

	let signature = did_att_key.sign(&ctype_creation_operation.encode());

	let did_builder = did_mock::ExtBuilder::default().with_dids(vec![(ALICE_DID, mock_did_details)]);
	let ctype_builder = ExtBuilder::from(did_builder);

	let mut ext = ctype_builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Ctype::submit_ctype_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				ctype_creation_operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidNonce
		);
	});
}

#[test]
fn check_no_attestation_key_ctype_creation() {
	let did_auth_key = did_mock::get_ed25519_authentication_key(true);
	let did_enc_key = did_mock::get_x25519_encryption_key(true);
	// Created but not added to the mock DID details
	let did_att_key = did_mock::get_sr25519_attestation_key(true);
	let mock_did_details = did_mock::generate_mock_did_details(did::PublicVerificationKey::from(did_auth_key.public()), did_enc_key);

	let ctype_creation_operation = ctype::CtypeCreationOperation {
		creator_did: did_mock::ALICE_DID,
		hash: H256::from_low_u64_be(1),
		tx_counter: mock_did_details.get_tx_counter_value() + 1u64
	};

	let signature = did_att_key.sign(&ctype_creation_operation.encode());

	let did_builder = did_mock::ExtBuilder::default().with_dids(vec![(ALICE_DID, mock_did_details)]);
	let ctype_builder = ExtBuilder::from(did_builder);

	let mut ext = ctype_builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Ctype::submit_ctype_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				ctype_creation_operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::VerificationKeysNotPresent
		);
	});
}

#[test]
fn check_invalid_signature_format_ctype_creation() {
	let did_auth_key = did_mock::get_ed25519_authentication_key(true);
	let did_enc_key = did_mock::get_x25519_encryption_key(true);
	let did_att_key = did_mock::get_sr25519_attestation_key(true);
	let wrong_format_att_key = did_mock::get_ed25519_attestation_key(true);
	let mut mock_did_details = did_mock::generate_mock_did_details(did::PublicVerificationKey::from(did_auth_key.public()), did_enc_key);
	mock_did_details.attestation_key = Some(did::PublicVerificationKey::from(did_att_key.public()));

	let ctype_creation_operation = ctype::CtypeCreationOperation {
		creator_did: did_mock::ALICE_DID,
		hash: H256::from_low_u64_be(1),
		tx_counter: mock_did_details.get_tx_counter_value() + 1u64
	};

	let signature = wrong_format_att_key.sign(&ctype_creation_operation.encode());

	let did_builder = did_mock::ExtBuilder::default().with_dids(vec![(ALICE_DID, mock_did_details)]);
	let ctype_builder = ExtBuilder::from(did_builder);

	let mut ext = ctype_builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Ctype::submit_ctype_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				ctype_creation_operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidSignatureFormat
		);
	});
}

#[test]
fn check_invalid_signature_ctype_creation() {
	let did_auth_key = did_mock::get_ed25519_authentication_key(true);
	let did_enc_key = did_mock::get_x25519_encryption_key(true);
	let did_att_key = did_mock::get_sr25519_attestation_key(true);
	let alternative_seed_att_key = did_mock::get_sr25519_attestation_key(false);
	let mut mock_did_details = did_mock::generate_mock_did_details(did::PublicVerificationKey::from(did_auth_key.public()), did_enc_key);
	mock_did_details.attestation_key = Some(did::PublicVerificationKey::from(did_att_key.public()));

	let ctype_creation_operation = ctype::CtypeCreationOperation {
		creator_did: did_mock::ALICE_DID,
		hash: H256::from_low_u64_be(1),
		tx_counter: mock_did_details.get_tx_counter_value() + 1u64
	};

	let signature = alternative_seed_att_key.sign(&ctype_creation_operation.encode());

	let did_builder = did_mock::ExtBuilder::default().with_dids(vec![(ALICE_DID, mock_did_details)]);
	let ctype_builder = ExtBuilder::from(did_builder);

	let mut ext = ctype_builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Ctype::submit_ctype_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				ctype_creation_operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidSignature
		);
	});
}
