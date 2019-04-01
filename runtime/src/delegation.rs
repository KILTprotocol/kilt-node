
use rstd::result;
use rstd::prelude::*;
use runtime_primitives::traits::{Hash, CheckEqual, SimpleBitOps, Member, Verify, MaybeDisplay};
use support::{dispatch::Result, StorageMap, Parameter, decl_module, decl_storage};
use parity_codec::{Encode, Decode};
use core::default::Default;

use runtime_primitives::codec::Codec;
use {system::{self, ensure_signed}, super::ctype};
use runtime_primitives::verify_encoded_lazy;


bitflags! {
    #[derive(Encode, Decode)]
    pub struct Permissions: u32 {
        const ATTEST = 0b00000001;
        const DELEGATE = 0b00000010;
    }
}


impl Permissions {
    fn as_u8(&self) -> [u8;4] {
        let x: u32 = (*self).bits;
        let b1 : u8 = ((x >> 24) & 0xff) as u8;
        let b2 : u8 = ((x >> 16) & 0xff) as u8;
        let b3 : u8 = ((x >> 8) & 0xff) as u8;
        let b4 : u8 = (x & 0xff) as u8;
        return [b4, b3, b2, b1];
    }
}

impl Default for Permissions {
    fn default() -> Self {
        return Permissions::ATTEST;
    }
}

pub trait Trait: ctype::Trait + system::Trait {
    type Signer: From<Self::AccountId> + Member + Codec;
	type Signature: Verify<Signer = Self::Signer> + Member + Codec + Default;
    type DelegationNodeId: Parameter + Member + Codec + MaybeDisplay + SimpleBitOps 
            + Default + Copy + CheckEqual + rstd::hash::Hash + AsRef<[u8]> + AsMut<[u8]>;

    fn print_hash(hash: Self::Hash);
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		pub fn create_root(origin, root_id: T::DelegationNodeId, ctype_hash: T::Hash) -> Result {
			let sender = ensure_signed(origin)?;
            if <Root<T>>::exists(root_id) {
                return Err("root already exist")
            }
            if !<ctype::CTYPEs<T>>::exists(ctype_hash) {
                return Err("CTYPE does not exist")
            }

			::runtime_io::print("insert Delegation Root");
			<Root<T>>::insert(root_id.clone(), (ctype_hash.clone(), sender.clone(), false));
            return Ok(());
        }
		
        pub fn add_delegation(origin, delegation_id: T::DelegationNodeId, 
                root_id: T::DelegationNodeId, parent_id: Option<T::DelegationNodeId>, 
                delegate: T::AccountId, permissions: Permissions, delegate_signature: T::Signature) -> Result {
			let sender = ensure_signed(origin)?;
            if <Delegations<T>>::exists(delegation_id) {
                return Err("delegation already exist")
            }
            
            let hash_root = Self::calculate_hash(delegation_id, root_id, parent_id, permissions);
            if !verify_encoded_lazy(&delegate_signature, &&hash_root, &delegate.clone().into()) {
                // TODO: abort on signature error
                ::runtime_io::print("Error: signature does not match, hash:");
                T::print_hash(hash_root);
                return Err("bad delegate signature")
            }
            
            if <Root<T>>::exists(root_id) {
                let root = <Root<T>>::get(root_id.clone());
                match parent_id {
                    Some(p) => {
                        if <Delegations<T>>::exists(p) {
                            let parent = <Delegations<T>>::get(p.clone());
                            if !parent.2.eq(&sender) {
                                return Err("not owner of parent")
                            } else if (parent.3 & Permissions::DELEGATE) != Permissions::DELEGATE {
                                return Err("not authorized to delegate")
                            } else {
                                // TODO: check for cycles?
                    			::runtime_io::print("insert Delegation with parent");
                                <Delegations<T>>::insert(delegation_id.clone(), (root_id.clone(), Some(p.clone()), delegate, permissions, false));
                                Self::add_child(delegation_id.clone(), p.clone());
                            }
                        } else {
                            return Err("parent not found")
                        }
                    },
                    None => {
                        if !root.1.eq(&sender) {
                            return Err("not owner of root")        
                        }
                        ::runtime_io::print("insert Delegation without parent");
                        <Delegations<T>>::insert(delegation_id.clone(), (root_id.clone(), None, delegate, permissions, false));
                        Self::add_child(delegation_id.clone(), root_id.clone());
                    }
                }
            } else {
                return Err("root not found")
            }
            return Ok(());
        }

        pub fn revoke_root(origin, root_id: T::DelegationNodeId) -> Result {
			let sender = ensure_signed(origin)?;
            if !<Root<T>>::exists(root_id) {
                return Err("root not found")
            }
            let mut r = <Root<T>>::get(root_id.clone());
            if !r.1.eq(&sender) {
                return Err("not permitted to revoke")
            }
            if !r.2 {
                r.2 = true;
                <Root<T>>::insert(root_id.clone(), r);
                Self::revoke_children(&root_id);
            }

            return Ok(());
        }

        pub fn revoke_delegation(origin, delegation_id: T::DelegationNodeId) -> Result {
			let sender = ensure_signed(origin)?;
            if !<Delegations<T>>::exists(delegation_id) {
                return Err("delegation not found")
            }
            if !Self::is_delegating(&sender, &delegation_id)? {
                return Err("not permitted to revoke")
            }
            Self::revoke(&delegation_id);
            return Ok(());
        }
    }
}

impl<T: Trait> Module<T> {

    pub fn calculate_hash(delegation_id: T::DelegationNodeId, 
                root_id: T::DelegationNodeId, parent_id: Option<T::DelegationNodeId>, 
                permissions: Permissions) -> T::Hash {
        let mut hashed_values : Vec<u8> = delegation_id.as_ref().to_vec();
        hashed_values.extend_from_slice(root_id.as_ref());
        match parent_id {
            Some(p) => hashed_values.extend_from_slice(p.as_ref()),
            None => {}
        }
        hashed_values.extend_from_slice(permissions.as_u8().as_ref());
        let hash_root = T::Hashing::hash(&hashed_values);
        return hash_root;
    }

    pub fn is_delegating(account: &T::AccountId, delegation: &T::DelegationNodeId) -> result::Result<bool, &'static str> {
        if !<Delegations<T>>::exists(delegation) {
            return Err("delegation not found")
        }
        let d = <Delegations<T>>::get(delegation);
        if d.2.eq(account) {
            Ok(true)
        } else {
            match d.1 {
                None => {
                    let r = <Root<T>>::get(d.0.clone());
                    Ok(r.1.eq(account))
                },
                Some(p) => {
                    return Self::is_delegating(account, &p)
                }
            }
        }
    }

    fn revoke(delegation: &T::DelegationNodeId) {
        let mut d = <Delegations<T>>::get(delegation.clone());
        if !d.4 {
            d.4 = true;
            <Delegations<T>>::insert(delegation.clone(), d);
            Self::revoke_children(delegation);
        }
    }

    fn revoke_children(delegation: &T::DelegationNodeId) {
        if <Children<T>>::exists(delegation) {
            let children = <Children<T>>::get(delegation);
            for child in children {
                Self::revoke(&child);
            }
        }
    }

    fn add_child(child: T::DelegationNodeId, parent: T::DelegationNodeId) {
        let mut children = <Children<T>>::get(parent.clone());
        children.push(child);
        <Children<T>>::insert(parent, children);
    }
}

decl_storage! {
	trait Store for Module<T: Trait> as Delegation {
        // Root: root-id => (ctype-hash, account, revoked)
		pub Root get(root): map T::DelegationNodeId => (T::Hash,T::AccountId,bool); 
        // Delegations: delegation-id => (root-id, parent-id?, account, permissions, revoked)
		pub Delegations get(delegation): map T::DelegationNodeId => (T::DelegationNodeId,Option<T::DelegationNodeId>,T::AccountId,Permissions,bool); 
		// Children: root-or-delegation-id => [delegation-id]
        pub Children get(children): map T::DelegationNodeId => Vec<T::DelegationNodeId>; 
	}
}



#[cfg(test)]
mod tests {
	use super::*;
	use system;
	use runtime_io::with_externalities;
	use primitives::{H256, H512, Blake2Hasher, ed25519 as x25519};
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
		type AccountId = <x25519::Signature as Verify>::Signer;
		type Header = Header;
		type Event = ();
		type Log = DigestItem;
		type Lookup = IdentityLookup<Self::AccountId>;
	}
	
	impl ctype::Trait for Test {
	}

	impl Trait for Test {
		type Signature = x25519::Signature;
        type Signer = <x25519::Signature as Verify>::Signer;
		type DelegationNodeId = H256;

        fn print_hash(hash: Self::Hash) {
		    ::runtime_io::print(&hash.as_bytes()[..]);
	    }
	}

	type Ctype = ctype::Module<Test>;
    type Delegation = Module<Test>;

	fn hash_to_u8<T : Encode> (hash : T) -> Vec<u8>{
		return hash.encode();
	}

	fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
		system::GenesisConfig::<Test>::default().build_storage().unwrap().0.into()
	}

	#[test]
	fn check_add_and_revoke_delegations() {
		with_externalities(&mut new_test_ext(), || {
			let pair_alice = x25519::Pair::from_seed(*b"Alice                           ");
			let account_hash_alice = pair_alice.public();
			let pair_bob = x25519::Pair::from_seed(*b"Bob                             ");
			let account_hash_bob = pair_bob.public();
			let pair_charlie = x25519::Pair::from_seed(*b"Charlie                         ");
			let account_hash_charlie = pair_charlie.public();

			let ctype_hash = H256::from_low_u64_be(1);
			let id_level_0 = H256::from_low_u64_be(1);
			let id_level_1 = H256::from_low_u64_be(2);
			let id_level_2_1 = H256::from_low_u64_be(21);
			let id_level_2_2 = H256::from_low_u64_be(22);
			let id_level_2_2_1 = H256::from_low_u64_be(221);

			assert_ok!(Ctype::add(Origin::signed(account_hash_alice.clone()), ctype_hash.clone()));

			assert_ok!(Delegation::create_root(Origin::signed(account_hash_alice.clone()), id_level_0.clone(), ctype_hash.clone()));
			assert_err!(Delegation::create_root(Origin::signed(account_hash_alice.clone()), id_level_0.clone(), ctype_hash.clone()),
                "root already exist");
			assert_err!(Delegation::create_root(Origin::signed(account_hash_alice.clone()), id_level_1.clone(), H256::from_low_u64_be(2)),
                "CTYPE does not exist");

			assert_ok!(Delegation::add_delegation(Origin::signed(account_hash_alice.clone()), id_level_1.clone(), id_level_0.clone(), 
                None, account_hash_bob.clone(), Permissions::DELEGATE, 
                x25519::Signature::from(pair_bob.sign(&hash_to_u8(
                    Delegation::calculate_hash(id_level_1.clone(), id_level_0.clone(), None, Permissions::DELEGATE))))));
			assert_err!(Delegation::add_delegation(Origin::signed(account_hash_alice.clone()), id_level_1.clone(), id_level_0.clone(), 
                None, account_hash_bob.clone(), Permissions::DELEGATE, 
                x25519::Signature::from(pair_bob.sign(&hash_to_u8(
                    Delegation::calculate_hash(id_level_1.clone(), id_level_0.clone(), None, Permissions::DELEGATE))))),
                "delegation already exist");
            assert_err!(Delegation::add_delegation(Origin::signed(account_hash_bob.clone()), id_level_2_1.clone(), id_level_0.clone(), 
                Some(id_level_1.clone()), account_hash_charlie.clone(), Permissions::ATTEST, x25519::Signature::from_h512(H512::from_low_u64_be(0))),
                "bad delegate signature");
			assert_err!(Delegation::add_delegation(Origin::signed(account_hash_charlie.clone()), id_level_2_1.clone(), id_level_0.clone(), 
                None, account_hash_bob.clone(), Permissions::DELEGATE, 
                x25519::Signature::from(pair_bob.sign(&hash_to_u8(
                    Delegation::calculate_hash(id_level_2_1.clone(), id_level_0.clone(), None, Permissions::DELEGATE))))),
                "not owner of root");
			assert_err!(Delegation::add_delegation(Origin::signed(account_hash_alice.clone()), id_level_2_1.clone(), id_level_1.clone(), 
                None, account_hash_bob.clone(), Permissions::DELEGATE, 
                x25519::Signature::from(pair_bob.sign(&hash_to_u8(
                    Delegation::calculate_hash(id_level_2_1.clone(), id_level_1.clone(), None, Permissions::DELEGATE))))),
                "root not found");


			assert_ok!(Delegation::add_delegation(Origin::signed(account_hash_bob.clone()), id_level_2_1.clone(), id_level_0.clone(), 
                Some(id_level_1.clone()), account_hash_charlie.clone(), Permissions::ATTEST, 
                x25519::Signature::from(pair_charlie.sign(&hash_to_u8(
                    Delegation::calculate_hash(id_level_2_1.clone(), id_level_0.clone(), Some(id_level_1.clone()), Permissions::ATTEST))))));
            assert_err!(Delegation::add_delegation(Origin::signed(account_hash_alice.clone()), id_level_2_2.clone(), id_level_0.clone(), 
                Some(id_level_1.clone()), account_hash_charlie.clone(), Permissions::ATTEST, 
                x25519::Signature::from(pair_charlie.sign(&hash_to_u8(
                    Delegation::calculate_hash(id_level_2_2.clone(), id_level_0.clone(), Some(id_level_1.clone()), Permissions::ATTEST))))),
                "not owner of parent");
            assert_err!(Delegation::add_delegation(Origin::signed(account_hash_charlie.clone()), id_level_2_2.clone(), id_level_0.clone(), 
                Some(id_level_2_1.clone()), account_hash_alice.clone(), Permissions::ATTEST, 
                x25519::Signature::from(pair_alice.sign(&hash_to_u8(
                    Delegation::calculate_hash(id_level_2_2.clone(), id_level_0.clone(), Some(id_level_2_1.clone()), Permissions::ATTEST))))),
                "not authorized to delegate");
            assert_err!(Delegation::add_delegation(Origin::signed(account_hash_bob.clone()), id_level_2_2.clone(), id_level_0.clone(), 
                Some(id_level_0.clone()), account_hash_charlie.clone(), Permissions::ATTEST, 
                x25519::Signature::from(pair_charlie.sign(&hash_to_u8(
                    Delegation::calculate_hash(id_level_2_2.clone(), id_level_0.clone(), Some(id_level_0.clone()), Permissions::ATTEST))))),
                "parent not found");
			
            assert_ok!(Delegation::add_delegation(Origin::signed(account_hash_bob.clone()), id_level_2_2.clone(), id_level_0.clone(), 
                Some(id_level_1.clone()), account_hash_charlie.clone(), Permissions::ATTEST | Permissions::DELEGATE, 
                x25519::Signature::from(pair_charlie.sign(&hash_to_u8(
                    Delegation::calculate_hash(id_level_2_2.clone(), id_level_0.clone(), Some(id_level_1.clone()), 
                    Permissions::ATTEST | Permissions::DELEGATE))))));

            assert_ok!(Delegation::add_delegation(Origin::signed(account_hash_charlie.clone()), id_level_2_2_1.clone(), id_level_0.clone(), 
                Some(id_level_2_2.clone()), account_hash_alice.clone(), Permissions::ATTEST, 
                x25519::Signature::from(pair_alice.sign(&hash_to_u8(
                    Delegation::calculate_hash(id_level_2_2_1.clone(), id_level_0.clone(), Some(id_level_2_2.clone()), Permissions::ATTEST))))));

            
            let root = Delegation::root(id_level_0.clone());
            assert_eq!(root.0, ctype_hash.clone());
			assert_eq!(root.1, account_hash_alice.clone());
			assert_eq!(root.2, false);

            let delegation_1 = Delegation::delegation(id_level_1.clone());
            assert_eq!(delegation_1.0, id_level_0.clone());
			assert_eq!(delegation_1.1, None);
			assert_eq!(delegation_1.2, account_hash_bob.clone());
			assert_eq!(delegation_1.3, Permissions::DELEGATE);
			assert_eq!(delegation_1.4, false);

            let delegation_2 = Delegation::delegation(id_level_2_2.clone());
            assert_eq!(delegation_2.0, id_level_0.clone());
			assert_eq!(delegation_2.1, Some(id_level_1.clone()));
			assert_eq!(delegation_2.2, account_hash_charlie.clone());
			assert_eq!(delegation_2.3, Permissions::ATTEST | Permissions::DELEGATE);
			assert_eq!(delegation_2.4, false);

            let children = Delegation::children(id_level_1.clone());
			assert_eq!(children.len(), 2);
            assert_eq!(children[0], id_level_2_1.clone());
            assert_eq!(children[1], id_level_2_2.clone());

            // check is_delgating
            assert_eq!(Delegation::is_delegating(&account_hash_alice, &id_level_1), Ok(true));
            assert_eq!(Delegation::is_delegating(&account_hash_alice, &id_level_2_1), Ok(true));
            assert_eq!(Delegation::is_delegating(&account_hash_bob, &id_level_2_1), Ok(true));
            assert_eq!(Delegation::is_delegating(&account_hash_charlie, &id_level_2_1), Ok(true));
            assert_eq!(Delegation::is_delegating(&account_hash_charlie, &id_level_1), Ok(false));
            assert_eq!(Delegation::is_delegating(&account_hash_charlie, &id_level_0), Err("delegation not found"));

            assert_err!(Delegation::revoke_delegation(Origin::signed(account_hash_charlie.clone()), H256::from_low_u64_be(999)),
                "delegation not found");
            assert_err!(Delegation::revoke_delegation(Origin::signed(account_hash_charlie.clone()), id_level_1.clone()),
                "not permitted to revoke");
            assert_ok!(Delegation::revoke_delegation(Origin::signed(account_hash_charlie.clone()), id_level_2_2.clone()));
            
			assert_eq!(Delegation::delegation(id_level_2_2.clone()).4, true);
			assert_eq!(Delegation::delegation(id_level_2_2_1.clone()).4, true);

            assert_err!(Delegation::revoke_root(Origin::signed(account_hash_bob.clone()), H256::from_low_u64_be(999)),
                "root not found");
            assert_err!(Delegation::revoke_root(Origin::signed(account_hash_bob.clone()), id_level_0.clone()),
                "not permitted to revoke");
            assert_ok!(Delegation::revoke_root(Origin::signed(account_hash_alice.clone()), id_level_0.clone()));
            
			assert_eq!(Delegation::root(id_level_0.clone()).2, true);
			assert_eq!(Delegation::delegation(id_level_1.clone()).4, true);
			assert_eq!(Delegation::delegation(id_level_2_1.clone()).4, true);
		});
	}
    
}