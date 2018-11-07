// initialise with:
// post({sender: runtime.balances.ss58Decode('F7Gh'), call: calls.demo.setPayment(1000)}).tie(console.log)

use parity_codec::Encode;
use runtime_primitives::traits::Hash;
use srml_support::{dispatch::Result, StorageValue};
use {
	balances,
	system::{self, ensure_signed},
};

pub trait Trait: balances::Trait {}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
	}
}

decl_storage! {
	trait Store for Module<T: Trait> as Demo {
	}
}
