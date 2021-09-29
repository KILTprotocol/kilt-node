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

use codec::{Decode, Encode};
use kilt_traits::VersionMigratorTrait;
use sp_runtime::traits::Zero;

use crate::*;

mod v1;
mod v2;

/// Storage version of the DID pallet.
#[derive(Copy, Clone, Encode, Eq, Decode, Ord, PartialEq, PartialOrd)]
pub enum DidStorageVersion {
	V1,
	V2,
	V3,
}

#[cfg(feature = "try-runtime")]
impl DidStorageVersion {
	/// The latest storage version.
	fn latest() -> Self {
		Self::V3
	}
}

// All nodes will default to this, which is not bad, as in case the "real"
// version is a later one (i.e. the node has been started with already the
// latest version), the migration will simply do nothing as there's nothing in
// the old storage entries to migrate from.
//
// It might get updated in the future when we know that no node is running this
// old version anymore.
impl Default for DidStorageVersion {
	fn default() -> Self {
		Self::V1
	}
}

impl<T: Config> VersionMigratorTrait<T> for DidStorageVersion {
	// It runs the right pre_migrate logic depending on the current storage version.
	#[cfg(feature = "try-runtime")]
	fn pre_migrate(&self) -> Result<(), String> {
		match *self {
			Self::V1 => v1::pre_migrate::<T>(),
			Self::V2 => v2::pre_migrate::<T>(),
			Self::V3 => Ok(()),
		}
	}

	// It runs the right migration logic depending on the current storage version.
	fn migrate(&self) -> Weight {
		match *self {
			Self::V1 => v1::migrate::<T>(),
			Self::V2 => v2::migrate::<T>(),
			Self::V3 => Weight::zero(),
		}
	}

	// It runs the right post_migrate logic depending on the current storage
	// version.
	#[cfg(feature = "try-runtime")]
	fn post_migrate(&self) -> Result<(), String> {
		match *self {
			Self::V1 => v1::post_migrate::<T>(),
			Self::V2 => v2::post_migrate::<T>(),
			Self::V3 => Ok(()),
		}
	}

	fn next_version(&self) -> Option<Self> {
		match self {
			DidStorageVersion::V1 => Some(DidStorageVersion::V2),
			DidStorageVersion::V2 => Some(DidStorageVersion::V3),
			DidStorageVersion::V3 => None,
		}
	}
}

