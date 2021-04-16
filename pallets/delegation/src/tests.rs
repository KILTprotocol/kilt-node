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

use crate::{self as delegation, mock::*};
use ctype::mock as ctype_mock;
use did::mock as did_mock;
use frame_support::{assert_noop, assert_ok};
use sp_core::Pair;

use codec::Encode;

// submit_delegation_root_creation_operation()

#[test]
fn check_submit_delegation_root_creation_operation_successful() {
	let auth_key = did_mock::get_ed25519_authentication_key(true);
	let enc_key = did_mock::get_x25519_encryption_key(true);
	let del_key = did_mock::get_sr25519_delegation_key(true);

	let mut delegator_details = did_mock::generate_mock_did_details(
		did::PublicVerificationKey::from(auth_key.public()),
		did::PublicEncryptionKey::from(enc_key),
	);
	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(del_key.public()));

	let ctype_hash = ctype_mock::get_ctype_hash(true);
	let root_id = get_delegation_root_id(true);

	let operation = delegation::DelegationRootCreationOperation {
		creator_did: did_mock::ALICE_DID,
		ctype_hash: ctype_hash,
		root_id: root_id,
		tx_counter: delegator_details.get_tx_counter_value() + 1u64,
	};
	let signature = del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(did_mock::ALICE_DID, delegator_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash, did_mock::ALICE_DID)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_ok!(Delegation::submit_delegation_root_creation_operation(
			Origin::signed(DEFAULT_ACCOUNT),
			operation.clone(),
			did::DidSignature::from(signature)
		));
	});

	let stored_delegation_root = ext
		.execute_with(|| Delegation::roots(&operation.root_id).expect("Delegation root should be present on chain."));

	assert_eq!(stored_delegation_root.ctype_hash, operation.ctype_hash);
	assert_eq!(stored_delegation_root.owner, operation.creator_did);
	assert_eq!(stored_delegation_root.revoked, false);

	// Verify that the DID tx counter has increased
	let new_delegator_details = ext.execute_with(|| {
		Did::get_did(&operation.creator_did).expect("Delegation root creator should be present on chain.")
	});
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value() + 1u64
	);
}

#[test]
fn check_did_not_found_submit_delegation_root_creation_operation() {
	let auth_key = did_mock::get_ed25519_authentication_key(true);
	let enc_key = did_mock::get_x25519_encryption_key(true);
	let del_key = did_mock::get_sr25519_delegation_key(true);

	let mut delegator_details = did_mock::generate_mock_did_details(
		did::PublicVerificationKey::from(auth_key.public()),
		did::PublicEncryptionKey::from(enc_key),
	);
	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(del_key.public()));

	let ctype_hash = ctype_mock::get_ctype_hash(true);
	let root_id = get_delegation_root_id(true);

	let operation = delegation::DelegationRootCreationOperation {
		creator_did: did_mock::ALICE_DID,
		ctype_hash: ctype_hash,
		root_id: root_id,
		tx_counter: delegator_details.get_tx_counter_value() + 1u64,
	};
	let signature = del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(did_mock::BOB_DID, delegator_details.clone())]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_root_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::DidNotPresent
		);
	});
}

#[test]
fn check_did_max_tx_counter_submit_delegation_root_creation_operation() {
	let auth_key = did_mock::get_ed25519_authentication_key(true);
	let enc_key = did_mock::get_x25519_encryption_key(true);
	let del_key = did_mock::get_sr25519_delegation_key(true);

	let mut delegator_details = did_mock::generate_mock_did_details(
		did::PublicVerificationKey::from(auth_key.public()),
		did::PublicEncryptionKey::from(enc_key),
	);
	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(del_key.public()));
	delegator_details.set_tx_counter(u64::MAX);

	let ctype_hash = ctype_mock::get_ctype_hash(true);
	let root_id = get_delegation_root_id(true);

	let operation = delegation::DelegationRootCreationOperation {
		creator_did: did_mock::ALICE_DID,
		ctype_hash: ctype_hash,
		root_id: root_id,
		tx_counter: delegator_details.get_tx_counter_value(),
	};
	let signature = del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(did_mock::ALICE_DID, delegator_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash.clone(), did_mock::ALICE_DID)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_root_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::MaxTxCounterValue
		);
	});
}

#[test]
fn check_did_too_small_tx_counter_submit_delegation_root_creation_operation() {
	let auth_key = did_mock::get_ed25519_authentication_key(true);
	let enc_key = did_mock::get_x25519_encryption_key(true);
	let del_key = did_mock::get_sr25519_delegation_key(true);

	let mut delegator_details = did_mock::generate_mock_did_details(
		did::PublicVerificationKey::from(auth_key.public()),
		did::PublicEncryptionKey::from(enc_key),
	);
	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(del_key.public()));
	delegator_details.set_tx_counter(1u64);

	let ctype_hash = ctype_mock::get_ctype_hash(true);
	let root_id = get_delegation_root_id(true);

	let operation = delegation::DelegationRootCreationOperation {
		creator_did: did_mock::ALICE_DID,
		ctype_hash: ctype_hash,
		root_id: root_id,
		tx_counter: delegator_details.get_tx_counter_value() - 1u64,
	};
	let signature = del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(did_mock::ALICE_DID, delegator_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash.clone(), did_mock::ALICE_DID)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_root_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidNonce
		);
	});
}

#[test]
fn check_did_equal_tx_counter_submit_delegation_root_creation_operation() {
	let auth_key = did_mock::get_ed25519_authentication_key(true);
	let enc_key = did_mock::get_x25519_encryption_key(true);
	let del_key = did_mock::get_sr25519_delegation_key(true);

	let mut delegator_details = did_mock::generate_mock_did_details(
		did::PublicVerificationKey::from(auth_key.public()),
		did::PublicEncryptionKey::from(enc_key),
	);
	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(del_key.public()));

	let ctype_hash = ctype_mock::get_ctype_hash(true);
	let root_id = get_delegation_root_id(true);

	let operation = delegation::DelegationRootCreationOperation {
		creator_did: did_mock::ALICE_DID,
		ctype_hash: ctype_hash,
		root_id: root_id,
		tx_counter: delegator_details.get_tx_counter_value(),
	};
	let signature = del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(did_mock::ALICE_DID, delegator_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash.clone(), did_mock::ALICE_DID)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_root_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidNonce
		);
	});
}

#[test]
fn check_did_too_large_tx_counter_submit_delegation_root_creation_operation() {
	let auth_key = did_mock::get_ed25519_authentication_key(true);
	let enc_key = did_mock::get_x25519_encryption_key(true);
	let del_key = did_mock::get_sr25519_delegation_key(true);

	let mut delegator_details = did_mock::generate_mock_did_details(
		did::PublicVerificationKey::from(auth_key.public()),
		did::PublicEncryptionKey::from(enc_key),
	);
	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(del_key.public()));

	let ctype_hash = ctype_mock::get_ctype_hash(true);
	let root_id = get_delegation_root_id(true);

	let operation = delegation::DelegationRootCreationOperation {
		creator_did: did_mock::ALICE_DID,
		ctype_hash: ctype_hash,
		root_id: root_id,
		tx_counter: delegator_details.get_tx_counter_value() + 2u64,
	};
	let signature = del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(did_mock::ALICE_DID, delegator_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash.clone(), did_mock::ALICE_DID)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_root_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidNonce
		);
	});
}

#[test]
fn check_did_delegation_key_not_present_submit_delegation_root_creation_operation() {
	let auth_key = did_mock::get_ed25519_authentication_key(true);
	let enc_key = did_mock::get_x25519_encryption_key(true);
	let del_key = did_mock::get_sr25519_delegation_key(true);

	let mut delegator_details = did_mock::generate_mock_did_details(
		did::PublicVerificationKey::from(auth_key.public()),
		did::PublicEncryptionKey::from(enc_key),
	);

	let ctype_hash = ctype_mock::get_ctype_hash(true);
	let root_id = get_delegation_root_id(true);

	let operation = delegation::DelegationRootCreationOperation {
		creator_did: did_mock::ALICE_DID,
		ctype_hash: ctype_hash,
		root_id: root_id,
		tx_counter: delegator_details.get_tx_counter_value() + 1u64,
	};
	let signature = del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(did_mock::ALICE_DID, delegator_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash.clone(), did_mock::ALICE_DID)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_root_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::VerificationKeysNotPresent
		);
	});
}

#[test]
fn check_did_invalid_signature_format_submit_delegation_root_creation_operation() {
	let auth_key = did_mock::get_ed25519_authentication_key(true);
	let enc_key = did_mock::get_x25519_encryption_key(true);
	let del_key = did_mock::get_sr25519_delegation_key(true);
	let alternative_del_key = did_mock::get_ed25519_delegation_key(true);

	let mut delegator_details = did_mock::generate_mock_did_details(
		did::PublicVerificationKey::from(auth_key.public()),
		did::PublicEncryptionKey::from(enc_key),
	);
	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(del_key.public()));

	let ctype_hash = ctype_mock::get_ctype_hash(true);
	let root_id = get_delegation_root_id(true);

	let operation = delegation::DelegationRootCreationOperation {
		creator_did: did_mock::ALICE_DID,
		ctype_hash: ctype_hash,
		root_id: root_id,
		tx_counter: delegator_details.get_tx_counter_value() + 1u64,
	};
	let signature = alternative_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(did_mock::ALICE_DID, delegator_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash.clone(), did_mock::ALICE_DID)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_root_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidSignatureFormat
		);
	});
}

#[test]
fn check_did_invalid_signature_submit_delegation_root_creation_operation() {
	let auth_key = did_mock::get_ed25519_authentication_key(true);
	let enc_key = did_mock::get_x25519_encryption_key(true);
	let del_key = did_mock::get_sr25519_delegation_key(true);
	let wrong_del_key = did_mock::get_sr25519_delegation_key(false);

	let mut delegator_details = did_mock::generate_mock_did_details(
		did::PublicVerificationKey::from(auth_key.public()),
		did::PublicEncryptionKey::from(enc_key),
	);
	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(del_key.public()));

	let ctype_hash = ctype_mock::get_ctype_hash(true);
	let root_id = get_delegation_root_id(true);

	let operation = delegation::DelegationRootCreationOperation {
		creator_did: did_mock::ALICE_DID,
		ctype_hash: ctype_hash,
		root_id: root_id,
		tx_counter: delegator_details.get_tx_counter_value() + 1u64,
	};
	let signature = wrong_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(did_mock::ALICE_DID, delegator_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash.clone(), did_mock::ALICE_DID)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_root_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidSignature
		);
	});
}

#[test]
fn check_duplicate_submit_delegation_root_creation_operation() {
	let auth_key = did_mock::get_ed25519_authentication_key(true);
	let enc_key = did_mock::get_x25519_encryption_key(true);
	let del_key = did_mock::get_sr25519_delegation_key(true);

	let mut delegator_details = did_mock::generate_mock_did_details(
		did::PublicVerificationKey::from(auth_key.public()),
		did::PublicEncryptionKey::from(enc_key),
	);
	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(del_key.public()));

	let ctype_hash = ctype_mock::get_ctype_hash(true);
	let root_id = get_delegation_root_id(true);

	let operation = delegation::DelegationRootCreationOperation {
		creator_did: did_mock::ALICE_DID,
		ctype_hash: ctype_hash,
		root_id: root_id,
		tx_counter: delegator_details.get_tx_counter_value() + 1u64,
	};
	let signature = del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(did_mock::ALICE_DID, delegator_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash, did_mock::ALICE_DID)]);
	let builder = ExtBuilder::from(builder).with_root_delegations(vec![(
		root_id,
		delegation::DelegationRoot {
			ctype_hash: ctype_hash.clone(),
			owner: did_mock::ALICE_DID,
			revoked: false,
		},
	)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_root_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			delegation::Error::<Test>::RootAlreadyExists
		);
	});
}

#[test]
fn check_ctype_not_found_submit_delegation_root_creation_operation() {
	let auth_key = did_mock::get_ed25519_authentication_key(true);
	let enc_key = did_mock::get_x25519_encryption_key(true);
	let del_key = did_mock::get_sr25519_delegation_key(true);

	let mut delegator_details = did_mock::generate_mock_did_details(
		did::PublicVerificationKey::from(auth_key.public()),
		did::PublicEncryptionKey::from(enc_key),
	);
	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(del_key.public()));

	let ctype_hash = ctype_mock::get_ctype_hash(true);
	let alternative_ctype_hash = ctype_mock::get_ctype_hash(false);
	let root_id = get_delegation_root_id(true);

	let operation = delegation::DelegationRootCreationOperation {
		creator_did: did_mock::ALICE_DID,
		ctype_hash: alternative_ctype_hash,
		root_id: root_id,
		tx_counter: delegator_details.get_tx_counter_value() + 1u64,
	};
	let signature = del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(did_mock::ALICE_DID, delegator_details.clone())]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_root_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			ctype::Error::<Test>::CTypeNotFound
		);
	});
}

// submit_delegation_creation_operation()

#[test]
fn check_submit_delegation_no_parent_creation_operation_successful() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_enc_key = did_mock::get_x25519_encryption_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);
	let delegate_enc_key = did_mock::get_x25519_encryption_key(false);

	let mut delegator_details = did_mock::generate_mock_did_details(
		did::PublicVerificationKey::from(delegator_auth_key.public()),
		did::PublicEncryptionKey::from(delegator_enc_key),
	);
	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(delegator_del_key.public()));

	let delegate_details = did_mock::generate_mock_did_details(
		did::PublicVerificationKey::from(delegate_auth_key.public()),
		delegate_enc_key,
	);

	let ctype_hash = ctype_mock::get_ctype_hash(true);
	let root_id = get_delegation_root_id(true);
	let delegation_id = get_delegation_id(true);
	let permissions = delegation::Permissions::DELEGATE;
	let delegate_signature = delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
		&delegation_id,
		&root_id,
		&None,
		&permissions,
	)));

	let operation = delegation::DelegationCreationOperation {
		creator_did: did_mock::ALICE_DID,
		delegate_did: did_mock::BOB_DID,
		delegation_id: delegation_id,
		parent_id: None,
		permissions: permissions,
		root_id: root_id,
		delegate_signature: did::DidSignature::from(delegate_signature),
		tx_counter: delegator_details.get_tx_counter_value() + 1u64,
	};
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(did_mock::ALICE_DID, delegator_details.clone()),
		(did_mock::BOB_DID, delegate_details.clone()),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash, did_mock::ALICE_DID)]);
	let builder = ExtBuilder::from(builder).with_root_delegations(vec![(
		root_id,
		delegation::DelegationRoot {
			owner: did_mock::ALICE_DID,
			ctype_hash: ctype_hash,
			revoked: false,
		},
	)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_ok!(Delegation::submit_delegation_creation_operation(
			Origin::signed(DEFAULT_ACCOUNT),
			operation.clone(),
			did::DidSignature::from(signature)
		));
	});

	let stored_delegation = ext.execute_with(|| {
		Delegation::delegations(&operation.delegation_id).expect("Delegation should be present on chain.")
	});

	assert_eq!(stored_delegation.root_id, operation.root_id);
	assert_eq!(stored_delegation.parent, None);
	assert_eq!(stored_delegation.owner, operation.delegate_did);
	assert_eq!(stored_delegation.permissions, operation.permissions);
	assert_eq!(stored_delegation.revoked, false);

	// Verify that the root has the new delegation among its children
	let stored_root_children = ext.execute_with(|| {
		Delegation::children(&operation.root_id).expect("Delegation root children should be present on chain.")
	});

	assert_eq!(stored_root_children, vec![operation.delegation_id]);

	// Verify that the DID tx counter has increased
	let new_delegator_details = ext
		.execute_with(|| Did::get_did(&operation.creator_did).expect("Delegation creator should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value() + 1u64
	);
}

#[test]
fn check_submit_delegation_with_parent_creation_operation_successful() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_enc_key = did_mock::get_x25519_encryption_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);
	let delegate_enc_key = did_mock::get_x25519_encryption_key(false);

	let mut delegator_details = did_mock::generate_mock_did_details(
		did::PublicVerificationKey::from(delegator_auth_key.public()),
		did::PublicEncryptionKey::from(delegator_enc_key),
	);
	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(delegator_del_key.public()));

	let delegate_details = did_mock::generate_mock_did_details(
		did::PublicVerificationKey::from(delegate_auth_key.public()),
		delegate_enc_key,
	);

	let ctype_hash = ctype_mock::get_ctype_hash(true);
	let root_id = get_delegation_root_id(true);
	let parent_id = get_delegation_id(true);
	let delegation_id = get_delegation_id(false);
	let permissions = delegation::Permissions::DELEGATE | delegation::Permissions::ATTEST;
	let delegate_signature = delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
		&delegation_id,
		&root_id,
		&Some(parent_id),
		&permissions,
	)));

	let operation = delegation::DelegationCreationOperation {
		creator_did: did_mock::ALICE_DID,
		delegate_did: did_mock::BOB_DID,
		delegation_id: delegation_id,
		parent_id: Some(parent_id),
		permissions: permissions,
		root_id: root_id,
		delegate_signature: did::DidSignature::from(delegate_signature),
		tx_counter: delegator_details.get_tx_counter_value() + 1u64,
	};
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(did_mock::ALICE_DID, delegator_details.clone()),
		(did_mock::BOB_DID, delegate_details.clone()),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash, did_mock::ALICE_DID)]);
	let builder = ExtBuilder::from(builder)
		.with_root_delegations(vec![(
			root_id,
			delegation::DelegationRoot {
				owner: did_mock::ALICE_DID,
				ctype_hash: ctype_hash,
				revoked: false,
			},
		)])
		.with_delegations(vec![(
			parent_id,
			delegation::DelegationNode {
				owner: did_mock::ALICE_DID,
				parent: None,
				revoked: false,
				root_id: root_id,
				permissions: permissions
			}
		)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_ok!(Delegation::submit_delegation_creation_operation(
			Origin::signed(DEFAULT_ACCOUNT),
			operation.clone(),
			did::DidSignature::from(signature)
		));
	});

	let stored_delegation = ext.execute_with(|| {
		Delegation::delegations(&operation.delegation_id).expect("Delegation should be present on chain.")
	});

	assert_eq!(stored_delegation.root_id, operation.root_id);
	assert_eq!(stored_delegation.parent, operation.parent_id);
	assert_eq!(stored_delegation.owner, operation.delegate_did);
	assert_eq!(stored_delegation.permissions, operation.permissions);
	assert_eq!(stored_delegation.revoked, false);

	// Verify that the parent has the new delegation among its children
	let stored_parent_children = ext.execute_with(|| {
		Delegation::children(&operation.parent_id.unwrap()).expect("Delegation parent children should be present on chain.")
	});

	assert_eq!(stored_parent_children, vec![delegation_id]);

	// Verify that the DID tx counter has increased
	let new_delegator_details = ext
		.execute_with(|| Did::get_did(&operation.creator_did).expect("Delegation creator should be present on chain."));
	assert_eq!(
		new_delegator_details.get_tx_counter_value(),
		delegator_details.get_tx_counter_value() + 1u64
	);
}

#[test]
fn check_delegator_did_not_found_submit_delegation_with_parent_creation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_enc_key = did_mock::get_x25519_encryption_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);
	let delegate_enc_key = did_mock::get_x25519_encryption_key(false);

	let mut delegator_details = did_mock::generate_mock_did_details(
		did::PublicVerificationKey::from(delegator_auth_key.public()),
		did::PublicEncryptionKey::from(delegator_enc_key),
	);
	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(delegator_del_key.public()));

	let delegate_details = did_mock::generate_mock_did_details(
		did::PublicVerificationKey::from(delegate_auth_key.public()),
		delegate_enc_key,
	);

	let ctype_hash = ctype_mock::get_ctype_hash(true);
	let root_id = get_delegation_root_id(true);
	let parent_id = get_delegation_id(true);
	let delegation_id = get_delegation_id(false);
	let permissions = delegation::Permissions::DELEGATE | delegation::Permissions::ATTEST;
	let delegate_signature = delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
		&delegation_id,
		&root_id,
		&Some(parent_id),
		&permissions,
	)));

	let operation = delegation::DelegationCreationOperation {
		creator_did: did_mock::ALICE_DID,
		delegate_did: did_mock::BOB_DID,
		delegation_id: delegation_id,
		parent_id: Some(parent_id),
		permissions: permissions,
		root_id: root_id,
		delegate_signature: did::DidSignature::from(delegate_signature),
		tx_counter: delegator_details.get_tx_counter_value() + 1u64,
	};
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(did_mock::CHARLIE_DID, delegator_details.clone()),
		(did_mock::BOB_DID, delegate_details.clone()),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash, did_mock::CHARLIE_DID)]);
	let builder = ExtBuilder::from(builder)
		.with_root_delegations(vec![(
			root_id,
			delegation::DelegationRoot {
				owner: did_mock::CHARLIE_DID,
				ctype_hash: ctype_hash,
				revoked: false,
			},
		)])
		.with_delegations(vec![(
			parent_id,
			delegation::DelegationNode {
				owner: did_mock::CHARLIE_DID,
				parent: None,
				revoked: false,
				root_id: root_id,
				permissions: permissions
			}
		)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::DidNotPresent
		);
	});
}

#[test]
fn check_delegator_max_tx_counter_value_submit_delegation_with_parent_creation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_enc_key = did_mock::get_x25519_encryption_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);
	let delegate_enc_key = did_mock::get_x25519_encryption_key(false);

	let mut delegator_details = did_mock::generate_mock_did_details(
		did::PublicVerificationKey::from(delegator_auth_key.public()),
		did::PublicEncryptionKey::from(delegator_enc_key),
	);
	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(delegator_del_key.public()));
	delegator_details.set_tx_counter(u64::MAX);

	let delegate_details = did_mock::generate_mock_did_details(
		did::PublicVerificationKey::from(delegate_auth_key.public()),
		delegate_enc_key,
	);

	let ctype_hash = ctype_mock::get_ctype_hash(true);
	let root_id = get_delegation_root_id(true);
	let parent_id = get_delegation_id(true);
	let delegation_id = get_delegation_id(false);
	let permissions = delegation::Permissions::DELEGATE | delegation::Permissions::ATTEST;
	let delegate_signature = delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
		&delegation_id,
		&root_id,
		&Some(parent_id),
		&permissions,
	)));

	let operation = delegation::DelegationCreationOperation {
		creator_did: did_mock::ALICE_DID,
		delegate_did: did_mock::BOB_DID,
		delegation_id: delegation_id,
		parent_id: Some(parent_id),
		permissions: permissions,
		root_id: root_id,
		delegate_signature: did::DidSignature::from(delegate_signature),
		tx_counter: delegator_details.get_tx_counter_value(),
	};
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(did_mock::ALICE_DID, delegator_details.clone()),
		(did_mock::BOB_DID, delegate_details.clone()),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash, did_mock::ALICE_DID)]);
	let builder = ExtBuilder::from(builder)
		.with_root_delegations(vec![(
			root_id,
			delegation::DelegationRoot {
				owner: did_mock::ALICE_DID,
				ctype_hash: ctype_hash,
				revoked: false,
			},
		)])
		.with_delegations(vec![(
			parent_id,
			delegation::DelegationNode {
				owner: did_mock::ALICE_DID,
				parent: None,
				revoked: false,
				root_id: root_id,
				permissions: permissions
			}
		)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::MaxTxCounterValue
		);
	});
}

#[test]
fn check_delegator_too_small_tx_counter_submit_delegation_with_parent_creation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_enc_key = did_mock::get_x25519_encryption_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);
	let delegate_enc_key = did_mock::get_x25519_encryption_key(false);

	let mut delegator_details = did_mock::generate_mock_did_details(
		did::PublicVerificationKey::from(delegator_auth_key.public()),
		did::PublicEncryptionKey::from(delegator_enc_key),
	);
	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(delegator_del_key.public()));
	delegator_details.set_tx_counter(1u64);

	let delegate_details = did_mock::generate_mock_did_details(
		did::PublicVerificationKey::from(delegate_auth_key.public()),
		delegate_enc_key,
	);

	let ctype_hash = ctype_mock::get_ctype_hash(true);
	let root_id = get_delegation_root_id(true);
	let parent_id = get_delegation_id(true);
	let delegation_id = get_delegation_id(false);
	let permissions = delegation::Permissions::DELEGATE | delegation::Permissions::ATTEST;
	let delegate_signature = delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
		&delegation_id,
		&root_id,
		&Some(parent_id),
		&permissions,
	)));

	let operation = delegation::DelegationCreationOperation {
		creator_did: did_mock::ALICE_DID,
		delegate_did: did_mock::BOB_DID,
		delegation_id: delegation_id,
		parent_id: Some(parent_id),
		permissions: permissions,
		root_id: root_id,
		delegate_signature: did::DidSignature::from(delegate_signature),
		tx_counter: delegator_details.get_tx_counter_value() - 1u64,
	};
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(did_mock::ALICE_DID, delegator_details.clone()),
		(did_mock::BOB_DID, delegate_details.clone()),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash, did_mock::ALICE_DID)]);
	let builder = ExtBuilder::from(builder)
		.with_root_delegations(vec![(
			root_id,
			delegation::DelegationRoot {
				owner: did_mock::ALICE_DID,
				ctype_hash: ctype_hash,
				revoked: false,
			},
		)])
		.with_delegations(vec![(
			parent_id,
			delegation::DelegationNode {
				owner: did_mock::ALICE_DID,
				parent: None,
				revoked: false,
				root_id: root_id,
				permissions: permissions
			}
		)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidNonce
		);
	});
}

#[test]
fn check_delegator_equal_tx_counter_submit_delegation_with_parent_creation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_enc_key = did_mock::get_x25519_encryption_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);
	let delegate_enc_key = did_mock::get_x25519_encryption_key(false);

	let mut delegator_details = did_mock::generate_mock_did_details(
		did::PublicVerificationKey::from(delegator_auth_key.public()),
		did::PublicEncryptionKey::from(delegator_enc_key),
	);
	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(delegator_del_key.public()));

	let delegate_details = did_mock::generate_mock_did_details(
		did::PublicVerificationKey::from(delegate_auth_key.public()),
		delegate_enc_key,
	);

	let ctype_hash = ctype_mock::get_ctype_hash(true);
	let root_id = get_delegation_root_id(true);
	let parent_id = get_delegation_id(true);
	let delegation_id = get_delegation_id(false);
	let permissions = delegation::Permissions::DELEGATE | delegation::Permissions::ATTEST;
	let delegate_signature = delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
		&delegation_id,
		&root_id,
		&Some(parent_id),
		&permissions,
	)));

	let operation = delegation::DelegationCreationOperation {
		creator_did: did_mock::ALICE_DID,
		delegate_did: did_mock::BOB_DID,
		delegation_id: delegation_id,
		parent_id: Some(parent_id),
		permissions: permissions,
		root_id: root_id,
		delegate_signature: did::DidSignature::from(delegate_signature),
		tx_counter: delegator_details.get_tx_counter_value(),
	};
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(did_mock::ALICE_DID, delegator_details.clone()),
		(did_mock::BOB_DID, delegate_details.clone()),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash, did_mock::ALICE_DID)]);
	let builder = ExtBuilder::from(builder)
		.with_root_delegations(vec![(
			root_id,
			delegation::DelegationRoot {
				owner: did_mock::ALICE_DID,
				ctype_hash: ctype_hash,
				revoked: false,
			},
		)])
		.with_delegations(vec![(
			parent_id,
			delegation::DelegationNode {
				owner: did_mock::ALICE_DID,
				parent: None,
				revoked: false,
				root_id: root_id,
				permissions: permissions
			}
		)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidNonce
		);
	});
}

#[test]
fn check_delegator_too_large_tx_counter_submit_delegation_with_parent_creation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_enc_key = did_mock::get_x25519_encryption_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);
	let delegate_enc_key = did_mock::get_x25519_encryption_key(false);

	let mut delegator_details = did_mock::generate_mock_did_details(
		did::PublicVerificationKey::from(delegator_auth_key.public()),
		did::PublicEncryptionKey::from(delegator_enc_key),
	);
	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(delegator_del_key.public()));

	let delegate_details = did_mock::generate_mock_did_details(
		did::PublicVerificationKey::from(delegate_auth_key.public()),
		delegate_enc_key,
	);

	let ctype_hash = ctype_mock::get_ctype_hash(true);
	let root_id = get_delegation_root_id(true);
	let parent_id = get_delegation_id(true);
	let delegation_id = get_delegation_id(false);
	let permissions = delegation::Permissions::DELEGATE | delegation::Permissions::ATTEST;
	let delegate_signature = delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
		&delegation_id,
		&root_id,
		&Some(parent_id),
		&permissions,
	)));

	let operation = delegation::DelegationCreationOperation {
		creator_did: did_mock::ALICE_DID,
		delegate_did: did_mock::BOB_DID,
		delegation_id: delegation_id,
		parent_id: Some(parent_id),
		permissions: permissions,
		root_id: root_id,
		delegate_signature: did::DidSignature::from(delegate_signature),
		tx_counter: delegator_details.get_tx_counter_value() + 2u64,
	};
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(did_mock::ALICE_DID, delegator_details.clone()),
		(did_mock::BOB_DID, delegate_details.clone()),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash, did_mock::ALICE_DID)]);
	let builder = ExtBuilder::from(builder)
		.with_root_delegations(vec![(
			root_id,
			delegation::DelegationRoot {
				owner: did_mock::ALICE_DID,
				ctype_hash: ctype_hash,
				revoked: false,
			},
		)])
		.with_delegations(vec![(
			parent_id,
			delegation::DelegationNode {
				owner: did_mock::ALICE_DID,
				parent: None,
				revoked: false,
				root_id: root_id,
				permissions: permissions
			}
		)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidNonce
		);
	});
}

#[test]
fn check_delegator_delegation_key_not_present_submit_delegation_with_parent_creation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_enc_key = did_mock::get_x25519_encryption_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);
	let delegate_enc_key = did_mock::get_x25519_encryption_key(false);

	let mut delegator_details = did_mock::generate_mock_did_details(
		did::PublicVerificationKey::from(delegator_auth_key.public()),
		did::PublicEncryptionKey::from(delegator_enc_key),
	);

	let delegate_details = did_mock::generate_mock_did_details(
		did::PublicVerificationKey::from(delegate_auth_key.public()),
		delegate_enc_key,
	);

	let ctype_hash = ctype_mock::get_ctype_hash(true);
	let root_id = get_delegation_root_id(true);
	let parent_id = get_delegation_id(true);
	let delegation_id = get_delegation_id(false);
	let permissions = delegation::Permissions::DELEGATE | delegation::Permissions::ATTEST;
	let delegate_signature = delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
		&delegation_id,
		&root_id,
		&Some(parent_id),
		&permissions,
	)));

	let operation = delegation::DelegationCreationOperation {
		creator_did: did_mock::ALICE_DID,
		delegate_did: did_mock::BOB_DID,
		delegation_id: delegation_id,
		parent_id: Some(parent_id),
		permissions: permissions,
		root_id: root_id,
		delegate_signature: did::DidSignature::from(delegate_signature),
		tx_counter: delegator_details.get_tx_counter_value() + 1u64,
	};
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(did_mock::ALICE_DID, delegator_details.clone()),
		(did_mock::BOB_DID, delegate_details.clone()),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash, did_mock::ALICE_DID)]);
	let builder = ExtBuilder::from(builder)
		.with_root_delegations(vec![(
			root_id,
			delegation::DelegationRoot {
				owner: did_mock::ALICE_DID,
				ctype_hash: ctype_hash,
				revoked: false,
			},
		)])
		.with_delegations(vec![(
			parent_id,
			delegation::DelegationNode {
				owner: did_mock::ALICE_DID,
				parent: None,
				revoked: false,
				root_id: root_id,
				permissions: permissions
			}
		)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::VerificationKeysNotPresent
		);
	});
}

#[test]
fn check_delegator_invalid_signature_format_submit_delegation_with_parent_creation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_enc_key = did_mock::get_x25519_encryption_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegator_alternative_del_key = did_mock::get_ed25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);
	let delegate_enc_key = did_mock::get_x25519_encryption_key(false);

	let mut delegator_details = did_mock::generate_mock_did_details(
		did::PublicVerificationKey::from(delegator_auth_key.public()),
		did::PublicEncryptionKey::from(delegator_enc_key),
	);
	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(delegator_del_key.public()));

	let delegate_details = did_mock::generate_mock_did_details(
		did::PublicVerificationKey::from(delegate_auth_key.public()),
		delegate_enc_key,
	);

	let ctype_hash = ctype_mock::get_ctype_hash(true);
	let root_id = get_delegation_root_id(true);
	let parent_id = get_delegation_id(true);
	let delegation_id = get_delegation_id(false);
	let permissions = delegation::Permissions::DELEGATE | delegation::Permissions::ATTEST;
	let delegate_signature = delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
		&delegation_id,
		&root_id,
		&Some(parent_id),
		&permissions,
	)));

	let operation = delegation::DelegationCreationOperation {
		creator_did: did_mock::ALICE_DID,
		delegate_did: did_mock::BOB_DID,
		delegation_id: delegation_id,
		parent_id: Some(parent_id),
		permissions: permissions,
		root_id: root_id,
		delegate_signature: did::DidSignature::from(delegate_signature),
		tx_counter: delegator_details.get_tx_counter_value() + 1u64,
	};
	let signature = delegator_alternative_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(did_mock::ALICE_DID, delegator_details.clone()),
		(did_mock::BOB_DID, delegate_details.clone()),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash, did_mock::ALICE_DID)]);
	let builder = ExtBuilder::from(builder)
		.with_root_delegations(vec![(
			root_id,
			delegation::DelegationRoot {
				owner: did_mock::ALICE_DID,
				ctype_hash: ctype_hash,
				revoked: false,
			},
		)])
		.with_delegations(vec![(
			parent_id,
			delegation::DelegationNode {
				owner: did_mock::ALICE_DID,
				parent: None,
				revoked: false,
				root_id: root_id,
				permissions: permissions
			}
		)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidSignatureFormat
		);
	});
}

#[test]
fn check_delegator_invalid_signature_submit_delegation_with_parent_creation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_enc_key = did_mock::get_x25519_encryption_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegator_invalid_del_key = did_mock::get_sr25519_delegation_key(false);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);
	let delegate_enc_key = did_mock::get_x25519_encryption_key(false);

	let mut delegator_details = did_mock::generate_mock_did_details(
		did::PublicVerificationKey::from(delegator_auth_key.public()),
		did::PublicEncryptionKey::from(delegator_enc_key),
	);
	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(delegator_del_key.public()));

	let delegate_details = did_mock::generate_mock_did_details(
		did::PublicVerificationKey::from(delegate_auth_key.public()),
		delegate_enc_key,
	);

	let ctype_hash = ctype_mock::get_ctype_hash(true);
	let root_id = get_delegation_root_id(true);
	let parent_id = get_delegation_id(true);
	let delegation_id = get_delegation_id(false);
	let permissions = delegation::Permissions::DELEGATE | delegation::Permissions::ATTEST;
	let delegate_signature = delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
		&delegation_id,
		&root_id,
		&Some(parent_id),
		&permissions,
	)));

	let operation = delegation::DelegationCreationOperation {
		creator_did: did_mock::ALICE_DID,
		delegate_did: did_mock::BOB_DID,
		delegation_id: delegation_id,
		parent_id: Some(parent_id),
		permissions: permissions,
		root_id: root_id,
		delegate_signature: did::DidSignature::from(delegate_signature),
		tx_counter: delegator_details.get_tx_counter_value() + 1u64,
	};
	let signature = delegator_invalid_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(did_mock::ALICE_DID, delegator_details.clone()),
		(did_mock::BOB_DID, delegate_details.clone()),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash, did_mock::ALICE_DID)]);
	let builder = ExtBuilder::from(builder)
		.with_root_delegations(vec![(
			root_id,
			delegation::DelegationRoot {
				owner: did_mock::ALICE_DID,
				ctype_hash: ctype_hash,
				revoked: false,
			},
		)])
		.with_delegations(vec![(
			parent_id,
			delegation::DelegationNode {
				owner: did_mock::ALICE_DID,
				parent: None,
				revoked: false,
				root_id: root_id,
				permissions: permissions
			}
		)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			did::Error::<Test>::InvalidSignature
		);
	});
}

#[test]
fn check_delegate_did_not_found_submit_delegation_with_parent_creation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_enc_key = did_mock::get_x25519_encryption_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);
	let delegate_enc_key = did_mock::get_x25519_encryption_key(false);

	let mut delegator_details = did_mock::generate_mock_did_details(
		did::PublicVerificationKey::from(delegator_auth_key.public()),
		did::PublicEncryptionKey::from(delegator_enc_key),
	);
	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(delegator_del_key.public()));

	let delegate_details = did_mock::generate_mock_did_details(
		did::PublicVerificationKey::from(delegate_auth_key.public()),
		delegate_enc_key,
	);

	let ctype_hash = ctype_mock::get_ctype_hash(true);
	let root_id = get_delegation_root_id(true);
	let parent_id = get_delegation_id(true);
	let delegation_id = get_delegation_id(false);
	let permissions = delegation::Permissions::DELEGATE | delegation::Permissions::ATTEST;
	let delegate_signature = delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
		&delegation_id,
		&root_id,
		&Some(parent_id),
		&permissions,
	)));

	let operation = delegation::DelegationCreationOperation {
		creator_did: did_mock::ALICE_DID,
		delegate_did: did_mock::BOB_DID,
		delegation_id: delegation_id,
		parent_id: Some(parent_id),
		permissions: permissions,
		root_id: root_id,
		delegate_signature: did::DidSignature::from(delegate_signature),
		tx_counter: delegator_details.get_tx_counter_value() + 1u64,
	};
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(did_mock::ALICE_DID, delegator_details.clone()),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash, did_mock::ALICE_DID)]);
	let builder = ExtBuilder::from(builder)
		.with_root_delegations(vec![(
			root_id,
			delegation::DelegationRoot {
				owner: did_mock::ALICE_DID,
				ctype_hash: ctype_hash,
				revoked: false,
			},
		)])
		.with_delegations(vec![(
			parent_id,
			delegation::DelegationNode {
				owner: did_mock::ALICE_DID,
				parent: None,
				revoked: false,
				root_id: root_id,
				permissions: permissions
			}
		)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			delegation::Error::<Test>::DelegateNotFound
		);
	});
}


#[test]
fn check_invalid_delegate_signature_submit_delegation_with_parent_creation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_enc_key = did_mock::get_x25519_encryption_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);
	let delegate_invalid_auth_key = did_mock::get_sr25519_authentication_key(false);
	let delegate_enc_key = did_mock::get_x25519_encryption_key(false);

	let mut delegator_details = did_mock::generate_mock_did_details(
		did::PublicVerificationKey::from(delegator_auth_key.public()),
		did::PublicEncryptionKey::from(delegator_enc_key),
	);
	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(delegator_del_key.public()));

	let delegate_details = did_mock::generate_mock_did_details(
		did::PublicVerificationKey::from(delegate_auth_key.public()),
		delegate_enc_key,
	);

	let ctype_hash = ctype_mock::get_ctype_hash(true);
	let root_id = get_delegation_root_id(true);
	let delegation_id = get_delegation_id(true);
	let permissions = delegation::Permissions::DELEGATE;
	let delegate_signature = delegate_invalid_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
		&delegation_id,
		&root_id,
		&None,
		&permissions,
	)));

	let operation = delegation::DelegationCreationOperation {
		creator_did: did_mock::ALICE_DID,
		delegate_did: did_mock::BOB_DID,
		delegation_id: delegation_id,
		parent_id: None,
		permissions: permissions,
		root_id: root_id,
		delegate_signature: did::DidSignature::from(delegate_signature),
		tx_counter: delegator_details.get_tx_counter_value() + 1u64,
	};
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(did_mock::ALICE_DID, delegator_details.clone()),
		(did_mock::BOB_DID, delegate_details.clone()),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash, did_mock::ALICE_DID)]);
	let builder = ExtBuilder::from(builder).with_root_delegations(vec![(
		root_id,
		delegation::DelegationRoot {
			owner: did_mock::ALICE_DID,
			ctype_hash: ctype_hash,
			revoked: false,
		},
	)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			delegation::Error::<Test>::InvalidDelegateSignature
		);
	});
}

#[test]
fn check_duplicate_delegation_submit_delegation_with_parent_creation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_enc_key = did_mock::get_x25519_encryption_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);
	let delegate_enc_key = did_mock::get_x25519_encryption_key(false);

	let mut delegator_details = did_mock::generate_mock_did_details(
		did::PublicVerificationKey::from(delegator_auth_key.public()),
		did::PublicEncryptionKey::from(delegator_enc_key),
	);
	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(delegator_del_key.public()));

	let delegate_details = did_mock::generate_mock_did_details(
		did::PublicVerificationKey::from(delegate_auth_key.public()),
		delegate_enc_key,
	);

	let ctype_hash = ctype_mock::get_ctype_hash(true);
	let root_id = get_delegation_root_id(true);
	let delegation_id = get_delegation_id(true);
	let permissions = delegation::Permissions::DELEGATE;
	let delegate_signature = delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
		&delegation_id,
		&root_id,
		&None,
		&permissions,
	)));

	let operation = delegation::DelegationCreationOperation {
		creator_did: did_mock::ALICE_DID,
		delegate_did: did_mock::BOB_DID,
		delegation_id: delegation_id,
		parent_id: None,
		permissions: permissions,
		root_id: root_id,
		delegate_signature: did::DidSignature::from(delegate_signature),
		tx_counter: delegator_details.get_tx_counter_value() + 1u64,
	};
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(did_mock::ALICE_DID, delegator_details.clone()),
		(did_mock::BOB_DID, delegate_details.clone()),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash, did_mock::ALICE_DID)]);
	let builder = ExtBuilder::from(builder)
	.with_root_delegations(vec![(
		root_id,
		delegation::DelegationRoot {
			owner: did_mock::ALICE_DID,
			ctype_hash: ctype_hash,
			revoked: false,
		},
	)])
	.with_delegations(vec![(
		delegation_id,
		delegation::DelegationNode {
			owner: did_mock::ALICE_DID,
			parent: None,
			revoked: false,
			root_id: root_id,
			permissions: permissions
		}
	)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			delegation::Error::<Test>::DelegationAlreadyExists
		);
	});
}

#[test]
fn check_root_not_existing_submit_delegation_with_parent_creation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_enc_key = did_mock::get_x25519_encryption_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);
	let delegate_enc_key = did_mock::get_x25519_encryption_key(false);

	let mut delegator_details = did_mock::generate_mock_did_details(
		did::PublicVerificationKey::from(delegator_auth_key.public()),
		did::PublicEncryptionKey::from(delegator_enc_key),
	);
	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(delegator_del_key.public()));

	let delegate_details = did_mock::generate_mock_did_details(
		did::PublicVerificationKey::from(delegate_auth_key.public()),
		delegate_enc_key,
	);

	let ctype_hash = ctype_mock::get_ctype_hash(true);
	let root_id = get_delegation_root_id(true);
	let delegation_id = get_delegation_id(true);
	let permissions = delegation::Permissions::DELEGATE;
	let delegate_signature = delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
		&delegation_id,
		&root_id,
		&None,
		&permissions,
	)));

	let operation = delegation::DelegationCreationOperation {
		creator_did: did_mock::ALICE_DID,
		delegate_did: did_mock::BOB_DID,
		delegation_id: delegation_id,
		parent_id: None,
		permissions: permissions,
		root_id: root_id,
		delegate_signature: did::DidSignature::from(delegate_signature),
		tx_counter: delegator_details.get_tx_counter_value() + 1u64,
	};
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(did_mock::ALICE_DID, delegator_details.clone()),
		(did_mock::BOB_DID, delegate_details.clone()),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash, did_mock::ALICE_DID)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			delegation::Error::<Test>::RootNotFound
		);
	});
}

#[test]
fn check_parent_not_existing_submit_delegation_with_parent_creation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_enc_key = did_mock::get_x25519_encryption_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);
	let delegate_enc_key = did_mock::get_x25519_encryption_key(false);

	let mut delegator_details = did_mock::generate_mock_did_details(
		did::PublicVerificationKey::from(delegator_auth_key.public()),
		did::PublicEncryptionKey::from(delegator_enc_key),
	);
	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(delegator_del_key.public()));

	let delegate_details = did_mock::generate_mock_did_details(
		did::PublicVerificationKey::from(delegate_auth_key.public()),
		delegate_enc_key,
	);

	let ctype_hash = ctype_mock::get_ctype_hash(true);
	let root_id = get_delegation_root_id(true);
	let delegation_id = get_delegation_id(true);
	let parent_id = get_delegation_id(false);
	let permissions = delegation::Permissions::DELEGATE;
	let delegate_signature = delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
		&delegation_id,
		&root_id,
		&Some(parent_id),
		&permissions,
	)));

	let operation = delegation::DelegationCreationOperation {
		creator_did: did_mock::ALICE_DID,
		delegate_did: did_mock::BOB_DID,
		delegation_id: delegation_id,
		parent_id: Some(parent_id),
		permissions: permissions,
		root_id: root_id,
		delegate_signature: did::DidSignature::from(delegate_signature),
		tx_counter: delegator_details.get_tx_counter_value() + 1u64,
	};
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(did_mock::ALICE_DID, delegator_details.clone()),
		(did_mock::BOB_DID, delegate_details.clone()),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash, did_mock::ALICE_DID)]);
	let builder = ExtBuilder::from(builder)
	.with_root_delegations(vec![(
		root_id,
		delegation::DelegationRoot {
			owner: did_mock::ALICE_DID,
			ctype_hash: ctype_hash,
			revoked: false,
		},
	)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			delegation::Error::<Test>::ParentDelegationNotFound
		);
	});
}

#[test]
fn check_not_owner_of_parent_submit_delegation_with_parent_creation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_enc_key = did_mock::get_x25519_encryption_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);
	let delegate_enc_key = did_mock::get_x25519_encryption_key(false);

	let mut delegator_details = did_mock::generate_mock_did_details(
		did::PublicVerificationKey::from(delegator_auth_key.public()),
		did::PublicEncryptionKey::from(delegator_enc_key),
	);
	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(delegator_del_key.public()));

	let delegate_details = did_mock::generate_mock_did_details(
		did::PublicVerificationKey::from(delegate_auth_key.public()),
		delegate_enc_key,
	);

	let ctype_hash = ctype_mock::get_ctype_hash(true);
	let root_id = get_delegation_root_id(true);
	let parent_id = get_delegation_id(true);
	let delegation_id = get_delegation_id(false);
	let permissions = delegation::Permissions::DELEGATE;
	let delegate_signature = delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
		&delegation_id,
		&root_id,
		&Some(parent_id),
		&permissions,
	)));

	let operation = delegation::DelegationCreationOperation {
		creator_did: did_mock::ALICE_DID,
		delegate_did: did_mock::BOB_DID,
		delegation_id,
		parent_id: Some(parent_id),
		permissions: permissions,
		root_id: root_id,
		delegate_signature: did::DidSignature::from(delegate_signature),
		tx_counter: delegator_details.get_tx_counter_value() + 1u64,
	};
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(did_mock::ALICE_DID, delegator_details.clone()),
		(did_mock::BOB_DID, delegate_details.clone()),
		(did_mock::CHARLIE_DID, delegate_details.clone()),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash, did_mock::ALICE_DID)]);
	let builder = ExtBuilder::from(builder)
	.with_root_delegations(vec![(
		root_id,
		delegation::DelegationRoot {
			owner: did_mock::ALICE_DID,
			ctype_hash: ctype_hash,
			revoked: false,
		},
	)])
	.with_delegations(vec![(
		parent_id,
		delegation::DelegationNode {
			owner: did_mock::CHARLIE_DID,
			parent: None,
			revoked: false,
			root_id: root_id,
			permissions: permissions
		}
	)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			delegation::Error::<Test>::NotOwnerOfParentDelegation
		);
	});
}

#[test]
fn check_unauthorised_delegation_submit_delegation_with_parent_creation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_enc_key = did_mock::get_x25519_encryption_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);
	let delegate_enc_key = did_mock::get_x25519_encryption_key(false);

	let mut delegator_details = did_mock::generate_mock_did_details(
		did::PublicVerificationKey::from(delegator_auth_key.public()),
		did::PublicEncryptionKey::from(delegator_enc_key),
	);
	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(delegator_del_key.public()));

	let delegate_details = did_mock::generate_mock_did_details(
		did::PublicVerificationKey::from(delegate_auth_key.public()),
		delegate_enc_key,
	);

	let ctype_hash = ctype_mock::get_ctype_hash(true);
	let root_id = get_delegation_root_id(true);
	let parent_id = get_delegation_id(true);
	let delegation_id = get_delegation_id(false);
	let permissions = delegation::Permissions::DELEGATE;
	let delegate_signature = delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
		&delegation_id,
		&root_id,
		&Some(parent_id),
		&permissions,
	)));

	let operation = delegation::DelegationCreationOperation {
		creator_did: did_mock::ALICE_DID,
		delegate_did: did_mock::BOB_DID,
		delegation_id,
		parent_id: Some(parent_id),
		permissions: permissions,
		root_id: root_id,
		delegate_signature: did::DidSignature::from(delegate_signature),
		tx_counter: delegator_details.get_tx_counter_value() + 1u64,
	};
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(did_mock::ALICE_DID, delegator_details.clone()),
		(did_mock::BOB_DID, delegate_details.clone()),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash, did_mock::ALICE_DID)]);
	let builder = ExtBuilder::from(builder)
	.with_root_delegations(vec![(
		root_id,
		delegation::DelegationRoot {
			owner: did_mock::ALICE_DID,
			ctype_hash: ctype_hash,
			revoked: false,
		},
	)])
	.with_delegations(vec![(
		parent_id,
		delegation::DelegationNode {
			owner: did_mock::ALICE_DID,
			parent: None,
			revoked: false,
			root_id: root_id,
			permissions: delegation::Permissions::ATTEST
		}
	)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			delegation::Error::<Test>::UnauthorizedDelegation
		);
	});
}

#[test]
fn check_not_owner_of_root_delegation_submit_delegation_with_parent_creation_operation() {
	let delegator_auth_key = did_mock::get_ed25519_authentication_key(true);
	let delegator_enc_key = did_mock::get_x25519_encryption_key(true);
	let delegator_del_key = did_mock::get_sr25519_delegation_key(true);
	let delegate_auth_key = did_mock::get_sr25519_authentication_key(true);
	let delegate_enc_key = did_mock::get_x25519_encryption_key(false);

	let mut delegator_details = did_mock::generate_mock_did_details(
		did::PublicVerificationKey::from(delegator_auth_key.public()),
		did::PublicEncryptionKey::from(delegator_enc_key),
	);
	delegator_details.delegation_key = Some(did::PublicVerificationKey::from(delegator_del_key.public()));

	let delegate_details = did_mock::generate_mock_did_details(
		did::PublicVerificationKey::from(delegate_auth_key.public()),
		delegate_enc_key,
	);

	let ctype_hash = ctype_mock::get_ctype_hash(true);
	let root_id = get_delegation_root_id(true);
	let delegation_id = get_delegation_id(false);
	let permissions = delegation::Permissions::DELEGATE;
	let delegate_signature = delegate_auth_key.sign(&hash_to_u8(Delegation::calculate_hash(
		&delegation_id,
		&root_id,
		&None,
		&permissions,
	)));

	let operation = delegation::DelegationCreationOperation {
		creator_did: did_mock::ALICE_DID,
		delegate_did: did_mock::BOB_DID,
		delegation_id,
		parent_id: None,
		permissions: permissions,
		root_id: root_id,
		delegate_signature: did::DidSignature::from(delegate_signature),
		tx_counter: delegator_details.get_tx_counter_value() + 1u64,
	};
	let signature = delegator_del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![
		(did_mock::ALICE_DID, delegator_details.clone()),
		(did_mock::BOB_DID, delegate_details.clone()),
	]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash, did_mock::ALICE_DID)]);
	let builder = ExtBuilder::from(builder)
	.with_root_delegations(vec![(
		root_id,
		delegation::DelegationRoot {
			owner: did_mock::BOB_DID,
			ctype_hash: ctype_hash,
			revoked: false,
		},
	)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_noop!(
			Delegation::submit_delegation_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			),
			delegation::Error::<Test>::NotOwnerOfRootDelegation
		);
	});
}
