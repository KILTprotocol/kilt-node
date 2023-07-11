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
use log;
use sp_runtime::SaturatedConversion;
use sp_std::marker::PhantomData;

use crate::{
	AccountIdOf, Config, CurrencyOf, DelegationNodes, HoldReason, Pallet, STORAGE_VERSION as TARGET_STORAGE_VERSION,
};

const CURRENT_STORAGE_VERSION: StorageVersion = StorageVersion::new(3);

pub struct BalanceMigration<T>(PhantomData<T>);

impl<T: Config> OnRuntimeUpgrade for BalanceMigration<T>
where
	<T as Config>::Currency: ReservableCurrency<T::AccountId>,
{
	fn on_runtime_upgrade() -> frame_support::weights::Weight {
		log::info!("Delegation: Initiating migration");

		let onchain_storage_version = Pallet::<T>::on_chain_storage_version();
		if onchain_storage_version == CURRENT_STORAGE_VERSION {
			TARGET_STORAGE_VERSION.put::<Pallet<T>>();
			<T as frame_system::Config>::DbWeight::get()
				.reads_writes(1, 1)
				.saturating_add(do_migration::<T>())
		} else {
			log::info!(
			"Delegation: No migration needed. This file should be deleted. Current storage version: {:?}, Required Version for update: {:?}", 
			onchain_storage_version,
			CURRENT_STORAGE_VERSION
		);

			<T as frame_system::Config>::DbWeight::get().reads(1)
		}
	}

	#[cfg(feature = "try-runtime")]
	fn pre_upgrade() -> Result<sp_std::vec::Vec<u8>, &'static str> {
		use sp_std::vec;

		let has_all_user_no_holds = DelegationNodes::<T>::iter_values()
			.map(|details: crate::DelegationNodeOf<T>| {
				kilt_support::migration::has_user_reserved_balance::<AccountIdOf<T>, CurrencyOf<T>>(
					&details.deposit.owner,
					&HoldReason::Deposit.into(),
				)
			})
			.all(|user| user);

		assert!(
			has_all_user_no_holds,
			"Pre Upgrade Delegation: there are users with holds!"
		);

		assert_eq!(Pallet::<T>::on_chain_storage_version(), CURRENT_STORAGE_VERSION);
		log::info!("Delegation: Pre migration checks successful");

		Ok(vec![])
	}

	#[cfg(feature = "try-runtime")]
	fn post_upgrade(_pre_state: sp_std::vec::Vec<u8>) -> Result<(), &'static str> {
		use frame_support::traits::fungible::InspectHold;
		use sp_runtime::Saturating;
		use sp_std::collections::btree_map::BTreeMap;

		use crate::BalanceOf;

		let mut map_user_deposit: BTreeMap<AccountIdOf<T>, BalanceOf<T>> = BTreeMap::new();

		DelegationNodes::<T>::iter_values().for_each(|details| {
			map_user_deposit
				.entry(details.deposit.owner)
				.and_modify(|balance| *balance = balance.saturating_add(details.deposit.amount))
				.or_insert(details.deposit.amount);
		});

		map_user_deposit
			.iter()
			.try_for_each(|(who, amount)| -> Result<(), &'static str> {
				let hold_balance: BalanceOf<T> =
					<T as Config>::Currency::balance_on_hold(&HoldReason::Deposit.into(), who).saturated_into();

				assert!(
					amount.eq(&hold_balance),
					"Delegation: Hold balance is not matching for attestation {:?}. Expected hold: {:?}. Real hold: {:?}",
					who,
					amount,
					hold_balance
				);
				Ok(())
			})?;

		assert_eq!(Pallet::<T>::on_chain_storage_version(), TARGET_STORAGE_VERSION);

		log::info!("Delegation: Post migration checks successful");
		Ok(())
	}
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
				&HoldReason::Deposit.into(),
				deposit.amount.saturated_into(),
			);

			if error.is_err() {
				log::error!(
					" Delegation: Could not convert reserves to hold from delegation: {:?}, error: {:?}",
					key,
					error
				);
			}

			// Currency::reserve and Currency::hold each read and write to the DB once.
			// Since we are uncertain about which operation may fail, in the event of an
			// error, we assume the worst-case scenario here.
			<T as frame_system::Config>::DbWeight::get().reads_writes(2, 2)
		})
		.fold(Weight::zero(), |acc, next| acc.saturating_add(next))
}
