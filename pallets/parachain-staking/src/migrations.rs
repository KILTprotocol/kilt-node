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
	traits::{Get, OnRuntimeUpgrade, ReservableCurrency},
	weights::Weight,
};
use kilt_support::{deposit::HFIdentifier, migration::switch_locks_to_freeze};
use pallet_balances::{BalanceLock, Locks};
use sp_runtime::SaturatedConversion;
use sp_std::marker::PhantomData;

use crate::{
	types::{AccountIdOf, CurrencyOf},
	Config, STAKING_ID,
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
		// use frame_support::ensure;
		use sp_std::vec;
		// // before the upgrade, there should be no account with holds
		// ensure!(is_upgraded::<T>(), "Pre upgrade: there are users with holds.");
		// log::info!("Staking: Starting pre migration checks!");

		Ok(vec![])
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_pre_state: sp_std::vec::Vec<u8>) -> Result<(), &'static str> {
		// use frame_support::ensure;

		// // before the upgrade, there should be no account with holds
		// ensure!(!is_upgraded::<T>(), "Post upgrade: there are users with reserves.");
		// log::info!("Staking: Post migration checks succeded!");

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
		.any(|is_staking| !is_staking)
}

fn do_migration<T: Config>() -> Weight
where
	<T as Config>::Currency: ReservableCurrency<T::AccountId>,
{
	Locks::<T>::iter()
		.map(|(user_id, locks)| {
			locks
				.iter()
				.map(|lock: &BalanceLock<_>| -> Weight {
					if lock.id == STAKING_ID {
						let error = switch_locks_to_freeze::<AccountIdOf<T>, CurrencyOf<T>>(
							user_id.clone(),
							STAKING_ID,
							&HFIdentifier::Staking,
							lock.amount.saturated_into(),
						);

						if error.is_ok() {
							return <T as frame_system::Config>::DbWeight::get().reads_writes(1, 1);
						}
					}
					<T as frame_system::Config>::DbWeight::get().reads_writes(0, 0)
				})
				.fold(Weight::zero(), |acc, next| acc.saturating_add(next))
		})
		.fold(Weight::zero(), |acc, next| acc.saturating_add(next))
}
