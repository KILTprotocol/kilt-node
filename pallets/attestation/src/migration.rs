use crate::*;
use frame_support::{
	dispatch::{Codec, Weight},
	storage::types::StorageMap,
	traits::{GetPalletVersion, PalletVersion},
	Identity, Parameter,
};
use sp_runtime::traits::{
	CheckEqual, MaybeDisplay, MaybeMallocSizeOf, MaybeSerializeDeserialize, Member, SimpleBitOps,
};
use sp_std::fmt::Debug;

pub const ATTESTATION_PALLET_PREFIX: &str = "Attestation";
pub const ATTESTATION_STORAGE_PREFIX: &str = "Attestations";

#[derive(Debug, Encode, Decode, PartialEq)]
pub struct AttestationStruct<Hash, AccountId, DelegationNodeId> {
	// hash of the CTYPE used for this attestation
	ctype_hash: Hash,
	// the account which executed the attestation
	attester: AccountId,
	// id of the delegation node (if existent)
	delegation_id: Option<DelegationNodeId>,
	// revocation status
	revoked: bool,
}

struct __Attestations;
impl frame_support::traits::StorageInstance for __Attestations {
	fn pallet_prefix() -> &'static str {
		ATTESTATION_PALLET_PREFIX
	}
	const STORAGE_PREFIX: &'static str = ATTESTATION_STORAGE_PREFIX;
}
pub trait V23ToV24 {
	type Module: GetPalletVersion;

	type Hash: Parameter
		+ Member
		+ MaybeSerializeDeserialize
		+ Debug
		+ MaybeDisplay
		+ SimpleBitOps
		+ Ord
		+ Default
		+ Copy
		+ CheckEqual
		+ sp_std::hash::Hash
		+ AsRef<[u8]>
		+ AsMut<[u8]>
		+ MaybeMallocSizeOf;

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
type Attestations<T: V23ToV24> = StorageMap<
	__Attestations,
	Identity,
	T::DelegationNodeId,
	Option<AttestationStruct<T::Hash, T::AccountId, T::DelegationNodeId>>,
>;

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

pub fn apply<T: V23ToV24>() -> Weight {
	let maybe_storage_version = <T::Module as GetPalletVersion>::storage_version();
	log::info!(
		"Running migration for attestation with storage version {:?}",
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
				"Attempted to apply attestion to 0.24.0 but failed because storage version is {:?}",
				maybe_storage_version
			);
			0
		}
	}
}

/// Migrate from the old legacy voting bond (fixed) to the new one (per-vote dynamic).
fn migrate_to_struct<T: V23ToV24>() {
	let mut counter = 0;
	<Attestations<T>>::translate::<
		Option<(T::Hash, T::AccountId, Option<T::DelegationNodeId>, bool)>,
		_,
	>(|_who, option| {
		counter += 1;
		option.map(|(ctype_hash, attester, delegation_id, revoked)| {
			Some(AttestationStruct {
				ctype_hash,
				attester,
				delegation_id,
				revoked,
			})
		})
	});

	log::info!("migrated {} attestation records.", counter,);
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::tests::*;
	use frame_support::storage::migration::{get_storage_value, put_storage_value};
	use sp_core::{ed25519, Hasher, Pair};
	use sp_runtime::{traits::IdentifyAccount, MultiSigner};

	#[test]
	fn migration_to_v24_should_work() {
		new_test_ext().execute_with(|| {
			type AttestationOld = (
				<Test as frame_system::Config>::Hash,
				<Test as frame_system::Config>::AccountId,
				Option<<Test as delegation::Config>::DelegationNodeId>,
				bool,
			);

			type AttestationNew = AttestationStruct<
				<Test as frame_system::Config>::Hash,
				<Test as frame_system::Config>::AccountId,
				<Test as delegation::Config>::DelegationNodeId,
			>;

			let blake_hash = sp_core::blake2_256(&[1]);
			let ctype_hash: <Test as frame_system::Config>::Hash =
				<Test as frame_system::Config>::Hashing::hash(b"ctype");
			let pair_attester = ed25519::Pair::from_seed(&*b"Alice                           ");
			let attester: <Test as frame_system::Config>::AccountId =
				MultiSigner::from(pair_attester.public()).into_account();
			let delegation_id: Option<<Test as delegation::Config>::DelegationNodeId> = None;
			let revoked = false;

			let attestation_old: AttestationOld =
				(ctype_hash, attester.clone(), delegation_id, revoked);

			put_storage_value::<Option<AttestationOld>>(
				ATTESTATION_PALLET_PREFIX.as_bytes(),
				ATTESTATION_STORAGE_PREFIX.as_bytes(),
				&blake_hash,
				Some(attestation_old.clone()),
			);
			assert_eq!(
				get_storage_value::<Option<AttestationOld>>(
					ATTESTATION_PALLET_PREFIX.as_bytes(),
					ATTESTATION_STORAGE_PREFIX.as_bytes(),
					&blake_hash,
				),
				Some(Some(attestation_old))
			);

			struct AttestationstructRuntimeUpgrade;
			impl V23ToV24 for AttestationstructRuntimeUpgrade {
				type Module = ModuleVersion;
				type Hash = <Test as frame_system::Config>::Hash;
				type AccountId = <Test as frame_system::Config>::AccountId;
				type DelegationNodeId = <Test as delegation::Config>::DelegationNodeId;
			}

			apply::<AttestationstructRuntimeUpgrade>();

			let attestation_new = AttestationNew {
				ctype_hash,
				attester,
				delegation_id,
				revoked,
			};

			assert_eq!(
				get_storage_value::<Option<AttestationNew>>(
					ATTESTATION_PALLET_PREFIX.as_bytes(),
					ATTESTATION_STORAGE_PREFIX.as_bytes(),
					&blake_hash,
				),
				Some(Some(attestation_new))
			);
		});
	}
}
