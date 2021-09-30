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
use frame_support::dispatch::Weight;
use kilt_traits::VersionMigratorTrait;
use sp_runtime::{
	codec::{Decode, Encode},
	traits::Zero,
};

use crate::*;

mod v2;
mod v3;
mod v4;
mod v5;

// A value placed in storage that represents the current version of the Staking
// storage. This value is used by the `on_runtime_upgrade` logic to determine
// whether we run storage migration logic. This should match directly with the
// semantic versions of the Rust crate.
#[derive(Copy, Clone, Encode, Eq, Decode, Debug, Ord, PartialEq, PartialOrd)]
pub enum StakingStorageVersion {
	V1_0_0,
	V2_0_0, // New Reward calculation, MaxCollatorCandidateStake
	V3_0_0, // Update InflationConfig
	V4,     // Sort TopCandidates and parachain-stakings by amount
	V5,     // Remove SelectedCandidates, Count Candidates
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
		Self::V5
	}
}

impl<T: Config> VersionMigratorTrait<T> for StakingStorageVersion {
	// It runs the right pre_migrate logic depending on the current storage version.
	#[cfg(feature = "try-runtime")]
	fn pre_migrate(&self) -> Result<(), &'static str> {
		match *self {
			Self::V1_0_0 => v2::pre_migrate::<T>(),
			Self::V2_0_0 => v3::pre_migrate::<T>(),
			Self::V3_0_0 => v4::pre_migrate::<T>(),
			Self::V4 => v5::pre_migrate::<T>(),
			Self::V5 => Ok(()),
		}
	}

	// It runs the right migration logic depending on the current storage version.
	fn migrate(&self) -> Weight {
		match *self {
			Self::V1_0_0 => v2::migrate::<T>(),
			Self::V2_0_0 => v3::migrate::<T>(),
			Self::V3_0_0 => v4::migrate::<T>(),
			Self::V4 => v5::migrate::<T>(),
			Self::V5 => Weight::zero(),
		}
	}

	fn next_version(&self) -> Option<Self> {
		match self {
			StakingStorageVersion::V1_0_0 => Some(StakingStorageVersion::V2_0_0),
			StakingStorageVersion::V2_0_0 => Some(StakingStorageVersion::V3_0_0),
			// Migration happens naturally, no need to point to the latest version
			StakingStorageVersion::V3_0_0 => Some(StakingStorageVersion::V4),
			StakingStorageVersion::V4 => Some(StakingStorageVersion::V5),
			StakingStorageVersion::V5 => None,
		}
	}

	// It runs the right post_migrate logic depending on the current storage
	// version.
	#[cfg(feature = "try-runtime")]
	fn post_migrate(&self) -> Result<(), &'static str> {
		match *self {
			Self::V1_0_0 => v2::post_migrate::<T>(),
			Self::V2_0_0 => v3::post_migrate::<T>(),
			Self::V3_0_0 => v4::post_migrate::<T>(),
			Self::V4 => v5::post_migrate::<T>(),
			Self::V5 => Ok(()),
		}
	}
}
