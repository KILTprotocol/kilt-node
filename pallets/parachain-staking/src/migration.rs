// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2022 BOTLabs GmbH

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
	set::OrderedSet,
	types::{BalanceOf, Delegator, Stake},
};

use super::*;
use codec::{Decode, Encode, MaxEncodedLen};
use core::marker::PhantomData;
use frame_support::{
	dispatch::GetStorageVersion,
	pallet_prelude::{StorageVersion, ValueQuery},
	parameter_types, storage_alias,
	traits::{Get, OnRuntimeUpgrade},
	weights::Weight,
	Blake2_128Concat, RuntimeDebug,
};
use scale_info::TypeInfo;
use sp_runtime::traits::Zero;

#[cfg(feature = "try-runtime")]
use sp_runtime::SaturatedConversion;

// Old delegator type needed for translating storage map
#[derive(Encode, Decode, Eq, MaxEncodedLen, PartialEq, RuntimeDebug, TypeInfo)]
#[scale_info(skip_type_params(MaxCollatorsPerDelegator))]
#[codec(mel_bound(AccountId: MaxEncodedLen, Balance: MaxEncodedLen))]
pub struct DelegatorOld<AccountId: Eq + Ord, Balance: Eq + Ord, MaxCollatorsPerDelegator: Get<u32>> {
	pub delegations: OrderedSet<Stake<AccountId, Balance>, MaxCollatorsPerDelegator>,
	pub total: Balance,
}
parameter_types! {
	const MaxCollatorsPerDelegator: u32 = 1;
}
// Old delegator state map required for pre checks
#[storage_alias]
type DelegatorStateOld<T: Config> = StorageMap<
	Pallet<T>,
	Blake2_128Concat,
	<T as frame_system::Config>::AccountId,
	DelegatorOld<<T as frame_system::Config>::AccountId, BalanceOf<T>, MaxCollatorsPerDelegator>,
>;

/// Number of delegators post migration
#[storage_alias]
type CounterForDelegators<T: Config> = StorageValue<Pallet<T>, u32, ValueQuery>;

pub struct StakingPayoutRefactor<T>(PhantomData<T>);
impl<T: Config> OnRuntimeUpgrade for StakingPayoutRefactor<T> {
	fn on_runtime_upgrade() -> Weight {
		let current = Pallet::<T>::current_storage_version();
		let onchain = Pallet::<T>::on_chain_storage_version();

		log::info!(
			"💰 Running migration with current storage version {:?} / onchain {:?}",
			current,
			onchain
		);

		if current == 8 && onchain == 7 {
			let num_delegators = migrate_delegators::<T>();
			log::info!("💰 Migrated {:?} delegator states", num_delegators);
			StorageVersion::new(8).put::<Pallet<T>>();
			T::DbWeight::get().reads_writes(num_delegators, num_delegators)
		} else {
			log::info!("💰 StakingPayoutRefactor did not execute. This probably should be removed");
			T::DbWeight::get().reads(1)
		}
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		let current = Pallet::<T>::on_chain_storage_version();

		assert_eq!(
			current, 7,
			"ParachainStaking on-chain StorageVersion is {:?} instead of 7",
			current
		);

		// sanity check each old entry
		for delegator in DelegatorStateOld::<T>::iter_values() {
			assert!(
				delegator.delegations.is_empty(),
				"There exists a delegator without a collator in pre migration!"
			);
			assert!(
				!delegator.total.is_zero(),
				"There exists a delegator without any self stake in pre migration!",
			)
		}
		log::info!(
			"💰 Staking migration pre check: {:?} delegators",
			DelegatorStateOld::<T>::iter().count()
		);

		assert!(
			CounterForDelegators::<T>::get().is_zero(),
			"CounterForDelegators already set."
		);
		// store number of delegators before migration to check against in post
		// migration
		CounterForDelegators::<T>::put(DelegatorState::<T>::iter_keys().count().saturated_into::<u32>());

		Ok(())
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade() -> Result<(), &'static str> {
		// new version must be set.
		let onchain = Pallet::<T>::on_chain_storage_version();

		assert_eq!(
			onchain, 8,
			"ParachainStaking StorageVersion post-migration is not 8, but {:?} instead.",
			onchain
		);

		let old_num_delegators: u32 = CounterForDelegators::<T>::get();
		let new_num_delegators: u32 = DelegatorState::<T>::iter_keys().count().saturated_into::<u32>();
		assert_eq!(
			old_num_delegators, new_num_delegators,
			"Number of delegators changed during migration! Before {:?} vs. now {:?}",
			old_num_delegators, new_num_delegators
		);
		log::info!(
			"💰 Number of delegators: Before {:?} vs. after {:?}",
			old_num_delegators,
			new_num_delegators
		);

		// sanity check each new entry
		for delegator in DelegatorState::<T>::iter_values() {
			assert!(
				!delegator.amount.is_zero(),
				"There exists a delegator without any self stake in post migration!",
			)
		}

		log::info!("💰 Post staking payout refactor upgrade checks match up.");
		Ok(())
	}
}

/// Translate all values from the DelegatorState StorageMap from old to new
fn migrate_delegators<T: Config>() -> u64 {
	let mut num_translations = 0;
	DelegatorState::<T>::translate_values(
		|old: DelegatorOld<T::AccountId, BalanceOf<T>, MaxCollatorsPerDelegator>| {
			num_translations += 1;

			// Should never occur because of pre checks but let's be save
			if old.total.is_zero() {
				log::debug!("Translating delegator with 0 stake amount")
			}
			if old.delegations.get(0).is_none() {
				log::debug!("Translating delegator without collator")
			}

			old.delegations.get(0).map(|stake| Delegator {
				amount: old.total,
				owner: stake.owner.clone(),
			})
		},
	);

	num_translations
}
