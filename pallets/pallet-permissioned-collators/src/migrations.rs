use frame_support::{storage_alias, traits::OnRuntimeUpgrade, weights::Weight};
use parachain_staking::{Config as StakingConfig, Round, TopCandidates};
use sp_runtime::BoundedVec;
use sp_std::vec::Vec;

#[cfg(feature = "try-runtime")]
use alloc::vec::Vec;

use crate::{types::RoundInfo, Collators, Config, Round as PermissionedRound};

mod v0 {
	use super::*;

	#[storage_alias]
	pub type Value<T: Config> = StorageValue<crate::Pallet<T>, u32>;
}

pub struct InnerMigrateV0ToV1<T: Config + StakingConfig>(core::marker::PhantomData<T>);

impl<T: crate::Config + StakingConfig> OnRuntimeUpgrade for InnerMigrateV0ToV1<T> {
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<Vec<u8>, sp_runtime::TryRuntimeError> {
		Ok(vec![])
	}

	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		let top_candidates = TopCandidates::<T>::get()
			.into_bounded_vec()
			.into_iter()
			.map(|stake| stake.owner)
			.collect::<Vec<T::AccountId>>();

		let permissioned_collators =
			BoundedVec::<<T as frame_system::Config>::AccountId, T::MaxCollators>::truncate_from(top_candidates);

		Collators::<T>::put(permissioned_collators);

		let round = Round::<T>::get();

		let new_round = RoundInfo {
			current: round.current,
			first: round.first,
			length: round.length,
		};

		PermissionedRound::<T>::put(new_round);

		Weight::from_parts(10_000, 0)
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(state: Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
		Ok(())
	}
}

pub type MigrateV0ToV1<T> = frame_support::migrations::VersionedMigration<
	0,
	1,
	InnerMigrateV0ToV1<T>,
	crate::pallet::Pallet<T>,
	<T as frame_system::Config>::DbWeight,
>;
