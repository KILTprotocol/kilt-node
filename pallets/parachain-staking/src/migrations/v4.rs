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
	types::{CandidateOf, Delegator},
	CandidatePool, CandidateState, Config, DelegatorState, StorageVersion,
};
use frame_support::{dispatch::Weight, traits::Get};

#[cfg(feature = "try-runtime")]
pub(crate) fn pre_migrate<T: Config>() -> Result<(), &'static str> {
	assert_eq!(StorageVersion::<T>::get(), StakingStorageVersion::V3_0_0);
	Ok(())
}

pub(crate) fn migrate<T: Config>() -> Weight {
	log::info!("Migrating staking to StakingStorageVersion::V4");

	// sort candidates from greatest to lowest
	CandidatePool::<T>::mutate(|candidates| candidates.sort_greatest_to_lowest());
	let mut n = 1u64;

	// for each candidate: sort delegators from greatest to lowest
	CandidateState::<T>::translate_values(|mut state: CandidateOf<T, T::MaxDelegatorsPerCollator>| {
		state.delegators.sort_greatest_to_lowest();
		n = n.saturating_add(1u64);
		Some(state)
	});

	// for each delegator: sort delegations from greatest to lowest
	DelegatorState::<T>::translate_values(
		|mut state: Delegator<T::AccountId, T::CurrencyBalance, T::MaxCollatorsPerDelegator>| {
			state.delegations.sort_greatest_to_lowest();
			n = n.saturating_add(1u64);
			Some(state)
		},
	);

	StorageVersion::<T>::put(StakingStorageVersion::V4);
	log::info!("Completed staking migration to StakingStorageVersion::V4");

	T::DbWeight::get().reads_writes(n, n)
}

#[cfg(feature = "try-runtime")]
pub(crate) fn post_migrate<T: Config>() -> Result<(), &'static str> {
	let mut candidates = CandidatePool::<T>::get();
	candidates.sort_greatest_to_lowest();
	assert_eq!(
		CandidatePool::<T>::get().into_bounded_vec().into_inner(),
		candidates.into_bounded_vec().into_inner()
	);
	assert_eq!(StorageVersion::<T>::get(), StakingStorageVersion::V4);
	Ok(())
}
