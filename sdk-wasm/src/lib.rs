mod utils;
 
use sp_core::blake2_128;
use wasm_bindgen::prelude::*;
use sp_core::Blake2Hasher;
use kilt_utils::calculate_key_id;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// #[wasm_bindgen]
// pub fn init() {
// 	std::panic::set_hook(Box::new(console_error_panic_hook::hook));
// }

#[wasm_bindgen]
extern "C" {
	fn alert(s: &str);
}

#[wasm_bindgen]
pub fn calculate_key(key: &str) -> Vec<u8> {
	let a = calculate_key_id::<Blake2Hasher, &str>(&key);
	alert(&a.0[0].to_string());
	blake2_128(&[1,2,3,4]).to_vec()
}
