// initialise with:
// post({sender: runtime.balances.ss58Decode('F7Gh'), call: calls.demo.setPayment(1000)}).tie(console.log)
use parity_codec::{Decode, Encode, Input, Output};
use rstd::prelude::*;
use runtime_primitives::traits::Hash;
use runtime_primitives::RuntimeString;
use srml_support::{dispatch::Result, StorageValue};

use {
	balances,
	system::{self, ensure_signed},
};

pub trait Trait: balances::Trait {}

pub struct Ctype {
	name: RuntimeString,
}

impl Encode for Ctype {
	/* fn encode_to<T: Output>(&self, dest: &mut T) {
		self.name.encode_to(dest)
	} */
}

impl Decode for Ctype {
	/* fn decode<I: Input>(value: &mut I) -> Option<Self> {
		Some(Ctype {
			name: core::str::from_utf8(&Vec::decode(value)?).into(),
		})
	} */
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

	}
}

decl_storage! {
	trait Store for Module<T: Trait> as CTYPEModule {
		pub CTYPEs get(ctypes): Vec<Ctype>;
	}
}
