// initialise with:
// post({sender: runtime.balances.ss58Decode('F7Gh'), call: calls.demo.setPayment(1000)}).tie(console.log)



use runtime_primitives::codec::Codec;
use runtime_primitives::verify_encoded_lazy;
use support::{dispatch::Result, StorageMap, decl_module, decl_storage};
use runtime_primitives::traits::{Member, Verify};
use {balances, system::ensure_signed};

pub trait Trait: balances::Trait {
	type Signature: Verify<Signer = Self::AccountId> + Member + Codec + Default;

	fn print_account(_: Self::AccountId);
	fn print_hash(_: Self::Hash);
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
		pub CTYPEs get(ctypes): map T::Hash => (T::Hash,T::Signature,T::AccountId);
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	use primitives::{Blake2Hasher, H256, H512};
	use runtime_io::with_externalities;
	use runtime_primitives::Ed25519Signature;
	use system;

	use sr_primitives::{
		testing::{Digest, DigestItem, Header},
		traits::BlakeTwo256,
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
	}
	impl balances::Trait for Test {
		type Balance = u64;
		type AccountIndex = u64;
		type OnFreeBalanceZero = ();
		type EnsureAccountLiquid = ();
		type Event = ();
	}

	impl Trait for Test {
		type Signature = Ed25519Signature;
		fn print_account(_a: Self::AccountId) {}
		fn print_hash(_a: Self::Hash) {}
	}
	type CType = Module<Test>;

	// This function basically just builds a genesis storage key/value store according to
	// our desired mockup.
	fn new_test_ext() -> sr_io::TestExternalities<Blake2Hasher> {
		let mut t = system::GenesisConfig::<Test>::default()
			.build_storage()
			.unwrap()
			.0;
		// We use default for brevity, but you can configure as desired if needed.
		t.extend(
			balances::GenesisConfig::<Test>::default()
				.build_storage()
				.unwrap()
				.0,
		);
		t.into()
	}

	#[test]
	fn it_works_for_default_value() {
		with_externalities(&mut new_test_ext(), || {
			assert_err!(
				CType::add(
					Origin::signed(H256::from(1)),
					H256::from(2),
					Ed25519Signature::from(H512::from(3))
				),
				"bad signature"
			);
		});
	}
}
