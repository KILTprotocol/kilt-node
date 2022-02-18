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

use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

// FIXME: Remove when migrating to v8
// #[deprecated(note = "use the pallet's `current_storage_version()` instead")]
/// Storage version of the delegation pallet.
#[derive(Copy, Clone, Encode, Eq, Decode, Ord, PartialEq, PartialOrd, TypeInfo, MaxEncodedLen)]
pub enum DelegationStorageVersion {
	V1,
	V2,
}

// All nodes will default to this, which is not bad, as in case the "real"
// version is a later one (i.e. the node has been started with already the
// latest version), the migration will simply do nothing as there's nothing in
// the old storage entries to migrate from.
//
// It might get updated in the future when we know that no node is running this
// old version anymore.
impl Default for DelegationStorageVersion {
	fn default() -> Self {
		Self::V2
	}
}

pub mod v3 {
	use super::*;
	use crate::{Config, Pallet};

	#[cfg(feature = "try-runtime")]
	use frame_support::traits::GetStorageVersion;

	use frame_support::{
		generate_storage_alias,
		pallet_prelude::Weight,
		traits::{Get, OnRuntimeUpgrade, PalletInfoAccess},
	};
	use log::info;
	use sp_std::marker::PhantomData;

	// Get storage item into scope which are removed during this migration
	generate_storage_alias!(Delegation, StorageVersion => Value<DelegationStorageVersion>);

	pub struct DelegationMigrationV3<T: Config>(PhantomData<T>);

	impl<T: Config> OnRuntimeUpgrade for DelegationMigrationV3<T> {
		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<(), &'static str> {
			assert!(StorageVersion::get() == Some(DelegationStorageVersion::V2));

			// no migration needed
			assert!(
				Pallet::<T>::current_storage_version() == 3,
				"New StorageVersion should be set via pallet macro already"
			);

			info!("👥  Delegation pallet to v3 passes PRE migrate checks ✅",);
			Ok(())
		}

		fn on_runtime_upgrade() -> Weight {
			// remove deprecated storage versioning entry
			frame_support::migration::remove_storage_prefix(Pallet::<T>::name().as_bytes(), b"StorageVersion", &[]);

			info!("👥  completed Delegation pallet migration to v3 ✅",);
			T::DbWeight::get().reads_writes(0, 1)
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade() -> Result<(), &'static str> {
			// check StorageVersion
			assert!(
				!frame_support::migration::have_storage_value(Pallet::<T>::name().as_bytes(), b"StorageVersion", &[]),
				"Old StorageVersion should not exist anymore"
			);

			info!(
				"👥  Delegation pallet migration to {:?} passes POST migrate checks ✅",
				Pallet::<T>::current_storage_version()
			);
			Ok(())
		}
	}
}
