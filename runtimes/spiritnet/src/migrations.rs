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
use frame_support::{traits::OnRuntimeUpgrade, StorageHasher, Twox128};

pub struct RemoveCrowdloanContributors;

impl OnRuntimeUpgrade for RemoveCrowdloanContributors {
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		use kilt_primitives::AccountId;

		log::info!("Pre check CrowdloanContributors-Removal.");
		let res = frame_support::storage::migration::get_storage_value::<AccountId>(
			b"CrowdloanContributors",
			b"RegistrarAccount",
			b"",
		)
		.map(|addr| addr != Default::default());
		if let Some(true) = res {
			Ok(())
		} else {
			Err("CrowdloanContributors not present")
		}
	}

	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		let entries = 4;
		frame_support::storage::unhashed::kill_prefix(&Twox128::hash(b"CrowdloanContributors"), Some(entries));

		<Runtime as frame_system::Config>::DbWeight::get().writes(entries.into())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		use sp_io::KillStorageResult;

		log::info!("Post check CrowdloanContributors-Removal.");
		let res = frame_support::storage::unhashed::kill_prefix(&Twox128::hash(b"CrowdloanContributors"), Some(0));

		match res {
			KillStorageResult::AllRemoved(0) | KillStorageResult::SomeRemaining(0) => Ok(()),
			KillStorageResult::AllRemoved(n) | KillStorageResult::SomeRemaining(n) => {
				log::error!("Remaining entries: {:?}", n);
				Err("CrowdloanContributors not removed")
			}
		}
	}
}
