#![cfg_attr(not(feature = "std"), no_std)]

use codec::Encode;
use sp_core::Hasher;
use sp_std::vec::Vec;

// todo: kill out frame_system::Config.
pub fn calculate_key_id<H: Hasher, S: Encode>(key: &S) -> <H as sp_core::Hasher>::Out {
	let hashed_values: Vec<u8> = key.encode();
	H::hash(&hashed_values)
}
