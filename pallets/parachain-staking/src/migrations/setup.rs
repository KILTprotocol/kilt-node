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

use crate::{migrations::StakingStorageVersion, Config, StorageVersion};
use frame_support::{traits::Get, weights::Weight};

#[cfg(feature = "try-runtime")]
use frame_support::ensure;

#[cfg(feature = "try-runtime")]
pub(crate) fn pre_migrate<T: Config>() -> Result<(), &'static str> {
	ensure!(
		StorageVersion::<T>::get() == StakingStorageVersion::None,
		"Current deployed version is not absent."
	);

	Ok(())
}

pub(crate) fn migrate<T: Config>() -> Weight {
	log::info!("Setting up staking storage version to latest declared.");

	StorageVersion::<T>::set(StakingStorageVersion::latest());
	T::DbWeight::get().writes(1)
}

#[cfg(feature = "try-runtime")]
pub(crate) fn post_migrate<T: Config>() -> Result<(), &'static str> {
	ensure!(
		StorageVersion::<T>::get() == StakingStorageVersion::latest(),
		"Current deployed version is not the latest after setup."
	);

	Ok(())
}
