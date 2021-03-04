use crate::*;
use frame_support::{
	storage::types::StorageMap,
	traits::{GetPalletVersion, PalletVersion},
	Identity,
};
use sp_runtime::traits::{
	CheckEqual, MaybeDisplay, MaybeSerializeDeserialize, Member, SimpleBitOps,
};
use sp_std::fmt::Debug;

pub const PALLET_PREFIX: &str = "Delegation";
pub const STORAGE_PREFIX: &str = "Delegations";

#[derive(Debug, Encode, Decode, PartialEq)]
pub struct DelegationNode<DelegationNodeId, AccountId> {
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
	type AccountId: Parameter
		+ Member
		+ MaybeSerializeDeserialize
		+ Debug
		+ MaybeDisplay
		+ Ord
		+ Default;
}

#[allow(type_alias_bounds)]
type Delegations<T: V23ToV24> = StorageMap<
	__Delegations,
	Identity,
	T::DelegationNodeId,
	Option<DelegationNode<T::DelegationNodeId, T::AccountId>>,
>;

pub fn apply<T: V23ToV24>() -> Weight {
	let maybe_storage_version = <T::Module as GetPalletVersion>::storage_version();
	frame_support::debug::info!(
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
			frame_support::debug::warn!(
				"Attempted to apply delegation to 0.24.0 but failed because storage version is {:?}",
				maybe_storage_version
			);
			0
		}
	}
}

/// Migrate from the old legacy voting bond (fixed) to the new one (per-vote dynamic).
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
			Some(DelegationNode {
				root_id,
				parent,
				owner,
				permissions,
				revoked,
			})
		})
	});

	frame_support::debug::info!("migrated {} delegations.", <Delegations<T>>::iter().count(),);
}
