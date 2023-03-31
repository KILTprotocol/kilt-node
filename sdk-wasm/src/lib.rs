mod traits;
mod utils;

use kilt_utils::{_calculate_key_id, _is_valid_web3_name};
use runtime_common::{
	constants::web3_names::{MAX_LENGTH, MIN_LENGTH},
	runtime_index::RuntimeSpiritnet,
};

use traits::blake2::BlakeTwo256;
use wasm_bindgen::prelude::*;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
	fn alert(s: &str);
	#[wasm_bindgen(js_namespace = console)]
	fn log(s: &str);
}

#[wasm_bindgen]
pub fn calculate_key_id(key: &str) -> Vec<u8> {
	_calculate_key_id::<BlakeTwo256, &str>(&key).to_fixed_bytes().to_vec()
}

#[wasm_bindgen]
pub fn is_valid_web3_name(name: &str) -> bool {
	let name_lenght: u32 = name.len().try_into().unwrap_throw();
	if name_lenght > MAX_LENGTH || name_lenght < MIN_LENGTH {
		return false;
	}
	_is_valid_web3_name(name.as_bytes())
}

#[wasm_bindgen]
pub fn derive_keys(call_id: String) -> bool {
	if call_id.len() > 4 || call_id.len() == 0 {
		return false;
	}
	let pallet_id = u32::from_str_radix(&call_id[..2], 16).unwrap();

	match pallet_id.into() {
		RuntimeSpiritnet::Attestation => (),
		RuntimeSpiritnet::Ctype => (),
		RuntimeSpiritnet::Delegation => (),
		RuntimeSpiritnet::Did => (),
		RuntimeSpiritnet::Web3Names => (),
		RuntimeSpiritnet::PublicCredentials => (),
		RuntimeSpiritnet::DidLookup => (),
		RuntimeSpiritnet::Utility => (),
		RuntimeSpiritnet::Utility => (),
		RuntimeSpiritnet::Utility => (),

		_ => (),
	};

	true
}
