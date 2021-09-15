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

use crate::{migrations::StakingStorageVersion, CandidateCount, CandidatePool, Config, StorageVersion};
use frame_support::{dispatch::Weight, storage::StorageValue, traits::Get};

#[cfg(feature = "try-runtime")]
pub(crate) fn pre_migrate<T: Config>() -> Result<(), &'static str> {
	assert_eq!(StorageVersion::<T>::get(), StakingStorageVersion::V4);
	Ok(())
}

pub(crate) fn migrate<T: Config>() -> Weight {
	log::info!("Migrating staking to StakingStorageVersion::V5");

	// Kill selected candidates list
	old::SelectedCandidates::<T>::kill();

	// count candidates
	let counter: u32 = CandidatePool::<T>::iter().fold(0, |acc, _| acc.saturating_add(1));
	CandidateCount::<T>::put(counter);

	// update storage version
	StorageVersion::<T>::put(StakingStorageVersion::V5);
	log::info!("Completed staking migration to StakingStorageVersion::V5");

	T::DbWeight::get().reads_writes(counter.saturating_add(2).into(), 3)
}

#[cfg(feature = "try-runtime")]
pub(crate) fn post_migrate<T: Config>() -> Result<(), &'static str> {
	assert_eq!(StorageVersion::<T>::get(), StakingStorageVersion::V5);
	assert!(CandidateCount::<T>::get() > T::MinCollators::get());
	Ok(())
}

pub(crate) mod old {
	use super::*;
	use frame_support::{decl_module, decl_storage};
	use sp_std::prelude::*;

	decl_module! {
		pub struct OldPallet<T: Config> for enum Call where origin: T::Origin {}
	}

	decl_storage! {
		trait Store for OldPallet<T: Config> as ParachainStaking {
			pub(crate) SelectedCandidates: Vec<T::AccountId>;
		}
	}
}
