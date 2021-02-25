mod v0 {
	use crate::{Config as Trait, Permissions};
	use frame_support::{decl_module, decl_storage};
	use sp_std::vec::Vec;

	decl_module! {
		pub struct Module<T: Trait> for enum Call where origin: T::Origin { }
	}

	decl_storage! {
		trait Store for Module<T: Trait> as Delegation {
			// Root: root-id => (ctype-hash, account, revoked)?
			pub Root get(fn root):map hasher(opaque_blake2_256) T::DelegationNodeId => Option<(T::Hash,T::AccountId,bool)>;
			// Delegations: delegation-id => (root-id, parent-id?, account, permissions, revoked)?
			pub Delegations get(fn delegation):map hasher(opaque_blake2_256) T::DelegationNodeId => Option<(T::DelegationNodeId,Option<T::DelegationNodeId>,T::AccountId,Permissions,bool)>;
			// Children: root-or-delegation-id => [delegation-id]
			pub Children get(fn children):map hasher(opaque_blake2_256) T::DelegationNodeId => Vec<T::DelegationNodeId>;
		}
	}
}

use crate::*;
use frame_support::{migration::StorageKeyIterator, Blake2_256};

pub fn migrate_to_struct<T: Config>() {
	let mut count = 0;
	StorageKeyIterator::<
		T::DelegationNodeId,
		Option<(
			T::DelegationNodeId,
			Option<T::DelegationNodeId>,
			T::AccountId,
			Permissions,
			bool,
		)>,
		Blake2_256,
	>::new(Delegations::<T>::module_prefix(), b"Voting")
	.for_each(|(node_id, (stake, votes))| {
		// Insert a new value into the same location, thus no need to do `.drain()`.
		let voter = DelegationNode {
			root_id: (),
			parent: (),
			owner: (),
			permissions: (),
			revoked: (),
		};
		Delegations::<T>::insert(node_id, voter);
		count += 1;
	});
}
