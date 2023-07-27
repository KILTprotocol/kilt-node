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
	mock::{claim_hash_from_seed, generate_base_attestation, sr25519_did_from_seed},
	AttesterOf, ClaimHashOf,
};
use delegation::{
	mock::{
		delegation_id_from_seed, generate_base_delegation_hierarchy_details, generate_base_delegation_node,
		get_delegation_hierarchy_id,
	},
	DelegationNodeIdOf,
};
use did::{did_details::DidVerificationKey, mock_utils::generate_base_did_details, DidIdentifierOf};
use frame_support::{
	assert_noop, assert_ok, assert_storage_noop,
	traits::{
		fungible::{Inspect, InspectHold},
		tokens::{Fortitude, Preservation},
	},
};
use pallet_did_lookup::linkable_account::LinkableAccountId;
use pallet_web3_names::{web3_name::AsciiWeb3Name, Web3NameOf};
use public_credentials::{mock::generate_base_credential_entry, CredentialIdOf, SubjectIdOf};
use sp_core::{ed25519, Pair};
use sp_runtime::BoundedVec;

use crate::{mock::runtime::*, Config, MigratedKeys};

#[test]
fn check_succesful_migration() {
	// attestaion
	let attester: AttesterOf<Test> = sr25519_did_from_seed(&ALICE_SEED);
	let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_12);
	let mut attestation = generate_base_attestation::<Test>(attester.clone(), ACCOUNT_00);
	attestation.deposit.amount = MICRO_KILT;

	// delegation
	let creator = sr25519_did_from_seed(&BOB_SEED);
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

			let mut requested_migrations = get_default_entries_to_migrate();

			let attestations: BoundedVec<ClaimHashOf<Test>, <Test as Config>::MaxMigrations> =
				BoundedVec::try_from([claim_hash].to_vec()).expect("Vec init should not fail for attestaions");

			let delegations: BoundedVec<DelegationNodeIdOf<Test>, <Test as Config>::MaxMigrations> =
				BoundedVec::try_from([parent_id].to_vec()).expect("Vec init should not fail for delegations");

			let did: BoundedVec<DidIdentifierOf<Test>, <Test as Config>::MaxMigrations> =
				BoundedVec::try_from([alice_did].to_vec()).expect("Vec init should not fail for did");

			let lookup: BoundedVec<LinkableAccountId, <Test as Config>::MaxMigrations> =
				BoundedVec::try_from([LINKABLE_ACCOUNT_00].to_vec()).expect("Vec init should not fail for lookup");

			let w3n: BoundedVec<Web3NameOf<Test>, <Test as Config>::MaxMigrations> =
				BoundedVec::try_from([web3_name_00].to_vec()).expect("Vec init should not fail for w3n");

			let public_credentials: BoundedVec<
				(SubjectIdOf<Test>, CredentialIdOf<Test>),
				<Test as Config>::MaxMigrations,
			> = BoundedVec::try_from([(subject_id, credential_id)].to_vec())
				.expect("Vec init should not fail for public_credentials");

			requested_migrations.attestation = attestations;
			requested_migrations.delegation = delegations;
			requested_migrations.did = did;
			requested_migrations.lookup = lookup;
			requested_migrations.w3n = w3n;
			requested_migrations.public_credentials = public_credentials;

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
fn check_unsuccesful_migration() {
	// attestaion
	let attester: AttesterOf<Test> = sr25519_did_from_seed(&ALICE_SEED);
	let claim_hash = claim_hash_from_seed(CLAIM_HASH_SEED_12);
	let mut attestation = generate_base_attestation::<Test>(attester.clone(), ACCOUNT_00);
	attestation.deposit.amount = MICRO_KILT;

	// delegation
	let creator = sr25519_did_from_seed(&BOB_SEED);
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
			translate_all_holds_to_reserves();

			let mut requested_migrations = get_default_entries_to_migrate();

			let attestations: BoundedVec<ClaimHashOf<Test>, <Test as Config>::MaxMigrations> =
				BoundedVec::try_from([claim_hash].to_vec()).expect("Vec init should not fail for attestaions");

			let delegations: BoundedVec<DelegationNodeIdOf<Test>, <Test as Config>::MaxMigrations> =
				BoundedVec::try_from([parent_id].to_vec()).expect("Vec init should not fail for delegations");

			let did: BoundedVec<DidIdentifierOf<Test>, <Test as Config>::MaxMigrations> =
				BoundedVec::try_from([alice_did].to_vec()).expect("Vec init should not fail for did");

			let lookup: BoundedVec<LinkableAccountId, <Test as Config>::MaxMigrations> =
				BoundedVec::try_from([LINKABLE_ACCOUNT_00].to_vec()).expect("Vec init should not fail for lookup");

			let w3n: BoundedVec<Web3NameOf<Test>, <Test as Config>::MaxMigrations> =
				BoundedVec::try_from([web3_name_00].to_vec()).expect("Vec init should not fail for w3n");

			let public_credentials: BoundedVec<
				(SubjectIdOf<Test>, CredentialIdOf<Test>),
				<Test as Config>::MaxMigrations,
			> = BoundedVec::try_from([(subject_id, credential_id)].to_vec())
				.expect("Vec init should not fail for public_credentials");

			requested_migrations.attestation = attestations;
			requested_migrations.delegation = delegations;
			requested_migrations.did = did;
			requested_migrations.lookup = lookup;
			requested_migrations.w3n = w3n;
			requested_migrations.public_credentials = public_credentials;

			assert_ok!(Migration::update_balance(
				RuntimeOrigin::signed(ACCOUNT_00),
				requested_migrations.clone()
			));

			// Nothing should happen now
			assert_storage_noop!(
				Migration::update_balance(RuntimeOrigin::signed(ACCOUNT_00), requested_migrations)
					.expect("should not panic")
			);
		});
}
