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
	traits::{fungible::freeze::Mutate as MutateFreeze, Get, LockableCurrency, OnRuntimeUpgrade, ReservableCurrency},
	weights::Weight,
};
use pallet_balances::{BalanceLock, Freezes, IdAmount, Locks};
use sp_runtime::SaturatedConversion;
use sp_std::marker::PhantomData;

use crate::{
	types::{AccountIdOf, CurrencyOf},
	Config, FreezeReason, STAKING_ID,
};

pub struct BalanceMigration<T>(PhantomData<T>);

impl<T: crate::pallet::Config> OnRuntimeUpgrade for BalanceMigration<T>
where
	<T as Config>::Currency: ReservableCurrency<T::AccountId>,
{
	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		log::info!("Staking: Initiating migration");
		if is_upgraded::<T>() {
			return do_migration::<T>();
		}

		log::info!("Staking: No migration needed. This file should be deleted.");
		<T as frame_system::Config>::DbWeight::get().reads_writes(0, 0)
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<sp_std::vec::Vec<u8>, &'static str> {
		use frame_support::ensure;
		use sp_std::vec;
		let count_freezes = pallet_balances::Freezes::<T>::iter().count();
		ensure!(count_freezes == 0, "Staking Pre: There are already freezes.");

		log::info!("Staking: Pre migration checks successful");

		Ok(vec![])
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_pre_state: sp_std::vec::Vec<u8>) -> Result<(), &'static str> {
		use frame_support::ensure;

		let count_freezes = pallet_balances::Freezes::<T>::iter().count();

		ensure!(count_freezes > 0, "Staking: There are still no freezes.");

		log::info!("Staking: Post migration checks successful");

		Ok(())
	}
}

/// If there exists one user with locks -> the migration has to be executed.
fn is_upgraded<T: Config>() -> bool
where
	<T as Config>::Currency: ReservableCurrency<T::AccountId>,
{
	Locks::<T>::iter_values()
		.flatten()
		.map(|lock: BalanceLock<_>| lock.id == STAKING_ID)
		.any(|is_staking| is_staking)
}

fn do_migration<T: Config>() -> Weight
where
	<T as Config>::Currency: ReservableCurrency<T::AccountId>,
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
) -> Weight {
	let freezes = Freezes::<T>::get(&user_id);

	let result = if let Some(IdAmount { amount, .. }) = freezes
		.iter()
		.find(|freeze| freeze.id == <T as crate::Config>::FreezeIdentifier::from(FreezeReason::Staking))
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
		return <T as frame_system::Config>::DbWeight::get().reads_writes(0, 0);
	}
	<T as frame_system::Config>::DbWeight::get().reads_writes(1, 1)
}
