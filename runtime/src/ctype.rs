// initialise with:
// post({sender: runtime.balances.ss58Decode('F7Gh'), call: calls.demo.setPayment(1000)}).tie(console.log)

use support::{dispatch::Result, StorageMap, decl_module, decl_storage, decl_event};
use {system, system::ensure_signed, super::error};

pub trait Trait: system::Trait + error::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_event!(
	pub enum Event<T> where <T as system::Trait>::AccountId, <T as system::Trait>::Hash {
		/// A CTYPE has been added
		CTypeCreated(AccountId, Hash),
	}
);

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		fn deposit_event<T>() = default;

		pub fn add(origin, hash: T::Hash) -> Result {
			if <CTYPEs<T>>::exists(hash) {
				return Self::error(Self::ERROR_CTYPE_ALREADY_EXISTS);
			}

			let sender = ensure_signed(origin)?;
			::runtime_io::print("insert CTYPE");
			<CTYPEs<T>>::insert(hash.clone(), sender.clone());
			Self::deposit_event(RawEvent::CTypeCreated(sender.clone(), hash.clone()));
			Ok(())
		}

	}
}

decl_storage! {
	trait Store for Module<T: Trait> as Ctype {
		pub CTYPEs get(ctypes): map T::Hash => T::AccountId;
	}
}

impl<T: Trait> Module<T> {
    
    pub const ERROR_BASE: u16 = 100;
    pub const ERROR_CTYPE_NOT_FOUND : error::ErrorType = (Self::ERROR_BASE + 1, "CTYPE not found");
    pub const ERROR_CTYPE_ALREADY_EXISTS : error::ErrorType = (Self::ERROR_BASE + 2, "CTYPE already exists");

    pub fn error(error_type: error::ErrorType) -> Result {
        return <error::Module<T>>::error(error_type);
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

	impl error::Trait for Test {
		type Event = ();
        type ErrorCode = u16;
	}

	impl Trait for Test {
		type Event = ();
	}

	type CType = Module<Test>;

	fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
		system::GenesisConfig::<Test>::default().build_storage().unwrap().0.into()
	}

	#[test]
	fn it_works_for_default_value() {
		with_externalities(&mut new_test_ext(), || {
			let account = H256::from_low_u64_be(1);
			let ctype_hash = H256::from_low_u64_be(2);
			assert_ok!(
				CType::add(
					Origin::signed(account.clone()),
					ctype_hash.clone()
				)
			);
			assert_eq!(<CTYPEs<Test>>::exists(ctype_hash), true);
            assert_eq!(CType::ctypes(ctype_hash.clone()), account.clone());
			assert_err!(
				CType::add(
					Origin::signed(account.clone()),
					ctype_hash.clone()
				),
				CType::ERROR_CTYPE_ALREADY_EXISTS.1
			);
		});
	}
}
