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

use frame_support::{dispatch::Weight, traits::Get};

use crate::{
	migrations::StakingStorageVersion, types::Delegator, CandidatePool, Config, DelegatorState, StorageVersion,
};

#[cfg(feature = "try-runtime")]
pub(crate) fn pre_migrate<T: Config>() -> Result<(), &'static str> {
	use crate::types::{BalanceOf, Stake};

	assert_eq!(StorageVersion::<T>::get(), StakingStorageVersion::V5);

	// ensure each delegator has at most one delegation (ideally should have exactly
	// one)
	let mut corrupt_delegation: u32 = 0;
	for candidate in CandidatePool::<T>::iter_values() {
		for delegation in candidate.delegators.into_iter() {
			if let Some(state) = DelegatorState::<T>::get(delegation.owner.clone()) {
				assert_eq!(state.delegations.len(), 1);
				if let Some(Stake::<T::AccountId, BalanceOf<T>> { amount, owner }) =
					state.delegations.into_bounded_vec().into_inner().get(0)
				{
					assert_eq!(amount, &state.total);
					assert_eq!(owner, &candidate.id);
				}
			} else {
				corrupt_delegation = corrupt_delegation.saturating_add(1);
			}
		}
	}

	log::info!("Found {} corrupt delegations", corrupt_delegation);
	Ok(())
}

pub(crate) fn migrate<T: Config>() -> Weight {
	log::info!("Migrating staking to StakingStorageVersion::V6");

	let mut reads = 0u64;
	let mut writes = 1u64;

	// iter candidate pool and check whether any delegator has a cleared state
	for candidate in CandidatePool::<T>::iter_values() {
		reads = reads.saturating_add(1u64);
		for delegation in candidate.delegators.into_iter() {
			// we do not have to mutate existing entries since MaxCollatorsPerDelegator = 1
			if !DelegatorState::<T>::contains_key(delegation.owner.clone()) {
				if let Ok(delegator) = Delegator::try_new(candidate.id.clone(), delegation.amount) {
					DelegatorState::<T>::insert(delegation.owner, delegator);
					writes = writes.saturating_add(1u64);
				}
			}
			reads = reads.saturating_add(1u64);
		}
	}

	// update storage version
	StorageVersion::<T>::put(StakingStorageVersion::V6);
	log::info!("Completed staking migration to StakingStorageVersion::V6");

	T::DbWeight::get().reads_writes(reads, writes)
}

#[cfg(feature = "try-runtime")]
pub(crate) fn post_migrate<T: Config>() -> Result<(), &'static str> {
	assert_eq!(StorageVersion::<T>::get(), StakingStorageVersion::V6);

	for candidate in CandidatePool::<T>::iter_values() {
		for delegation in candidate.delegators.into_iter() {
			assert!(DelegatorState::<T>::contains_key(delegation.owner));
		}
	}

	Ok(())
}
