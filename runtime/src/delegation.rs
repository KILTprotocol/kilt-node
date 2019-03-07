
use rstd::prelude::*;
use runtime_primitives::traits::{CheckEqual, SimpleBitOps, Member, MaybeDisplay};
use support::{dispatch::Result, StorageMap, Parameter, decl_module, decl_storage};
use parity_codec_derive::{Encode, Decode};
use core::default::Default;

use runtime_primitives::codec::Codec;
use {balances, system::{self, ensure_signed}, super::ctype};


bitflags! {
    #[derive(Encode, Decode)]
    pub struct Permissions: u32 {
        const ATTEST = 0b00000001;
        const DELEGATE = 0b00000010;
    }
}

impl Default for Permissions {
    fn default() -> Self {
        
        return Permissions::ATTEST;
    }
}

pub trait Trait: ctype::Trait + balances::Trait {
    type DelegationNodeId: Parameter + Member + Codec + MaybeDisplay + SimpleBitOps 
            + Default + Copy + CheckEqual + rstd::hash::Hash + AsRef<[u8]> + AsMut<[u8]>;
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn create_root(origin, root_id: T::DelegationNodeId, ctype_hash: T::Hash) -> Result {
			let sender = ensure_signed(origin)?;
            if <Root<T>>::exists(root_id) {
                return Err("root already exist")
            }
            if !<ctype::CTYPEs<T>>::exists(ctype_hash) {
                return Err("CTYPE does not exist")
            }

			<Root<T>>::insert(root_id.clone(), (ctype_hash.clone(), sender.clone(), false));

            return Ok(());
        }
		
        fn add_delegation(origin, delegation_id: T::DelegationNodeId, 
                root_id: T::DelegationNodeId, parent_id: Option<T::DelegationNodeId>, 
                permissions: Permissions) -> Result {
			let sender = ensure_signed(origin)?;
            if <Delegations<T>>::exists(delegation_id) {
                return Err("delegation already exist")
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
                                <Delegations<T>>::insert(delegation_id.clone(), (root_id.clone(), Some(p.clone()), sender, permissions, false));
                            }
                        } else {
                            return Err("parent not found")
                        }
                    },
                    None => {
                        if root.1 != sender {
                            return Err("not owner of root")        
                        }
                        <Delegations<T>>::insert(delegation_id.clone(), (root_id.clone(), None, sender, permissions, false));
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
