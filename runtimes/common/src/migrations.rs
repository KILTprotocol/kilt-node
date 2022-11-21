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

use sp_std::marker::PhantomData;

use frame_support::{
	traits::{Get, OnRuntimeUpgrade},
	StorageHasher, Twox128,
};

pub struct RemoveRelocationPallets<R>(PhantomData<R>);

impl<R: frame_system::Config> OnRuntimeUpgrade for RemoveRelocationPallets<R> {
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		log::info!("Pre check RemoveRelocationPallets.");
		let has_migration_storage = frame_support::storage::migration::have_storage_value(
			b"RelayMigration",
			b"RelayNumberStrictlyIncreases",
			b"",
		);
		let has_filter_storage = frame_support::storage::migration::have_storage_value(b"DynFilter", b"Filter", b"");

		match (has_migration_storage, has_filter_storage) {
			(false, false) => Err("Pallets not present"),
			(true, false) => Err("DynFilter not present"),
			(false, true) => Err("RelayMigration not present"),
			_ => Ok(()),
		}
	}

	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		let entries: u32 = 2;
		if frame_support::storage::unhashed::clear_prefix(&Twox128::hash(b"RelayMigration"), Some(entries), None)
			.maybe_cursor
			.is_some()
		{
			log::warn!("Pallet RelayMigration not removed entirely")
		}
		if frame_support::storage::unhashed::clear_prefix(&Twox128::hash(b"DynFilter"), Some(entries), None)
			.maybe_cursor
			.is_some()
		{
			log::warn!("Pallet DynFilter not removed entirely")
		}

		<R as frame_system::Config>::DbWeight::get().writes((entries * 2).into())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		log::info!("Post check RemoveRelocationPallets.");
		let has_migration_storage = frame_support::storage::migration::have_storage_value(
			b"RelayMigration",
			b"RelayNumberStrictlyIncreases",
			b"",
		);
		let has_filter_storage = frame_support::storage::migration::have_storage_value(b"DynFilter", b"Filter", b"");

		match (has_migration_storage, has_filter_storage) {
			(false, false) => Ok(()),
			(true, false) => Err("RelayMigration still present"),
			(false, true) => Err("DynFilter still present"),
			(true, true) => Err("Pallets still present"),
		}
	}
}
