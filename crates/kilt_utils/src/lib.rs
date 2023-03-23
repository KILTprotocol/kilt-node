#![cfg_attr(not(feature = "std"), no_std)]

use codec::Encode;
use sp_runtime::traits::Hash;
use sp_std::vec::Vec;

// todo: kill out frame_system::Config.
pub fn calculate_key_id<T: frame_system::Config, S: Encode>(key: &S) -> <T>::Hash {
	let hashed_values: Vec<u8> = key.encode();
	T::Hashing::hash(&hashed_values)
}
