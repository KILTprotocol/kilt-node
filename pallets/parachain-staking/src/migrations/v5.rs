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

use crate::{
	migrations::StakingStorageVersion,
	set::OrderedSet,
	types::{BalanceOf, Candidate, Stake, TotalStake},
	CandidateCount, CandidatePool, Config, StorageVersion, TopCandidates, TotalCollatorStake,
};
use frame_support::{
	dispatch::Weight,
	storage::{IterableStorageMap, StorageValue},
	traits::Get,
};

#[cfg(feature = "try-runtime")]
pub(crate) fn pre_migrate<T: Config>() -> Result<(), &'static str> {
	assert_eq!(StorageVersion::<T>::get(), StakingStorageVersion::V4);
	Ok(())
}

pub(crate) fn migrate<T: Config>() -> Weight {
	log::info!("Migrating staking to StakingStorageVersion::V5");

	// Kill selected candidates list
	old::SelectedCandidates::<T>::kill();

	// migrate CandidatePool to TopCandidates
	// DANGER: this needs to happen before we write to the new CandidatePool!
	let old_candidate_pool = old::CandidatePool::<T>::get();
	TopCandidates::<T>::put(old_candidate_pool);
	old::CandidatePool::<T>::kill();

	// Copy over total
	let total = old::Total::<T>::get();
	TotalCollatorStake::<T>::set(total);
	old::Total::<T>::kill();

	// kill, copy & count candidates
	let counter: u32 = old::CollatorState::<T>::drain().fold(0, |acc, (key, candidate)| {
		CandidatePool::<T>::insert(&key, candidate);
		acc.saturating_add(1)
	});
	CandidateCount::<T>::put(counter);

	// update storage version
	StorageVersion::<T>::put(StakingStorageVersion::V5);
	log::info!("Completed staking migration to StakingStorageVersion::V5");

	// Writes: 3 * Kill, 3 * Put, `counter` * inserts, at max 32 additional kills <
	// counter + 38 Reads: (counter + 2) * get
	T::DbWeight::get().reads_writes(counter.saturating_add(38).into(), counter.saturating_add(2).into())
}

#[cfg(feature = "try-runtime")]
pub(crate) fn post_migrate<T: Config>() -> Result<(), &'static str> {
	use sp_runtime::SaturatedConversion;

	assert_eq!(StorageVersion::<T>::get(), StakingStorageVersion::V5);
	log::info!(
		"CandidateCount = {} >= {} = MinCollators",
		CandidateCount::<T>::get(),
		T::MinCollators::get()
	);
	assert!(CandidateCount::<T>::get() >= T::MinCollators::get());
	log::info!(
		"TopCandidates = {} >= {} = MinCollators",
		TopCandidates::<T>::get().len(),
		T::MinCollators::get()
	);
	assert!(TopCandidates::<T>::get().len().saturated_into::<u32>() >= T::MinCollators::get());
	assert!(TopCandidates::<T>::get().len().saturated_into::<u32>() <= CandidateCount::<T>::get());
	assert!(
		TotalCollatorStake::<T>::get().collators
			>= CandidateCount::<T>::get().saturated_into::<BalanceOf<T>>() * T::MinCollatorStake::get()
	);
	let counter: u32 = old::CollatorState::<T>::iter().fold(0, |acc, (_, _)| acc.saturating_add(1));
	assert!(counter == 0);
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
			pub(crate) Total: TotalStake<BalanceOf<T>>;
			pub(crate) SelectedCandidates: Vec<T::AccountId>;
			pub(crate) CollatorState: map hasher(twox_64_concat) T::AccountId => Option<Candidate<T::AccountId, BalanceOf<T>, T::MaxDelegatorsPerCollator>>;
			pub(crate) CandidatePool: OrderedSet<Stake<T::AccountId, BalanceOf<T>>, T::MaxTopCandidates>;
		}
	}
}
