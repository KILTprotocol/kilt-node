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

use crate as pallet_did;
use crate::*;
use test_utils::*;

use codec::Encode;
use frame_support::{
	assert_noop, assert_ok, parameter_types,
	weights::{
		constants::{BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight},
		DispatchClass,
	},
};
use frame_system::limits::{BlockLength, BlockWeights};
use kilt_primitives::Signature;
use sp_core::{ed25519, Pair, H256};
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
	MultiSigner,
};

use sp_std::vec::Vec;

pub type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
pub type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Module, Call, Config, Storage, Event<T>},
		Did: pallet_did::{Module, Call, Storage, Event<T>},
	}
);

parameter_types! {
	pub RuntimeBlockLength: BlockLength =
		BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub RuntimeBlockWeights: BlockWeights = BlockWeights::builder()
		.base_block(BlockExecutionWeight::get())
		.for_class(DispatchClass::all(), |weights| {
			weights.base_extrinsic = ExtrinsicBaseWeight::get();
		})
		.for_class(DispatchClass::Normal, |weights| {
			weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
		})
		.for_class(DispatchClass::Operational, |weights| {
			weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
			// Operational transactions have some extra reserved space, so that they
			// are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
			weights.reserved = Some(
				MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT
			);
		})
		.avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
		.build_or_panic();
	pub const SS58Prefix: u8 = 38;
	pub const BlockHashCount: u64 = 250;
}

impl frame_system::Config for Test {
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = ();
	type BlockHashCount = BlockHashCount;
	type DbWeight = RocksDbWeight;
	type Version = ();

	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type BaseCallFilter = ();
	type SystemWeightInfo = ();
	type BlockWeights = RuntimeBlockWeights;
	type BlockLength = RuntimeBlockLength;
	type SS58Prefix = SS58Prefix;
}

impl Config for Test {
	type Event = ();
	type WeightInfo = ();
}

#[test]
fn check_successful_did_creation() {
	let did_identifier = AccountId32::from([0u8; 32]);
	let did_enc_keypair_seed = [1u8; 32];
	let pair = ed25519::Pair::from_seed(&*b"Alice                           ");
	let account = MultiSigner::from(pair.public()).into_account();

	// New DID with only ed25519 auth key and x25519 encryption key.
	new_test_ext().execute_with(|| {
		let account = account.clone();
		let did_auth_keypair_seed = [2u8; 32];
		let did_auth_keypair = ed25519::Pair::from_seed(&did_auth_keypair_seed);
		let did_creation_operation = DIDCreationOperation {
			did: did_identifier.clone(),
			new_auth_key: PublicVerificationKey::Ed25519(did_auth_keypair.public().into()),
			new_key_agreement_key: PublicEncryptionKey::X55519(did_enc_keypair_seed),
			new_attestation_key: None,
			new_delegation_key: None,
			new_endpoint_url: None,
		};
		let operation_signature = did_auth_keypair.sign(&did_creation_operation.encode());
		assert_ok!(Did::submit_did_create_operation(
			Origin::signed(account),
			did_creation_operation.clone(),
			operation_signature.encode()
		));

		let stored_did: DIDDetails = {
			let did_details = Did::get_did(did_identifier.clone());
			assert!(did_details.is_some());
			did_details.unwrap()
		};
		assert_eq!(stored_did.auth_key, did_creation_operation.new_auth_key);
		assert_eq!(
			stored_did.key_agreement_key,
			did_creation_operation.new_key_agreement_key
		);
		assert_eq!(
			stored_did.delegation_key,
			did_creation_operation.new_delegation_key
		);
		assert_eq!(
			stored_did.attestation_key,
			did_creation_operation.new_attestation_key
		);
		assert_eq!(
			stored_did.verification_keys,
			<BTreeSet<PublicVerificationKey>>::new()
		);
		assert_eq!(
			stored_did.endpoint_url,
			did_creation_operation.new_endpoint_url
		);
		assert_eq!(stored_did.last_tx_counter, 0u64);
	});

	// New DID with only sr25519 auth key and x25519 encryptio key.
	new_test_ext().execute_with(|| {
		let account = account.clone();
		let did_auth_keypair_seed = [2u8; 32];
		let did_auth_keypair = sr25519::Pair::from_seed(&did_auth_keypair_seed);
		let did_creation_operation = DIDCreationOperation {
			did: did_identifier.clone(),
			new_auth_key: PublicVerificationKey::Sr25519(did_auth_keypair.public().into()),
			new_key_agreement_key: PublicEncryptionKey::X55519(did_enc_keypair_seed),
			new_attestation_key: None,
			new_delegation_key: None,
			new_endpoint_url: None,
		};
		let operation_signature = did_auth_keypair.sign(&did_creation_operation.encode());
		assert_ok!(Did::submit_did_create_operation(
			Origin::signed(account),
			did_creation_operation.clone(),
			operation_signature.encode()
		));

		let stored_did: DIDDetails = {
			let did_details = Did::get_did(did_identifier.clone());
			assert!(did_details.is_some());
			did_details.unwrap()
		};
		assert_eq!(stored_did.auth_key, did_creation_operation.new_auth_key);
		assert_eq!(
			stored_did.key_agreement_key,
			did_creation_operation.new_key_agreement_key
		);
		assert_eq!(
			stored_did.delegation_key,
			did_creation_operation.new_delegation_key
		);
		assert_eq!(
			stored_did.attestation_key,
			did_creation_operation.new_attestation_key
		);
		assert_eq!(
			stored_did.verification_keys,
			<BTreeSet<PublicVerificationKey>>::new()
		);
		assert_eq!(
			stored_did.endpoint_url,
			did_creation_operation.new_endpoint_url
		);
		assert_eq!(stored_did.last_tx_counter, 0u64);
	});

	// New DID with all keys and endpoint URL set.
	new_test_ext().execute_with(|| {
		let account = account.clone();
		let test_verification_seed = [2u8; 32];
		let did_auth_keypair = sr25519::Pair::from_seed(&test_verification_seed);
		let did_attestation_keypair = sr25519::Pair::from_seed(&test_verification_seed);
		let did_delegation_keypair = ed25519::Pair::from_seed(&test_verification_seed);
		let did_creation_operation = DIDCreationOperation {
			did: did_identifier.clone(),
			new_auth_key: PublicVerificationKey::Sr25519(did_auth_keypair.public().into()),
			new_key_agreement_key: PublicEncryptionKey::X55519(did_enc_keypair_seed),
			new_attestation_key: Some(PublicVerificationKey::Sr25519(
				did_attestation_keypair.public().into(),
			)),
			new_delegation_key: Some(PublicVerificationKey::Ed25519(
				did_delegation_keypair.public().into(),
			)),
			new_endpoint_url: Some("https://kilt.io".into()),
		};
		let operation_signature = did_auth_keypair.sign(&did_creation_operation.encode());
		assert_ok!(Did::submit_did_create_operation(
			Origin::signed(account),
			did_creation_operation.clone(),
			operation_signature.encode()
		));

		let stored_did: DIDDetails = {
			let did_details = Did::get_did(did_identifier.clone());
			assert!(did_details.is_some());
			did_details.unwrap()
		};
		assert_eq!(stored_did.auth_key, did_creation_operation.new_auth_key);
		assert_eq!(
			stored_did.key_agreement_key,
			did_creation_operation.new_key_agreement_key
		);
		assert_eq!(
			stored_did.delegation_key,
			did_creation_operation.new_delegation_key
		);
		assert_eq!(
			stored_did.attestation_key,
			did_creation_operation.new_attestation_key
		);
		assert_eq!(
			stored_did.verification_keys,
			<BTreeSet<PublicVerificationKey>>::new()
		);
		assert_eq!(
			stored_did.endpoint_url,
			did_creation_operation.new_endpoint_url
		);
		assert_eq!(stored_did.last_tx_counter, 0u64);
	});
}

#[test]
fn check_invalid_did_creation() {
	let did_identifier = AccountId32::from([0u8; 32]);
	let pair = ed25519::Pair::from_seed(&*b"Alice                           ");
	let account = MultiSigner::from(pair.public()).into_account();

	// Duplicate DID creation
	new_test_ext().execute_with(|| {
		let account_copy_1 = account.clone();

		let did_auth_keypair_seed = [2u8; 32];
		let did_auth_keypair = ed25519::Pair::from_seed(&did_auth_keypair_seed);
		let did_enc_keypair_seed = [1u8; 32];
		let did_creation_operation = DIDCreationOperation {
			did: did_identifier.clone(),
			new_auth_key: PublicVerificationKey::Ed25519(did_auth_keypair.public().into()),
			new_key_agreement_key: PublicEncryptionKey::X55519(did_enc_keypair_seed),
			new_attestation_key: None,
			new_delegation_key: None,
			new_endpoint_url: None,
		};
		let operation_signature = did_auth_keypair.sign(&did_creation_operation.encode());

		assert_ok!(Did::submit_did_create_operation(
			Origin::signed(account_copy_1),
			did_creation_operation.clone(),
			operation_signature.encode()
		));

		let account_copy_2 = account.clone();
		assert_noop!(
			Did::submit_did_create_operation(
				Origin::signed(account_copy_2),
				did_creation_operation.clone(),
				operation_signature.encode()
			),
			Error::<Test>::DIDAlreadyPresent
		);
	});

	// Invalid signature format provided
	new_test_ext().execute_with(|| {
		let account = account.clone();
		let did_auth_keypair_seed = [2u8; 32];
		let did_auth_keypair = ed25519::Pair::from_seed(&did_auth_keypair_seed);
		let did_enc_keypair_seed = [1u8; 32];
		let did_creation_operation = DIDCreationOperation {
			did: did_identifier.clone(),
			new_auth_key: PublicVerificationKey::Ed25519(did_auth_keypair.public().into()),
			new_key_agreement_key: PublicEncryptionKey::X55519(did_enc_keypair_seed),
			new_attestation_key: None,
			new_delegation_key: None,
			new_endpoint_url: None,
		};

		// 0-byte signature
		let sig_length = 0usize;
		let account_copy_1 = account.clone();
		assert_noop!(
			Did::submit_did_create_operation(
				Origin::signed(account_copy_1),
				did_creation_operation.clone(),
				vec![0; sig_length]
			),
			Error::<Test>::InvalidSignatureFormat
		);

		// (expected_length - 1)-byte signature
		let sig_length = did_creation_operation
			.new_auth_key
			.get_expected_signature_size()
			- 1;
		let account_copy_2 = account.clone();
		assert_noop!(
			Did::submit_did_create_operation(
				Origin::signed(account_copy_2),
				did_creation_operation.clone(),
				vec![0; sig_length]
			),
			Error::<Test>::InvalidSignatureFormat
		);

		// (expected_length - +1)-byte signature
		let sig_length = did_creation_operation
			.new_auth_key
			.get_expected_signature_size()
			+ 1;
		let account_copy_3 = account.clone();
		assert_noop!(
			Did::submit_did_create_operation(
				Origin::signed(account_copy_3),
				did_creation_operation.clone(),
				vec![0; sig_length]
			),
			Error::<Test>::InvalidSignatureFormat
		);

		// Very long signature
		let account_copy_4 = account.clone();
		assert_noop!(
			Did::submit_did_create_operation(
				Origin::signed(account_copy_4),
				did_creation_operation.clone(),
				vec![0; 1_000_000_000usize]
			),
			Error::<Test>::InvalidSignatureFormat
		);
	});

	// Invalid signature provided
	new_test_ext().execute_with(|| {
		let account = account.clone();
		let did_auth_keypair_seed = [2u8; 32];
		let did_auth_keypair = ed25519::Pair::from_seed(&did_auth_keypair_seed);
		let did_enc_keypair_seed = [1u8; 32];
		let did_creation_operation = DIDCreationOperation {
			did: did_identifier.clone(),
			new_auth_key: PublicVerificationKey::Ed25519(did_auth_keypair.public().into()),
			new_key_agreement_key: PublicEncryptionKey::X55519(did_enc_keypair_seed),
			new_attestation_key: None,
			new_delegation_key: None,
			new_endpoint_url: None,
		};

		// Test with 0 signature
		let sig_length = did_creation_operation
			.new_auth_key
			.get_expected_signature_size();
		let zero_signature = vec![0u8; sig_length];
		let account_copy_1 = account.clone();
		assert_noop!(
			Did::submit_did_create_operation(
				Origin::signed(account_copy_1),
				did_creation_operation.clone(),
				zero_signature
			),
			Error::<Test>::InvalidSignature
		);
	})
}

#[test]
fn check_verify_successful_did_operation_signature() {
	// Create and store a valid DID to use for verifying signatures for the different operations.
	let pair = ed25519::Pair::from_seed(&*b"Alice                           ");
	let did_identifier = AccountId32::from([0u8; 32]);
	let did_auth_keypair_seed = [1u8; 32];
	let did_enc_keypair_seed = [2u8; 32];
	let did_attestation_seed = [3u8; 32];
	let did_delegation_seed = [4u8; 32];
	let did_auth_keypair = ed25519::Pair::from_seed(&did_auth_keypair_seed);
	let did_attestation_keypair = sr25519::Pair::from_seed(&did_attestation_seed);
	let did_delegation_keypair = ed25519::Pair::from_seed(&did_delegation_seed);
	let did_creation_operation = DIDCreationOperation {
		did: did_identifier.clone(),
		new_auth_key: PublicVerificationKey::Ed25519(did_auth_keypair.public().into()),
		new_key_agreement_key: PublicEncryptionKey::X55519(did_enc_keypair_seed),
		new_attestation_key: Some(PublicVerificationKey::Sr25519(
			did_attestation_keypair.public().into(),
		)),
		new_delegation_key: Some(PublicVerificationKey::Ed25519(
			did_delegation_keypair.public().into(),
		)),
		new_endpoint_url: None,
	};

	let operation_signature = did_auth_keypair.sign(&did_creation_operation.encode());

	// Valid authentication key
	new_test_ext().execute_with(|| {
		let account = MultiSigner::from(pair.public()).into_account();
		assert_ok!(Did::submit_did_create_operation(
			Origin::signed(account),
			did_creation_operation.clone(),
			operation_signature.encode()
		));

		let test_did_op = TestDIDOperation {
			did: did_identifier.clone(),
			verification_key_type: DIDVerificationKeyType::Authentication,
		};
		let did_op_signature = did_auth_keypair.sign(&test_did_op.encode());
		let did_op_signature_encoded = did_op_signature.encode();

		assert_ok!(Did::verify_did_operation_signature::<TestDIDOperation>(
			&test_did_op,
			&did_op_signature_encoded
		));
	});

	// Valid attestation key
	new_test_ext().execute_with(|| {
		let account = MultiSigner::from(pair.public()).into_account();
		assert_ok!(Did::submit_did_create_operation(
			Origin::signed(account),
			did_creation_operation.clone(),
			operation_signature.encode()
		));

		let test_did_op = TestDIDOperation {
			did: did_identifier.clone(),
			verification_key_type: DIDVerificationKeyType::AssertionMethod,
		};
		let did_op_signature = did_attestation_keypair.sign(&test_did_op.encode());
		let did_op_signature_encoded = did_op_signature.encode();

		assert_ok!(Did::verify_did_operation_signature::<TestDIDOperation>(
			&test_did_op,
			&did_op_signature_encoded
		));
	});

	// Valid delegation key
	new_test_ext().execute_with(|| {
		let account = MultiSigner::from(pair.public()).into_account();
		assert_ok!(Did::submit_did_create_operation(
			Origin::signed(account),
			did_creation_operation.clone(),
			operation_signature.encode()
		));

		let test_did_op = TestDIDOperation {
			did: did_identifier.clone(),
			verification_key_type: DIDVerificationKeyType::CapabilityDelegation,
		};
		let did_op_signature = did_delegation_keypair.sign(&test_did_op.encode());
		let did_op_signature_encoded = did_op_signature.encode();

		assert_ok!(Did::verify_did_operation_signature::<TestDIDOperation>(
			&test_did_op,
			&did_op_signature_encoded
		));
	});
}

#[test]
fn check_verify_invalid_did_operation_signature() {
	// Create and store a valid DID to use for verifying signatures for the different operations.
	let pair = ed25519::Pair::from_seed(&*b"Alice                           ");
	let did_identifier = AccountId32::from([0u8; 32]);
	let did_auth_keypair_seed = [1u8; 32];
	let did_enc_keypair_seed = [2u8; 32];
	let did_attestation_seed = [3u8; 32];
	let did_delegation_seed = [4u8; 32];
	let did_auth_keypair = ed25519::Pair::from_seed(&did_auth_keypair_seed);
	let did_attestation_keypair = sr25519::Pair::from_seed(&did_attestation_seed);
	let did_delegation_keypair = ed25519::Pair::from_seed(&did_delegation_seed);
	let did_creation_operation = DIDCreationOperation {
		did: did_identifier.clone(),
		new_auth_key: PublicVerificationKey::Ed25519(did_auth_keypair.public().into()),
		new_key_agreement_key: PublicEncryptionKey::X55519(did_enc_keypair_seed),
		new_attestation_key: Some(PublicVerificationKey::Sr25519(
			did_attestation_keypair.public().into(),
		)),
		new_delegation_key: Some(PublicVerificationKey::Ed25519(
			did_delegation_keypair.public().into(),
		)),
		new_endpoint_url: None,
	};

	let operation_signature = did_auth_keypair.sign(&did_creation_operation.encode());

	// DID not present on chain
	new_test_ext().execute_with(|| {
		let unsaved_did_identifier = AccountId32::from([255u8; 32]);
		let test_did_op = TestDIDOperation {
			did: unsaved_did_identifier,
			verification_key_type: DIDVerificationKeyType::Authentication,
		};
		let did_op_signature = did_auth_keypair.sign(&test_did_op.encode());
		let did_op_signature_encoded = did_op_signature.encode();

		assert_noop!(
			Did::verify_did_operation_signature::<TestDIDOperation>(
				&test_did_op,
				&did_op_signature_encoded
			),
			DIDError::StorageError(StorageError::DIDNotPresent)
		);
	});

	// Specified verification key not present in the DID document
	new_test_ext().execute_with(|| {
		let did_creation_operation = DIDCreationOperation {
			did: did_identifier.clone(),
			new_auth_key: PublicVerificationKey::Ed25519(did_auth_keypair.public().into()),
			new_key_agreement_key: PublicEncryptionKey::X55519(did_enc_keypair_seed),
			new_attestation_key: Some(PublicVerificationKey::Sr25519(
				did_attestation_keypair.public().into(),
			)),
			new_delegation_key: None, // No delegation key specified
			new_endpoint_url: None,
		};
		let operation_signature = did_auth_keypair.sign(&did_creation_operation.encode());

		let account = MultiSigner::from(pair.public()).into_account();
		assert_ok!(Did::submit_did_create_operation(
			Origin::signed(account),
			did_creation_operation.clone(),
			operation_signature.encode()
		));

		let test_verification_key_required = DIDVerificationKeyType::CapabilityDelegation;
		let test_did_op = TestDIDOperation {
			did: did_identifier.clone(),
			verification_key_type: test_verification_key_required.clone(),
		};
		let did_op_signature = did_delegation_keypair.sign(&test_did_op.encode());
		let did_op_signature_encoded = did_op_signature.encode();

		assert_noop!(
			Did::verify_did_operation_signature::<TestDIDOperation>(
				&test_did_op,
				&did_op_signature_encoded
			),
			DIDError::StorageError(StorageError::VerificationkeyNotPresent(
				test_verification_key_required.clone()
			))
		);
	});

	// Invalid signature format
	new_test_ext().execute_with(|| {
		let account = MultiSigner::from(pair.public()).into_account();
		assert_ok!(Did::submit_did_create_operation(
			Origin::signed(account),
			did_creation_operation.clone(),
			operation_signature.encode()
		));

		let test_did_op = TestDIDOperation {
			did: did_identifier.clone(),
			verification_key_type: DIDVerificationKeyType::CapabilityDelegation,
		};
		let invalid_signature_encoded = vec![
			0u8;
			did_creation_operation
				.new_delegation_key
				.clone()
				.unwrap()
				.get_expected_signature_size()
				+ 1
		]; // Expected signature length + 1 byte, all 0s.

		assert_noop!(
			Did::verify_did_operation_signature::<TestDIDOperation>(
				&test_did_op,
				invalid_signature_encoded.as_ref()
			),
			DIDError::SignatureError(SignatureError::InvalidSignatureFormat)
		);
	});

	// Invalid signature
	new_test_ext().execute_with(|| {
		let account = MultiSigner::from(pair.public()).into_account();
		assert_ok!(Did::submit_did_create_operation(
			Origin::signed(account),
			did_creation_operation.clone(),
			operation_signature.encode()
		));

		let test_did_op = TestDIDOperation {
			did: did_identifier.clone(),
			verification_key_type: DIDVerificationKeyType::CapabilityDelegation,
		};
		let invalid_signature_encoded = vec![
			0u8;
			did_creation_operation
				.new_delegation_key
				.clone()
				.unwrap()
				.get_expected_signature_size()
		]; // Expected length, but all 0s.

		assert_noop!(
			Did::verify_did_operation_signature::<TestDIDOperation>(
				&test_did_op,
				invalid_signature_encoded.as_ref()
			),
			DIDError::SignatureError(SignatureError::InvalidSignature)
		);
	});
}
