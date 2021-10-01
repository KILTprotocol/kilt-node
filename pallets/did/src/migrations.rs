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

mod setup;
mod v1;
mod v2;

/// Storage version of the DID pallet.
#[derive(Copy, Clone, Encode, Eq, Decode, Ord, PartialEq, PartialOrd)]
pub enum DidStorageVersion {
	None,
	V1,
	V2,
	V3,
}

impl DidStorageVersion {
	fn latest() -> Self {
		Self::V3
	}
}

impl Default for DidStorageVersion {
	fn default() -> Self {
		Self::None
	}
}

impl<T: Config> VersionMigratorTrait<T> for DidStorageVersion {
	// It runs the right pre_migrate logic depending on the current storage version.
	#[cfg(feature = "try-runtime")]
	fn pre_migrate(&self) -> Result<(), &'static str> {
		match *self {
			Self::None => setup::pre_migrate::<T>(),
			Self::V1 => v1::pre_migrate::<T>(),
			Self::V2 => v2::pre_migrate::<T>(),
			Self::V3 => Ok(()),
		}
	}

	// It runs the right migration logic depending on the current storage version.
	fn migrate(&self) -> Weight {
		match *self {
			Self::None => setup::migrate::<T>(),
			Self::V1 => v1::migrate::<T>(),
			Self::V2 => v2::migrate::<T>(),
			Self::V3 => Weight::zero(),
		}
	}

	fn next_version(&self) -> Option<Self> {
		match self {
			Self::V1 => Some(Self::V2),
			Self::V2 => Some(Self::V3),
			Self::V3 | Self::None => None,
		}
	}

	// It runs the right post_migrate logic depending on the current storage
	// version.
	#[cfg(feature = "try-runtime")]
	fn post_migrate(&self) -> Result<(), &'static str> {
		match *self {
			Self::None => setup::post_migrate::<T>(),
			Self::V1 => v1::post_migrate::<T>(),
			Self::V2 => v2::post_migrate::<T>(),
			Self::V3 => Ok(()),
		}
	}
}
