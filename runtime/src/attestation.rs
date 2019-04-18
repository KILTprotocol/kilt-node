
use rstd::result;
use rstd::prelude::*;
use support::{dispatch::Result, StorageMap, decl_module, decl_storage, decl_event};
use {system, super::delegation, super::ctype, super::error, system::ensure_signed};

pub trait Trait: system::Trait + delegation::Trait + error::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_event!(
	pub enum Event<T> where <T as system::Trait>::AccountId, <T as system::Trait>::Hash, 
			<T as delegation::Trait>::DelegationNodeId {
		/// An attestation has been added
		AttestationCreated(AccountId, Hash, Hash, Option<DelegationNodeId>),
		/// An attestation has been revoked
		AttestationRevoked(AccountId, Hash),
	}
);

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		fn deposit_event<T>() = default;

		pub fn add(origin, claim_hash: T::Hash, ctype_hash: T::Hash, delegation_id: Option<T::DelegationNodeId>) -> Result {
			let sender = ensure_signed(origin)?;
			if !<ctype::CTYPEs<T>>::exists(ctype_hash) {
				return Self::error(<ctype::Module<T>>::ERROR_CTYPE_NOT_FOUND);
			}

			match delegation_id {
				Some(d) => {
					if !<delegation::Delegations<T>>::exists(d.clone()) {
						return Self::error(<delegation::Module<T>>::ERROR_DELEGATION_NOT_FOUND);
					}
					let delegation = <delegation::Delegations<T>>::get(d.clone());
					if delegation.4 {
						return Self::error(Self::ERROR_DELEGATION_REVOKED);
					} else if !delegation.2.eq(&sender) {
						return Self::error(Self::ERROR_NOT_DELEGATED_TO_ATTESTER);
					} else if (delegation.3 & delegation::Permissions::ATTEST) != delegation::Permissions::ATTEST {
						return Self::error(Self::ERROR_DELEGATION_NOT_AUTHORIZED_TO_ATTEST);
					} else {
						let root = <delegation::Root<T>>::get(delegation.0.clone());
						if !root.0.eq(&ctype_hash) {
							return Self::error(Self::ERROR_CTYPE_OF_DELEGATION_NOT_MATCHING);
						}
					}
				},
				None => {}
			}

			if <Attestations<T>>::exists(claim_hash.clone()) {
				return Self::error(Self::ERROR_ALREADY_ATTESTED);
			}

			::runtime_io::print("insert Attestation");
			<Attestations<T>>::insert(claim_hash.clone(), (ctype_hash.clone(), sender.clone(), delegation_id.clone(), false));
			match delegation_id {
				Some(d) => {
					let mut delegated_attestations = <DelegatedAttestations<T>>::get(d);
					delegated_attestations.push(claim_hash.clone());
					<DelegatedAttestations<T>>::insert(d.clone(), delegated_attestations);
				},
				None => {}
			}

			Self::deposit_event(RawEvent::AttestationCreated(sender.clone(), claim_hash.clone(), 
					ctype_hash.clone(), delegation_id.clone()));
			Ok(())
		}

		pub fn revoke(origin, claim_hash: T::Hash) -> Result {
			let sender = ensure_signed(origin)?;

			if !<Attestations<T>>::exists(claim_hash.clone()) {
				return Self::error(Self::ERROR_ATTESTATION_NOT_FOUND);
			}
			
			let mut existing_attestation = <Attestations<T>>::get(claim_hash.clone());
			if !existing_attestation.1.eq(&sender) {
				match existing_attestation.2 {
					Some(d) => {
						if !Self::is_delegating(&sender, &d)? {
							return Self::error(Self::ERROR_NOT_PERMITTED_TO_REVOKE_ATTESTATION);
						}
					},
					None => {
							return Self::error(Self::ERROR_NOT_PERMITTED_TO_REVOKE_ATTESTATION);
					}
				}
			}
			
			if existing_attestation.3 {
				return Self::error(Self::ERROR_ALREADY_REVOKED);
			}

			::runtime_io::print("revoking Attestation");
			existing_attestation.3 = true;
			<Attestations<T>>::insert(claim_hash.clone(), existing_attestation.clone());
			Self::deposit_event(RawEvent::AttestationRevoked(sender.clone(), claim_hash.clone()));
			Ok(())
		}
	}
}

impl<T: Trait> Module<T> {

    const ERROR_BASE: u16 = 200;
    const ERROR_ALREADY_ATTESTED : error::ErrorType = (Self::ERROR_BASE + 1, "already attested");
    const ERROR_ALREADY_REVOKED : error::ErrorType = (Self::ERROR_BASE + 2, "already revoked");
    const ERROR_ATTESTATION_NOT_FOUND : error::ErrorType = (Self::ERROR_BASE + 3, "attestation not found");
    const ERROR_DELEGATION_REVOKED : error::ErrorType = (Self::ERROR_BASE + 4, "delegation revoked");
    const ERROR_NOT_DELEGATED_TO_ATTESTER : error::ErrorType = (Self::ERROR_BASE + 5, "not delegated to attester");
    const ERROR_DELEGATION_NOT_AUTHORIZED_TO_ATTEST : error::ErrorType = (Self::ERROR_BASE + 6, "delegation not authorized to attest");
    const ERROR_CTYPE_OF_DELEGATION_NOT_MATCHING : error::ErrorType = (Self::ERROR_BASE + 7, "CTYPE of delegation does not match");
    const ERROR_NOT_PERMITTED_TO_REVOKE_ATTESTATION : error::ErrorType = (Self::ERROR_BASE + 8, "not permitted to revoke attestation");
	

    pub fn error(error_type: error::ErrorType) -> Result {
        return <error::Module<T>>::error(error_type);
    }
	
    fn is_delegating(account: &T::AccountId, delegation: &T::DelegationNodeId) -> result::Result<bool, &'static str> {
		<delegation::Module<T>>::is_delegating(account, delegation)
	}
}

decl_storage! {
	trait Store for Module<T: Trait> as Attestation {
		// Attestations: claim-hash -> [(ctype-hash, account, delegation-id?, revoked)]
		Attestations get(attestations): map T::Hash => (T::Hash,T::AccountId,Option<T::DelegationNodeId>,bool);
		// DelegatedAttestations: delegation-id -> [claim-hash]
		DelegatedAttestations get(delegated_attestations): map T::DelegationNodeId => Vec<T::Hash>;
	}
}


#[cfg(test)]
mod tests {
	use super::*;
	use system;
	use runtime_io::with_externalities;
	use primitives::{H256, Blake2Hasher, ed25519 as x25519};
	use primitives::*;
	use support::{impl_outer_origin, assert_ok, assert_err};
	use parity_codec::Encode;

	use runtime_primitives::{
		BuildStorage, traits::{BlakeTwo256, IdentityLookup, Verify}, testing::{Digest, DigestItem, Header}
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
		type AccountId = <x25519::Signature as Verify>::Signer;
		type Header = Header;
		type Event = ();
		type Log = DigestItem;
		type Lookup = IdentityLookup<Self::AccountId>;
	}
	
	impl ctype::Trait for Test {
		type Event = ();
	}

    impl error::Trait for Test {
        type ErrorCode = u16;
        type Event = ();
    }

	impl delegation::Trait for Test {
		type Event = ();
		type Signature = x25519::Signature;
		type Signer = <x25519::Signature as Verify>::Signer;
		type DelegationNodeId = H256;
		fn print_hash(hash: Self::Hash) {
			::runtime_io::print(&hash.as_bytes()[..]);
		}
	}

	impl Trait for Test {
		type Event = ();
	}

	type Attestation = Module<Test>;
	type CType = ctype::Module<Test>;
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
			let pair = x25519::Pair::from_seed(*b"Alice                           ");
			let hash = H256::from_low_u64_be(1);
			let account_hash = pair.public();
			assert_ok!(CType::add(Origin::signed(account_hash.clone()), hash.clone()));
			assert_ok!(Attestation::add(Origin::signed(account_hash.clone()), hash.clone(), hash.clone(), None));
			let existing_attestation_for_claim = Attestation::attestations(hash.clone());
			assert_eq!(existing_attestation_for_claim.0, hash.clone());
			assert_eq!(existing_attestation_for_claim.1, account_hash.clone());
			assert_eq!(existing_attestation_for_claim.3, false);
		});
	}

	#[test]
	fn check_revoke_attestation() {
		with_externalities(&mut new_test_ext(), || {
			let pair = x25519::Pair::from_seed(*b"Alice                           ");
			let hash = H256::from_low_u64_be(1);
			let account_hash = pair.public();
			assert_ok!(CType::add(Origin::signed(account_hash.clone()), hash.clone()));
			assert_ok!(Attestation::add(Origin::signed(account_hash.clone()), hash.clone(), hash.clone(), None));
			assert_ok!(Attestation::revoke(Origin::signed(account_hash.clone()), hash.clone()));
			let existing_attestation_for_claim = Attestation::attestations(hash.clone());
			assert_eq!(existing_attestation_for_claim.0, hash.clone());
			assert_eq!(existing_attestation_for_claim.1, account_hash.clone());
			assert_eq!(existing_attestation_for_claim.3, true);
		});
	}

	#[test]
	fn check_double_attestation() {
		with_externalities(&mut new_test_ext(), || {
			let pair = x25519::Pair::from_seed(*b"Alice                           ");
			let hash = H256::from_low_u64_be(1);
			let account_hash = pair.public();
			assert_ok!(CType::add(Origin::signed(account_hash.clone()), hash.clone()));
			assert_ok!(Attestation::add(Origin::signed(account_hash.clone()), hash.clone(), hash.clone(), None));
			assert_err!(Attestation::add(Origin::signed(account_hash.clone()), hash.clone(), hash.clone(), None), 
				Attestation::ERROR_ALREADY_ATTESTED.1);
		});
	}

	#[test]
	fn check_double_revoke_attestation() {
		with_externalities(&mut new_test_ext(), || {
			let pair = x25519::Pair::from_seed(*b"Alice                           ");
			let hash = H256::from_low_u64_be(1);
			let account_hash = pair.public();
			assert_ok!(CType::add(Origin::signed(account_hash.clone()), hash.clone()));
			assert_ok!(Attestation::add(Origin::signed(account_hash.clone()), hash.clone(), hash.clone(), None));
			assert_ok!(Attestation::revoke(Origin::signed(account_hash.clone()), hash.clone()));
			assert_err!(Attestation::revoke(Origin::signed(account_hash.clone()), hash.clone()), 
				Attestation::ERROR_ALREADY_REVOKED.1);
		});
	}

	#[test]
	fn check_revoke_unknown() {
		with_externalities(&mut new_test_ext(), || {
			let pair = x25519::Pair::from_seed(*b"Alice                           ");
			let hash = H256::from_low_u64_be(1);
			let account_hash = pair.public();
			assert_err!(Attestation::revoke(Origin::signed(account_hash.clone()), hash.clone()), 
				Attestation::ERROR_ATTESTATION_NOT_FOUND.1);
		});
	}

	#[test]
	fn check_revoke_not_permitted() {
		with_externalities(&mut new_test_ext(), || {
			let pair_alice = x25519::Pair::from_seed(*b"Alice                           ");
			let account_hash_alice = pair_alice.public();
			let pair_bob = x25519::Pair::from_seed(*b"Bob                             ");
			let account_hash_bob = pair_bob.public();
			let hash = H256::from_low_u64_be(1);
			assert_ok!(CType::add(Origin::signed(account_hash_alice.clone()), hash.clone()));
			assert_ok!(Attestation::add(Origin::signed(account_hash_alice.clone()), hash.clone(), hash.clone(), None));
			assert_err!(Attestation::revoke(Origin::signed(account_hash_bob.clone()), hash.clone()), 
				Attestation::ERROR_NOT_PERMITTED_TO_REVOKE_ATTESTATION.1);
		});
	}

	#[test]
	fn check_add_attestation_with_delegation() {
		with_externalities(&mut new_test_ext(), || {
			let pair_alice = x25519::Pair::from_seed(*b"Alice                           ");
			let account_hash_alice = pair_alice.public();
			let pair_bob = x25519::Pair::from_seed(*b"Bob                             ");
			let account_hash_bob = pair_bob.public();
			let pair_charlie = x25519::Pair::from_seed(*b"Charlie                         ");
			let account_hash_charlie = pair_charlie.public();

			let ctype_hash = H256::from_low_u64_be(1);
			let other_ctype_hash = H256::from_low_u64_be(2);
			let claim_hash = H256::from_low_u64_be(1);
			
			let delegation_root = H256::from_low_u64_be(0);
			let delegation_1 = H256::from_low_u64_be(1);
			let delegation_2 = H256::from_low_u64_be(2);

			assert_ok!(CType::add(Origin::signed(account_hash_alice.clone()), ctype_hash.clone()));

			assert_err!(Attestation::add(Origin::signed(account_hash_alice.clone()), claim_hash.clone(), ctype_hash.clone(), Some(delegation_1)),
				Delegation::ERROR_DELEGATION_NOT_FOUND.1);

			assert_ok!(Delegation::create_root(Origin::signed(account_hash_alice.clone()), delegation_root.clone(), ctype_hash.clone()));
			assert_ok!(Delegation::add_delegation(Origin::signed(account_hash_alice.clone()), delegation_1.clone(), delegation_root.clone(), 
                None, account_hash_bob.clone(), delegation::Permissions::DELEGATE, 
                x25519::Signature::from(pair_bob.sign(&hash_to_u8(
                    Delegation::calculate_hash(delegation_1.clone(), delegation_root.clone(), None, delegation::Permissions::DELEGATE))))));
			assert_ok!(Delegation::add_delegation(Origin::signed(account_hash_alice.clone()), delegation_2.clone(), delegation_root.clone(), 
                None, account_hash_bob.clone(), delegation::Permissions::ATTEST, 
                x25519::Signature::from(pair_bob.sign(&hash_to_u8(
                    Delegation::calculate_hash(delegation_2.clone(), delegation_root.clone(), None, delegation::Permissions::ATTEST))))));

			assert_err!(Attestation::add(Origin::signed(account_hash_bob.clone()), claim_hash.clone(), other_ctype_hash.clone(), Some(delegation_2)),
				CType::ERROR_CTYPE_NOT_FOUND.1);
			assert_ok!(CType::add(Origin::signed(account_hash_alice.clone()), other_ctype_hash.clone()));
			assert_err!(Attestation::add(Origin::signed(account_hash_bob.clone()), claim_hash.clone(), other_ctype_hash.clone(), Some(delegation_2)),
				Attestation::ERROR_CTYPE_OF_DELEGATION_NOT_MATCHING.1);
			assert_err!(Attestation::add(Origin::signed(account_hash_alice.clone()), claim_hash.clone(), ctype_hash.clone(), Some(delegation_2)),
				Attestation::ERROR_NOT_DELEGATED_TO_ATTESTER.1);
			assert_err!(Attestation::add(Origin::signed(account_hash_bob.clone()), claim_hash.clone(), ctype_hash.clone(), Some(delegation_1)),
				Attestation::ERROR_DELEGATION_NOT_AUTHORIZED_TO_ATTEST.1);
			assert_ok!(Attestation::add(Origin::signed(account_hash_bob.clone()), claim_hash.clone(), ctype_hash.clone(), Some(delegation_2)));

			let existing_attestations_for_delegation = Attestation::delegated_attestations(delegation_2.clone());
			assert_eq!(existing_attestations_for_delegation.len(), 1);
			assert_eq!(existing_attestations_for_delegation[0], claim_hash.clone());
			
			assert_ok!(Delegation::revoke_root(Origin::signed(account_hash_alice.clone()), delegation_root.clone()));
			assert_err!(Attestation::add(Origin::signed(account_hash_bob.clone()), claim_hash.clone(), ctype_hash.clone(), Some(delegation_2)),
				Attestation::ERROR_DELEGATION_REVOKED.1);

			assert_err!(Attestation::revoke(Origin::signed(account_hash_charlie.clone()), claim_hash.clone()),
				Attestation::ERROR_NOT_PERMITTED_TO_REVOKE_ATTESTATION.1);
			assert_ok!(Attestation::revoke(Origin::signed(account_hash_alice.clone()), claim_hash.clone()));
		});
	}
}
