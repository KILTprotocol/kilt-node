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
#[derive(Copy, Clone, Encode, Eq, Decode, Debug, Ord, PartialEq, PartialOrd, TypeInfo)]
pub enum StakingStorageVersion {
	V1_0_0,
	V2_0_0, // New Reward calculation, MaxCollatorCandidateStake
	V3_0_0, // Update InflationConfig
	V4,     // Sort TopCandidates and parachain-stakings by amount
	V5,     // Remove SelectedCandidates, Count Candidates
	V6,     // Fix delegator replacement bug
	V7,     // CountedStorageMap for CandidatePool
}

#[cfg(feature = "try-runtime")]
impl StakingStorageVersion {
	/// The latest storage version.
	#[allow(dead_code)]
	fn latest() -> Self {
		Self::V7
	}
}

// All nodes will default to this, which is not bad, as in case the "real"
// version is a later one (i.e. the node has been started with already the
// latest version), the migration will simply do nothing as there's nothing in
// the old storage entries to migrate from.
//
// It might get updated in the future when we know that no node is running this
// old version anymore.
impl Default for StakingStorageVersion {
	fn default() -> Self {
		Self::V6
	}
}

pub mod v8 {
	use super::*;
	use crate::{CandidatePool, Config, StorageVersion};

	use frame_support::{
		generate_storage_alias,
		pallet_prelude::Weight,
		storage::migration::{put_storage_value, remove_storage_prefix},
		traits::Get,
	};
	use log::info;
	use sp_runtime::traits::Zero;

	generate_storage_alias!(ParachainStaking, CandidateCount => Value<u32>);

	pub fn pre_migrate<T: Config>() -> Result<(), &'static str> {
		assert!(
			!CandidateCount::get().unwrap().is_zero(),
			"CandidateCount already migrated"
		);
		assert!(CandidatePool::<T>::count().is_zero(), "Candidate counter already set");
		assert!(StorageVersion::<T>::get() == StakingStorageVersion::V6);

		info!("parachain staking migration to v7 passes PRE migrate checks ✅",);
		Ok(())
	}

	pub fn migrate<T: Config>() -> Weight {
		let candidate_count = CandidatePool::<T>::iter().count() as u32;

		remove_storage_prefix(b"ParachainStaking", b"CandidateCount", &[]);
		CandidatePool::<T>::initialize_counter();
		StorageVersion::<T>::put(StakingStorageVersion::V7);

		info!("completed parachain staking migration to v7 ✅",);
		T::DbWeight::get().reads_writes(candidate_count.saturating_add(2).into(), 1)
	}

	pub fn post_migrate<T: Config>() -> Result<(), &'static str> {
		assert!(
			CandidateCount::get().is_none(),
			"CandidateCount should not exist anymore"
		);
		assert!(
			!frame_support::migration::have_storage_value(b"StakePallet", b"CandidateCount", &[]),
			"CandidateCount should not exist anymore"
		);
		assert!(
			!CandidatePool::<T>::count().is_zero(),
			"Candidate counter should have been set"
		);
		assert!(StorageVersion::<T>::get() == StakingStorageVersion::V7);

		// use current_storage_version

		info!("parachain staking migration to v7 passes POST migrate checks ✅",);
		Ok(())
	}
}
