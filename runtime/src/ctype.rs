// initialise with:
// post({sender: runtime.balances.ss58Decode('F7Gh'), call: calls.demo.setPayment(1000)}).tie(console.log)

use support::{dispatch::Result, StorageMap, decl_module, decl_storage};
use {system, system::ensure_signed};

pub trait Trait: system::Trait {
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		pub fn add(origin, hash: T::Hash) -> Result {
			if <CTYPEs<T>>::exists(hash) {
				return Err("CTYPE already exists")
			}

			let sender = ensure_signed(origin)?;
			::runtime_io::print("insert CTYPE");
			<CTYPEs<T>>::insert(hash.clone(), (hash.clone(), sender.clone()));
			Ok(())
		}

	}
}

decl_storage! {
	trait Store for Module<T: Trait> as Ctype {
		pub CTYPEs get(ctypes): map T::Hash => (T::Hash,T::AccountId);
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	use primitives::{Blake2Hasher, H256};
	use runtime_io::with_externalities;
	use system;
	use support::{impl_outer_origin, assert_ok, assert_err};
	use runtime_primitives::{
		testing::{Digest, DigestItem, Header},
		traits::{BlakeTwo256,IdentityLookup},
		BuildStorage,
	};

	impl_outer_origin! {
		pub enum Origin for Test {}
	}

	#[derive(Clone, Eq, PartialEq)]
	pub struct Test;
	impl system::Trait for Test {
		type Origin = Origin;
		type Index = u64;
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type Digest = Digest;
		type AccountId = H256;
		type Header = Header;
		type Event = ();
		type Log = DigestItem;
		type Lookup = IdentityLookup<H256>;
	}

	impl Trait for Test {
	}
	type CType = Module<Test>;

	fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
		system::GenesisConfig::<Test>::default().build_storage().unwrap().0.into()
	}

	#[test]
	fn it_works_for_default_value() {
		with_externalities(&mut new_test_ext(), || {
			assert_ok!(
				CType::add(
					Origin::signed(H256::from_low_u64_be(1)),
					H256::from_low_u64_be(2)
				)
			);
			assert_err!(
				CType::add(
					Origin::signed(H256::from_low_u64_be(1)),
					H256::from_low_u64_be(2)
				),
				"CTYPE already exists"
			);
		});
	}
}
