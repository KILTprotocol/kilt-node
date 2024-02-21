use did::did_details::DidVerificationKey;
use frame_support::assert_err;
use pallet_dip_provider::traits::IdentityCommitmentGenerator;

use crate::{
	constants::dip_provider::MAX_LINKED_ACCOUNTS,
	dip::{
		merkle::{DidMerkleProofError, DidMerkleRootGenerator},
		mock::{create_linked_info, TestRuntime, ACCOUNT, DID_IDENTIFIER},
	},
};

#[test]
fn generate_commitment_unsupported_version() {
	let linked_info = create_linked_info(
		DidVerificationKey::Account(ACCOUNT),
		Some(b"ntn_x2"),
		MAX_LINKED_ACCOUNTS,
	);
	assert_err!(
		DidMerkleRootGenerator::<TestRuntime>::generate_commitment(&DID_IDENTIFIER, &linked_info, 1,),
		DidMerkleProofError::UnsupportedVersion
	);
}
