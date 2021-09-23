// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2021 BOTLabs GmbH

// The KILT Blockchain is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The KILT Blockchain is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

// If you feel like getting in touch with us, you can do so at info@botlabs.org

/// Deprecated types used in version 1.
pub(crate) mod v1 {
	use codec::{Decode, Encode};

	use crate::*;

	#[derive(Clone, Debug, Encode, Decode, PartialEq)]
	pub struct DelegationRoot<T: Config> {
		pub(crate) ctype_hash: CtypeHashOf<T>,
		pub(crate) owner: DelegatorIdOf<T>,
		pub(crate) revoked: bool,
	}

	impl<T: Config> DelegationRoot<T> {
		#[cfg(test)]
		pub(crate) fn new(ctype_hash: CtypeHashOf<T>, owner: DelegatorIdOf<T>) -> Self {
			DelegationRoot {
				ctype_hash,
				owner,
				revoked: false,
			}
		}
	}

	#[derive(Clone, Debug, Encode, Decode, PartialEq)]
	pub struct DelegationNode<T: Config> {
		pub(crate) root_id: DelegationNodeIdOf<T>,
		pub(crate) parent: Option<DelegationNodeIdOf<T>>,
		pub(crate) owner: DelegatorIdOf<T>,
		pub(crate) permissions: Permissions,
		pub(crate) revoked: bool,
	}

	impl<T: Config> DelegationNode<T> {
		#[cfg(test)]
		pub(crate) fn new_root_child(
			root_id: DelegationNodeIdOf<T>,
			owner: DelegatorIdOf<T>,
			permissions: Permissions,
		) -> Self {
			DelegationNode {
				root_id,
				owner,
				permissions,
				revoked: false,
				parent: None,
			}
		}

		#[cfg(test)]
		pub(crate) fn new_node_child(
			root_id: DelegationNodeIdOf<T>,
			parent: DelegationNodeIdOf<T>,
			owner: DelegatorIdOf<T>,
			permissions: Permissions,
		) -> Self {
			DelegationNode {
				root_id,
				parent: Some(parent),
				owner,
				permissions,
				revoked: false,
			}
		}
	}

	pub(crate) mod storage {
		use frame_support::{decl_module, decl_storage};
		use sp_std::prelude::*;

		use super::*;

		decl_module! {
			pub struct OldPallet<T: Config> for enum Call where origin: <T as frame_system::Config>::Origin {}
		}

		decl_storage! {
			trait Store for OldPallet<T: Config> as Delegation {
				pub(crate) Roots get(fn roots): map hasher(blake2_128_concat) DelegationNodeIdOf<T> => Option<DelegationRoot<T>>;
				pub(crate) Delegations get(fn delegations): map hasher(blake2_128_concat) DelegationNodeIdOf<T> => Option<super::DelegationNode<T>>;
				pub(crate) Children get(fn children): map hasher(blake2_128_concat) DelegationNodeIdOf<T> => Option<Vec<DelegationNodeIdOf<T>>>;
			}
		}
	}
}
