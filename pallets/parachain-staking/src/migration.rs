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
use core::marker::PhantomData;
use frame_support::{
	dispatch::GetStorageVersion,
	pallet_prelude::ValueQuery,
	parameter_types, storage_alias,
	traits::{Get, OnRuntimeUpgrade},
	weights::Weight,
	RuntimeDebug,
};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

#[cfg(feature = "try-runtime")]
use sp_runtime::SaturatedConversion;

// Old types
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

/// Number of delegators post migration
#[storage_alias]
type CounterForDelegators<T: Config> = StorageValue<Pallet<T>, u32, ValueQuery>;

pub struct StakingPayoutRefactor<T>(PhantomData<T>);
impl<T: Config> OnRuntimeUpgrade for StakingPayoutRefactor<T> {
	fn on_runtime_upgrade() -> Weight {
		let current = Pallet::<T>::current_storage_version();
		let onchain = Pallet::<T>::on_chain_storage_version();

		log::info!(
			"Running migration with current storage version {:?} / onchain {:?}",
			current,
			onchain
		);

		if current == 8 && onchain == 7 {
			let num_delegators = migrate_delegators::<T>();
			T::DbWeight::get().reads_writes(num_delegators, num_delegators)
		} else {
			log::info!("StakingPayoutRefactor did not execute. This probably should be removed");
			T::DbWeight::get().reads(1)
		}
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<(), &'static str> {
		use sp_runtime::traits::Zero;

		let current = Pallet::<T>::current_storage_version();

		assert_eq!(
			current, 7,
			"ParachainStaking StorageVersion is {:?} instead of 7",
			current
		);
		assert!(
			CounterForDelegators::<T>::get().is_zero(),
			"CounterForDelegators already set."
		);
		// store number of delegators before migration
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
		Ok(())
	}
}

fn migrate_delegators<T: Config>() -> u64 {
	let mut counter = 0;
	DelegatorState::<T>::translate_values::<
		Option<DelegatorOld<T::AccountId, BalanceOf<T>, MaxCollatorsPerDelegator>>,
		_,
	>(|maybe_old| {
		counter += 1;
		maybe_old
			.map(|old| {
				old.delegations.get(0).map(|stake| Delegator {
					amount: old.total,
					owner: stake.owner.clone(),
				})
			})
			.unwrap_or(None)
	});

	counter
}
