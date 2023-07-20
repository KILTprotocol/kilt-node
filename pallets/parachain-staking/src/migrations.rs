// KILT Blockchain – https://botlabs.org
// Copyright (C) 2019-2023 BOTLabs GmbH

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

use frame_support::{
	traits::{
		fungible::freeze::Mutate as MutateFreeze, Get, GetStorageVersion, LockIdentifier, LockableCurrency,
		OnRuntimeUpgrade, ReservableCurrency, StorageVersion,
	},
	weights::Weight,
};
use pallet_balances::{BalanceLock, Freezes, IdAmount, Locks};
use sp_runtime::SaturatedConversion;
use sp_std::marker::PhantomData;

use crate::{
	types::{AccountIdOf, CurrencyOf},
	Config, FreezeReason, Pallet, STORAGE_VERSION as TARGET_STORAGE_VERSION,
};

const STAKING_ID: LockIdentifier = *b"kiltpstk";

const CURRENT_STORAGE_VERSION: StorageVersion = StorageVersion::new(8);

pub struct BalanceMigration<T>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for BalanceMigration<T>
where
	<T as Config>::Currency: ReservableCurrency<T::AccountId>,
	<T as Config>::Currency: LockableCurrency<T::AccountId>,
{
	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		log::info!("Staking: Initiating migration");

		let onchain_storage_version = Pallet::<T>::on_chain_storage_version();
		if onchain_storage_version == CURRENT_STORAGE_VERSION {
			TARGET_STORAGE_VERSION.put::<Pallet<T>>();
			<T as frame_system::Config>::DbWeight::get()
				.reads_writes(1, 1)
				.saturating_add(do_migration::<T>())
		} else {
			log::info!(
				"Staking: No migration needed. This file should be deleted. Current storage version: {:?}, Required Version for update: {:?}", 
				onchain_storage_version,
				CURRENT_STORAGE_VERSION
			);
			<T as frame_system::Config>::DbWeight::get().reads(1)
		}
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<sp_std::vec::Vec<u8>, &'static str> {
		use sp_runtime::traits::Zero;
		use sp_std::vec;

		let count_freezes = pallet_balances::Freezes::<T>::iter().count();
		assert!(count_freezes.is_zero(), "Staking Pre: There are already freezes.");

		assert_eq!(Pallet::<T>::on_chain_storage_version(), CURRENT_STORAGE_VERSION);

		log::info!("Staking: Pre migration checks successful");

		Ok(vec![])
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_pre_state: sp_std::vec::Vec<u8>) -> Result<(), &'static str> {
		use sp_runtime::traits::Zero;

		let count_freezes = pallet_balances::Freezes::<T>::iter().count();

		assert!(!count_freezes.is_zero(), "Staking: There are still no freezes.");

		assert_eq!(Pallet::<T>::on_chain_storage_version(), TARGET_STORAGE_VERSION);

		log::info!("Staking: Post migration checks successful");

		Ok(())
	}
}

fn do_migration<T: Config>() -> Weight
where
	<T as Config>::Currency: ReservableCurrency<T::AccountId>,
	<T as Config>::Currency: LockableCurrency<T::AccountId>,
{
	Locks::<T>::iter()
		.map(|(user_id, locks)| {
			let weight = locks
				.iter()
				.map(|lock: &BalanceLock<_>| -> Weight {
					if lock.id == STAKING_ID {
						return update_or_create_freeze::<T>(user_id.clone(), lock);
					}
					<T as frame_system::Config>::DbWeight::get().reads_writes(0, 0)
				})
				.fold(Weight::zero(), |acc, next| acc.saturating_add(next));
			weight
		})
		.fold(Weight::zero(), |acc, next| acc.saturating_add(next))
}

fn update_or_create_freeze<T: Config>(
	user_id: T::AccountId,
	lock: &BalanceLock<<T as pallet_balances::Config>::Balance>,
) -> Weight
where
	CurrencyOf<T>: LockableCurrency<AccountIdOf<T>>,
{
	let freezes = Freezes::<T>::get(&user_id);

	let result = if let Some(IdAmount { amount, .. }) = freezes
		.iter()
		.find(|freeze| freeze.id == <T as Config>::FreezeIdentifier::from(FreezeReason::Staking).into())
	{
		let total_lock = lock
			.amount
			.saturated_into::<u128>()
			.saturating_add((*amount).saturated_into());

		<CurrencyOf<T> as MutateFreeze<AccountIdOf<T>>>::extend_freeze(
			&<T as crate::Config>::FreezeIdentifier::from(FreezeReason::Staking),
			&user_id,
			total_lock.saturated_into(),
		)
	} else {
		<CurrencyOf<T> as MutateFreeze<AccountIdOf<T>>>::set_freeze(
			&<T as crate::Config>::FreezeIdentifier::from(FreezeReason::Staking),
			&user_id,
			lock.amount.saturated_into(),
		)
	};

	<CurrencyOf<T> as LockableCurrency<AccountIdOf<T>>>::remove_lock(STAKING_ID, &user_id);

	if result.is_err() {
		return <T as frame_system::Config>::DbWeight::get().reads(1);
	}

	// Currency::reserve and Currency::hold each read and write to the DB once.
	// Since we are uncertain about which operation may fail, in the event of an
	// error, we assume the worst-case scenario here.
	<T as frame_system::Config>::DbWeight::get().reads_writes(2, 2)
}
