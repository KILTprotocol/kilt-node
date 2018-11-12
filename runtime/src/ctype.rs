// initialise with:
// post({sender: runtime.balances.ss58Decode('F7Gh'), call: calls.demo.setPayment(1000)}).tie(console.log)
use rstd::prelude::*;
use runtime_primitives::Ed25519Signature;
use srml_support::{StorageValue, dispatch::Result};
use {balances, system::ensure_signed};

pub trait Trait: balances::Trait {}

#[derive(PartialEq, Eq, Clone, Encode, Decode)]
pub struct Ctype<T,S,A> {
	hash: T,
	signature: S,
	origin: A,
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn add(origin, hash:T::Hash, signature:Ed25519Signature) -> Result {
			let sender = ensure_signed(origin)?;
			let ctype = Ctype {
				hash: hash,
				signature: signature,
				origin: sender,
			};

			let mut ctypes = <CTYPEs<T>>::get();
			ctypes.push(ctype);
			// TODO: is this necessary?
			<CTYPEs<T>>::put(ctypes);
			Ok(())
		}
	}
}

decl_storage! {
	trait Store for Module<T: Trait> as CTYPEModule {
		CTYPEs get(ctypes): Vec<Ctype<T::Hash,Ed25519Signature,T::AccountId>>;
	}
}
