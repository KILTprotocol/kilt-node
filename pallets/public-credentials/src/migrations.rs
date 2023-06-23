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
	traits::{Get, GetStorageVersion, OnRuntimeUpgrade, ReservableCurrency, StorageVersion},
	weights::Weight,
};
use kilt_support::migration::switch_reserved_to_hold;
use sp_runtime::SaturatedConversion;
use sp_std::marker::PhantomData;

use crate::{AccountIdOf, Config, Credentials, CurrencyOf, HoldReason, Pallet};

pub struct BalanceMigration<T>(PhantomData<T>);

impl<T: crate::pallet::Config> OnRuntimeUpgrade for BalanceMigration<T>
where
	<T as Config>::Currency: ReservableCurrency<T::AccountId>,
{
	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		log::info!("Public Credentials: Initiating migration");

		let onchain_storage_version = Pallet::<T>::on_chain_storage_version();

		if onchain_storage_version.eq(&StorageVersion::new(1)) {
			StorageVersion::new(2).put::<Pallet<T>>();
			return do_migration::<T>();
		}

		log::info!(
			"Public Credential: No migration needed. This file should be deleted. Current storage version: {:?}, Required Version for update: {:?}", 
			onchain_storage_version,
			StorageVersion::new(1)
		);
		<T as frame_system::Config>::DbWeight::get().reads_writes(0, 0)
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<sp_std::vec::Vec<u8>, &'static str> {
		use sp_std::vec;

		let has_all_user_no_holds = Credentials::<T>::iter_values()
			.map(|details: crate::CredentialEntryOf<T>| {
				kilt_support::migration::has_user_holds::<AccountIdOf<T>, CurrencyOf<T>>(
					&details.deposit.owner,
					&T::RuntimeHoldReason::from(HoldReason::Deposit),
				)
			})
			.all(|user| user);

		assert!(
			has_all_user_no_holds,
			"Pre Upgrade Public Credentials: there are users with holds!"
		);

		assert_eq!(Pallet::<T>::on_chain_storage_version(), StorageVersion::new(1));

		log::info!("Public Credentials: Pre migration checks successful");

		Ok(vec![])
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_pre_state: sp_std::vec::Vec<u8>) -> Result<(), &'static str> {
		use frame_support::traits::fungible::InspectHold;

		Credentials::<T>::iter().try_for_each(|(key, key2, details)| -> Result<(), &'static str> {
			let hold_balance: u128 = <T as Config>::Currency::balance_on_hold(
				&T::RuntimeHoldReason::from(HoldReason::Deposit),
				&details.deposit.owner,
			)
			.saturated_into();
			assert!(
				details.deposit.amount.saturated_into::<u128>() <= hold_balance,
				"Public Credentails: Hold balance is not matching for credential {:?} {:?}. Expected hold: {:?}. Real hold: {:?}",
				key, key2, details.deposit.amount, hold_balance
			);

			Ok(())
		})?;

		assert_eq!(Pallet::<T>::on_chain_storage_version(), StorageVersion::new(2));

		log::info!("Public Credentials: Post migration checks successful");
		Ok(())
	}
}

fn do_migration<T: Config>() -> Weight
where
	<T as Config>::Currency: ReservableCurrency<T::AccountId>,
{
	Credentials::<T>::iter()
		.map(|(key, key2, pc_details)| -> Weight {
			let deposit = pc_details.deposit;
			let error = switch_reserved_to_hold::<AccountIdOf<T>, CurrencyOf<T>>(
				deposit.owner,
				&T::RuntimeHoldReason::from(HoldReason::Deposit),
				deposit.amount.saturated_into(),
			);

			if error.is_ok() {
				return <T as frame_system::Config>::DbWeight::get().reads_writes(1, 1);
			}

			log::error!(
				" Public Credential: Could not convert reserves to hold from credential: {:?} {:?}, error: {:?}",
				key,
				key2,
				error
			);

			<T as frame_system::Config>::DbWeight::get().reads_writes(0, 0)
		})
		.fold(Weight::zero(), |acc, next| acc.saturating_add(next))
}
