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

use std::collections::BTreeSet;

use frame_support::assert_ok;
use kilt_primitives::AccountId;
use sp_core::{ed25519, sr25519};
use sp_core::{Pair, Public};

use codec::Encode;

use crate::mock::*;
use crate::{self as did, DIDCreationOperation};

#[test]
fn check_successfull_simpleed25519_creation() {
	ExtBuilder::default().build().execute_with(|| {
		let auth_key = get_ed25519_authentication_key(true);
		let enc_key = get_x25519_encryption_key();
		let did_creation_operation = generate_simple_did_creation_operation(ALICE_DID, did::PublicVerificationKey::from(auth_key.public()), enc_key);
		let signature = auth_key.sign(did_creation_operation.encode().as_ref());

		assert_ok!(
			Did::submit_did_create_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				did_creation_operation.clone(),
				did::DIDSignature::from(signature),
			)
		);

		let stored_did: did::DIDDetails = {
			let did_details = Did::get_did(ALICE_DID);
			assert!(did_details.is_some());
			did_details.unwrap()
		};
		assert_eq!(stored_did.auth_key, did_creation_operation.new_auth_key);
		assert_eq!(
			stored_did.key_agreement_key,
			did_creation_operation.new_key_agreement_key
		);
		assert_eq!(stored_did.delegation_key, did_creation_operation.new_delegation_key);
		assert_eq!(stored_did.attestation_key, did_creation_operation.new_attestation_key);
		assert_eq!(stored_did.verification_keys, <BTreeSet<did::PublicVerificationKey>>::new());
		assert_eq!(stored_did.endpoint_url, did_creation_operation.new_endpoint_url);
		assert_eq!(stored_did.last_tx_counter, 0u64);
	})
}
// #[test]
// fn check_successful_did_creation() {
// 	let did_identifier = AccountId::from([0u8; 32]);
// 	let did_auth_keypair_seed = [1u8; 32];
// 	let did_enc_keypair_public_bytes = [2u8; 32];
// 	let pair = ed25519::Pair::from_seed(&*b"Alice                           ");
// 	let account = MultiSigner::from(pair.public()).into_account();

// 	// New DID with only ed25519 auth key and x25519 encryption key.
// 	new_test_ext().execute_with(|| {
// 		let account = account.clone();
// 		let did_auth_keypair = ed25519::Pair::from_seed(&did_auth_keypair_seed);
// 		let did_creation_operation = DIDCreationOperation {
// 			did: did_identifier.clone(),
// 			new_auth_key: PublicVerificationKey::from(did_auth_keypair.public()),
// 			new_key_agreement_key: PublicEncryptionKey::X55519(did_enc_keypair_public_bytes),
// 			new_attestation_key: None,
// 			new_delegation_key: None,
// 			new_endpoint_url: None,
// 		};

// 		let operation_signature = did_auth_keypair.sign(&did_creation_operation.encode());

// 		assert_ok!(Did::submit_did_create_operation(
// 			Origin::signed(account),
// 			did_creation_operation.clone(),
// 			DIDSignature::from(operation_signature),
// 		));

// 		let stored_did: DIDDetails = {
// 			let did_details = Did::get_did(did_identifier.clone());
// 			assert!(did_details.is_some());
// 			did_details.unwrap()
// 		};
// 		assert_eq!(stored_did.auth_key, did_creation_operation.new_auth_key);
// 		assert_eq!(
// 			stored_did.key_agreement_key,
// 			did_creation_operation.new_key_agreement_key
// 		);
// 		assert_eq!(stored_did.delegation_key, did_creation_operation.new_delegation_key);
// 		assert_eq!(stored_did.attestation_key, did_creation_operation.new_attestation_key);
// 		assert_eq!(stored_did.verification_keys, <BTreeSet<PublicVerificationKey>>::new());
// 		assert_eq!(stored_did.endpoint_url, did_creation_operation.new_endpoint_url);
// 		assert_eq!(stored_did.last_tx_counter, 0u64);
// 	});

// 	// New DID with only sr25519 auth key and x25519 encryptio key.
// 	new_test_ext().execute_with(|| {
// 		let account = account.clone();
// 		let did_auth_keypair = sr25519::Pair::from_seed(&did_auth_keypair_seed);
// 		let did_creation_operation = DIDCreationOperation {
// 			did: did_identifier.clone(),
// 			new_auth_key: PublicVerificationKey::from(did_auth_keypair.public()),
// 			new_key_agreement_key: PublicEncryptionKey::X55519(did_enc_keypair_public_bytes),
// 			new_attestation_key: None,
// 			new_delegation_key: None,
// 			new_endpoint_url: None,
// 		};

// 		let operation_signature = did_auth_keypair.sign(&did_creation_operation.encode());

// 		assert_ok!(Did::submit_did_create_operation(
// 			Origin::signed(account),
// 			did_creation_operation.clone(),
// 			DIDSignature::from(operation_signature),
// 		));

// 		let stored_did: DIDDetails = {
// 			let did_details = Did::get_did(did_identifier.clone());
// 			assert!(did_details.is_some());
// 			did_details.unwrap()
// 		};
// 		assert_eq!(stored_did.auth_key, did_creation_operation.new_auth_key);
// 		assert_eq!(
// 			stored_did.key_agreement_key,
// 			did_creation_operation.new_key_agreement_key
// 		);
// 		assert_eq!(stored_did.delegation_key, did_creation_operation.new_delegation_key);
// 		assert_eq!(stored_did.attestation_key, did_creation_operation.new_attestation_key);
// 		assert_eq!(stored_did.verification_keys, <BTreeSet<PublicVerificationKey>>::new());
// 		assert_eq!(stored_did.endpoint_url, did_creation_operation.new_endpoint_url);
// 		assert_eq!(stored_did.last_tx_counter, 0u64);
// 	});

// 	// New DID with all keys and endpoint URL set.
// 	new_test_ext().execute_with(|| {
// 		let account = account.clone();
// 		let test_verification_seed = did_auth_keypair_seed;
// 		let did_auth_keypair = sr25519::Pair::from_seed(&test_verification_seed);
// 		let did_attestation_keypair = sr25519::Pair::from_seed(&test_verification_seed);
// 		let did_delegation_keypair = ed25519::Pair::from_seed(&test_verification_seed);
// 		let did_creation_operation = DIDCreationOperation {
// 			did: did_identifier.clone(),
// 			new_auth_key: PublicVerificationKey::from(did_auth_keypair.public()),
// 			new_key_agreement_key: PublicEncryptionKey::X55519(did_enc_keypair_public_bytes),
// 			new_attestation_key: Some(PublicVerificationKey::from(did_attestation_keypair.public())),
// 			new_delegation_key: Some(PublicVerificationKey::from(did_delegation_keypair.public())),
// 			new_endpoint_url: Some("https://kilt.io".into()),
// 		};

// 		let operation_signature = did_auth_keypair.sign(&did_creation_operation.encode());

// 		assert_ok!(Did::submit_did_create_operation(
// 			Origin::signed(account),
// 			did_creation_operation.clone(),
// 			DIDSignature::from(operation_signature),
// 		));

// 		let stored_did: DIDDetails = {
// 			let did_details = Did::get_did(did_identifier.clone());
// 			assert!(did_details.is_some());
// 			did_details.unwrap()
// 		};
// 		assert_eq!(stored_did.auth_key, did_creation_operation.new_auth_key);
// 		assert_eq!(
// 			stored_did.key_agreement_key,
// 			did_creation_operation.new_key_agreement_key
// 		);
// 		assert_eq!(stored_did.delegation_key, did_creation_operation.new_delegation_key);
// 		assert_eq!(stored_did.attestation_key, did_creation_operation.new_attestation_key);
// 		assert_eq!(stored_did.verification_keys, <BTreeSet<PublicVerificationKey>>::new());
// 		assert_eq!(stored_did.endpoint_url, did_creation_operation.new_endpoint_url);
// 		assert_eq!(stored_did.last_tx_counter, 0u64);
// 	});
// }

// #[test]
// fn check_invalid_did_creation() {
// 	let did_identifier = AccountId::from([0u8; 32]);
// 	let pair = ed25519::Pair::from_seed(&*b"Alice                           ");
// 	let account = MultiSigner::from(pair.public()).into_account();

// 	// Duplicate DID creation
// 	new_test_ext().execute_with(|| {
// 		let account_copy_1 = account.clone();

// 		let did_auth_keypair_seed = [2u8; 32];
// 		let did_auth_keypair = ed25519::Pair::from_seed(&did_auth_keypair_seed);
// 		let did_enc_keypair_public_bytes = [1u8; 32];
// 		let did_creation_operation = DIDCreationOperation {
// 			did: did_identifier.clone(),
// 			new_auth_key: PublicVerificationKey::from(did_auth_keypair.public()),
// 			new_key_agreement_key: PublicEncryptionKey::X55519(did_enc_keypair_public_bytes),
// 			new_attestation_key: None,
// 			new_delegation_key: None,
// 			new_endpoint_url: None,
// 		};

// 		let operation_signature = did_auth_keypair.sign(&did_creation_operation.encode());

// 		assert_ok!(Did::submit_did_create_operation(
// 			Origin::signed(account_copy_1),
// 			did_creation_operation.clone(),
// 			DIDSignature::from(operation_signature.clone()),
// 		));

// 		let account_copy_2 = account.clone();
// 		assert_noop!(
// 			Did::submit_did_create_operation(
// 				Origin::signed(account_copy_2),
// 				did_creation_operation.clone(),
// 				DIDSignature::from(operation_signature.clone()),
// 			),
// 			Error::<Test>::DIDAlreadyPresent
// 		);
// 	});

// 	// Invalid signature format provided
// 	new_test_ext().execute_with(|| {
// 		let account = account.clone();
// 		let did_auth_keypair_seed = [2u8; 32];
// 		let did_auth_keypair = ed25519::Pair::from_seed(&did_auth_keypair_seed);
// 		let did_enc_keypair_public_bytes = [1u8; 32];
// 		let did_creation_operation = DIDCreationOperation {
// 			did: did_identifier.clone(),
// 			new_auth_key: PublicVerificationKey::from(did_auth_keypair.public()),
// 			new_key_agreement_key: PublicEncryptionKey::X55519(did_enc_keypair_public_bytes),
// 			new_attestation_key: None,
// 			new_delegation_key: None,
// 			new_endpoint_url: None,
// 		};

// 		// Expected a Ed25519 key, used a Sr25519 one.
// 		let wrong_signing_keypair = sr25519::Pair::from_seed(&did_auth_keypair_seed);
// 		let wrong_operation_signature = wrong_signing_keypair.sign(&did_creation_operation.encode());

// 		assert_noop!(
// 			Did::submit_did_create_operation(
// 				Origin::signed(account),
// 				did_creation_operation.clone(),
// 				DIDSignature::from(wrong_operation_signature)
// 			),
// 			Error::<Test>::InvalidSignatureFormat
// 		);
// 	});

// 	// Invalid signature provided
// 	new_test_ext().execute_with(|| {
// 		let account = account.clone();
// 		let did_auth_keypair_seed = [2u8; 32];
// 		let did_auth_keypair = ed25519::Pair::from_seed(&did_auth_keypair_seed);
// 		let did_enc_keypair_public_bytes = [1u8; 32];
// 		let did_creation_operation = DIDCreationOperation {
// 			did: did_identifier.clone(),
// 			new_auth_key: PublicVerificationKey::from(did_auth_keypair.public()),
// 			new_key_agreement_key: PublicEncryptionKey::X55519(did_enc_keypair_public_bytes),
// 			new_attestation_key: None,
// 			new_delegation_key: None,
// 			new_endpoint_url: None,
// 		};

// 		// Same type of signature as the one expected, but different keypair used.
// 		let wrong_keypair = [255u8; 32];
// 		let wrong_signing_keypair = ed25519::Pair::from_seed(&wrong_keypair);
// 		let wrong_operation_signature = wrong_signing_keypair.sign(&did_creation_operation.encode());

// 		assert_noop!(
// 			Did::submit_did_create_operation(
// 				Origin::signed(account),
// 				did_creation_operation.clone(),
// 				DIDSignature::from(wrong_operation_signature)
// 			),
// 			Error::<Test>::InvalidSignature
// 		);
// 	})
// }

// #[test]
// fn check_verify_successful_did_operation_signature() {
// 	// Create and store a valid DID to use for verifying signatures for the
// 	// different operations.
// 	let pair = ed25519::Pair::from_seed(&*b"Alice                           ");
// 	let did_identifier = AccountId::from([0u8; 32]);
// 	let did_auth_keypair_seed = [1u8; 32];
// 	let did_enc_keypair_public_bytes = [2u8; 32];
// 	let did_attestation_seed = [3u8; 32];
// 	let did_delegation_seed = [4u8; 32];
// 	let did_auth_keypair = ed25519::Pair::from_seed(&did_auth_keypair_seed);
// 	let did_attestation_keypair = sr25519::Pair::from_seed(&did_attestation_seed);
// 	let did_delegation_keypair = ed25519::Pair::from_seed(&did_delegation_seed);
// 	let did_creation_operation = DIDCreationOperation {
// 		did: did_identifier.clone(),
// 		new_auth_key: PublicVerificationKey::from(did_auth_keypair.public()),
// 		new_key_agreement_key: PublicEncryptionKey::X55519(did_enc_keypair_public_bytes),
// 		new_attestation_key: Some(PublicVerificationKey::from(did_attestation_keypair.public())),
// 		new_delegation_key: Some(PublicVerificationKey::from(did_delegation_keypair.public())),
// 		new_endpoint_url: None,
// 	};

// 	let operation_signature = did_auth_keypair.sign(&did_creation_operation.encode());

// 	// Valid authentication key
// 	new_test_ext().execute_with(|| {
// 		let account = MultiSigner::from(pair.public()).into_account();
// 		let operation_signature = operation_signature.clone();
// 		assert_ok!(Did::submit_did_create_operation(
// 			Origin::signed(account),
// 			did_creation_operation.clone(),
// 			DIDSignature::from(operation_signature)
// 		));

// 		let test_did_op = TestDIDOperation {
// 			did: did_identifier.clone(),
// 			verification_key_type: DIDVerificationKeyType::Authentication,
// 		};
// 		let did_op_signature = did_auth_keypair.sign(&test_did_op.encode());

// 		assert_ok!(Did::verify_did_operation_signature::<TestDIDOperation<AccountId>>(
// 			&test_did_op,
// 			DIDSignature::from(did_op_signature)
// 		));
// 	});

// 	// Valid attestation key
// 	new_test_ext().execute_with(|| {
// 		let account = MultiSigner::from(pair.public()).into_account();
// 		let operation_signature = operation_signature.clone();
// 		assert_ok!(Did::submit_did_create_operation(
// 			Origin::signed(account),
// 			did_creation_operation.clone(),
// 			DIDSignature::from(operation_signature)
// 		));

// 		let test_did_op = TestDIDOperation {
// 			did: did_identifier.clone(),
// 			verification_key_type: DIDVerificationKeyType::AssertionMethod,
// 		};
// 		let did_op_signature = did_attestation_keypair.sign(&test_did_op.encode());

// 		assert_ok!(Did::verify_did_operation_signature::<TestDIDOperation<AccountId>>(
// 			&test_did_op,
// 			DIDSignature::from(did_op_signature)
// 		));
// 	});

// 	// Valid delegation key
// 	new_test_ext().execute_with(|| {
// 		let account = MultiSigner::from(pair.public()).into_account();
// 		let operation_signature = operation_signature.clone();
// 		assert_ok!(Did::submit_did_create_operation(
// 			Origin::signed(account),
// 			did_creation_operation.clone(),
// 			DIDSignature::from(operation_signature)
// 		));

// 		let test_did_op = TestDIDOperation {
// 			did: did_identifier.clone(),
// 			verification_key_type: DIDVerificationKeyType::CapabilityDelegation,
// 		};
// 		let did_op_signature = did_delegation_keypair.sign(&test_did_op.encode());

// 		assert_ok!(Did::verify_did_operation_signature::<TestDIDOperation<AccountId>>(
// 			&test_did_op,
// 			DIDSignature::from(did_op_signature.clone())
// 		));
// 	});
// }

// #[test]
// fn check_verify_invalid_did_operation_signature() {
// 	// Create and store a valid DID to use for verifying signatures for the
// 	// different operations.
// 	let pair = ed25519::Pair::from_seed(&*b"Alice                           ");
// 	let account = MultiSigner::from(pair.public()).into_account();
// 	let did_identifier = AccountId::from([0u8; 32]);
// 	let did_auth_keypair_seed = [1u8; 32];
// 	let did_enc_keypair_public_bytes = [2u8; 32];
// 	let did_attestation_seed = [3u8; 32];
// 	let did_delegation_seed = [4u8; 32];
// 	let did_auth_keypair = ed25519::Pair::from_seed(&did_auth_keypair_seed);
// 	let did_attestation_keypair = sr25519::Pair::from_seed(&did_attestation_seed);
// 	let did_delegation_keypair = ed25519::Pair::from_seed(&did_delegation_seed);
// 	let did_creation_operation = DIDCreationOperation {
// 		did: did_identifier.clone(),
// 		new_auth_key: PublicVerificationKey::from(did_auth_keypair.public()),
// 		new_key_agreement_key: PublicEncryptionKey::X55519(did_enc_keypair_public_bytes),
// 		new_attestation_key: Some(PublicVerificationKey::from(did_attestation_keypair.public())),
// 		new_delegation_key: Some(PublicVerificationKey::from(did_delegation_keypair.public())),
// 		new_endpoint_url: None,
// 	};

// 	let operation_signature = did_auth_keypair.sign(&did_creation_operation.encode());

// 	// DID not present on chain
// 	new_test_ext().execute_with(|| {
// 		let unsaved_did_identifier = AccountId::from([0u8; 32]);
// 		let test_did_op = TestDIDOperation {
// 			did: unsaved_did_identifier,
// 			verification_key_type: DIDVerificationKeyType::Authentication,
// 		};
// 		let did_op_signature = did_auth_keypair.sign(&test_did_op.encode());

// 		assert_noop!(
// 			Did::verify_did_operation_signature::<TestDIDOperation<AccountId>>(
// 				&test_did_op,
// 				DIDSignature::from(did_op_signature)
// 			),
// 			DIDError::StorageError(StorageError::DIDNotPresent)
// 		);
// 	});

// 	// Specified verification key not present in the DID document
// 	new_test_ext().execute_with(|| {
// 		let did_creation_operation = DIDCreationOperation {
// 			did: did_identifier.clone(),
// 			new_auth_key: PublicVerificationKey::from(did_auth_keypair.public()),
// 			new_key_agreement_key: PublicEncryptionKey::X55519(did_enc_keypair_public_bytes),
// 			new_attestation_key: Some(PublicVerificationKey::from(did_attestation_keypair.public())),
// 			new_delegation_key: None, // No delegation key specified
// 			new_endpoint_url: None,
// 		};
// 		let operation_signature = did_auth_keypair.sign(&did_creation_operation.encode());
// 		let account = account.clone();

// 		assert_ok!(Did::submit_did_create_operation(
// 			Origin::signed(account),
// 			did_creation_operation.clone(),
// 			DIDSignature::from(operation_signature)
// 		));

// 		let test_verification_key_required = DIDVerificationKeyType::CapabilityDelegation;
// 		let test_did_op = TestDIDOperation {
// 			did: did_identifier.clone(),
// 			verification_key_type: test_verification_key_required.clone(),
// 		};
// 		let did_op_signature = did_delegation_keypair.sign(&test_did_op.encode());

// 		assert_noop!(
// 			Did::verify_did_operation_signature::<TestDIDOperation<AccountId>>(
// 				&test_did_op,
// 				DIDSignature::from(did_op_signature)
// 			),
// 			DIDError::StorageError(StorageError::VerificationkeyNotPresent(
// 				test_verification_key_required.clone()
// 			))
// 		);
// 	});

// 	// Invalid signature format
// 	new_test_ext().execute_with(|| {
// 		let account = account.clone();
// 		assert_ok!(Did::submit_did_create_operation(
// 			Origin::signed(account),
// 			did_creation_operation.clone(),
// 			DIDSignature::from(operation_signature.clone())
// 		));

// 		let test_did_op = TestDIDOperation {
// 			did: did_identifier.clone(),
// 			verification_key_type: DIDVerificationKeyType::CapabilityDelegation,
// 		};

// 		// Expected an Ed25519 signature, but Sr25519 provided.
// 		let wrong_signing_keypair = sr25519::Pair::from_seed(&did_auth_keypair_seed);
// 		let wrong_operation_signature = wrong_signing_keypair.sign(&did_creation_operation.encode());

// 		assert_noop!(
// 			Did::verify_did_operation_signature::<TestDIDOperation<AccountId>>(
// 				&test_did_op,
// 				DIDSignature::from(wrong_operation_signature)
// 			),
// 			DIDError::SignatureError(SignatureError::InvalidSignatureFormat)
// 		);
// 	});

// 	// Invalid signature
// 	new_test_ext().execute_with(|| {
// 		let account = account.clone();
// 		assert_ok!(Did::submit_did_create_operation(
// 			Origin::signed(account),
// 			did_creation_operation.clone(),
// 			DIDSignature::from(operation_signature.clone()),
// 		));

// 		let test_did_op = TestDIDOperation {
// 			did: did_identifier.clone(),
// 			verification_key_type: DIDVerificationKeyType::CapabilityDelegation,
// 		};

// 		// Same type of signature but different keypair
// 		let wrong_keypair = [255u8; 32];
// 		let wrong_signing_keypair = ed25519::Pair::from_seed(&wrong_keypair);
// 		let wrong_operation_signature = wrong_signing_keypair.sign(&did_creation_operation.encode());

// 		assert_noop!(
// 			Did::verify_did_operation_signature::<TestDIDOperation<AccountId>>(
// 				&test_did_op,
// 				DIDSignature::from(wrong_operation_signature)
// 			),
// 			DIDError::SignatureError(SignatureError::InvalidSignature)
// 		);
// 	});
// }
