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

use core::marker::PhantomData;
use frame_support::traits::Get;

pub struct RemoveKiltLaunch<R>(PhantomData<R>);
impl<R: frame_system::Config> frame_support::traits::OnRuntimeUpgrade for RemoveKiltLaunch<R> {
	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		let prefix: [u8; 16] = sp_io::hashing::twox_128(b"KiltLaunch");

		let items = match frame_support::storage::unhashed::kill_prefix(&prefix, Some(6)) {
			sp_io::KillStorageResult::AllRemoved(n) => {
				log::info!("🚀 Successfully removed all {:?} storage items of the launch pallet", n);
				n
			}
			sp_io::KillStorageResult::SomeRemaining(n) => {
				log::warn!(
					"🚀  Failed to remove all storage items of the launch pallet, {:?} are remaining",
					n
				);
				n
			}
		};
		<R as frame_system::Config>::DbWeight::get().writes(items.into())
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		let prefix: [u8; 16] = sp_io::hashing::twox_128(b"KiltLaunch");

		assert!(
			sp_io::storage::next_key(&prefix).map_or(false, |next_key| next_key.starts_with(&prefix)),
			"🚀 Pre check: Launch pallet storage does not exist!"
		);
		Ok(())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		let prefix: [u8; 16] = sp_io::hashing::twox_128(b"KiltLaunch");
		assert!(
			sp_io::storage::next_key(&prefix,).map_or(true, |next_key| !next_key.starts_with(&prefix)),
			"🚀 Post check: Launch pallet storage still exists!"
		);
		Ok(())
	}
}
