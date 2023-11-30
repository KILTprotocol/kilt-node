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

use attestation::{
	mock::{claim_hash_from_seed, generate_base_attestation, sr25519_did_from_public_key},
	Attestations, AttesterOf,
};
use ctype::mock::get_ctype_hash;
use delegation::{
	mock::{
		delegation_id_from_seed, generate_base_delegation_hierarchy_details, generate_base_delegation_node,
		get_delegation_hierarchy_id,
	},
	DelegationNodes,
};
use did::{
	did_details::DidVerificationKey,
	mock_utils::{generate_base_did_creation_details, generate_base_did_details},
};
use frame_support::{
	assert_ok, assert_storage_noop,
	traits::{
		fungible::{Inspect, InspectHold},
		tokens::{Fortitude, Preservation},
	},
};
use kilt_support::mock::mock_origin::{self, DoubleOrigin};
use pallet_did_lookup::{
	associate_account_request::{get_challenge, AssociateAccountRequest},
	linkable_account::LinkableAccountId,
	ConnectedDids,
};
use pallet_web3_names::{web3_name::AsciiWeb3Name, Owner};
use parity_scale_codec::Encode;
use public_credentials::{
	mock::{generate_base_credential_entry, generate_base_public_credential_creation_op, generate_credential_id},
	CredentialIdOf, Credentials, InputClaimsContentOf,
};
use sp_core::{ed25519, sr25519, Pair};
use sp_runtime::{traits::IdentifyAccount, BoundedVec, MultiSignature, MultiSigner};

use crate::{mock::*, EntriesToMigrate, MigratedKeys, Pallet};

#[test]
fn check_succesful_migration() {
	// attestaion
	let attester: AttesterOf<Test> = sr25519_did_from_public_key(&ALICE_SEED);
	let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_12);
	let mut attestation = generate_base_attestation::<Test>(attester.clone(), ACCOUNT_00);
	attestation.deposit.amount = MICRO_KILT;

	// delegation
	let creator = sr25519_did_from_public_key(&BOB_SEED);
	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details::<Test>();
	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_3);
	let mut parent_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, creator.clone(), Some(hierarchy_root_id), ACCOUNT_00);
	parent_node.deposit.amount = MICRO_KILT;

	//did
	let auth_key = ed25519::Pair::from_seed(&ALICE_SEED);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let mut did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(ACCOUNT_00));
	did_details.deposit.amount = MICRO_KILT;

	//w3n
	let web3_name_00 = AsciiWeb3Name::try_from(WEB3_NAME_00_INPUT.to_vec()).expect("W3n name creation should not fail");

	// public credentials
	let deposit = kilt_support::Deposit {
		owner: ACCOUNT_00,
		amount: MICRO_KILT,
	};
	let subject_id: <Test as public_credentials::Config>::SubjectId = SUBJECT_ID_00;
	let new_credential = generate_base_credential_entry::<Test>(
		ACCOUNT_00,
		0,
		attester.clone(),
		Some(attestation.ctype_hash),
		Some(deposit),
	);
	let credential_id: CredentialIdOf<Test> = CredentialIdOf::<Test>::default();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, KILT)])
		.with_ctypes(vec![(attestation.ctype_hash, attester)])
		.with_attestations(vec![(claim_hash, attestation)])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, creator, ACCOUNT_00)])
		.with_delegations(vec![(parent_id, parent_node)])
		.with_dids(vec![(alice_did.clone(), did_details)])
		.with_connections(vec![(ACCOUNT_00, DID_00, LINKABLE_ACCOUNT_00)])
		.with_web3_names(vec![(DID_00, web3_name_00.clone(), ACCOUNT_00)])
		.with_public_credentials(vec![(subject_id, credential_id, new_credential)])
		.build()
		.execute_with(|| {
			let count = MigratedKeys::<Test>::iter().count() as u32;
			let cursor = MigratedKeys::<Test>::clear(count, None).maybe_cursor;

			assert!(cursor.is_none());

			translate_all_holds_to_reserves();
			// pre migration checks
			let count_migrated_entries_pre_migration = MigratedKeys::<Test>::iter().count();
			assert_eq!(count_migrated_entries_pre_migration, 0);

			let free_balance_pre_migration = pallet_balances::Pallet::<Test>::reducible_balance(
				&ACCOUNT_00,
				Preservation::Protect,
				Fortitude::Polite,
			);

			assert!(free_balance_pre_migration < KILT);

			let attestation =
				BoundedVec::try_from([claim_hash].to_vec()).expect("Vec init should not fail for attestaions");

			let delegation =
				BoundedVec::try_from([parent_id].to_vec()).expect("Vec init should not fail for delegations");

			let did = BoundedVec::try_from([alice_did].to_vec()).expect("Vec init should not fail for did");

			let lookup =
				BoundedVec::try_from([LINKABLE_ACCOUNT_00].to_vec()).expect("Vec init should not fail for lookup");

			let w3n = BoundedVec::try_from([web3_name_00].to_vec()).expect("Vec init should not fail for w3n");

			let public_credentials = BoundedVec::try_from([(subject_id, credential_id)].to_vec())
				.expect("Vec init should not fail for public_credentials");

			let requested_migrations = EntriesToMigrate {
				attestation,
				delegation,
				did,
				lookup,
				w3n,
				public_credentials,
			};

			assert_ok!(Migration::update_balance(
				RuntimeOrigin::signed(ACCOUNT_00),
				requested_migrations
			));

			// post migration checks

			let count_migrated_entries_post_migration = MigratedKeys::<Test>::iter().count();
			assert_eq!(count_migrated_entries_post_migration, 6);

			// the free balance should be the same

			let free_balance_post_migration = pallet_balances::Pallet::<Test>::reducible_balance(
				&ACCOUNT_00,
				Preservation::Protect,
				Fortitude::Polite,
			);

			assert_eq!(free_balance_post_migration, free_balance_pre_migration);

			// The deposits should be holds now

			let attestaion_deposit =
				pallet_balances::Pallet::<Test>::balance_on_hold(&attestation::HoldReason::Deposit.into(), &ACCOUNT_00);

			let did_deposit =
				pallet_balances::Pallet::<Test>::balance_on_hold(&did::HoldReason::Deposit.into(), &ACCOUNT_00);

			let delegation_deposit =
				pallet_balances::Pallet::<Test>::balance_on_hold(&delegation::HoldReason::Deposit.into(), &ACCOUNT_00);

			let w3n_deposit = pallet_balances::Pallet::<Test>::balance_on_hold(
				&pallet_web3_names::HoldReason::Deposit.into(),
				&ACCOUNT_00,
			);

			let public_credentials_deposit = pallet_balances::Pallet::<Test>::balance_on_hold(
				&public_credentials::HoldReason::Deposit.into(),
				&ACCOUNT_00,
			);

			let did_lookup_deposit = pallet_balances::Pallet::<Test>::balance_on_hold(
				&pallet_did_lookup::HoldReason::Deposit.into(),
				&ACCOUNT_00,
			);

			assert_eq!(attestaion_deposit, MICRO_KILT);
			assert_eq!(did_deposit, MICRO_KILT);
			assert_eq!(delegation_deposit, MICRO_KILT);
			assert_eq!(w3n_deposit, MICRO_KILT);
			assert_eq!(public_credentials_deposit, MICRO_KILT);
			assert_eq!(did_lookup_deposit, MICRO_KILT);
		});
}

#[test]
fn check_attempt_to_migrate_already_migrated_keys() {
	// attestaion
	let attester: AttesterOf<Test> = sr25519_did_from_public_key(&ALICE_SEED);
	let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_12);
	let mut attestation = generate_base_attestation::<Test>(attester.clone(), ACCOUNT_00);
	attestation.deposit.amount = MICRO_KILT;

	// delegation
	let creator = sr25519_did_from_public_key(&BOB_SEED);
	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details::<Test>();
	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_3);
	let mut parent_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, creator.clone(), Some(hierarchy_root_id), ACCOUNT_00);
	parent_node.deposit.amount = MICRO_KILT;

	//did
	let auth_key = ed25519::Pair::from_seed(&ALICE_SEED);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let mut did_details =
		generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(ACCOUNT_00));
	did_details.deposit.amount = MICRO_KILT;

	//w3n
	let web3_name_00 = AsciiWeb3Name::try_from(WEB3_NAME_00_INPUT.to_vec()).expect("W3n name creation should not fail");

	// public credentials
	let deposit = kilt_support::Deposit {
		owner: ACCOUNT_00,
		amount: MICRO_KILT,
	};
	let subject_id: <Test as public_credentials::Config>::SubjectId = SUBJECT_ID_00;
	let new_credential = generate_base_credential_entry::<Test>(
		ACCOUNT_00,
		0,
		attester.clone(),
		Some(attestation.ctype_hash),
		Some(deposit),
	);
	let credential_id: CredentialIdOf<Test> = CredentialIdOf::<Test>::default();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, KILT)])
		.with_ctypes(vec![(attestation.ctype_hash, attester)])
		.with_attestations(vec![(claim_hash, attestation)])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, creator, ACCOUNT_00)])
		.with_delegations(vec![(parent_id, parent_node)])
		.with_dids(vec![(alice_did.clone(), did_details)])
		.with_connections(vec![(ACCOUNT_00, DID_00, LINKABLE_ACCOUNT_00)])
		.with_web3_names(vec![(DID_00, web3_name_00.clone(), ACCOUNT_00)])
		.with_public_credentials(vec![(subject_id, credential_id, new_credential)])
		.build()
		.execute_with(|| {
			let attestation =
				BoundedVec::try_from([claim_hash].to_vec()).expect("Vec init should not fail for attestaions");

			let delegation =
				BoundedVec::try_from([parent_id].to_vec()).expect("Vec init should not fail for delegations");

			let did = BoundedVec::try_from([alice_did].to_vec()).expect("Vec init should not fail for did");

			let lookup =
				BoundedVec::try_from([LINKABLE_ACCOUNT_00].to_vec()).expect("Vec init should not fail for lookup");

			let w3n = BoundedVec::try_from([web3_name_00].to_vec()).expect("Vec init should not fail for w3n");

			let public_credentials = BoundedVec::try_from([(subject_id, credential_id)].to_vec())
				.expect("Vec init should not fail for public_credentials");

			let requested_migrations = EntriesToMigrate {
				attestation,
				delegation,
				did,
				lookup,
				w3n,
				public_credentials,
			};

			assert_ok!(Migration::update_balance(
				RuntimeOrigin::signed(ACCOUNT_00),
				requested_migrations.clone()
			));

			// Since the keys are already migrated, a second attempt should have not affect to the storage.
			assert_storage_noop!(
				Migration::update_balance(RuntimeOrigin::signed(ACCOUNT_00), requested_migrations)
					.expect("Update balance should not panic")
			);
		});
}

#[test]
fn check_excluded_keys_attestation() {
	// attestaion
	let attester: AttesterOf<Test> = sr25519_did_from_public_key(&ALICE_SEED);
	let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_12);
	let mut attestation = generate_base_attestation::<Test>(attester.clone(), ACCOUNT_00);
	attestation.deposit.amount = MICRO_KILT;
	let ctype = get_ctype_hash::<Test>(true);

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, KILT)])
		.with_ctypes(vec![(attestation.ctype_hash, attester.clone())])
		.build()
		.execute_with(|| {
			let hashed_key = Attestations::<Test>::hashed_key_for(claim_hash);

			assert!(!Pallet::<Test>::is_key_migrated(&hashed_key));

			assert_ok!(Attestation::add(
				DoubleOrigin(ACCOUNT_00, attester).into(),
				claim_hash,
				ctype,
				None
			));

			assert!(Pallet::<Test>::is_key_migrated(&hashed_key));

			let attestation =
				BoundedVec::try_from([claim_hash].to_vec()).expect("Vec init should not fail for attestaions");

			let requested_migrations = EntriesToMigrate {
				attestation,
				..Default::default()
			};

			assert_storage_noop!(
				Migration::update_balance(RuntimeOrigin::signed(ACCOUNT_00), requested_migrations)
					.expect("Update balance should not panic")
			);
		});
}

#[test]
fn check_excluded_keys_delegation() {
	// delegation
	let creator = sr25519_did_from_public_key(&BOB_SEED);
	let delegate = sr25519_did_from_public_key(&ALICE_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details::<Test>();
	let parent_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_3);
	let mut parent_node =
		generate_base_delegation_node::<Test>(hierarchy_root_id, creator.clone(), Some(hierarchy_root_id), ACCOUNT_00);
	parent_node.deposit.amount = MICRO_KILT;

	let delegation_id = delegation_id_from_seed::<Test>(1000);

	let delegation_hash = Delegation::calculate_delegation_creation_hash(
		&delegation_id,
		&hierarchy_root_id,
		&parent_id,
		&parent_node.details.permissions,
	)
	.encode();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, KILT)])
		.with_delegation_hierarchies(vec![(
			hierarchy_root_id,
			hierarchy_details,
			creator.clone(),
			ACCOUNT_00,
		)])
		.with_delegations(vec![(parent_id, parent_node.clone())])
		.build()
		.execute_with(|| {
			let hashed_key = DelegationNodes::<Test>::hashed_key_for(delegation_id);

			assert!(!Pallet::<Test>::is_key_migrated(&hashed_key));

			let delegate_signature = (delegate.clone(), delegation_hash);

			assert_ok!(Delegation::add_delegation(
				DoubleOrigin(ACCOUNT_00, creator).into(),
				delegation_id,
				parent_id,
				delegate,
				parent_node.details.permissions,
				delegate_signature
			));

			assert!(Pallet::<Test>::is_key_migrated(&hashed_key));

			let delegation =
				BoundedVec::try_from([delegation_id].to_vec()).expect("Vec init should not fail for attestaions");

			let requested_migrations = EntriesToMigrate {
				delegation,
				..Default::default()
			};

			assert_storage_noop!(
				Migration::update_balance(RuntimeOrigin::signed(ACCOUNT_00), requested_migrations)
					.expect("Update balance should not panic")
			);
		});
}

#[test]
fn check_excluded_keys_did() {
	let auth_key = ed25519::Pair::from_seed(&ALICE_SEED);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());
	let details = generate_base_did_creation_details::<Test>(alice_did.clone(), ACCOUNT_00);
	let signature = auth_key.sign(details.encode().as_ref());

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, KILT)])
		.build()
		.execute_with(|| {
			let hashed_key = did::Did::<Test>::hashed_key_for(&alice_did);

			assert!(!Pallet::<Test>::is_key_migrated(&hashed_key));

			assert_ok!(Did::create(
				RuntimeOrigin::signed(ACCOUNT_00),
				Box::new(details),
				did::DidSignature::from(signature),
			));

			assert!(Pallet::<Test>::is_key_migrated(&hashed_key));

			let did = BoundedVec::try_from([alice_did].to_vec()).expect("Vec init should not fail for did");

			let requested_migrations = EntriesToMigrate {
				did,
				..Default::default()
			};

			assert_storage_noop!(
				Migration::update_balance(RuntimeOrigin::signed(ACCOUNT_00), requested_migrations)
					.expect("Update balance should not panic")
			);
		});
}

#[test]
fn check_excluded_keys_lookup() {
	let pair_alice = sr25519::Pair::from_seed(b"Alice                           ");
	let expire_at: BlockNumber = 500;
	let account_hash_alice = MultiSigner::from(pair_alice.public()).into_account();
	let sig_alice_0 = MultiSignature::from(
		pair_alice.sign(&[b"<Bytes>", get_challenge(&DID_00, expire_at).as_bytes(), b"</Bytes>"].concat()[..]),
	);

	let linked_acc = LinkableAccountId::from(account_hash_alice.clone());

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, KILT)])
		.build()
		.execute_with(|| {
			let hashed_key = ConnectedDids::<Test>::hashed_key_for(&linked_acc);

			assert!(!Pallet::<Test>::is_key_migrated(&hashed_key));

			assert_ok!(DidLookup::associate_account(
				mock_origin::DoubleOrigin(ACCOUNT_00, DID_00).into(),
				AssociateAccountRequest::Polkadot(account_hash_alice.clone(), sig_alice_0),
				expire_at,
			));

			assert!(Pallet::<Test>::is_key_migrated(&hashed_key));

			let lookup = BoundedVec::try_from([linked_acc].to_vec()).expect("Vec init should not fail for did");

			let requested_migrations = EntriesToMigrate {
				lookup,
				..Default::default()
			};

			assert_storage_noop!(
				Migration::update_balance(RuntimeOrigin::signed(ACCOUNT_00), requested_migrations)
					.expect("Update balance should not panic")
			);
		});
}

#[test]
fn check_excluded_keys_w3n() {
	let web3_name_00 = AsciiWeb3Name::try_from(WEB3_NAME_00_INPUT.to_vec()).expect("W3n name creation should not fail");

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, KILT)])
		.build()
		.execute_with(|| {
			let hashed_key = Owner::<Test>::hashed_key_for(&web3_name_00);

			assert!(!Pallet::<Test>::is_key_migrated(&hashed_key));

			assert_ok!(pallet_web3_names::Pallet::<Test>::claim(
				mock_origin::DoubleOrigin(ACCOUNT_00, DID_00).into(),
				web3_name_00.0.clone()
			));

			assert!(Pallet::<Test>::is_key_migrated(&hashed_key));

			let w3n = BoundedVec::try_from([web3_name_00].to_vec()).expect("Vec init should not fail for did");

			let requested_migrations = EntriesToMigrate {
				w3n,
				..Default::default()
			};

			assert_storage_noop!(
				Migration::update_balance(RuntimeOrigin::signed(ACCOUNT_00), requested_migrations)
					.expect("Update balance should not panic")
			);
		});
}

#[test]
fn check_excluded_keys_public_credentials() {
	let attester = sr25519_did_from_public_key(&ALICE_SEED);
	let subject_id = SUBJECT_ID_00;
	let ctype_hash = get_ctype_hash::<Test>(true);

	let new_credential = generate_base_public_credential_creation_op::<Test>(
		subject_id.into(),
		ctype_hash,
		InputClaimsContentOf::<Test>::default(),
	);
	let credential_id: CredentialIdOf<Test> = generate_credential_id::<Test>(&new_credential, &attester);

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, KILT)])
		.with_ctypes(vec![(ctype_hash, attester.clone())])
		.build()
		.execute_with(|| {
			let hashed_key = Credentials::<Test>::hashed_key_for(subject_id, credential_id);

			assert!(!Pallet::<Test>::is_key_migrated(&hashed_key));

			assert_ok!(PublicCredentials::add(
				DoubleOrigin(ACCOUNT_00, attester.clone()).into(),
				Box::new(new_credential.clone())
			));

			assert!(Pallet::<Test>::is_key_migrated(&hashed_key));

			let public_credentials =
				BoundedVec::try_from([(subject_id, credential_id)].to_vec()).expect("Vec init should not fail for did");

			let requested_migrations = EntriesToMigrate {
				public_credentials,
				..Default::default()
			};

			assert_storage_noop!(
				Migration::update_balance(RuntimeOrigin::signed(ACCOUNT_00), requested_migrations)
					.expect("Update balance should not panic")
			);
		});
}

#[test]
fn migrate_key_by_update_deposit_did() {
	let auth_key = ed25519::Pair::from_seed(&ALICE_SEED);
	let alice_did = get_did_identifier_from_ed25519_key(auth_key.public());

	let mut details = generate_base_did_details::<Test>(DidVerificationKey::from(auth_key.public()), Some(ACCOUNT_00));
	details.deposit.amount = MICRO_KILT;

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, KILT * 2)])
		.with_dids(vec![(alice_did.clone(), details)])
		.build()
		.execute_with(|| {
			translate_all_holds_to_reserves();
			let reserved_balance = pallet_balances::Pallet::<Test>::reserved_balance(ACCOUNT_00);

			assert_eq!(reserved_balance, MICRO_KILT);

			let did_key = did::Did::<Test>::hashed_key_for(&alice_did);
			assert!(!Pallet::<Test>::is_key_migrated(&did_key));

			assert_ok!(Did::update_deposit(
				RuntimeOrigin::signed(ACCOUNT_00),
				alice_did.clone()
			));

			let did_details = Did::get_did(alice_did);

			assert!(did_details.is_some());

			assert!(Pallet::<Test>::is_key_migrated(&did_key));

			let hold_balance =
				pallet_balances::Pallet::<Test>::balance_on_hold(&did::HoldReason::Deposit.into(), &ACCOUNT_00);

			assert_eq!(hold_balance, did_details.unwrap().deposit.amount);
		});
}

#[test]
fn migrate_key_by_update_deposit_delegation() {
	let creator = sr25519_did_from_public_key(&BOB_SEED);

	let hierarchy_root_id = get_delegation_hierarchy_id::<Test>(true);
	let hierarchy_details = generate_base_delegation_hierarchy_details::<Test>();
	let delegation_id = delegation_id_from_seed::<Test>(DELEGATION_ID_SEED_3);
	let delegation_details =
		generate_base_delegation_node::<Test>(hierarchy_root_id, creator.clone(), Some(hierarchy_root_id), ACCOUNT_00);

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, KILT)])
		.with_delegation_hierarchies(vec![(hierarchy_root_id, hierarchy_details, creator, ACCOUNT_00)])
		.with_delegations(vec![(delegation_id, delegation_details)])
		.build()
		.execute_with(|| {
			translate_all_holds_to_reserves();
			clear_storage();

			let reserved_balance = pallet_balances::Pallet::<Test>::reserved_balance(ACCOUNT_00);

			// the deposit for the hierarchy and delegation are for each 1 MIRCO_KILT
			assert_eq!(reserved_balance, 2 * MICRO_KILT);

			let hashed_key = DelegationNodes::<Test>::hashed_key_for(delegation_id);
			assert!(!Pallet::<Test>::is_key_migrated(&hashed_key));

			assert_ok!(Delegation::update_deposit(
				RuntimeOrigin::signed(ACCOUNT_00),
				delegation_id,
			));

			assert!(Pallet::<Test>::is_key_migrated(&hashed_key));

			let delegation_node = DelegationNodes::<Test>::get(delegation_id);

			assert!(delegation_node.is_some());

			let hold_balance =
				pallet_balances::Pallet::<Test>::balance_on_hold(&delegation::HoldReason::Deposit.into(), &ACCOUNT_00);

			// we have only update the delegation node. Not the hierarchy
			assert_eq!(hold_balance, MICRO_KILT);
		});
}

#[test]
fn migrate_key_by_update_deposit_lookup() {
	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, KILT)])
		.with_connections(vec![(ACCOUNT_00, DID_00, LINKABLE_ACCOUNT_00)])
		.build()
		.execute_with(|| {
			translate_all_holds_to_reserves();
			clear_storage();

			let reserved_balance = pallet_balances::Pallet::<Test>::reserved_balance(ACCOUNT_00);

			assert_eq!(reserved_balance, MICRO_KILT);

			let hashed_key = ConnectedDids::<Test>::hashed_key_for(&LINKABLE_ACCOUNT_00);

			assert!(!Pallet::<Test>::is_key_migrated(&hashed_key));

			assert_ok!(DidLookup::update_deposit(
				RuntimeOrigin::signed(ACCOUNT_00),
				LINKABLE_ACCOUNT_00
			));

			assert!(Pallet::<Test>::is_key_migrated(&hashed_key));

			let hold_balance = pallet_balances::Pallet::<Test>::balance_on_hold(
				&pallet_did_lookup::HoldReason::Deposit.into(),
				&ACCOUNT_00,
			);
			assert_eq!(hold_balance, MICRO_KILT);
		});
}

#[test]
fn migrate_key_by_update_deposit_w3n() {
	let web3_name_00 = AsciiWeb3Name::try_from(WEB3_NAME_00_INPUT.to_vec()).expect("W3n name creation should not fail");

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, KILT)])
		.with_web3_names(vec![(DID_00, web3_name_00.clone(), ACCOUNT_00)])
		.build()
		.execute_with(|| {
			translate_all_holds_to_reserves();
			clear_storage();

			let reserved_balance = pallet_balances::Pallet::<Test>::reserved_balance(ACCOUNT_00);

			assert_eq!(reserved_balance, MICRO_KILT);

			let hashed_key = Owner::<Test>::hashed_key_for(&web3_name_00);

			assert!(!Pallet::<Test>::is_key_migrated(&hashed_key));

			assert_ok!(W3n::update_deposit(
				RuntimeOrigin::signed(ACCOUNT_00),
				BoundedVec::try_from(WEB3_NAME_00_INPUT.to_vec()).expect("Input should not fail")
			));

			assert!(Pallet::<Test>::is_key_migrated(&hashed_key));

			let hold_balance = pallet_balances::Pallet::<Test>::balance_on_hold(
				&pallet_web3_names::HoldReason::Deposit.into(),
				&ACCOUNT_00,
			);
			assert_eq!(hold_balance, MICRO_KILT);
		});
}

#[test]
fn migrate_key_by_update_deposit_public_credentials() {
	let attester: AttesterOf<Test> = sr25519_did_from_public_key(&ALICE_SEED);
	let ctype_hash = get_ctype_hash::<Test>(true);

	let deposit = kilt_support::Deposit {
		owner: ACCOUNT_00,
		amount: 10 * MICRO_KILT,
	};

	let subject_id: <Test as public_credentials::Config>::SubjectId = SUBJECT_ID_00;
	let new_credential =
		generate_base_credential_entry::<Test>(ACCOUNT_00, 0, attester, Some(ctype_hash), Some(deposit));
	let credential_id: CredentialIdOf<Test> = CredentialIdOf::<Test>::default();

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, KILT)])
		.with_public_credentials(vec![(subject_id, credential_id, new_credential)])
		.build()
		.execute_with(|| {
			translate_all_holds_to_reserves();
			clear_storage();

			let reserved_balance = pallet_balances::Pallet::<Test>::reserved_balance(ACCOUNT_00);

			assert_eq!(reserved_balance, 10 * MICRO_KILT);

			let hashed_key = Credentials::<Test>::hashed_key_for(subject_id, credential_id);

			assert!(!Pallet::<Test>::is_key_migrated(&hashed_key));

			assert_ok!(PublicCredentials::update_deposit(
				RuntimeOrigin::signed(ACCOUNT_00),
				credential_id
			));

			assert!(Pallet::<Test>::is_key_migrated(&hashed_key));

			let hold_balance = pallet_balances::Pallet::<Test>::balance_on_hold(
				&public_credentials::HoldReason::Deposit.into(),
				&ACCOUNT_00,
			);
			assert_eq!(hold_balance, 10 * MICRO_KILT);
		});
}

#[test]
fn migrate_key_by_update_deposit_attestation() {
	// attestaion
	let attester: AttesterOf<Test> = sr25519_did_from_public_key(&ALICE_SEED);
	let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_12);
	let mut attestation = generate_base_attestation::<Test>(attester, ACCOUNT_00);
	attestation.deposit.amount = MICRO_KILT;

	ExtBuilder::default()
		.with_balances(vec![(ACCOUNT_00, KILT)])
		.with_attestations(vec![(claim_hash, attestation)])
		.build()
		.execute_with(|| {
			translate_all_holds_to_reserves();
			clear_storage();

			let reserved_balance = pallet_balances::Pallet::<Test>::reserved_balance(ACCOUNT_00);

			assert_eq!(reserved_balance, MICRO_KILT);

			let hashed_key = Attestations::<Test>::hashed_key_for(claim_hash);

			assert!(!Pallet::<Test>::is_key_migrated(&hashed_key));

			assert_ok!(Attestation::update_deposit(
				RuntimeOrigin::signed(ACCOUNT_00),
				claim_hash
			));

			assert!(Pallet::<Test>::is_key_migrated(&hashed_key));

			let hold_balance =
				pallet_balances::Pallet::<Test>::balance_on_hold(&attestation::HoldReason::Deposit.into(), &ACCOUNT_00);
			assert_eq!(hold_balance, MICRO_KILT);
		});
}
