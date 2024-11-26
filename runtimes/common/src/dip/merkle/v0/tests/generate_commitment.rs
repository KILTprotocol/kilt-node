use did::did_details::DidVerificationKey;
use frame_support::assert_ok;

use crate::{
	constants::dip_provider::MAX_LINKED_ACCOUNTS,
	dip::{
		merkle::v0::generate_commitment,
		mock::{create_linked_info, TestRuntime, ACCOUNT},
	},
};

#[test]
fn generate_commitment_for_complete_info() {
	let linked_info = create_linked_info(
		DidVerificationKey::Account(ACCOUNT),
		Some(b"ntn_x2"),
		MAX_LINKED_ACCOUNTS,
	);
	assert_ok!(generate_commitment::<TestRuntime, MAX_LINKED_ACCOUNTS>(&linked_info));
}

#[test]
fn generate_commitment_for_did_details() {
	let linked_info = create_linked_info(DidVerificationKey::Account(ACCOUNT), Option::<Vec<u8>>::None, 0);
	assert_ok!(generate_commitment::<TestRuntime, MAX_LINKED_ACCOUNTS>(&linked_info));
}

#[test]
fn generate_commitment_for_did_details_and_web3name() {
	let linked_info = create_linked_info(DidVerificationKey::Account(ACCOUNT), Some(b"ntn_x2"), 0);
	assert_ok!(generate_commitment::<TestRuntime, MAX_LINKED_ACCOUNTS>(&linked_info));
}

#[test]
fn generate_commitment_for_did_details_and_max_linked_accounts() {
	let linked_info = create_linked_info(
		DidVerificationKey::Account(ACCOUNT),
		Option::<Vec<u8>>::None,
		MAX_LINKED_ACCOUNTS,
	);
	assert_ok!(generate_commitment::<TestRuntime, MAX_LINKED_ACCOUNTS>(&linked_info));
}
