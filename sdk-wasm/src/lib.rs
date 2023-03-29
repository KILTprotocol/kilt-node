mod traits;
mod utils;

use traits::blake2::BlakeTwo256;
use wasm_bindgen::prelude::*;

use kilt_utils::calculate_key_id;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
	fn alert(s: &str);
}

#[wasm_bindgen]
pub fn calculate_key(key: &str) -> Vec<u8> {
	calculate_key_id::<BlakeTwo256, &str>(&key).to_fixed_bytes().to_vec()
}
