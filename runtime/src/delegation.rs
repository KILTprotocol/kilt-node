
use rstd::prelude::*;
use runtime_primitives::traits::{Hash, CheckEqual, SimpleBitOps, Member, Verify, MaybeDisplay};
use support::{dispatch::Result, StorageMap, Parameter, decl_module, decl_storage};
use parity_codec_derive::{Encode, Decode};
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
        return [b1, b2, b3, b4];
    }
}

impl Default for Permissions {
    fn default() -> Self {
        return Permissions::ATTEST;
    }
}

pub trait Trait: ctype::Trait + system::Trait {
	type Signature: Verify<Signer = Self::AccountId> + Member + Codec + Default;
    type DelegationNodeId: Parameter + Member + Codec + MaybeDisplay + SimpleBitOps 
            + Default + Copy + CheckEqual + rstd::hash::Hash + AsRef<[u8]> + AsMut<[u8]>;
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
            
            let mut hashed_values : Vec<Vec<u8>> = Vec::new();
            hashed_values.push(delegation_id.as_ref().to_vec());
            hashed_values.push(root_id.as_ref().to_vec());
            match parent_id {
                Some(p) => hashed_values.push(p.as_ref().to_vec()),
                None => {}
            }
            let p = permissions.as_u8();
            hashed_values.push((&p).to_vec());
            let hashed_value_array = hashed_values.iter().map(Vec::as_slice).collect::<Vec<_>>();
            let hash_root = T::Hashing::enumerated_trie_root(&hashed_value_array);
            if !verify_encoded_lazy(&delegate_signature, &hash_root, &delegate) {
                // TODO: abort on signature error
                ::runtime_io::print("WARNING: SIGNATURE DOES NOT MATCH!");
                // return Err("bad delegate signature")
            }
            
            if <Root<T>>::exists(root_id) {
                let root = <Root<T>>::get(root_id.clone());
                match parent_id {
                    Some(p) => {
                        if <Delegations<T>>::exists(p) {
                            let parent = <Delegations<T>>::get(p.clone());
                            if parent.2 != sender {
                                return Err("not owner of parent")
                            } else if parent.3 & Permissions::DELEGATE != Permissions::DELEGATE {
                                return Err("not authorized to delegate")
                            } else {
                                // TODO: check for cycles
                    			::runtime_io::print("insert Delegation with parent");
                                <Delegations<T>>::insert(delegation_id.clone(), (root_id.clone(), Some(p.clone()), delegate, permissions, false));
                            }
                        } else {
                            return Err("parent not found")
                        }
                    },
                    None => {
                        if root.1 != sender {
                            return Err("not owner of root")        
                        }
                        ::runtime_io::print("insert Delegation without parent");
                        <Delegations<T>>::insert(delegation_id.clone(), (root_id.clone(), None, delegate, permissions, false));
                    }
                }
            } else {
                return Err("root not found")
            }
            return Ok(());
        }
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
