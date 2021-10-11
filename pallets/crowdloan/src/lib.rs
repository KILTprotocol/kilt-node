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

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod tests;

#[cfg(test)]
pub(crate) mod mock;

pub use crate::pallet::*;

#[frame_support::pallet]
pub mod pallet {

	use frame_support::{
		pallet_prelude::*,
		traits::{Currency, StorageVersion},
	};
	use frame_system::{pallet_prelude::*, WeightInfo};
	use sp_runtime::traits::{BadOrigin, StaticLookup};

	pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	pub type BalanceOf<T> = <<T as Config>::Currency as Currency<AccountIdOf<T>>>::Balance;

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Currency: Currency<AccountIdOf<Self>>;
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type WeightInfo: WeightInfo;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T>(_);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub admin_account: AccountIdOf<T>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			Self {
				admin_account: AccountIdOf::<T>::default(),
			}
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			AdminAccount::<T>::set(self.admin_account.clone());
		}
	}

	#[pallet::storage]
	#[pallet::getter(fn admin_account)]
	pub type AdminAccount<T> = StorageValue<_, AccountIdOf<T>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn contributions)]
	pub type Contributions<T> = StorageMap<_, Blake2_128Concat, AccountIdOf<T>, BalanceOf<T>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		NewAdminAccountSet(AccountIdOf<T>, AccountIdOf<T>),
		NewContributionSet(AccountIdOf<T>, Option<BalanceOf<T>>, BalanceOf<T>),
		ContributionRemoved(AccountIdOf<T>),
	}

	#[pallet::error]
	pub enum Error<T> {
		ContributorNotPresent,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(1)]
		pub fn set_admin_account(origin: OriginFor<T>, new_account: <<T as frame_system::Config>::Lookup as StaticLookup>::Source) -> DispatchResult {
			let who = ensure_signed(origin)?;

			let old_account = AdminAccount::<T>::get();
			ensure!(who == old_account, BadOrigin);

			let looked_up_account = <T as frame_system::Config>::Lookup::lookup(new_account)?;
			AdminAccount::<T>::set(looked_up_account.clone());

			Self::deposit_event(Event::NewAdminAccountSet(old_account, looked_up_account));

			Ok(())
		}

		// Allow for overrides.
		#[pallet::weight(1)]
		pub fn set_new_contribution(
			origin: OriginFor<T>,
			contributor_account: <<T as frame_system::Config>::Lookup as StaticLookup>::Source,
			amount: BalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(who == AdminAccount::<T>::get(), BadOrigin);

			let looked_up_account = <T as frame_system::Config>::Lookup::lookup(contributor_account)?;
			let old_amount = Contributions::<T>::mutate(&looked_up_account, |entry| {
				let existing_entry = *entry;
				*entry = Some(amount);
				existing_entry
			});

			Self::deposit_event(Event::NewContributionSet(looked_up_account, old_amount, amount));

			Ok(())
		}

		#[pallet::weight(1)]
		pub fn remove_contribution(origin: OriginFor<T>, contributor_account: <<T as frame_system::Config>::Lookup as StaticLookup>::Source) -> DispatchResult {
			let who = ensure_signed(origin)?;
			ensure!(who == AdminAccount::<T>::get(), BadOrigin);

			let looked_up_account = <T as frame_system::Config>::Lookup::lookup(contributor_account)?;
			Contributions::<T>::take(&looked_up_account).ok_or(Error::<T>::ContributorNotPresent)?;

			Self::deposit_event(Event::ContributionRemoved(looked_up_account));

			Ok(())
		}
	}
}
