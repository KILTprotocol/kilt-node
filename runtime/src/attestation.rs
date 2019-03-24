
use rstd::result;
use rstd::prelude::*;
use support::{dispatch::Result, StorageMap, decl_module, decl_storage};
use {system, super::delegation, super::ctype, system::ensure_signed};

pub trait Trait: system::Trait + delegation::Trait {
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		pub fn add(origin, claim_hash: T::Hash, ctype_hash: T::Hash, delegation_id: Option<T::DelegationNodeId>) -> Result {
			let sender = ensure_signed(origin)?;
			if !<ctype::CTYPEs<T>>::exists(ctype_hash) {
				return Err("CTYPE not found")
			}

			match delegation_id {
				Some(d) => {
					if !<delegation::Delegations<T>>::exists(d.clone()) {
						return Err("delegation not found")
					}
					let delegation = <delegation::Delegations<T>>::get(d.clone());
					if delegation.4 {
						return Err("delegation revoked")
					} else if !delegation.2.eq(&sender) {
						return Err("not delegated to attester")
					} else if (delegation.3 & delegation::Permissions::ATTEST) != delegation::Permissions::ATTEST {
						return Err("delegation not authorized to attest")
					} else {
						let root = <delegation::Root<T>>::get(delegation.0.clone());
						if !root.0.eq(&ctype_hash) {
							return Err("CTYPE of delegation does not match")
						}
					}
				},
				None => {}
			}

			let mut existing_attestations_for_claim = <Attestations<T>>::get(claim_hash.clone());
			let mut last_attested : Option<(T::Hash,T::AccountId,Option<T::DelegationNodeId>,bool)> = None;
			for v in existing_attestations_for_claim.clone() {
				if v.1.eq(&sender) && v.2.eq(&delegation_id) {
					last_attested = Some(v);
					break;
				}
			}
			match last_attested {
				Some(_v)	=> return Err("already attested"),
				None	=> {
					::runtime_io::print("insert Attestation");
					existing_attestations_for_claim.push((ctype_hash.clone(), sender.clone(), delegation_id.clone(), false));
					<Attestations<T>>::insert(claim_hash.clone(), existing_attestations_for_claim);
					match delegation_id {
						Some(d) => {
							let mut delegated_attestations = <DelegatedAttestations<T>>::get(d);
							delegated_attestations.push(claim_hash.clone());
							<DelegatedAttestations<T>>::insert(d.clone(), delegated_attestations);
						},
						None => {}
					}
					Ok(())
				},
			}
		}

		pub fn revoke(origin, claim_hash: T::Hash) -> Result {
			let sender = ensure_signed(origin)?;

			let mut last_attested : bool = false;
			let mut existing_attestations_for_claim = <Attestations<T>>::get(claim_hash.clone());
			for v in existing_attestations_for_claim.iter_mut() {
				if !v.3 {
					if v.1.eq(&sender) {
						last_attested = true;
						v.3 = true;
					} else {
						// check delegator in case of delegation
						match v.2 {
							Some(d) => {
								if Self::is_delegating(&sender, &d)? {
									last_attested = true;
									v.3 = true;
								}
							},
							None => {}
						}
					}
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

impl<T: Trait> Module<T> {
    fn is_delegating(account: &T::AccountId, delegation: &T::DelegationNodeId) -> result::Result<bool, &'static str> {
		<delegation::Module<T>>::is_delegating(account, delegation)
	}
}

decl_storage! {
	trait Store for Module<T: Trait> as Attestation {
		// Attestations: claim-hash -> [(ctype-hash, account, delegation-id?, revoked)]
		Attestations get(attestations): map T::Hash => Vec<(T::Hash,T::AccountId,Option<T::DelegationNodeId>,bool)>;
		// DelegatedAttestations: delegation-id -> [claim-hash]
		DelegatedAttestations get(delegated_attestations): map T::DelegationNodeId => Vec<T::Hash>;
	}
}


#[cfg(test)]
mod tests {
	use super::*;
	use system;
	use runtime_io::with_externalities;
	use primitives::{H256, Blake2Hasher};
	use runtime_primitives::Ed25519Signature;
	use primitives::*;
	use support::{impl_outer_origin, assert_ok, assert_err};
	use parity_codec::Encode;

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
	
	impl ctype::Trait for Test {
	}

	impl delegation::Trait for Test {
		type Signature = Ed25519Signature;
		type DelegationNodeId = H256;
		fn print_hash(hash: Self::Hash) {
			::runtime_io::print(&hash.as_bytes()[..]);
		}
}

	impl Trait for Test {
	}

	type Attestation = Module<Test>;
	type Ctype = ctype::Module<Test>;
	type Delegation = delegation::Module<Test>;

	fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
		system::GenesisConfig::<Test>::default().build_storage().unwrap().0.into()
	}

	fn hash_to_u8<T : Encode> (hash : T) -> Vec<u8>{
		return hash.encode();
	}


	#[test]
	fn check_add_attestation() {
		with_externalities(&mut new_test_ext(), || {
			let pair = ed25519::Pair::from_seed(b"Alice                           ");
			let hash = H256::from_low_u64_be(1);
			let account_hash = H256::from(pair.public().0);
			assert_ok!(Ctype::add(Origin::signed(account_hash.clone()), hash.clone()));
			assert_ok!(Attestation::add(Origin::signed(account_hash.clone()), hash.clone(), hash.clone(), None));
			let existing_attestations_for_claim = Attestation::attestations(hash.clone());
			assert_eq!(existing_attestations_for_claim.len(), 1);
			assert_eq!(existing_attestations_for_claim[0].0, hash.clone());
			assert_eq!(existing_attestations_for_claim[0].1, account_hash.clone());
			assert_eq!(existing_attestations_for_claim[0].3, false);
		});
	}

	#[test]
	fn check_revoke_attestation() {
		with_externalities(&mut new_test_ext(), || {
			let pair = ed25519::Pair::from_seed(b"Alice                           ");
			let hash = H256::from_low_u64_be(1);
			let account_hash = H256::from(pair.public().0);
			assert_ok!(Ctype::add(Origin::signed(account_hash.clone()), hash.clone()));
			assert_ok!(Attestation::add(Origin::signed(account_hash.clone()), hash.clone(), hash.clone(), None));
			assert_ok!(Attestation::revoke(Origin::signed(account_hash.clone()), hash.clone()));
			let existing_attestations_for_claim = Attestation::attestations(hash.clone());
			assert_eq!(existing_attestations_for_claim.len(), 1);
			assert_eq!(existing_attestations_for_claim[0].0, hash.clone());
			assert_eq!(existing_attestations_for_claim[0].1, account_hash.clone());
			assert_eq!(existing_attestations_for_claim[0].3, true);
		});
	}

	#[test]
	fn check_double_attestation() {
		with_externalities(&mut new_test_ext(), || {
			let pair = ed25519::Pair::from_seed(b"Alice                           ");
			let hash = H256::from_low_u64_be(1);
			let account_hash = H256::from(pair.public().0);
			assert_ok!(Ctype::add(Origin::signed(account_hash.clone()), hash.clone()));
			assert_ok!(Attestation::add(Origin::signed(account_hash.clone()), hash.clone(), hash.clone(), None));
			assert_err!(Attestation::add(Origin::signed(account_hash.clone()), hash.clone(), hash.clone(), None), "already attested");
		});
	}

	#[test]
	fn check_double_revoke_attestation() {
		with_externalities(&mut new_test_ext(), || {
			let pair = ed25519::Pair::from_seed(b"Alice                           ");
			let hash = H256::from_low_u64_be(1);
			let account_hash = H256::from(pair.public().0);
			assert_ok!(Ctype::add(Origin::signed(account_hash.clone()), hash.clone()));
			assert_ok!(Attestation::add(Origin::signed(account_hash.clone()), hash.clone(), hash.clone(), None));
			assert_ok!(Attestation::revoke(Origin::signed(account_hash.clone()), hash.clone()));
			assert_err!(Attestation::revoke(Origin::signed(account_hash.clone()), hash.clone()), "no valid attestation found");
		});
	}


	#[test]
	fn check_add_attestation_withdelegation() {
		with_externalities(&mut new_test_ext(), || {
			let pair_alice = ed25519::Pair::from_seed(b"Alice                           ");
			let account_hash_alice = H256::from(pair_alice.public().0);
			let pair_bob = ed25519::Pair::from_seed(b"Bob                             ");
			let account_hash_bob = H256::from(pair_bob.public().0);

			let ctype_hash = H256::from_low_u64_be(1);
			let other_ctype_hash = H256::from_low_u64_be(2);
			let claim_hash = H256::from_low_u64_be(1);
			
			let delegation_root = H256::from_low_u64_be(0);
			let delegation_1 = H256::from_low_u64_be(1);
			let delegation_2 = H256::from_low_u64_be(2);

			assert_ok!(Ctype::add(Origin::signed(account_hash_alice.clone()), ctype_hash.clone()));

			assert_err!(Attestation::add(Origin::signed(account_hash_alice.clone()), claim_hash.clone(), ctype_hash.clone(), Some(delegation_1)),
				"delegation not found");

			assert_ok!(Delegation::create_root(Origin::signed(account_hash_alice.clone()), delegation_root.clone(), ctype_hash.clone()));
			assert_ok!(Delegation::add_delegation(Origin::signed(account_hash_alice.clone()), delegation_1.clone(), delegation_root.clone(), 
                None, account_hash_bob.clone(), delegation::Permissions::DELEGATE, 
                Ed25519Signature::from(pair_bob.sign(&hash_to_u8(
                    Delegation::calculate_hash(delegation_1.clone(), delegation_root.clone(), None, delegation::Permissions::DELEGATE))))));
			assert_ok!(Delegation::add_delegation(Origin::signed(account_hash_alice.clone()), delegation_2.clone(), delegation_root.clone(), 
                None, account_hash_bob.clone(), delegation::Permissions::ATTEST, 
                Ed25519Signature::from(pair_bob.sign(&hash_to_u8(
                    Delegation::calculate_hash(delegation_2.clone(), delegation_root.clone(), None, delegation::Permissions::ATTEST))))));

			assert_err!(Attestation::add(Origin::signed(account_hash_bob.clone()), claim_hash.clone(), other_ctype_hash.clone(), Some(delegation_2)),
				"CTYPE not found");
			assert_ok!(Ctype::add(Origin::signed(account_hash_alice.clone()), other_ctype_hash.clone()));
			assert_err!(Attestation::add(Origin::signed(account_hash_bob.clone()), claim_hash.clone(), other_ctype_hash.clone(), Some(delegation_2)),
				"CTYPE of delegation does not match");
			assert_err!(Attestation::add(Origin::signed(account_hash_alice.clone()), claim_hash.clone(), ctype_hash.clone(), Some(delegation_2)),
				"not delegated to attester");
			assert_err!(Attestation::add(Origin::signed(account_hash_bob.clone()), claim_hash.clone(), ctype_hash.clone(), Some(delegation_1)),
				"delegation not authorized to attest");
			assert_ok!(Attestation::add(Origin::signed(account_hash_bob.clone()), claim_hash.clone(), ctype_hash.clone(), Some(delegation_2)));

			let existing_attestations_for_delegation = Attestation::delegated_attestations(delegation_2.clone());
			assert_eq!(existing_attestations_for_delegation.len(), 1);
			assert_eq!(existing_attestations_for_delegation[0], claim_hash.clone());
			
			assert_ok!(Delegation::revoke_root(Origin::signed(account_hash_alice.clone()), delegation_root.clone()));
			assert_err!(Attestation::add(Origin::signed(account_hash_bob.clone()), claim_hash.clone(), ctype_hash.clone(), Some(delegation_2)),
				"delegation revoked");
		});
	}
}
