// initialise with:
// post({sender: runtime.balances.ss58Decode('F7Gh'), call: calls.demo.setPayment(1000)}).tie(console.log)

use traits::{Verify,Member};
use sr_primitives::verify_encoded_lazy;
use runtime_primitives::codec::{Codec};
use srml_support::{StorageMap, dispatch::Result};
use {balances, system::ensure_signed};

pub trait Trait: balances::Trait {
	type Signature: Verify<Signer=Self::AccountId> + Member + Codec + Default;

	fn print_account(Self::AccountId);
	fn print_hash(Self::Hash);
}


decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		fn add(origin, hash: T::Hash, signature: T::Signature) -> Result {
			::runtime_io::print("got hash:");
			T::print_hash(hash.clone());

			if <CTYPEs<T>>::exists(hash) {
				let existing_ctype_from_map = <CTYPEs<T>>::get(hash.clone());
				::runtime_io::print("existing hash:");
				T::print_hash(existing_ctype_from_map.0.clone());
				::runtime_io::print("existing origin:");
				T::print_account(existing_ctype_from_map.2.clone());
			}

			let sender = ensure_signed(origin)?;
			let h = hash.clone();
			if !verify_encoded_lazy(&signature, &h, &sender) {
				return Err("bad signature")
			}

			<CTYPEs<T>>::insert(hash.clone(), (hash.clone(), signature.clone(), sender.clone()));
			let ctype_from_map = <CTYPEs<T>>::get(hash.clone());
			::runtime_io::print("after insert hash:");
			T::print_hash(ctype_from_map.0.clone());
			::runtime_io::print("after insert origin:");
			T::print_account(ctype_from_map.2.clone());
			Ok(())
		}

	}
}

decl_storage! {
	trait Store for Module<T: Trait> as Ctype {
		CTYPEs get(ctypes): map T::Hash => (T::Hash,T::Signature,T::AccountId);
	}
}
