mod utils;

use kilt_utils::calculate_key_id;
use sp_runtime::traits::BlakeTwo256;
use wasm_bindgen::prelude::*;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn init() {
	std::panic::set_hook(Box::new(console_error_panic_hook::hook));
}

#[wasm_bindgen]
extern "C" {
	fn alert(s: &str);
}

#[wasm_bindgen]
pub fn calculate_key(key: &str) -> Vec<u8> {
	calculate_key_id::<BlakeTwo256, &str>(&key).to_fixed_bytes().to_vec()
}
