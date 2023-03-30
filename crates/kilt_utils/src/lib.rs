#![cfg_attr(not(feature = "std"), no_std)]

use codec::Encode;
use sp_core::Hasher;
use sp_std::vec::Vec;

pub fn _calculate_key_id<H: Hasher, S: Encode>(key: &S) -> <H as sp_core::Hasher>::Out {
	let hashed_values: Vec<u8> = key.encode();
	H::hash(&hashed_values)
}

/// Verify that a given slice can be used as a web3 name.
pub fn _is_valid_web3_name(input: &[u8]) -> bool {
	input
		.iter()
		.all(|c| matches!(c, b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_'))
}
