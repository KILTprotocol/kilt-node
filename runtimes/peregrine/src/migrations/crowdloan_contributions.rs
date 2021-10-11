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

use frame_support::traits::OnRuntimeUpgrade;
use crate::Runtime;
use kilt_primitives::AccountId;

pub struct CrowdloanContributionsMigration;

impl OnRuntimeUpgrade for CrowdloanContributionsMigration {

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		assert_eq!(
			kilt_crowdloan::AdminAccount::<Runtime>::get(),
			AccountId::default(),
			"Admin account for crowdloan pallet is not the default one."
		);
		Ok(())
	}

    fn on_runtime_upgrade() -> frame_support::weights::Weight {
		kilt_crowdloan::AdminAccount::<Runtime>::set(AccountId::default());
		0u64.into()
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		assert_ne!(
			kilt_crowdloan::AdminAccount::<Runtime>::get(),
			AccountId::default(),
			"Admin account for crowdloan pallet is the default one."
		);
		Ok(())
	}
}
