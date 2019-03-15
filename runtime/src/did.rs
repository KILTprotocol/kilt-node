

use rstd::prelude::*;
use runtime_primitives::traits::{Member};
use support::{dispatch::Result, StorageMap, Parameter, decl_module, decl_storage};
use runtime_primitives::codec::Codec;
use {system, system::ensure_signed};

pub trait Trait: system::Trait {
    type PublicSigningKey : Parameter + Member + Codec + Default;
    type PublicBoxKey : Parameter + Member + Codec + Default;
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		pub fn add(origin, sign_key: T::PublicSigningKey, box_key: T::PublicBoxKey, doc_ref: Option<Vec<u8>>) -> Result {
			let sender = ensure_signed(origin)?;
			<DID<T>>::insert(sender.clone(), (sign_key, box_key, doc_ref));
            Ok(())
		}
		
        pub fn remove(origin) -> Result {
			let sender = ensure_signed(origin)?;
			<DID<T>>::remove(sender.clone());
            Ok(())
		}
	}
}

decl_storage! {
	trait Store for Module<T: Trait> as DID {
		// DID: account-id -> (did-reference?)
		DID get(dids): map T::AccountId => (T::PublicSigningKey, T::PublicBoxKey, Option<Vec<u8>>);
	}
}


#[cfg(test)]
mod tests {
	use super::*;
	use system;
	use runtime_io::with_externalities;
	use primitives::{H256, Blake2Hasher};
	use primitives::*;
	use support::{impl_outer_origin, assert_ok};

	use runtime_primitives::{
		BuildStorage, traits::{BlakeTwo256, IdentityLookup}, testing::{Digest, DigestItem, Header}
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
        type PublicSigningKey = H256;
        type PublicBoxKey = H256;
	}

	type DID = Module<Test>;

	fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
		system::GenesisConfig::<Test>::default().build_storage().unwrap().0.into()
	}

	#[test]
	fn check_add_did() {
		with_externalities(&mut new_test_ext(), || {
			let pair = ed25519::Pair::from_seed(b"Alice                           ");
			let hash = H256::from_low_u64_be(1);
			let account_hash = H256::from(pair.public().0);
			assert_ok!(DID::add(Origin::signed(account_hash.clone()), 
                    hash.clone(), hash.clone(), Some(b"http://kilt.org/submit".to_vec())));
		});
	}
}
