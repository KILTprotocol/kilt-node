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

use crate::*;
use frame_support::{
	storage::types::StorageMap,
	traits::{GetPalletVersion, PalletVersion},
	Identity,
};
use sp_runtime::traits::{CheckEqual, MaybeDisplay, MaybeSerializeDeserialize, Member, SimpleBitOps};
use sp_std::fmt::Debug;

pub const PALLET_PREFIX: &str = "Delegation";
pub const STORAGE_PREFIX: &str = "Delegations";

#[derive(Debug, Encode, Decode, PartialEq)]
pub struct DelegationNodeNew<DelegationNodeId, AccountId> {
	pub root_id: DelegationNodeId,
	pub parent: Option<DelegationNodeId>,
	pub owner: AccountId,
	pub permissions: Permissions,
	pub revoked: bool,
}

struct __Delegations;
impl frame_support::traits::StorageInstance for __Delegations {
	fn pallet_prefix() -> &'static str {
		PALLET_PREFIX
	}
	const STORAGE_PREFIX: &'static str = STORAGE_PREFIX;
}
pub trait V23ToV24 {
	type Module: GetPalletVersion;

	type DelegationNodeId: Parameter
		+ Member
		+ Codec
		+ MaybeDisplay
		+ SimpleBitOps
		+ Default
		+ Copy
		+ CheckEqual
		+ sp_std::hash::Hash
		+ AsRef<[u8]>
		+ AsMut<[u8]>;

	/// The user account identifier type for the runtime.
	type AccountId: Parameter + Member + MaybeSerializeDeserialize + Debug + MaybeDisplay + Ord + Default;
}

#[allow(type_alias_bounds)]
type Delegations<T: V23ToV24> = StorageMap<
	__Delegations,
	Identity,
	T::DelegationNodeId,
	Option<DelegationNodeNew<T::DelegationNodeId, T::AccountId>>,
>;

pub fn apply<T: V23ToV24>() -> Weight {
	let maybe_storage_version = <T::Module as GetPalletVersion>::storage_version();
	log::info!(
		"Running migration for delegation with storage version {:?}",
		maybe_storage_version
	);
	match maybe_storage_version {
		Some(storage_version) if storage_version < PalletVersion::new(0, 24, 0) => {
			migrate_to_struct::<T>();
			Weight::max_value()
		}
		// pallet versioning is introduced in 3.0.0, thus missing when upgrading from 2.0.0
		None => {
			migrate_to_struct::<T>();
			Weight::max_value()
		}
		_ => {
			log::warn!(
				"Attempted to apply delegation to 0.24.0 but failed because storage version is {:?}",
				maybe_storage_version
			);
			0
		}
	}
}

/// Migrate from the old legacy voting bond (fixed) to the new one (per-vote
/// dynamic).
fn migrate_to_struct<T: V23ToV24>() {
	<Delegations<T>>::translate::<
		Option<(
			T::DelegationNodeId,
			Option<T::DelegationNodeId>,
			T::AccountId,
			Permissions,
			bool,
		)>,
		_,
	>(|_delegation_id, option| {
		option.map(|(root_id, parent, owner, permissions, revoked)| {
			Some(DelegationNodeNew {
				root_id,
				parent,
				owner,
				permissions,
				revoked,
			})
		})
	});

	log::info!("migrated {} delegations.", <Delegations<T>>::iter().count(),);
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::tests::*;
	use frame_support::{
		storage::migration::{get_storage_value, put_storage_value},
		traits::{GetPalletVersion, PalletVersion},
	};
	use sp_core::{ed25519, Pair};
	use sp_runtime::{traits::IdentifyAccount, MultiSigner};

	#[test]
	fn migration_to_v24_should_work() {
		// setup migration data
		type DelegationOld = (
			<Test as Config>::DelegationNodeId,
			Option<<Test as Config>::DelegationNodeId>,
			<Test as frame_system::Config>::AccountId,
			Permissions,
			bool,
		);
		// set storage version
		struct ModuleVersion;
		impl GetPalletVersion for ModuleVersion {
			fn current_version() -> PalletVersion {
				PalletVersion {
					major: 0,
					minor: 23,
					patch: 0,
				}
			}
			fn storage_version() -> Option<PalletVersion> {
				Some(Self::current_version())
			}
		}
		struct DelegationStructRuntimeUpgrade;
		impl migration::V23ToV24 for DelegationStructRuntimeUpgrade {
			type AccountId = <Test as frame_system::Config>::AccountId;
			type DelegationNodeId = <Test as Config>::DelegationNodeId;
			// Note: Delegation::storage_version() resolves to `None` :(
			type Module = ModuleVersion;
		}

		new_test_ext().execute_with(|| {
			// setup data independent of migration
			let pair_delegate = ed25519::Pair::from_seed(&*b"Alice                           ");
			let acc_delegate = MultiSigner::from(pair_delegate.public()).into_account();
			let root_id: <Test as Config>::DelegationNodeId = <Test as frame_system::Config>::Hashing::hash(&[0]);
			let delegation_id: <Test as Config>::DelegationNodeId = <Test as frame_system::Config>::Hashing::hash(&[1]);
			let blake_hash = sp_core::blake2_256(&delegation_id.as_bytes());

			// store and check old DelegationNode type
			let delegation_old: DelegationOld = (root_id, None, acc_delegate.clone(), Permissions::DELEGATE, false);
			put_storage_value::<Option<DelegationOld>>(
				crate::migration::PALLET_PREFIX.as_bytes(),
				crate::migration::STORAGE_PREFIX.as_bytes(),
				&blake_hash,
				Some(delegation_old.clone()),
			);
			assert_eq!(
				get_storage_value::<Option<DelegationOld>>(
					crate::migration::PALLET_PREFIX.as_bytes(),
					crate::migration::STORAGE_PREFIX.as_bytes(),
					&blake_hash,
				),
				Some(Some(delegation_old))
			);

			// apply migration
			crate::migration::apply::<DelegationStructRuntimeUpgrade>();

			// setup and check new DelegationNode type
			let delegation_new = DelegationNode::<Test> {
				root_id,
				parent: None,
				owner: acc_delegate.clone(),
				permissions: Permissions::DELEGATE,
				revoked: false,
			};
			assert_eq!(
				get_storage_value::<Option<DelegationNode::<Test>>>(
					crate::migration::PALLET_PREFIX.as_bytes(),
					crate::migration::STORAGE_PREFIX.as_bytes(),
					&blake_hash
				),
				Some(Some(delegation_new))
			);
		});
	}
}
