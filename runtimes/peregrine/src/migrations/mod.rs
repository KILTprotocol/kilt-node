use crate::{DidLookup, Runtime};
use frame_support::traits::OnRuntimeUpgrade;
use sp_std::marker::PhantomData;

// #[cfg(feature = "try-runtime")]
use frame_support::traits::GetStorageVersion;

pub struct LookupReverseIndexMigration<T: pallet_did_lookup::Config>(PhantomData<T>);

impl OnRuntimeUpgrade for LookupReverseIndexMigration<Runtime> {

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		assert_eq!(DidLookup::on_chain_storage_version().saturating_add(1), DidLookup::on_chain_storage_version());
		assert_eq!(DidLookup::<Runtime>::connected_accounts().iter().count(), 0);
	}

    fn on_runtime_upgrade() -> frame_support::weights::Weight {
		// TODO: Populate the ConnectedAccounts map and set the new storage version
		0u64
	}

	#[cfg(feature = "try-runtime")]
    fn post_upgrade() -> Result<(), &'static str> {
		// TODO: Verify that the ConnectedAccounts map contains the same number of elements as the map and that the on-chain and runtime storage versions match.
		Ok(())
	}
}
