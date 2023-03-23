#![cfg_attr(not(feature = "std"), no_std)]

use codec::Encode;
use sp_core::H256;
use sp_runtime::traits::{BlakeTwo256, Hash};
use sp_std::vec::Vec;

pub fn calculate_key_id<S: Encode>(key: &S) {
	let hashed_values: Vec<u8> = key.encode();
	BlakeTwo256::hash(&hashed_values)
}
