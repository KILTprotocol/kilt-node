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

use crate::{inflation::InflationInfo, migrations::StakingStorageVersion, pallet::*};
use frame_support::{dispatch::Weight, traits::Get};
use kilt_primitives::constants::INFLATION_CONFIG;

#[cfg(feature = "try-runtime")]
pub(crate) fn pre_migrate<T: Config>() -> Result<(), &'static str> {
	assert_eq!(StorageVersion::<T>::get(), StakingStorageVersion::V2_0_0);
	Ok(())
}

pub(crate) fn migrate<T: Config>() -> Weight {
	log::info!("Migrating staking to StakingStorageVersion::V3_0_0");

	// update rewards per block
	InflationConfig::<T>::mutate(|inflation| *inflation = InflationInfo::from(INFLATION_CONFIG));

	StorageVersion::<T>::put(StakingStorageVersion::V3_0_0);
	log::info!("Completed staking migration to StakingStorageVersion::V3_0_0");

	T::DbWeight::get().reads_writes(1, 2)
}

#[cfg(feature = "try-runtime")]
pub(crate) fn post_migrate<T: Config>() -> Result<(), &'static str> {
	assert_eq!(InflationConfig::<T>::get(), InflationInfo::from(INFLATION_CONFIG));
	assert_eq!(StorageVersion::<T>::get(), StakingStorageVersion::V3_0_0);
	Ok(())
}
