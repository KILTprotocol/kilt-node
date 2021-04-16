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
use did::mock as did_mock;
use ctype::mock as ctype_mock;
use frame_support::{assert_noop, assert_ok};
use sp_core::Pair;

use codec::Encode;

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
		tx_counter: delegator_details.get_tx_counter_value() + 1u64
	};
	let signature = del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(did_mock::ALICE_DID, delegator_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash, did_mock::ALICE_DID)]);

	let mut ext = builder.build();

	ext.execute_with(|| {
		assert_ok!(
			Delegation::submit_delegation_root_creation_operation(
				Origin::signed(DEFAULT_ACCOUNT),
				operation.clone(),
				did::DidSignature::from(signature)
			)
		);
	});

	let stored_delegation_root = ext.execute_with(|| {
		Delegation::roots(&operation.root_id).expect("Delegation root should be present on chain.")
	});

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
		tx_counter: delegator_details.get_tx_counter_value() + 1u64
	};
	let signature = del_key.sign(&operation.encode());

	let builder = did_mock::ExtBuilder::default().with_dids(vec![(did_mock::ALICE_DID, delegator_details.clone())]);
	let builder = ctype_mock::ExtBuilder::from(builder).with_ctypes(vec![(ctype_hash, did_mock::ALICE_DID)]);
	let builder = ExtBuilder::from(builder).with_root_delegations(vec![(root_id, delegation::DelegationRoot {
		ctype_hash: ctype_hash.clone(),
		owner: did_mock::ALICE_DID,
		revoked: false
	})]);

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
		tx_counter: delegator_details.get_tx_counter_value() + 1u64
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
			ctype::Error::<Test>::NotFound
		);
	});
}

// #[test]
// fn check_add_and_revoke_delegations() {
// 	new_test_ext().execute_with(|| {
// 		let pair_alice = ed25519::Pair::from_seed(&*b"Alice                           ");
// 		let account_hash_alice = MultiSigner::from(pair_alice.public()).into_account();
// 		let pair_bob = ed25519::Pair::from_seed(&*b"Bob                             ");
// 		let account_hash_bob = MultiSigner::from(pair_bob.public()).into_account();
// 		let pair_charlie = ed25519::Pair::from_seed(&*b"Charlie                         ");
// 		let account_hash_charlie = MultiSigner::from(pair_charlie.public()).into_account();

// 		let ctype_hash = H256::from_low_u64_be(1);
// 		let id_level_0 = H256::from_low_u64_be(1);
// 		let id_level_1 = H256::from_low_u64_be(2);
// 		let id_level_2_1 = H256::from_low_u64_be(21);
// 		let id_level_2_2 = H256::from_low_u64_be(22);
// 		let id_level_2_2_1 = H256::from_low_u64_be(221);

// 		assert_ok!(Delegation::add_delegation(
// 			Origin::signed(account_hash_alice.clone()),
// 			id_level_1,
// 			id_level_0,
// 			None,
// 			account_hash_bob.clone(),
// 			Permissions::DELEGATE,
// 			MultiSignature::from(pair_bob.sign(&hash_to_u8(Delegation::calculate_hash(
// 				id_level_1,
// 				id_level_0,
// 				None,
// 				Permissions::DELEGATE
// 			))))
// 		));
// 		assert_noop!(
// 			Delegation::add_delegation(
// 				Origin::signed(account_hash_alice.clone()),
// 				id_level_1,
// 				id_level_0,
// 				None,
// 				account_hash_bob.clone(),
// 				Permissions::DELEGATE,
// 				MultiSignature::from(pair_bob.sign(&hash_to_u8(Delegation::calculate_hash(
// 					id_level_1,
// 					id_level_0,
// 					None,
// 					Permissions::DELEGATE
// 				))))
// 			),
// 			Error::<Test>::AlreadyExists
// 		);
// 		assert_noop!(
// 			Delegation::add_delegation(
// 				Origin::signed(account_hash_bob.clone()),
// 				id_level_2_1,
// 				id_level_0,
// 				Some(id_level_1),
// 				account_hash_charlie.clone(),
// 				Permissions::ATTEST,
// 				MultiSignature::from(ed25519::Signature::from_h512(H512::from_low_u64_be(0)))
// 			),
// 			Error::<Test>::BadSignature,
// 		);
// 		assert_noop!(
// 			Delegation::add_delegation(
// 				Origin::signed(account_hash_charlie.clone()),
// 				id_level_2_1,
// 				id_level_0,
// 				None,
// 				account_hash_bob.clone(),
// 				Permissions::DELEGATE,
// 				MultiSignature::from(pair_bob.sign(&hash_to_u8(Delegation::calculate_hash(
// 					id_level_2_1,
// 					id_level_0,
// 					None,
// 					Permissions::DELEGATE
// 				))))
// 			),
// 			Error::<Test>::NotOwnerOfRoot,
// 		);
// 		assert_noop!(
// 			Delegation::add_delegation(
// 				Origin::signed(account_hash_alice.clone()),
// 				id_level_2_1,
// 				id_level_1,
// 				None,
// 				account_hash_bob.clone(),
// 				Permissions::DELEGATE,
// 				MultiSignature::from(pair_bob.sign(&hash_to_u8(Delegation::calculate_hash(
// 					id_level_2_1,
// 					id_level_1,
// 					None,
// 					Permissions::DELEGATE
// 				))))
// 			),
// 			Error::<Test>::RootNotFound
// 		);

// 		assert_ok!(Delegation::add_delegation(
// 			Origin::signed(account_hash_bob.clone()),
// 			id_level_2_1,
// 			id_level_0,
// 			Some(id_level_1),
// 			account_hash_charlie.clone(),
// 			Permissions::ATTEST,
// 			MultiSignature::from(pair_charlie.sign(&hash_to_u8(Delegation::calculate_hash(
// 				id_level_2_1,
// 				id_level_0,
// 				Some(id_level_1),
// 				Permissions::ATTEST
// 			))))
// 		));
// 		assert_noop!(
// 			Delegation::add_delegation(
// 				Origin::signed(account_hash_alice.clone()),
// 				id_level_2_2,
// 				id_level_0,
// 				Some(id_level_1),
// 				account_hash_charlie.clone(),
// 				Permissions::ATTEST,
// 				MultiSignature::from(pair_charlie.sign(&hash_to_u8(Delegation::calculate_hash(
// 					id_level_2_2,
// 					id_level_0,
// 					Some(id_level_1),
// 					Permissions::ATTEST
// 				))))
// 			),
// 			Error::<Test>::NotOwnerOfParent
// 		);
// 		assert_noop!(
// 			Delegation::add_delegation(
// 				Origin::signed(account_hash_charlie.clone()),
// 				id_level_2_2,
// 				id_level_0,
// 				Some(id_level_2_1),
// 				account_hash_alice.clone(),
// 				Permissions::ATTEST,
// 				MultiSignature::from(pair_alice.sign(&hash_to_u8(Delegation::calculate_hash(
// 					id_level_2_2,
// 					id_level_0,
// 					Some(id_level_2_1),
// 					Permissions::ATTEST
// 				))))
// 			),
// 			Error::<Test>::UnauthorizedDelegation
// 		);
// 		assert_noop!(
// 			Delegation::add_delegation(
// 				Origin::signed(account_hash_bob.clone()),
// 				id_level_2_2,
// 				id_level_0,
// 				Some(id_level_0),
// 				account_hash_charlie.clone(),
// 				Permissions::ATTEST,
// 				MultiSignature::from(pair_charlie.sign(&hash_to_u8(Delegation::calculate_hash(
// 					id_level_2_2,
// 					id_level_0,
// 					Some(id_level_0),
// 					Permissions::ATTEST
// 				))))
// 			),
// 			Error::<Test>::ParentNotFound
// 		);

// 		assert_ok!(Delegation::add_delegation(
// 			Origin::signed(account_hash_bob.clone()),
// 			id_level_2_2,
// 			id_level_0,
// 			Some(id_level_1),
// 			account_hash_charlie.clone(),
// 			Permissions::ATTEST | Permissions::DELEGATE,
// 			MultiSignature::from(pair_charlie.sign(&hash_to_u8(Delegation::calculate_hash(
// 				id_level_2_2,
// 				id_level_0,
// 				Some(id_level_1),
// 				Permissions::ATTEST | Permissions::DELEGATE
// 			))))
// 		));

// 		assert_ok!(Delegation::add_delegation(
// 			Origin::signed(account_hash_charlie.clone()),
// 			id_level_2_2_1,
// 			id_level_0,
// 			Some(id_level_2_2),
// 			account_hash_alice.clone(),
// 			Permissions::ATTEST,
// 			MultiSignature::from(pair_alice.sign(&hash_to_u8(Delegation::calculate_hash(
// 				id_level_2_2_1,
// 				id_level_0,
// 				Some(id_level_2_2),
// 				Permissions::ATTEST
// 			))))
// 		));

// 		let root = {
// 			let opt = Delegation::root(id_level_0);
// 			assert!(opt.is_some());
// 			opt.unwrap()
// 		};
// 		assert_eq!(root.ctype_hash, ctype_hash);
// 		assert_eq!(root.owner, account_hash_alice);
// 		assert_eq!(root.revoked, false);

// 		let delegation_1 = {
// 			let opt = Delegation::delegation(id_level_1);
// 			assert!(opt.is_some());
// 			opt.unwrap()
// 		};
// 		assert_eq!(delegation_1.root_id, id_level_0);
// 		assert_eq!(delegation_1.parent, None);
// 		assert_eq!(delegation_1.owner, account_hash_bob);
// 		assert_eq!(delegation_1.permissions, Permissions::DELEGATE);
// 		assert_eq!(delegation_1.revoked, false);

// 		let delegation_2 = {
// 			let opt = Delegation::delegation(id_level_2_2);
// 			assert!(opt.is_some());
// 			opt.unwrap()
// 		};
// 		assert_eq!(delegation_2.root_id, id_level_0);
// 		assert_eq!(delegation_2.parent, Some(id_level_1));
// 		assert_eq!(delegation_2.owner, account_hash_charlie);
// 		assert_eq!(delegation_2.permissions, Permissions::ATTEST | Permissions::DELEGATE);
// 		assert_eq!(delegation_2.revoked, false);

// 		let children = Delegation::children(id_level_1);
// 		assert_eq!(children.len(), 2);
// 		assert_eq!(children[0], id_level_2_1);
// 		assert_eq!(children[1], id_level_2_2);

// 		// check is_delgating
// 		assert_eq!(Delegation::is_delegating(&account_hash_alice, &id_level_1, 3), Ok(true));
// 		assert_eq!(
// 			Delegation::is_delegating(&account_hash_alice, &id_level_2_1, 3),
// 			Ok(true)
// 		);
// 		assert_eq!(Delegation::is_delegating(&account_hash_bob, &id_level_2_1, 3), Ok(true));
// 		assert_eq!(
// 			Delegation::is_delegating(&account_hash_charlie, &id_level_2_1, 1),
// 			Ok(true)
// 		);
// 		let res = Delegation::is_delegating(&account_hash_charlie, &id_level_0, 1);
// 		assert!(res.is_err(), "Expected error got {:?}", res);
// 		assert_eq!(
// 			Delegation::is_delegating(&account_hash_charlie, &id_level_1, 3),
// 			Ok(false)
// 		);
// 		assert_noop!(
// 			Delegation::is_delegating(&account_hash_charlie, &id_level_0, 3),
// 			Error::<Test>::DelegationNotFound
// 		);
// 		assert_noop!(
// 			Delegation::revoke_delegation(
// 				Origin::signed(account_hash_charlie.clone()),
// 				H256::from_low_u64_be(999),
// 				10,
// 				1
// 			),
// 			Error::<Test>::DelegationNotFound
// 		);
// 		assert_noop!(
// 			Delegation::revoke_delegation(Origin::signed(account_hash_charlie.clone()), id_level_1, 10, 1),
// 			Error::<Test>::UnauthorizedRevocation,
// 		);
// 		assert_ok!(Delegation::revoke_delegation(
// 			Origin::signed(account_hash_charlie),
// 			id_level_2_2,
// 			10,
// 			2
// 		));

// 		assert_eq!(Delegation::delegation(id_level_2_2).unwrap().revoked, true);
// 		assert_eq!(Delegation::delegation(id_level_2_2_1).unwrap().revoked, true);
// 		assert_noop!(
// 			Delegation::revoke_root(Origin::signed(account_hash_bob.clone()), H256::from_low_u64_be(999), 1),
// 			Error::<Test>::RootNotFound
// 		);
// 		assert_noop!(
// 			Delegation::revoke_root(Origin::signed(account_hash_bob), id_level_0, 1),
// 			Error::<Test>::UnauthorizedRevocation,
// 		);
// 		assert_noop!(
// 			Delegation::revoke_root(Origin::signed(account_hash_alice.clone()), id_level_0, 0),
// 			crate::Error::<Test>::ExceededRevocationBounds,
// 		);

// 		assert_ok!(Delegation::revoke_root(
// 			Origin::signed(account_hash_alice),
// 			id_level_0,
// 			3
// 		));
// 		assert_eq!(Delegation::root(id_level_0).unwrap().revoked, true);
// 		assert_eq!(Delegation::delegation(id_level_1).unwrap().revoked, true);
// 		assert_eq!(Delegation::delegation(id_level_2_1).unwrap().revoked, true);
// 	});
// }
