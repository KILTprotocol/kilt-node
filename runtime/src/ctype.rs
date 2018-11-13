// initialise with:
// post({sender: runtime.balances.ss58Decode('F7Gh'), call: calls.demo.setPayment(1000)}).tie(console.log)
use traits::{Verify,Member};
use sr_primitives::verify_encoded_lazy;
use runtime_primitives::codec::{Codec};
use rstd::prelude::*;
use srml_support::{StorageMap, dispatch::Result};
use {balances, system::ensure_signed};

pub trait Trait: balances::Trait {
	type Signature: Verify<Signer=Self::AccountId> + Member + Codec + Default;
}

#[derive(PartialEq, Eq, Clone, Encode, Decode)]
#[derive(Default)]
pub struct Ctype<T,S,A> {
	hash: T,
	signature: S,
	origin: A,
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		fn add(origin, hash: T::Hash, signature: T::Signature) -> Result {
			let sender = ensure_signed(origin)?;
			let payload = (hash, sender.clone());
			if !verify_encoded_lazy(&signature, &payload, &sender) {
				return Err("bad signature")
			}

			let ctype = Ctype {
				hash: hash,
				signature: signature,
				origin: sender,
			};

			<CTYPEs<T>>::insert(hash, ctype);
			Ok(())
		}

	}
}

decl_storage! {
	trait Store for Module<T: Trait> as CTYPEModule {
		CTYPEs get(ctypes): map T::Hash => Ctype<T::Hash,T::Signature,T::AccountId>;
	}
}
