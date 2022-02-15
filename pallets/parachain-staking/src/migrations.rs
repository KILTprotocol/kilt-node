// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2022 BOTLabs GmbH

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
use scale_info::TypeInfo;
use sp_runtime::codec::{Decode, Encode};

// A value placed in storage that represents the current version of the Staking
// storage. This value is used by the `on_runtime_upgrade` logic to determine
// whether we run storage migration logic. This should match directly with the
// semantic versions of the Rust crate.

// FIXME: Remove when migrating to v8
// #[deprecated(note = "use the pallet's `current_storage_version()` instead")]
#[derive(Copy, Clone, Encode, Eq, Decode, Debug, Ord, PartialEq, PartialOrd, TypeInfo)]
pub enum StakingStorageVersion {
	V1_0_0,
	V2_0_0, // New Reward calculation, MaxCollatorCandidateStake
	V3_0_0, // Update InflationConfig
	V4,     // Sort TopCandidates and parachain-stakings by amount
	V5,     // Remove SelectedCandidates, Count Candidates
	V6,     // Fix delegator replacement bug
}

// Migration for ParachainStaking pallet to mutate CandidatePool to be
// a CountedStorageMap and switch to new Pallet StorageVersion paradigm.
pub mod v7 {
	use super::*;
	use crate::{CandidatePool, Config, Pallet};

	#[cfg(feature = "try-runtime")]
	use frame_support::traits::GetStorageVersion;
	use frame_support::{
		generate_storage_alias,
		traits::{Get, OnRuntimeUpgrade, PalletInfoAccess, StorageVersion as NewStorageVersion},
		weights::Weight,
	};
	use log::info;
	#[cfg(feature = "try-runtime")]
	use sp_runtime::traits::Zero;
	use sp_std::marker::PhantomData;

	// Get storage items into scope which are removed during this migration
	generate_storage_alias!(ParachainStaking, CandidateCount => Value<u32>);
	generate_storage_alias!(ParachainStaking, StorageVersion => Value<StakingStorageVersion>);

	pub struct ParachainStakingMigrationV7<T: Config>(PhantomData<T>);

	impl<T: Config> OnRuntimeUpgrade for ParachainStakingMigrationV7<T> {
		fn on_runtime_upgrade() -> Weight {
			let staking_pallet_name = Pallet::<T>::name().as_bytes();

			// migrate CandidateCount
			frame_support::migration::remove_storage_prefix(staking_pallet_name, b"CandidateCount", &[]);
			let candidate_count = CandidatePool::<T>::initialize_counter();

			// migrate StorageVersion to new paradigm
			frame_support::migration::remove_storage_prefix(staking_pallet_name, b"StorageVersion", &[]);
			NewStorageVersion::new(7).put::<Pallet<T>>();

			info!("💰 completed parachain staking migration to v7 ✅",);
			T::DbWeight::get().reads_writes(candidate_count.saturating_add(1).into(), 4)
		}

		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<(), &'static str> {
			assert!(
				!CandidateCount::get().unwrap().is_zero(),
				"CandidateCount already migrated"
			);
			assert!(CandidatePool::<T>::count().is_zero(), "Candidate counter already set");
			assert!(StorageVersion::get() == Some(StakingStorageVersion::V6));

			assert!(CandidatePool::<T>::initialize_counter() == CandidatePool::<T>::iter().count() as u32);

			info!("💰 parachain staking migration to v7 passes PRE migrate checks ✅",);
			Ok(())
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade() -> Result<(), &'static str> {
			let staking_pallet_name = Pallet::<T>::name().as_bytes();

			// check count
			assert!(
				!frame_support::migration::have_storage_value(staking_pallet_name, b"CandidateCount", &[]),
				"CandidateCount should not exist anymore"
			);
			assert!(
				!CandidatePool::<T>::count().is_zero(),
				"Candidate counter should have been set"
			);

			// check StorageVersion
			assert!(
				!frame_support::migration::have_storage_value(staking_pallet_name, b"StorageVersion", &[]),
				"Old StorageVersion should not exist anymore"
			);
			assert_eq!(
				Pallet::<T>::current_storage_version(),
				7,
				"StorageVersion should have migrated to new paradigm"
			);

			info!(
				"💰 parachain staking migration to {:?} passes POST migrate checks ✅",
				Pallet::<T>::current_storage_version()
			);
			Ok(())
		}
	}
}
