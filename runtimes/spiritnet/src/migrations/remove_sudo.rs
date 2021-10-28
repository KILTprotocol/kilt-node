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

use crate::Runtime;
use frame_support::traits::OnRuntimeUpgrade;
use hex_literal::hex;

pub struct RemoveSudo;

impl OnRuntimeUpgrade for RemoveSudo {
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		log::info!("Pre check Sudo-Removal.");
		Ok(())
	}

	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		// Magic bytes are the sudo pallet prefix
		let res = frame_support::storage::unhashed::kill_prefix(&hex!["5c0d1176a568c1f92944340dbfed9e9c"], Some(1));

		// there should only be one key removed.
		debug_assert!(matches!(res, sp_io::KillStorageResult::AllRemoved(1)));

		<Runtime as frame_system::Config>::DbWeight::get().writes(1)
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		log::info!("Post check Sudo-Removal.");
		Ok(())
	}
}
