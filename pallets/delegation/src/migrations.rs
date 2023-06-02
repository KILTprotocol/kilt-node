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
use kilt_support::{
	deposit::{HFIdentifier, Pallets},
	migration::{has_user_holds, switch_reserved_to_hold},
};
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
		if is_upgraded::<T>() {
			return do_migration::<T>();
		}

		log::info!("Delegation: No migration needed. This file should be deleted.");
		<T as frame_system::Config>::DbWeight::get().reads_writes(0, 0)
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<sp_std::vec::Vec<u8>, &'static str> {
		use frame_support::ensure;
		use sp_std::vec;

		let has_all_user_no_holds = DelegationNodes::<T>::iter_values()
			.map(|details: DelegationNode<T>| {
				has_user_holds::<AccountIdOf<T>, CurrencyOf<T>>(
					&details.deposit.owner,
					&HFIdentifier::Deposit(Pallets::Delegation),
				)
			})
			.all(|user| user);

		ensure!(
			has_all_user_no_holds,
			"Pre Upgrade Delegation: there are users with holds!"
		);

		log::info!("Delegation: Pre migration checks successful");

		Ok(vec![])
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_pre_state: sp_std::vec::Vec<u8>) -> Result<(), &'static str> {
		use frame_support::{ensure, traits::fungible::InspectHold};
		use kilt_support::test_utils::log_and_return_error_message;

		DelegationNodes::<T>::iter().try_for_each(|(key, details)| -> Result<(), &'static str> {
			let hold_balance: u128 = <T as Config>::Currency::balance_on_hold(
				&HFIdentifier::Deposit(Pallets::Delegation),
				&details.deposit.owner,
			)
			.saturated_into();
			ensure!(
				details.deposit.amount.saturated_into::<u128>() <= hold_balance,
				log_and_return_error_message(scale_info::prelude::format!(
					"Delegation: Hold balance is not matching for delegation node {:?}. Expected hold: {:?}. Real hold: {:?}",
					key, details.deposit.amount, hold_balance
				))
			);

			ensure!(!is_upgraded::<T>(), "Delegation: Users have still no holds");

			Ok(())
		})?;
		log::info!("Delegation: Post migration checks successful");
		Ok(())
	}
}

/// Checks if there is an user, who has still reserved balance and no holds. If
/// yes, the migration is not executed yet.
fn is_upgraded<T: Config>() -> bool
where
	<T as Config>::Currency: ReservableCurrency<T::AccountId>,
{
	DelegationNodes::<T>::iter_values()
		.map(|details: DelegationNode<T>| {
			has_user_holds::<AccountIdOf<T>, CurrencyOf<T>>(
				&details.deposit.owner,
				&HFIdentifier::Deposit(Pallets::Delegation),
			)
		})
		.all(|user| user)
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
				&HFIdentifier::Deposit(Pallets::Delegation),
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
