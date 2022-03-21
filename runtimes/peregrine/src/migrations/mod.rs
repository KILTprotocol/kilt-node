use crate::{DidLookup, Runtime, Weight};
use frame_support::traits::{OnRuntimeUpgrade, StorageVersion as NewStorageVersion};
use sp_std::marker::PhantomData;

#[cfg(feature = "try-runtime")]
use frame_support::traits::GetStorageVersion;

pub struct LookupReverseIndexMigration<T: pallet_did_lookup::Config>(PhantomData<T>);

impl OnRuntimeUpgrade for LookupReverseIndexMigration<Runtime> {

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		assert_ne!(DidLookup::on_chain_storage_version(), DidLookup::current_storage_version());
		assert_eq!(DidLookup::<Runtime>::connected_accounts().iter().count(), 0);
	}

    fn on_runtime_upgrade() -> frame_support::weights::Weight {
		// Account for the new storage version written below.
		let mut total_weight: Weight = Runtime::DbWeight::get().write(1);

		pallet_did_lookup::ConnectedDids::<Runtime>::iter().for_each(|(account, record)| {
			pallet_did_lookup::ConnectedAccounts::<Runtime>::insert(record.did, account, ());
			// One read for the `ConnectedDids` entry, one write for the new `ConnectedAccounts` entry.
			total_weight = total_weight.saturating_add(Runtime::DbWeight::get().reads_writes(1, 1))
		});

		NewStorageVersion::new(DidLookup::current_storage_version()).put::<DidLookup>();

		total_weight
	}

	#[cfg(feature = "try-runtime")]
    fn post_upgrade() -> Result<(), &'static str> {
		assert_eq!(DidLookup::on_chain_storage_version(), DidLookup::current_storage_version());

		// Verify DID -> Account integrity.
		pallet_did_lookup::ConnectedDids::<Runtime>::iter().for_each(|(account, record)| {
			assert!(pallet_did_lookup::ConnectedAccounts::<Runtime>::contains_key(record.did, account));
		});
		// Verify Account -> DID integrity.
		pallet_did_lookup::ConnectedAccounts::<Runtime>::iter().for_each(|(did, account, _)| {
			assert!(pallet_did_lookup::ConnectedDids::<Runtime>::contains_key(did, account));
		});
	}
}
