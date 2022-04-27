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
use hex_literal::hex;

pub struct RemoveKiltLaunch<R>(PhantomData<R>);
impl<R: frame_system::Config> frame_support::traits::OnRuntimeUpgrade for RemoveKiltLaunch<R> {
	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		let items = match frame_support::storage::unhashed::kill_prefix(&hex!("37be294ab4b5aa76f1df3f80e7c180ef"), None)
		{
			sp_io::KillStorageResult::AllRemoved(n) => {
				log::info!("🚀 Successfully removed all {} storage items of the launch pallet", n);
				n
			}
			sp_io::KillStorageResult::SomeRemaining(n) => {
				log::warn!(
					"🚀  Failed to remove all storage items of the launch pallet, {} are remaining",
					n
				);
				n
			}
		};
		<R as frame_system::Config>::DbWeight::get().writes(items.into())
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		// FIXME: Why does this fail?
		log::info!(
			"🚀 Pre check: Launch pallet storage exists {}?",
			frame_support::storage::migration::have_storage_value(
				&hex!("37be294ab4b5aa76f1df3f80e7c180ef"),
				// b"KiltLaunch"
				// b"TransferAccount",
				&hex!("73c7528dff85a7339c3d647527b5affb"),
				&[]
			)
		);

		assert!(frame_support::storage::migration::have_storage_value(
			&hex!("37be294ab4b5aa76f1df3f80e7c180ef"),
			// b"KiltLaunch",
			// b"TransferAccount",
			&hex!("73c7528dff85a7339c3d647527b5affb"),
			&[]
		));
		Ok(())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		match frame_support::storage::unhashed::kill_prefix(&hex!("37be294ab4b5aa76f1df3f80e7c180ef"), Some(1)) {
			sp_io::KillStorageResult::AllRemoved(0) => {
				log::info!("🚀 Post check: Launch pallet storage successfully removed");
				Ok(())
			}
			_ => Err("🚀 Post check: Launch pallet storage still exists!"),
		}
	}
}
