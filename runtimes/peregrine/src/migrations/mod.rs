use frame_support::traits::OnRuntimeUpgrade;
use sp_std::marker::PhantomData;

pub struct LookupReverseIndexMigration<T: pallet_did_lookup::Config>(PhantomData<T>);

impl OnRuntimeUpgrade for LookupReverseIndexMigration<Runtime> {

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		assert_eq!(DidLookup::<Runtime>::)
		assert_eq!(DidLookup::<Runtime>::connected_accounts().iter().count(), 0);
	}

    fn on_runtime_upgrade() -> frame_support::weights::Weight {
		0
	}

	#[cfg(feature = "try-runtime")]
    fn post_upgrade() -> Result<(), &'static str> {
		Ok(())
	}
}
