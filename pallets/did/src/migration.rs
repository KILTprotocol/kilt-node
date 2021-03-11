use crate::*;
use frame_support::{
	dispatch::Weight,
	storage::types::StorageMap,
	traits::{GetPalletVersion, PalletVersion},
	Identity,
};
use sp_runtime::traits::{MaybeDisplay, MaybeSerializeDeserialize, Member};
use sp_std::fmt::Debug;

pub const DID_MODULE_PREFIX: &str = "Did";
pub const DID_STORAGE_PREFIX: &str = "DIDs";

#[derive(Debug, Encode, Decode, PartialEq)]
pub struct DidRecord<PublicSigningKey, PublicBoxKey> {
	sign_key: PublicSigningKey,
	box_key: PublicBoxKey,
	doc_ref: Option<Vec<u8>>,
}

struct __Dids;
impl frame_support::traits::StorageInstance for __Dids {
	fn pallet_prefix() -> &'static str {
		DID_MODULE_PREFIX
	}
	const STORAGE_PREFIX: &'static str = DID_STORAGE_PREFIX;
}
pub trait V23ToV24 {
	type Module: GetPalletVersion;

	type PublicSigningKey: Parameter + Member + Codec;
	type PublicBoxKey: Parameter + Member + Codec;

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
type Dids<T: V23ToV24> = StorageMap<
	__Dids,
	Identity,
	T::AccountId,
	Option<DidRecord<T::PublicSigningKey, T::PublicBoxKey>>,
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

/// Migrate from the old legacy voting bond (fixed) to the new one (per-vote dynamic).
fn migrate_to_struct<T: V23ToV24>() {
	let mut counter = 0;
	<Dids<T>>::translate::<Option<(T::PublicSigningKey, T::PublicBoxKey, Option<Vec<u8>>)>, _>(
		|_who, option| {
			counter += 1;
			option.map(|(sign_key, box_key, doc_ref)| {
				Some(DidRecord {
					sign_key,
					box_key,
					doc_ref,
				})
			})
		},
	);

	log::info!("migrated {} did records.", counter,);
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::tests::*;

	#[test]
	fn migration_to_v24_should_work() {
		new_test_ext().execute_with(|| {
			use frame_support::storage::migration::{get_storage_value, put_storage_value};

			type DidRecordOld = (
				<Test as Config>::PublicSigningKey,
				<Test as Config>::PublicBoxKey,
				Option<Vec<u8>>,
			);

			type DidRecordNew =
				DidRecord<<Test as Config>::PublicSigningKey, <Test as Config>::PublicBoxKey>;

			let did_old: DidRecordOld = (
				Default::default(),
				Default::default(),
				Some(
					"lkahsdflöasdhflkjahsdfjkasdjölkjSADÖKkash"
						.as_bytes()
						.to_vec(),
				),
			);
			let blake_hash = sp_core::blake2_256(&[1]);

			put_storage_value::<Option<DidRecordOld>>(
				DID_MODULE_PREFIX.as_bytes(),
				DID_STORAGE_PREFIX.as_bytes(),
				&blake_hash,
				Some(did_old.clone()),
			);
			assert_eq!(
				get_storage_value::<Option<DidRecordOld>>(
					DID_MODULE_PREFIX.as_bytes(),
					DID_STORAGE_PREFIX.as_bytes(),
					&blake_hash,
				),
				Some(Some(did_old))
			);

			struct DidStructRuntimeUpgrade;
			impl V23ToV24 for DidStructRuntimeUpgrade {
				type Module = ModuleVersion;

				type PublicSigningKey = <Test as Config>::PublicSigningKey;
				type PublicBoxKey = <Test as Config>::PublicBoxKey;
				type AccountId = <Test as frame_system::Config>::AccountId;
			}

			apply::<DidStructRuntimeUpgrade>();

			let new_did = DidRecordNew {
				sign_key: Default::default(),
				box_key: Default::default(),
				doc_ref: Some(
					"lkahsdflöasdhflkjahsdfjkasdjölkjSADÖKkash"
						.as_bytes()
						.to_vec(),
				),
			};

			assert_eq!(
				get_storage_value::<Option<DidRecordNew>>(
					DID_MODULE_PREFIX.as_bytes(),
					DID_STORAGE_PREFIX.as_bytes(),
					&blake_hash,
				),
				Some(Some(new_did))
			);
		});
	}
}
