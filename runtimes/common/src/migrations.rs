// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2024 BOTLabs GmbH

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

use frame_support::{
	traits::{GetStorageVersion, OnRuntimeUpgrade, PalletInfoAccess, StorageVersion},
	weights::Weight,
};
use sp_core::Get;
use sp_std::{fmt::Debug, marker::PhantomData};
use sp_weights::RuntimeDbWeight;

const LOG_TARGET: &str = "migration::BumpStorageVersion";

/// There are some pallets without a storage version.
/// Based on the changes in the PR <https://github.com/paritytech/substrate/pull/13417>,
/// pallets without a storage version or with a wrong version throw an error
/// in the try state tests.
pub struct BumpStorageVersion<T, W>(PhantomData<(T, W)>)
where
	T: GetStorageVersion + PalletInfoAccess,
	T::CurrentStorageVersion: Debug + Into<StorageVersion>,
	StorageVersion: PartialOrd<T::CurrentStorageVersion>,
	W: Get<RuntimeDbWeight>;

impl<T, W> OnRuntimeUpgrade for BumpStorageVersion<T, W>
where
	T: GetStorageVersion + PalletInfoAccess,
	T::CurrentStorageVersion: Debug + Into<StorageVersion>,
	StorageVersion: PartialOrd<T::CurrentStorageVersion>,
	W: Get<RuntimeDbWeight>,
{
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<sp_std::vec::Vec<u8>, sp_runtime::TryRuntimeError> {
		let (on_chain_version, current_version) = (T::on_chain_storage_version(), T::current_storage_version());
		let pallet_name = T::name();
		if on_chain_version < current_version {
			log::trace!(target: LOG_TARGET, "Pallet {:?} to be migrated from version {:?} to version {:?}.", pallet_name, on_chain_version, current_version);
		} else {
			log::trace!(target: LOG_TARGET, "Pallet {:?} already on latest version {:?}. No migration will run.", pallet_name, current_version);
		}
		Ok([].into())
	}

	fn on_runtime_upgrade() -> Weight {
		log::info!(target: LOG_TARGET, "Initiating migration.");

		let (on_chain_version, current_version) = (T::on_chain_storage_version(), T::current_storage_version());
		let pallet_name = T::name();

		if on_chain_version < current_version {
			log::trace!(target: LOG_TARGET, "Pallet {:?} to be migrated from version {:?} to version {:?}.", pallet_name, on_chain_version, current_version);
			current_version.into().put::<T>();
			W::get().reads_writes(1, 1)
		} else {
			log::trace!(target: LOG_TARGET, "Pallet {:?} already on latest version {:?}. No migration will run.", pallet_name, current_version);
			W::get().reads(1)
		}
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_state: sp_std::vec::Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
		let (on_chain_version, current_version) = (T::on_chain_storage_version(), T::current_storage_version());

		if on_chain_version < current_version {
			Err(sp_runtime::TryRuntimeError::Other(
				"Pallet storage version was not updated to the latest version.",
			))
		} else {
			Ok(())
		}
	}
}
