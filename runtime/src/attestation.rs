use rstd::prelude::*;
use runtime_primitives::codec::Codec;
use sr_primitives::verify_encoded_lazy;
use srml_support::{dispatch::Result, StorageMap};
use traits::{Member, Verify};
use {balances, system::ensure_signed};

pub trait Trait: balances::Trait {
	type Signature: Verify<Signer = Self::AccountId> + Member + Codec + Default;
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		fn add(origin, claim_hash: T::Hash, signature: T::Signature) -> Result {
			let sender = ensure_signed(origin)?;
			if !verify_encoded_lazy(&signature, &claim_hash, &sender) {
				return Err("bad signature")
			}

			let mut existing_attestations_for_claim = <Attestations<T>>::get(claim_hash.clone());
			let mut last_attested : Option<(T::Hash,T::AccountId,T::Signature,bool)> = None;
			for v in existing_attestations_for_claim.clone() {
				if v.1.eq(&sender) {
					last_attested = Some(v);
					break;
				}
			}
			match last_attested {
				Some(_v)	=> return Err("already attested"),
				None	=> {
					existing_attestations_for_claim.push((claim_hash.clone(), sender.clone(), signature.clone(), false));
					<Attestations<T>>::insert(claim_hash.clone(), existing_attestations_for_claim);
					Ok(())
				},
			}
		}

		fn revoke(origin, claim_hash: T::Hash, signature: T::Signature) -> Result {
			let sender = ensure_signed(origin)?;
			if !verify_encoded_lazy(&signature, &claim_hash, &sender) {
				return Err("bad signature")
			}

			let mut last_attested : bool = false;
			let mut existing_attestations_for_claim = <Attestations<T>>::get(claim_hash.clone());
			for v in existing_attestations_for_claim.iter_mut() {
				if v.1.eq(&sender) && !v.3 {
					last_attested = true;
					v.3 = true;
				}
			}
			if last_attested {
				<Attestations<T>>::insert(claim_hash.clone(), existing_attestations_for_claim.clone());
				Ok(())
			} else {
				Err("no valid attestation found")
			}
		}
	}
}

decl_storage! {
	trait Store for Module<T: Trait> as Attestation {
		Attestations get(attestations): map T::Hash => Vec<(T::Hash,T::AccountId,T::Signature,bool)>;
	}
}


#[cfg(test)]
mod tests {
	use super::*;
	use parity_codec::alloc::vec::Vec;
	use system;
	use runtime_io::with_externalities;
	use primitives::{H256, H512, Blake2Hasher};
	use runtime_primitives::Ed25519Signature;
	use primitives::*;
	use parity_codec::Encode;

	use sr_primitives::{
		BuildStorage, traits::{BlakeTwo256}, testing::{Digest, DigestItem, Header}
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
	}
	type Attestation = Module<Test>;

	// This function basically just builds a genesis storage key/value store according to
	// our desired mockup.
	fn new_test_ext() -> sr_io::TestExternalities<Blake2Hasher> {
		let mut t = system::GenesisConfig::<Test>::default().build_storage().unwrap().0;
		// We use default for brevity, but you can configure as desired if needed.
		t.extend(balances::GenesisConfig::<Test>::default().build_storage().unwrap().0);
		t.into()
	}

	fn hash_to_u8<T : Encode> (hash : T) -> Vec<u8>{
		return hash.encode();
	}

	#[test]
	fn check_bad_signature() {
		with_externalities(&mut new_test_ext(), || {
			assert_err!(Attestation::add(Origin::signed(H256::from(1)), H256::from(2), Ed25519Signature::from(H512::from(3))), "bad signature");
			assert_err!(Attestation::revoke(Origin::signed(H256::from(1)), H256::from(2), Ed25519Signature::from(H512::from(3))), "bad signature");
		});
	}

	#[test]
	fn check_add_attestation() {
		with_externalities(&mut new_test_ext(), || {
			let pair = ed25519::Pair::from_seed(b"Alice                           ");
			let hash = H256::from(1);
			let bytes = hash_to_u8(hash);
			let signed = Ed25519Signature::from(pair.sign(&bytes));
			let account_hash = H256::from(pair.public().0);
			assert_ok!(Attestation::add(Origin::signed(account_hash.clone()), hash.clone(), signed.clone()));
			let existing_attestations_for_claim = Attestation::attestations(hash.clone());
			assert_eq!(existing_attestations_for_claim.len(), 1);
			assert_eq!(existing_attestations_for_claim[0].0, hash.clone());
			assert_eq!(existing_attestations_for_claim[0].1, account_hash.clone());
			assert_eq!(existing_attestations_for_claim[0].2, signed.clone());
			assert_eq!(existing_attestations_for_claim[0].3, false);
		});
	}

	#[test]
	fn check_revoke_attestation() {
		with_externalities(&mut new_test_ext(), || {
			let pair = ed25519::Pair::from_seed(b"Alice                           ");
			let hash = H256::from(1);
			let bytes = hash_to_u8(hash);
			let signed = Ed25519Signature::from(pair.sign(&bytes));
			let account_hash = H256::from(pair.public().0);
			assert_ok!(Attestation::add(Origin::signed(account_hash.clone()), hash.clone(), signed.clone()));
			assert_ok!(Attestation::revoke(Origin::signed(account_hash.clone()), hash.clone(), signed.clone()));
			let existing_attestations_for_claim = Attestation::attestations(hash.clone());
			assert_eq!(existing_attestations_for_claim.len(), 1);
			assert_eq!(existing_attestations_for_claim[0].0, hash.clone());
			assert_eq!(existing_attestations_for_claim[0].1, account_hash.clone());
			assert_eq!(existing_attestations_for_claim[0].2, signed.clone());
			assert_eq!(existing_attestations_for_claim[0].3, true);
		});
	}

	#[test]
	fn check_double_attestation() {
		with_externalities(&mut new_test_ext(), || {
			let pair = ed25519::Pair::from_seed(b"Alice                           ");
			let hash = H256::from(1);
			let bytes = hash_to_u8(hash);
			let signed = Ed25519Signature::from(pair.sign(&bytes));
			let account_hash = H256::from(pair.public().0);
			assert_ok!(Attestation::add(Origin::signed(account_hash.clone()), hash.clone(), signed.clone()));
			assert_err!(Attestation::add(Origin::signed(account_hash.clone()), hash.clone(), signed.clone()), "already attested");
		});
	}

	#[test]
	fn check_double_revoke_attestation() {
		with_externalities(&mut new_test_ext(), || {
			let pair = ed25519::Pair::from_seed(b"Alice                           ");
			let hash = H256::from(1);
			let bytes = hash_to_u8(hash);
			let signed = Ed25519Signature::from(pair.sign(&bytes));
			let account_hash = H256::from(pair.public().0);
			assert_ok!(Attestation::add(Origin::signed(account_hash.clone()), hash.clone(), signed.clone()));
			assert_ok!(Attestation::revoke(Origin::signed(account_hash.clone()), hash.clone(), signed.clone()));
			assert_err!(Attestation::revoke(Origin::signed(account_hash.clone()), hash.clone(), signed.clone()), "no valid attestation found");
		});
	}
}
