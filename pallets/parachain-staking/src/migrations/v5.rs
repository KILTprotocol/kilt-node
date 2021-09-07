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

use crate::{migrations::StakingStorageVersion, CandidateCount, CandidatePool, Config, StorageVersion, TopCandidates};
use frame_support::{dispatch::Weight, storage::unhashed::kill_prefix, traits::Get};
pub use sp_runtime::traits::Zero;

#[cfg(feature = "try-runtime")]
pub(crate) fn pre_migrate<T: Config>() -> Result<(), &'static str> {
	assert_eq!(StorageVersion::<T>::get(), StakingStorageVersion::V4);
	Ok(())
}

pub(crate) fn migrate<T: Config>() -> Weight {
	log::info!("Migrating staking to StakingStorageVersion::V5");

	let candidate_count = CandidatePool::<T>::iter().fold(0u32, |acc, _| acc.saturating_add(1));
	CandidateCount::<T>::put(candidate_count);

	StorageVersion::<T>::put(StakingStorageVersion::V5);
	log::info!("Completed staking migration to StakingStorageVersion::V5");

	T::DbWeight::get().reads_writes(candidate_count.saturating_add(2).into(), 3)
}

#[cfg(feature = "try-runtime")]
pub(crate) fn post_migrate<T: Config>() -> Result<(), &'static str> {
	let mut candidates = TopCandidates::<T>::get();
	candidates.sort_greatest_to_lowest();
	assert_eq!(
		TopCandidates::<T>::get().into_bounded_vec().into_inner(),
		candidates.into_bounded_vec().into_inner()
	);
	assert_eq!(StorageVersion::<T>::get(), StakingStorageVersion::V5);
	Ok(())
}

pub(crate) mod storage {
	use frame_support::{decl_module, decl_storage};
	use sp_std::prelude::*;

	use super::*;

	decl_module! {
		pub struct OldPallet<T: Config> for enum Call where origin: T::Origin {}
	}

	decl_storage! {
		trait Store for OldPallet<T: Config> as ParachainStaking {
			pub(crate) CollatorState: map hasher(twox_64_concat) T::AccountId => Option<crate::types::Candidate<T::AccountId, crate::types::BalanceOf<T>, T::MaxDelegatorsPerCollator>>;
			pub(crate) SelectedCandidates: Vec<T::AccountId>;
			pub(crate) Total: crate::types::TotalStake<crate::types::BalanceOf<T>>;
			pub(crate) CandidatePool: Vec<crate::types::Stake<T::AccountId, crate::types::BalanceOf<T>>>;
		}
	}
}
