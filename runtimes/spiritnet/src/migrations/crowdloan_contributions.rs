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
#[cfg(feature = "try-runtime")]
use frame_support::traits::GetStorageVersion;
use frame_support::traits::{OnRuntimeUpgrade, StorageVersion};
use hex_literal::hex;
use kilt_primitives::AccountId;

// Same as Spiritnet transfer account.
pub const NEW_ADMIN_ACCOUNT: [u8; 32] = hex!("de28ef5b1691663300a2edb97202791e89bb6985ffdaa4c405d68c826b634b76");

pub struct CrowdloanContributionsSetup;

impl OnRuntimeUpgrade for CrowdloanContributionsSetup {
	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		assert_eq!(
			crowdloan::Pallet::<Runtime>::on_chain_storage_version(),
			StorageVersion::default(),
			"On-chain storage version for crowdloan pallet pre-migration not the expected default."
		);
		log::info!("Setting up crowdloan contributions pallet.");
		Ok(())
	}

	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		let admin_account = AccountId::new(NEW_ADMIN_ACCOUNT);
		log::info!(
			"Setting crowdloan contributions pallet admin account to {:?}.",
			&admin_account
		);
		crowdloan::RegistrarAccount::<Runtime>::set(admin_account);
		StorageVersion::put::<crowdloan::Pallet<Runtime>>(&crowdloan::STORAGE_VERSION);

		<Runtime as frame_system::Config>::DbWeight::get().writes(2)
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		assert_eq!(
			crowdloan::Pallet::<Runtime>::on_chain_storage_version(),
			crowdloan::Pallet::<Runtime>::current_storage_version(),
			"On-chain storage version for crowdloan pallet post-migration not the latest."
		);
		assert_eq!(
			crowdloan::RegistrarAccount::<Runtime>::get(),
			AccountId::new(NEW_ADMIN_ACCOUNT),
			"Admin account set different than the expected one."
		);
		log::info!("Crowdloan contributions pallet set up.");
		Ok(())
	}
}
