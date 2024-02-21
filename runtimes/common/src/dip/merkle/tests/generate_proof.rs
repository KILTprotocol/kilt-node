use did::did_details::DidVerificationKey;
use frame_support::assert_err;

use crate::{
	constants::dip_provider::MAX_LINKED_ACCOUNTS,
	dip::{
		merkle::{DidMerkleProofError, DidMerkleRootGenerator},
		mock::{create_linked_info, TestRuntime, ACCOUNT},
	},
};

#[test]
fn generate_proof_unsupported_version() {
	let linked_info = create_linked_info(
		DidVerificationKey::Account(ACCOUNT),
		Some(b"ntn_x2"),
		MAX_LINKED_ACCOUNTS,
	);
	assert_err!(
		DidMerkleRootGenerator::<TestRuntime>::generate_proof(&linked_info, 1, [].into_iter(), false, [].into_iter()),
		DidMerkleProofError::UnsupportedVersion
	);
}
