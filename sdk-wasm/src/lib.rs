mod traits;
mod utils;

use kilt_utils::{_calculate_key_id, _is_valid_web3_name};
use runtime_common::{
	constants::web3_names::{MAX_LENGTH, MIN_LENGTH},
	runtime_index::{CallIndexDid, CallIndexUtility, RuntimeSpiritnet},
};
use traits::blake2::BlakeTwo256;
use wasm_bindgen::prelude::*;

use did::DidVerificationKeyRelationship;

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
pub fn derive_keys(call_encoded: String) -> Result<did::DidVerificationKeyRelationship, JsValue> {
	if call_encoded.len() < 4 {
		return Err(JsValue::from_str(
			"The call has to be encoded and provided as Hexstring with a length of at least 4",
		));
	}
	let pallet_index =
		u32::from_str_radix(&call_encoded[..2], 16).or(Err(JsValue::from_str("pallet index is wrong encoded")))?;
	let call_index_encoded =
		u32::from_str_radix(&call_encoded[2..4], 16).or(Err(JsValue::from_str("call index is wrong encoded")))?;

	match pallet_index.into() {
		RuntimeSpiritnet::Attestation => Ok(DidVerificationKeyRelationship::AssertionMethod),
		RuntimeSpiritnet::Ctype => Ok(DidVerificationKeyRelationship::CapabilityDelegation),
		RuntimeSpiritnet::Delegation => Ok(did::DidVerificationKeyRelationship::CapabilityDelegation),

		RuntimeSpiritnet::Web3Names => Ok(DidVerificationKeyRelationship::Authentication),
		RuntimeSpiritnet::PublicCredentials => Ok(DidVerificationKeyRelationship::AssertionMethod),
		RuntimeSpiritnet::DidLookup => Ok(DidVerificationKeyRelationship::Authentication),
		RuntimeSpiritnet::Did => match call_index_encoded.into() {
			CallIndexDid::Create => Err(JsValue::from_str("create can not be executed")),
			_ => Ok(DidVerificationKeyRelationship::Authentication),
		},
		RuntimeSpiritnet::Utility => match call_index_encoded.into() {
			CallIndexUtility::Batch => todo!(),
			CallIndexUtility::BatchAll => todo!(),
			CallIndexUtility::ForceBatch => todo!(),
			_ => Err(JsValue::from_str("pallet index does not exists")),
		},
		_ => Err(JsValue::from_str("pallet index does not exists")),
	}
}
