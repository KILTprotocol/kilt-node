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
	deprecated::v1_v4::{
		storage::{
			CandidatePool as CandidatePoolV4, CollatorState as CollatorStateV4, DelegatorState as DelegatorStateV4,
			SelectedCandidates as SelectedCandidatesV4,
		},
		CollatorOf as CollatorOfV4, Delegator as DelegatorV4, OrderedSet as OrderedSetV4,
	},
	migrations::StakingStorageVersion,
	pallet::*,
	set::OrderedSet,
	types::{BalanceOf, Collator, Delegator, StakeOf},
	CandidatePool, SelectedCandidates,
};
use frame_support::{dispatch::Weight, traits::Get, BoundedVec, StoragePrefixedMap, StorageValue};
use sp_runtime::{traits::Zero, SaturatedConversion};
use sp_std::convert::TryFrom;

#[cfg(feature = "try-runtime")]
pub(crate) fn pre_migrate<T: Config>() -> Result<(), &'static str> {
	assert_eq!(StorageVersion::<T>::get(), StakingStorageVersion::V4);

	// candidate pool
	let candidates = check_migrate_candidate_pool::<T>();
	assert!(candidates.is_ok());
	assert!(!candidates.unwrap().len().is_zero());

	// selected candidates
	let collators = check_migrate_selected_candidates::<T>();
	assert!(collators.is_ok());
	assert!(!collators.unwrap().len().is_zero());

	// collator state
	assert!(CollatorStateV4::<T>::iter_values().all(|old: CollatorOfV4<T>| {
		migrate_ordered_set::<T, <T as Config>::MaxDelegatorsPerCollator>(old.delegators).is_ok()
	}));

	// delegator state
	assert!(
		DelegatorStateV4::<T>::iter_values().all(|old: DelegatorV4<T::AccountId, BalanceOf<T>>| {
			migrate_ordered_set::<T, <T as Config>::MaxCollatorsPerDelegator>(old.delegations).is_ok()
		})
	);
	Ok(())
}

// TODO: Docs
pub(crate) fn migrate_ordered_set<T: Config, S: Get<u32>>(
	old_set: OrderedSetV4<StakeOf<T>>,
) -> Result<OrderedSet<StakeOf<T>, S>, ()> {
	let bv = BoundedVec::<StakeOf<T>, S>::try_from(old_set.into_vec())?;
	Ok(OrderedSet::<StakeOf<T>, S>::from(bv))
}

// TODO: Docs
fn check_migrate_candidate_pool<T: Config>() -> Result<OrderedSet<StakeOf<T>, <T as Config>::MaxCollatorCandidates>, ()>
{
	// if runtime testing, check whether inner vector has been untouched
	#[cfg(feature = "try-runtime")]
	{
		assert_eq!(
			CandidatePoolV4::<T>::get().into_vec(),
			migrate_ordered_set::<T, <T as Config>::MaxCollatorCandidates>(CandidatePoolV4::<T>::get())?
				.into_bounded_vec()
				.into_inner()
		);
	}
	let candidates = migrate_ordered_set::<T, <T as Config>::MaxCollatorCandidates>(CandidatePoolV4::<T>::get())?;
	Ok(candidates)
}

// TODO: Docs
fn check_migrate_selected_candidates<T: Config>(
) -> Result<BoundedVec<T::AccountId, <T as Config>::MaxCollatorCandidates>, ()> {
	let candidates =
		BoundedVec::<T::AccountId, <T as Config>::MaxCollatorCandidates>::try_from(SelectedCandidatesV4::<T>::get())?;
	// if runtime testing, check whether inner vector has been untouched
	#[cfg(feature = "try-runtime")]
	{
		assert_eq!(SelectedCandidatesV4::<T>::get(), candidates.clone().into_inner());
	}
	Ok(candidates)
}

// TODO: Docs
fn migrate_collator_state<T: Config>() -> u64 {
	CollatorState::<T>::translate_values(|old: Option<CollatorOfV4<T>>| {
		old.map(
			|CollatorOfV4::<T> {
			     id,
			     stake,
			     total,
			     state,
			     delegators,
			 }| Collator::<T::AccountId, BalanceOf<T>, <T as Config>::MaxDelegatorsPerCollator> {
				id,
				stake,
				total,
				state,
				delegators: migrate_ordered_set::<T, <T as Config>::MaxDelegatorsPerCollator>(delegators)
					.expect("Exceeding MaxDelegatorsPerCollator bound has been checked in V4 already!"),
			},
		)
	});
	CollatorState::<T>::iter().count().saturated_into()
}

// TODO: Docs
fn migrate_delegator_state<T: Config>() -> u64 {
	DelegatorState::<T>::translate_values(|old: Option<DelegatorV4<T::AccountId, BalanceOf<T>>>| {
		old.map(
			|o| Delegator::<T::AccountId, BalanceOf<T>, <T as Config>::MaxCollatorsPerDelegator> {
				total: o.total,
				delegations: migrate_ordered_set::<T, <T as Config>::MaxCollatorsPerDelegator>(o.delegations)
					.expect("Exceeding MaxCollatorsPerDelegator bound has been checked in V4 already!"),
			},
		)
	});
	DelegatorState::<T>::iter().count().saturated_into()
}

pub(crate) fn migrate<T: Config>() -> Weight {
	log::info!("Migrating staking to StakingStorageVersion::V5");

	// migrate candidates
	CandidatePool::<T>::put(check_migrate_candidate_pool::<T>().expect("Should have thrown in pre_migrate"));
	SelectedCandidates::<T>::put(check_migrate_selected_candidates::<T>().expect("Should have thrown in pre_migrate"));

	// migrate collator state
	let num_collator_states = migrate_collator_state::<T>();

	// migrate delegator state
	let num_delegator_states = migrate_delegator_state::<T>();

	StorageVersion::<T>::put(StakingStorageVersion::V5);
	log::info!("Completed staking migration to StakingStorageVersion::V5");

	T::DbWeight::get().reads_writes(
		2u64.saturating_add(num_collator_states)
			.saturating_add(num_delegator_states),
		3u64.saturating_add(num_collator_states)
			.saturating_add(num_delegator_states),
	)
}

#[cfg(feature = "try-runtime")]
pub(crate) fn post_migrate<T: Config>() -> Result<(), &'static str> {
	assert!(!CandidatePool::<T>::get().len().is_zero());
	assert!(!SelectedCandidates::<T>::get().len().is_zero());
	assert_eq!(StorageVersion::<T>::get(), StakingStorageVersion::V5);
	log::info!("Staking storage version migrated from v4 to v5");
	Ok(())
}

// Tests for the v1 storage migrator.
#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		mock::{ExtBuilder, Test},
		types::Stake,
	};

	#[test]
	fn fail_version_higher() {
		ExtBuilder::default()
			.with_balances(vec![(1, 100)])
			.with_collators(vec![(1, 100)])
			.with_storage_version(StakingStorageVersion::V5)
			.build()
			.execute_with(|| {
				#[cfg(feature = "try-runtime")]
				assert!(pre_migrate::<Test>().is_err(), "Pre-migration for v5 should fail.");
			});
	}

	#[test]
	fn v5_migration_works() {
		ExtBuilder::default()
			.with_balances(vec![
				(1, 100),
				(2, 100),
				(3, 100),
				(4, 100),
				(5, 100),
				(6, 100),
				(7, 100),
			])
			.with_collators(vec![(1, 100), (2, 100), (3, 100)])
			.with_delegators(vec![(4, 1, 100), (5, 1, 100), (6, 1, 100), (7, 2, 100)])
			.with_storage_version(StakingStorageVersion::V4)
			.build()
			.execute_with(|| {
				assert!(!CandidatePool::<Test>::get().is_empty());
				#[cfg(feature = "try-runtime")]
				assert!(pre_migrate::<Test>().is_ok(), "Pre-migration for v5 should not fail.");

				migrate::<Test>();

				#[cfg(feature = "try-runtime")]
				assert!(post_migrate::<Test>().is_ok(), "Post-migration for v5 should not fail.");

				// FIXME: Fails because the mock already sets up with the new
				// types. Would need to add a mock just for this migration.
				// Seems overkill. assert_eq!(
				// 	CandidatePool::<Test>::get(),
				// 	OrderedSet::<StakeOf<Test>, <Test as
				// Config>::MaxCollatorCandidates>::from_sorted_set( 		BoundedVec:
				// :try_from(vec![ 			Stake { owner: 1, amount: 400 },
				// 			Stake { owner: 2, amount: 200 },
				// 			Stake { owner: 3, amount: 100 },
				// 		])
				// 		.unwrap()
				// 	)
				// );
			});
	}
}
