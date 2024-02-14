// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2024 BOTLabs GmbH

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

use did::did_details::DidVerificationKey;
use frame_support::assert_noop;
use pallet_dip_provider::traits::IdentityProvider;

use crate::{
	constants::dip_provider::MAX_LINKED_ACCOUNTS,
	dip::{
		did::{LinkedDidInfoOf, LinkedDidInfoProvider, LinkedDidInfoProviderError, Web3OwnershipOf},
		mock::{create_linked_info, ExtBuilder, TestRuntime, ACCOUNT, DID_IDENTIFIER, SUBMITTER},
	},
};

#[test]
fn linked_did_info_provider_retrieve_max_capacity() {
	let auth_key = DidVerificationKey::Account(ACCOUNT);
	let LinkedDidInfoOf {
		did_details,
		web3_name_details,
		linked_accounts,
	} = create_linked_info(auth_key, Some(b"ntn_x2"), MAX_LINKED_ACCOUNTS);
	let web3_name: Option<pallet_web3_names::web3_name::AsciiWeb3Name<TestRuntime>> =
		web3_name_details.map(|n| n.web3_name);

	ExtBuilder::default()
		.with_dids(vec![(
			DID_IDENTIFIER,
			did_details.clone(),
			web3_name.clone(),
			linked_accounts.clone().into_inner(),
			SUBMITTER,
		)])
		.build()
		.execute_with(|| {
			let identity: LinkedDidInfoOf<TestRuntime, MAX_LINKED_ACCOUNTS> =
				LinkedDidInfoProvider::retrieve(&DID_IDENTIFIER).expect("Should not fail to fetch identity details.");
			assert_eq!(identity.did_details, did_details);
			assert_eq!(
				identity.web3_name_details,
				Some(Web3OwnershipOf::<TestRuntime> {
					web3_name: web3_name.unwrap(),
					claimed_at: 0
				})
			);
			assert!(identity.linked_accounts.iter().all(|i| linked_accounts.contains(i)));
			assert!(linked_accounts.iter().all(|i| identity.linked_accounts.contains(i)));
			assert_eq!(identity.linked_accounts.len(), MAX_LINKED_ACCOUNTS as usize);
		});
}

#[test]
fn linked_did_info_provider_retrieve_only_did_details() {
	let auth_key = DidVerificationKey::Account(ACCOUNT);
	let LinkedDidInfoOf { did_details, .. } = create_linked_info(auth_key, Option::<Vec<u8>>::None, 0);

	ExtBuilder::default()
		.with_dids(vec![(DID_IDENTIFIER, did_details.clone(), None, vec![], SUBMITTER)])
		.build()
		.execute_with(|| {
			let identity: LinkedDidInfoOf<TestRuntime, MAX_LINKED_ACCOUNTS> =
				LinkedDidInfoProvider::retrieve(&DID_IDENTIFIER).expect("Should not fail to fetch identity details.");
			assert_eq!(identity.did_details, did_details);
			assert_eq!(identity.linked_accounts, vec![]);
			assert!(identity.web3_name_details.is_none())
		});
}

#[test]
fn linked_did_info_provider_retrieve_delete_did() {
	ExtBuilder::default()
		.with_deleted_dids(vec![DID_IDENTIFIER])
		.build()
		.execute_with(|| {
			assert_noop!(
				LinkedDidInfoProvider::retrieve(&DID_IDENTIFIER)
					as Result<LinkedDidInfoOf<TestRuntime, MAX_LINKED_ACCOUNTS>, _>,
				LinkedDidInfoProviderError::DidDeleted
			);
		});
}

#[test]
fn linked_did_info_provider_retrieve_did_not_found() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			LinkedDidInfoProvider::retrieve(&DID_IDENTIFIER)
				as Result<LinkedDidInfoOf<TestRuntime, MAX_LINKED_ACCOUNTS>, _>,
			LinkedDidInfoProviderError::DidNotFound
		);
	});
}

#[test]
#[should_panic = "Cannot cast generated vector of linked accounts with length 11 to BoundedVec with max limit of 10."]
fn linked_did_info_provider_retrieve_too_many_linked_accounts() {
	let auth_key = DidVerificationKey::Account(ACCOUNT);
	let LinkedDidInfoOf {
		did_details,
		web3_name_details,
		linked_accounts,
	} = create_linked_info(auth_key, Some(b"ntn_x2"), MAX_LINKED_ACCOUNTS + 1);
	let web3_name = web3_name_details.map(|n| n.web3_name);

	ExtBuilder::default()
		.with_dids(vec![(
			DID_IDENTIFIER,
			did_details,
			web3_name,
			linked_accounts.into_inner(),
			SUBMITTER,
		)])
		.build()
		.execute_with(|| {
			assert_noop!(
				LinkedDidInfoProvider::retrieve(&DID_IDENTIFIER)
					as Result<LinkedDidInfoOf<TestRuntime, MAX_LINKED_ACCOUNTS>, _>,
				LinkedDidInfoProviderError::TooManyLinkedAccounts
			);
		});
}
