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
use kilt_support::migration::{has_user_holds_and_no_reserves, switch_reserved_to_hold};
use log;
use sp_runtime::SaturatedConversion;
use sp_std::marker::PhantomData;

use crate::{AccountIdOf, Config, CurrencyOf, DelegationNode, DelegationNodes};

pub struct BalanceMigration<T>(PhantomData<T>);

impl<T: crate::pallet::Config> OnRuntimeUpgrade for BalanceMigration<T>
where
	<T as Config>::Currency: ReservableCurrency<T::AccountId>,
{
	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		log::info!("Delegation: Initiating migration");
		if ensure_upgraded::<T>() {
			return do_migration::<T>();
		}

		log::info!("Delegation: No migration needed. This file should be deleted.");
		<T as frame_system::Config>::DbWeight::get().reads_writes(0, 0)
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<sp_std::vec::Vec<u8>, &'static str> {
		use frame_support::ensure;
		use sp_std::vec;

		let has_one_user_holds = DelegationNodes::<T>::iter_values()
			.map(|details: DelegationNode<T>| {
				has_user_holds_and_no_reserves::<AccountIdOf<T>, CurrencyOf<T>>(&details.deposit.owner)
			})
			.all(|user| !user);

		// before the upgrade, there should be no account with holds
		ensure!(has_one_user_holds, "Pre upgrade: there are users with holds.");

		Ok(vec![])
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_pre_state: sp_std::vec::Vec<u8>) -> Result<(), &'static str> {
		use frame_support::ensure;

		let has_all_user_holds = DelegationNodes::<T>::iter_values()
			.map(|details: DelegationNode<T>| {
				has_user_holds_and_no_reserves::<AccountIdOf<T>, CurrencyOf<T>>(&details.deposit.owner)
			})
			.all(|user| user);

		// before the upgrade, there should be no account with holds
		ensure!(has_all_user_holds, "Post upgrade: there are user with reserves.");

		Ok(())
	}
}

/// Checks if there is an user, who has still reserved balance and no holds. If
/// yes, the migration is not executed yet.
fn ensure_upgraded<T: Config>() -> bool
where
	<T as Config>::Currency: ReservableCurrency<T::AccountId>,
{
	DelegationNodes::<T>::iter_values()
		.map(|details: DelegationNode<T>| {
			has_user_holds_and_no_reserves::<AccountIdOf<T>, CurrencyOf<T>>(&details.deposit.owner)
		})
		.any(|user| !user)
}

fn do_migration<T: Config>() -> Weight
where
	<T as Config>::Currency: ReservableCurrency<T::AccountId>,
{
	DelegationNodes::<T>::iter()
		.map(|(key, delegation_detail)| -> Weight {
			let deposit = delegation_detail.deposit;
			let error = switch_reserved_to_hold::<AccountIdOf<T>, CurrencyOf<T>>(
				deposit.owner,
				deposit.amount.saturated_into(),
			);

			if error.is_ok() {
				return <T as frame_system::Config>::DbWeight::get().reads_writes(1, 1);
			}

			log::error!(
				" Delegation: Could not convert reserves to hold from delegation: {:?} ",
				key
			);

			<T as frame_system::Config>::DbWeight::get().reads_writes(0, 0)
		})
		.fold(Weight::zero(), |acc, next| acc.saturating_add(next))
}
